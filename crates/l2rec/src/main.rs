//! `tq-l2rec`: record Binance public L2 data until killed, `--minutes` elapse, or `--max-mb` is hit.
//!
//! Public endpoints only: no API key is read and no order endpoint exists in this binary.

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use tq_feeds::binance::{TungsteniteWs, WsConnector};
use tq_l2rec::{event_line, msg_line, parse_symbol, snap_line, time_line, HourlyWriter, Market};

const SNAPSHOT_EVERY: Duration = Duration::from_secs(600);
const FLUSH_EVERY: Duration = Duration::from_secs(5);
const REST_TIMEOUT: Duration = Duration::from_secs(10);
const FIRST_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);
const BYTES_PER_MB: u64 = 1024 * 1024;

#[derive(Clone, Copy, ValueEnum)]
enum MarketArg {
    Spot,
    Um,
}

#[derive(Parser)]
#[command(about = "Record Binance PUBLIC order-book diffs, trades and top-of-book (no keys, no orders)")]
struct Args {
    #[arg(default_values_t = ["BTCUSDT".to_string(), "ETHUSDT".to_string()])]
    symbols: Vec<String>,
    #[arg(long, value_enum, default_value = "um")]
    market: MarketArg,
    #[arg(long, default_value = "data/l2")]
    out: PathBuf,
    /// Stop after this many wall-clock minutes (default: run until killed).
    #[arg(long)]
    minutes: Option<u64>,
    /// Stop when the market's output directory reaches this size.
    #[arg(long, default_value_t = 35_000)]
    max_mb: u64,
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

struct Recorder {
    market: Market,
    symbols: Vec<String>,
    writer: HourlyWriter,
    rest: ureq::Agent,
    deadline: Option<Instant>,
    max_bytes: u64,
}

enum Stop {
    Deadline,
    DiskCap,
}

impl Recorder {
    fn log(&mut self, event: &str, info: &str) -> Result<(), String> {
        eprintln!("{event}: {info}");
        let ts = now_ms();
        self.writer.write(ts, &event_line(ts, event, info)).map_err(|e| format!("write failed: {e}"))
    }

    fn server_time(&mut self) -> Result<(), String> {
        let tx = now_ms();
        match self.rest.get(self.market.time_url()).call() {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| e.to_string())?;
                let rx = now_ms();
                self.writer.write(rx, &time_line(tx, rx, body.trim())).map_err(|e| format!("write failed: {e}"))
            }
            Err(e) => self.log("servertime_failed", &e.to_string()),
        }
    }

    fn snapshots(&mut self) -> Result<(), String> {
        self.server_time()?;
        for sym in self.symbols.clone() {
            match self.rest.get(&self.market.snapshot_url(&sym)).call() {
                Ok(resp) => {
                    let body = resp.into_string().map_err(|e| e.to_string())?;
                    let ts = now_ms();
                    self.writer.write(ts, &snap_line(ts, &sym, body.trim())).map_err(|e| format!("write failed: {e}"))?;
                }
                // A missed snapshot only delays the offline book rebuild until the next one.
                Err(e) => self.log("snapshot_failed", &format!("{sym}: {e}"))?,
            }
        }
        Ok(())
    }

    fn stop_reason(&self) -> Option<Stop> {
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            Some(Stop::Deadline)
        } else if self.writer.bytes_on_disk() >= self.max_bytes {
            Some(Stop::DiskCap)
        } else {
            None
        }
    }

    /// One websocket session. `Ok(Some)` means stop for good; `Err` means reconnect.
    fn session(&mut self, url: &str) -> Result<Option<Stop>, String> {
        let mut conn = TungsteniteWs.connect(url).map_err(|e| e.to_string())?;
        self.log("connect", url)?;
        self.snapshots()?;
        let (mut last_snap, mut last_flush) = (Instant::now(), Instant::now());
        loop {
            if let Some(text) = conn.next_text().map_err(|e| e.to_string())? {
                let ts = now_ms();
                self.writer.write(ts, &msg_line(ts, &text)).map_err(|e| format!("write failed: {e}"))?;
            }
            if last_snap.elapsed() >= SNAPSHOT_EVERY {
                self.snapshots()?;
                last_snap = Instant::now();
            }
            if last_flush.elapsed() >= FLUSH_EVERY {
                self.writer.flush().map_err(|e| format!("flush failed: {e}"))?;
                last_flush = Instant::now();
                if let Some(stop) = self.stop_reason() {
                    return Ok(Some(stop));
                }
            }
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let symbols = args.symbols.iter().map(|s| parse_symbol(s)).collect::<Result<Vec<_>, _>>()?;
    let market = match args.market {
        MarketArg::Spot => Market::Spot,
        MarketArg::Um => Market::Um,
    };
    let dir = args.out.join(market.name());
    let writer = HourlyWriter::new(dir.clone()).map_err(|e| format!("cannot open {}: {e}", dir.display()))?;
    let mut rec = Recorder {
        market,
        symbols: symbols.clone(),
        writer,
        rest: ureq::AgentBuilder::new().timeout(REST_TIMEOUT).build(),
        deadline: args.minutes.map(|m| Instant::now() + Duration::from_secs(m * 60)),
        max_bytes: args.max_mb * BYTES_PER_MB,
    };
    let url = market.stream_url(&symbols);
    let mut backoff = FIRST_BACKOFF;
    let stop = loop {
        if let Some(stop) = rec.stop_reason() {
            break stop;
        }
        match rec.session(&url) {
            Ok(Some(stop)) => break stop,
            Ok(None) => {}
            Err(e) if e.starts_with("write failed") || e.starts_with("flush failed") => return Err(e),
            Err(e) => {
                rec.log("disconnect", &e)?;
                thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        }
        backoff = FIRST_BACKOFF;
    };
    let why = match stop {
        Stop::Deadline => "deadline reached",
        Stop::DiskCap => "disk cap reached",
    };
    rec.log("stop", why)?;
    rec.writer.finish().map_err(|e| format!("finish failed: {e}"))?;
    eprintln!("stopped: {why}; {} MB in {}", rec.writer.bytes_on_disk() / BYTES_PER_MB, dir.display());
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

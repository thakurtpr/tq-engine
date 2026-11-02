//! `tq-l2rec`: record Binance public L2 data until killed, `--minutes` elapse, or `--max-mb` is hit.
//!
//! Public endpoints only: no API key is read and no order endpoint exists in this binary.
//! One thread per websocket route (see `Market::connections`) writes into one shared hourly file,
//! so lines from different routes interleave in arrival order; consumers order by exchange time.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use tq_feeds::binance::{TungsteniteWs, WsConnector};
use tq_l2rec::{event_line, msg_line, parse_symbol, snap_line, stream_name, time_line, HourlyWriter, Market};

const SNAPSHOT_EVERY: Duration = Duration::from_secs(600);
const FLUSH_EVERY: Duration = Duration::from_secs(5);
const REST_TIMEOUT: Duration = Duration::from_secs(10);
const FIRST_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);
/// Every subscribed stream must deliver within this long after connecting, or the run is useless.
const SILENT_STREAM_LIMIT: Duration = Duration::from_secs(30);
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

struct Shared {
    market: Market,
    symbols: Vec<String>,
    writer: Mutex<HourlyWriter>,
    rest: ureq::Agent,
    stop: AtomicBool,
    fatal: Mutex<Option<String>>,
}

/// Write errors and silent streams end the whole run; network errors only reconnect.
enum SessionEnd {
    Reconnect(String),
    Fatal(String),
    Stopped,
}

impl Shared {
    fn write(&self, ts: i64, line: &str) -> Result<(), SessionEnd> {
        let mut w = self.writer.lock().map_err(|_| SessionEnd::Fatal("writer lock poisoned".into()))?;
        w.write(ts, line).map_err(|e| SessionEnd::Fatal(format!("write failed: {e}")))
    }

    fn log(&self, event: &str, info: &str) -> Result<(), SessionEnd> {
        eprintln!("{event}: {info}");
        let ts = now_ms();
        self.write(ts, &event_line(ts, event, info))
    }

    fn fail(&self, why: String) {
        let mut f = self.fatal.lock().unwrap_or_else(|p| p.into_inner());
        f.get_or_insert(why);
        self.stop.store(true, Ordering::SeqCst);
    }

    fn server_time(&self) -> Result<(), SessionEnd> {
        let tx = now_ms();
        match self.rest.get(self.market.time_url()).call() {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| SessionEnd::Reconnect(e.to_string()))?;
                let rx = now_ms();
                self.write(rx, &time_line(tx, rx, body.trim()))
            }
            Err(e) => self.log("servertime_failed", &e.to_string()),
        }
    }

    fn snapshots(&self) -> Result<(), SessionEnd> {
        self.server_time()?;
        for sym in &self.symbols {
            match self.rest.get(&self.market.snapshot_url(sym)).call() {
                Ok(resp) => {
                    let body = resp.into_string().map_err(|e| SessionEnd::Reconnect(e.to_string()))?;
                    let ts = now_ms();
                    self.write(ts, &snap_line(ts, sym, body.trim()))?;
                }
                // A missed snapshot only delays the offline book rebuild until the next one.
                Err(e) => self.log("snapshot_failed", &format!("{sym}: {e}"))?,
            }
        }
        Ok(())
    }

    /// One websocket session. The primary connection (the one carrying depth) also takes snapshots.
    fn session(&self, url: &str, streams: &[String], primary: bool) -> SessionEnd {
        let mut conn = match TungsteniteWs.connect(url) {
            Ok(c) => c,
            Err(e) => return SessionEnd::Reconnect(e.to_string()),
        };
        let mut run = || -> Result<SessionEnd, SessionEnd> {
            self.log("connect", url)?;
            if primary {
                self.snapshots()?;
            }
            let (started, mut last_snap) = (Instant::now(), Instant::now());
            let mut unseen: BTreeSet<&str> = streams.iter().map(String::as_str).collect();
            while !self.stop.load(Ordering::SeqCst) {
                if let Some(text) = conn.next_text().map_err(|e| SessionEnd::Reconnect(e.to_string()))? {
                    if let Some(name) = stream_name(&text) {
                        unseen.remove(name);
                    }
                    let ts = now_ms();
                    self.write(ts, &msg_line(ts, &text))?;
                }
                if !unseen.is_empty() && started.elapsed() >= SILENT_STREAM_LIMIT {
                    let why = format!("no message within {}s on {unseen:?} via {url}", SILENT_STREAM_LIMIT.as_secs());
                    self.log("stream_silent", &why)?;
                    return Err(SessionEnd::Fatal(why));
                }
                if primary && last_snap.elapsed() >= SNAPSHOT_EVERY {
                    self.snapshots()?;
                    last_snap = Instant::now();
                }
            }
            Ok(SessionEnd::Stopped)
        };
        run().unwrap_or_else(|end| end)
    }

    fn connection_loop(&self, url: String, streams: Vec<String>, primary: bool) {
        let mut backoff = FIRST_BACKOFF;
        while !self.stop.load(Ordering::SeqCst) {
            match self.session(&url, &streams, primary) {
                SessionEnd::Stopped => return,
                SessionEnd::Fatal(why) => return self.fail(why),
                SessionEnd::Reconnect(e) => {
                    if let Err(SessionEnd::Fatal(why)) = self.log("disconnect", &format!("{url}: {e}")) {
                        return self.fail(why);
                    }
                    thread::sleep(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
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
    let shared = Arc::new(Shared {
        market,
        symbols: symbols.clone(),
        writer: Mutex::new(writer),
        rest: ureq::AgentBuilder::new().timeout(REST_TIMEOUT).build(),
        stop: AtomicBool::new(false),
        fatal: Mutex::new(None),
    });
    let threads: Vec<_> = market
        .connections(&symbols)
        .into_iter()
        .enumerate()
        .map(|(i, (url, streams))| {
            let s = Arc::clone(&shared);
            thread::spawn(move || s.connection_loop(url, streams, i == 0))
        })
        .collect();
    let deadline = args.minutes.map(|m| Instant::now() + Duration::from_secs(m * 60));
    let max_bytes = args.max_mb * BYTES_PER_MB;
    let why = loop {
        thread::sleep(FLUSH_EVERY);
        if shared.stop.load(Ordering::SeqCst) {
            break "fatal error";
        }
        let bytes = {
            let mut w = shared.writer.lock().map_err(|_| "writer lock poisoned".to_string())?;
            w.flush().map_err(|e| format!("flush failed: {e}"))?;
            w.bytes_on_disk()
        };
        if deadline.is_some_and(|d| Instant::now() >= d) {
            break "deadline reached";
        }
        if bytes >= max_bytes {
            break "disk cap reached";
        }
    };
    shared.stop.store(true, Ordering::SeqCst);
    for t in threads {
        let _ = t.join();
    }
    let _ = shared.log("stop", why);
    let mut w = shared.writer.lock().map_err(|_| "writer lock poisoned".to_string())?;
    w.finish().map_err(|e| format!("finish failed: {e}"))?;
    eprintln!("stopped: {why}; {} MB in {}", w.bytes_on_disk() / BYTES_PER_MB, dir.display());
    drop(w);
    let fatal = shared.fatal.lock().unwrap_or_else(|p| p.into_inner()).take();
    fatal.map_or(Ok(()), Err)
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

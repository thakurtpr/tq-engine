//! Raw market-data recorder for maker-side research (edge-hunt-01 candidate B).
//!
//! Records exactly what Binance sends on its PUBLIC combined stream, untouched, so the order book
//! can be rebuilt offline with Binance's documented diff-depth sync: a REST depth snapshot, then
//! every diff whose update ids follow it. Each line is one JSON object:
//!   `{"rx":<local ms>,"m":<raw stream message>}`                 websocket message
//!   `{"rx":<local ms>,"snap":"BTCUSDT","m":<raw REST depth body>}` REST snapshot
//!   `{"rx":<local ms>,"tx":<local ms>,"servertime":{"serverTime":<ms>}}` clock-offset probe
//!   `{"rx":<local ms>,"event":"connect"|"disconnect","info":"..."}` link state, so gaps are explicit
//! Files rotate per UTC hour: `<out>/<market>/<YYYY-MM-DDTHH>.jsonl.gz`.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::DateTime;
use flate2::write::GzEncoder;
use flate2::Compression;

const HOUR_MS: i64 = 3_600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    Spot,
    /// USD-M perpetual futures.
    Um,
}

impl Market {
    pub fn name(self) -> &'static str {
        match self {
            Market::Spot => "spot",
            Market::Um => "um",
        }
    }

    pub fn stream_url(self, symbols: &[String]) -> String {
        let base = match self {
            Market::Spot => "wss://stream.binance.com:9443",
            Market::Um => "wss://fstream.binance.com",
        };
        let streams: Vec<String> = symbols
            .iter()
            .flat_map(|s| {
                let s = s.to_ascii_lowercase();
                [format!("{s}@depth@100ms"), format!("{s}@aggTrade"), format!("{s}@bookTicker")]
            })
            .collect();
        format!("{base}/stream?streams={}", streams.join("/"))
    }

    /// Public depth snapshot; spot weight 50, USD-M weight 20 at limit 1000.
    pub fn snapshot_url(self, symbol: &str) -> String {
        match self {
            Market::Spot => format!("https://api.binance.com/api/v3/depth?symbol={symbol}&limit=1000"),
            Market::Um => format!("https://fapi.binance.com/fapi/v1/depth?symbol={symbol}&limit=1000"),
        }
    }

    /// Public server time (weight 1), logged with send and receive times to estimate clock offset.
    pub fn time_url(self) -> &'static str {
        match self {
            Market::Spot => "https://api.binance.com/api/v3/time",
            Market::Um => "https://fapi.binance.com/fapi/v1/time",
        }
    }
}

/// Upper-case alphanumeric Binance symbol, or an error naming the bad input.
pub fn parse_symbol(raw: &str) -> Result<String, String> {
    let s = raw.trim().to_ascii_uppercase();
    if s.len() >= 2 && s.len() <= 20 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
        Ok(s)
    } else {
        Err(format!("invalid Binance symbol {raw:?}"))
    }
}

pub fn msg_line(rx_ms: i64, raw: &str) -> String {
    format!("{{\"rx\":{rx_ms},\"m\":{raw}}}")
}

pub fn snap_line(rx_ms: i64, symbol: &str, raw: &str) -> String {
    format!("{{\"rx\":{rx_ms},\"snap\":\"{symbol}\",\"m\":{raw}}}")
}

/// Offset estimate: server time minus the midpoint of `tx_ms` and `rx_ms`.
pub fn time_line(tx_ms: i64, rx_ms: i64, raw: &str) -> String {
    format!("{{\"rx\":{rx_ms},\"tx\":{tx_ms},\"servertime\":{raw}}}")
}

pub fn event_line(rx_ms: i64, event: &str, info: &str) -> String {
    let info: String = info.chars().filter(|c| *c != '"' && *c != '\\' && !c.is_control()).collect();
    format!("{{\"rx\":{rx_ms},\"event\":\"{event}\",\"info\":\"{info}\"}}")
}

pub fn hour_file(dir: &Path, ts_ms: i64) -> PathBuf {
    let hour = ts_ms.div_euclid(HOUR_MS) * HOUR_MS;
    let stamp = DateTime::from_timestamp_millis(hour).map_or_else(|| hour.to_string(), |d| d.format("%Y-%m-%dT%H").to_string());
    dir.join(format!("{stamp}.jsonl.gz"))
}

/// Gzip JSONL writer that rolls to a new file at each UTC hour. An existing file for the same
/// hour (a restart) is never overwritten: the new one gets a `.N` suffix.
pub struct HourlyWriter {
    dir: PathBuf,
    current: Option<(i64, PathBuf, GzEncoder<File>)>,
    closed_bytes: u64,
}

impl HourlyWriter {
    pub fn new(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        let closed_bytes = dir_bytes(&dir)?;
        Ok(Self { dir, current: None, closed_bytes })
    }

    pub fn write(&mut self, ts_ms: i64, line: &str) -> io::Result<()> {
        let hour = ts_ms.div_euclid(HOUR_MS);
        if self.current.as_ref().map(|c| c.0) != Some(hour) {
            self.rotate(ts_ms)?;
        }
        let enc = &mut self.current.as_mut().expect("rotate opened a file").2;
        enc.write_all(line.as_bytes())?;
        enc.write_all(b"\n")
    }

    /// Sync-flush so a crash loses at most the last flush interval, not the hour.
    pub fn flush(&mut self) -> io::Result<()> {
        self.current.as_mut().map_or(Ok(()), |c| c.2.flush())
    }

    /// Bytes on disk in finished files plus the open file so far.
    pub fn bytes_on_disk(&self) -> u64 {
        let open = self.current.as_ref().and_then(|c| fs::metadata(&c.1).ok()).map_or(0, |m| m.len());
        self.closed_bytes + open
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if let Some((_, path, enc)) = self.current.take() {
            enc.finish()?;
            self.closed_bytes += fs::metadata(path)?.len();
        }
        Ok(())
    }

    fn rotate(&mut self, ts_ms: i64) -> io::Result<()> {
        self.finish()?;
        let mut path = hour_file(&self.dir, ts_ms);
        let mut n = 1;
        while path.exists() {
            path = hour_file(&self.dir, ts_ms).with_extension(format!("gz.{n}"));
            n += 1;
        }
        let enc = GzEncoder::new(File::create(&path)?, Compression::default());
        self.current = Some((ts_ms.div_euclid(HOUR_MS), path, enc));
        Ok(())
    }
}

fn dir_bytes(dir: &Path) -> io::Result<u64> {
    fs::read_dir(dir)?.try_fold(0, |acc, e| Ok(acc + e?.metadata()?.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::io::Read;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tq-l2rec-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn gunzip(p: &Path) -> String {
        let mut s = String::new();
        GzDecoder::new(File::open(p).unwrap()).read_to_string(&mut s).unwrap();
        s
    }

    #[test]
    fn stream_url_subscribes_depth_trades_and_top_of_book_per_symbol() {
        let url = Market::Um.stream_url(&["BTCUSDT".into(), "ETHUSDT".into()]);
        assert_eq!(
            url,
            "wss://fstream.binance.com/stream?streams=btcusdt@depth@100ms/btcusdt@aggTrade/btcusdt@bookTicker/\
             ethusdt@depth@100ms/ethusdt@aggTrade/ethusdt@bookTicker"
        );
    }

    #[test]
    fn rejects_symbols_that_could_inject_into_urls() {
        assert_eq!(parse_symbol(" btcusdt ").unwrap(), "BTCUSDT");
        assert!(parse_symbol("BTC&limit=5").is_err());
        assert!(parse_symbol("X").is_err());
    }

    #[test]
    fn event_info_cannot_break_the_json_line() {
        assert_eq!(event_line(5, "disconnect", "bad \"x\"\\\n"), r#"{"rx":5,"event":"disconnect","info":"bad x"}"#);
    }

    #[test]
    fn hour_file_names_use_utc_hour() {
        // 2026-11-02T23:59:59.999Z
        let p = hour_file(Path::new("/d"), 1_793_663_999_999);
        assert_eq!(p, PathBuf::from("/d/2026-11-02T23.jsonl.gz"));
    }

    #[test]
    fn writer_rotates_each_hour_and_never_overwrites_on_restart() {
        let dir = tmp("rotate");
        let t0 = 1_793_660_400_000; // 2026-11-02T23:00Z
        let mut w = HourlyWriter::new(dir.clone()).unwrap();
        w.write(t0, "a").unwrap();
        w.write(t0 + HOUR_MS, "b").unwrap();
        w.finish().unwrap();
        let mut again = HourlyWriter::new(dir.clone()).unwrap();
        again.write(t0, "c").unwrap();
        again.finish().unwrap();
        assert_eq!(gunzip(&dir.join("2026-11-02T23.jsonl.gz")), "a\n");
        assert_eq!(gunzip(&dir.join("2026-11-03T00.jsonl.gz")), "b\n");
        assert_eq!(gunzip(&dir.join("2026-11-02T23.jsonl.gz.1")), "c\n");
        assert!(again.bytes_on_disk() > 0);
        fs::remove_dir_all(&dir).unwrap();
    }
}

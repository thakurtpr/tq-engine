//! Intraday bar files: Binance spot kline CSVs (scripts/fetch_binance_klines.py) and the NSE
//! intraday CSVs (scripts/fetch_nse_intraday.py).
//!
//! Only the first six columns are read: `open_time, open, high, low, close, volume`. A first
//! line that does not start with a digit is a header and is skipped. `open_time` is epoch ms;
//! Binance archives from 2025-01 use MICROseconds, which are detected (> 1e14) and converted.
//! Prices are parsed straight into `Decimal`. Bars must be strictly increasing in time across
//! all files of a directory, or loading fails (no silent sorting or de-duplication).

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rust_decimal::Decimal;
use tq_core::{Bar, Timestamp};

use crate::DataError;

/// Above this an `open_time` is microseconds, not milliseconds (1e14 ms is year 5138).
const MICROS_THRESHOLD: i64 = 100_000_000_000_000;

/// Loads intraday bars from CSV files. `[from, to)` is a half-open window in epoch ms.
#[derive(Debug, Clone)]
pub struct IntradayCsv {
    from: Timestamp,
    to: Timestamp,
}

impl IntradayCsv {
    pub fn new(from: Timestamp, to: Timestamp) -> Self {
        Self { from, to }
    }

    /// All bars in `[from, to)` from every `*.csv` in `dir`, files read in name order.
    pub fn load_dir(&self, dir: &Path) -> Result<Vec<Bar>, DataError> {
        let mut bars = Vec::new();
        for file in csv_files(dir)? {
            bars.extend(self.load_file(&file)?);
        }
        if let Some(i) = bars.windows(2).position(|w| w[1].ts <= w[0].ts) {
            return Err(DataError::Unordered(i + 1));
        }
        Ok(bars)
    }

    /// Bars in `[from, to)` from one file.
    pub fn load_file(&self, path: &Path) -> Result<Vec<Bar>, DataError> {
        let name = path.display().to_string();
        let text = fs::read_to_string(path).map_err(|e| DataError::Io { file: name.clone(), msg: e.to_string() })?;
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let is_header = i == 0 && !line.starts_with(|c: char| c.is_ascii_digit());
            if line.trim().is_empty() || is_header {
                continue;
            }
            let bar = parse_line(line).map_err(|msg| DataError::Parse { file: name.clone(), line: i as u64 + 1, msg })?;
            if bar.ts >= self.from && bar.ts < self.to {
                out.push(bar);
            }
        }
        Ok(out)
    }
}

fn csv_files(dir: &Path) -> Result<Vec<PathBuf>, DataError> {
    let io = |e: std::io::Error| DataError::Io { file: dir.display().to_string(), msg: e.to_string() };
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(io)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "csv"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(DataError::NotFound(dir.display().to_string()));
    }
    Ok(files)
}

/// Parses one row into a validated bar with an ms timestamp.
pub fn parse_line(line: &str) -> Result<Bar, String> {
    let cols: Vec<&str> = line.split(',').map(str::trim).collect();
    if cols.len() < 6 {
        return Err(format!("expected at least 6 columns, got {}", cols.len()));
    }
    let raw_ts: i64 = cols[0].parse().map_err(|_| format!("bad open_time {:?}", cols[0]))?;
    let ts = if raw_ts > MICROS_THRESHOLD { raw_ts / 1000 } else { raw_ts };
    let num = |k: usize| Decimal::from_str(cols[k]).map_err(|_| format!("bad number {:?} in column {}", cols[k], k + 1));
    Bar::new(ts, num(1)?, num(2)?, num(3)?, num(4)?, num(5)?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const ROW_MS: &str = "1640995200000,46216.93,46271.08,46208.37,46250.00,40.57,1640995259999,1876000.1,1000,20.1,930000.2,0";

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tq-intraday-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn parses_binance_row_with_decimal_prices() {
        let b = parse_line(ROW_MS).unwrap();
        assert_eq!(b.ts, 1_640_995_200_000);
        assert_eq!(b.open, dec!(46216.93));
        assert_eq!(b.volume, dec!(40.57));
    }

    #[test]
    fn microsecond_timestamps_become_ms() {
        let b = parse_line("1735689600000000,1,2,0.5,1.5,10,1735689659999999").unwrap();
        assert_eq!(b.ts, 1_735_689_600_000);
    }

    #[test]
    fn rejects_invalid_ohlc_and_short_rows() {
        assert!(parse_line("1,10,9,11,10,1").is_err());
        assert!(parse_line("1,10,11,9").is_err());
    }

    #[test]
    fn loads_dir_in_name_order_skipping_header_and_filtering_window() {
        let d = tmp_dir("ok");
        fs::write(d.join("X-2022-02.csv"), "120000,1,1,1,1,1\n180000,1,1,1,1,1\n").unwrap();
        fs::write(d.join("X-2022-01.csv"), "ts,open,high,low,close,volume\n0,1,1,1,1,1\n60000,1,1,1,1,1\n").unwrap();
        let bars = IntradayCsv::new(60_000, 180_000).load_dir(&d).unwrap();
        assert_eq!(bars.iter().map(|b| b.ts).collect::<Vec<_>>(), vec![60_000, 120_000]);
    }

    #[test]
    fn duplicate_or_unordered_bars_fail_loudly() {
        let d = tmp_dir("dup");
        fs::write(d.join("a.csv"), "0,1,1,1,1,1\n0,1,1,1,1,1\n").unwrap();
        assert_eq!(IntradayCsv::new(0, i64::MAX).load_dir(&d).unwrap_err(), DataError::Unordered(1));
    }

    #[test]
    fn empty_dir_is_not_found() {
        let d = tmp_dir("empty");
        assert!(matches!(IntradayCsv::new(0, 1).load_dir(&d), Err(DataError::NotFound(_))));
    }
}

//! Buy-and-hold benchmark from an index CSV (e.g. ^NSEI saved as `NSEI.csv`).
//!
//! ^NSEI is the NIFTY 50 PRICE index: it excludes dividends, so it understates the total-return
//! index by the dividend yield (about 1.2-1.5% a year for 2012-2021). Beating it is necessary,
//! not sufficient, for the protocol's NIFTY 50 TRI gate.

use std::path::Path;

use rust_decimal::Decimal;
use tq_backtest::returns::{daily_returns, stats, ReturnStats};
use tq_core::{Symbol, Timestamp};
use tq_data::{BarSource, CsvDaily};

use crate::CliError;

pub struct Benchmark {
    pub name: String,
    pub stats: ReturnStats,
    pub returns: Vec<(Timestamp, f64)>,
}

/// Holds the index from the last close before `start_ts` to the last close on or before
/// `end_ts`, the same span as a strategy curve that starts with cash at `start_ts`.
pub fn load(path: &Path, start_ts: Timestamp, end_ts: Timestamp) -> Result<Benchmark, CliError> {
    let usage = |m: String| CliError::Usage(format!("benchmark {}: {m}", path.display()));
    let stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| usage("bad file name".into()))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    let bars = CsvDaily::new(dir).bars(&Symbol(stem.to_string())).map_err(|e| usage(e.to_string()))?;
    let base = bars.iter().take_while(|b| b.ts < start_ts).last().map(|b| b.close);
    let curve: Vec<(Timestamp, Decimal)> =
        bars.iter().filter(|b| b.ts >= start_ts && b.ts <= end_ts).map(|b| (b.ts, b.close)).collect();
    let initial = base.ok_or_else(|| usage("needs at least one bar before --start".into()))?;
    if curve.is_empty() {
        return Err(usage("no bars inside the window".into()));
    }
    Ok(Benchmark { name: stem.to_string(), stats: stats(initial, &curve), returns: daily_returns(initial, &curve) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tq_data::ist_close_ts;

    fn date(s: &str) -> Timestamp {
        ist_close_ts(chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap())
    }

    #[test]
    fn holds_from_the_close_before_start_to_end() {
        let dir = std::env::temp_dir().join(format!("tq-bench-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("IDX.csv");
        let rows = "date,open,high,low,close,volume\n2011-12-30,100,100,100,100,0\n2012-01-02,110,110,110,110,0\n2012-01-03,99,99,99,99,0\n2012-01-04,500,500,500,500,0\n";
        std::fs::write(&path, rows).unwrap();
        let b = load(&path, date("2012-01-01"), date("2012-01-03")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(b.name, "IDX");
        assert_eq!(b.returns.len(), 2);
        assert!((b.stats.total_return_pct - -1.0).abs() < 1e-9);
        assert!((b.stats.max_drawdown_pct - 10.0).abs() < 1e-9);
    }
}

//! `CsvDaily`: daily bars from a directory of `<SYMBOL>.csv` files.
//!
//! Format: header `date,open,high,low,close,volume`, date as `YYYY-MM-DD`, one row per session,
//! oldest first. Every bad row is an error naming the file and line; nothing is skipped.

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use csv::{ReaderBuilder, StringRecord, Trim};
use rust_decimal::Decimal;
use tq_core::{Bar, Symbol};

use crate::{ist_close_ts, BarSource, DataError};

const HEADER: [&str; 6] = ["date", "open", "high", "low", "close", "volume"];
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Reads `<dir>/<SYMBOL>.csv`. An exchange prefix is dropped, so `NSE:INFY` reads `INFY.csv`.
#[derive(Debug, Clone)]
pub struct CsvDaily {
    dir: PathBuf,
}

impl CsvDaily {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

impl BarSource for CsvDaily {
    fn bars(&self, symbol: &Symbol) -> Result<Vec<Bar>, DataError> {
        let path = self.dir.join(format!("{}.csv", file_stem(symbol)?));
        let file_name = path.display().to_string();
        let file = File::open(&path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => DataError::NotFound(symbol.0.clone()),
            _ => DataError::Io {
                file: file_name.clone(),
                msg: e.to_string(),
            },
        })?;
        read_bars(file, &file_name)
    }
}

/// The ticker part of `symbol`, restricted to characters NSE tickers use (e.g. `M&M`,
/// `BAJAJ-AUTO`), so a symbol can never escape the data directory.
fn file_stem(symbol: &Symbol) -> Result<&str, DataError> {
    let stem = symbol.0.rsplit(':').next().unwrap_or_default();
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '&' | '_');
    if stem.is_empty() || !stem.chars().all(allowed) {
        return Err(DataError::InvalidSymbol(symbol.0.clone()));
    }
    Ok(stem)
}

/// Parses CSV text into strictly time-ordered bars. `file` is used only in error messages.
pub(crate) fn read_bars<R: Read>(input: R, file: &str) -> Result<Vec<Bar>, DataError> {
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(input);
    let mut records = reader.into_records();
    let header = records
        .next()
        .transpose()
        .map_err(|e| csv_error(file, &e))?;
    check_header(header.as_ref(), file)?;
    let mut bars: Vec<Bar> = Vec::new();
    for record in records {
        let record = record.map_err(|e| csv_error(file, &e))?;
        let line = record.position().map_or(0, |p| p.line());
        let parse_err = |msg: String| DataError::Parse {
            file: file.to_string(),
            line,
            msg,
        };
        let bar = parse_row(&record).map_err(parse_err)?;
        check_order(bars.last(), &bar).map_err(|m| parse_err(m.to_string()))?;
        bars.push(bar);
    }
    Ok(bars)
}

fn check_header(header: Option<&StringRecord>, file: &str) -> Result<(), DataError> {
    let found: Vec<&str> = header.map(|h| h.iter().collect()).unwrap_or_default();
    if found == HEADER {
        return Ok(());
    }
    let msg = format!(
        "expected header `{}`, found `{}`",
        HEADER.join(","),
        found.join(",")
    );
    Err(DataError::Parse {
        file: file.to_string(),
        line: 1,
        msg,
    })
}

fn parse_row(record: &StringRecord) -> Result<Bar, String> {
    if record.len() != HEADER.len() {
        return Err(format!(
            "expected {} fields, found {}",
            HEADER.len(),
            record.len()
        ));
    }
    let raw = &record[0];
    let date = NaiveDate::parse_from_str(raw, DATE_FORMAT)
        .map_err(|e| format!("bad date `{raw}` (want YYYY-MM-DD): {e}"))?;
    let num = |i: usize| parse_decimal(HEADER[i], &record[i]);
    Bar::new(
        ist_close_ts(date),
        num(1)?,
        num(2)?,
        num(3)?,
        num(4)?,
        num(5)?,
    )
    .map_err(|e| e.to_string())
}

/// Parses straight from text into `Decimal`, never through `f64`.
fn parse_decimal(field: &str, raw: &str) -> Result<Decimal, String> {
    Decimal::from_str(raw).map_err(|e| format!("bad {field} `{raw}`: {e}"))
}

fn check_order(prev: Option<&Bar>, bar: &Bar) -> Result<(), &'static str> {
    match prev {
        Some(p) if bar.ts == p.ts => Err("duplicate date"),
        Some(p) if bar.ts < p.ts => Err("date out of order (rows must be oldest first)"),
        _ => Ok(()),
    }
}

fn csv_error(file: &str, e: &csv::Error) -> DataError {
    let line = e.position().map_or(0, |p| p.line());
    DataError::Parse {
        file: file.to_string(),
        line,
        msg: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_and_padded_fields_are_accepted() {
        let text = "date,open,high,low,close,volume\r\n2024-01-01, 10.5 ,11,10,10.5,7\r\n";
        let bars = read_bars(text.as_bytes(), "t.csv").unwrap();
        assert_eq!(bars.len(), 1);
    }

    #[test]
    fn missing_header_is_rejected() {
        let err = read_bars("".as_bytes(), "t.csv").unwrap_err();
        assert!(matches!(err, DataError::Parse { line: 1, .. }));
    }

    #[test]
    fn symbol_with_slash_is_invalid() {
        assert!(file_stem(&Symbol("a/b".into())).is_err());
        assert_eq!(file_stem(&Symbol("NSE:M&M".into())), Ok("M&M"));
    }
}

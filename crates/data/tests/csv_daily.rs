use std::path::PathBuf;

use rust_decimal_macros::dec;
use tq_core::Symbol;
use tq_data::{ist_close_ts, BarSource, CsvDaily, DataError};

fn source() -> CsvDaily {
    CsvDaily::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"))
}

fn sym(s: &str) -> Symbol {
    Symbol(s.into())
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Asserts the error is a parse error for `file` at `line` and its message contains `needle`.
fn assert_parse_err(symbol: &str, line: u64, needle: &str) {
    let err = source().bars(&sym(symbol)).unwrap_err();
    match &err {
        DataError::Parse { file, line: l, msg } => {
            assert!(file.ends_with(&format!("{symbol}.csv")), "file was {file}");
            assert_eq!(*l, line, "wrong line in {err}");
            assert!(msg.contains(needle), "message `{msg}` lacks `{needle}`");
        }
        other => panic!("expected parse error, got {other:?}"),
    }
    let text = err.to_string();
    assert!(
        text.contains(&format!("{symbol}.csv:{line}")),
        "display was {text}"
    );
}

#[test]
fn loads_all_rows_of_valid_file() {
    let bars = source().bars(&sym("SYNTH")).unwrap();
    assert_eq!(bars.len(), 30);
}

#[test]
fn parses_prices_exactly_as_decimal() {
    let bars = source().bars(&sym("SYNTH")).unwrap();
    let first = &bars[0];
    assert_eq!(first.open, dec!(100.25));
    assert_eq!(first.high, dec!(101.60));
    assert_eq!(first.low, dec!(99.15));
    assert_eq!(first.close, dec!(100.50));
    assert_eq!(first.volume, dec!(101000));
}

#[test]
fn timestamp_is_1530_ist_as_utc_millis() {
    let bars = source().bars(&sym("SYNTH")).unwrap();
    // 2024-01-01 15:30 IST == 2024-01-01 10:00 UTC == 1704103200000 ms.
    assert_eq!(bars[0].ts, 1_704_103_200_000);
    assert_eq!(ist_close_ts(date(2024, 1, 1)), 1_704_103_200_000);
}

#[test]
fn bars_are_oldest_first() {
    let bars = source().bars(&sym("SYNTH")).unwrap();
    assert!(bars.windows(2).all(|w| w[0].ts < w[1].ts));
}

#[test]
fn exchange_prefix_is_stripped_from_symbol() {
    let bars = source().bars(&sym("NSE:SYNTH")).unwrap();
    assert_eq!(bars.len(), 30);
}

#[test]
fn missing_file_is_not_found() {
    assert_eq!(
        source().bars(&sym("NOPE")),
        Err(DataError::NotFound("NOPE".into()))
    );
}

#[test]
fn rejects_path_traversal_symbol() {
    let err = source().bars(&sym("../SYNTH")).unwrap_err();
    assert!(matches!(err, DataError::InvalidSymbol(_)), "got {err:?}");
}

#[test]
fn rejects_invalid_ohlc_naming_file_and_line() {
    assert_parse_err("BADOHLC", 4, "high below low");
}

#[test]
fn rejects_duplicate_date() {
    assert_parse_err("DUPDATE", 5, "duplicate date");
}

#[test]
fn rejects_out_of_order_date() {
    assert_parse_err("UNORDERED", 4, "out of order");
}

#[test]
fn rejects_non_numeric_price() {
    assert_parse_err("BADPRICE", 5, "open");
}

#[test]
fn rejects_bad_date_format() {
    assert_parse_err("BADDATE", 3, "date");
}

#[test]
fn rejects_row_with_missing_field() {
    assert_parse_err("SHORTROW", 4, "fields");
}

#[test]
fn rejects_unexpected_header() {
    assert_parse_err("BADHEADER", 1, "header");
}

#[test]
fn empty_file_yields_no_bars() {
    assert_eq!(source().bars(&sym("EMPTY")).unwrap(), vec![]);
}

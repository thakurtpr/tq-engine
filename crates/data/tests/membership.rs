//! Point-in-time index membership parsed from `date,symbol` snapshots.

use chrono::NaiveDate;
use tq_data::{DataError, Membership};

const CSV: &str = "date,symbol\n2012-01-01,AAA\n2012-01-01,BBB\n2012-04-27,AAA\n2012-04-27,CCC\n2013-04-01,BBB\n2013-04-01,CCC\n";

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn membership() -> Membership {
    Membership::from_reader(CSV.as_bytes(), "m.csv").unwrap()
}

#[test]
fn snapshot_in_effect_holds_until_the_next_one() {
    let m = membership();
    assert!(m.is_member("BBB", d(2012, 4, 26)));
    assert!(!m.is_member("BBB", d(2012, 4, 27)), "removed on the effective date");
    assert!(m.is_member("CCC", d(2012, 4, 27)), "added on the effective date");
    assert!(m.is_member("BBB", d(2020, 1, 1)), "last snapshot holds open-ended");
}

#[test]
fn nobody_is_a_member_before_the_first_snapshot() {
    assert!(!membership().is_member("AAA", d(2011, 12, 31)));
}

#[test]
fn windows_merge_consecutive_snapshots_and_reopen_after_a_gap() {
    let m = membership();
    assert_eq!(m.windows("AAA"), vec![(d(2012, 1, 1), Some(d(2013, 4, 1)))]);
    assert_eq!(m.windows("BBB"), vec![(d(2012, 1, 1), Some(d(2012, 4, 27))), (d(2013, 4, 1), None)]);
    assert_eq!(m.windows("CCC"), vec![(d(2012, 4, 27), None)]);
    assert!(m.windows("ZZZ").is_empty());
}

#[test]
fn lists_every_symbol_ever_a_member() {
    let all: Vec<String> = membership().symbols().into_iter().collect();
    assert_eq!(all, vec!["AAA", "BBB", "CCC"]);
}

#[test]
fn rejects_bad_header_bad_date_and_empty_file() {
    let bad = |text: &str| Membership::from_reader(text.as_bytes(), "m.csv").unwrap_err();
    assert!(matches!(bad("day,ticker\n2012-01-01,A\n"), DataError::Parse { line: 1, .. }));
    assert!(matches!(bad("date,symbol\n2012-13-01,A\n"), DataError::Parse { line: 2, .. }));
    assert!(matches!(bad("date,symbol\n"), DataError::Parse { .. }));
}

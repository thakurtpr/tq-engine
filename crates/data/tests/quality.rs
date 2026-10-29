use std::path::PathBuf;

use chrono::NaiveDate;
use rust_decimal_macros::dec;
use tq_core::{Bar, Symbol};
use tq_data::{ist_close_ts, quality_report, BarSource, CsvDaily, Gap, Jump};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn synth_bars() -> Vec<Bar> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    CsvDaily::new(dir).bars(&Symbol("SYNTH".into())).unwrap()
}

#[test]
fn reports_count_and_date_range() {
    let r = quality_report(&synth_bars()).unwrap();
    assert_eq!(r.bars, 30);
    assert_eq!(r.first, Some(date(2024, 1, 1)));
    assert_eq!(r.last, Some(date(2024, 2, 16)));
}

#[test]
fn flags_gap_longer_than_five_days_but_not_weekends() {
    let r = quality_report(&synth_bars()).unwrap();
    assert_eq!(
        r.gaps,
        vec![Gap {
            from: date(2024, 1, 19),
            to: date(2024, 1, 29),
            days: 10
        }]
    );
}

#[test]
fn flags_zero_volume_days() {
    let r = quality_report(&synth_bars()).unwrap();
    assert_eq!(r.zero_volume, vec![date(2024, 1, 12)]);
}

#[test]
fn flags_close_to_close_jump_over_twenty_percent() {
    let r = quality_report(&synth_bars()).unwrap();
    assert_eq!(
        r.jumps,
        vec![Jump {
            from: date(2024, 2, 1),
            to: date(2024, 2, 2),
            change: dec!(0.25)
        }]
    );
}

#[test]
fn flags_downward_jump_and_ignores_exactly_twenty_percent() {
    let bar = |d: NaiveDate, c| Bar::new(ist_close_ts(d), c, c, c, c, dec!(1)).unwrap();
    let bars = vec![
        bar(date(2024, 3, 1), dec!(100)),
        bar(date(2024, 3, 4), dec!(120)), // +20% exactly: not flagged
        bar(date(2024, 3, 5), dec!(60)),  // -50%: a 1:1 bonus left unadjusted
    ];
    let r = quality_report(&bars).unwrap();
    assert_eq!(
        r.jumps,
        vec![Jump {
            from: date(2024, 3, 4),
            to: date(2024, 3, 5),
            change: dec!(-0.5)
        }]
    );
}

#[test]
fn empty_input_gives_empty_report() {
    let r = quality_report(&[]).unwrap();
    assert_eq!(r.bars, 0);
    assert_eq!(r.first, None);
    assert_eq!(r.last, None);
    assert!(r.gaps.is_empty() && r.zero_volume.is_empty() && r.jumps.is_empty());
}

#[test]
fn rejects_timestamp_outside_calendar_range() {
    let b = Bar::new(i64::MAX, dec!(1), dec!(1), dec!(1), dec!(1), dec!(1)).unwrap();
    assert!(quality_report(&[b]).is_err());
}

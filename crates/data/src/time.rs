//! NSE calendar helpers. Daily bars are stamped at the 15:30 IST close, stored as UTC epoch ms.

use chrono::{DateTime, NaiveDate, NaiveTime};
use tq_core::Timestamp;

const HOUR_MS: i64 = 3_600_000;
const MINUTE_MS: i64 = 60_000;
/// 15:30 IST is 10:00 UTC (IST is UTC+05:30 with no daylight saving).
const CLOSE_UTC_OFFSET_MS: i64 = 10 * HOUR_MS;
const IST_OFFSET_MS: i64 = 5 * HOUR_MS + 30 * MINUTE_MS;

/// UTC epoch ms of the 15:30 IST market close on `date`.
pub fn ist_close_ts(date: NaiveDate) -> Timestamp {
    date.and_time(NaiveTime::MIN).and_utc().timestamp_millis() + CLOSE_UTC_OFFSET_MS
}

/// The IST calendar date of `ts`, or `None` if it is outside chrono's range.
pub fn ist_date(ts: Timestamp) -> Option<NaiveDate> {
    let local = ts.checked_add(IST_OFFSET_MS)?;
    DateTime::from_timestamp_millis(local).map(|dt| dt.date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_ts_round_trips_to_same_date() {
        let d = NaiveDate::from_ymd_opt(2025, 3, 31).unwrap();
        assert_eq!(ist_date(ist_close_ts(d)), Some(d));
    }

    #[test]
    fn early_utc_morning_is_already_next_ist_day() {
        // 2024-01-01 20:00 UTC is 2024-01-02 01:30 IST.
        let ts = ist_close_ts(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()) + 10 * HOUR_MS;
        assert_eq!(ist_date(ts), NaiveDate::from_ymd_opt(2024, 1, 2));
    }
}

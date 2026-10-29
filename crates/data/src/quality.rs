//! Data quality checks for a daily bar series. Reports findings; never prints or mutates.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use tq_core::Bar;

use crate::{ist_date, DataError};

/// Calendar days between consecutive bars above which data may be missing. Weekends and
/// typical NSE holiday clusters stay at or below this.
pub const GAP_THRESHOLD_DAYS: i64 = 5;
/// Absolute close-to-close change (0.20 = 20%) above which a move looks like an unadjusted
/// split or bonus rather than trading.
pub const JUMP_THRESHOLD: Decimal = Decimal::from_parts(20, 0, 0, false, 2);

/// Consecutive bars more than `GAP_THRESHOLD_DAYS` calendar days apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub days: i64,
}

/// A close-to-close move beyond `JUMP_THRESHOLD`. `change` is fractional: 0.25 = +25%.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jump {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub change: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub bars: usize,
    pub first: Option<NaiveDate>,
    pub last: Option<NaiveDate>,
    pub gaps: Vec<Gap>,
    pub zero_volume: Vec<NaiveDate>,
    pub jumps: Vec<Jump>,
}

/// Summarises `bars` (oldest first). Dates are IST trading dates.
pub fn quality_report(bars: &[Bar]) -> Result<QualityReport, DataError> {
    let dates = bars
        .iter()
        .map(|b| ist_date(b.ts).ok_or(DataError::BadTimestamp(b.ts)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(QualityReport {
        bars: bars.len(),
        first: dates.first().copied(),
        last: dates.last().copied(),
        gaps: find_gaps(&dates),
        zero_volume: dates
            .iter()
            .zip(bars)
            .filter(|(_, b)| b.volume.is_zero())
            .map(|(d, _)| *d)
            .collect(),
        jumps: find_jumps(&dates, bars),
    })
}

fn find_gaps(dates: &[NaiveDate]) -> Vec<Gap> {
    dates
        .windows(2)
        .filter_map(|w| {
            let days = (w[1] - w[0]).num_days();
            (days > GAP_THRESHOLD_DAYS).then_some(Gap {
                from: w[0],
                to: w[1],
                days,
            })
        })
        .collect()
}

fn find_jumps(dates: &[NaiveDate], bars: &[Bar]) -> Vec<Jump> {
    dates
        .windows(2)
        .zip(bars.windows(2))
        .filter_map(|(d, b)| {
            // `Bar::new` guarantees close > 0, so the division only fails on hand-built bars.
            let change = (b[1].close - b[0].close).checked_div(b[0].close)?;
            (change.abs() > JUMP_THRESHOLD).then_some(Jump {
                from: d[0],
                to: d[1],
                change,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_threshold_is_twenty_percent() {
        assert_eq!(JUMP_THRESHOLD, Decimal::new(2, 1));
    }
}

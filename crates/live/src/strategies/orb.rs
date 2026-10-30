//! (b) Opening-range breakout, NSE 5m bars. Pre-registered 2026-10-30; do not tune.
//!
//! Range = high and low of the bars opening 09:15-09:30 IST (three 5m bars). From the 09:30 bar
//! on, long when a bar CLOSES above the range high (fills at the next open), stop at the range
//! low, at most one entry per day. The runner flattens at 15:15 and blocks later entries.

use rust_decimal::Decimal;
use tq_core::{Bar, Fill, Side, Symbol};
use tq_strategy::{Signal, Strategy};

use super::{long_entry, Holding};
use crate::session::{Session, NSE_OPEN_MIN};

pub const RANGE_END_MIN: i64 = 9 * 60 + 30;

#[derive(Debug, Clone)]
pub struct OpeningRangeBreakout {
    session: Session,
    day: Option<i64>,
    high: Option<Decimal>,
    low: Option<Decimal>,
    entered_today: bool,
    hold: Holding,
}

impl OpeningRangeBreakout {
    pub fn new(session: Session) -> Self {
        Self { session, day: None, high: None, low: None, entered_today: false, hold: Holding::default() }
    }

    fn roll(&mut self, bar: &Bar) {
        let day = self.session.day_key(bar.ts);
        if self.day != Some(day) {
            *self = Self { day: Some(day), hold: self.hold, ..Self::new(self.session) };
        }
    }
}

impl Strategy for OpeningRangeBreakout {
    fn name(&self) -> &str {
        "opening_range_breakout"
    }

    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal> {
        self.roll(bar);
        let m = self.session.minute_of_day(bar.ts);
        if (NSE_OPEN_MIN..RANGE_END_MIN).contains(&m) {
            self.high = Some(self.high.map_or(bar.high, |h| h.max(bar.high)));
            self.low = Some(self.low.map_or(bar.low, |l| l.min(bar.low)));
            return Vec::new();
        }
        if m < RANGE_END_MIN || self.hold.in_position || self.entered_today {
            return Vec::new();
        }
        match (self.high, self.low) {
            (Some(h), Some(l)) if bar.close > h => long_entry(symbol, bar.close, l),
            _ => Vec::new(),
        }
    }

    fn on_fill(&mut self, fill: &Fill) {
        self.hold.on_fill(fill.side);
        if fill.side == Side::Buy {
            self.entered_today = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{bar, fill, sym};
    use super::*;
    use rust_decimal_macros::dec;

    /// 2024-01-02 at `h:m` IST as UTC epoch ms.
    fn ist(h: i64, m: i64) -> i64 {
        1_704_153_600_000 + (h * 60 + m) * 60_000 - 19_800_000
    }

    fn build_range(s: &mut OpeningRangeBreakout) {
        for (i, (hi, lo)) in [(dec!(101), dec!(99)), (dec!(102), dec!(100)), (dec!(101), dec!(98))].into_iter().enumerate() {
            let b = bar(ist(9, 15 + 5 * i as i64), dec!(100), hi, lo, dec!(100), dec!(1));
            assert!(s.on_bar(&sym(), &b).is_empty());
        }
    }

    #[test]
    fn breakout_above_range_high_enters_with_range_low_stop() {
        let mut s = OpeningRangeBreakout::new(Session::Nse);
        build_range(&mut s);
        assert!(s.on_bar(&sym(), &bar(ist(9, 30), dec!(100), dec!(102), dec!(100), dec!(101.5), dec!(1))).is_empty());
        let sig = s.on_bar(&sym(), &bar(ist(9, 35), dec!(101.5), dec!(103), dec!(101), dec!(102.5), dec!(1)));
        let Some(Signal::Enter(o)) = sig.first() else { panic!("expected entry") };
        assert_eq!((o.entry, o.stop), (dec!(102.5), dec!(98)));
    }

    #[test]
    fn only_one_entry_per_day_and_range_resets_next_day() {
        let mut s = OpeningRangeBreakout::new(Session::Nse);
        build_range(&mut s);
        s.on_fill(&fill(Side::Buy, dec!(103)));
        s.on_fill(&fill(Side::Sell, dec!(104)));
        assert!(s.on_bar(&sym(), &bar(ist(10, 0), dec!(103), dec!(104), dec!(103), dec!(104), dec!(1))).is_empty());
        let next_day = ist(9, 35) + 86_400_000;
        assert!(s.on_bar(&sym(), &bar(next_day, dec!(103), dec!(104), dec!(103), dec!(104), dec!(1))).is_empty());
    }
}

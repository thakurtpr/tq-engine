//! Cross-sectional 12-1 momentum rotation (Jegadeesh & Titman 1993), pre-registered 2026-10-30.
//!
//! On each rebalance date (the engine calls it after every month's last close), rank the
//! eligible members by the return from 252 trading days ago to 21 trading days ago (about twelve
//! months, skipping the most recent month, whose short-term reversal would otherwise contaminate
//! the signal), and hold the top 10 at equal weight from the next open. A member needs 253 closes
//! (today plus 252 bars of history) to be ranked; others are dropped. Ties go to the earlier ticker.
//! Parameters are fixed from the literature, NOT tuned: changing any of them is a new trial.

use rust_decimal::Decimal;
use tq_core::{Symbol, Timestamp};
use tq_strategy::{History, Rebalancer, Target};

use crate::ConfigError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MomentumRotationConfig {
    /// Trading days back to the start of the formation window (12 months).
    pub lookback: usize,
    /// Most recent trading days skipped (1 month).
    pub skip: usize,
    /// Names held, equal weight.
    pub top_n: usize,
}

impl Default for MomentumRotationConfig {
    fn default() -> Self {
        Self { lookback: 252, skip: 21, top_n: 10 }
    }
}

#[derive(Debug)]
pub struct MomentumRotation {
    config: MomentumRotationConfig,
}

impl MomentumRotation {
    pub fn new(config: MomentumRotationConfig) -> Result<Self, ConfigError> {
        if config.top_n == 0 {
            return Err(ConfigError("top_n must be positive"));
        }
        if config.skip >= config.lookback {
            return Err(ConfigError("skip must be shorter than lookback"));
        }
        Ok(Self { config })
    }

    /// 12-1 return of one close series, `None` without enough history or a zero base.
    fn score(&self, closes: &[Decimal]) -> Option<Decimal> {
        let last = closes.len().checked_sub(1)?;
        let start = closes.get(last.checked_sub(self.config.lookback)?)?;
        let end = closes.get(last - self.config.skip)?;
        (!start.is_zero()).then(|| end / start - Decimal::ONE)
    }
}

impl Rebalancer for MomentumRotation {
    fn name(&self) -> &str {
        "momentum-rotation"
    }

    fn targets(&mut self, _date: Timestamp, members: &[Symbol], history: &History) -> Vec<Target> {
        let mut ranked: Vec<(&Symbol, Decimal)> =
            members.iter().filter_map(|m| history.get(m).and_then(|c| self.score(c)).map(|s| (m, s))).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0 .0.cmp(&b.0 .0)));
        let weight = Decimal::ONE / Decimal::from(self.config.top_n);
        ranked.into_iter().take(self.config.top_n).map(|(s, _)| (s.clone(), weight)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sym(s: &str) -> Symbol {
        Symbol(s.into())
    }

    /// 253 closes: 100 at t-252, `mid` from t-251 to t-21, then `recent` for the last 20 days.
    fn closes(mid: Decimal, recent: Decimal) -> Vec<Decimal> {
        let mut c = vec![dec!(100)];
        c.extend(std::iter::repeat_n(mid, 232));
        c.extend(std::iter::repeat_n(recent, 20));
        c
    }

    fn small(top_n: usize) -> MomentumRotation {
        MomentumRotation::new(MomentumRotationConfig { top_n, ..Default::default() }).unwrap()
    }

    #[test]
    fn defaults_are_the_pre_registered_parameters() {
        assert_eq!(MomentumRotationConfig::default(), MomentumRotationConfig { lookback: 252, skip: 21, top_n: 10 });
    }

    #[test]
    fn ranks_on_twelve_minus_one_return_ignoring_the_last_month() {
        // A: +50% to t-21 then crashes; B: +20% and soars in the skipped month. A ranks first.
        let h: History = [(sym("A"), closes(dec!(150), dec!(50))), (sym("B"), closes(dec!(120), dec!(300)))].into();
        let t = small(1).targets(0, &[sym("A"), sym("B")], &h);
        assert_eq!(t, vec![(sym("A"), dec!(1))]);
    }

    #[test]
    fn holds_top_n_equal_weight_members_only_with_full_history() {
        let mut short = closes(dec!(500), dec!(500));
        short.remove(0); // 252 closes: one short of a ranking.
        let h: History = [
            (sym("A"), closes(dec!(110), dec!(110))),
            (sym("B"), closes(dec!(130), dec!(130))),
            (sym("C"), closes(dec!(120), dec!(120))),
            (sym("NOTMEMBER"), closes(dec!(900), dec!(900))),
            (sym("SHORT"), short),
        ]
        .into();
        let members = [sym("A"), sym("B"), sym("C"), sym("SHORT")];
        assert_eq!(small(2).targets(0, &members, &h), vec![(sym("B"), dec!(0.5)), (sym("C"), dec!(0.5))]);
        let all = small(10).targets(0, &members, &h);
        assert_eq!(all.len(), 3, "fewer eligible than top_n: the rest stays cash");
        assert!(all.iter().all(|t| t.1 == dec!(0.1)));
    }

    #[test]
    fn ties_go_to_the_earlier_ticker() {
        let h: History = [(sym("Z"), closes(dec!(120), dec!(1))), (sym("Y"), closes(dec!(120), dec!(1)))].into();
        assert_eq!(small(1).targets(0, &[sym("Z"), sym("Y")], &h), vec![(sym("Y"), dec!(1))]);
    }

    #[test]
    fn rejects_degenerate_configs() {
        assert!(MomentumRotation::new(MomentumRotationConfig { top_n: 0, ..Default::default() }).is_err());
        assert!(MomentumRotation::new(MomentumRotationConfig { skip: 252, ..Default::default() }).is_err());
    }
}

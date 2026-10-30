//! Index trend filter (Faber 2007, "A Quantitative Approach to Tactical Asset Allocation"),
//! pre-registered 2026-10-30: hold the index fund while its month-end close is above the simple
//! average of the last 10 month-end closes (the current one included), otherwise hold cash.
//! The engine calls `targets` after every month's last close, warm-up months included, so the
//! list of month-end closes is complete. Parameters are fixed, NOT tuned.

use rust_decimal::Decimal;
use tq_core::{Price, Symbol, Timestamp};
use tq_strategy::{History, Rebalancer, Target};

use crate::ConfigError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexTrendFilterConfig {
    /// The index fund (e.g. NSE:NIFTYBEES).
    pub symbol: Symbol,
    /// Month-end closes in the moving average.
    pub months: usize,
}

impl IndexTrendFilterConfig {
    /// Faber's 10-month rule on `symbol`.
    pub fn faber(symbol: Symbol) -> Self {
        Self { symbol, months: 10 }
    }
}

#[derive(Debug)]
pub struct IndexTrendFilter {
    config: IndexTrendFilterConfig,
    month_closes: Vec<Price>,
}

impl IndexTrendFilter {
    pub fn new(config: IndexTrendFilterConfig) -> Result<Self, ConfigError> {
        if config.months == 0 {
            return Err(ConfigError("months must be positive"));
        }
        Ok(Self { config, month_closes: Vec::new() })
    }
}

impl Rebalancer for IndexTrendFilter {
    fn name(&self) -> &str {
        "index-trend-filter"
    }

    fn targets(&mut self, _date: Timestamp, members: &[Symbol], history: &History) -> Vec<Target> {
        let Some(close) = history.get(&self.config.symbol).and_then(|c| c.last()) else { return Vec::new() };
        self.month_closes.push(*close);
        let n = self.config.months;
        if self.month_closes.len() < n || !members.contains(&self.config.symbol) {
            return Vec::new();
        }
        let window = &self.month_closes[self.month_closes.len() - n..];
        let sma = window.iter().sum::<Decimal>() / Decimal::from(n);
        if *close > sma {
            vec![(self.config.symbol.clone(), Decimal::ONE)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn bees() -> Symbol {
        Symbol("NSE:NIFTYBEES".into())
    }

    /// Feeds month-end closes one call at a time; returns whether each call held the fund.
    fn holds(closes: &[Decimal]) -> Vec<bool> {
        let mut f = IndexTrendFilter::new(IndexTrendFilterConfig { months: 3, ..IndexTrendFilterConfig::faber(bees()) }).unwrap();
        let mut h = History::new();
        closes
            .iter()
            .map(|c| {
                h.entry(bees()).or_default().push(*c);
                !f.targets(0, &[bees()], &h).is_empty()
            })
            .collect()
    }

    #[test]
    fn faber_uses_ten_months() {
        assert_eq!(IndexTrendFilterConfig::faber(bees()).months, 10);
    }

    #[test]
    fn holds_only_above_the_moving_average_after_warm_up() {
        // SMA3 at month 3 = 100: 102 is above. Month 4: (100+102+90)/3 = 97.33, 90 below.
        // Month 5: (102+90+120)/3 = 104, 120 above. Equal to the average is not above.
        assert_eq!(holds(&[dec!(98), dec!(100), dec!(102), dec!(90), dec!(120)]), vec![false, false, true, false, true]);
        assert_eq!(holds(&[dec!(100), dec!(100), dec!(100)]), vec![false, false, false]);
    }

    #[test]
    fn full_weight_in_the_fund_only() {
        let mut f = IndexTrendFilter::new(IndexTrendFilterConfig { months: 1, ..IndexTrendFilterConfig::faber(bees()) }).unwrap();
        let h: History = [(bees(), vec![dec!(1), dec!(2)])].into();
        // SMA1 equals the close, so never above: months = 1 always holds cash.
        assert!(f.targets(0, &[bees()], &h).is_empty());
        assert!(IndexTrendFilter::new(IndexTrendFilterConfig { months: 0, ..IndexTrendFilterConfig::faber(bees()) }).is_err());
    }

    #[test]
    fn cash_when_the_fund_has_no_history_or_is_not_offered() {
        let mut f = IndexTrendFilter::new(IndexTrendFilterConfig { months: 1, ..IndexTrendFilterConfig::faber(bees()) }).unwrap();
        assert!(f.targets(0, &[bees()], &History::new()).is_empty());
        let h: History = [(bees(), vec![dec!(1)])].into();
        assert!(f.targets(0, &[], &h).is_empty());
    }
}

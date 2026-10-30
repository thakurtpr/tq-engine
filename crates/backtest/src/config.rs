//! Run configuration: starting cash, position sizing, slippage, entry priority and warm-up.

use rust_decimal::Decimal;
use tq_core::{Price, Qty, Side, Timestamp};
use tq_risk::size_for_risk;

use crate::priority::Priority;
use crate::slippage::{LiquidityStats, SlippageModel};
use crate::BacktestError;

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;
const BPS_PER_UNIT: Decimal = Decimal::from_parts(10_000, 0, 0, false, 0);
/// Percent of equity risked per trade by default. A CEO placeholder: the user has not chosen it.
pub const DEFAULT_RISK_PCT: Decimal = Decimal::from_parts(5, 0, 0, false, 1);
/// Slippage per side, in basis points, from `docs/validation-protocol.md` (NIFTY 50 names).
pub const DEFAULT_SLIPPAGE_BPS: Decimal = Decimal::from_parts(5, 0, 0, false, 0);

/// How the engine sizes an entry when it fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sizing {
    /// Keep the strategy's quantity (scenario tests that script exact sizes).
    AsSignalled,
    /// Whole shares so that hitting the stop loses at most this percent of current equity,
    /// then cut down to the risk gate's notional cap.
    RiskPct(Decimal),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunConfig {
    pub initial_cash: Decimal,
    pub sizing: Sizing,
    /// Base adverse slippage per side: buys fill this many bps above, sells below, the raw price.
    pub slippage_bps: Decimal,
    /// The liquidity-aware model of `docs/slippage-model.md` (impact, stop 2x, ADV cap, tick).
    /// `None`: flat `slippage_bps` on every fill (scenario tests and reproducing old reports).
    pub slippage_model: Option<SlippageModel>,
    /// Stress multiplier on the whole slippage (1, 2 or 3 in the protocol's cost stress).
    pub slippage_mult: Decimal,
    /// Order in which one day's entry signals compete for slots and cash.
    pub priority: Priority,
    /// Bars before this timestamp only warm up the strategy: no orders, no equity points.
    pub trade_from: Timestamp,
    /// Half-open `[from, to)` timestamp windows in which an entry may FILL (e.g. index membership).
    /// `None` allows entries at any time. Exits are never restricted.
    pub entry_windows: Option<Vec<(Timestamp, Timestamp)>>,
}

impl RunConfig {
    /// Risk-based sizing at [`DEFAULT_RISK_PCT`] and [`DEFAULT_SLIPPAGE_BPS`], no warm-up.
    pub fn new(initial_cash: Decimal) -> Self {
        Self {
            initial_cash,
            sizing: Sizing::RiskPct(DEFAULT_RISK_PCT),
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            slippage_model: None,
            slippage_mult: Decimal::ONE,
            priority: Priority::Alphabetical,
            trade_from: Timestamp::MIN,
            entry_windows: None,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), BacktestError> {
        if self.initial_cash <= Decimal::ZERO {
            return Err(BacktestError::NonPositiveCash);
        }
        if matches!(self.sizing, Sizing::RiskPct(p) if p <= Decimal::ZERO) {
            return Err(BacktestError::InvalidConfig("risk percent must be positive"));
        }
        if self.slippage_bps < Decimal::ZERO {
            return Err(BacktestError::InvalidConfig("slippage must not be negative"));
        }
        if self.slippage_mult <= Decimal::ZERO {
            return Err(BacktestError::InvalidConfig("slippage multiplier must be positive"));
        }
        Ok(())
    }

    /// `raw` moved against the trader: base bps (or the impact model for large orders, 2x for
    /// stop exits), times the stress multiplier, then rounded to the tick under the model.
    pub(crate) fn fill_price(&self, raw: Price, side: Side, qty: Qty, liq: &LiquidityStats, stop_exit: bool) -> Price {
        let bps = match &self.slippage_model {
            Some(m) => m.bps(self.slippage_bps, qty, liq, stop_exit),
            None => self.slippage_bps,
        };
        let frac = bps * self.slippage_mult / BPS_PER_UNIT;
        let price = match side {
            Side::Buy => raw * (Decimal::ONE + frac),
            Side::Sell => raw * (Decimal::ONE - frac),
        };
        self.slippage_model.as_ref().map_or(price, |m| m.round_to_tick(price, side))
    }

    /// True when the model's impact term (not the flat base) applies to this fill.
    pub(crate) fn above_impact_threshold(&self, qty: Qty, liq: &LiquidityStats) -> bool {
        self.slippage_model.as_ref().is_some_and(|m| m.above_threshold(qty, liq))
    }

    /// Largest entry the liquidity cap allows; `None` without a model (no cap).
    pub(crate) fn adv_cap(&self, liq: &LiquidityStats) -> Option<Qty> {
        self.slippage_model.as_ref().map(|m| m.adv_cap(liq))
    }
}

/// Risk-budget size at `fill_price`, cut to `max_notional_pct` of `equity`. Zero when neither
/// allows a whole share.
pub(crate) fn risk_sized_qty(equity: Decimal, risk_pct: Decimal, fill_price: Price, stop: Price, max_notional_pct: Decimal) -> Qty {
    if fill_price <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let by_risk = size_for_risk(equity, risk_pct, fill_price, stop);
    let by_cap = (equity * max_notional_pct / HUNDRED / fill_price).floor();
    by_risk.min(by_cap).max(Decimal::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn default_constants() {
        assert_eq!((DEFAULT_RISK_PCT, DEFAULT_SLIPPAGE_BPS), (dec!(0.5), dec!(5)));
    }

    #[test]
    fn risk_size_is_min_of_risk_budget_and_notional_cap() {
        assert_eq!(risk_sized_qty(dec!(100000), dec!(0.5), dec!(100), dec!(95), dec!(20)), dec!(100));
        assert_eq!(risk_sized_qty(dec!(100000), dec!(0.5), dec!(100), dec!(99.9), dec!(20)), dec!(200));
        assert_eq!(risk_sized_qty(dec!(100000), dec!(0.5), dec!(0), dec!(-1), dec!(20)), dec!(0));
    }

    fn liq(adv: Decimal, sigma: f64) -> LiquidityStats {
        LiquidityStats { adv: Some(adv), sigma: Some(sigma) }
    }

    #[test]
    fn flat_fill_price_scales_with_the_multiplier_and_skips_ticks() {
        let c = RunConfig { slippage_mult: dec!(3), ..RunConfig::new(dec!(1)) };
        assert_eq!(c.fill_price(dec!(100), Side::Buy, dec!(1), &liq(dec!(1), 0.5), true), dec!(100.15));
        assert_eq!(c.adv_cap(&liq(dec!(1000), 0.01)), None);
    }

    #[test]
    fn model_fill_price_doubles_on_stops_and_rounds_against_the_trader() {
        let c = RunConfig { slippage_model: Some(SlippageModel::default()), ..RunConfig::new(dec!(1)) };
        let l = liq(dec!(1000000), 0.015);
        // 5 bps on 1001 = 1001.5005 -> tick up to 1001.55; a stop sell pays 10 bps: 999.999 -> 999.95.
        assert_eq!(c.fill_price(dec!(1001), Side::Buy, dec!(10), &l, false), dec!(1001.55));
        assert_eq!(c.fill_price(dec!(1001), Side::Sell, dec!(10), &l, true), dec!(999.95));
        assert_eq!(c.adv_cap(&l), Some(dec!(10000)));
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        let c = RunConfig { slippage_mult: dec!(0), ..RunConfig::new(dec!(1)) };
        assert!(c.validate().is_err());
    }
}

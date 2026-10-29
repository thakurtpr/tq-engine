//! Run configuration: starting cash, position sizing, slippage and the warm-up cut-off.

use rust_decimal::Decimal;
use tq_core::{Price, Qty, Side, Timestamp};
use tq_risk::size_for_risk;

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
    /// Adverse slippage per side: buys fill this many bps above, sells below, the raw price.
    pub slippage_bps: Decimal,
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
        Ok(())
    }

    /// Whether an entry may fill at `ts`.
    pub(crate) fn entry_allowed(&self, ts: Timestamp) -> bool {
        self.entry_windows.as_ref().is_none_or(|w| w.iter().any(|&(from, to)| from <= ts && ts < to))
    }

    /// `price` moved against the trader by the slippage.
    pub(crate) fn slipped(&self, price: Price, side: Side) -> Price {
        let frac = self.slippage_bps / BPS_PER_UNIT;
        match side {
            Side::Buy => price * (Decimal::ONE + frac),
            Side::Sell => price * (Decimal::ONE - frac),
        }
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
}

//! Gate for buys of a cross-sectional rebalancing strategy (trade to target weights).
//!
//! These positions have no stop: an unstopped long can lose its whole notional, so the per-trade
//! risk check (which needs a stop) does not apply. Risk is bounded instead by the notional cap on
//! the RESULTING position, the position count, the daily loss and drawdown limits, and the kill
//! switch. Sells are never gated (exits must always be possible).

use rust_decimal::Decimal;
use tq_core::{OrderRequest, Price, Qty, Side, Symbol};

use crate::{AccountState, ApprovedOrder, RiskGate, RiskViolation, HUNDRED};

/// Stop recorded on an approved rebalance buy: Rs 0.01, i.e. effectively no stop, so the order's
/// `risk_amount()` is (almost) its whole notional. Brokers ignore the stop.
pub const NO_STOP: Price = Decimal::from_parts(1, 0, 0, false, 2);

impl RiskGate {
    /// Approves buying `qty` at `price` on top of `held` shares already owned.
    pub fn check_rebalance_buy(
        &self,
        symbol: Symbol,
        qty: Qty,
        price: Price,
        held: Qty,
        acct: &AccountState,
    ) -> Result<ApprovedOrder, RiskViolation> {
        if acct.kill_switch {
            return Err(RiskViolation::KillSwitch);
        }
        if acct.equity <= Decimal::ZERO {
            return Err(RiskViolation::NoEquity);
        }
        self.check_loss_limits(acct)?;
        if held.is_zero() && acct.open_positions >= self.limits.max_open_positions {
            return Err(RiskViolation::TooManyPositions);
        }
        let order = OrderRequest::new(symbol, Side::Buy, qty, price, NO_STOP).map_err(RiskViolation::InvalidOrder)?;
        let limit = acct.equity * self.limits.max_position_notional_pct / HUNDRED;
        let notional = (held + qty) * price;
        if notional > limit {
            return Err(RiskViolation::PositionSize { actual: notional, limit });
        }
        Ok(ApprovedOrder(order))
    }
}

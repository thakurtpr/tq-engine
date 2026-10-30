//! Hard risk gate. Every order passes `RiskGate::check` before it reaches any broker.
//! Strategies and agents cannot bypass it: brokers only accept `ApprovedOrder`, which only
//! this crate can construct.

use rust_decimal::Decimal;
use thiserror::Error;
use tq_core::{CoreError, OrderRequest, Price, Qty, Side};

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RiskViolation {
    #[error("kill switch is engaged")]
    KillSwitch,
    #[error("risk per trade {actual} exceeds limit {limit}")]
    TradeRisk { actual: Decimal, limit: Decimal },
    #[error("daily loss limit reached")]
    DailyLoss,
    #[error("max drawdown reached")]
    Drawdown,
    #[error("max open positions reached")]
    TooManyPositions,
    #[error("position notional {actual} exceeds limit {limit}")]
    PositionSize { actual: Decimal, limit: Decimal },
    #[error("account equity must be positive")]
    NoEquity,
    #[error("short selling is not allowed: entries must be buys (long-only, no leverage)")]
    ShortSelling,
    #[error("fill price is at or through the stop (gap); entry skipped")]
    GapThroughStop,
    #[error("invalid order at fill: {0}")]
    InvalidOrder(CoreError),
    #[error("invalid account state: {0}")]
    InvalidAccount(&'static str),
    #[error("risk limits are placeholders; the user must set and confirm them before live trading")]
    LimitsNotConfirmed,
}

/// Limits in percent of equity. Risk per trade (0.5%) and the drawdown kill switch (10%) are the
/// user's decision of 2026-10-30; the other limits are still placeholders. `user_confirmed`
/// stays false until the human confirms the full set for live use.
#[derive(Debug, Clone, PartialEq)]
pub struct RiskLimits {
    pub max_risk_per_trade_pct: Decimal,
    pub max_daily_loss_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub max_position_notional_pct: Decimal,
    pub max_open_positions: usize,
    /// UNSET-BY-USER until the human sets real values and flips this to true.
    pub user_confirmed: bool,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_risk_per_trade_pct: Decimal::from_parts(5, 0, 0, false, 1),
            max_daily_loss_pct: Decimal::TWO,
            max_drawdown_pct: Decimal::TEN,
            max_position_notional_pct: Decimal::from(20),
            max_open_positions: 5,
            user_confirmed: false,
        }
    }
}

impl RiskLimits {
    /// Live trading must call this at startup and refuse to run on error.
    pub fn ensure_live_ready(&self) -> Result<(), RiskViolation> {
        if self.user_confirmed {
            Ok(())
        } else {
            Err(RiskViolation::LimitsNotConfirmed)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountState {
    pub equity: Decimal,
    pub start_of_day_equity: Decimal,
    pub peak_equity: Decimal,
    pub open_positions: usize,
    pub kill_switch: bool,
}

impl AccountState {
    /// Validated constructor: equity and start-of-day equity positive, peak at least equity.
    /// Starts with no open positions and the kill switch off.
    pub fn new(equity: Decimal, start_of_day_equity: Decimal, peak_equity: Decimal) -> Result<Self, RiskViolation> {
        if equity <= Decimal::ZERO {
            return Err(RiskViolation::InvalidAccount("equity must be positive"));
        }
        if start_of_day_equity <= Decimal::ZERO {
            return Err(RiskViolation::InvalidAccount("start-of-day equity must be positive"));
        }
        if peak_equity < equity {
            return Err(RiskViolation::InvalidAccount("peak equity below equity"));
        }
        Ok(Self { equity, start_of_day_equity, peak_equity, open_positions: 0, kill_switch: false })
    }

    /// A flat account where equity is also the day's start and the peak.
    pub fn fresh(equity: Decimal) -> Result<Self, RiskViolation> {
        Self::new(equity, equity, equity)
    }
}

/// Whole shares (rounded down) such that hitting `stop` loses at most `risk_pct`% of `equity`.
/// Returns zero for non-positive equity or risk, or when entry equals stop.
pub fn size_for_risk(equity: Decimal, risk_pct: Decimal, entry: Price, stop: Price) -> Qty {
    let per_share = (entry - stop).abs();
    if equity <= Decimal::ZERO || risk_pct <= Decimal::ZERO || per_share.is_zero() {
        return Decimal::ZERO;
    }
    (equity * risk_pct / HUNDRED / per_share).floor()
}

/// An order that passed the gate. The field is private so only `RiskGate` can create one.
/// It is deliberately not `Clone`: brokers consume it, so one approval fills at most once.
///
/// ```compile_fail
/// fn needs_clone<T: Clone>() {}
/// needs_clone::<tq_risk::ApprovedOrder>();
/// ```
#[derive(Debug, PartialEq)]
pub struct ApprovedOrder(OrderRequest);

impl ApprovedOrder {
    pub fn order(&self) -> &OrderRequest {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct RiskGate {
    limits: RiskLimits,
}

impl RiskGate {
    pub fn new(limits: RiskLimits) -> Self {
        Self { limits }
    }

    pub fn limits(&self) -> &RiskLimits {
        &self.limits
    }

    pub fn check(&self, order: OrderRequest, acct: &AccountState) -> Result<ApprovedOrder, RiskViolation> {
        if acct.kill_switch {
            return Err(RiskViolation::KillSwitch);
        }
        if order.side == Side::Sell {
            return Err(RiskViolation::ShortSelling);
        }
        if acct.equity <= Decimal::ZERO {
            return Err(RiskViolation::NoEquity);
        }
        self.check_account(acct)?;
        self.check_order(&order, acct)?;
        Ok(ApprovedOrder(order))
    }

    /// Re-checks an approved order at the price it will actually fill at (e.g. the next open).
    /// Rejects fills at or through the stop and re-applies the per-order risk and size limits.
    pub fn reprice_at_fill(
        &self,
        approved: ApprovedOrder,
        fill_price: Price,
        acct: &AccountState,
    ) -> Result<ApprovedOrder, RiskViolation> {
        let qty = approved.0.qty;
        self.resize_at_fill(approved, qty, fill_price, acct)
    }

    /// Like [`Self::reprice_at_fill`], but also replaces the quantity, e.g. with a size computed
    /// from the account's risk budget at the fill price. The new size passes the same checks.
    pub fn resize_at_fill(
        &self,
        approved: ApprovedOrder,
        qty: Qty,
        fill_price: Price,
        acct: &AccountState,
    ) -> Result<ApprovedOrder, RiskViolation> {
        let o = approved.0;
        let gapped_through_stop = match o.side {
            Side::Buy => fill_price <= o.stop,
            Side::Sell => fill_price >= o.stop,
        };
        if gapped_through_stop {
            return Err(RiskViolation::GapThroughStop);
        }
        let resized = OrderRequest::new(o.symbol, o.side, qty, fill_price, o.stop).map_err(RiskViolation::InvalidOrder)?;
        self.check_order(&resized, acct)?;
        Ok(ApprovedOrder(resized))
    }

    /// True once equity has fallen `max_drawdown_pct` from its peak. Engines use this to
    /// engage the kill switch automatically.
    pub fn drawdown_breached(&self, acct: &AccountState) -> bool {
        let drawdown = acct.peak_equity - acct.equity;
        drawdown >= acct.peak_equity * self.limits.max_drawdown_pct / HUNDRED
    }

    fn check_account(&self, acct: &AccountState) -> Result<(), RiskViolation> {
        let l = &self.limits;
        let day_loss = acct.start_of_day_equity - acct.equity;
        if day_loss >= acct.start_of_day_equity * l.max_daily_loss_pct / HUNDRED {
            return Err(RiskViolation::DailyLoss);
        }
        if self.drawdown_breached(acct) {
            return Err(RiskViolation::Drawdown);
        }
        if acct.open_positions >= l.max_open_positions {
            return Err(RiskViolation::TooManyPositions);
        }
        Ok(())
    }

    fn check_order(&self, order: &OrderRequest, acct: &AccountState) -> Result<(), RiskViolation> {
        let l = &self.limits;
        let risk_limit = acct.equity * l.max_risk_per_trade_pct / HUNDRED;
        let risk = order.risk_amount();
        if risk > risk_limit {
            return Err(RiskViolation::TradeRisk { actual: risk, limit: risk_limit });
        }
        let notional_limit = acct.equity * l.max_position_notional_pct / HUNDRED;
        let notional = order.notional();
        if notional > notional_limit {
            return Err(RiskViolation::PositionSize { actual: notional, limit: notional_limit });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

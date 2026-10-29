//! Hard risk gate. Every order passes `RiskGate::check` before it reaches any broker.
//! Strategies and agents cannot bypass it: brokers only accept `ApprovedOrder`, which only
//! this crate can construct.

use rust_decimal::Decimal;
use thiserror::Error;
use tq_core::OrderRequest;

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
    #[error("risk limits are placeholders; the user must set and confirm them before live trading")]
    LimitsNotConfirmed,
}

/// Limits in percent of equity. Defaults are conservative PLACEHOLDERS, not the user's choice.
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
            max_risk_per_trade_pct: Decimal::ONE,
            max_daily_loss_pct: Decimal::TWO,
            max_drawdown_pct: Decimal::from(5),
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

/// An order that passed the gate. The field is private so only `RiskGate` can create one.
#[derive(Debug, Clone, PartialEq)]
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
        if acct.equity <= Decimal::ZERO {
            return Err(RiskViolation::NoEquity);
        }
        self.check_account(acct)?;
        self.check_order(&order, acct)?;
        Ok(ApprovedOrder(order))
    }

    fn check_account(&self, acct: &AccountState) -> Result<(), RiskViolation> {
        let l = &self.limits;
        let day_loss = acct.start_of_day_equity - acct.equity;
        if day_loss >= acct.start_of_day_equity * l.max_daily_loss_pct / HUNDRED {
            return Err(RiskViolation::DailyLoss);
        }
        let drawdown = acct.peak_equity - acct.equity;
        if drawdown >= acct.peak_equity * l.max_drawdown_pct / HUNDRED {
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
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::{Side, Symbol};

    fn acct() -> AccountState {
        AccountState {
            equity: dec!(100000),
            start_of_day_equity: dec!(100000),
            peak_equity: dec!(100000),
            open_positions: 0,
            kill_switch: false,
        }
    }

    fn buy(qty: Decimal, entry: Decimal, stop: Decimal) -> OrderRequest {
        OrderRequest::new(Symbol("NSE:TCS".into()), Side::Buy, qty, entry, stop).unwrap()
    }

    fn gate() -> RiskGate {
        RiskGate::new(RiskLimits::default())
    }

    #[test]
    fn approves_order_within_all_limits() {
        // risk 10 * 50 = 500 (0.5%), notional 10 * 1000 = 10_000 (10%)
        assert!(gate().check(buy(dec!(10), dec!(1000), dec!(950)), &acct()).is_ok());
    }

    #[test]
    fn rejects_when_trade_risk_exceeds_one_percent() {
        // risk 10 * 150 = 1500 > 1000
        let r = gate().check(buy(dec!(10), dec!(1000), dec!(850)), &acct());
        assert_eq!(r, Err(RiskViolation::TradeRisk { actual: dec!(1500), limit: dec!(1000) }));
    }

    #[test]
    fn rejects_oversized_position_even_with_tight_stop() {
        // notional 30 * 1000 = 30_000 > 20_000; risk only 30
        let r = gate().check(buy(dec!(30), dec!(1000), dec!(999)), &acct());
        assert!(matches!(r, Err(RiskViolation::PositionSize { .. })));
    }

    #[test]
    fn rejects_everything_when_kill_switch_engaged() {
        let a = AccountState { kill_switch: true, ..acct() };
        assert_eq!(gate().check(buy(dec!(1), dec!(100), dec!(99)), &a), Err(RiskViolation::KillSwitch));
    }

    #[test]
    fn stops_trading_after_daily_loss_limit() {
        let a = AccountState { equity: dec!(98000), peak_equity: dec!(100000), ..acct() };
        assert_eq!(gate().check(buy(dec!(1), dec!(100), dec!(99)), &a), Err(RiskViolation::DailyLoss));
    }

    #[test]
    fn stops_trading_at_max_drawdown_from_peak() {
        let a = AccountState {
            equity: dec!(95000),
            start_of_day_equity: dec!(95500),
            peak_equity: dec!(100000),
            ..acct()
        };
        assert_eq!(gate().check(buy(dec!(1), dec!(100), dec!(99)), &a), Err(RiskViolation::Drawdown));
    }

    #[test]
    fn caps_number_of_open_positions() {
        let a = AccountState { open_positions: 5, ..acct() };
        assert_eq!(gate().check(buy(dec!(1), dec!(100), dec!(99)), &a), Err(RiskViolation::TooManyPositions));
    }

    #[test]
    fn rejects_when_equity_not_positive() {
        let a = AccountState { equity: dec!(0), ..acct() };
        assert_eq!(gate().check(buy(dec!(1), dec!(100), dec!(99)), &a), Err(RiskViolation::NoEquity));
    }

    #[test]
    fn placeholder_limits_block_live_trading() {
        assert_eq!(RiskLimits::default().ensure_live_ready(), Err(RiskViolation::LimitsNotConfirmed));
    }

    #[test]
    fn confirmed_limits_allow_live_startup() {
        let l = RiskLimits { user_confirmed: true, ..RiskLimits::default() };
        assert!(l.ensure_live_ready().is_ok());
    }
}

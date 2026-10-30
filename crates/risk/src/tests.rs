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
fn defaults_are_the_users_decision_but_not_confirmed() {
    // User decision 2026-10-30: 0.5% risk per trade, 10% drawdown kill switch. Others unchanged.
    let l = RiskLimits::default();
    assert_eq!((l.max_risk_per_trade_pct, l.max_drawdown_pct), (dec!(0.5), dec!(10)));
    assert_eq!((l.max_daily_loss_pct, l.max_position_notional_pct, l.max_open_positions), (dec!(2), dec!(20), 5));
    assert!(!l.user_confirmed);
}

#[test]
fn approves_order_within_all_limits() {
    // risk 10 * 50 = 500 (exactly 0.5%), notional 10 * 1000 = 10_000 (10%)
    assert!(gate().check(buy(dec!(10), dec!(1000), dec!(950)), &acct()).is_ok());
}

#[test]
fn rejects_when_trade_risk_exceeds_half_a_percent() {
    // risk 10 * 51 = 510 > 500
    let r = gate().check(buy(dec!(10), dec!(1000), dec!(949)), &acct());
    assert_eq!(r, Err(RiskViolation::TradeRisk { actual: dec!(510), limit: dec!(500) }));
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
        equity: dec!(90000),
        start_of_day_equity: dec!(90500),
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
fn size_for_risk_rounds_down_to_whole_shares() {
    // 1% of 100_000 = 1000 risk; stop distance 30 -> 33.33 -> 33 shares
    assert_eq!(size_for_risk(dec!(100000), dec!(1), dec!(1000), dec!(970)), dec!(33));
}

#[test]
fn size_for_risk_is_zero_for_degenerate_inputs() {
    assert_eq!(size_for_risk(dec!(100000), dec!(1), dec!(100), dec!(100)), dec!(0));
    assert_eq!(size_for_risk(dec!(0), dec!(1), dec!(100), dec!(95)), dec!(0));
    assert_eq!(size_for_risk(dec!(100000), dec!(-1), dec!(100), dec!(95)), dec!(0));
}

#[test]
fn size_for_risk_handles_short_side_stop() {
    assert_eq!(size_for_risk(dec!(10000), dec!(1), dec!(100), dec!(102)), dec!(50));
}

#[test]
fn drawdown_breach_detected_at_limit() {
    let g = gate();
    assert!(g.drawdown_breached(&AccountState { equity: dec!(90000), ..acct() }));
    assert!(!g.drawdown_breached(&AccountState { equity: dec!(90001), ..acct() }));
}

#[test]
fn fresh_account_starts_flat_at_its_equity() {
    assert_eq!(AccountState::fresh(dec!(100000)), Ok(acct()));
}

#[test]
fn account_new_validates_invariants() {
    let bad = |e, sod, peak| AccountState::new(e, sod, peak).is_err();
    assert!(bad(dec!(0), dec!(100), dec!(100)));
    assert!(bad(dec!(100), dec!(0), dec!(100)));
    assert!(bad(dec!(100), dec!(100), dec!(99)));
    assert!(AccountState::new(dec!(90), dec!(95), dec!(100)).is_ok());
}

#[test]
fn rejects_sell_entries_long_only() {
    let o = OrderRequest::new(Symbol("NSE:TCS".into()), Side::Sell, dec!(1), dec!(100), dec!(101)).unwrap();
    assert_eq!(gate().check(o, &acct()), Err(RiskViolation::ShortSelling));
}

#[test]
fn reprice_rejects_fill_at_or_below_stop() {
    let approved = gate().check(buy(dec!(10), dec!(1000), dec!(950)), &acct()).unwrap();
    assert_eq!(gate().reprice_at_fill(approved, dec!(950), &acct()), Err(RiskViolation::GapThroughStop));
}

#[test]
fn reprice_rechecks_risk_at_actual_fill_price() {
    // risk at close 10 * 50 = 500 ok; gap up to 1200 -> 10 * 250 = 2500 > 500
    let approved = gate().check(buy(dec!(10), dec!(1000), dec!(950)), &acct()).unwrap();
    let r = gate().reprice_at_fill(approved, dec!(1200), &acct());
    assert_eq!(r, Err(RiskViolation::TradeRisk { actual: dec!(2500), limit: dec!(500) }));
}

#[test]
fn reprice_keeps_order_within_limits_at_fill_price() {
    // 8 shares: risk 400 at 1000, 480 at 1010, both inside 0.5% (500).
    let approved = gate().check(buy(dec!(8), dec!(1000), dec!(950)), &acct()).unwrap();
    let repriced = gate().reprice_at_fill(approved, dec!(1010), &acct()).unwrap();
    assert_eq!(repriced.order().entry, dec!(1010));
}

#[test]
fn resize_replaces_qty_and_price_and_rechecks_limits() {
    let approved = gate().check(buy(dec!(1), dec!(1000), dec!(950)), &acct()).unwrap();
    // 9 shares at 1005, stop 950: risk 495 <= 500.
    let resized = gate().resize_at_fill(approved, dec!(9), dec!(1005), &acct()).unwrap();
    assert_eq!((resized.order().qty, resized.order().entry), (dec!(9), dec!(1005)));
}

#[test]
fn resize_rejects_qty_that_breaks_notional_cap() {
    // 25 * 1000 = 25_000 > 20% of 100_000; risk 25 * 10 = 250 is fine.
    let approved = gate().check(buy(dec!(1), dec!(1000), dec!(990)), &acct()).unwrap();
    let r = gate().resize_at_fill(approved, dec!(25), dec!(1000), &acct());
    assert!(matches!(r, Err(RiskViolation::PositionSize { .. })), "{r:?}");
}

#[test]
fn resize_rejects_gap_through_stop_and_zero_qty() {
    let g = gate();
    let approved = g.check(buy(dec!(1), dec!(1000), dec!(950)), &acct()).unwrap();
    assert_eq!(g.resize_at_fill(approved, dec!(5), dec!(940), &acct()), Err(RiskViolation::GapThroughStop));
    let approved = g.check(buy(dec!(1), dec!(1000), dec!(950)), &acct()).unwrap();
    let r = g.resize_at_fill(approved, dec!(0), dec!(1000), &acct());
    assert!(matches!(r, Err(RiskViolation::InvalidOrder(_))), "{r:?}");
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

fn rebuy(qty: Decimal, price: Decimal, held: Decimal, a: &AccountState) -> Result<ApprovedOrder, RiskViolation> {
    gate().check_rebalance_buy(Symbol("NSE:TCS".into()), qty, price, held, a)
}

#[test]
fn rebalance_buy_within_notional_cap_is_approved_without_a_stop() {
    // 20 * 1000 = 20_000 = 20% of equity: at the cap, allowed. No stop -> whole notional at risk.
    let approved = rebuy(dec!(20), dec!(1000), dec!(0), &acct()).unwrap();
    assert_eq!((approved.order().qty, approved.order().entry), (dec!(20), dec!(1000)));
    assert_eq!(approved.order().stop, NO_STOP);
}

#[test]
fn rebalance_buy_caps_the_resulting_position_not_just_the_order() {
    let r = rebuy(dec!(5), dec!(1000), dec!(16), &acct());
    assert!(matches!(r, Err(RiskViolation::PositionSize { .. })), "{r:?}");
}

#[test]
fn rebalance_buy_counts_positions_only_when_opening_one() {
    let full = AccountState { open_positions: 5, ..acct() };
    assert_eq!(rebuy(dec!(1), dec!(100), dec!(0), &full), Err(RiskViolation::TooManyPositions));
    assert!(rebuy(dec!(1), dec!(100), dec!(3), &full).is_ok(), "topping up an existing holding");
}

#[test]
fn rebalance_buy_respects_kill_switch_and_loss_limits() {
    assert_eq!(rebuy(dec!(1), dec!(100), dec!(0), &AccountState { kill_switch: true, ..acct() }), Err(RiskViolation::KillSwitch));
    let down = AccountState { equity: dec!(98000), peak_equity: dec!(100000), ..acct() };
    assert_eq!(rebuy(dec!(1), dec!(100), dec!(0), &down), Err(RiskViolation::DailyLoss));
    let dd = AccountState { equity: dec!(90000), start_of_day_equity: dec!(90000), ..acct() };
    assert_eq!(rebuy(dec!(1), dec!(100), dec!(0), &dd), Err(RiskViolation::Drawdown));
    assert!(matches!(rebuy(dec!(0), dec!(100), dec!(0), &acct()), Err(RiskViolation::InvalidOrder(_))));
}

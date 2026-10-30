//! Shared-account portfolio: one cash balance and one risk state across symbols.

use std::collections::HashMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_backtest::{run_portfolio, BacktestError, ExitReason, Priority, RunConfig, RunReport, Series, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, OrderRequest, Side, Symbol, Timestamp};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategy::{Signal, Strategy};

const CASH: Decimal = dec!(100000);

/// Emits scripted signals keyed by (symbol, bar timestamp).
#[derive(Default)]
struct Script(HashMap<(String, Timestamp), Vec<Signal>>);

impl Script {
    fn at(mut self, sym: &str, ts: Timestamp, signal: Signal) -> Self {
        self.0.entry((sym.to_string(), ts)).or_default().push(signal);
        self
    }
}

impl Strategy for Script {
    fn name(&self) -> &str {
        "script"
    }
    fn on_bar(&mut self, s: &Symbol, b: &Bar) -> Vec<Signal> {
        self.0.remove(&(s.0.clone(), b.ts)).unwrap_or_default()
    }
}

fn sym(s: &str) -> Symbol {
    Symbol(s.into())
}

fn bar(ts: Timestamp, o: Decimal, l: Decimal, c: Decimal) -> Bar {
    Bar::new(ts, o, o.max(c), l, c, dec!(1000000)).unwrap()
}

fn flat(ts: Timestamp, p: Decimal) -> Bar {
    bar(ts, p, p, p)
}

fn flats(from: Timestamp, to: Timestamp, p: Decimal) -> Vec<Bar> {
    (from..to).map(|ts| flat(ts, p)).collect()
}

fn series(s: &str, bars: Vec<Bar>) -> Series {
    Series { symbol: sym(s), bars, entry_windows: None }
}

fn enter(s: &str, qty: Decimal, entry: Decimal, stop: Decimal) -> Signal {
    Signal::Enter(OrderRequest::new(sym(s), Side::Buy, qty, entry, stop).unwrap())
}

fn scripted() -> RunConfig {
    RunConfig { sizing: Sizing::AsSignalled, slippage_bps: dec!(0), ..RunConfig::new(CASH) }
}

fn limits(max_positions: usize) -> RiskLimits {
    RiskLimits { max_open_positions: max_positions, max_position_notional_pct: dec!(100), max_risk_per_trade_pct: dec!(10), ..RiskLimits::default() }
}

fn go(script: Script, limits: RiskLimits, config: &RunConfig, series: &[Series]) -> RunReport {
    let mut s = script;
    run_portfolio(&mut s, &mut Paper::default(), &RiskGate::new(limits), config, series).unwrap()
}

fn traded(r: &RunReport) -> Vec<String> {
    r.trades.iter().map(|t| t.symbol.0.clone()).collect()
}

#[test]
fn position_cap_binds_across_symbols_alphabetically() {
    let s = Script::default().at("B", 0, enter("B", dec!(10), dec!(100), dec!(95))).at("A", 0, enter("A", dec!(10), dec!(100), dec!(95)));
    let data = [series("B", flats(0, 3, dec!(100))), series("A", flats(0, 3, dec!(100)))];
    let r = go(s, limits(1), &scripted(), &data);
    assert_eq!(r.open_positions_curve, vec![0, 1, 1]);
    assert_eq!(r.rejections, vec!["risk: max open positions reached".to_string()]);
    assert_eq!(r.orders_filled, 1);
}

#[test]
fn random_priority_lets_either_symbol_win_depending_on_seed() {
    let mut winners = std::collections::BTreeSet::new();
    for seed in 0..20 {
        let s = Script::default().at("A", 0, enter("A", dec!(10), dec!(100), dec!(95))).at("B", 0, enter("B", dec!(10), dec!(100), dec!(95)))
            .at("A", 1, Signal::Exit(sym("A"))).at("B", 1, Signal::Exit(sym("B")));
        let config = RunConfig { priority: Priority::Random { seed }, ..scripted() };
        let r = go(s, limits(1), &config, &[series("A", flats(0, 3, dec!(100))), series("B", flats(0, 3, dec!(100)))]);
        winners.extend(traded(&r));
    }
    assert_eq!(winners.into_iter().collect::<Vec<_>>(), ["A", "B"]);
}

#[test]
fn cash_is_shared_so_a_second_entry_can_be_unaffordable() {
    // A takes 80_000 of the 100_000; B wants another 50_000.
    let s = Script::default().at("A", 0, enter("A", dec!(800), dec!(100), dec!(95))).at("B", 0, enter("B", dec!(500), dec!(100), dec!(95)));
    let r = go(s, limits(5), &scripted(), &[series("A", flats(0, 3, dec!(100))), series("B", flats(0, 3, dec!(100)))]);
    assert_eq!(r.open_positions_curve.last(), Some(&1));
    assert!(r.rejections.iter().any(|m| m == "insufficient cash for B"), "{:?}", r.rejections);
}

#[test]
fn a_loss_in_one_symbol_trips_the_kill_switch_for_all() {
    // A: 900 sh, stop 90; close 93 marks a 6.3% drawdown. B's later entry is blocked.
    let a = vec![flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(99), dec!(93), dec!(93)), flat(3, dec!(93)), flat(4, dec!(93))];
    let s = Script::default().at("A", 0, enter("A", dec!(900), dec!(100), dec!(90))).at("B", 3, enter("B", dec!(10), dec!(100), dec!(95)));
    let kill_at_5 = RiskLimits { max_drawdown_pct: dec!(5), ..limits(5) };
    let r = go(s, kill_at_5, &scripted(), &[series("A", a), series("B", flats(0, 5, dec!(100)))]);
    assert_eq!(r.kill_switch_engaged_at, Some(2));
    assert_eq!(r.rejections, vec!["risk: kill switch is engaged".to_string()]);
}

#[test]
fn calendar_is_the_union_and_a_missing_bar_keeps_the_last_close() {
    // B has no bar at ts 2: its position stays marked at 110 and its pending exit waits.
    let b = vec![flat(0, dec!(100)), flat(1, dec!(110)), flat(3, dec!(120)), flat(4, dec!(120))];
    let s = Script::default().at("B", 0, enter("B", dec!(100), dec!(100), dec!(95))).at("B", 1, Signal::Exit(sym("B")));
    let r = go(s, limits(5), &scripted(), &[series("A", flats(0, 5, dec!(50))), series("B", b)]);
    assert_eq!(r.equity_curve.len(), 5);
    assert_eq!(r.equity_curve[2].1, r.equity_curve[1].1, "B marked at 110 on the day it has no bar");
    assert_eq!((r.trades[0].exit_ts, r.trades[0].exit_price), (3, dec!(120)));
}

#[test]
fn entry_windows_are_per_series_and_config_windows_are_refused() {
    let s = Script::default().at("A", 0, enter("A", dec!(10), dec!(100), dec!(95)));
    let a = Series { entry_windows: Some(vec![(2, 9)]), ..series("A", flats(0, 3, dec!(100))) };
    let r = go(s, limits(5), &scripted(), std::slice::from_ref(&a));
    assert_eq!(r.entries_outside_universe, 1);
    let bad = RunConfig { entry_windows: Some(vec![(0, 1)]), ..scripted() };
    let err = run_portfolio(&mut Script::default(), &mut Paper::default(), &RiskGate::new(limits(5)), &bad, &[a]);
    assert!(matches!(err, Err(BacktestError::InvalidConfig(_))), "{err:?}");
}

#[test]
fn duplicate_symbols_are_refused() {
    let data = [series("A", flats(0, 2, dec!(1))), series("A", flats(0, 2, dec!(1)))];
    let r = run_portfolio(&mut Script::default(), &mut Paper::default(), &RiskGate::new(limits(5)), &scripted(), &data);
    assert!(matches!(r, Err(BacktestError::InvalidConfig(_))));
}

#[test]
fn exposure_curve_tracks_invested_value() {
    let s = Script::default().at("A", 0, enter("A", dec!(10), dec!(100), dec!(95)));
    let a = vec![flat(0, dec!(100)), flat(1, dec!(100)), flat(2, dec!(110))];
    let r = go(s, limits(5), &scripted(), &[series("A", a)]);
    assert_eq!(r.exposure_curve, vec![dec!(0), dec!(1000), dec!(1100)]);
}

#[test]
fn trades_record_slippage_in_rupees() {
    // 10 bps flat: buy 10 @ 100 -> 100.1 (Rs 1), sell @ 110 -> 109.89 (Rs 1.10).
    let s = Script::default().at("A", 0, enter("A", dec!(10), dec!(100), dec!(95))).at("A", 1, Signal::Exit(sym("A")));
    let config = RunConfig { slippage_bps: dec!(10), ..scripted() };
    let r = go(s, limits(5), &config, &[series("A", vec![flat(0, dec!(100)), flat(1, dec!(100)), flat(2, dec!(110))])]);
    assert_eq!((r.trades[0].slippage, r.trades[0].exit_reason), (dec!(2.1), ExitReason::Signal));
}

//! Run-config behaviour: risk-based sizing at the fill, notional cap, slippage, warm-up.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_backtest::{run, BacktestError, RunConfig, RunReport, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, OrderRequest, Side, Symbol, Timestamp};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategy::{Signal, Strategy};

const CASH: Decimal = dec!(100000);

/// Emits a scripted signal on chosen bars; entries carry a 1-share placeholder qty.
struct Script(Vec<(Timestamp, Signal)>);

impl Strategy for Script {
    fn name(&self) -> &str {
        "script"
    }
    fn on_bar(&mut self, _s: &Symbol, b: &Bar) -> Vec<Signal> {
        self.0.iter().filter(|(ts, _)| *ts == b.ts).map(|(_, s)| s.clone()).collect()
    }
}

fn x() -> Symbol {
    Symbol("NSE:X".into())
}

fn flat(ts: Timestamp, p: Decimal) -> Bar {
    Bar::new(ts, p, p, p, p, dec!(1000)).unwrap()
}

fn enter(entry: Decimal, stop: Decimal) -> Signal {
    Signal::Enter(OrderRequest::new(x(), Side::Buy, dec!(1), entry, stop).unwrap())
}

fn go(script: Vec<(Timestamp, Signal)>, config: &RunConfig, bars: &[Bar]) -> (RunReport, Paper) {
    let mut paper = Paper::default();
    let gate = RiskGate::new(RiskLimits::default());
    let report = run(&mut Script(script), &mut paper, &gate, config, &x(), bars).unwrap();
    (report, paper)
}

fn risk(pct: Decimal) -> RunConfig {
    RunConfig { sizing: Sizing::RiskPct(pct), slippage_bps: dec!(0), ..RunConfig::new(CASH) }
}

#[test]
fn defaults_are_half_percent_risk_and_five_bps_slippage() {
    let c = RunConfig::new(CASH);
    assert_eq!((c.sizing, c.slippage_bps), (Sizing::RiskPct(dec!(0.5)), dec!(5)));
}

#[test]
fn placeholder_qty_is_resized_from_equity_risk_and_fill_price() {
    // 0.5% of 100_000 = 500 at risk; fill 101, stop 95 -> 500 / 6 = 83.3 -> 83 shares.
    let (_, p) = go(vec![(0, enter(dec!(100), dec!(95)))], &risk(dec!(0.5)), &[flat(0, dec!(100)), flat(1, dec!(101))]);
    assert_eq!((p.fills()[0].qty, p.fills()[0].price), (dec!(83), dec!(101)));
}

#[test]
fn risk_size_is_cut_to_the_notional_cap_instead_of_rejected() {
    // Stop 0.5 away -> 1000 shares by risk, but 20% of 100_000 at 100 caps it at 200.
    let (r, p) = go(vec![(0, enter(dec!(100), dec!(99.5)))], &risk(dec!(0.5)), &[flat(0, dec!(100)), flat(1, dec!(100))]);
    assert_eq!(p.fills()[0].qty, dec!(200));
    assert!(r.rejections.is_empty(), "{:?}", r.rejections);
}

#[test]
fn entry_is_skipped_when_risk_budget_buys_zero_shares() {
    // 500 at risk, 600 per share to the stop -> 0 shares.
    let (r, p) = go(vec![(0, enter(dec!(1000), dec!(400)))], &risk(dec!(0.5)), &[flat(0, dec!(1000)), flat(1, dec!(1000))]);
    assert!(p.fills().is_empty());
    assert_eq!(r.entries_sized_to_zero, 1);
}

#[test]
fn slippage_worsens_entry_and_exit_prices() {
    let config = RunConfig { slippage_bps: dec!(10), ..risk(dec!(0.5)) };
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), flat(2, dec!(110))];
    let (r, _) = go(vec![(0, enter(dec!(100), dec!(95))), (1, Signal::Exit(x()))], &config, &bars);
    assert_eq!((r.trades[0].entry_price, r.trades[0].exit_price), (dec!(100.1), dec!(109.89)));
}

#[test]
fn stop_exit_pays_slippage_below_the_stop() {
    let config = RunConfig { slippage_bps: dec!(10), ..risk(dec!(0.5)) };
    let low = Bar::new(2, dec!(99), dec!(99), dec!(94), dec!(96), dec!(1000)).unwrap();
    let (r, _) = go(vec![(0, enter(dec!(100), dec!(95)))], &config, &[flat(0, dec!(100)), flat(1, dec!(100)), low]);
    assert_eq!(r.trades[0].exit_price, dec!(94.905));
}

#[test]
fn warm_up_bars_feed_the_strategy_but_never_trade_or_count() {
    let config = RunConfig { trade_from: 2, ..risk(dec!(0.5)) };
    let bars: Vec<Bar> = (0..4).map(|ts| flat(ts, dec!(100))).collect();
    let (r, p) = go(vec![(0, enter(dec!(100), dec!(95))), (2, enter(dec!(100), dec!(95)))], &config, &bars);
    assert_eq!(r.equity_curve.first().map(|e| e.0), Some(2));
    assert_eq!((r.bars, r.orders_proposed, p.fills().len()), (2, 1, 1));
    assert_eq!(p.fills()[0].ts, 3);
}

#[test]
fn trade_reports_r_multiple_and_net_return() {
    // 100 shares at 100, stop 95 (risk 500); exit 110 -> gross 1000.
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), flat(2, dec!(110))];
    let (r, _) = go(vec![(0, enter(dec!(100), dec!(95))), (1, Signal::Exit(x()))], &risk(dec!(0.5)), &bars);
    let t = &r.trades[0];
    assert_eq!((t.qty, t.stop), (dec!(100), dec!(95)));
    assert_eq!(t.r_multiple(), Some(t.net_pnl / dec!(500)));
    assert_eq!(t.net_return_pct(), t.net_pnl / dec!(10000) * dec!(100));
}

#[test]
fn rejects_non_positive_risk_or_negative_slippage() {
    let bars = [flat(0, dec!(100))];
    let mut paper = Paper::default();
    let gate = RiskGate::new(RiskLimits::default());
    for bad in [risk(dec!(0)), RunConfig { slippage_bps: dec!(-1), ..risk(dec!(0.5)) }] {
        let r = run(&mut Script(vec![]), &mut paper, &gate, &bad, &x(), &bars);
        assert!(matches!(r, Err(BacktestError::InvalidConfig(_))), "{r:?}");
    }
}

#[test]
fn entries_fill_only_inside_entry_windows() {
    // Member for fills at ts 2..4 (half-open). Signal at 0 fills at 1: outside, skipped.
    // Signal at 2 fills at 3: inside.
    let config = RunConfig { entry_windows: Some(vec![(2, 4)]), ..risk(dec!(0.5)) };
    let bars: Vec<Bar> = (0..5).map(|ts| flat(ts, dec!(100))).collect();
    let (r, p) = go(vec![(0, enter(dec!(100), dec!(95))), (2, enter(dec!(100), dec!(95)))], &config, &bars);
    assert_eq!(r.entries_outside_universe, 1);
    assert_eq!(p.fills().iter().map(|f| f.ts).collect::<Vec<_>>(), vec![3]);
}

#[test]
fn exits_are_allowed_after_the_entry_window_closes() {
    let config = RunConfig { entry_windows: Some(vec![(1, 2)]), ..risk(dec!(0.5)) };
    let bars: Vec<Bar> = (0..5).map(|ts| flat(ts, dec!(100))).collect();
    let (r, _) = go(vec![(0, enter(dec!(100), dec!(95))), (3, Signal::Exit(x()))], &config, &bars);
    assert_eq!(r.trades.len(), 1);
    assert_eq!((r.trades[0].entry_ts, r.trades[0].exit_ts), (1, 4));
}

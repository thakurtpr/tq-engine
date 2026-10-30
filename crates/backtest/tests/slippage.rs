//! The adopted slippage model inside the engine: ADV cap, prior-bar liquidity, stop 2x, ticks.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_backtest::{run, ExitReason, RunConfig, RunReport, SlippageModel, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, OrderRequest, Side, Symbol, Timestamp};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategy::{Signal, Strategy};

const CASH: Decimal = dec!(1000000);
/// Bars 0..20 build the 20-bar ADV; the signal is on bar 20 and fills at bar 21's open.
const SIGNAL_TS: Timestamp = 20;

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

fn bar(ts: Timestamp, o: Decimal, l: Decimal, c: Decimal, v: Decimal) -> Bar {
    Bar::new(ts, o, o.max(c), l, c, v).unwrap()
}

fn history(volume: Decimal) -> Vec<Bar> {
    (0..=SIGNAL_TS).map(|ts| bar(ts, dec!(100), dec!(100), dec!(100), volume)).collect()
}

fn enter() -> Signal {
    Signal::Enter(OrderRequest::new(x(), Side::Buy, dec!(1), dec!(100), dec!(95)).unwrap())
}

fn adopted(mult: Decimal) -> RunConfig {
    RunConfig { sizing: Sizing::RiskPct(dec!(0.5)), slippage_model: Some(SlippageModel::default()), slippage_mult: mult, ..RunConfig::new(CASH) }
}

fn go(script: Vec<(Timestamp, Signal)>, config: &RunConfig, bars: &[Bar]) -> (RunReport, Paper) {
    let mut paper = Paper::default();
    let report = run(&mut Script(script), &mut paper, &RiskGate::new(RiskLimits::default()), config, &x(), bars).unwrap();
    (report, paper)
}

#[test]
fn entries_are_cut_to_one_percent_of_adv_and_counted() {
    // Risk budget 5_000 / ~5 per share = ~995 shares; ADV 1_000 caps the order at 10.
    let mut bars = history(dec!(1000));
    bars.push(bar(21, dec!(100), dec!(100), dec!(100), dec!(1000)));
    let (r, p) = go(vec![(SIGNAL_TS, enter())], &adopted(dec!(1)), &bars);
    assert_eq!(p.fills()[0].qty, dec!(10));
    assert_eq!(r.entries_adv_capped, 1);
}

#[test]
fn adv_uses_only_bars_before_the_fill() {
    // A huge volume on the fill bar itself must not lift the cap.
    let mut bars = history(dec!(1000));
    bars.push(bar(21, dec!(100), dec!(100), dec!(100), dec!(1000000000)));
    let (_, p) = go(vec![(SIGNAL_TS, enter())], &adopted(dec!(1)), &bars);
    assert_eq!(p.fills()[0].qty, dec!(10));
}

#[test]
fn no_liquidity_history_means_no_entry() {
    let bars: Vec<Bar> = (0..3).map(|ts| bar(ts, dec!(100), dec!(100), dec!(100), dec!(1000000))).collect();
    let (r, p) = go(vec![(0, enter())], &adopted(dec!(1)), &bars);
    assert!(p.fills().is_empty());
    assert_eq!(r.entries_sized_to_zero, 1);
}

#[test]
fn stop_exits_pay_double_and_round_down_to_the_tick() {
    // Entry: 100 * 1.0005 = 100.05 (on tick). Stop 95 hit; 10 bps: 94.905 -> 94.90.
    let mut bars = history(dec!(100000000));
    bars.push(bar(21, dec!(100), dec!(100), dec!(100), dec!(1)));
    bars.push(bar(22, dec!(99), dec!(94), dec!(96), dec!(1)));
    let (r, _) = go(vec![(SIGNAL_TS, enter())], &adopted(dec!(1)), &bars);
    let t = &r.trades[0];
    assert_eq!((t.entry_price, t.exit_price, t.exit_reason), (dec!(100.05), dec!(94.90), ExitReason::StopLoss));
    assert_eq!(t.slippage, (dec!(0.05) + dec!(0.10)) * t.qty);
}

#[test]
fn stress_multiplier_scales_the_whole_slippage() {
    // 3x: entry 15 bps -> 100.15; stop exit 30 bps on 95 -> 94.715 -> 94.70.
    let mut bars = history(dec!(100000000));
    bars.push(bar(21, dec!(100), dec!(100), dec!(100), dec!(1)));
    bars.push(bar(22, dec!(99), dec!(94), dec!(96), dec!(1)));
    let (r, _) = go(vec![(SIGNAL_TS, enter())], &adopted(dec!(3)), &bars);
    assert_eq!((r.trades[0].entry_price, r.trades[0].exit_price), (dec!(100.15), dec!(94.70)));
}

#[test]
fn turnover_slippage_and_impact_fills_are_counted() {
    // ADV 1_000: the capped 10-share entry is 1% of ADV, above the 0.05% impact threshold.
    // A deep book (ADV 1e8) keeps the stop exit below it.
    let mut bars = history(dec!(1000));
    bars.push(bar(21, dec!(100), dec!(100), dec!(100), dec!(100000000)));
    bars.extend((22..42).map(|ts| bar(ts, dec!(100), dec!(100), dec!(100), dec!(100000000))));
    bars.push(bar(42, dec!(99), dec!(94), dec!(96), dec!(1)));
    let (r, p) = go(vec![(SIGNAL_TS, enter())], &adopted(dec!(1)), &bars);
    let turnover: Decimal = p.fills().iter().map(|f| f.price * f.qty).sum();
    assert_eq!(r.turnover, turnover);
    assert_eq!(r.total_slippage, r.trades[0].slippage);
    assert_eq!(r.fills_above_impact_threshold, 1);
}

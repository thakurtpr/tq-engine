//! Scenario tests for the backtest engine: fills, costs, positions, exits, stops, risk.

use std::collections::HashMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_backtest::{run, BacktestError, ExitReason, RunConfig, RunReport, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, OrderRequest, Side, Symbol, Timestamp};
use tq_costs::CostSchedule;
use tq_risk::{RiskGate, RiskLimits};
use tq_strategy::{Signal, Strategy};

const CASH: Decimal = dec!(100000);

/// Emits pre-scripted signals keyed by bar timestamp.
#[derive(Default)]
struct Scripted {
    by_ts: HashMap<Timestamp, Vec<Signal>>,
    fills: usize,
}

impl Scripted {
    fn at(mut self, ts: Timestamp, signal: Signal) -> Self {
        self.by_ts.entry(ts).or_default().push(signal);
        self
    }
}

impl Strategy for Scripted {
    fn name(&self) -> &str {
        "scripted"
    }
    fn on_bar(&mut self, _s: &Symbol, b: &Bar) -> Vec<Signal> {
        self.by_ts.remove(&b.ts).unwrap_or_default()
    }
    fn on_fill(&mut self, _fill: &tq_core::Fill) {
        self.fills += 1;
    }
}

fn x() -> Symbol {
    Symbol("NSE:X".into())
}

fn bar(ts: Timestamp, o: Decimal, h: Decimal, l: Decimal, c: Decimal) -> Bar {
    Bar::new(ts, o, h, l, c, dec!(1000)).unwrap()
}

fn flat(ts: Timestamp, p: Decimal) -> Bar {
    bar(ts, p, p, p, p)
}

fn enter(qty: Decimal, entry: Decimal, stop: Decimal) -> Signal {
    Signal::Enter(OrderRequest::new(x(), Side::Buy, qty, entry, stop).unwrap())
}

fn exit() -> Signal {
    Signal::Exit(x())
}

fn buy_cost(turnover: Decimal) -> Decimal {
    CostSchedule::zerodha_nse_delivery().compute(Side::Buy, turnover).total()
}

fn round_trip(buy: Decimal, sell: Decimal) -> Decimal {
    CostSchedule::zerodha_nse_delivery().round_trip(buy, sell)
}

fn go(s: &mut Scripted, limits: RiskLimits, bars: &[Bar]) -> (RunReport, Paper) {
    let mut paper = Paper::default();
    let report = run(s, &mut paper, &RiskGate::new(limits), &as_signalled(), &x(), bars).unwrap();
    (report, paper)
}

/// Scenario tests script exact quantities, so they keep the signalled size and pay no slippage.
fn as_signalled() -> RunConfig {
    RunConfig { sizing: Sizing::AsSignalled, slippage_bps: dec!(0), ..RunConfig::new(CASH) }
}

/// Wide limits so scenario tests can hold large positions with distant stops.
fn wide(risk_pct: Decimal) -> RiskLimits {
    RiskLimits { max_risk_per_trade_pct: risk_pct, max_position_notional_pct: dec!(100), ..RiskLimits::default() }
}

#[test]
fn fills_at_next_bar_open_never_same_bar() {
    let bars = [flat(0, dec!(100)), bar(1, dec!(101), dec!(110), dec!(100), dec!(105))];
    let mut s = Scripted::default().at(0, enter(dec!(10), dec!(100), dec!(95)));
    let (r, p) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.orders_filled, 1);
    assert_eq!((p.fills()[0].ts, p.fills()[0].price), (1, dec!(101)));
    assert_eq!(s.fills, 1);
}

#[test]
fn entry_costs_reduce_marked_equity() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(100))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95)));
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.equity_curve, vec![(0, CASH), (1, CASH - buy_cost(dec!(10000)))]);
    assert_eq!(r.metrics.total_costs, buy_cost(dec!(10000)));
}

#[test]
fn equity_is_marked_to_market_at_close() {
    let bars = [flat(0, dec!(100)), bar(1, dec!(100), dec!(108), dec!(99), dec!(107))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95)));
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.equity_curve[1].1, CASH + dec!(700) - buy_cost(dec!(10000)));
}

#[test]
fn second_enter_while_open_is_ignored_and_counted() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), flat(2, dec!(100))];
    let mut s = Scripted::default()
        .at(0, enter(dec!(10), dec!(100), dec!(95)))
        .at(0, enter(dec!(10), dec!(100), dec!(95)))
        .at(1, enter(dec!(10), dec!(100), dec!(95)));
    let (r, p) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.duplicate_entries_ignored, 2);
    assert_eq!(p.fills().len(), 1);
}

#[test]
fn exit_signal_closes_at_next_open_and_records_trade() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(110), dec!(112), dec!(109), dec!(111))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95))).at(1, exit());
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    let t = &r.trades[0];
    let costs = round_trip(dec!(10000), dec!(11000));
    assert_eq!((t.entry_ts, t.entry_price, t.exit_ts, t.exit_price), (1, dec!(100), 2, dec!(110)));
    assert_eq!((t.qty, t.gross_pnl, t.costs, t.net_pnl), (dec!(100), dec!(1000), costs, dec!(1000) - costs));
    assert_eq!(t.exit_reason, ExitReason::Signal);
    assert_eq!(r.equity_curve[2].1, CASH + dec!(1000) - costs);
}

#[test]
fn stop_exits_at_stop_price_when_open_is_above_it() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(99), dec!(99), dec!(94), dec!(96))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95)));
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!((r.trades[0].exit_price, r.trades[0].exit_reason), (dec!(95), ExitReason::StopLoss));
    assert_eq!(s.fills, 2, "a stop exit must reach Strategy::on_fill so the strategy goes flat");
}

#[test]
fn stop_gap_down_exits_at_the_worse_open() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(90), dec!(92), dec!(88), dec!(91))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95)));
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.trades[0].exit_price, dec!(90));
}

#[test]
fn stops_are_checked_before_new_signals() {
    // Stopped out on bar 2, so the Enter emitted on bar 2 is a fresh entry, not a duplicate.
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(99), dec!(99), dec!(94), dec!(96)), flat(3, dec!(96))];
    let mut s = Scripted::default()
        .at(0, enter(dec!(10), dec!(100), dec!(95)))
        .at(2, enter(dec!(10), dec!(96), dec!(90)));
    let (r, p) = go(&mut s, RiskLimits::default(), &bars);
    assert_eq!(r.duplicate_entries_ignored, 0);
    assert_eq!(p.fills().len(), 3);
}

#[test]
fn entry_skipped_when_next_open_gaps_through_stop() {
    let bars = [flat(0, dec!(100)), flat(1, dec!(94))];
    let mut s = Scripted::default().at(0, enter(dec!(10), dec!(100), dec!(95)));
    let (r, p) = go(&mut s, RiskLimits::default(), &bars);
    assert!(p.fills().is_empty());
    assert!(r.rejections[0].contains("stop"), "{:?}", r.rejections);
}

#[test]
fn entry_rejected_when_gap_up_breaks_risk_limit() {
    // Risk at close 100 * 5 = 500; at a 112 open it is 100 * 17 = 1700 > 1000.
    let bars = [flat(0, dec!(100)), flat(1, dec!(112))];
    let mut s = Scripted::default().at(0, enter(dec!(100), dec!(100), dec!(95)));
    let (r, p) = go(&mut s, RiskLimits::default(), &bars);
    assert!(p.fills().is_empty());
    assert!(r.rejections[0].contains("risk per trade"), "{:?}", r.rejections);
}

#[test]
fn daily_loss_limit_trips_during_run() {
    // 900 sh @ 100, stop 97: stopped at 97 loses 2700 + costs (~2.8%): daily limit 2%, drawdown 5%.
    let bars = [flat(0, dec!(100)), flat(1, dec!(100)), bar(2, dec!(99), dec!(99), dec!(96), dec!(96)), flat(3, dec!(96))];
    let mut s = Scripted::default()
        .at(0, enter(dec!(900), dec!(100), dec!(97)))
        .at(2, enter(dec!(10), dec!(96), dec!(90)));
    let (r, _) = go(&mut s, wide(dec!(5)), &bars);
    assert_eq!(r.trades[0].exit_reason, ExitReason::StopLoss);
    assert_eq!(r.rejections, vec!["risk: daily loss limit reached".to_string()]);
    assert_eq!(r.kill_switch_engaged_at, None);
}

#[test]
fn drawdown_engages_kill_switch_which_blocks_entries_but_never_exits() {
    // 900 sh @ 100, stop 90; close at 94 marks a 5400 + costs loss (> 5%) -> kill switch.
    let bars = [
        flat(0, dec!(100)),
        flat(1, dec!(100)),
        bar(2, dec!(99), dec!(99), dec!(93), dec!(94)),
        flat(3, dec!(95)),
        flat(4, dec!(95)),
        flat(5, dec!(95)),
    ];
    let mut s = Scripted::default()
        .at(0, enter(dec!(900), dec!(100), dec!(90)))
        .at(2, exit())
        .at(4, enter(dec!(10), dec!(95), dec!(90)));
    let limits = RiskLimits { max_drawdown_pct: dec!(5), ..wide(dec!(10)) };
    let (r, _) = go(&mut s, limits, &bars);
    assert_eq!(r.kill_switch_engaged_at, Some(2));
    assert_eq!((r.trades.len(), r.trades[0].exit_ts), (1, 3));
    assert_eq!(r.rejections, vec!["risk: kill switch is engaged".to_string()]);
}

#[test]
fn zero_gross_edge_loses_money_net_of_costs() {
    // Prices never move: every round trip is gross-flat, so costs are a pure loss.
    let bars: Vec<Bar> = (0..20).map(|ts| flat(ts, dec!(100))).collect();
    let mut s = Scripted::default();
    for ts in (0..18).step_by(2) {
        // risk 100 * 4 = 400, inside 0.5% of the (cost-eroded) equity.
        s = s.at(ts, enter(dec!(100), dec!(100), dec!(96))).at(ts + 1, exit());
    }
    let (r, _) = go(&mut s, RiskLimits::default(), &bars);
    let gross: Decimal = r.trades.iter().map(|t| t.gross_pnl).sum();
    let net: Decimal = r.trades.iter().map(|t| t.net_pnl).sum();
    assert_eq!((r.trades.len(), gross), (9, dec!(0)));
    assert!(net < dec!(0));
    assert_eq!(r.equity_curve.last().map(|p| p.1), Some(CASH + net));
    assert!(r.metrics.total_return_pct < dec!(0));
    assert_eq!(r.metrics.win_rate_pct, dec!(0));
}

#[test]
fn rejects_non_positive_cash() {
    let mut paper = Paper::default();
    let gate = RiskGate::new(RiskLimits::default());
    let config = RunConfig { initial_cash: dec!(0), ..as_signalled() };
    let r = run(&mut Scripted::default(), &mut paper, &gate, &config, &x(), &[flat(0, dec!(1))]);
    assert_eq!(r.err(), Some(BacktestError::NonPositiveCash));
}

#[test]
fn rejects_bars_out_of_order() {
    let mut paper = Paper::default();
    let gate = RiskGate::new(RiskLimits::default());
    let bars = [flat(1, dec!(1)), flat(1, dec!(1))];
    let r = run(&mut Scripted::default(), &mut paper, &gate, &as_signalled(), &x(), &bars);
    assert_eq!(r.err(), Some(BacktestError::BarsOutOfOrder));
}

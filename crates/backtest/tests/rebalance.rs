//! Cross-sectional rebalancing path: month-end targets traded at the next open, sells before
//! buys, every buy through the risk gate.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_backtest::{run_rebalance, RunConfig, RunReport, Series, Sizing, SlippageModel};
use tq_broker::Paper;
use tq_core::{Bar, Symbol, Timestamp};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategy::{History, Rebalancer, Target};

const CASH: Decimal = dec!(100000);
const DAY: i64 = 86_400_000;
/// 2012-01-02 15:30 IST; +29 days is Tuesday 2012-01-31, +30 is 2012-02-01.
const JAN2: Timestamp = 1_325_498_400_000;

fn day(n: i64) -> Timestamp {
    JAN2 + n * DAY
}

fn sym(s: &str) -> Symbol {
    Symbol(s.into())
}

fn flat(ts: Timestamp, p: Decimal) -> Bar {
    Bar::new(ts, p, p, p, p, dec!(100000000)).unwrap()
}

/// Days `from..to` (calendar days, so month ends fall where the calendar puts them) at `p`.
fn flats(from: i64, to: i64, p: Decimal) -> Vec<Bar> {
    (from..to).map(|d| flat(day(d), p)).collect()
}

fn series(s: &str, bars: Vec<Bar>) -> Series {
    Series { symbol: sym(s), bars, entry_windows: None }
}

/// Returns one scripted target book per call, in order; records the members it was shown.
#[derive(Default)]
struct Script {
    books: Vec<Vec<Target>>,
    calls: usize,
    seen: Vec<(Timestamp, Vec<String>)>,
}

impl Rebalancer for Script {
    fn name(&self) -> &str {
        "script"
    }
    fn targets(&mut self, date: Timestamp, members: &[Symbol], _h: &History) -> Vec<Target> {
        self.seen.push((date, members.iter().map(|m| m.0.clone()).collect()));
        self.calls += 1;
        self.books.get(self.calls - 1).cloned().unwrap_or_default()
    }
}

fn book(items: &[(&str, Decimal)]) -> Vec<Target> {
    items.iter().map(|(s, w)| (sym(s), *w)).collect()
}

fn frictionless() -> RunConfig {
    RunConfig { sizing: Sizing::AsSignalled, slippage_bps: dec!(0), ..RunConfig::new(CASH) }
}

fn limits() -> RiskLimits {
    RiskLimits { max_position_notional_pct: dec!(100), ..RiskLimits::default() }
}

fn go(script: &mut Script, limits: RiskLimits, config: &RunConfig, data: &[Series]) -> (RunReport, Paper) {
    let mut paper = Paper::default();
    let r = run_rebalance(script, &mut paper, &RiskGate::new(limits), config, data).unwrap();
    (r, paper)
}

#[test]
fn month_end_targets_fill_at_the_next_open_with_a_cash_buffer() {
    // Month end is day 29 (Jan 31). 99.5% of 100_000 at 100 -> 995 shares on Feb 1.
    let mut s = Script { books: vec![book(&[("A", dec!(1))])], ..Script::default() };
    let (r, p) = go(&mut s, limits(), &frictionless(), &[series("A", flats(0, 35, dec!(100)))]);
    assert_eq!(s.seen.iter().map(|x| x.0).collect::<Vec<_>>(), vec![day(29)], "only month ends");
    assert_eq!((p.fills()[0].ts, p.fills()[0].qty, p.fills()[0].price), (day(30), dec!(995), dec!(100)));
    assert_eq!(r.rebalances, 1);
}

#[test]
fn sells_come_before_buys_so_a_full_rotation_is_funded() {
    // Jan 31: all in A. Feb 29 (day 58): rotate to B. Without selling first B is unaffordable.
    let mut s = Script { books: vec![book(&[("A", dec!(1))]), book(&[("B", dec!(1))])], ..Script::default() };
    let data = [series("A", flats(0, 62, dec!(100))), series("B", flats(0, 62, dec!(50)))];
    let (r, p) = go(&mut s, limits(), &frictionless(), &data);
    assert_eq!(p.holding(&sym("A")), dec!(0));
    assert!(p.holding(&sym("B")) > dec!(1900), "{}", p.holding(&sym("B")));
    assert_eq!(r.trades.len(), 1);
    assert_eq!((r.trades[0].symbol.0.as_str(), r.trades[0].exit_ts), ("A", day(59)));
    assert!(r.rejections.is_empty(), "{:?}", r.rejections);
}

#[test]
fn warm_up_targets_only_fill_if_the_fill_day_is_in_the_window() {
    // trade_from = day 30: the Jan 31 decision fills on Feb 1; nothing earlier trades.
    let mut s = Script { books: vec![book(&[("A", dec!(1))])], ..Script::default() };
    let config = RunConfig { trade_from: day(30), ..frictionless() };
    let (r, p) = go(&mut s, limits(), &config, &[series("A", flats(0, 35, dec!(100)))]);
    assert_eq!(p.fills()[0].ts, day(30));
    assert_eq!(r.equity_curve.first().map(|e| e.0), Some(day(30)));
    let config = RunConfig { trade_from: day(31), ..frictionless() };
    let mut s = Script { books: vec![book(&[("A", dec!(1))])], ..Script::default() };
    let (_, p) = go(&mut s, limits(), &config, &[series("A", flats(0, 35, dec!(100)))]);
    assert!(p.fills().is_empty(), "a decision whose fill day is warm-up is dropped");
}

#[test]
fn buys_pass_the_risk_gate_notional_cap() {
    let mut s = Script { books: vec![book(&[("A", dec!(1))])], ..Script::default() };
    let (r, p) = go(&mut s, RiskLimits::default(), &frictionless(), &[series("A", flats(0, 35, dec!(100)))]);
    assert!(p.fills().is_empty());
    assert!(r.rejections[0].starts_with("risk: position notional"), "{:?}", r.rejections);
}

#[test]
fn small_top_ups_are_skipped_but_new_positions_are_not() {
    // Month 2 asks for the same 100% in A: the drift is tiny, so no trade.
    let mut s = Script { books: vec![book(&[("A", dec!(1))]), book(&[("A", dec!(1))])], ..Script::default() };
    let (r, p) = go(&mut s, limits(), &frictionless(), &[series("A", flats(0, 62, dec!(100)))]);
    assert_eq!((p.fills().len(), r.rebalances), (1, 2));
}

#[test]
fn kill_switch_blocks_buys_but_never_sells() {
    // A falls 20% in February: kill switch (10%). March rotation to B: A is sold, B blocked.
    let mut a = flats(0, 31, dec!(100));
    a.extend(flats(31, 62, dec!(80)));
    let mut s = Script { books: vec![book(&[("A", dec!(1))]), book(&[("B", dec!(1))])], ..Script::default() };
    let (r, p) = go(&mut s, limits(), &frictionless(), &[series("A", a), series("B", flats(0, 62, dec!(50)))]);
    assert_eq!(r.kill_switch_engaged_at, Some(day(31)));
    assert_eq!((p.holding(&sym("A")), p.holding(&sym("B"))), (dec!(0), dec!(0)));
    assert_eq!(r.rejections, vec!["risk: kill switch is engaged".to_string()]);
}

#[test]
fn only_point_in_time_members_are_offered_and_targeted() {
    // B joins the index on Feb 15 (day 44): absent from the Jan 31 member list, present on Feb 29.
    let b = Series { entry_windows: Some(vec![(day(44), Timestamp::MAX)]), ..series("B", flats(0, 62, dec!(50))) };
    let mut s = Script { books: vec![book(&[("B", dec!(0.5))])], ..Script::default() };
    let (r, p) = go(&mut s, limits(), &frictionless(), &[series("A", flats(0, 62, dec!(100))), b]);
    assert_eq!(s.seen[0].1, vec!["A"]);
    assert_eq!(s.seen[1].1, vec!["A", "B"]);
    assert_eq!(p.holding(&sym("B")), dec!(0), "a target outside the member list is dropped");
    assert_eq!(r.rejections, vec!["target not an index member: B".to_string()]);
}

#[test]
fn partial_sells_record_trades_with_average_cost_and_prorated_costs() {
    // 100% A, then 50% A at a higher price: half the shares are sold as one trade.
    let mut a = flats(0, 31, dec!(100));
    a.extend(flats(31, 62, dec!(110)));
    let mut s = Script { books: vec![book(&[("A", dec!(1))]), book(&[("A", dec!(0.5))])], ..Script::default() };
    let mut paper = Paper::default();
    let config = RunConfig { slippage_bps: dec!(0), ..RunConfig::new(CASH) };
    let r = run_rebalance(&mut s, &mut paper, &RiskGate::new(limits()), &config, &[series("A", a)]).unwrap();
    let t = &r.trades[0];
    let (buy, sell) = (&paper.fills()[0], &paper.fills()[1]);
    assert_eq!((t.entry_price, t.exit_price), (dec!(100), dec!(110)));
    assert_eq!(t.gross_pnl, dec!(10) * t.qty);
    assert_eq!(t.costs, tq_costs::round_paise(buy.costs * t.qty / buy.qty + sell.costs));
    assert_eq!(t.r_multiple(), None, "no stop, so no R");
    let costs: Decimal = paper.fills().iter().map(|f| f.costs).sum();
    let held = paper.holding(&sym("A"));
    assert_eq!(r.equity_curve.last().map(|e| e.1), Some(CASH - costs + dec!(10) * t.qty + held * dec!(10)));
}

#[test]
fn invalid_weights_are_rejected_and_the_book_is_left_alone() {
    let mut s = Script { books: vec![book(&[("A", dec!(0.7)), ("B", dec!(0.7))])], ..Script::default() };
    let data = [series("A", flats(0, 35, dec!(100))), series("B", flats(0, 35, dec!(100)))];
    let (r, p) = go(&mut s, limits(), &frictionless(), &data);
    assert!(p.fills().is_empty());
    assert_eq!(r.rejections, vec!["targets rejected: weights must be positive and sum to at most 1".to_string()]);
}

/// Closes alternate between `p` (even days) and `p * 1.02` (odd days), so sigma is about 2%.
fn with_volume(from: i64, to: i64, p: Decimal, v: Decimal) -> Vec<Bar> {
    let price = |d: i64| if d % 2 == 0 { p } else { p * dec!(1.02) };
    (from..to).map(|d| Bar::new(day(d), price(d), price(d), price(d), price(d), v).unwrap()).collect()
}

#[test]
fn buys_pay_modelled_slippage_and_the_adv_cap_unless_adv_is_unknown() {
    // A: ADV 10_000 shares -> cap 100 shares, 1% of ADV, impact 3.5 + 0.7 * ~2% * 0.1 * 1e4 bps.
    // B: zero volume -> no cap, base 5 bps. The fill day (day 30) opens at 100 for both.
    let config = RunConfig { slippage_model: Some(SlippageModel { tick: dec!(0), ..SlippageModel::default() }), ..RunConfig::new(CASH) };
    let mut s = Script { books: vec![book(&[("A", dec!(0.4)), ("B", dec!(0.4))])], ..Script::default() };
    let data = [series("A", with_volume(0, 35, dec!(100), dec!(10000))), series("B", with_volume(0, 35, dec!(100), dec!(0)))];
    let (r, p) = go(&mut s, limits(), &config, &data);
    let fill = |name: &str| p.fills().iter().find(|f| f.symbol.0 == name).map(|f| (f.qty, f.price));
    assert_eq!(fill("A").map(|f| f.0), Some(dec!(100)));
    assert!(fill("A").is_some_and(|f| f.1 > dec!(100.1)), "impact above the 5 bps base: {:?}", fill("A"));
    assert_eq!(fill("B"), Some((dec!(398), dec!(100.05))), "base 5 bps only");
    assert_eq!((r.entries_adv_capped, r.fills_without_adv, r.fills_above_impact_threshold), (1, 1, 1));
}

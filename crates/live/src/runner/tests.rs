//! Runner rules: next-open fills with slippage, stops, stale/gap handling, holdout refusal,
//! NSE flatten, TDS as cash (not P&L).

use std::collections::{HashMap, HashSet};

use rust_decimal_macros::dec;
use tq_core::{Bar, Symbol};
use tq_costs::{CryptoSpotFees, Liquidity};
use tq_risk::RiskLimits;

use super::*;
use crate::journal::test_support::SharedBuf;
use crate::strategies::long_entry;

const M: i64 = 60_000;

/// Emits an entry (stop given) or exit at fixed bar indexes.
struct Script {
    n: usize,
    enters: HashMap<usize, Decimal>,
    exits: HashSet<usize>,
}

impl Strategy for Script {
    fn name(&self) -> &str {
        "script"
    }
    fn on_bar(&mut self, s: &Symbol, bar: &Bar) -> Vec<Signal> {
        let i = self.n;
        self.n += 1;
        if let Some(stop) = self.enters.get(&i) {
            return long_entry(s, bar.close, *stop);
        }
        if self.exits.contains(&i) { vec![Signal::Exit(s.clone())] } else { Vec::new() }
    }
}

fn script(enters: &[(usize, Decimal)], exits: &[usize]) -> Script {
    Script { n: 0, enters: enters.iter().copied().collect(), exits: exits.iter().copied().collect() }
}

fn cfg(session: Session) -> RunnerConfig {
    RunnerConfig {
        symbol: Symbol("BTCUSDT".into()),
        lot: dec!(1),
        initial_cash: dec!(100000),
        slippage_bps: dec!(10),
        costs: CryptoSpotFees::default().schedule(Liquidity::Taker),
        limits: RiskLimits::default(),
        session,
        max_stale_ms: 15_000,
        mark_every: 1,
        tds: None,
        holdout_from: None,
    }
}

fn runner(c: RunnerConfig, s: Script) -> (Runner<Script>, SharedBuf) {
    let buf = SharedBuf::default();
    (Runner::new(c, s, Journal::to_writer(Box::new(buf.clone()))).unwrap(), buf)
}

fn ev(ts: i64, o: Decimal, h: Decimal, l: Decimal, c: Decimal, late_ms: i64) -> FeedEvent {
    let bar = Bar::new(ts, o, h, l, c, dec!(1)).unwrap();
    FeedEvent::Bar(BarEvent { symbol: Symbol("BTCUSDT".into()), bar, interval_ms: M, received_ms: ts + M + late_ms, source: Source::Websocket })
}

fn flat(ts: i64) -> FeedEvent {
    ev(ts, dec!(100), dec!(100), dec!(100), dec!(100), 0)
}

fn kinds(buf: &SharedBuf, kind: &str) -> Vec<serde_json::Value> {
    buf.lines().into_iter().filter(|l| l["type"] == kind).collect()
}

#[test]
fn entry_fills_at_next_open_sized_by_risk_and_exit_pays_slippage_both_ways() {
    let (mut r, buf) = runner(cfg(Session::Crypto24x7), script(&[(0, dec!(95))], &[2]));
    for i in 0..4 {
        r.on_event(flat(i * M)).unwrap();
    }
    let fills = kinds(&buf, "fill");
    // 0.5% of 100000 = 500 risk / (100.1 - 95) = 98.03 -> 98 units at 100 + 10 bps.
    assert_eq!((fills[0]["ts"].as_i64(), fills[0]["units"].as_str(), fills[0]["price"].as_str()), (Some(M), Some("98"), Some("100.1")));
    assert_eq!((fills[1]["ts"].as_i64(), fills[1]["price"].as_str()), (Some(3 * M), Some("99.9")));
    let out = r.finish().unwrap();
    let t = &out.trades[0];
    assert_eq!(t.gross_pnl, dec!(0));
    assert_eq!(t.slippage, dec!(19.6));
    assert_eq!(t.net_pnl, -dec!(19.6) - t.fees);
}

#[test]
fn stop_exits_at_min_of_open_and_stop() {
    let (mut r, _) = runner(cfg(Session::Crypto24x7), script(&[(0, dec!(95))], &[]));
    r.on_event(flat(0)).unwrap();
    r.on_event(flat(M)).unwrap();
    r.on_event(ev(2 * M, dec!(99), dec!(99), dec!(94), dec!(96), 0)).unwrap();
    r.on_event(ev(3 * M, dec!(90), dec!(91), dec!(80), dec!(85), 0)).unwrap();
    let out = r.finish().unwrap();
    assert_eq!(out.trades.len(), 1);
    assert_eq!(out.trades[0].reason, ExitReason::StopLoss);
    assert_eq!(out.trades[0].raw_exit, dec!(95));
}

#[test]
fn stale_bar_signals_are_suppressed_and_gaps_cancel_pending_entries() {
    let (mut r, buf) = runner(cfg(Session::Crypto24x7), script(&[(0, dec!(95)), (1, dec!(95))], &[]));
    r.on_event(ev(0, dec!(100), dec!(100), dec!(100), dec!(100), 20_000)).unwrap();
    r.on_event(flat(M)).unwrap();
    r.on_event(FeedEvent::Gap { expected_ts: 2 * M, got_ts: 5 * M, missing_bars: 3 }).unwrap();
    r.on_event(flat(5 * M)).unwrap();
    assert!(kinds(&buf, "fill").is_empty());
    let actions: Vec<String> = kinds(&buf, "signal").iter().map(|s| s["action"].as_str().unwrap_or("").to_string()).collect();
    assert_eq!(actions[0], "suppressed: stale bar");
    assert_eq!(kinds(&buf, "cancel")[0]["why"], "feed gap");
    assert_eq!(r.finish().unwrap().counters.missing_bars, 3);
}

#[test]
fn holdout_bars_are_refused() {
    let mut c = cfg(Session::Crypto24x7);
    c.holdout_from = Some(10 * M);
    let (mut r, _) = runner(c, script(&[], &[]));
    r.on_event(flat(9 * M)).unwrap();
    assert!(matches!(r.on_event(flat(10 * M)), Err(LiveError::Holdout(_))));
}

/// 2024-01-02 at `h:m` IST as UTC epoch ms.
fn ist(h: i64, m: i64) -> i64 {
    1_704_153_600_000 + (h * 60 + m) * M - 19_800_000
}

#[test]
fn nse_flattens_at_1515_and_blocks_entries_filling_after_it() {
    let mut c = cfg(Session::Nse);
    c.costs = tq_costs::CostSchedule::zerodha_nse_intraday();
    let (mut r, buf) = runner(c, script(&[(0, dec!(95)), (4, dec!(95))], &[]));
    let five = |h, m| FeedEvent::Bar(match flat(ist(h, m)) {
        FeedEvent::Bar(b) => BarEvent { interval_ms: 5 * M, received_ms: ist(h, m) + 5 * M, ..b },
        other => panic!("{other:?}"),
    });
    for (h, m) in [(15, 0), (15, 5), (15, 10), (15, 15), (15, 20)] {
        r.on_event(five(h, m)).unwrap();
    }
    let out = r.finish().unwrap();
    assert_eq!(out.trades.len(), 1);
    assert_eq!((out.trades[0].reason, out.trades[0].exit_ts), (ExitReason::SessionFlatten, ist(15, 15)));
    let blocked = kinds(&buf, "signal").iter().any(|s| s["action"].as_str().is_some_and(|a| a.contains("flatten time")));
    assert!(blocked);
}

#[test]
fn out_of_session_bars_are_ignored() {
    let (mut r, _) = runner(cfg(Session::Nse), script(&[], &[]));
    r.on_event(flat(ist(15, 35))).unwrap();
    assert_eq!(r.finish().unwrap().counters.out_of_session, 1);
}

#[test]
fn tds_moves_cash_to_a_receivable_without_touching_equity_or_pnl() {
    let mut c = cfg(Session::Crypto24x7);
    c.tds = Some(tq_costs::Tds194S::default());
    c.slippage_bps = dec!(0);
    let (mut r, buf) = runner(c, script(&[(0, dec!(95))], &[1]));
    for i in 0..4 {
        r.on_event(flat(i * M)).unwrap();
    }
    let out = r.finish().unwrap();
    // Sold 100 units at 100 -> 1% TDS = 100; equity = cash + receivable = 100000 - fees.
    assert_eq!(out.tds_receivable, dec!(100));
    assert_eq!(out.final_equity, dec!(100000) - out.trades[0].fees);
    assert_eq!(kinds(&buf, "fill")[1]["tds"], "100");
}

#[test]
fn crypto_daily_series_has_one_point_per_utc_day() {
    let (mut r, _) = runner(cfg(Session::Crypto24x7), script(&[], &[]));
    for ts in [0, 86_400_000 - M, 86_400_000, 2 * 86_400_000] {
        r.on_event(flat(ts)).unwrap();
    }
    assert_eq!(r.finish().unwrap().daily.len(), 3);
}

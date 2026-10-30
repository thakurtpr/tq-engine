//! Cross-sectional rebalancing path over one shared account.
//!
//! Per calendar day: (1) if targets are pending, trade to them at the open: roll the account to
//! the open, sell down (full exits always, trims above [`MIN_TRADE_PCT`] of equity), then buy in
//! `config.priority` order, each buy through `RiskGate::check_rebalance_buy`; (2) mark to market
//! at the close (kill switch on max drawdown); (3) append the close to the price history; (4) on
//! the last trading day of a month (the next calendar day is in another month), ask the
//! rebalancer for targets, which fill at the next open. Liquidity statistics take the day's bars
//! last, so fills only see earlier bars.
//!
//! Targets are sized on equity at the open, less a [`CASH_BUFFER_PCT`] reserve for costs and
//! slippage. Warm-up month ends (before `trade_from`) still reach the rebalancer so its state is
//! complete; their targets only trade if the fill day is inside the window.

mod orders;

use std::collections::HashMap;

use rust_decimal::Decimal;
use tq_broker::Broker;
use tq_core::{Price, Qty, Symbol, Timestamp};
use tq_risk::{AccountState, RiskGate};
use tq_strategy::{weight_sum_ok, History, Rebalancer, Target};

use crate::engine::{DayBar, EntryWindows};
use crate::metrics;
use crate::portfolio::{calendar, validate_series};
use crate::returns::month_key;
use crate::slippage::{Liquidity, LiquidityStats, DEFAULT_LOOKBACK};
use crate::{BacktestError, RunConfig, RunReport, Series};

/// Percent of equity held back from target weights to pay costs and slippage.
pub const CASH_BUFFER_PCT: Decimal = Decimal::from_parts(5, 0, 0, false, 1);
/// Trims and top-ups smaller than this percent of equity are skipped (exits and new positions
/// never are): below it, fixed charges such as the DP fee dominate the trade.
pub const MIN_TRADE_PCT: Decimal = Decimal::from_parts(5, 0, 0, false, 1);

/// A long holding at average cost. Entry costs and slippage not yet charged to a closed trade.
#[derive(Debug, Clone)]
struct Holding {
    qty: Qty,
    avg_price: Price,
    entry_costs: Decimal,
    entry_slippage: Decimal,
    entry_ts: Timestamp,
    last_price: Price,
}

/// Runs `rebalancer` over all `series` in ONE account. `config.sizing` is ignored (weights size
/// every order); `config.entry_windows` must be `None` (membership belongs to each [`Series`]).
pub fn run_rebalance<R: Rebalancer, B: Broker>(
    rebalancer: &mut R,
    broker: &mut B,
    gate: &RiskGate,
    config: &RunConfig,
    series: &[Series],
) -> Result<RunReport, BacktestError> {
    config.validate()?;
    if config.entry_windows.is_some() {
        return Err(BacktestError::InvalidConfig("portfolio runs take entry windows per series"));
    }
    validate_series(series)?;
    let acct = AccountState::fresh(config.initial_cash).map_err(|_| BacktestError::NonPositiveCash)?;
    let windows: EntryWindows =
        series.iter().filter_map(|s| s.entry_windows.clone().map(|w| (s.symbol.clone(), w))).collect();
    let days: Vec<(Timestamp, Vec<DayBar>)> = calendar(series).into_iter().collect();
    let mut book = Book::new(rebalancer, broker, gate, config, acct, windows);
    for (i, (ts, day)) in days.iter().enumerate() {
        let month_end = days.get(i + 1).is_some_and(|next| month_key(next.0) != month_key(*ts));
        book.step(*ts, day, month_end);
    }
    Ok(book.finish())
}

struct Book<'a, R, B> {
    rebalancer: &'a mut R,
    broker: &'a mut B,
    gate: &'a RiskGate,
    config: &'a RunConfig,
    windows: EntryWindows,
    cash: Decimal,
    holdings: HashMap<Symbol, Holding>,
    acct: AccountState,
    /// Equity at the previous close: the start-of-day equity for today's loss limit.
    prev_close: Decimal,
    pending: Option<Vec<Target>>,
    history: History,
    liquidity: HashMap<Symbol, Liquidity>,
    total_costs: Decimal,
    report: RunReport,
}

impl<'a, R: Rebalancer, B: Broker> Book<'a, R, B> {
    fn new(rebalancer: &'a mut R, broker: &'a mut B, gate: &'a RiskGate, config: &'a RunConfig, acct: AccountState, windows: EntryWindows) -> Self {
        Self {
            rebalancer,
            broker,
            gate,
            config,
            windows,
            cash: acct.equity,
            holdings: HashMap::new(),
            prev_close: acct.equity,
            acct,
            pending: None,
            history: History::new(),
            liquidity: HashMap::new(),
            total_costs: Decimal::ZERO,
            report: RunReport::default(),
        }
    }

    fn step(&mut self, ts: Timestamp, day: &[DayBar], month_end: bool) {
        let pending = self.pending.take();
        if ts >= self.config.trade_from {
            if let Some(targets) = pending {
                self.rebalance(ts, day, targets);
            }
            self.mark_to_market(ts, day);
        }
        for d in day {
            self.history.entry(d.symbol.clone()).or_default().push(d.bar.close);
        }
        if month_end {
            self.decide(ts, day);
        }
        let lookback = self.config.slippage_model.as_ref().map_or(DEFAULT_LOOKBACK, |m| m.lookback);
        for d in day {
            self.liquidity.entry(d.symbol.clone()).or_insert_with(|| Liquidity::new(lookback)).push(d.bar);
        }
    }

    fn is_member(&self, symbol: &Symbol, ts: Timestamp) -> bool {
        self.windows.get(symbol).is_none_or(|w| w.iter().any(|&(from, to)| from <= ts && ts < to))
    }

    /// Asks for targets after the close; keeps valid targets on members for the next open.
    fn decide(&mut self, ts: Timestamp, day: &[DayBar]) {
        let members: Vec<Symbol> = day.iter().filter(|d| self.is_member(d.symbol, ts)).map(|d| d.symbol.clone()).collect();
        let targets = self.rebalancer.targets(ts, &members, &self.history);
        if !weight_sum_ok(&targets) {
            let msg = "targets rejected: weights must be positive and sum to at most 1";
            return self.report.rejections.push(msg.to_string());
        }
        let (ok, outside): (Vec<Target>, Vec<Target>) = targets.into_iter().partition(|t| members.contains(&t.0));
        for t in outside {
            self.report.rejections.push(format!("target not an index member: {}", t.0 .0));
        }
        self.pending = Some(ok);
    }

    fn liquidity_of(&self, symbol: &Symbol) -> LiquidityStats {
        self.liquidity.get(symbol).map(Liquidity::stats).unwrap_or_default()
    }

    /// Marks holdings at today's close (others keep their last price); kill switch on drawdown.
    fn mark_to_market(&mut self, ts: Timestamp, day: &[DayBar]) {
        for d in day {
            if let Some(h) = self.holdings.get_mut(d.symbol) {
                h.last_price = d.bar.close;
            }
        }
        let invested: Decimal = self.holdings.values().map(|h| h.qty * h.last_price).sum();
        let equity = self.cash + invested;
        self.roll_account(equity);
        self.prev_close = equity;
        if !self.acct.kill_switch && self.gate.drawdown_breached(&self.acct) {
            self.acct.kill_switch = true;
            self.report.kill_switch_engaged_at = Some(ts);
        }
        self.report.equity_curve.push((ts, equity));
        self.report.exposure_curve.push(invested);
        self.report.open_positions_curve.push(self.holdings.len());
    }

    /// Account state at `equity`, with today's loss measured from the previous close.
    fn roll_account(&mut self, equity: Decimal) {
        self.acct = AccountState {
            equity,
            start_of_day_equity: self.prev_close,
            peak_equity: self.acct.peak_equity.max(equity),
            open_positions: self.holdings.len(),
            kill_switch: self.acct.kill_switch,
        };
    }

    fn finish(self) -> RunReport {
        let r = self.report;
        let metrics = metrics::compute(self.config.initial_cash, &r.equity_curve, &r.trades, self.total_costs);
        RunReport { bars: r.equity_curve.len(), metrics, ..r }
    }
}

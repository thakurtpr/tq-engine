//! Per-day state machine over ONE account: cash, positions, live account state, pending orders.
//!
//! A "day" is every symbol's bar sharing one timestamp. A single-symbol run is the special case
//! of one bar per day, so the per-symbol and portfolio modes share every rule below.

mod fills;

use std::collections::HashMap;

use rust_decimal::Decimal;
use tq_broker::Broker;
use tq_core::{Bar, OrderRequest, Price, Qty, Symbol, Timestamp};
use tq_risk::{AccountState, ApprovedOrder, RiskGate};
use tq_strategy::{Signal, Strategy};

use crate::config::RunConfig;
use crate::metrics;
use crate::report::RunReport;
use crate::slippage::{Liquidity, LiquidityStats, DEFAULT_LOOKBACK};

/// One symbol's bar on a calendar day. `has_next` is false on the symbol's last bar, whose
/// signals could never fill.
pub(crate) struct DayBar<'b> {
    pub(crate) symbol: &'b Symbol,
    pub(crate) bar: &'b Bar,
    pub(crate) has_next: bool,
}

/// Half-open `[from, to)` fill-time windows in which a symbol may be entered.
pub(crate) type EntryWindows = HashMap<Symbol, Vec<(Timestamp, Timestamp)>>;

/// An open long position.
#[derive(Debug, Clone)]
struct Position {
    qty: Qty,
    avg_entry: Price,
    stop: Price,
    /// Unrounded entry-side costs; the trade total is rounded once at exit.
    entry_costs: Decimal,
    entry_ts: Timestamp,
    last_price: Price,
    /// Rupees paid to slippage on the entry fill.
    entry_slippage: Decimal,
}

pub(crate) struct Engine<'a, S, B> {
    strategy: &'a mut S,
    broker: &'a mut B,
    gate: &'a RiskGate,
    config: &'a RunConfig,
    /// Symbols absent from the map may be entered at any time.
    entry_windows: EntryWindows,
    cash: Decimal,
    positions: HashMap<Symbol, Position>,
    acct: AccountState,
    /// Approved entries waiting for their symbol's next open, in priority order.
    pending_entries: Vec<ApprovedOrder>,
    pending_exits: Vec<Symbol>,
    liquidity: HashMap<Symbol, Liquidity>,
    total_costs: Decimal,
    report: RunReport,
}

impl<'a, S: Strategy, B: Broker> Engine<'a, S, B> {
    pub(crate) fn new(
        strategy: &'a mut S,
        broker: &'a mut B,
        gate: &'a RiskGate,
        config: &'a RunConfig,
        acct: AccountState,
        entry_windows: EntryWindows,
    ) -> Self {
        Self {
            strategy,
            broker,
            gate,
            config,
            entry_windows,
            cash: acct.equity,
            positions: HashMap::new(),
            acct,
            pending_entries: Vec::new(),
            pending_exits: Vec::new(),
            liquidity: HashMap::new(),
            total_costs: Decimal::ZERO,
            report: RunReport::default(),
        }
    }

    /// Processes one calendar day: (1) pending exits then entries fill at the open, (2) stops,
    /// (3) mark to market at the close, (4) signals, queued for the next open. Before
    /// `trade_from` the strategy only sees the bars (warm-up). Liquidity statistics take the
    /// day's bars last, so fills only ever see earlier bars.
    pub(crate) fn step(&mut self, ts: Timestamp, day: &[DayBar]) {
        if ts < self.config.trade_from {
            for d in day {
                self.strategy.on_bar(d.symbol, d.bar);
            }
        } else {
            self.fill_pending(day);
            for d in day {
                self.check_stop(d.symbol, d.bar);
            }
            self.mark_to_market(ts, day);
            self.signals(ts, day);
        }
        self.observe(day);
    }

    pub(crate) fn finish(self) -> RunReport {
        let r = self.report;
        let metrics = metrics::compute(self.config.initial_cash, &r.equity_curve, &r.trades, self.total_costs);
        RunReport { bars: r.equity_curve.len(), metrics, ..r }
    }

    fn observe(&mut self, day: &[DayBar]) {
        let lookback = self.config.slippage_model.as_ref().map_or(DEFAULT_LOOKBACK, |m| m.lookback);
        for d in day {
            self.liquidity.entry(d.symbol.clone()).or_insert_with(|| Liquidity::new(lookback)).push(d.bar);
        }
    }

    fn liquidity_of(&self, symbol: &Symbol) -> LiquidityStats {
        self.liquidity.get(symbol).map(Liquidity::stats).unwrap_or_default()
    }

    /// Collects the day's signals; exits queue at once, entries compete in priority order.
    fn signals(&mut self, ts: Timestamp, day: &[DayBar]) {
        let mut candidates = Vec::new();
        for d in day.iter().filter(|d| d.has_next) {
            for signal in self.strategy.on_bar(d.symbol, d.bar) {
                match signal {
                    Signal::Exit(symbol) => self.queue_exit(symbol),
                    Signal::Enter(order) => {
                        self.report.orders_proposed += 1;
                        if &order.symbol == d.symbol {
                            candidates.push(order);
                        } else {
                            let msg = format!("entry ignored: {} is not the bar's symbol {}", order.symbol.0, d.symbol.0);
                            self.report.rejections.push(msg);
                        }
                    }
                }
            }
        }
        self.config.priority.order(ts, &mut candidates);
        for order in candidates {
            self.queue_entry(order);
        }
    }

    /// Gate check at signal time; pending entries count as open positions.
    fn queue_entry(&mut self, order: OrderRequest) {
        let pending = self.pending_entries.iter().any(|a| a.order().symbol == order.symbol);
        if pending || self.positions.contains_key(&order.symbol) {
            self.report.duplicate_entries_ignored += 1;
            return;
        }
        let acct = AccountState { open_positions: self.positions.len() + self.pending_entries.len(), ..self.acct.clone() };
        match self.gate.check(order, &acct) {
            Ok(approved) => self.pending_entries.push(approved),
            Err(v) => self.report.rejections.push(format!("risk: {v}")),
        }
    }

    /// Exits bypass the risk gate and the kill switch by design.
    fn queue_exit(&mut self, symbol: Symbol) {
        if self.positions.contains_key(&symbol) && !self.pending_exits.contains(&symbol) {
            self.pending_exits.push(symbol);
        }
    }

    /// Marks positions with a bar today at its close (others keep their last price) and rolls the
    /// account: the previous close's equity becomes start-of-day equity. Engages the kill switch
    /// once max drawdown is hit.
    fn mark_to_market(&mut self, ts: Timestamp, day: &[DayBar]) {
        for d in day {
            if let Some(p) = self.positions.get_mut(d.symbol) {
                p.last_price = d.bar.close;
            }
        }
        let invested: Decimal = self.positions.values().map(|p| p.qty * p.last_price).sum();
        let equity = self.cash + invested;
        self.acct = AccountState {
            equity,
            start_of_day_equity: self.acct.equity,
            peak_equity: self.acct.peak_equity.max(equity),
            open_positions: self.positions.len(),
            kill_switch: self.acct.kill_switch,
        };
        if !self.acct.kill_switch && self.gate.drawdown_breached(&self.acct) {
            self.acct.kill_switch = true;
            self.report.kill_switch_engaged_at = Some(ts);
        }
        self.report.equity_curve.push((ts, equity));
        self.report.exposure_curve.push(invested);
        self.report.open_positions_curve.push(self.positions.len());
    }
}

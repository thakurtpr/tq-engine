//! Per-bar state machine: positions, cash, live account state and pending orders.

use std::collections::HashMap;
use std::mem;

use rust_decimal::Decimal;
use tq_broker::Broker;
use tq_core::{Bar, Fill, Price, Qty, Symbol, Timestamp};
use tq_costs::round_paise;
use tq_risk::{AccountState, ApprovedOrder, RiskGate};
use tq_strategy::{Signal, Strategy};

use crate::metrics;
use crate::report::{ExitReason, RunReport, Trade};

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
}

pub(crate) struct Engine<'a, S, B> {
    strategy: &'a mut S,
    broker: &'a mut B,
    gate: &'a RiskGate,
    symbol: &'a Symbol,
    cash: Decimal,
    positions: HashMap<Symbol, Position>,
    acct: AccountState,
    pending_entries: Vec<ApprovedOrder>,
    pending_exits: Vec<Symbol>,
    total_costs: Decimal,
    report: RunReport,
}

impl<'a, S: Strategy, B: Broker> Engine<'a, S, B> {
    pub(crate) fn new(strategy: &'a mut S, broker: &'a mut B, gate: &'a RiskGate, symbol: &'a Symbol, acct: AccountState) -> Self {
        Self {
            strategy,
            broker,
            gate,
            symbol,
            cash: acct.equity,
            positions: HashMap::new(),
            acct,
            pending_entries: Vec::new(),
            pending_exits: Vec::new(),
            total_costs: Decimal::ZERO,
            report: RunReport::default(),
        }
    }

    /// Processes one bar. `has_next` is false on the last bar, whose signals could never fill.
    pub(crate) fn step(&mut self, bar: &Bar, has_next: bool) {
        self.fill_pending(bar);
        self.check_stop(bar);
        self.mark_to_market(bar);
        if has_next {
            for signal in self.strategy.on_bar(self.symbol, bar) {
                self.queue(signal);
            }
        }
    }

    pub(crate) fn finish(self, initial_cash: Decimal, bars: usize) -> RunReport {
        let r = self.report;
        let metrics = metrics::compute(initial_cash, &r.equity_curve, &r.trades, self.total_costs);
        RunReport { bars, metrics, ..r }
    }

    fn queue(&mut self, signal: Signal) {
        match signal {
            Signal::Enter(order) => self.queue_entry(order),
            Signal::Exit(symbol) => self.queue_exit(symbol),
        }
    }

    fn queue_entry(&mut self, order: tq_core::OrderRequest) {
        self.report.orders_proposed += 1;
        if &order.symbol != self.symbol {
            self.report.rejections.push(format!("entry ignored: {} is not the backtested symbol", order.symbol.0));
            return;
        }
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

    fn fill_pending(&mut self, bar: &Bar) {
        for symbol in mem::take(&mut self.pending_exits) {
            self.close(&symbol, bar.ts, bar.open, ExitReason::Signal);
        }
        for approved in mem::take(&mut self.pending_entries) {
            self.open(approved, bar);
        }
    }

    fn open(&mut self, approved: ApprovedOrder, bar: &Bar) {
        let approved = match self.gate.reprice_at_fill(approved, bar.open, &self.acct) {
            Ok(a) => a,
            Err(v) => return self.report.rejections.push(format!("risk at fill: {v}")),
        };
        let (symbol, stop) = (approved.order().symbol.clone(), approved.order().stop);
        if approved.order().notional() > self.cash {
            return self.report.rejections.push(format!("insufficient cash for {}", symbol.0));
        }
        let fill = match self.broker.submit(approved, bar.ts, bar.open) {
            Ok(f) => f,
            Err(e) => return self.report.rejections.push(e.to_string()),
        };
        self.cash -= fill.price * fill.qty + fill.costs;
        self.total_costs += fill.costs;
        let position = Position {
            qty: fill.qty,
            avg_entry: fill.price,
            stop,
            entry_costs: fill.costs,
            entry_ts: fill.ts,
            last_price: fill.price,
        };
        self.positions.insert(symbol, position);
        self.on_fill(&fill);
    }

    fn close(&mut self, symbol: &Symbol, ts: Timestamp, price: Price, reason: ExitReason) {
        let Some(pos) = self.positions.remove(symbol) else { return };
        match self.broker.close(symbol, pos.qty, ts, price) {
            Ok(fill) => self.record_trade(pos, &fill, reason),
            Err(e) => {
                self.report.rejections.push(format!("exit failed, position kept: {e}"));
                self.positions.insert(symbol.clone(), pos);
            }
        }
    }

    fn record_trade(&mut self, pos: Position, fill: &Fill, exit_reason: ExitReason) {
        let costs = round_paise(pos.entry_costs + fill.costs);
        let exit_costs = costs - pos.entry_costs;
        let gross_pnl = (fill.price - pos.avg_entry) * fill.qty;
        self.cash += fill.price * fill.qty - exit_costs;
        self.total_costs += exit_costs;
        self.report.trades.push(Trade {
            symbol: fill.symbol.clone(),
            entry_ts: pos.entry_ts,
            entry_price: pos.avg_entry,
            exit_ts: fill.ts,
            exit_price: fill.price,
            qty: fill.qty,
            gross_pnl,
            costs,
            net_pnl: gross_pnl - costs,
            exit_reason,
        });
        self.on_fill(fill);
    }

    fn on_fill(&mut self, fill: &Fill) {
        self.report.orders_filled += 1;
        self.strategy.on_fill(fill);
    }

    /// Stop-loss: if the low touches the stop, exit at `min(open, stop)` (a gap down fills at the open).
    fn check_stop(&mut self, bar: &Bar) {
        let Some(stop) = self.positions.get(self.symbol).map(|p| p.stop) else { return };
        if bar.low <= stop {
            self.close(self.symbol, bar.ts, bar.open.min(stop), ExitReason::StopLoss);
        }
    }

    /// Marks positions at the close and rolls the account: the previous close's equity becomes
    /// start-of-day equity (daily bars). Engages the kill switch once max drawdown is hit.
    fn mark_to_market(&mut self, bar: &Bar) {
        if let Some(p) = self.positions.get_mut(self.symbol) {
            p.last_price = bar.close;
        }
        let equity = self.cash + self.positions.values().map(|p| p.qty * p.last_price).sum::<Decimal>();
        self.acct = AccountState {
            equity,
            start_of_day_equity: self.acct.equity,
            peak_equity: self.acct.peak_equity.max(equity),
            open_positions: self.positions.len(),
            kill_switch: self.acct.kill_switch,
        };
        if !self.acct.kill_switch && self.gate.drawdown_breached(&self.acct) {
            self.acct.kill_switch = true;
            self.report.kill_switch_engaged_at = Some(bar.ts);
        }
        self.report.equity_curve.push((bar.ts, equity));
    }
}

//! Fills: entries sized at the open, exits at the open or the stop, trades recorded with costs
//! and slippage kept apart.

use std::collections::HashMap;
use std::mem;

use rust_decimal::Decimal;
use tq_broker::Broker;
use tq_core::{Bar, Fill, Price, Qty, Side, Symbol, Timestamp};
use tq_costs::round_paise;
use tq_risk::{ApprovedOrder, RiskViolation};
use tq_strategy::Strategy;

use super::{DayBar, Engine, Position};
use crate::config::{risk_sized_qty, Sizing};
use crate::report::{ExitReason, Trade};
use crate::slippage::LiquidityStats;

impl<S: Strategy, B: Broker> Engine<'_, S, B> {
    /// Exits first, then entries, for symbols with a bar today; the rest wait for their next bar.
    pub(super) fn fill_pending(&mut self, day: &[DayBar]) {
        let today: HashMap<&Symbol, &Bar> = day.iter().map(|d| (d.symbol, d.bar)).collect();
        for symbol in mem::take(&mut self.pending_exits) {
            match today.get(&symbol) {
                Some(bar) => self.close(&symbol, bar.ts, bar.open, ExitReason::Signal),
                None => self.pending_exits.push(symbol),
            }
        }
        for approved in mem::take(&mut self.pending_entries) {
            match today.get(&approved.order().symbol) {
                Some(bar) => self.open(approved, bar),
                None => self.pending_entries.push(approved),
            }
        }
    }

    fn entry_allowed(&self, symbol: &Symbol, ts: Timestamp) -> bool {
        self.entry_windows.get(symbol).is_none_or(|w| w.iter().any(|&(from, to)| from <= ts && ts < to))
    }

    fn open(&mut self, approved: ApprovedOrder, bar: &Bar) {
        let symbol = approved.order().symbol.clone();
        if !self.entry_allowed(&symbol, bar.ts) {
            return self.report.entries_outside_universe += 1;
        }
        let liq = self.liquidity_of(&symbol);
        let (approved, price) = match self.size_at_fill(approved, bar.open, &liq) {
            Ok(Some(sized)) => sized,
            Ok(None) => return self.report.entries_sized_to_zero += 1,
            Err(v) => return self.report.rejections.push(format!("risk at fill: {v}")),
        };
        let stop = approved.order().stop;
        if approved.order().notional() > self.cash {
            return self.report.rejections.push(format!("insufficient cash for {}", symbol.0));
        }
        let fill = match self.broker.submit(approved, bar.ts, price) {
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
            entry_slippage: (fill.price - bar.open) * fill.qty,
        };
        self.positions.insert(symbol, position);
        self.on_fill(&fill);
    }

    /// Final size and price at the open. Sizes at the base-slipped price, cuts to the ADV cap
    /// (counted), prices that size (impact grows with size), and re-sizes down if the price rose.
    /// `Ok(None)`: no whole share fits, so the entry is skipped.
    fn size_at_fill(
        &mut self,
        approved: ApprovedOrder,
        open: Price,
        liq: &LiquidityStats,
    ) -> Result<Option<(ApprovedOrder, Price)>, RiskViolation> {
        let base_price = self.config.fill_price(open, Side::Buy, Decimal::ZERO, liq, false);
        let wanted = self.wanted_qty(&approved, base_price);
        let qty = match self.config.adv_cap(liq) {
            Some(cap) if wanted > cap => {
                self.report.entries_adv_capped += 1;
                cap
            }
            _ => wanted,
        };
        let price = self.config.fill_price(open, Side::Buy, qty, liq, false);
        let qty = if price > base_price { self.wanted_qty(&approved, price).min(qty) } else { qty };
        if qty.is_zero() {
            return Ok(None);
        }
        self.gate.resize_at_fill(approved, qty, price, &self.acct).map(|a| Some((a, price)))
    }

    fn wanted_qty(&self, approved: &ApprovedOrder, price: Price) -> Qty {
        match self.config.sizing {
            Sizing::AsSignalled => approved.order().qty,
            Sizing::RiskPct(pct) => {
                let cap = self.gate.limits().max_position_notional_pct;
                risk_sized_qty(self.acct.equity, pct, price, approved.order().stop, cap)
            }
        }
    }

    /// Closes the whole position at `raw_price` less sell-side slippage (2x for stops under
    /// the adopted model).
    fn close(&mut self, symbol: &Symbol, ts: Timestamp, raw_price: Price, reason: ExitReason) {
        let Some(pos) = self.positions.remove(symbol) else { return };
        let liq = self.liquidity_of(symbol);
        let price = self.config.fill_price(raw_price, Side::Sell, pos.qty, &liq, reason == ExitReason::StopLoss);
        match self.broker.close(symbol, pos.qty, ts, price) {
            Ok(fill) => {
                let exit_slippage = (raw_price - fill.price) * fill.qty;
                self.record_trade(pos, &fill, reason, exit_slippage);
            }
            Err(e) => {
                self.report.rejections.push(format!("exit failed, position kept: {e}"));
                self.positions.insert(symbol.clone(), pos);
            }
        }
    }

    fn record_trade(&mut self, pos: Position, fill: &Fill, exit_reason: ExitReason, exit_slippage: Decimal) {
        let costs = round_paise(pos.entry_costs + fill.costs);
        let exit_costs = costs - pos.entry_costs;
        let gross_pnl = (fill.price - pos.avg_entry) * fill.qty;
        self.cash += fill.price * fill.qty - exit_costs;
        self.total_costs += exit_costs;
        self.report.trades.push(Trade {
            symbol: fill.symbol.clone(),
            entry_ts: pos.entry_ts,
            entry_price: pos.avg_entry,
            stop: pos.stop,
            exit_ts: fill.ts,
            exit_price: fill.price,
            qty: fill.qty,
            gross_pnl,
            costs,
            net_pnl: gross_pnl - costs,
            exit_reason,
            slippage: pos.entry_slippage + exit_slippage,
        });
        self.on_fill(fill);
    }

    fn on_fill(&mut self, fill: &Fill) {
        self.report.orders_filled += 1;
        self.strategy.on_fill(fill);
    }

    /// Stop-loss: if the low touches the stop, exit at `min(open, stop)` (a gap down fills at the open).
    pub(super) fn check_stop(&mut self, symbol: &Symbol, bar: &Bar) {
        let Some(stop) = self.positions.get(symbol).map(|p| p.stop) else { return };
        if bar.low <= stop {
            self.close(symbol, bar.ts, bar.open.min(stop), ExitReason::StopLoss);
        }
    }
}

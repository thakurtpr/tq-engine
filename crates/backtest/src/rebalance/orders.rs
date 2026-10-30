//! Trading to target weights at the open: sells first, then buys through the risk gate.
//! Every sell of a holding (full or partial) is recorded as a closed trade at average cost, with
//! the matching share of the entry costs and slippage.

use std::collections::HashMap;

use rust_decimal::Decimal;
use tq_broker::Broker;
use tq_core::{Bar, Fill, Price, Qty, Side, Symbol, Timestamp};
use tq_costs::round_paise;
use tq_risk::AccountState;
use tq_strategy::{Rebalancer, Target};

use super::{Book, Holding, CASH_BUFFER_PCT, MIN_TRADE_PCT};
use crate::engine::DayBar;
use crate::report::{ExitReason, Trade};
use crate::slippage::LiquidityStats;

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;

type Opens<'d> = HashMap<&'d Symbol, &'d Bar>;

impl<R: Rebalancer, B: Broker> Book<'_, R, B> {
    pub(super) fn rebalance(&mut self, ts: Timestamp, day: &[DayBar], targets: Vec<Target>) {
        self.report.rebalances += 1;
        let opens: Opens = day.iter().map(|d| (d.symbol, d.bar)).collect();
        let marked = |(s, h): (&Symbol, &Holding)| h.qty * opens.get(s).map_or(h.last_price, |b| b.open);
        let equity = self.cash + self.holdings.iter().map(marked).sum::<Decimal>();
        self.roll_account(equity);
        let investable = equity * (Decimal::ONE - CASH_BUFFER_PCT / HUNDRED);
        let mut wanted: HashMap<Symbol, Qty> = HashMap::new();
        for (symbol, weight) in targets {
            // Without a bar today the holding is left as it is.
            let qty = opens.get(&symbol).map_or_else(|| self.held(&symbol), |bar| (weight * investable / bar.open).floor());
            wanted.insert(symbol, qty);
        }
        let min_trade = equity * MIN_TRADE_PCT / HUNDRED;
        self.sell_down(&opens, &wanted, min_trade);
        self.buy_up(ts, &opens, &wanted, min_trade);
    }

    fn held(&self, symbol: &Symbol) -> Qty {
        self.holdings.get(symbol).map_or(Decimal::ZERO, |h| h.qty)
    }

    fn sell_down(&mut self, opens: &Opens, wanted: &HashMap<Symbol, Qty>, min_trade: Decimal) {
        let mut held: Vec<(Symbol, Qty)> = self.holdings.iter().map(|(s, h)| (s.clone(), h.qty)).collect();
        held.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
        for (symbol, qty) in held {
            let target = wanted.get(&symbol).copied().unwrap_or(Decimal::ZERO);
            let excess = qty - target;
            if excess <= Decimal::ZERO {
                continue;
            }
            match opens.get(&symbol) {
                Some(bar) if target.is_zero() || excess * bar.open >= min_trade => self.sell(&symbol, excess, bar),
                Some(_) => {}
                None => self.report.rejections.push(format!("sell waits: no bar today for {}", symbol.0)),
            }
        }
    }

    fn buy_up(&mut self, ts: Timestamp, opens: &Opens, wanted: &HashMap<Symbol, Qty>, min_trade: Decimal) {
        let mut buys: Vec<Symbol> = wanted.iter().filter(|(s, q)| **q > self.held(s)).map(|(s, _)| s.clone()).collect();
        self.config.priority.order_symbols(ts, &mut buys);
        for symbol in buys {
            let (Some(bar), Some(target)) = (opens.get(&symbol), wanted.get(&symbol)) else { continue };
            let held = self.held(&symbol);
            let shortfall = *target - held;
            if held.is_zero() || shortfall * bar.open >= min_trade {
                self.buy(&symbol, shortfall, bar);
            }
        }
    }

    /// Caps at 1% of ADV (when ADV is known and positive), prices the order, fits it to cash,
    /// then asks the risk gate.
    fn buy(&mut self, symbol: &Symbol, wanted: Qty, bar: &Bar) {
        let liq = self.liquidity_of(symbol);
        let qty = self.adv_capped(wanted, &liq);
        let price = self.config.fill_price(bar.open, Side::Buy, qty, &liq, false);
        let affordable = (self.cash * (Decimal::ONE - CASH_BUFFER_PCT / HUNDRED) / price).floor().max(Decimal::ZERO);
        let qty = qty.min(affordable);
        if qty.is_zero() {
            return self.report.entries_sized_to_zero += 1;
        }
        let acct = AccountState { open_positions: self.holdings.len(), ..self.acct.clone() };
        let approved = match self.gate.check_rebalance_buy(symbol.clone(), qty, price, self.held(symbol), &acct) {
            Ok(a) => a,
            Err(v) => return self.report.rejections.push(format!("risk: {v}")),
        };
        match self.broker.submit(approved, bar.ts, price) {
            Ok(fill) => self.add_to_holding(&fill, bar.open, &liq),
            Err(e) => self.report.rejections.push(e.to_string()),
        }
    }

    fn adv_capped(&mut self, qty: Qty, liq: &LiquidityStats) -> Qty {
        let Some(model) = &self.config.slippage_model else { return qty };
        if liq.adv.is_none_or(|a| a <= Decimal::ZERO) {
            self.report.fills_without_adv += 1;
            return qty;
        }
        let cap = model.adv_cap(liq);
        if qty > cap {
            self.report.entries_adv_capped += 1;
            return cap;
        }
        qty
    }

    fn add_to_holding(&mut self, fill: &Fill, open: Price, liq: &LiquidityStats) {
        let slippage = (fill.price - open) * fill.qty;
        self.cash -= fill.price * fill.qty + fill.costs;
        self.total_costs += fill.costs;
        self.count_fill(fill, slippage, liq);
        let h = self.holdings.entry(fill.symbol.clone()).or_insert(Holding {
            qty: Decimal::ZERO,
            avg_price: fill.price,
            entry_costs: Decimal::ZERO,
            entry_slippage: Decimal::ZERO,
            entry_ts: fill.ts,
            last_price: fill.price,
        });
        h.avg_price = (h.avg_price * h.qty + fill.price * fill.qty) / (h.qty + fill.qty);
        h.qty += fill.qty;
        h.entry_costs += fill.costs;
        h.entry_slippage += slippage;
    }

    fn sell(&mut self, symbol: &Symbol, qty: Qty, bar: &Bar) {
        let liq = self.liquidity_of(symbol);
        let price = self.config.fill_price(bar.open, Side::Sell, qty, &liq, false);
        match self.broker.close(symbol, qty, bar.ts, price) {
            Ok(fill) => {
                let slippage = (bar.open - fill.price) * fill.qty;
                self.count_fill(&fill, slippage, &liq);
                self.cash += fill.price * fill.qty - fill.costs;
                self.total_costs += fill.costs;
                self.record_sale(&fill, slippage);
            }
            Err(e) => self.report.rejections.push(format!("sell failed, holding kept: {e}")),
        }
    }

    /// Closes `fill.qty` of the holding as one trade; the rest stays at the same average cost.
    fn record_sale(&mut self, fill: &Fill, exit_slippage: Decimal) {
        let Some(h) = self.holdings.get_mut(&fill.symbol) else { return };
        let share = fill.qty / h.qty;
        let (entry_costs, entry_slippage) = (h.entry_costs * share, h.entry_slippage * share);
        let gross_pnl = (fill.price - h.avg_price) * fill.qty;
        let costs = round_paise(entry_costs + fill.costs);
        let trade = Trade {
            symbol: fill.symbol.clone(),
            entry_ts: h.entry_ts,
            entry_price: h.avg_price,
            // No stop: recorded at the entry price so `r_multiple()` is undefined.
            stop: h.avg_price,
            exit_ts: fill.ts,
            exit_price: fill.price,
            qty: fill.qty,
            gross_pnl,
            costs,
            net_pnl: gross_pnl - costs,
            exit_reason: ExitReason::Signal,
            slippage: entry_slippage + exit_slippage,
        };
        h.qty -= fill.qty;
        h.entry_costs -= entry_costs;
        h.entry_slippage -= entry_slippage;
        if h.qty.is_zero() {
            self.holdings.remove(&fill.symbol);
        }
        self.report.trades.push(trade);
    }

    fn count_fill(&mut self, fill: &Fill, slippage: Decimal, liq: &LiquidityStats) {
        self.report.orders_filled += 1;
        self.report.turnover += fill.price * fill.qty;
        self.report.total_slippage += slippage;
        if self.config.above_impact_threshold(fill.qty, liq) {
            self.report.fills_above_impact_threshold += 1;
        }
    }
}

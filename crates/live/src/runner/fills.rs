//! Fills at the bar open (entries and exits), stops, and the session flatten. Entries are sized
//! at the fill price from the risk budget and re-checked by the gate; exits are never gated.

use rust_decimal::Decimal;
use serde_json::json;
use tq_broker::Broker;
use tq_core::{Bar, Fill, Price, Qty, Side, Timestamp};
use tq_risk::{size_for_risk, ApprovedOrder};
use tq_strategy::Strategy;

use super::{ExitReason, Position, Runner, TradeRecord};
use crate::journal::dec;
use crate::session::Session;
use crate::{units, LiveError};

const BPS: Decimal = Decimal::from_parts(10_000, 0, 0, false, 0);
const HUNDRED: Decimal = Decimal::ONE_HUNDRED;

/// `raw` moved against the trader by `bps`: buys pay more, sells receive less.
pub(super) fn slip(raw: Price, side: Side, bps: Decimal) -> Price {
    let f = bps / BPS;
    match side {
        Side::Buy => raw * (Decimal::ONE + f),
        Side::Sell => raw * (Decimal::ONE - f),
    }
}

impl<S: Strategy> Runner<S> {
    pub(super) fn fill_pending(&mut self, bar: &Bar) -> Result<(), LiveError> {
        if std::mem::take(&mut self.pending_exit) {
            self.close(bar.ts, bar.open, ExitReason::Signal)?;
        }
        if let Some(approved) = self.pending_entry.take() {
            self.open(approved, bar)?;
        }
        Ok(())
    }

    /// NSE: flat at 15:15 IST, or at the first bar of a later day if the 15:15 bar was missing.
    pub(super) fn flatten_if_due(&mut self, bar: &Bar) -> Result<(), LiveError> {
        let s = self.cfg.session;
        let Some(entry_ts) = self.pos.as_ref().map(|p| p.entry_ts) else { return Ok(()) };
        let overnight = s == Session::Nse && s.day_key(entry_ts) != s.day_key(bar.ts);
        if s.flatten_due(bar.ts) || overnight {
            self.close(bar.ts, bar.open, ExitReason::SessionFlatten)?;
        }
        Ok(())
    }

    /// Resting stop: exits at `min(open, stop)` when the low touches it (gap down fills at open).
    pub(super) fn check_stop(&mut self, bar: &Bar) -> Result<(), LiveError> {
        let Some(stop) = self.pos.as_ref().map(|p| p.stop) else { return Ok(()) };
        if bar.low <= stop {
            self.close(bar.ts, bar.open.min(stop), ExitReason::StopLoss)?;
        }
        Ok(())
    }

    /// Whole units such that the stop risks at most the per-trade budget and the notional stays
    /// within the position cap, both measured on current equity.
    fn size(&self, price: Price, stop: Price) -> Qty {
        let l = &self.cfg.limits;
        let by_risk = size_for_risk(self.acct.equity, l.max_risk_per_trade_pct, price, stop);
        let by_notional = (self.acct.equity * l.max_position_notional_pct / HUNDRED / price).floor();
        by_risk.min(by_notional).max(Decimal::ZERO)
    }

    fn reject(&mut self, ts: Timestamp, why: String) -> Result<(), LiveError> {
        self.counters.rejected += 1;
        Ok(self.journal.record("signal", ts, json!({"kind": "enter", "action": format!("rejected at fill: {why}")}))?)
    }

    fn open(&mut self, approved: ApprovedOrder, bar: &Bar) -> Result<(), LiveError> {
        let price = slip(bar.open, Side::Buy, self.cfg.slippage_bps);
        let stop = approved.order().stop;
        let qty = self.size(price, stop);
        if qty.is_zero() {
            return self.reject(bar.ts, "sized to zero units".into());
        }
        let approved = match self.gate.resize_at_fill(approved, qty, price, &self.acct) {
            Ok(a) => a,
            Err(v) => return self.reject(bar.ts, v.to_string()),
        };
        let est_fee = self.cfg.costs.compute(Side::Buy, price * qty).total();
        if price * qty + est_fee > self.cash {
            return self.reject(bar.ts, format!("insufficient cash {} (TDS receivable {})", self.cash, self.tds.receivable()));
        }
        let fill = match self.broker.submit(approved, bar.ts, price) {
            Ok(f) => f,
            Err(e) => return self.reject(bar.ts, e.to_string()),
        };
        self.cash -= fill.price * fill.qty + fill.costs;
        let pos = Position { qty, entry_fill: price, raw_entry: bar.open, stop, entry_fee: fill.costs, entry_ts: bar.ts, last_price: price };
        self.pos = Some(pos);
        self.journal_fill(&fill, bar.open, Decimal::ZERO, "entry")?;
        self.strategy.on_fill(&fill);
        Ok(())
    }

    pub(super) fn close(&mut self, ts: Timestamp, raw: Price, reason: ExitReason) -> Result<(), LiveError> {
        let Some(p) = self.pos.take() else { return Ok(()) };
        let price = slip(raw, Side::Sell, self.cfg.slippage_bps);
        let fill = match self.broker.close(&self.cfg.symbol, p.qty, ts, price) {
            Ok(f) => f,
            Err(e) => {
                self.pos = Some(p);
                return Ok(self.journal.record("status", ts, json!({"msg": format!("exit failed, position kept: {e}")}))?);
            }
        };
        let tds = self.tds.on_sell(fill.price * fill.qty);
        self.cash += fill.price * fill.qty - fill.costs - tds;
        self.pending_exit = false;
        let trade = trade_record(&p, &fill, raw, tds, reason);
        self.journal_fill(&fill, raw, tds, reason.as_str())?;
        self.journal_trade(&trade)?;
        self.trades.push(trade);
        self.strategy.on_fill(&fill);
        Ok(())
    }

    fn journal_fill(&mut self, fill: &Fill, raw: Price, tds: Decimal, reason: &str) -> Result<(), LiveError> {
        let lot = self.cfg.lot;
        let side = if fill.side == Side::Buy { "buy" } else { "sell" };
        let fields = json!({
            "side": side, "reason": reason, "units": dec(fill.qty), "qty": dec(units::base_qty(fill.qty, lot)),
            "price": dec(units::base_price(fill.price, lot)), "raw_price": dec(units::base_price(raw, lot)),
            "notional": dec(fill.price * fill.qty), "fee": dec(fill.costs), "tds": dec(tds), "cash_after": dec(self.cash),
        });
        Ok(self.journal.record("fill", fill.ts, fields)?)
    }

    fn journal_trade(&mut self, t: &TradeRecord) -> Result<(), LiveError> {
        let fields = json!({
            "entry_ts": t.entry_ts, "reason": t.reason.as_str(), "gross": dec(t.gross_pnl), "fees": dec(t.fees),
            "slippage": dec(t.slippage), "net": dec(t.net_pnl), "notional": dec(t.notional()),
        });
        Ok(self.journal.record("trade", t.exit_ts, fields)?)
    }
}

fn trade_record(p: &Position, fill: &Fill, raw_exit: Price, tds: Decimal, reason: ExitReason) -> TradeRecord {
    let gross_pnl = (raw_exit - p.raw_entry) * p.qty;
    let fees = p.entry_fee + fill.costs;
    let filled_pnl = (fill.price - p.entry_fill) * p.qty;
    TradeRecord {
        entry_ts: p.entry_ts,
        exit_ts: fill.ts,
        qty: p.qty,
        raw_entry: p.raw_entry,
        raw_exit,
        entry_fill: p.entry_fill,
        exit_fill: fill.price,
        gross_pnl,
        fees,
        slippage: gross_pnl - filled_pnl,
        net_pnl: filled_pnl - fees,
        tds,
        reason,
    }
}

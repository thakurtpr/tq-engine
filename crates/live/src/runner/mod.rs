//! The paper runner's event loop over ONE symbol and ONE account.
//!
//! Per closed bar, in order: (1) refuse held-out bars, skip bars outside the session; (2) roll
//! the session day (start-of-day equity for the daily-loss limit); (3) drop a pending entry if
//! the bar is stale or follows a feed gap; (4) on a fresh bar, fill the pending exit then the
//! pending entry at this bar's OPEN (entries are cancelled once the session's flatten time is
//! reached); (5) forced session flatten at the open; (6) stop check: low <= stop exits at
//! `min(open, stop)`, as in `tq-backtest` (a resting stop, so it applies to stale bars too);
//! (7) mark to market at the close, engaging the kill switch on max drawdown; (8) ask the
//! strategy; entries pass the risk gate now and again at the fill price. Signals from stale bars
//! are journaled and discarded: the runner never trades on a stale bar.

mod fills;
#[cfg(test)]
mod tests;
mod trade;

pub use trade::{ExitReason, TradeRecord};

use std::time::SystemTime;

use rust_decimal::Decimal;
use serde_json::json;
use tq_broker::Paper;
use tq_core::{Bar, OrderRequest, Price, Qty, Symbol, Timestamp};
use tq_costs::{CostSchedule, Tds194S, TdsLedger};
use tq_feeds::{BarEvent, BarStream, FeedEvent, Source};
use tq_risk::{AccountState, ApprovedOrder, RiskGate, RiskLimits};
use tq_strategy::{Signal, Strategy};

use crate::journal::{dec, Journal};
use crate::session::Session;
use crate::{units, LiveError};

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub symbol: Symbol,
    /// Base-asset quantity of one unit (see `crate::units`); 1 for shares.
    pub lot: Decimal,
    pub initial_cash: Decimal,
    /// Adverse slippage per side, in bps of the raw fill price.
    pub slippage_bps: Decimal,
    pub costs: CostSchedule,
    pub limits: RiskLimits,
    pub session: Session,
    /// A bar received more than this long after its close is stale.
    pub max_stale_ms: i64,
    /// Journal an equity mark every N bars (1 = every bar).
    pub mark_every: u64,
    /// Indian-resident 1% TDS on sells (cash withheld, booked as a receivable).
    pub tds: Option<Tds194S>,
    /// Bars at or after this time are refused (the held-out period).
    pub holdout_from: Option<Timestamp>,
}

#[derive(Debug, Clone)]
struct Position {
    qty: Qty,
    entry_fill: Price,
    raw_entry: Price,
    stop: Price,
    entry_fee: Decimal,
    entry_ts: Timestamp,
    last_price: Price,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counters {
    pub bars: u64,
    pub out_of_session: u64,
    pub stale_bars: u64,
    pub gaps: u64,
    pub missing_bars: i64,
    pub entry_signals: u64,
    pub rejected: u64,
    pub suppressed: u64,
    pub cancelled: u64,
}

/// Everything a report needs once the run ends.
#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub trades: Vec<TradeRecord>,
    /// (session day key, equity at that day's last bar).
    pub daily: Vec<(i64, Decimal)>,
    pub initial_cash: Decimal,
    pub final_equity: Decimal,
    pub tds_receivable: Decimal,
    pub kill_switch_at: Option<Timestamp>,
    pub open_at_end: bool,
    pub counters: Counters,
}

pub struct Runner<S> {
    cfg: RunnerConfig,
    strategy: S,
    gate: RiskGate,
    broker: Paper,
    journal: Journal,
    cash: Decimal,
    acct: AccountState,
    pos: Option<Position>,
    pending_entry: Option<ApprovedOrder>,
    pending_exit: bool,
    gap_pending: bool,
    day: Option<i64>,
    tds: TdsLedger,
    trades: Vec<TradeRecord>,
    daily: Vec<(i64, Decimal)>,
    kill_switch_at: Option<Timestamp>,
    counters: Counters,
}

impl<S: Strategy> Runner<S> {
    pub fn new(cfg: RunnerConfig, strategy: S, journal: Journal) -> Result<Self, LiveError> {
        let acct = AccountState::fresh(cfg.initial_cash).map_err(|e| LiveError::Config(e.to_string()))?;
        if cfg.lot <= Decimal::ZERO || cfg.slippage_bps < Decimal::ZERO || cfg.mark_every == 0 {
            return Err(LiveError::Config("lot and mark_every must be positive, slippage non-negative".into()));
        }
        let tds = TdsLedger::new(cfg.tds.clone().unwrap_or(Tds194S { rate_pct: Decimal::ZERO }));
        Ok(Self {
            gate: RiskGate::new(cfg.limits.clone()),
            broker: Paper::new(cfg.costs.clone()),
            cash: cfg.initial_cash,
            cfg,
            strategy,
            journal,
            acct,
            pos: None,
            pending_entry: None,
            pending_exit: false,
            gap_pending: false,
            day: None,
            tds,
            trades: Vec::new(),
            daily: Vec::new(),
            kill_switch_at: None,
            counters: Counters::default(),
        })
    }

    pub fn journal(&mut self) -> &mut Journal {
        &mut self.journal
    }

    pub fn on_event(&mut self, ev: FeedEvent) -> Result<(), LiveError> {
        match ev {
            FeedEvent::Bar(b) => self.on_bar(&b),
            FeedEvent::Gap { expected_ts, got_ts, missing_bars } => {
                self.counters.gaps += 1;
                self.counters.missing_bars += missing_bars;
                self.gap_pending = true;
                let fields = json!({"expected_ts": expected_ts, "got_ts": got_ts, "missing_bars": missing_bars});
                Ok(self.journal.record("gap", got_ts, fields)?)
            }
            FeedEvent::Status(msg) => Ok(self.journal.record("status", now_ms(), json!({"msg": msg}))?),
        }
    }

    pub fn on_bar(&mut self, ev: &BarEvent) -> Result<(), LiveError> {
        if self.cfg.holdout_from.is_some_and(|h| ev.bar.ts >= h) {
            return Err(LiveError::Holdout(ev.bar.ts));
        }
        if !self.cfg.session.in_session(ev.bar.ts) {
            self.counters.out_of_session += 1;
            return Ok(());
        }
        let bar = units::to_units(&ev.bar, self.cfg.lot)?;
        let fresh = ev.lateness_ms() <= self.cfg.max_stale_ms;
        self.roll_day(&bar)?;
        self.screen(ev, fresh)?;
        if fresh {
            self.fill_pending(&bar)?;
            self.flatten_if_due(&bar)?;
        }
        self.check_stop(&bar)?;
        self.mark(&bar)?;
        self.signals(&bar, ev.close_ts(), fresh)?;
        self.counters.bars += 1;
        Ok(())
    }

    /// Cancels the pending entry after a gap, on a stale bar, or at the session flatten time.
    fn screen(&mut self, ev: &BarEvent, fresh: bool) -> Result<(), LiveError> {
        let ts = ev.bar.ts;
        if !fresh {
            self.counters.stale_bars += 1;
            if ev.source != Source::Backfill {
                self.journal.record("stale", ts, json!({"lateness_ms": ev.lateness_ms(), "source": ev.source.as_str()}))?;
            }
            self.cancel_entry(ts, "stale bar")?;
        }
        if std::mem::take(&mut self.gap_pending) {
            self.cancel_entry(ts, "feed gap")?;
        }
        if self.cfg.session.flatten_due(ts) {
            self.cancel_entry(ts, "session flatten time")?;
        }
        Ok(())
    }

    fn cancel_entry(&mut self, ts: Timestamp, why: &str) -> Result<(), LiveError> {
        if self.pending_entry.take().is_some() {
            self.counters.cancelled += 1;
            self.journal.record("cancel", ts, json!({"order": "entry", "why": why}))?;
        }
        Ok(())
    }

    fn roll_day(&mut self, bar: &Bar) -> Result<(), LiveError> {
        let key = self.cfg.session.day_key(bar.ts);
        match self.day {
            Some(d) if d == key => return Ok(()),
            Some(d) => self.daily.push((d, self.acct.equity)),
            None => {}
        }
        self.day = Some(key);
        self.acct.start_of_day_equity = self.acct.equity;
        Ok(self.journal.record("session", bar.ts, json!({"day": key, "start_equity": dec(self.acct.equity)}))?)
    }

    fn mark(&mut self, bar: &Bar) -> Result<(), LiveError> {
        if let Some(p) = self.pos.as_mut() {
            p.last_price = bar.close;
        }
        let invested = self.pos.as_ref().map_or(Decimal::ZERO, |p| p.qty * p.last_price);
        let equity = self.cash + invested + self.tds.receivable();
        self.acct.equity = equity;
        self.acct.peak_equity = self.acct.peak_equity.max(equity);
        self.acct.open_positions = usize::from(self.pos.is_some());
        if !self.acct.kill_switch && self.gate.drawdown_breached(&self.acct) {
            self.acct.kill_switch = true;
            self.kill_switch_at = Some(bar.ts);
            self.journal.record("status", bar.ts, json!({"msg": "kill switch engaged: max drawdown reached"}))?;
        }
        if self.counters.bars.is_multiple_of(self.cfg.mark_every) {
            let fields = json!({
                "equity": dec(equity), "cash": dec(self.cash), "tds_receivable": dec(self.tds.receivable()),
                "units": dec(self.pos.as_ref().map_or(Decimal::ZERO, |p| p.qty)),
                "close": dec(units::base_price(bar.close, self.cfg.lot)),
            });
            self.journal.record("mark", bar.ts, fields)?;
        }
        Ok(())
    }

    fn signals(&mut self, bar: &Bar, next_open: Timestamp, fresh: bool) -> Result<(), LiveError> {
        for signal in self.strategy.on_bar(&self.cfg.symbol, bar) {
            match signal {
                Signal::Exit(_) if self.pos.is_some() && !self.pending_exit => {
                    let action = if fresh { "queued" } else { "queued (fills at next fresh bar)" };
                    self.pending_exit = true;
                    self.journal.record("signal", bar.ts, json!({"kind": "exit", "action": action}))?;
                }
                Signal::Exit(_) => {}
                Signal::Enter(order) => self.on_enter(order, bar.ts, next_open, fresh)?,
            }
        }
        Ok(())
    }

    fn on_enter(&mut self, order: OrderRequest, ts: Timestamp, next_open: Timestamp, fresh: bool) -> Result<(), LiveError> {
        if self.pos.is_some() || self.pending_entry.is_some() {
            return Ok(());
        }
        self.counters.entry_signals += 1;
        let lot = self.cfg.lot;
        let mut fields = json!({"kind": "enter", "entry": dec(units::base_price(order.entry, lot)), "stop": dec(units::base_price(order.stop, lot))});
        let action = if !fresh {
            self.counters.suppressed += 1;
            "suppressed: stale bar".to_string()
        } else if self.cfg.session.flatten_due(next_open) {
            self.counters.suppressed += 1;
            "suppressed: fill would be at or after the session flatten time".to_string()
        } else {
            match self.gate.check(order, &self.acct) {
                Ok(approved) => {
                    self.pending_entry = Some(approved);
                    "queued for next open".to_string()
                }
                Err(v) => {
                    self.counters.rejected += 1;
                    format!("rejected: {v}")
                }
            }
        };
        fields["action"] = json!(action);
        Ok(self.journal.record("signal", ts, fields)?)
    }

    pub fn finish(mut self) -> Result<RunOutcome, LiveError> {
        if let Some(d) = self.day {
            self.daily.push((d, self.acct.equity));
        }
        self.journal.flush()?;
        Ok(RunOutcome {
            trades: self.trades,
            daily: self.daily,
            initial_cash: self.cfg.initial_cash,
            final_equity: self.acct.equity,
            tds_receivable: self.tds.receivable(),
            kill_switch_at: self.kill_switch_at,
            open_at_end: self.pos.is_some(),
            counters: self.counters,
        })
    }
}

fn now_ms() -> Timestamp {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// Drives `runner` from `feed` until the feed ends or the wall-clock `deadline` passes. Wall
/// clock, not `Instant`: on macOS `Instant` stops while the machine sleeps.
pub fn run<S: Strategy, F: BarStream>(runner: &mut Runner<S>, feed: &mut F, deadline: Option<SystemTime>) -> Result<(), LiveError> {
    while let Some(ev) = feed.next_event()? {
        runner.on_event(ev)?;
        if deadline.is_some_and(|d| SystemTime::now() >= d) {
            break;
        }
    }
    Ok(())
}

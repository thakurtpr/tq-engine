//! Backtest output: trades, equity curve and summary metrics.

use rust_decimal::Decimal;
use tq_core::{Price, Qty, Symbol, Timestamp};

use crate::metrics::Metrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// The strategy emitted `Signal::Exit`; filled at the next bar's open.
    Signal,
    /// The bar's low touched the stop; filled at `min(open, stop)`.
    StopLoss,
}

/// One closed round trip. `costs` is rounded once, to paise, for the whole trade.
#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub symbol: Symbol,
    pub entry_ts: Timestamp,
    pub entry_price: Price,
    pub exit_ts: Timestamp,
    pub exit_price: Price,
    pub qty: Qty,
    pub gross_pnl: Decimal,
    pub costs: Decimal,
    pub net_pnl: Decimal,
    pub exit_reason: ExitReason,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RunReport {
    pub bars: usize,
    pub orders_proposed: usize,
    /// Entry and exit fills.
    pub orders_filled: usize,
    /// `Enter` signals ignored because a position (or pending entry) was already open.
    pub duplicate_entries_ignored: usize,
    pub rejections: Vec<String>,
    /// Bar at which the drawdown limit engaged the kill switch, if it did.
    pub kill_switch_engaged_at: Option<Timestamp>,
    /// Mark-to-market equity at each bar's close.
    pub equity_curve: Vec<(Timestamp, Decimal)>,
    /// Closed trades only; a position still open at the end is in the equity curve, not here.
    pub trades: Vec<Trade>,
    pub metrics: Metrics,
}

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
    /// Stop at entry; `(entry_price - stop) * qty` is the trade's initial risk (1R).
    pub stop: Price,
    pub exit_ts: Timestamp,
    pub exit_price: Price,
    pub qty: Qty,
    /// P&L at the filled prices, so it is already AFTER slippage.
    pub gross_pnl: Decimal,
    pub costs: Decimal,
    pub net_pnl: Decimal,
    pub exit_reason: ExitReason,
    /// Rupees lost to slippage on both fills versus the raw open/stop prices (>= 0).
    /// `gross_pnl + slippage` is the move before any friction.
    pub slippage: Decimal,
}

impl Trade {
    /// Net P&L in multiples of the initial risk. `None` if the stop was at the entry price.
    pub fn r_multiple(&self) -> Option<Decimal> {
        let risk = (self.entry_price - self.stop) * self.qty;
        (!risk.is_zero()).then(|| self.net_pnl / risk)
    }

    /// Net P&L as a percent of the entry notional (zero for a zero notional).
    pub fn net_return_pct(&self) -> Decimal {
        let notional = self.entry_price * self.qty;
        if notional.is_zero() {
            return Decimal::ZERO;
        }
        self.net_pnl / notional * Decimal::ONE_HUNDRED
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RunReport {
    pub bars: usize,
    pub orders_proposed: usize,
    /// Entry and exit fills.
    pub orders_filled: usize,
    /// `Enter` signals ignored because a position (or pending entry) was already open.
    pub duplicate_entries_ignored: usize,
    /// Approved entries skipped at the fill because the risk budget bought zero whole shares.
    pub entries_sized_to_zero: usize,
    /// Approved entries skipped at the fill because the fill date was outside `entry_windows`.
    pub entries_outside_universe: usize,
    /// Entries cut down to the liquidity cap (1% of 20-day ADV); see `slippage.rs`.
    pub entries_adv_capped: usize,
    pub rejections: Vec<String>,
    /// Bar at which the drawdown limit engaged the kill switch, if it did.
    pub kill_switch_engaged_at: Option<Timestamp>,
    /// Mark-to-market equity at each bar's close.
    pub equity_curve: Vec<(Timestamp, Decimal)>,
    /// Market value of open positions at each bar's close (same timestamps as `equity_curve`).
    pub exposure_curve: Vec<Decimal>,
    /// Open positions at each bar's close.
    pub open_positions_curve: Vec<usize>,
    /// Closed trades only; a position still open at the end is in the equity curve, not here.
    pub trades: Vec<Trade>,
    pub metrics: Metrics,
}

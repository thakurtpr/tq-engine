//! A closed round trip, with gross (no fees, no slippage), fees and slippage kept apart.

use rust_decimal::Decimal;
use tq_core::{Price, Qty, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    Signal,
    StopLoss,
    /// NSE forced flatten at 15:15 IST (or at the first bar of a new day if 15:15 was missed).
    SessionFlatten,
}

impl ExitReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Signal => "signal",
            Self::StopLoss => "stop",
            Self::SessionFlatten => "session_flatten",
        }
    }
}

/// Prices are per UNIT (see `crate::units`); money fields are in the quote currency.
#[derive(Debug, Clone, PartialEq)]
pub struct TradeRecord {
    pub entry_ts: Timestamp,
    pub exit_ts: Timestamp,
    pub qty: Qty,
    /// Bar open the entry filled against, before slippage.
    pub raw_entry: Price,
    pub raw_exit: Price,
    /// Actual fill prices, after slippage.
    pub entry_fill: Price,
    pub exit_fill: Price,
    /// `(raw_exit - raw_entry) * qty`: what the move paid before any friction.
    pub gross_pnl: Decimal,
    pub fees: Decimal,
    pub slippage: Decimal,
    /// `(exit_fill - entry_fill) * qty - fees`.
    pub net_pnl: Decimal,
    /// TDS withheld on the exit (cash, reclaimable); not part of `net_pnl`.
    pub tds: Decimal,
    pub reason: ExitReason,
}

impl TradeRecord {
    /// Entry notional at the raw price: the base for per-trade bps figures.
    pub fn notional(&self) -> Decimal {
        self.raw_entry * self.qty
    }
}

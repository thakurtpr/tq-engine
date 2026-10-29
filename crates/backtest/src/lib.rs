//! Event-driven backtester over daily bars, long-only.
//!
//! Per bar, in order: (1) fill pending exits, then pending entries, at the open;
//! (2) check stops against the low; (3) mark to market at the close and update the live
//! `AccountState` (engaging the kill switch on max drawdown); (4) ask the strategy for signals,
//! which fill at the NEXT bar's open, so a strategy never trades on the bar it just saw.
//! Entries pass the risk gate twice (at signal time and at the fill price); exits never do.
//! Next: slippage, walk-forward and Monte Carlo.

mod engine;
pub mod metrics;
mod report;

use rust_decimal::Decimal;
use thiserror::Error;
use tq_broker::Broker;
use tq_core::{Bar, Symbol};
use tq_risk::{AccountState, RiskGate};
use tq_strategy::Strategy;

use crate::engine::Engine;
pub use crate::metrics::Metrics;
pub use crate::report::{ExitReason, RunReport, Trade};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BacktestError {
    #[error("initial cash must be positive")]
    NonPositiveCash,
    #[error("bars must be in strictly increasing timestamp order")]
    BarsOutOfOrder,
}

/// Runs `strategy` over `bars` of one `symbol`, starting with `initial_cash` and no positions.
pub fn run<S: Strategy, B: Broker>(
    strategy: &mut S,
    broker: &mut B,
    gate: &RiskGate,
    initial_cash: Decimal,
    symbol: &Symbol,
    bars: &[Bar],
) -> Result<RunReport, BacktestError> {
    if bars.windows(2).any(|w| w[0].ts >= w[1].ts) {
        return Err(BacktestError::BarsOutOfOrder);
    }
    let acct = AccountState::fresh(initial_cash).map_err(|_| BacktestError::NonPositiveCash)?;
    let mut engine = Engine::new(strategy, broker, gate, symbol, acct);
    for (i, bar) in bars.iter().enumerate() {
        engine.step(bar, i + 1 < bars.len());
    }
    Ok(engine.finish(initial_cash, bars.len()))
}

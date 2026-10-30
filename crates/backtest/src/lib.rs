//! Event-driven backtester over daily bars, long-only, one shared account.
//!
//! Per calendar day, in order: (1) fill pending exits, then pending entries, at the open;
//! (2) check stops against the low; (3) mark to market at the close and update the live
//! `AccountState` (engaging the kill switch on max drawdown); (4) ask the strategy for signals,
//! which fill at the NEXT bar's open, so a strategy never trades on the bar it just saw.
//! Entries pass the risk gate twice (at signal time and at the fill price); exits never do.
//! Entries are sized at the fill from the run's risk budget ([`Sizing`]) and every fill pays
//! slippage ([`RunConfig::slippage_model`]).
//!
//! [`run_portfolio`] steps many symbols on one date calendar with ONE cash balance and ONE
//! `AccountState`, so the position cap, daily loss limit and kill switch bind across the
//! portfolio. [`run`] is the one-symbol case (a diagnostic, not a performance claim).
//!
//! [`run_rebalance`] is the cross-sectional path: a `Rebalancer` names target weights after each
//! month-end close and the engine trades to them at the next open (sells first, then buys
//! through the risk gate), paying the same costs and slippage.

mod config;
mod engine;
pub mod metrics;
mod portfolio;
mod priority;
mod rebalance;
mod report;
pub mod returns;
pub mod tstats;
mod slippage;

use thiserror::Error;
use tq_broker::Broker;
use tq_core::{Bar, Symbol};
use tq_risk::RiskGate;
use tq_strategy::Strategy;

pub use crate::config::{RunConfig, Sizing, DEFAULT_RISK_PCT, DEFAULT_SLIPPAGE_BPS};
pub use crate::metrics::Metrics;
pub use crate::portfolio::{run_portfolio, Series};
pub use crate::priority::Priority;
pub use crate::rebalance::{run_rebalance, CASH_BUFFER_PCT, MIN_TRADE_PCT};
pub use crate::report::{ExitReason, RunReport, Trade};
pub use crate::slippage::{Liquidity, LiquidityStats, SlippageModel};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BacktestError {
    #[error("initial cash must be positive")]
    NonPositiveCash,
    #[error("bars must be in strictly increasing timestamp order")]
    BarsOutOfOrder,
    #[error("invalid run config: {0}")]
    InvalidConfig(&'static str),
}

/// Runs `strategy` over `bars` of one `symbol`, starting with `config.initial_cash` and no
/// positions. Bars before `config.trade_from` only warm the strategy up.
pub fn run<S: Strategy, B: Broker>(
    strategy: &mut S,
    broker: &mut B,
    gate: &RiskGate,
    config: &RunConfig,
    symbol: &Symbol,
    bars: &[Bar],
) -> Result<RunReport, BacktestError> {
    let windows = config.entry_windows.clone();
    let config = RunConfig { entry_windows: None, ..config.clone() };
    let series = [Series { symbol: symbol.clone(), bars: bars.to_vec(), entry_windows: windows }];
    run_portfolio(strategy, broker, gate, &config, &series)
}

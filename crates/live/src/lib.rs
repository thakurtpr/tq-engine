//! Intraday PAPER-trading runner. One event loop consumes a [`tq_feeds::BarStream`] (historical
//! replay or a live public feed) and drives the same `Strategy` trait, `RiskGate`, `Paper` broker
//! and cost schedules as the backtester. It never talks to an exchange's order API.
//!
//! Rules (see [`runner`]): signals fill at the next bar's open, stops behave as in
//! `tq-backtest` (exit at `min(open, stop)` when the low touches the stop), stale or
//! post-gap bars never open positions, the session decides the daily-loss day and any forced
//! flatten, and every signal, fill, trade and equity mark is appended to a JSONL journal.

pub mod journal;
pub mod runner;
pub mod session;
pub mod stats;
pub mod strategies;
pub mod units;

use thiserror::Error;
use tq_core::Timestamp;

/// 2024-01-01T00:00:00Z. Crypto bars from here on are the pre-registered holdout for the
/// intraday strategies: replays refuse to read them (reports/intraday-01.md).
pub const HOLDOUT_START_MS: Timestamp = 1_704_067_200_000;

#[derive(Debug, Error)]
pub enum LiveError {
    #[error("feed: {0}")]
    Feed(#[from] tq_feeds::FeedError),
    #[error("journal: {0}")]
    Journal(#[from] std::io::Error),
    #[error("bar at {0} is inside the held-out period (>= 2024-01-01 UTC); refusing to replay it")]
    Holdout(Timestamp),
    #[error("invalid bar after unit scaling: {0}")]
    Bar(#[from] tq_core::CoreError),
    #[error("invalid config: {0}")]
    Config(String),
}

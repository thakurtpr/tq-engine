//! Concrete strategies. Owned by the strategy engineer.
//!
//! The swing strategies (`Strategy`) are textbook long-only systems used to validate the engine
//! end to end. The rebalancers (`Rebalancer`) are pre-registered cross-sectional and index
//! strategies with literature parameters (backtest-04), plus buy-and-hold as the benchmark.
//! None of them has a proven edge: treat each as unproven until it passes walk-forward
//! testing net of the full Indian cost model.
//!
//! Conventions shared by every strategy here:
//! - Indicator state is kept per symbol, so one instance can trade a whole universe.
//! - Entries use the bar's close as the reference price; the engine fills at the next open.
//! - Quantity is [`PLACEHOLDER_QTY`]; the engine resizes every entry from the stop distance.
//! - A position opens only when a `Buy` fill arrives via `on_fill` (so a risk rejection
//!   leaves the strategy flat) and closes as soon as the strategy emits `Exit`, because exits
//!   are never blocked by risk.
//! - Only the bar passed to `on_bar` and earlier bars are ever used (no look-ahead).

mod book;
mod buy_and_hold;
mod donchian_breakout;
mod index_trend_filter;
mod momentum_rotation;
mod trend_pullback;

#[cfg(test)]
mod test_support;

pub use book::PLACEHOLDER_QTY;
pub use buy_and_hold::BuyAndHold;
pub use donchian_breakout::{DonchianBreakout, DonchianBreakoutConfig};
pub use index_trend_filter::{IndexTrendFilter, IndexTrendFilterConfig};
pub use momentum_rotation::{MomentumRotation, MomentumRotationConfig};
pub use trend_pullback::{TrendPullback, TrendPullbackConfig};

use std::fmt;

/// A strategy config that cannot produce a working strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub &'static str);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid strategy config: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

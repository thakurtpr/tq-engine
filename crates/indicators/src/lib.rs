//! Streaming indicators. Each update is O(1) (amortised for rolling extremes) so the same
//! code runs on daily bars and ticks. Every `update` returns `None` until the indicator has
//! seen enough data to be meaningful, and every constructor returns `None` for a zero period.

mod atr;
mod ema;
mod rolling;
mod rsi;
mod sma;

pub use atr::Atr;
pub use ema::Ema;
pub use rolling::{RollingHigh, RollingLow};
pub use rsi::Rsi;
pub use sma::Sma;

//! PRE-REGISTERED intraday strategies (2026-10-30). Parameters are fixed constants in each
//! module and must not be tuned: reports/intraday-01.md tests them exactly as written here.
//!
//! - [`VwapReversion`] (a): 1m, long when close < session VWAP - 2 x 20-bar stdev of closes;
//!   exit when close >= VWAP or after 30 bars held; stop entry - 1.5 x ATR(14).
//! - [`OpeningRangeBreakout`] (b): NSE 5m, long when close > the 09:15-09:30 high, stop at the
//!   range low, at most one entry per day, flat at 15:15 (the runner flattens).
//! - [`MomentumBurst`] (c): 1m, long when the 5-bar return > 3 x stdev of the previous 100
//!   5-bar returns and volume > 2 x the previous 100 bars' mean volume; exit after 10 bars held;
//!   stop entry - 1 x ATR(14).
//!
//! All are long-only (the risk gate rejects shorts). Each proposes one unit; the runner sizes at
//! the fill. Statistics (stdev, means) are `f64`; prices and stops stay `Decimal`.

mod momentum;
mod orb;
mod vwap;

pub use momentum::MomentumBurst;
pub use orb::OpeningRangeBreakout;
pub use vwap::VwapReversion;

use std::collections::VecDeque;

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use tq_core::{OrderRequest, Side, Symbol};
use tq_strategy::Signal;

/// A fixed-length window of `f64` samples with mean and sample standard deviation.
#[derive(Debug, Clone)]
pub struct Window {
    cap: usize,
    buf: VecDeque<f64>,
}

impl Window {
    pub fn new(cap: usize) -> Self {
        Self { cap, buf: VecDeque::with_capacity(cap + 1) }
    }

    pub fn push(&mut self, x: f64) {
        self.buf.push_back(x);
        if self.buf.len() > self.cap {
            self.buf.pop_front();
        }
    }

    pub fn is_full(&self) -> bool {
        self.buf.len() == self.cap
    }

    pub fn mean(&self) -> Option<f64> {
        (!self.buf.is_empty()).then(|| self.buf.iter().sum::<f64>() / self.buf.len() as f64)
    }

    /// Sample standard deviation (n - 1); `None` below two samples.
    pub fn stdev(&self) -> Option<f64> {
        let n = self.buf.len();
        let m = self.mean().filter(|_| n >= 2)?;
        let ss: f64 = self.buf.iter().map(|x| (x - m).powi(2)).sum();
        Some((ss / (n - 1) as f64).sqrt())
    }
}

pub(crate) fn to_f64(x: Decimal) -> f64 {
    x.to_f64().unwrap_or(f64::NAN)
}

pub(crate) fn from_f64(x: f64) -> Option<Decimal> {
    Decimal::from_f64(x)
}

/// A one-unit long entry at `entry` with `stop`, or nothing if the stop is not below the entry.
pub(crate) fn long_entry(symbol: &Symbol, entry: Decimal, stop: Decimal) -> Vec<Signal> {
    if stop <= Decimal::ZERO {
        return Vec::new();
    }
    OrderRequest::new(symbol.clone(), Side::Buy, Decimal::ONE, entry, stop).map(Signal::Enter).into_iter().collect()
}

/// Position state as seen through fills (the runner reports stops and flattens as fills too).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Holding {
    pub in_position: bool,
    pub bars_held: u32,
}

impl Holding {
    pub fn on_fill(&mut self, side: Side) {
        *self = Self { in_position: side == Side::Buy, bars_held: 0 };
    }

    /// Counts a bar that closed while in the position; returns the bars held so far.
    pub fn tick(&mut self) -> u32 {
        if self.in_position {
            self.bars_held += 1;
        }
        self.bars_held
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use rust_decimal::Decimal;
    use tq_core::{Bar, Fill, Side, Symbol};

    pub fn sym() -> Symbol {
        Symbol("T".into())
    }

    pub fn bar(ts: i64, o: Decimal, h: Decimal, l: Decimal, c: Decimal, v: Decimal) -> Bar {
        Bar::new(ts, o, h, l, c, v).unwrap()
    }

    pub fn fill(side: Side, price: Decimal) -> Fill {
        Fill { ts: 0, symbol: sym(), side, qty: Decimal::ONE, price, costs: Decimal::ZERO }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_keeps_last_n_and_computes_sample_stdev() {
        let mut w = Window::new(3);
        for x in [100.0, 1.0, 2.0, 3.0] {
            w.push(x);
        }
        assert!(w.is_full());
        assert_eq!(w.mean(), Some(2.0));
        assert_eq!(w.stdev(), Some(1.0));
    }

    #[test]
    fn long_entry_requires_stop_below_entry() {
        let s = Symbol("X".into());
        assert_eq!(long_entry(&s, Decimal::TEN, Decimal::TEN).len(), 0);
        assert_eq!(long_entry(&s, Decimal::TEN, Decimal::ONE).len(), 1);
    }
}

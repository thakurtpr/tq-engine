//! Core domain types. Money is always `Decimal`, never `f64`.

use rust_decimal::Decimal;
use thiserror::Error;

pub type Price = Decimal;
pub type Qty = Decimal;
/// Milliseconds since the Unix epoch. Kept as an integer so replays are exact.
pub type Timestamp = i64;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("invalid bar: {0}")]
    InvalidBar(&'static str),
    #[error("invalid order: {0}")]
    InvalidOrder(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub ts: Timestamp,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Qty,
}

impl Bar {
    pub fn new(
        ts: Timestamp,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: Qty,
    ) -> Result<Self, CoreError> {
        if low <= Decimal::ZERO {
            return Err(CoreError::InvalidBar("low must be positive"));
        }
        if high < low {
            return Err(CoreError::InvalidBar("high below low"));
        }
        let within = |p: Price| p >= low && p <= high;
        if !within(open) || !within(close) {
            return Err(CoreError::InvalidBar("open/close outside high-low range"));
        }
        if volume < Decimal::ZERO {
            return Err(CoreError::InvalidBar("negative volume"));
        }
        Ok(Self { ts, open, high, low, close, volume })
    }
}

/// A request to trade. Every order carries a stop so risk can be sized before it is sent.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderRequest {
    pub symbol: Symbol,
    pub side: Side,
    pub qty: Qty,
    pub entry: Price,
    pub stop: Price,
}

impl OrderRequest {
    pub fn new(symbol: Symbol, side: Side, qty: Qty, entry: Price, stop: Price) -> Result<Self, CoreError> {
        if qty <= Decimal::ZERO {
            return Err(CoreError::InvalidOrder("qty must be positive"));
        }
        if entry <= Decimal::ZERO || stop <= Decimal::ZERO {
            return Err(CoreError::InvalidOrder("prices must be positive"));
        }
        let stop_on_wrong_side = match side {
            Side::Buy => stop >= entry,
            Side::Sell => stop <= entry,
        };
        if stop_on_wrong_side {
            return Err(CoreError::InvalidOrder("stop on wrong side of entry"));
        }
        Ok(Self { symbol, side, qty, entry, stop })
    }

    /// Money lost if the stop is hit (before costs and slippage).
    pub fn risk_amount(&self) -> Decimal {
        (self.entry - self.stop).abs() * self.qty
    }

    pub fn notional(&self) -> Decimal {
        self.entry * self.qty
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub ts: Timestamp,
    pub symbol: Symbol,
    pub side: Side,
    pub qty: Qty,
    pub price: Price,
    pub costs: Decimal,
}

/// Injectable clock so backtests and live share code and replays are deterministic.
pub trait Clock {
    fn now(&self) -> Timestamp;
}

#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Timestamp);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sym() -> Symbol {
        Symbol("NSE:INFY".into())
    }

    #[test]
    fn bar_rejects_high_below_low() {
        let r = Bar::new(0, dec!(10), dec!(9), dec!(11), dec!(10), dec!(1));
        assert_eq!(r, Err(CoreError::InvalidBar("high below low")));
    }

    #[test]
    fn bar_rejects_close_outside_range() {
        assert!(Bar::new(0, dec!(10), dec!(12), dec!(9), dec!(13), dec!(1)).is_err());
    }

    #[test]
    fn bar_accepts_valid_ohlc() {
        assert!(Bar::new(0, dec!(10), dec!(12), dec!(9), dec!(11), dec!(100)).is_ok());
    }

    #[test]
    fn buy_order_requires_stop_below_entry() {
        let r = OrderRequest::new(sym(), Side::Buy, dec!(10), dec!(100), dec!(101));
        assert_eq!(r, Err(CoreError::InvalidOrder("stop on wrong side of entry")));
    }

    #[test]
    fn sell_order_requires_stop_above_entry() {
        assert!(OrderRequest::new(sym(), Side::Sell, dec!(10), dec!(100), dec!(99)).is_err());
    }

    #[test]
    fn order_rejects_zero_qty() {
        assert!(OrderRequest::new(sym(), Side::Buy, dec!(0), dec!(100), dec!(95)).is_err());
    }

    #[test]
    fn risk_amount_is_stop_distance_times_qty() {
        let o = OrderRequest::new(sym(), Side::Buy, dec!(10), dec!(100), dec!(95)).unwrap();
        assert_eq!(o.risk_amount(), dec!(50));
        assert_eq!(o.notional(), dec!(1000));
    }

    #[test]
    fn fixed_clock_returns_set_time() {
        assert_eq!(FixedClock(42).now(), 42);
    }
}

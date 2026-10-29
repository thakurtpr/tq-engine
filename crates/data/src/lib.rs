//! Market data sources. Kite historical, bhavcopy and Parquet loaders implement `BarSource` later.

use thiserror::Error;
use tq_core::{Bar, Symbol};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DataError {
    #[error("no data for {0}")]
    NotFound(String),
    #[error("bars out of time order at index {0}")]
    Unordered(usize),
}

pub trait BarSource {
    /// Daily bars for `symbol`, oldest first.
    fn bars(&self, symbol: &Symbol) -> Result<Vec<Bar>, DataError>;
}

/// In-memory source for tests and fixtures. Rejects bars that are not strictly time-ordered,
/// since unordered data silently creates look-ahead bias in a backtest.
#[derive(Debug)]
pub struct InMemory {
    symbol: Symbol,
    bars: Vec<Bar>,
}

impl InMemory {
    pub fn new(symbol: Symbol, bars: Vec<Bar>) -> Result<Self, DataError> {
        if let Some(i) = bars.windows(2).position(|w| w[1].ts <= w[0].ts) {
            return Err(DataError::Unordered(i + 1));
        }
        Ok(Self { symbol, bars })
    }
}

impl BarSource for InMemory {
    fn bars(&self, symbol: &Symbol) -> Result<Vec<Bar>, DataError> {
        if *symbol != self.symbol {
            return Err(DataError::NotFound(symbol.0.clone()));
        }
        Ok(self.bars.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn bar(ts: i64) -> Bar {
        Bar::new(ts, dec!(10), dec!(11), dec!(9), dec!(10), dec!(1)).unwrap()
    }

    #[test]
    fn rejects_unordered_bars() {
        let r = InMemory::new(Symbol("X".into()), vec![bar(2), bar(1)]);
        assert_eq!(r.unwrap_err(), DataError::Unordered(1));
    }

    #[test]
    fn unknown_symbol_is_not_found() {
        let src = InMemory::new(Symbol("X".into()), vec![bar(1)]).unwrap();
        assert!(src.bars(&Symbol("Y".into())).is_err());
    }
}

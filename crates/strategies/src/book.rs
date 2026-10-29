//! Per-symbol state and the helpers every long-only strategy here shares.

use rust_decimal::Decimal;
use std::collections::HashMap;
use tq_core::{OrderRequest, Price, Qty, Side, Symbol};
use tq_strategy::Signal;

/// Quantity put on every entry. It is a placeholder: the backtest engine (and the live risk
/// gate) resize each order from `entry - stop` and the account's per-trade risk budget, so a
/// strategy only decides *whether* to trade and *where* the stop goes.
pub const PLACEHOLDER_QTY: Qty = Decimal::ONE;

/// Whether the strategy currently holds a long in a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Position {
    #[default]
    Flat,
    /// `bars_held` counts the bars seen by `on_bar` since the entry fill.
    Long { bars_held: usize },
}

impl Position {
    /// New position after a fill: a buy opens a long, a sell closes it.
    pub(crate) fn after_fill(side: Side) -> Self {
        match side {
            Side::Buy => Position::Long { bars_held: 0 },
            Side::Sell => Position::Flat,
        }
    }
}

/// Indicator state per symbol, each cloned from a validated template on first sight.
#[derive(Debug, Clone)]
pub(crate) struct Book<S: Clone> {
    template: S,
    states: HashMap<Symbol, S>,
}

impl<S: Clone> Book<S> {
    pub(crate) fn new(template: S) -> Self {
        Self { template, states: HashMap::new() }
    }

    pub(crate) fn state(&mut self, symbol: &Symbol) -> &mut S {
        let template = &self.template;
        self.states.entry(symbol.clone()).or_insert_with(|| template.clone())
    }

    /// State for a symbol already seen by `on_bar`; `None` for a stray fill.
    pub(crate) fn existing(&mut self, symbol: &Symbol) -> Option<&mut S> {
        self.states.get_mut(symbol)
    }
}

/// A long entry at `close` with the stop `atr_mult` ATRs below. `None` when the stop would
/// not be a valid positive price below entry (e.g. zero ATR), so no order is proposed.
pub(crate) fn long_entry(symbol: &Symbol, close: Price, atr: Decimal, atr_mult: Decimal) -> Option<Signal> {
    let stop = close - atr * atr_mult;
    OrderRequest::new(symbol.clone(), Side::Buy, PLACEHOLDER_QTY, close, stop).ok().map(Signal::Enter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sym() -> Symbol {
        Symbol("NSE:TEST".into())
    }

    #[test]
    fn long_entry_puts_stop_atr_multiple_below_close() {
        let Some(Signal::Enter(o)) = long_entry(&sym(), dec!(100), dec!(3), dec!(2)) else { panic!("no entry") };
        assert_eq!((o.side, o.qty, o.entry, o.stop), (Side::Buy, PLACEHOLDER_QTY, dec!(100), dec!(94)));
    }

    #[test]
    fn long_entry_skips_zero_atr_and_non_positive_stop() {
        assert!(long_entry(&sym(), dec!(100), dec!(0), dec!(2)).is_none());
        assert!(long_entry(&sym(), dec!(10), dec!(5), dec!(2)).is_none());
    }

    #[test]
    fn fills_drive_position() {
        assert_eq!(Position::after_fill(Side::Buy), Position::Long { bars_held: 0 });
        assert_eq!(Position::after_fill(Side::Sell), Position::Flat);
    }
}

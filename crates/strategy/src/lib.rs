//! Strategy interface, identical in backtest, paper and live.
//! Strategies only propose orders; the risk gate decides whether they are sent.

use tq_core::{Bar, Fill, OrderRequest, Symbol};

/// What a strategy wants after seeing a bar.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    /// Open a position. Goes through the risk gate.
    Enter(OrderRequest),
    /// Close the whole open position in this symbol. Exits are never blocked by risk.
    Exit(Symbol),
}

pub trait Strategy {
    fn name(&self) -> &str;

    /// Called on each closed bar. Returns signals for the engine.
    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal>;

    /// Called when one of this strategy's orders fills.
    fn on_fill(&mut self, _fill: &Fill) {}
}

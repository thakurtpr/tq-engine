//! Strategy interface, identical in backtest, paper and live.
//! Strategies only propose orders; the risk gate decides whether they are sent.

use tq_core::{Bar, Fill, OrderRequest, Symbol};

pub trait Strategy {
    fn name(&self) -> &str;

    /// Called on each closed bar. Returns order proposals for the risk gate.
    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<OrderRequest>;

    /// Called when one of this strategy's orders fills.
    fn on_fill(&mut self, _fill: &Fill) {}
}

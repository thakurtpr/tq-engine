//! Cross-sectional rebalancing strategies: on each rebalance date, say what the whole portfolio
//! should hold, as weights of equity. The engine trades to those weights at the next open
//! (sells first, then buys, every buy through the risk gate). Unlike [`crate::Strategy`], a
//! rebalancer never sizes orders or sets stops.

use std::collections::HashMap;

use rust_decimal::Decimal;
use tq_core::{Price, Symbol, Timestamp};

/// Daily closes per symbol, oldest first, up to and including the rebalance date's close.
/// Only symbols with at least one bar so far are present.
pub type History = HashMap<Symbol, Vec<Price>>;

/// A symbol and its target weight as a fraction of equity (0.1 = 10%).
pub type Target = (Symbol, Decimal);

pub trait Rebalancer {
    fn name(&self) -> &str;

    /// Called after the close of every rebalance date, including warm-up dates before the
    /// trading window (so state such as a monthly price list stays complete). `members` are the
    /// symbols eligible on `date` (point-in-time index members with a bar that day), in ticker
    /// order. Returns the full target book: anything held but not listed is sold. Weights must
    /// be positive and sum to at most 1; the remainder is cash.
    fn targets(&mut self, date: Timestamp, members: &[Symbol], history: &History) -> Vec<Target>;
}

/// True when every weight is positive and they sum to at most 1 (the engine rejects others).
pub fn weight_sum_ok(targets: &[Target]) -> bool {
    targets.iter().all(|t| t.1 > Decimal::ZERO) && targets.iter().map(|t| t.1).sum::<Decimal>() <= Decimal::ONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn t(w: Decimal) -> Target {
        (Symbol("X".into()), w)
    }

    #[test]
    fn weights_must_be_positive_and_not_leveraged() {
        assert!(weight_sum_ok(&[t(dec!(0.5)), t(dec!(0.5))]));
        assert!(weight_sum_ok(&[]), "all cash");
        assert!(!weight_sum_ok(&[t(dec!(0.6)), t(dec!(0.5))]));
        assert!(!weight_sum_ok(&[t(dec!(0))]));
        assert!(!weight_sum_ok(&[t(dec!(-0.1))]));
    }
}

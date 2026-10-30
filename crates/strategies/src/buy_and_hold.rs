//! Buy and hold one fund: the honest benchmark. Targets 100% in the fund on every rebalance date;
//! the engine's dust band means only the first purchase (and leftover cash above the band) trades.

use rust_decimal::Decimal;
use tq_core::{Symbol, Timestamp};
use tq_strategy::{History, Rebalancer, Target};

#[derive(Debug)]
pub struct BuyAndHold {
    symbol: Symbol,
}

impl BuyAndHold {
    pub fn new(symbol: Symbol) -> Self {
        Self { symbol }
    }
}

impl Rebalancer for BuyAndHold {
    fn name(&self) -> &str {
        "buy-and-hold"
    }

    fn targets(&mut self, _date: Timestamp, members: &[Symbol], _history: &History) -> Vec<Target> {
        if members.contains(&self.symbol) {
            vec![(self.symbol.clone(), Decimal::ONE)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_fully_in_the_fund_when_offered() {
        let s = Symbol("NSE:NIFTYBEES".into());
        let mut b = BuyAndHold::new(s.clone());
        assert_eq!(b.targets(0, std::slice::from_ref(&s), &History::new()), vec![(s, Decimal::ONE)]);
        assert!(b.targets(0, &[], &History::new()).is_empty());
    }
}

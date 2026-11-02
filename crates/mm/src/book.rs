//! Price-level order book. Quantities are absolute; 0 removes the level (removing an absent level is normal).

use std::collections::BTreeMap;

use crate::record::Levels;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Book {
    pub bids: BTreeMap<i64, i64>,
    pub asks: BTreeMap<i64, i64>,
}

fn apply_side(side: &mut BTreeMap<i64, i64>, levels: &Levels) {
    for &(px, qty) in levels {
        if qty == 0 {
            side.remove(&px);
        } else {
            side.insert(px, qty);
        }
    }
}

impl Book {
    pub fn from_snapshot(bids: &Levels, asks: &Levels) -> Self {
        let mut b = Book::default();
        b.apply(bids, asks);
        b
    }

    pub fn apply(&mut self, bids: &Levels, asks: &Levels) {
        apply_side(&mut self.bids, bids);
        apply_side(&mut self.asks, asks);
    }

    pub fn best_bid(&self) -> Option<(i64, i64)> {
        self.bids.iter().next_back().map(|(p, q)| (*p, *q))
    }

    pub fn best_ask(&self) -> Option<(i64, i64)> {
        self.asks.iter().next().map(|(p, q)| (*p, *q))
    }

    /// Best `n` levels per side, best first.
    pub fn top(&self, n: usize) -> (Levels, Levels) {
        let bids = self.bids.iter().rev().take(n).map(|(p, q)| (*p, *q)).collect();
        let asks = self.asks.iter().take(n).map(|(p, q)| (*p, *q)).collect();
        (bids, asks)
    }

    pub fn crossed(&self) -> bool {
        matches!((self.best_bid(), self.best_ask()), (Some((b, _)), Some((a, _))) if b >= a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_absolute_quantities_and_zero_removes() {
        let mut b = Book::from_snapshot(&vec![(100, 5), (99, 1)], &vec![(101, 2)]);
        b.apply(&vec![(100, 0), (98, 3), (97, 0)], &vec![(101, 7)]);
        assert_eq!(b.best_bid(), Some((99, 1)));
        assert_eq!(b.best_ask(), Some((101, 7)));
        assert_eq!(b.top(5).0, vec![(99, 1), (98, 3)]);
        assert!(!b.crossed());
    }
}

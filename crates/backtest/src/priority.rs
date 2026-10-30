//! Which entry signals get the free slots and cash when there are more signals than room.
//!
//! Candidates from one day's close are ordered by this rule, then offered to the risk gate in
//! that order; the gate's position cap and the cash check decide how many fill. Fills at the
//! next open happen in the same order, so earlier candidates also get the cash first.
//!
//! - `Alphabetical` (default, the deterministic baseline): by ticker. Arbitrary but
//!   reproducible, and unrelated to the signal, so it neither helps nor hurts the strategy.
//! - `Random { seed }`: a seeded shuffle per day, for a sensitivity analysis over seeds. Uses
//!   SplitMix64 (Steele, Lea & Flood 2014) so runs are reproducible without a `rand` dependency.

use tq_core::{OrderRequest, Symbol, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Priority {
    #[default]
    Alphabetical,
    Random { seed: u64 },
}

impl Priority {
    /// Orders one day's entry candidates in place.
    pub fn order(&self, ts: Timestamp, candidates: &mut [OrderRequest]) {
        candidates.sort_by(|a, b| a.symbol.0.cmp(&b.symbol.0));
        self.shuffle(ts, candidates);
    }

    /// Orders one day's rebalance buys in place (same rule as [`Self::order`]).
    pub fn order_symbols(&self, ts: Timestamp, symbols: &mut [Symbol]) {
        symbols.sort_by(|a, b| a.0.cmp(&b.0));
        self.shuffle(ts, symbols);
    }

    /// Shuffles a ticker-sorted list so the result depends only on (seed, day, symbols).
    fn shuffle<T>(&self, ts: Timestamp, items: &mut [T]) {
        if let Priority::Random { seed } = *self {
            let mut rng = SplitMix64::new(seed ^ (ts as u64).rotate_left(17));
            for i in (1..items.len()).rev() {
                let j = rng.below(i as u64 + 1) as usize;
                items.swap(i, j);
            }
        }
    }
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n` (modulo bias is below 1e-17 for the small `n` used here).
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::{Side, Symbol};

    fn order(sym: &str) -> OrderRequest {
        OrderRequest::new(Symbol(sym.into()), Side::Buy, dec!(1), dec!(100), dec!(90)).unwrap()
    }

    fn names(c: &[OrderRequest]) -> Vec<String> {
        c.iter().map(|o| o.symbol.0.clone()).collect()
    }

    fn many() -> Vec<OrderRequest> {
        ["E", "B", "D", "A", "C", "F", "H", "G"].into_iter().map(order).collect()
    }

    #[test]
    fn alphabetical_sorts_by_ticker() {
        let mut c = vec![order("TCS"), order("INFY"), order("SBIN")];
        Priority::Alphabetical.order(0, &mut c);
        assert_eq!(names(&c), ["INFY", "SBIN", "TCS"]);
    }

    #[test]
    fn random_is_reproducible_for_a_seed_and_day_regardless_of_input_order() {
        let (mut a, mut b) = (many(), many());
        b.reverse();
        Priority::Random { seed: 7 }.order(42, &mut a);
        Priority::Random { seed: 7 }.order(42, &mut b);
        assert_eq!(names(&a), names(&b));
    }

    #[test]
    fn symbols_follow_the_same_order_as_orders() {
        let mut orders = many();
        let mut syms: Vec<Symbol> = many().into_iter().map(|o| o.symbol).collect();
        Priority::Random { seed: 9 }.order(42, &mut orders);
        Priority::Random { seed: 9 }.order_symbols(42, &mut syms);
        assert_eq!(names(&orders), syms.iter().map(|s| s.0.clone()).collect::<Vec<_>>());
    }

    #[test]
    fn random_differs_across_seeds_and_days_and_keeps_every_candidate() {
        let shuffled = |seed, ts| {
            let mut c = many();
            Priority::Random { seed }.order(ts, &mut c);
            names(&c)
        };
        assert_ne!(shuffled(1, 42), shuffled(2, 42));
        assert_ne!(shuffled(1, 42), shuffled(1, 43));
        let mut all = shuffled(3, 5);
        all.sort();
        assert_eq!(all, ["A", "B", "C", "D", "E", "F", "G", "H"]);
    }
}

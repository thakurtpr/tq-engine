use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Highest value of the last `period` values (Donchian upper band), including the latest.
#[derive(Debug, Clone)]
pub struct RollingHigh(Extreme);

/// Lowest value of the last `period` values (Donchian lower band), including the latest.
#[derive(Debug, Clone)]
pub struct RollingLow(Extreme);

impl RollingHigh {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        Extreme::new(period, |kept, new| kept <= new).map(Self)
    }

    /// Feeds one value; returns the window maximum once the window is full.
    pub fn update(&mut self, value: Decimal) -> Option<Decimal> {
        self.0.update(value)
    }
}

impl RollingLow {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        Extreme::new(period, |kept, new| kept >= new).map(Self)
    }

    /// Feeds one value; returns the window minimum once the window is full.
    pub fn update(&mut self, value: Decimal) -> Option<Decimal> {
        self.0.update(value)
    }
}

/// Monotonic deque: the front is always the window extreme. Each value is pushed and popped
/// at most once, so updates are amortised O(1).
#[derive(Debug, Clone)]
struct Extreme {
    period: usize,
    count: usize,
    deque: VecDeque<(usize, Decimal)>,
    /// True when an already-kept value can never be the extreme again once `new` arrives.
    dominated: fn(Decimal, Decimal) -> bool,
}

impl Extreme {
    fn new(period: usize, dominated: fn(Decimal, Decimal) -> bool) -> Option<Self> {
        (period > 0).then(|| Self { period, count: 0, deque: VecDeque::with_capacity(period), dominated })
    }

    fn update(&mut self, value: Decimal) -> Option<Decimal> {
        while self.deque.back().is_some_and(|&(_, kept)| (self.dominated)(kept, value)) {
            self.deque.pop_back();
        }
        self.deque.push_back((self.count, value));
        self.count += 1;
        // Indices still in the window are those >= count - period.
        while self.deque.front().is_some_and(|&(i, _)| i + self.period < self.count) {
            self.deque.pop_front();
        }
        let warmed = self.count >= self.period;
        self.deque.front().filter(|_| warmed).map(|&(_, v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn feed<F: FnMut(Decimal) -> Option<Decimal>>(mut f: F, xs: &[Decimal]) -> Vec<Option<Decimal>> {
        xs.iter().map(|x| f(*x)).collect()
    }

    #[test]
    fn zero_period_is_rejected() {
        assert!(RollingHigh::new(0).is_none());
        assert!(RollingLow::new(0).is_none());
    }

    #[test]
    fn rolling_high_drops_values_that_leave_the_window() {
        let mut h = RollingHigh::new(3).unwrap();
        let xs = [dec!(5), dec!(3), dec!(4), dec!(2), dec!(6), dec!(1), dec!(1), dec!(1)];
        let want = [None, None, Some(dec!(5)), Some(dec!(4)), Some(dec!(6)), Some(dec!(6)), Some(dec!(6)), Some(dec!(1))];
        assert_eq!(feed(|x| h.update(x), &xs), want);
    }

    #[test]
    fn rolling_low_drops_values_that_leave_the_window() {
        let mut l = RollingLow::new(3).unwrap();
        let xs = [dec!(1), dec!(3), dec!(2), dec!(4), dec!(0.5), dec!(9), dec!(9), dec!(9)];
        let want = [None, None, Some(dec!(1)), Some(dec!(2)), Some(dec!(0.5)), Some(dec!(0.5)), Some(dec!(0.5)), Some(dec!(9))];
        assert_eq!(feed(|x| l.update(x), &xs), want);
    }

    #[test]
    fn equal_values_are_handled() {
        let mut h = RollingHigh::new(2).unwrap();
        let xs = [dec!(7), dec!(7), dec!(7), dec!(3)];
        assert_eq!(feed(|x| h.update(x), &xs), [None, Some(dec!(7)), Some(dec!(7)), Some(dec!(7))]);
    }
}

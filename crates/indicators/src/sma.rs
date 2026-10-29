use rust_decimal::Decimal;
use std::collections::VecDeque;

/// Simple moving average over the last `period` values.
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    window: VecDeque<Decimal>,
    sum: Decimal,
}

impl Sma {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        (period > 0).then(|| Self { period, window: VecDeque::with_capacity(period), sum: Decimal::ZERO })
    }

    /// Feeds one value; returns the average once the window is full.
    pub fn update(&mut self, value: Decimal) -> Option<Decimal> {
        self.window.push_back(value);
        self.sum += value;
        if self.window.len() > self.period {
            self.sum -= self.window.pop_front().unwrap_or_default();
        }
        (self.window.len() == self.period).then(|| self.sum / Decimal::from(self.period))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn zero_period_is_rejected() {
        assert!(Sma::new(0).is_none());
    }

    #[test]
    fn returns_none_until_window_full() {
        let mut s = Sma::new(3).unwrap();
        assert_eq!(s.update(dec!(1)), None);
        assert_eq!(s.update(dec!(2)), None);
        assert_eq!(s.update(dec!(3)), Some(dec!(2)));
    }

    #[test]
    fn rolls_window_forward() {
        let mut s = Sma::new(2).unwrap();
        s.update(dec!(10));
        s.update(dec!(20));
        assert_eq!(s.update(dec!(40)), Some(dec!(30)));
    }
}

use rust_decimal::Decimal;

/// Exponential moving average with `alpha = 2 / (period + 1)`, seeded with the simple
/// average of the first `period` values (so the first output appears on value `period`).
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    alpha: Decimal,
    seen: usize,
    seed_sum: Decimal,
    value: Option<Decimal>,
}

impl Ema {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        let alpha = Decimal::TWO / Decimal::from(period.checked_add(1)?);
        (period > 0).then_some(Self { period, alpha, seen: 0, seed_sum: Decimal::ZERO, value: None })
    }

    /// Feeds one value; returns the average once `period` values have been seen.
    pub fn update(&mut self, value: Decimal) -> Option<Decimal> {
        let next = match self.value {
            Some(prev) => prev + self.alpha * (value - prev),
            None => {
                self.seen += 1;
                self.seed_sum += value;
                if self.seen < self.period {
                    return None;
                }
                self.seed_sum / Decimal::from(self.period)
            }
        };
        self.value = Some(next);
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn zero_period_is_rejected() {
        assert!(Ema::new(0).is_none());
    }

    #[test]
    fn seeds_with_sma_then_smooths() {
        // period 3 => alpha 0.5. Seed = (1+2+3)/3 = 2; then 2 + 0.5*(4-2) = 3; 3 + 0.5*(7-3) = 5.
        let mut e = Ema::new(3).unwrap();
        assert_eq!(e.update(dec!(1)), None);
        assert_eq!(e.update(dec!(2)), None);
        assert_eq!(e.update(dec!(3)), Some(dec!(2)));
        assert_eq!(e.update(dec!(4)), Some(dec!(3)));
        assert_eq!(e.update(dec!(7)), Some(dec!(5)));
    }

    #[test]
    fn period_one_tracks_input() {
        let mut e = Ema::new(1).unwrap();
        assert_eq!(e.update(dec!(5)), Some(dec!(5)));
        assert_eq!(e.update(dec!(9)), Some(dec!(9)));
    }
}

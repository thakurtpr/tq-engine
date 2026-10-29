use rust_decimal::Decimal;

/// Relative Strength Index with Wilder smoothing.
///
/// Average gain/loss are seeded with the simple mean of the first `period` changes, then
/// smoothed as `avg = (prev_avg * (period - 1) + x) / period`. Needs `period + 1` values.
/// `RSI = 100 * avg_gain / (avg_gain + avg_loss)`, which equals `100 - 100 / (1 + RS)` and is
/// defined as 50 when price has not moved at all.
#[derive(Debug, Clone)]
pub struct Rsi {
    period: usize,
    prev: Option<Decimal>,
    changes_seen: usize,
    avg_gain: Decimal,
    avg_loss: Decimal,
}

impl Rsi {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        (period > 0).then_some(Self {
            period,
            prev: None,
            changes_seen: 0,
            avg_gain: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
        })
    }

    /// Feeds one value; returns the RSI (0..=100) once `period` changes have been seen.
    pub fn update(&mut self, value: Decimal) -> Option<Decimal> {
        let prev = self.prev.replace(value)?;
        let change = value - prev;
        let gain = change.max(Decimal::ZERO);
        let loss = (-change).max(Decimal::ZERO);
        let n = Decimal::from(self.period);
        if self.changes_seen < self.period {
            // Seeding: sum the first `period` changes, then divide once for a simple mean.
            self.changes_seen += 1;
            self.avg_gain += gain;
            self.avg_loss += loss;
            if self.changes_seen < self.period {
                return None;
            }
            self.avg_gain /= n;
            self.avg_loss /= n;
        } else {
            self.avg_gain = (self.avg_gain * (n - Decimal::ONE) + gain) / n;
            self.avg_loss = (self.avg_loss * (n - Decimal::ONE) + loss) / n;
        }
        Some(self.value())
    }

    fn value(&self) -> Decimal {
        let total = self.avg_gain + self.avg_loss;
        if total.is_zero() {
            return Decimal::ONE_HUNDRED / Decimal::TWO;
        }
        Decimal::ONE_HUNDRED * self.avg_gain / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn zero_period_is_rejected() {
        assert!(Rsi::new(0).is_none());
    }

    #[test]
    fn seeds_then_wilder_smooths() {
        let mut r = Rsi::new(2).unwrap();
        assert_eq!(r.update(dec!(10)), None);
        assert_eq!(r.update(dec!(13)), None); // +3
        // -1 => avg gain 1.5, avg loss 0.5 => 100 * 1.5 / 2 = 75.
        assert_eq!(r.update(dec!(12)), Some(dec!(75)));
        // -1 => gain (1.5 + 0)/2 = 0.75, loss (0.5 + 1)/2 = 0.75 => 50.
        assert_eq!(r.update(dec!(11)), Some(dec!(50)));
    }

    #[test]
    fn only_gains_is_100_and_flat_is_50() {
        let mut up = Rsi::new(2).unwrap();
        up.update(dec!(1));
        up.update(dec!(2));
        assert_eq!(up.update(dec!(3)), Some(dec!(100)));

        let mut flat = Rsi::new(2).unwrap();
        flat.update(dec!(5));
        flat.update(dec!(5));
        assert_eq!(flat.update(dec!(5)), Some(dec!(50)));
    }

    #[test]
    fn only_losses_is_zero() {
        let mut r = Rsi::new(2).unwrap();
        r.update(dec!(3));
        r.update(dec!(2));
        assert_eq!(r.update(dec!(1)), Some(dec!(0)));
    }
}

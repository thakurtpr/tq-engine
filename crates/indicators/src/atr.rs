use rust_decimal::Decimal;

/// Average True Range with Wilder smoothing.
///
/// True range is `max(high - low, |high - prev_close|, |low - prev_close|)`; the first bar has
/// no previous close so its true range is `high - low`. The first ATR is the simple mean of the
/// first `period` true ranges, then `atr = (prev_atr * (period - 1) + tr) / period`.
#[derive(Debug, Clone)]
pub struct Atr {
    period: usize,
    prev_close: Option<Decimal>,
    seen: usize,
    seed_sum: Decimal,
    value: Option<Decimal>,
}

impl Atr {
    /// Returns `None` for a zero period.
    pub fn new(period: usize) -> Option<Self> {
        (period > 0).then_some(Self { period, prev_close: None, seen: 0, seed_sum: Decimal::ZERO, value: None })
    }

    /// Feeds one bar; returns the ATR once `period` bars have been seen.
    pub fn update(&mut self, high: Decimal, low: Decimal, close: Decimal) -> Option<Decimal> {
        let tr = true_range(high, low, self.prev_close);
        self.prev_close = Some(close);
        let n = Decimal::from(self.period);
        let next = match self.value {
            Some(prev) => (prev * (n - Decimal::ONE) + tr) / n,
            None => {
                self.seen += 1;
                self.seed_sum += tr;
                if self.seen < self.period {
                    return None;
                }
                self.seed_sum / n
            }
        };
        self.value = Some(next);
        self.value
    }
}

fn true_range(high: Decimal, low: Decimal, prev_close: Option<Decimal>) -> Decimal {
    let range = high - low;
    match prev_close {
        Some(pc) => range.max((high - pc).abs()).max((low - pc).abs()),
        None => range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn zero_period_is_rejected() {
        assert!(Atr::new(0).is_none());
    }

    #[test]
    fn seeds_with_mean_true_range_then_wilder_smooths() {
        let mut a = Atr::new(3).unwrap();
        assert_eq!(a.update(dec!(10), dec!(8), dec!(9)), None); // TR 2
        assert_eq!(a.update(dec!(11), dec!(9), dec!(10)), None); // TR 2
        assert_eq!(a.update(dec!(12), dec!(10), dec!(11)), Some(dec!(2))); // TR 2, seed 2
        // Gap up: TR = max(2, |16-11|, |14-11|) = 5 => (2*2 + 5)/3 = 3.
        assert_eq!(a.update(dec!(16), dec!(14), dec!(15)), Some(dec!(3)));
        // Wide bar: TR = max(6, |17-15|, |11-15|) = 6 => (3*2 + 6)/3 = 4.
        assert_eq!(a.update(dec!(17), dec!(11), dec!(12)), Some(dec!(4)));
    }

    #[test]
    fn gap_down_uses_previous_close() {
        let mut a = Atr::new(1).unwrap();
        assert_eq!(a.update(dec!(101), dec!(99), dec!(100)), Some(dec!(2)));
        // TR = max(1, |91-100|, |90-100|) = 10; period 1 => ATR = TR.
        assert_eq!(a.update(dec!(91), dec!(90), dec!(90)), Some(dec!(10)));
    }
}

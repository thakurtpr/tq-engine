//! Lot units. `tq_core::OrderRequest` (and so the risk gate) only accepts whole quantities,
//! which fits shares but not 0.0123 BTC. The runner therefore trades in LOTS: one unit is `lot`
//! of the base asset (e.g. 0.00001 BTC, Binance's BTCUSDT step size). A bar is converted by
//! multiplying prices by `lot` and dividing volume by `lot`, so money (price * qty) is unchanged
//! and exact in `Decimal`. All three strategies are scale-invariant (returns, VWAP distance in
//! stdevs, ATR stops, volume ratios), so they behave identically on scaled bars.

use rust_decimal::Decimal;
use tq_core::{Bar, CoreError, Price, Qty};

pub fn to_units(bar: &Bar, lot: Decimal) -> Result<Bar, CoreError> {
    if lot <= Decimal::ZERO {
        return Err(CoreError::InvalidBar("lot must be positive"));
    }
    Bar::new(bar.ts, bar.open * lot, bar.high * lot, bar.low * lot, bar.close * lot, bar.volume / lot)
}

/// Price per base asset of a per-unit price.
pub fn base_price(unit_price: Price, lot: Decimal) -> Price {
    if lot.is_zero() {
        return unit_price;
    }
    (unit_price / lot).normalize()
}

/// Base-asset quantity of `units` lots.
pub fn base_qty(units: Qty, lot: Decimal) -> Qty {
    (units * lot).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn scaling_keeps_notional_and_round_trips_prices() {
        let b = Bar::new(0, dec!(16500.12), dec!(16510), dec!(16490.5), dec!(16505), dec!(40.57)).unwrap();
        let u = to_units(&b, dec!(0.00001)).unwrap();
        assert_eq!(u.open, dec!(0.1650012));
        assert_eq!(u.volume, dec!(4057000));
        assert_eq!(base_price(u.open, dec!(0.00001)), dec!(16500.12));
        // 1000 units of 0.00001 BTC = 0.01 BTC; notional identical either way.
        assert_eq!(u.open * dec!(1000), b.open * base_qty(dec!(1000), dec!(0.00001)));
    }

    #[test]
    fn rejects_non_positive_lot() {
        let b = Bar::new(0, dec!(1), dec!(1), dec!(1), dec!(1), dec!(1)).unwrap();
        assert!(to_units(&b, dec!(0)).is_err());
    }
}

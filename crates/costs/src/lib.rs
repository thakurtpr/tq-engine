//! Indian equity transaction costs. Rates are data, not code: a fee change is a config edit.
//!
//! Defaults are Zerodha NSE rates as known for 2024-25 and are UNVERIFIED until
//! docs/costs-india.md confirms them with sources and dates.

use rust_decimal::Decimal;
use tq_core::Side;

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;

/// All `*_pct` fields are percent of turnover (price * qty).
#[derive(Debug, Clone, PartialEq)]
pub struct CostSchedule {
    pub brokerage_pct: Decimal,
    /// Flat cap per order (e.g. Rs 20 intraday). `None` means uncapped.
    pub brokerage_cap: Option<Decimal>,
    pub stt_buy_pct: Decimal,
    pub stt_sell_pct: Decimal,
    pub exchange_pct: Decimal,
    pub sebi_pct: Decimal,
    pub stamp_buy_pct: Decimal,
    pub gst_pct: Decimal,
    /// Depository charge per sell order (delivery only), before GST.
    pub dp_per_sell: Decimal,
}

impl CostSchedule {
    pub fn zerodha_nse_delivery() -> Self {
        Self {
            brokerage_pct: Decimal::ZERO,
            brokerage_cap: None,
            stt_buy_pct: Decimal::new(1, 1),    // 0.1
            stt_sell_pct: Decimal::new(1, 1),   // 0.1
            exchange_pct: Decimal::new(297, 5), // 0.00297
            sebi_pct: Decimal::new(1, 4),       // 0.0001 (Rs 10 / crore)
            stamp_buy_pct: Decimal::new(15, 3), // 0.015
            gst_pct: Decimal::from(18),
            dp_per_sell: Decimal::new(135, 1), // 13.5
        }
    }

    pub fn zerodha_nse_intraday() -> Self {
        Self {
            brokerage_pct: Decimal::new(3, 2), // 0.03
            brokerage_cap: Some(Decimal::from(20)),
            stt_buy_pct: Decimal::ZERO,
            stt_sell_pct: Decimal::new(25, 3),  // 0.025
            stamp_buy_pct: Decimal::new(3, 3), // 0.003
            dp_per_sell: Decimal::ZERO,
            ..Self::zerodha_nse_delivery()
        }
    }

    pub fn compute(&self, side: Side, turnover: Decimal) -> CostBreakdown {
        let pct = |rate: Decimal| turnover * rate / HUNDRED;
        let raw_brokerage = pct(self.brokerage_pct);
        let brokerage = self.brokerage_cap.map_or(raw_brokerage, |cap| raw_brokerage.min(cap));
        let (stt, stamp, dp) = match side {
            Side::Buy => (pct(self.stt_buy_pct), pct(self.stamp_buy_pct), Decimal::ZERO),
            Side::Sell => (pct(self.stt_sell_pct), Decimal::ZERO, self.dp_per_sell),
        };
        let exchange = pct(self.exchange_pct);
        let sebi = pct(self.sebi_pct);
        // GST applies to services (brokerage, exchange, SEBI, DP), not to taxes (STT, stamp).
        let gst = (brokerage + exchange + sebi + dp) * self.gst_pct / HUNDRED;
        CostBreakdown { brokerage, stt, exchange, sebi, stamp, dp, gst }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostBreakdown {
    pub brokerage: Decimal,
    pub stt: Decimal,
    pub exchange: Decimal,
    pub sebi: Decimal,
    pub stamp: Decimal,
    pub dp: Decimal,
    pub gst: Decimal,
}

impl CostBreakdown {
    /// Total in rupees, rounded to paise.
    pub fn total(&self) -> Decimal {
        (self.brokerage + self.stt + self.exchange + self.sebi + self.stamp + self.dp + self.gst).round_dp(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn delivery_buy_pays_stt_and_stamp_no_brokerage() {
        // 100 @ 1000: stt 100, exch 2.97, sebi 0.10, stamp 15, gst 0.18 * 3.07 = 0.5526
        let c = CostSchedule::zerodha_nse_delivery().compute(Side::Buy, dec!(100000));
        assert_eq!(c.brokerage, dec!(0));
        assert_eq!(c.stamp, dec!(15));
        assert_eq!(c.total(), dec!(118.62));
    }

    #[test]
    fn delivery_sell_pays_dp_and_no_stamp() {
        // 100 @ 1010: stt 101, exch 2.9997, sebi 0.101, dp 13.5, gst 0.18 * 16.6007 = 2.988126
        let c = CostSchedule::zerodha_nse_delivery().compute(Side::Sell, dec!(101000));
        assert_eq!(c.stamp, dec!(0));
        assert_eq!(c.dp, dec!(13.5));
        assert_eq!(c.total(), dec!(120.59));
    }

    #[test]
    fn intraday_brokerage_is_capped_at_twenty() {
        // 0.03% of 100000 = 30, capped to 20
        let c = CostSchedule::zerodha_nse_intraday().compute(Side::Buy, dec!(100000));
        assert_eq!(c.brokerage, dec!(20));
        assert_eq!(c.total(), dec!(30.22));
    }

    #[test]
    fn intraday_small_order_pays_percentage_brokerage() {
        let c = CostSchedule::zerodha_nse_intraday().compute(Side::Buy, dec!(10000));
        assert_eq!(c.brokerage, dec!(3));
    }

    #[test]
    fn intraday_stt_only_on_sell() {
        let s = CostSchedule::zerodha_nse_intraday();
        assert_eq!(s.compute(Side::Buy, dec!(100000)).stt, dec!(0));
        assert_eq!(s.compute(Side::Sell, dec!(100000)).stt, dec!(25));
    }

    #[test]
    fn round_trip_delivery_cost_is_about_quarter_percent() {
        // A 1% move on delivery loses ~0.24% to costs: the bar any swing edge must clear.
        let s = CostSchedule::zerodha_nse_delivery();
        let total = s.compute(Side::Buy, dec!(100000)).total() + s.compute(Side::Sell, dec!(101000)).total();
        assert_eq!(total, dec!(239.21));
    }
}

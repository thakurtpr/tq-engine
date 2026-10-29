//! Indian equity transaction costs. Rates are data, not code: a fee change is a config edit.
//!
//! Source: docs/costs-india.md, which cites <https://zerodha.com/charges> (fetched 2026-10-30).
//! The doc's worked examples are this crate's test fixtures.
//!
//! Rounding: STT is rounded to whole rupees, half-up, PER ORDER (STT Rules 2004, Rule 4; Zerodha
//! support examples). Every other component stays unrounded, and a trade's cost is rounded ONCE,
//! to paise (half-up), over the whole round trip (see [`CostSchedule::round_trip`]).
//! Whether brokers apply the STT rounding per order or per contract-note day total is
//! unverified; they differ only with several same-day fills in one scrip (under Rs 1 per day).
//!
//! Known over-charge: the DP charge is levied once per scrip per sell day, but
//! [`CostSchedule::compute`] adds it to every delivery sell. Multiple partial sells of the same
//! scrip on one day are therefore over-charged. The backtester closes whole positions, at most
//! one exit per symbol per daily bar, so it is not affected today.

use rust_decimal::{Decimal, RoundingStrategy};
use tq_core::Side;

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;
const PAISE_DP: u32 = 2;
const RUPEE_DP: u32 = 0;

/// Rounds rupees to paise, half-up (2.525 -> 2.53).
pub fn round_paise(amount: Decimal) -> Decimal {
    amount.round_dp_with_strategy(PAISE_DP, RoundingStrategy::MidpointAwayFromZero)
}

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
    /// Depository charge per sell (delivery only), before GST. Rs 13.00 + 18% GST = Rs 15.34.
    pub dp_per_sell: Decimal,
}

impl CostSchedule {
    pub fn zerodha_nse_delivery() -> Self {
        Self {
            brokerage_pct: Decimal::ZERO,
            brokerage_cap: None,
            stt_buy_pct: Decimal::new(1, 1),    // 0.1
            stt_sell_pct: Decimal::new(1, 1),   // 0.1
            exchange_pct: Decimal::new(307, 5), // 0.00307 (NSE)
            sebi_pct: Decimal::new(1, 4),       // 0.0001 (Rs 10 / crore)
            stamp_buy_pct: Decimal::new(15, 3), // 0.015
            gst_pct: Decimal::from(18),
            dp_per_sell: Decimal::from(13), // 3.50 CDSL + 9.50 Zerodha
        }
    }

    pub fn zerodha_bse_delivery() -> Self {
        Self {
            exchange_pct: Decimal::new(375, 5), // 0.00375 (BSE)
            ..Self::zerodha_nse_delivery()
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

    /// Costs of one order on one side: STT in whole rupees, everything else unrounded.
    pub fn compute(&self, side: Side, turnover: Decimal) -> CostBreakdown {
        let pct = |rate: Decimal| turnover * rate / HUNDRED;
        let raw_brokerage = pct(self.brokerage_pct);
        let brokerage = self.brokerage_cap.map_or(raw_brokerage, |cap| raw_brokerage.min(cap));
        let (stt, stamp, dp) = match side {
            Side::Buy => (pct(self.stt_buy_pct), pct(self.stamp_buy_pct), Decimal::ZERO),
            Side::Sell => (pct(self.stt_sell_pct), Decimal::ZERO, self.dp_per_sell),
        };
        let stt = stt.round_dp_with_strategy(RUPEE_DP, RoundingStrategy::MidpointAwayFromZero);
        let exchange = pct(self.exchange_pct);
        let sebi = pct(self.sebi_pct);
        // GST applies to services (brokerage, exchange, SEBI, DP), not to taxes (STT, stamp).
        let gst = (brokerage + exchange + sebi + dp) * self.gst_pct / HUNDRED;
        CostBreakdown { brokerage, stt, exchange, sebi, stamp, dp, gst }
    }

    /// Long round trip (buy then sell) cost, rounded once to paise.
    pub fn round_trip(&self, buy_turnover: Decimal, sell_turnover: Decimal) -> Decimal {
        let buy = self.compute(Side::Buy, buy_turnover).total();
        let sell = self.compute(Side::Sell, sell_turnover).total();
        round_paise(buy + sell)
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
    /// Total in rupees, not rounded to paise. Round once per trade with [`round_paise`].
    pub fn total(&self) -> Decimal {
        self.brokerage + self.stt + self.exchange + self.sebi + self.stamp + self.dp + self.gst
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // Fixtures from docs/costs-india.md: buy 100 @ 1000, sell 100 @ 1010.
    const BUY: Decimal = dec!(100000);
    const SELL: Decimal = dec!(101000);

    #[test]
    fn nse_delivery_round_trip_matches_doc_fixture() {
        assert_eq!(CostSchedule::zerodha_nse_delivery().round_trip(BUY, SELL), dec!(238.86));
    }

    #[test]
    fn nse_intraday_round_trip_matches_doc_fixture() {
        // STT 25.25 rounds to 25 (per order, whole rupees, half-up).
        assert_eq!(CostSchedule::zerodha_nse_intraday().round_trip(BUY, SELL), dec!(82.72));
    }

    #[test]
    fn bse_delivery_round_trip_matches_doc_fixture() {
        assert_eq!(CostSchedule::zerodha_bse_delivery().round_trip(BUY, SELL), dec!(240.47));
    }

    #[test]
    fn nse_intraday_small_round_trip_matches_doc_fixture() {
        let s = CostSchedule::zerodha_nse_intraday();
        // STT 2.525 rounds UP to 3.
        assert_eq!(s.round_trip(dec!(10000), dec!(10100)), dec!(11.17));
    }

    #[test]
    fn components_stay_unrounded_until_the_trade_total() {
        // STT 100 + exch 3.07 + SEBI 0.1 + stamp 15 + GST 0.18 * 3.17 = 118.7406
        let c = CostSchedule::zerodha_nse_delivery().compute(Side::Buy, BUY);
        assert_eq!(c.exchange, dec!(3.07));
        assert_eq!(c.gst, dec!(0.5706));
        assert_eq!(c.total(), dec!(118.7406));
    }

    #[test]
    fn delivery_sell_pays_dp_pre_gst_and_no_stamp() {
        let c = CostSchedule::zerodha_nse_delivery().compute(Side::Sell, SELL);
        assert_eq!(c.stamp, dec!(0));
        assert_eq!(c.dp, dec!(13));
    }

    #[test]
    fn intraday_brokerage_is_capped_at_twenty_per_order() {
        let c = CostSchedule::zerodha_nse_intraday().compute(Side::Buy, BUY);
        assert_eq!(c.brokerage, dec!(20));
    }

    #[test]
    fn intraday_stt_only_on_sell() {
        let s = CostSchedule::zerodha_nse_intraday();
        assert_eq!(s.compute(Side::Buy, BUY).stt, dec!(0));
        assert_eq!(s.compute(Side::Sell, BUY).stt, dec!(25));
    }

    #[test]
    fn stt_rounds_to_whole_rupees_half_up_per_order() {
        let s = CostSchedule::zerodha_nse_intraday();
        assert_eq!(s.compute(Side::Sell, dec!(10100)).stt, dec!(3)); // 2.525
        assert_eq!(s.compute(Side::Sell, dec!(101000)).stt, dec!(25)); // 25.25
        assert_eq!(s.compute(Side::Sell, dec!(402000)).stt, dec!(101)); // 100.50
    }

    #[test]
    fn rounds_half_paisa_up() {
        assert_eq!(round_paise(dec!(2.525)), dec!(2.53));
        assert_eq!(round_paise(dec!(2.524)), dec!(2.52));
    }
}

//! Crypto spot costs: exchange trading fees, and the Indian-resident 1% TDS on VDA sells.
//!
//! Fees: Binance spot regular tier (VIP 0) is 0.10% maker and 0.10% taker, before any BNB
//! discount. Rates are config ([`CryptoSpotFees`]). See docs/costs-crypto.md.
//!
//! TDS (Income-tax Act 1961 s.194S; carried into the Income-tax Act 2025 as s.393(1), Table
//! Sl. No. 8(vi), from 2026-04-01): 1% of the sale consideration is withheld on the transfer
//! of a virtual digital asset. It is NOT a cost: it is an advance tax payment, credited against
//! the year's tax liability (refundable if it exceeds it). It is therefore modelled as cash that
//! leaves the trading account and becomes a receivable ([`TdsLedger`]). It still matters for
//! a high-turnover strategy because the cash is locked until the tax return is processed.

use rust_decimal::Decimal;

use crate::CostSchedule;

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;

/// Whether an order adds liquidity (resting limit order) or takes it (market order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liquidity {
    Maker,
    Taker,
}

/// Exchange fees in percent of turnover (price * qty, in the quote currency).
#[derive(Debug, Clone, PartialEq)]
pub struct CryptoSpotFees {
    pub maker_pct: Decimal,
    pub taker_pct: Decimal,
}

impl Default for CryptoSpotFees {
    /// Binance spot VIP 0, no BNB discount: 0.10% maker, 0.10% taker.
    fn default() -> Self {
        Self { maker_pct: Decimal::new(1, 1), taker_pct: Decimal::new(1, 1) }
    }
}

impl CryptoSpotFees {
    pub fn rate_pct(&self, liquidity: Liquidity) -> Decimal {
        match liquidity {
            Liquidity::Maker => self.maker_pct,
            Liquidity::Taker => self.taker_pct,
        }
    }

    /// Fee on one fill, in the quote currency, unrounded.
    pub fn fee(&self, liquidity: Liquidity, turnover: Decimal) -> Decimal {
        turnover * self.rate_pct(liquidity) / HUNDRED
    }

    /// A [`CostSchedule`] charging only this fee (no STT, stamp, GST or DP), so the existing
    /// `Paper` broker can bill crypto fills. Market orders pay [`Liquidity::Taker`].
    pub fn schedule(&self, liquidity: Liquidity) -> CostSchedule {
        CostSchedule {
            brokerage_pct: self.rate_pct(liquidity),
            brokerage_cap: None,
            stt_buy_pct: Decimal::ZERO,
            stt_sell_pct: Decimal::ZERO,
            exchange_pct: Decimal::ZERO,
            sebi_pct: Decimal::ZERO,
            stamp_buy_pct: Decimal::ZERO,
            gst_pct: Decimal::ZERO,
            dp_per_sell: Decimal::ZERO,
        }
    }
}

/// Section 194S TDS: percent of the sell consideration withheld. The statutory per-year
/// exemption thresholds (Rs 50,000 / Rs 10,000) are ignored, which over-withholds slightly and
/// is the conservative choice for an active trader who crosses them within days.
#[derive(Debug, Clone, PartialEq)]
pub struct Tds194S {
    pub rate_pct: Decimal,
}

impl Default for Tds194S {
    fn default() -> Self {
        Self { rate_pct: Decimal::ONE }
    }
}

impl Tds194S {
    pub fn withheld(&self, sell_turnover: Decimal) -> Decimal {
        if sell_turnover <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        sell_turnover * self.rate_pct / HUNDRED
    }
}

/// Running total of TDS withheld: cash out of the account, but an asset (tax credit).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TdsLedger {
    rule: Tds194S,
    receivable: Decimal,
}

impl TdsLedger {
    pub fn new(rule: Tds194S) -> Self {
        Self { rule, receivable: Decimal::ZERO }
    }

    /// Records a sell and returns the cash withheld from its proceeds.
    pub fn on_sell(&mut self, sell_turnover: Decimal) -> Decimal {
        let w = self.rule.withheld(sell_turnover);
        self.receivable += w;
        w
    }

    /// TDS withheld so far and not yet reclaimed.
    pub fn receivable(&self) -> Decimal {
        self.receivable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::Side;

    #[test]
    fn default_binance_fee_is_ten_bps_each_way() {
        let f = CryptoSpotFees::default();
        assert_eq!(f.fee(Liquidity::Taker, dec!(10000)), dec!(10));
        assert_eq!(f.fee(Liquidity::Maker, dec!(10000)), dec!(10));
    }

    #[test]
    fn fees_are_configurable_per_liquidity() {
        let f = CryptoSpotFees { maker_pct: dec!(0.02), taker_pct: dec!(0.05) };
        assert_eq!(f.fee(Liquidity::Maker, dec!(10000)), dec!(2));
        assert_eq!(f.fee(Liquidity::Taker, dec!(10000)), dec!(5));
    }

    #[test]
    fn schedule_charges_only_the_fee_on_both_sides() {
        let s = CryptoSpotFees::default().schedule(Liquidity::Taker);
        assert_eq!(s.compute(Side::Buy, dec!(2500.50)).total(), dec!(2.50050));
        assert_eq!(s.compute(Side::Sell, dec!(2500.50)).total(), dec!(2.50050));
    }

    #[test]
    fn tds_is_one_percent_of_sell_consideration() {
        assert_eq!(Tds194S::default().withheld(dec!(12345.60)), dec!(123.4560));
        assert_eq!(Tds194S::default().withheld(dec!(0)), dec!(0));
    }

    #[test]
    fn ledger_accumulates_reclaimable_tds() {
        let mut l = TdsLedger::new(Tds194S::default());
        assert_eq!(l.on_sell(dec!(1000)), dec!(10));
        assert_eq!(l.on_sell(dec!(500)), dec!(5));
        assert_eq!(l.receivable(), dec!(15));
    }
}

//! Liquidity-aware fill prices: the adopted model of `docs/slippage-model.md`, section 5.
//!
//! - Base slippage (5 bps per side by default) for orders up to 0.05% of 20-day ADV.
//! - Above that, square-root impact: `half_spread + open_noise + Y * sigma * sqrt(q / ADV)`,
//!   floored at the base (a bigger order never pays less than a small one).
//! - Stop-loss exits pay 2x; the whole figure is then scaled by the run's stress multiplier.
//! - Entries are cut to 1% of 20-day ADV (and recorded as capped by the engine).
//! - Filled prices are rounded to the Rs 0.05 tick in the direction worse for the trader.
//! - ADV and sigma come only from bars strictly before the fill bar ([`Liquidity`]).
//!
//! Money stays `Decimal`. Sigma and the square root are statistics, computed in `f64` and
//! converted back to `Decimal` basis points rounded to [`BPS_DP`] places.

use std::collections::VecDeque;

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use tq_core::{Bar, Price, Qty, Side};

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;
const BPS_PER_UNIT: f64 = 10_000.0;
/// Decimal places kept on a computed impact, in basis points.
pub const BPS_DP: u32 = 4;
/// Bars of history behind ADV and sigma.
pub const DEFAULT_LOOKBACK: usize = 20;

/// Parameters of the adopted model. `Default` gives the values in `docs/slippage-model.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct SlippageModel {
    /// Orders up to this percent of ADV pay only the base slippage.
    pub impact_threshold_adv_pct: Decimal,
    pub half_spread_bps: Decimal,
    pub open_noise_bps: Decimal,
    /// The square-root law's coefficient `Y`.
    pub impact_coef: Decimal,
    /// Multiplier on stop-loss exits.
    pub stop_exit_mult: Decimal,
    /// Entries are cut to this percent of ADV, in whole shares.
    pub adv_cap_pct: Decimal,
    /// Exchange tick; filled prices are rounded to it, against the trader.
    pub tick: Decimal,
    pub lookback: usize,
}

impl Default for SlippageModel {
    fn default() -> Self {
        Self {
            impact_threshold_adv_pct: Decimal::new(5, 2),
            half_spread_bps: Decimal::new(15, 1),
            open_noise_bps: Decimal::TWO,
            impact_coef: Decimal::new(7, 1),
            stop_exit_mult: Decimal::TWO,
            adv_cap_pct: Decimal::ONE,
            tick: Decimal::new(5, 2),
            lookback: DEFAULT_LOOKBACK,
        }
    }
}

/// ADV (shares) and sigma (daily close-to-close return sd) from prior bars; `None` until warm.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LiquidityStats {
    pub adv: Option<Decimal>,
    pub sigma: Option<f64>,
}

impl SlippageModel {
    /// Adverse slippage in bps before the stress multiplier. `stop_exit` doubles it.
    pub fn bps(&self, base_bps: Decimal, qty: Qty, stats: &LiquidityStats, stop_exit: bool) -> Decimal {
        let bps = self.impact_bps(qty, stats).map_or(base_bps, |impact| impact.max(base_bps));
        if stop_exit {
            bps * self.stop_exit_mult
        } else {
            bps
        }
    }

    /// Square-root formula, only when the order exceeds the threshold and ADV and sigma are known.
    fn impact_bps(&self, qty: Qty, stats: &LiquidityStats) -> Option<Decimal> {
        let adv = stats.adv.filter(|a| *a > Decimal::ZERO)?;
        if qty / adv * HUNDRED <= self.impact_threshold_adv_pct {
            return None;
        }
        let participation = (qty / adv).to_f64()?;
        let impact = self.impact_coef.to_f64()? * stats.sigma? * participation.sqrt() * BPS_PER_UNIT;
        let fixed = self.half_spread_bps + self.open_noise_bps;
        Decimal::from_f64(impact).map(|i| (fixed + i).round_dp(BPS_DP))
    }

    /// Largest entry in whole shares: `adv_cap_pct` of ADV. Zero when ADV is unknown, so an
    /// entry is never sized without a liquidity estimate.
    pub fn adv_cap(&self, stats: &LiquidityStats) -> Qty {
        stats.adv.map_or(Decimal::ZERO, |adv| (adv * self.adv_cap_pct / HUNDRED).floor())
    }

    /// Rounds to the tick against the trader: buys up, sells down (never below one tick).
    pub fn round_to_tick(&self, price: Price, side: Side) -> Price {
        if self.tick <= Decimal::ZERO {
            return price;
        }
        let ticks = price / self.tick;
        match side {
            Side::Buy => ticks.ceil() * self.tick,
            Side::Sell => (ticks.floor() * self.tick).max(self.tick),
        }
    }
}

/// Rolling volume and close history of one symbol. Push each bar AFTER its fills are done, so
/// [`Liquidity::stats`] only ever sees bars before the one being filled.
#[derive(Debug, Clone)]
pub struct Liquidity {
    lookback: usize,
    volumes: VecDeque<Decimal>,
    closes: VecDeque<Decimal>,
}

impl Liquidity {
    pub fn new(lookback: usize) -> Self {
        let lookback = lookback.max(2);
        Self { lookback, volumes: VecDeque::with_capacity(lookback), closes: VecDeque::with_capacity(lookback + 1) }
    }

    pub fn push(&mut self, bar: &Bar) {
        self.volumes.push_back(bar.volume);
        self.closes.push_back(bar.close);
        if self.volumes.len() > self.lookback {
            self.volumes.pop_front();
        }
        if self.closes.len() > self.lookback + 1 {
            self.closes.pop_front();
        }
    }

    pub fn stats(&self) -> LiquidityStats {
        let adv = (self.volumes.len() == self.lookback)
            .then(|| self.volumes.iter().sum::<Decimal>() / Decimal::from(self.lookback));
        LiquidityStats { adv, sigma: self.sigma() }
    }

    /// Sample sd of the last `lookback` close-to-close returns.
    fn sigma(&self) -> Option<f64> {
        if self.closes.len() <= self.lookback {
            return None;
        }
        let closes: Vec<f64> = self.closes.iter().map(|c| c.to_f64()).collect::<Option<_>>()?;
        let rets: Vec<f64> = closes.windows(2).map(|w| w[1] / w[0] - 1.0).collect();
        let n = rets.len() as f64;
        let mean = rets.iter().sum::<f64>() / n;
        let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        Some(var.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn stats(adv: Decimal, sigma: f64) -> LiquidityStats {
        LiquidityStats { adv: Some(adv), sigma: Some(sigma) }
    }

    #[test]
    fn small_orders_pay_the_base_rate() {
        // 500 of 1_000_000 shares = 0.05% of ADV: still the flat base.
        let m = SlippageModel::default();
        assert_eq!(m.bps(dec!(5), dec!(500), &stats(dec!(1000000), 0.015), false), dec!(5));
    }

    #[test]
    fn large_orders_pay_square_root_impact() {
        // 1% of ADV, sigma 1.5%: 3.5 + 0.7 * 0.015 * 0.1 * 10_000 = 14.0 bps (doc table: 11.0-18.5).
        let m = SlippageModel::default();
        assert_eq!(m.bps(dec!(5), dec!(10000), &stats(dec!(1000000), 0.015), false), dec!(14));
    }

    #[test]
    fn impact_never_undercuts_the_base() {
        // 0.1% of ADV in a very quiet stock: formula gives 3.5 + 0.7 * 0.001 * 0.0316 * 1e4 = 3.72.
        let m = SlippageModel::default();
        assert_eq!(m.bps(dec!(5), dec!(1000), &stats(dec!(1000000), 0.001), false), dec!(5));
    }

    #[test]
    fn stop_exits_pay_double() {
        let m = SlippageModel::default();
        assert_eq!(m.bps(dec!(5), dec!(1), &stats(dec!(1000000), 0.015), true), dec!(10));
    }

    #[test]
    fn unknown_liquidity_falls_back_to_base_and_caps_entries_at_zero() {
        let m = SlippageModel::default();
        let none = LiquidityStats::default();
        assert_eq!(m.bps(dec!(5), dec!(100000), &none, false), dec!(5));
        assert_eq!(m.adv_cap(&none), dec!(0));
    }

    #[test]
    fn adv_cap_is_one_percent_in_whole_shares() {
        assert_eq!(SlippageModel::default().adv_cap(&stats(dec!(123456), 0.01)), dec!(1234));
    }

    #[test]
    fn tick_rounding_goes_against_the_trader() {
        let m = SlippageModel::default();
        assert_eq!(m.round_to_tick(dec!(100.051), Side::Buy), dec!(100.10));
        assert_eq!(m.round_to_tick(dec!(100.099), Side::Sell), dec!(100.05));
        assert_eq!(m.round_to_tick(dec!(100.05), Side::Buy), dec!(100.05));
        assert_eq!(m.round_to_tick(dec!(0.01), Side::Sell), dec!(0.05));
    }

    fn bar(close: Decimal, volume: Decimal) -> Bar {
        Bar::new(0, close, close, close, close, volume).unwrap()
    }

    #[test]
    fn liquidity_needs_a_full_window_and_rolls_forward() {
        let mut l = Liquidity::new(3);
        for v in [dec!(100), dec!(200)] {
            l.push(&bar(dec!(10), v));
        }
        assert_eq!(l.stats(), LiquidityStats::default());
        l.push(&bar(dec!(10), dec!(300)));
        assert_eq!(l.stats().adv, Some(dec!(200)));
        assert_eq!(l.stats().sigma, None, "3 closes give only 2 returns");
        l.push(&bar(dec!(10), dec!(600)));
        assert_eq!(l.stats().adv, Some(dec!(1100) / dec!(3)));
        assert_eq!(l.stats().sigma, Some(0.0));
    }

    #[test]
    fn sigma_is_sample_sd_of_close_to_close_returns() {
        // Returns +10%, -10%, +10%: mean 1/30, sample sd = sqrt(((2/30)^2 * 2 + (4/30)^2) / 2).
        let mut l = Liquidity::new(3);
        for c in [dec!(100), dec!(110), dec!(99), dec!(108.9)] {
            l.push(&bar(c, dec!(1)));
        }
        let expected = ((2.0_f64 / 30.0).powi(2) * 2.0 + (4.0_f64 / 30.0).powi(2)) / 2.0;
        assert!((l.stats().sigma.unwrap_or(0.0) - expected.sqrt()).abs() < 1e-12);
    }
}

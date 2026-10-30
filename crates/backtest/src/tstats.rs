//! Gate 4c statistics of `docs/validation-protocol.md` v2, section 4.
//!
//! - **Primary:** t of the mean daily portfolio return with a Newey-West (Bartlett kernel) HAC
//!   standard error, bandwidth `L = max(floor(4 (T/100)^(2/9)), longest holding period)`, capped
//!   at `T/4`.
//! - **Cross-checks:** the naive t of calendar-month returns (compounded daily returns), and the
//!   month-clustered t of per-trade returns (computed by the caller from its trades).
//! - **Gate value:** the HAC t, unless a cross-check is more than 0.5 below it; then the lowest.
//!
//! Statistics, not money: `f64` throughout.

use std::collections::BTreeMap;

use tq_core::Timestamp;

use crate::returns::month_key;
use crate::report::Trade;

/// A cross-check this far below the HAC t makes the gate use the lowest of the three.
pub const CROSS_CHECK_GAP: f64 = 0.5;

/// The automatic Newey-West bandwidth `floor(4 (T/100)^(2/9))`.
pub fn newey_west_lag(t: usize) -> usize {
    (4.0 * (t as f64 / 100.0).powf(2.0 / 9.0)).floor() as usize
}

/// Protocol bandwidth: the larger of the automatic lag and the longest holding period (trading
/// days), capped at `T/4`.
pub fn protocol_lag(t: usize, longest_hold: usize) -> usize {
    newey_west_lag(t).max(longest_hold).min(t / 4)
}

/// HAC t of the mean against zero with `lag` Bartlett-weighted autocovariances.
/// `None` below two values or with a non-positive long-run variance.
pub fn hac_t(values: &[f64], lag: usize) -> Option<f64> {
    let n = values.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mean = values.iter().sum::<f64>() / nf;
    let e: Vec<f64> = values.iter().map(|x| x - mean).collect();
    let gamma = |j: usize| e[j..].iter().zip(&e).map(|(a, b)| a * b).sum::<f64>() / nf;
    let lag = lag.min(n - 1);
    let weighted: f64 = (1..=lag).map(|j| (1.0 - j as f64 / (lag as f64 + 1.0)) * gamma(j)).sum();
    let lrv = gamma(0) + 2.0 * weighted;
    (lrv > 0.0).then(|| mean / (lrv / nf).sqrt())
}

/// Compounded return of each calendar month (IST) of a daily return series, oldest first.
pub fn monthly_returns(daily: &[(Timestamp, f64)]) -> Vec<f64> {
    let mut growth: BTreeMap<(i64, u32), f64> = BTreeMap::new();
    for &(ts, r) in daily {
        *growth.entry(month_key(ts)).or_insert(1.0) *= 1.0 + r;
    }
    growth.into_values().map(|g| g - 1.0).collect()
}

/// Naive one-sample t of the mean; `None` below two values or with zero spread.
pub fn naive_t(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (var > 0.0).then(|| mean / (var / n).sqrt())
}

/// The figures gate 4c prints, and the one it is judged on.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GateT {
    pub hac: Option<f64>,
    pub lag: usize,
    pub monthly: Option<f64>,
    pub per_trade_clustered: Option<f64>,
    /// HAC, or the lowest of the three when a cross-check is more than 0.5 below HAC.
    pub gate: Option<f64>,
}

/// Gate 4c on a daily return series; `longest_hold` in trading days (0 without closed trades).
pub fn gate_t(daily: &[(Timestamp, f64)], longest_hold: usize, per_trade_clustered: Option<f64>) -> GateT {
    let values: Vec<f64> = daily.iter().map(|d| d.1).collect();
    let lag = protocol_lag(values.len(), longest_hold);
    let hac = hac_t(&values, lag);
    let monthly = naive_t(&monthly_returns(daily));
    GateT { hac, lag, monthly, per_trade_clustered, gate: judge(hac, monthly, per_trade_clustered) }
}

/// Longest closed trade in trading days: equity-curve points from entry to exit, inclusive of
/// the exit day and exclusive of the entry day (a next-day exit holds 1 day). 0 without trades.
pub fn longest_hold(curve: &[(Timestamp, rust_decimal::Decimal)], trades: &[Trade]) -> usize {
    let days: Vec<Timestamp> = curve.iter().map(|p| p.0).collect();
    let index = |ts: Timestamp| days.partition_point(|d| *d <= ts);
    trades.iter().map(|t| index(t.exit_ts).saturating_sub(index(t.entry_ts))).max().unwrap_or(0)
}

fn judge(hac: Option<f64>, monthly: Option<f64>, per_trade: Option<f64>) -> Option<f64> {
    let h = hac?;
    let checks: Vec<f64> = [monthly, per_trade].into_iter().flatten().collect();
    if checks.iter().any(|c| *c < h - CROSS_CHECK_GAP) {
        Some(checks.into_iter().fold(h, f64::min))
    } else {
        Some(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JAN2_2012: Timestamp = 1_325_498_400_000;
    const DAY_MS: i64 = 86_400_000;

    #[test]
    fn hac_matches_a_hand_calculation() {
        // x = 1,3,2,6: mean 3, e = -2,0,-1,3; gamma0 = 14/4 = 3.5; gamma1 = -3/4.
        // L = 1: lrv = 3.5 + 2 * 0.5 * -0.75 = 2.75 -> se = sqrt(2.75 / 4).
        let x = [1.0, 3.0, 2.0, 6.0];
        let t1 = hac_t(&x, 1).unwrap_or(0.0);
        assert!((t1 - 3.0 / (2.75_f64 / 4.0).sqrt()).abs() < 1e-12);
        let t0 = hac_t(&x, 0).unwrap_or(0.0);
        assert!((t0 - 3.0 / (3.5_f64 / 4.0).sqrt()).abs() < 1e-12, "L = 0 is the population-sd t");
        assert_eq!(hac_t(&[1.0], 0), None);
    }

    #[test]
    fn bandwidth_follows_the_protocol() {
        assert_eq!(newey_west_lag(2465), 8, "review: L = 8 for the 10-year window");
        assert_eq!(protocol_lag(2465, 198), 198);
        assert_eq!(protocol_lag(2465, 2000), 616, "capped at T/4");
        assert_eq!(protocol_lag(100, 0), 4);
    }

    #[test]
    fn monthly_returns_compound_days_within_a_month() {
        let d = [(JAN2_2012, 0.1), (JAN2_2012 + DAY_MS, 0.1), (JAN2_2012 + 31 * DAY_MS, -0.5)];
        let m = monthly_returns(&d);
        assert!((m[0] - 0.21).abs() < 1e-12 && (m[1] + 0.5).abs() < 1e-12);
    }

    #[test]
    fn gate_uses_hac_unless_a_cross_check_is_far_below() {
        assert_eq!(judge(Some(2.0), Some(1.6), Some(1.9)), Some(2.0));
        assert_eq!(judge(Some(2.0), Some(1.4), Some(1.9)), Some(1.4));
        assert_eq!(judge(Some(2.0), Some(1.9), Some(0.1)), Some(0.1));
        assert_eq!(judge(Some(2.0), None, Some(2.5)), Some(2.0), "a higher cross-check never raises it");
        assert_eq!(judge(None, Some(1.0), None), None);
    }

    #[test]
    fn longest_hold_counts_curve_days_between_entry_and_exit() {
        use crate::report::ExitReason;
        use rust_decimal::Decimal;
        use tq_core::Symbol;
        let curve: Vec<(Timestamp, Decimal)> = (0..10).map(|i| (i, Decimal::ONE)).collect();
        let trade = |entry, exit| Trade {
            symbol: Symbol("X".into()),
            entry_ts: entry,
            entry_price: Decimal::ONE,
            stop: Decimal::ZERO,
            exit_ts: exit,
            exit_price: Decimal::ONE,
            qty: Decimal::ONE,
            gross_pnl: Decimal::ZERO,
            costs: Decimal::ZERO,
            net_pnl: Decimal::ZERO,
            exit_reason: ExitReason::Signal,
            slippage: Decimal::ZERO,
        };
        assert_eq!(longest_hold(&curve, &[trade(1, 2), trade(2, 7)]), 5);
        assert_eq!(longest_hold(&curve, &[]), 0);
    }

    #[test]
    fn gate_t_reports_every_figure() {
        let daily: Vec<(Timestamp, f64)> =
            (0..400).map(|i| (JAN2_2012 + i * DAY_MS, if i % 3 == 0 { 0.01 } else { -0.002 })).collect();
        let g = gate_t(&daily, 0, None);
        assert_eq!(g.lag, newey_west_lag(400));
        assert!(g.hac.is_some() && g.monthly.is_some());
        assert_eq!(g.gate, judge(g.hac, g.monthly, None));
    }
}

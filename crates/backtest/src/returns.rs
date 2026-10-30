//! Portfolio statistics from a daily equity curve: CAGR, Sharpe, max drawdown, Calmar,
//! exposure and a month-clustered t-statistic of daily returns.
//!
//! These are statistics, not money, so they are `f64` (they need roots and powers); only the
//! max drawdown, computed from `Decimal` equity, starts exact.
//!
//! The t-statistic follows the independent review's `docs/review-backtest-01-scripts/tstat.py`
//! (`cluster_t`): cluster-robust standard error of the mean with the CR1 small-sample factor
//! G/(G-1), clusters = calendar months. The review applied it to trade returns keyed by entry
//! month; here it is applied to daily portfolio returns keyed by the month of each day, which
//! allows for correlation between days of the same month.

use std::collections::BTreeMap;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use tq_core::Timestamp;

use crate::metrics::max_drawdown_pct;

const TRADING_DAYS_PER_YEAR: f64 = 252.0;
const DAY_MS: i64 = 86_400_000;
const DAYS_PER_YEAR: f64 = 365.25;
/// IST is UTC+05:30; dates are taken in IST so a bar's calendar day is the NSE session's day.
const IST_OFFSET_MS: i64 = 19_800_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReturnStats {
    pub days: usize,
    /// Calendar years from the day before the first point to the last point.
    pub years: f64,
    pub total_return_pct: f64,
    pub cagr_pct: Option<f64>,
    /// Annualised (sqrt 252) Sharpe of daily returns, risk-free rate 0.
    pub sharpe: Option<f64>,
    pub max_drawdown_pct: f64,
    /// CAGR / max drawdown; `None` without a drawdown.
    pub calmar: Option<f64>,
    /// Month-clustered t of the mean daily return (tstat.py's CR1 estimator) and the month count.
    pub t_month_clustered: Option<f64>,
    pub months: usize,
}

/// Daily simple returns, the first measured from `initial`.
pub fn daily_returns(initial: Decimal, curve: &[(Timestamp, Decimal)]) -> Vec<(Timestamp, f64)> {
    let mut prev = initial;
    let mut out = Vec::with_capacity(curve.len());
    for &(ts, e) in curve {
        if let Some(r) = (!prev.is_zero()).then(|| (e / prev - Decimal::ONE).to_f64()).flatten() {
            out.push((ts, r));
        }
        prev = e;
    }
    out
}

pub fn stats(initial: Decimal, curve: &[(Timestamp, Decimal)]) -> ReturnStats {
    let (Some(first), Some(last)) = (curve.first(), curve.last()) else { return ReturnStats::default() };
    let rets = daily_returns(initial, curve);
    let growth = (last.1 / initial).to_f64().unwrap_or(f64::NAN);
    let years = ((last.0 - first.0) / DAY_MS + 1) as f64 / DAYS_PER_YEAR;
    let cagr = (growth > 0.0 && years > 0.0).then(|| (growth.powf(1.0 / years) - 1.0) * 100.0);
    let equity: Vec<Decimal> = std::iter::once(initial).chain(curve.iter().map(|p| p.1)).collect();
    let mdd = max_drawdown_pct(&equity).to_f64().unwrap_or(f64::NAN);
    let values: Vec<f64> = rets.iter().map(|r| r.1).collect();
    let months: Vec<(i64, u32)> = rets.iter().map(|r| month_key(r.0)).collect();
    let clustered = cluster_t(&values, &months);
    ReturnStats {
        days: curve.len(),
        years,
        total_return_pct: (growth - 1.0) * 100.0,
        cagr_pct: cagr,
        sharpe: sharpe(&values),
        max_drawdown_pct: mdd,
        calmar: cagr.filter(|_| mdd > 0.0).map(|c| c / mdd),
        t_month_clustered: clustered.map(|c| c.0),
        months: clustered.map_or(0, |c| c.1),
    }
}

/// Annualised Sharpe of daily returns (sample sd); `None` below two values or with zero spread.
pub fn sharpe(values: &[f64]) -> Option<f64> {
    let (mean, sd) = mean_sd(values)?;
    (sd > 0.0).then(|| mean / sd * TRADING_DAYS_PER_YEAR.sqrt())
}

fn mean_sd(values: &[f64]) -> Option<(f64, f64)> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    Some((mean, var.sqrt()))
}

/// Cluster-robust t of the mean against zero, exactly as tstat.py's `cluster_t`:
/// `se = sqrt(sum_g (sum_{i in g} (x_i - mean))^2 * G / (G - 1)) / n`. Returns `(t, G)`;
/// `None` with fewer than two clusters or zero standard error.
pub fn cluster_t<K: Ord + Clone>(values: &[f64], keys: &[K]) -> Option<(f64, usize)> {
    if values.is_empty() || values.len() != keys.len() {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let mut sums: BTreeMap<K, f64> = BTreeMap::new();
    for (v, k) in values.iter().zip(keys) {
        *sums.entry(k.clone()).or_default() += v - mean;
    }
    let g = sums.len();
    if g < 2 {
        return None;
    }
    let gf = g as f64;
    let se = (sums.values().map(|s| s * s).sum::<f64>() * gf / (gf - 1.0)).sqrt() / n;
    (se > 0.0).then(|| (mean / se, g))
}

/// Mean of `exposure / equity`, in percent of equity, over the days with positive equity.
pub fn exposure_pct(curve: &[(Timestamp, Decimal)], exposure: &[Decimal]) -> Option<f64> {
    let shares: Vec<f64> = curve
        .iter()
        .zip(exposure)
        .filter(|(p, _)| p.1 > Decimal::ZERO)
        .filter_map(|(p, x)| (*x / p.1).to_f64())
        .collect();
    (!shares.is_empty()).then(|| shares.iter().sum::<f64>() / shares.len() as f64 * 100.0)
}

/// Return of each calendar year in the curve, in percent (the first from `initial`).
pub fn yearly_returns(initial: Decimal, curve: &[(Timestamp, Decimal)]) -> Vec<(i64, f64)> {
    let mut year_end: BTreeMap<i64, Decimal> = BTreeMap::new();
    for &(ts, e) in curve {
        year_end.insert(civil_date(ts).0, e);
    }
    let mut prev = initial;
    let mut out = Vec::new();
    for (year, end) in year_end {
        if let Some(r) = (!prev.is_zero()).then(|| ((end / prev - Decimal::ONE) * Decimal::ONE_HUNDRED).to_f64()).flatten() {
            out.push((year, r));
        }
        prev = end;
    }
    out
}

/// Strategy minus benchmark daily return on the dates both have.
pub fn excess_returns(a: &[(Timestamp, f64)], b: &[(Timestamp, f64)]) -> Vec<(Timestamp, f64)> {
    let bench: BTreeMap<Timestamp, f64> = b.iter().copied().collect();
    a.iter().filter_map(|&(ts, r)| bench.get(&ts).map(|rb| (ts, r - rb))).collect()
}

/// `(year, month)` of `ts` in IST.
pub fn month_key(ts: Timestamp) -> (i64, u32) {
    let (y, m, _) = civil_date(ts);
    (y, m)
}

/// IST calendar date of `ts` (H. Hinnant's days-to-civil algorithm, proleptic Gregorian).
pub fn civil_date(ts: Timestamp) -> (i64, u32, u32) {
    let z = (ts + IST_OFFSET_MS).div_euclid(DAY_MS) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// 2012-01-02 15:30 IST, the first NSE session of 2012.
    const JAN2_2012: Timestamp = 1_325_498_400_000;

    #[test]
    fn civil_date_reads_ist_calendar_days() {
        assert_eq!(civil_date(JAN2_2012), (2012, 1, 2));
        assert_eq!(civil_date(0), (1970, 1, 1));
        assert_eq!(civil_date(JAN2_2012 + 59 * DAY_MS), (2012, 3, 1), "2012 is a leap year");
        assert_eq!(month_key(JAN2_2012 + 365 * DAY_MS), (2013, 1));
    }

    #[test]
    fn cluster_t_matches_a_hand_calculation() {
        // Values 1,3 (month A) and 2,6 (month B): mean 3; cluster sums -2 and +2; G = 2.
        // se = sqrt((4 + 4) * 2) / 4 = 1, so t = 3.
        let t = cluster_t(&[1.0, 3.0, 2.0, 6.0], &["A", "A", "B", "B"]);
        assert_eq!(t, Some((3.0, 2)));
        assert_eq!(cluster_t(&[1.0, 2.0], &["A", "A"]), None, "one cluster");
    }

    #[test]
    fn singleton_clusters_reduce_to_a_scaled_naive_t() {
        // With one value per cluster, se^2 = sum (x - m)^2 * G/(G-1) / n^2 = s^2 / n.
        let xs = [1.0, 2.0, 4.0, 7.0];
        let (t, g) = cluster_t(&xs, &[1, 2, 3, 4]).unwrap_or((0.0, 0));
        let (m, sd) = mean_sd(&xs).unwrap_or((0.0, 1.0));
        assert_eq!(g, 4);
        assert!((t - m / (sd / 2.0)).abs() < 1e-12);
    }

    fn curve(points: &[Decimal]) -> Vec<(Timestamp, Decimal)> {
        points.iter().enumerate().map(|(i, e)| (JAN2_2012 + i as i64 * DAY_MS, *e)).collect()
    }

    #[test]
    fn stats_cover_growth_drawdown_and_calmar() {
        // 365 calendar days of data doubling the account: 1 year at 365.25 days -> CAGR ~100%.
        let mut pts = vec![dec!(100); 364];
        pts.push(dec!(200));
        pts[100] = dec!(80);
        let s = stats(dec!(100), &curve(&pts));
        assert!((s.years - 365.0 / 365.25).abs() < 1e-12);
        assert!((s.total_return_pct - 100.0).abs() < 1e-9);
        assert!((s.cagr_pct.unwrap_or(0.0) - (2.0_f64.powf(365.25 / 365.0) - 1.0) * 100.0).abs() < 1e-9);
        assert!((s.max_drawdown_pct - 20.0).abs() < 1e-12);
        assert!((s.calmar.unwrap_or(0.0) - s.cagr_pct.unwrap_or(0.0) / 20.0).abs() < 1e-12);
        assert_eq!(s.months, 12);
    }

    #[test]
    fn stats_of_an_empty_curve_are_default() {
        assert_eq!(stats(dec!(100), &[]), ReturnStats::default());
    }

    #[test]
    fn exposure_is_mean_invested_share_of_equity() {
        let c = curve(&[dec!(100), dec!(200)]);
        assert_eq!(exposure_pct(&c, &[dec!(50), dec!(50)]), Some(37.5));
        assert_eq!(exposure_pct(&[], &[]), None);
    }

    #[test]
    fn yearly_returns_chain_year_end_equity() {
        let c = vec![(JAN2_2012, dec!(110)), (JAN2_2012 + 400 * DAY_MS, dec!(121))];
        let y = yearly_returns(dec!(100), &c);
        assert_eq!(y.iter().map(|p| p.0).collect::<Vec<_>>(), vec![2012, 2013]);
        assert!((y[0].1 - 10.0).abs() < 1e-9 && (y[1].1 - 10.0).abs() < 1e-9);
    }

    #[test]
    fn excess_returns_use_common_dates_only() {
        let a = [(1, 0.02), (2, 0.01), (3, 0.0)];
        let b = [(2, 0.03), (3, -0.01)];
        assert_eq!(excess_returns(&a, &b), vec![(2, 0.01 - 0.03), (3, 0.01)]);
    }

    #[test]
    fn daily_returns_start_from_initial_equity() {
        let r = daily_returns(dec!(100), &curve(&[dec!(110), dec!(99)]));
        assert!((r[0].1 - 0.1).abs() < 1e-12 && (r[1].1 + 0.1).abs() < 1e-12);
    }
}

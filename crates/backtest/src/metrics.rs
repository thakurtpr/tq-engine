//! Summary statistics. Everything is `Decimal` except Sharpe, which needs a square root
//! and is the only figure computed in `f64`.

use rust_decimal::Decimal;
use tq_core::Timestamp;

use crate::report::Trade;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Metrics {
    pub total_return_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub trades: usize,
    /// Share of closed trades with positive net P&L, in percent.
    pub win_rate_pct: Decimal,
    /// Sum of net wins over the absolute sum of net losses; `None` when there are no losses.
    pub profit_factor: Option<Decimal>,
    /// Mean net P&L of winning trades (zero if none).
    pub avg_win: Decimal,
    /// Mean net P&L of losing trades, as a negative number (zero if none).
    pub avg_loss: Decimal,
    pub total_costs: Decimal,
    /// Annualised (sqrt 252) Sharpe of per-bar returns, risk-free rate 0. Uses `f64` only here.
    /// `None` with fewer than two returns or zero volatility.
    pub sharpe: Option<f64>,
}

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;
const TRADING_DAYS_PER_YEAR: f64 = 252.0;

/// `initial` must be positive (the engine validates it). `curve` is close equity per bar.
pub fn compute(initial: Decimal, curve: &[(Timestamp, Decimal)], trades: &[Trade], total_costs: Decimal) -> Metrics {
    let equity: Vec<Decimal> = std::iter::once(initial).chain(curve.iter().map(|p| p.1)).collect();
    let last = equity.last().copied().unwrap_or(initial);
    let total_return_pct = if initial.is_zero() { Decimal::ZERO } else { (last - initial) / initial * HUNDRED };
    Metrics {
        total_return_pct,
        max_drawdown_pct: max_drawdown_pct(&equity),
        total_costs,
        sharpe: sharpe(&equity),
        ..trade_stats(trades)
    }
}

fn max_drawdown_pct(equity: &[Decimal]) -> Decimal {
    let mut peak = Decimal::ZERO;
    let mut worst = Decimal::ZERO;
    for &e in equity {
        peak = peak.max(e);
        if peak > Decimal::ZERO {
            worst = worst.max((peak - e) / peak * HUNDRED);
        }
    }
    worst
}

fn mean(values: &[Decimal]) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }
    values.iter().sum::<Decimal>() / Decimal::from(values.len())
}

fn trade_stats(trades: &[Trade]) -> Metrics {
    let wins: Vec<Decimal> = trades.iter().map(|t| t.net_pnl).filter(|p| *p > Decimal::ZERO).collect();
    let losses: Vec<Decimal> = trades.iter().map(|t| t.net_pnl).filter(|p| *p < Decimal::ZERO).collect();
    let gross_win: Decimal = wins.iter().sum();
    let gross_loss: Decimal = losses.iter().sum::<Decimal>().abs();
    let win_rate_pct = if trades.is_empty() {
        Decimal::ZERO
    } else {
        Decimal::from(wins.len()) / Decimal::from(trades.len()) * HUNDRED
    };
    Metrics {
        trades: trades.len(),
        win_rate_pct,
        profit_factor: (!gross_loss.is_zero()).then(|| gross_win / gross_loss),
        avg_win: mean(&wins),
        avg_loss: mean(&losses),
        ..Metrics::default()
    }
}

/// Annualised Sharpe of per-bar (daily) returns. The one place `f64` is used: it needs `sqrt`.
fn sharpe(equity: &[Decimal]) -> Option<f64> {
    use rust_decimal::prelude::ToPrimitive;
    let returns: Vec<f64> = equity
        .windows(2)
        .map(|w| if w[0].is_zero() { None } else { (w[1] / w[0] - Decimal::ONE).to_f64() })
        .collect::<Option<_>>()?;
    if returns.len() < 2 {
        return None;
    }
    let n = returns.len() as f64;
    let avg = returns.iter().sum::<f64>() / n;
    let var = returns.iter().map(|r| (r - avg).powi(2)).sum::<f64>() / (n - 1.0);
    let sd = var.sqrt();
    (sd > 0.0).then(|| avg / sd * TRADING_DAYS_PER_YEAR.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ExitReason;
    use rust_decimal_macros::dec;
    use tq_core::Symbol;

    fn trade(net: Decimal) -> Trade {
        Trade {
            symbol: Symbol("X".into()),
            entry_ts: 0,
            entry_price: dec!(100),
            stop: dec!(90),
            exit_ts: 1,
            exit_price: dec!(100),
            qty: dec!(1),
            gross_pnl: net,
            costs: dec!(0),
            net_pnl: net,
            exit_reason: ExitReason::Signal,
        }
    }

    fn curve(points: &[Decimal]) -> Vec<(Timestamp, Decimal)> {
        points.iter().enumerate().map(|(i, e)| (i as Timestamp, *e)).collect()
    }

    #[test]
    fn total_return_and_max_drawdown_from_curve() {
        let c = curve(&[dec!(110), dec!(88), dec!(120)]);
        let m = compute(dec!(100), &c, &[], dec!(5));
        assert_eq!(m.total_return_pct, dec!(20));
        assert_eq!(m.max_drawdown_pct, dec!(20)); // 110 -> 88
        assert_eq!(m.total_costs, dec!(5));
    }

    #[test]
    fn trade_statistics() {
        let t = [trade(dec!(300)), trade(dec!(100)), trade(dec!(-100)), trade(dec!(-300))];
        let m = compute(dec!(100), &[], &t, dec!(0));
        assert_eq!(m.trades, 4);
        assert_eq!(m.win_rate_pct, dec!(50));
        assert_eq!(m.profit_factor, Some(dec!(1)));
        assert_eq!((m.avg_win, m.avg_loss), (dec!(200), dec!(-200)));
    }

    #[test]
    fn profit_factor_undefined_without_losses() {
        let m = compute(dec!(100), &[], &[trade(dec!(10))], dec!(0));
        assert_eq!(m.profit_factor, None);
        assert_eq!(m.avg_loss, dec!(0));
    }

    #[test]
    fn sharpe_annualises_daily_returns() {
        // +10% then -10%: mean 0 -> Sharpe 0. +1% then +3%: mean 2%, sample sd sqrt(0.0002).
        let flat = compute(dec!(100), &curve(&[dec!(110), dec!(99)]), &[], dec!(0));
        assert_eq!(flat.sharpe, Some(0.0));
        let up = compute(dec!(100), &curve(&[dec!(101), dec!(104.03)]), &[], dec!(0));
        let expected = 0.02 / 0.0002_f64.sqrt() * 252.0_f64.sqrt();
        assert!((up.sharpe.unwrap_or(0.0) - expected).abs() < 1e-9);
    }

    #[test]
    fn sharpe_undefined_for_short_or_flat_series() {
        assert_eq!(compute(dec!(100), &curve(&[dec!(101)]), &[], dec!(0)).sharpe, None);
        assert_eq!(compute(dec!(100), &curve(&[dec!(100), dec!(100)]), &[], dec!(0)).sharpe, None);
    }
}

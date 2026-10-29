//! Pooled trade statistics across independently backtested symbols.
//! Money stays `Decimal`; only the t-statistic and Sharpe medians use `f64` (square roots).

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use tq_backtest::{RunReport, Trade};

const HUNDRED: Decimal = Decimal::ONE_HUNDRED;
/// Protocol section 3: net P&L must survive dropping this many best trades.
pub const TOP_TRADES_DROPPED: usize = 5;

/// One symbol's run, or why it was not run.
#[derive(Debug)]
pub struct SymbolResult {
    pub symbol: String,
    pub outcome: Result<RunReport, String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Pooled {
    pub symbols_run: usize,
    pub symbols_with_trades: usize,
    pub kill_switches: usize,
    pub trades: usize,
    pub win_rate_pct: Decimal,
    pub mean_net_trade_pct: Decimal,
    pub mean_r: Option<Decimal>,
    pub profit_factor: Option<Decimal>,
    /// Mean / (sample sd / sqrt n) of per-trade net returns in percent.
    pub t_stat: Option<f64>,
    pub gross_pnl: Decimal,
    pub costs: Decimal,
    pub net_pnl: Decimal,
    /// Closed-trade costs over gross P&L, in percent; `None` unless gross P&L is positive.
    pub costs_pct_of_gross: Option<Decimal>,
    pub median_sharpe: Option<f64>,
    pub median_max_dd_pct: Option<Decimal>,
    /// Largest single-symbol share of total net P&L (percent); `None` unless net P&L is positive.
    pub top_symbol_share: Option<(String, Decimal)>,
    pub net_pnl_without_top_trades: Decimal,
}

pub fn pool(results: &[SymbolResult]) -> Pooled {
    let reports: Vec<(&str, &RunReport)> =
        results.iter().filter_map(|r| r.outcome.as_ref().ok().map(|rep| (r.symbol.as_str(), rep))).collect();
    let trades: Vec<&Trade> = reports.iter().flat_map(|(_, r)| r.trades.iter()).collect();
    let net_pnl: Decimal = trades.iter().map(|t| t.net_pnl).sum();
    let sharpes: Vec<f64> = reports.iter().filter_map(|(_, r)| r.metrics.sharpe).collect();
    let dds: Vec<Decimal> = reports.iter().filter(|(_, r)| r.bars > 0).map(|(_, r)| r.metrics.max_drawdown_pct).collect();
    Pooled {
        symbols_run: reports.len(),
        symbols_with_trades: reports.iter().filter(|(_, r)| !r.trades.is_empty()).count(),
        kill_switches: reports.iter().filter(|(_, r)| r.kill_switch_engaged_at.is_some()).count(),
        median_sharpe: median_f64(sharpes),
        median_max_dd_pct: median_decimal(dds),
        top_symbol_share: top_symbol_share(&reports, net_pnl),
        net_pnl_without_top_trades: net_without_top(&trades, TOP_TRADES_DROPPED),
        ..trade_stats(&trades)
    }
}

fn trade_stats(trades: &[&Trade]) -> Pooled {
    let n = Decimal::from(trades.len().max(1));
    let rets: Vec<Decimal> = trades.iter().map(|t| t.net_return_pct()).collect();
    let rs: Vec<Decimal> = trades.iter().filter_map(|t| t.r_multiple()).collect();
    let wins: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p > Decimal::ZERO).sum();
    let losses: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p < Decimal::ZERO).sum::<Decimal>().abs();
    let gross_pnl: Decimal = trades.iter().map(|t| t.gross_pnl).sum();
    let costs: Decimal = trades.iter().map(|t| t.costs).sum();
    let win_count = trades.iter().filter(|t| t.net_pnl > Decimal::ZERO).count();
    Pooled {
        trades: trades.len(),
        win_rate_pct: Decimal::from(win_count) / n * HUNDRED,
        mean_net_trade_pct: rets.iter().sum::<Decimal>() / n,
        mean_r: (!rs.is_empty()).then(|| rs.iter().sum::<Decimal>() / Decimal::from(rs.len())),
        profit_factor: (!losses.is_zero()).then(|| wins / losses),
        t_stat: t_stat(&rets),
        gross_pnl,
        costs,
        net_pnl: gross_pnl - costs,
        costs_pct_of_gross: (gross_pnl > Decimal::ZERO).then(|| costs / gross_pnl * HUNDRED),
        ..Pooled::default()
    }
}

/// One-sample t-statistic of the mean against zero. `None` below two values or with zero spread.
pub fn t_stat(values: &[Decimal]) -> Option<f64> {
    let xs: Vec<f64> = values.iter().map(|v| v.to_f64()).collect::<Option<_>>()?;
    if xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (var > 0.0).then(|| mean / (var.sqrt() / n.sqrt()))
}

pub fn median_f64(mut xs: Vec<f64>) -> Option<f64> {
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(xs[n / 2]),
        _ => Some((xs[n / 2 - 1] + xs[n / 2]) / 2.0),
    }
}

pub fn median_decimal(mut xs: Vec<Decimal>) -> Option<Decimal> {
    xs.sort();
    let n = xs.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(xs[n / 2]),
        _ => Some((xs[n / 2 - 1] + xs[n / 2]) / Decimal::TWO),
    }
}

fn top_symbol_share(reports: &[(&str, &RunReport)], total_net: Decimal) -> Option<(String, Decimal)> {
    if total_net <= Decimal::ZERO {
        return None;
    }
    reports
        .iter()
        .map(|(s, r)| (s.to_string(), r.trades.iter().map(|t| t.net_pnl).sum::<Decimal>()))
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|(s, net)| (s, net / total_net * HUNDRED))
}

fn net_without_top(trades: &[&Trade], drop: usize) -> Decimal {
    let mut nets: Vec<Decimal> = trades.iter().map(|t| t.net_pnl).collect();
    nets.sort_by(|a, b| b.cmp(a));
    nets.iter().skip(drop).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tq_backtest::ExitReason;
    use tq_core::Symbol;

    /// 10 shares at 100, stop 90 (1R = 100); `gross` before `costs`.
    fn trade(gross: Decimal, costs: Decimal) -> Trade {
        Trade {
            symbol: Symbol("X".into()),
            entry_ts: 0,
            entry_price: dec!(100),
            stop: dec!(90),
            exit_ts: 1,
            exit_price: dec!(100),
            qty: dec!(10),
            gross_pnl: gross,
            costs,
            net_pnl: gross - costs,
            exit_reason: ExitReason::Signal,
        }
    }

    fn result(symbol: &str, trades: Vec<Trade>) -> SymbolResult {
        SymbolResult { symbol: symbol.into(), outcome: Ok(RunReport { trades, bars: 1, ..RunReport::default() }) }
    }

    #[test]
    fn pools_trades_across_symbols() {
        let rs = [
            result("A", vec![trade(dec!(210), dec!(10)), trade(dec!(-90), dec!(10))]),
            result("B", vec![trade(dec!(60), dec!(10))]),
            SymbolResult { symbol: "C".into(), outcome: Err("no data".into()) },
        ];
        let p = pool(&rs);
        assert_eq!((p.symbols_run, p.symbols_with_trades, p.trades), (2, 2, 3));
        assert_eq!((p.gross_pnl, p.costs, p.net_pnl), (dec!(180), dec!(30), dec!(150)));
        // net returns 20%, -10%, 5% of the 1000 notional; R = 2, -1, 0.5.
        assert_eq!((p.mean_net_trade_pct, p.mean_r), (dec!(5), Some(dec!(0.5))));
        assert_eq!(p.profit_factor, Some(dec!(2.5)));
        assert_eq!(p.win_rate_pct, dec!(2) / dec!(3) * dec!(100));
        assert_eq!(p.costs_pct_of_gross, Some(dec!(30) / dec!(180) * dec!(100)));
        assert_eq!(p.top_symbol_share, Some(("A".to_string(), dec!(100) / dec!(150) * dec!(100))));
        assert_eq!(p.net_pnl_without_top_trades, dec!(0));
    }

    #[test]
    fn t_stat_matches_hand_calculation() {
        // mean 2, sample sd 1, n 3 -> 2 / (1 / sqrt 3)
        let t = t_stat(&[dec!(1), dec!(2), dec!(3)]).unwrap_or(0.0);
        assert!((t - 2.0 * 3.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(t_stat(&[dec!(1)]), None);
        assert_eq!(t_stat(&[dec!(1), dec!(1)]), None);
    }

    #[test]
    fn medians_handle_odd_even_and_empty() {
        assert_eq!(median_f64(vec![3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median_f64(vec![]), None);
        assert_eq!(median_decimal(vec![dec!(4), dec!(1), dec!(2), dec!(3)]), Some(dec!(2.5)));
    }

    #[test]
    fn shares_and_cost_ratio_undefined_when_losing() {
        let p = pool(&[result("A", vec![trade(dec!(-50), dec!(10))])]);
        assert_eq!((p.costs_pct_of_gross, p.top_symbol_share, p.profit_factor), (None, None, Some(dec!(0))));
    }
}

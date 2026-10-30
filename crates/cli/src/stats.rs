//! Closed-trade statistics, for one portfolio account or pooled across per-symbol accounts.
//! Money stays `Decimal`; t-statistics and Sharpe medians use `f64` (square roots).

use std::collections::BTreeMap;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use tq_backtest::returns::{cluster_t, month_key};
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
pub struct TradeStats {
    pub trades: usize,
    pub symbols_traded: usize,
    pub win_rate_pct: Decimal,
    pub mean_net_trade_pct: Decimal,
    pub mean_r: Option<Decimal>,
    pub profit_factor: Option<Decimal>,
    /// Naive mean / (sd / sqrt n) of per-trade net returns in percent (assumes independence).
    pub t_stat: Option<f64>,
    /// Same mean, standard error clustered by entry month (review `tstat.py`, CR1); `(t, months)`.
    pub t_month_clustered: Option<(f64, usize)>,
    /// P&L at filled prices (after slippage, before costs).
    pub gross_pnl: Decimal,
    pub costs: Decimal,
    pub slippage: Decimal,
    pub net_pnl: Decimal,
    /// (costs + slippage) / (gross + slippage), percent; `None` unless the pre-friction move is positive.
    pub friction_pct_of_gross: Option<Decimal>,
    /// Largest single-symbol share of total net P&L (percent); `None` unless net P&L is positive.
    pub top_symbol_share: Option<(String, Decimal)>,
    pub net_pnl_without_top_trades: Decimal,
}

pub fn trade_stats(trades: &[&Trade]) -> TradeStats {
    let n = Decimal::from(trades.len().max(1));
    let rets: Vec<Decimal> = trades.iter().map(|t| t.net_return_pct()).collect();
    let rs: Vec<Decimal> = trades.iter().filter_map(|t| t.r_multiple()).collect();
    let wins: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p > Decimal::ZERO).sum();
    let losses: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p < Decimal::ZERO).sum::<Decimal>().abs();
    let sum = |f: fn(&Trade) -> Decimal| trades.iter().map(|t| f(t)).sum::<Decimal>();
    let (gross_pnl, costs, slippage) = (sum(|t| t.gross_pnl), sum(|t| t.costs), sum(|t| t.slippage));
    let net_pnl = gross_pnl - costs;
    let pre_friction = gross_pnl + slippage;
    TradeStats {
        trades: trades.len(),
        symbols_traded: trades.iter().map(|t| &t.symbol.0).collect::<std::collections::BTreeSet<_>>().len(),
        win_rate_pct: Decimal::from(trades.iter().filter(|t| t.net_pnl > Decimal::ZERO).count()) / n * HUNDRED,
        mean_net_trade_pct: rets.iter().sum::<Decimal>() / n,
        mean_r: (!rs.is_empty()).then(|| rs.iter().sum::<Decimal>() / Decimal::from(rs.len())),
        profit_factor: (!losses.is_zero()).then(|| wins / losses),
        t_stat: t_stat(&rets),
        t_month_clustered: month_clustered_t(trades, &rets),
        gross_pnl,
        costs,
        slippage,
        net_pnl,
        friction_pct_of_gross: (pre_friction > Decimal::ZERO).then(|| (costs + slippage) / pre_friction * HUNDRED),
        top_symbol_share: top_symbol_share(trades, net_pnl),
        net_pnl_without_top_trades: net_without_top(trades, TOP_TRADES_DROPPED),
    }
}

fn month_clustered_t(trades: &[&Trade], rets: &[Decimal]) -> Option<(f64, usize)> {
    let xs: Vec<f64> = rets.iter().map(|r| r.to_f64()).collect::<Option<_>>()?;
    let keys: Vec<(i64, u32)> = trades.iter().map(|t| month_key(t.entry_ts)).collect();
    cluster_t(&xs, &keys)
}

/// Independent per-symbol accounts, pooled. A DIAGNOSTIC: the accounts share no cash or limits.
#[derive(Debug, Default, PartialEq)]
pub struct Pooled {
    pub symbols_run: usize,
    pub symbols_with_trades: usize,
    pub kill_switches: usize,
    pub median_sharpe: Option<f64>,
    pub median_max_dd_pct: Option<Decimal>,
    pub trades: TradeStats,
}

pub fn pool(results: &[SymbolResult]) -> Pooled {
    let reports: Vec<&RunReport> = results.iter().filter_map(|r| r.outcome.as_ref().ok()).collect();
    let trades: Vec<&Trade> = reports.iter().flat_map(|r| r.trades.iter()).collect();
    Pooled {
        symbols_run: reports.len(),
        symbols_with_trades: reports.iter().filter(|r| !r.trades.is_empty()).count(),
        kill_switches: reports.iter().filter(|r| r.kill_switch_engaged_at.is_some()).count(),
        median_sharpe: median_f64(reports.iter().filter_map(|r| r.metrics.sharpe).collect()),
        median_max_dd_pct: median_decimal(reports.iter().filter(|r| r.bars > 0).map(|r| r.metrics.max_drawdown_pct).collect()),
        trades: trade_stats(&trades),
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

fn top_symbol_share(trades: &[&Trade], total_net: Decimal) -> Option<(String, Decimal)> {
    if total_net <= Decimal::ZERO {
        return None;
    }
    let mut by_symbol: BTreeMap<&str, Decimal> = BTreeMap::new();
    for t in trades {
        *by_symbol.entry(t.symbol.0.as_str()).or_default() += t.net_pnl;
    }
    by_symbol.into_iter().max_by(|a, b| a.1.cmp(&b.1)).map(|(s, net)| (s.to_string(), net / total_net * HUNDRED))
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

    /// 10 shares at 100, stop 90 (1R = 100); `gross` before `costs`; entered on day `day`.
    fn trade(symbol: &str, gross: Decimal, costs: Decimal, day: i64) -> Trade {
        Trade {
            symbol: Symbol(symbol.into()),
            entry_ts: 1_325_498_400_000 + day * 86_400_000,
            entry_price: dec!(100),
            stop: dec!(90),
            exit_ts: 1,
            exit_price: dec!(100),
            qty: dec!(10),
            gross_pnl: gross,
            costs,
            net_pnl: gross - costs,
            exit_reason: ExitReason::Signal,
            slippage: dec!(2),
        }
    }

    fn result(symbol: &str, trades: Vec<Trade>) -> SymbolResult {
        SymbolResult { symbol: symbol.into(), outcome: Ok(RunReport { trades, bars: 1, ..RunReport::default() }) }
    }

    #[test]
    fn pools_trades_across_symbols() {
        let rs = [
            result("A", vec![trade("A", dec!(210), dec!(10), 0), trade("A", dec!(-90), dec!(10), 40)]),
            result("B", vec![trade("B", dec!(60), dec!(10), 80)]),
            SymbolResult { symbol: "C".into(), outcome: Err("no data".into()) },
        ];
        let p = pool(&rs);
        assert_eq!((p.symbols_run, p.symbols_with_trades, p.trades.trades), (2, 2, 3));
        let t = &p.trades;
        assert_eq!((t.gross_pnl, t.costs, t.slippage, t.net_pnl), (dec!(180), dec!(30), dec!(6), dec!(150)));
        // net returns 20%, -10%, 5% of the 1000 notional; R = 2, -1, 0.5.
        assert_eq!((t.mean_net_trade_pct, t.mean_r), (dec!(5), Some(dec!(0.5))));
        assert_eq!(t.profit_factor, Some(dec!(2.5)));
        assert_eq!(t.win_rate_pct, dec!(2) / dec!(3) * dec!(100));
        assert_eq!(t.friction_pct_of_gross, Some(dec!(36) / dec!(186) * dec!(100)));
        assert_eq!(t.top_symbol_share, Some(("A".to_string(), dec!(100) / dec!(150) * dec!(100))));
        assert_eq!(t.net_pnl_without_top_trades, dec!(0));
        assert_eq!(t.t_month_clustered.map(|c| c.1), Some(3), "Jan, Feb and Mar 2012");
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
    fn same_month_trades_form_one_cluster() {
        let ts = [trade("A", dec!(100), dec!(0), 0), trade("B", dec!(50), dec!(0), 3), trade("C", dec!(-20), dec!(0), 40)];
        let refs: Vec<&Trade> = ts.iter().collect();
        assert_eq!(trade_stats(&refs).t_month_clustered.map(|c| c.1), Some(2));
    }

    #[test]
    fn medians_handle_odd_even_and_empty() {
        assert_eq!(median_f64(vec![3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median_f64(vec![]), None);
        assert_eq!(median_decimal(vec![dec!(4), dec!(1), dec!(2), dec!(3)]), Some(dec!(2.5)));
    }

    #[test]
    fn shares_and_cost_ratio_undefined_when_losing() {
        let p = pool(&[result("A", vec![trade("A", dec!(-50), dec!(10), 0)])]);
        let t = &p.trades;
        assert_eq!((t.friction_pct_of_gross, t.top_symbol_share.clone(), t.profit_factor), (None, None, Some(dec!(0))));
    }
}

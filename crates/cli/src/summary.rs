//! One CSV line per portfolio run (`--summary-csv`), for stress and seed tables.

use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

use crate::args::BacktestArgs;
use crate::backtest::Setup;
use crate::benchmark::Benchmark;
use crate::report_portfolio::PortfolioView;
use crate::CliError;

const HEADER: &str = "label,strategy,max_positions,kill_switch_pct,slippage_model,slippage_mult,priority,trades,\
cagr_pct,max_dd_pct,sharpe,t_month_clustered,calmar,exposure_pct,final_equity,bench_cagr_pct,bench_sharpe,\
bench_max_dd_pct,excess_t,hac_t,hac_lag,monthly_t,trade_clustered_t,gate_t,excess_hac_t,fills_above_impact,\
entries_adv_capped,turnover,total_costs,total_slippage,initial_cash,manifest_sha256";

fn f(x: Option<f64>) -> String {
    x.map_or_else(String::new, |v| format!("{v:.4}"))
}

pub fn line(args: &BacktestArgs, setup: &Setup, v: &PortfolioView, bench: Option<&Benchmark>, manifest_sha: &str) -> String {
    let l = setup.gate.limits();
    let r = &v.returns;
    let b = bench.map(|b| &b.stats);
    let priority = format!("{:?}", setup.config.priority).replace([',', ' '], "");
    let mut s = String::new();
    let _ = write!(
        s,
        "{},{:?},{},{},{:?},{},{priority},{},{},{:.4},{},{},{},{},{},{},{},{},{},{},{manifest_sha}",
        args.label.replace(',', ";"),
        args.strategy,
        l.max_open_positions,
        l.max_drawdown_pct,
        args.slippage_model,
        args.slippage_mult,
        v.trades.trades,
        f(r.cagr_pct),
        r.max_drawdown_pct,
        f(r.sharpe),
        f(r.t_month_clustered),
        f(r.calmar),
        f(v.exposure_pct),
        v.report.equity_curve.last().map_or_else(String::new, |p| p.1.round_dp(2).to_string()),
        f(b.and_then(|b| b.cagr_pct)),
        f(b.and_then(|b| b.sharpe)),
        f(b.map(|b| b.max_drawdown_pct)),
        f(v.excess_t.map(|e| e.0)),
        extra(v, args),
    );
    s
}

/// Protocol v2 gate-4c figures, liquidity counters and money totals.
fn extra(v: &PortfolioView, args: &BacktestArgs) -> String {
    let (g, rep) = (&v.gate_t, v.report);
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{}",
        f(g.hac),
        g.lag,
        f(g.monthly),
        f(g.per_trade_clustered),
        f(g.gate),
        f(v.excess_hac_t),
        rep.fills_above_impact_threshold,
        rep.entries_adv_capped,
        rep.turnover.round_dp(2),
        rep.metrics.total_costs.round_dp(2),
        rep.total_slippage.round_dp(2),
        args.cash,
    )
}

/// Appends `line`, writing the header first if the file is new or empty.
pub fn append(path: &Path, line: &str) -> Result<(), CliError> {
    let err = |e: std::io::Error| CliError::Write { path: path.display().to_string(), msg: e.to_string() };
    let is_new = std::fs::metadata(path).map_or(true, |m| m.len() == 0);
    let mut file = OpenOptions::new().create(true).append(true).open(path).map_err(err)?;
    if is_new {
        writeln!(file, "{HEADER}").map_err(err)?;
    }
    writeln!(file, "{line}").map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_once_then_lines() {
        let path = std::env::temp_dir().join(format!("tq-summary-{}.csv", std::process::id()));
        let _ = std::fs::remove_file(&path);
        append(&path, "a").unwrap();
        append(&path, "b").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(text, format!("{HEADER}\na\nb\n"));
        assert_eq!(HEADER.split(',').count(), 32);
    }
}

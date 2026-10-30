//! Markdown for a shared-account portfolio run: the headline figures, the benchmark, trade
//! statistics, the entry funnel, yearly returns and the gate table.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use rust_decimal::Decimal;
use tq_backtest::returns::{civil_date, cluster_t, daily_returns, excess_returns, exposure_pct, month_key, stats, ReturnStats};
use tq_backtest::tstats::{gate_t, hac_t, longest_hold, protocol_lag, GateT};
use tq_backtest::{RunReport, Trade};
use tq_core::Timestamp;
use tq_data::ist_date;

use crate::args::BacktestArgs;
use crate::backtest::Setup;
use crate::benchmark::Benchmark;
use crate::gates::portfolio_rows;
use crate::markdown::{dec, opt_f64, trade_rows, write_coverage, write_header};
use crate::report_liquidity::{self, Friction};
use crate::stats::{trade_stats, TradeStats};
use crate::universe::Coverage;

/// Everything the report and the summary CSV need from one portfolio run.
pub struct PortfolioView<'a> {
    pub report: &'a RunReport,
    pub returns: ReturnStats,
    pub trades: TradeStats,
    pub exposure_pct: Option<f64>,
    /// Month-clustered t of strategy minus benchmark daily returns, with the month count.
    pub excess_t: Option<(f64, usize)>,
    /// HAC t (protocol bandwidth) of strategy minus benchmark daily returns.
    pub excess_hac_t: Option<f64>,
    /// Gate 4c figures (protocol v2): HAC primary, monthly and per-trade cross-checks.
    pub gate_t: GateT,
    pub friction: Friction,
}

pub fn view<'a>(report: &'a RunReport, cash: Decimal, bench: Option<&Benchmark>) -> PortfolioView<'a> {
    let trades: Vec<&Trade> = report.trades.iter().collect();
    let daily = daily_returns(cash, &report.equity_curve);
    let hold = longest_hold(&report.equity_curve, &report.trades);
    let excess = bench.map(|b| excess_returns(&daily, &b.returns));
    let excess_t = excess.as_ref().and_then(|ex| {
        let keys: Vec<(i64, u32)> = ex.iter().map(|e| month_key(e.0)).collect();
        cluster_t(&ex.iter().map(|e| e.1).collect::<Vec<_>>(), &keys)
    });
    let excess_hac_t = excess.as_ref().and_then(|ex| hac_t(&ex.iter().map(|e| e.1).collect::<Vec<_>>(), protocol_lag(ex.len(), hold)));
    let trade_stats = trade_stats(&trades);
    let gate_t = gate_t(&daily, hold, trade_stats.t_month_clustered.map(|c| c.0));
    let returns = stats(cash, &report.equity_curve);
    PortfolioView {
        report,
        friction: report_liquidity::friction(report, cash, returns.years),
        returns,
        trades: trade_stats,
        exposure_pct: exposure_pct(&report.equity_curve, &report.exposure_curve),
        excess_t,
        excess_hac_t,
        gate_t,
    }
}

pub struct Context<'a> {
    pub args: &'a BacktestArgs,
    pub setup: &'a Setup,
    pub coverage: Option<&'a Coverage>,
    pub bench: Option<&'a Benchmark>,
    pub skipped: &'a [(String, String)],
    pub manifest: &'a str,
}

pub fn render(ctx: &Context, v: &PortfolioView) -> String {
    let mut out = String::new();
    // Writing to a String cannot fail; the results are ignored on purpose.
    let _ = write_header(&mut out, ctx.args, ctx.setup, ctx.manifest);
    let _ = write_headline(&mut out, v, ctx.bench);
    let _ = write_trades(&mut out, &v.trades);
    if !ctx.args.strategy.is_rebalancer() {
        let _ = write_funnel(&mut out, v.report);
    }
    let _ = report_liquidity::write(&mut out, v.report, &v.friction, ctx.args.strategy.is_rebalancer());
    let _ = write_years(&mut out, ctx.args.cash, v.report, ctx.bench);
    let _ = write_gates(&mut out, v, ctx.bench);
    if let Some(c) = ctx.coverage {
        let _ = write_coverage(&mut out, c);
    }
    if !ctx.skipped.is_empty() {
        let _ = writeln!(out, "## Not run (no usable data)\n");
        for (s, why) in ctx.skipped {
            let _ = writeln!(out, "- {s}: {why}");
        }
    }
    out
}

fn pct(x: Option<f64>) -> String {
    x.map_or_else(|| "n/a".into(), |v| format!("{v:.2}%"))
}

fn write_headline(out: &mut String, v: &PortfolioView, bench: Option<&Benchmark>) -> std::fmt::Result {
    let (r, rep) = (&v.returns, v.report);
    let b = bench.map(|b| &b.stats);
    let col = |f: &dyn Fn(&ReturnStats) -> String| b.map_or_else(|| "-".to_string(), f);
    let positions = &rep.open_positions_curve;
    let mean_pos = positions.iter().sum::<usize>() as f64 / positions.len().max(1) as f64;
    let kill = rep.kill_switch_engaged_at.and_then(ist_date).map_or_else(|| "never".to_string(), |d| d.to_string());
    writeln!(out, "## Portfolio result (one shared account)\n")?;
    writeln!(out, "| Metric | Strategy | {} buy-and-hold |\n| --- | ---: | ---: |", bench.map_or("Benchmark", |b| b.name.as_str()))?;
    writeln!(out, "| Total return | {} | {} |", pct(Some(r.total_return_pct)), col(&|s| pct(Some(s.total_return_pct))))?;
    writeln!(out, "| CAGR | {} | {} |", pct(r.cagr_pct), col(&|s| pct(s.cagr_pct)))?;
    writeln!(out, "| Sharpe (daily, annualised, rf 0) | {} | {} |", opt_f64(r.sharpe), col(&|s| opt_f64(s.sharpe)))?;
    let g = &v.gate_t;
    writeln!(out, "| t, HAC Newey-West daily returns (gate 4c primary, L = {}) | {} | - |", g.lag, opt_f64(g.hac))?;
    writeln!(out, "| t, monthly returns (4c cross-check) | {} | - |", opt_f64(g.monthly))?;
    writeln!(out, "| t, per-trade, entry-month clustered (4c cross-check) | {} | - |", opt_f64(g.per_trade_clustered))?;
    writeln!(out, "| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **{}** | - |", opt_f64(g.gate))?;
    writeln!(out, "| t, month-clustered daily returns (backtest-03 figure) | {} (G = {}) | {} |", opt_f64(r.t_month_clustered), r.months, col(&|s| opt_f64(s.t_month_clustered)))?;
    writeln!(out, "| Max drawdown | {:.2}% | {} |", r.max_drawdown_pct, col(&|s| format!("{:.2}%", s.max_drawdown_pct)))?;
    writeln!(out, "| Calmar (CAGR / max DD) | {} | {} |", opt_f64(r.calmar), col(&|s| opt_f64(s.calmar)))?;
    writeln!(out, "| Exposure (mean invested / equity) | {} | 100% |", pct(v.exposure_pct))?;
    writeln!(out, "| Open positions: mean / max | {mean_pos:.2} / {} | - |", positions.iter().max().unwrap_or(&0))?;
    writeln!(out, "| Final equity (Rs) | {} | - |", rep.equity_curve.last().map_or_else(|| "n/a".into(), |p| dec(p.1)))?;
    writeln!(out, "| Kill switch engaged | {kill} | - |\n")?;
    if let Some((t, g)) = v.excess_t {
        writeln!(out, "Strategy minus benchmark daily return: HAC t = {} (L = {}); month-clustered t = {t:.2} (G = {g}).\n", opt_f64(v.excess_hac_t), v.gate_t.lag)?;
    }
    if let Some(b) = bench {
        writeln!(out, "{}\n", benchmark_note(&b.name))?;
    }
    Ok(())
}

fn benchmark_note(name: &str) -> String {
    if name == "NSEI" {
        "The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.".into()
    } else {
        format!("The benchmark is frictionless buy-and-hold of {name} on its dividend-adjusted closes (for NIFTYBEES: close to the NIFTY 50 TRI less the fund's expense ratio and tracking error).")
    }
}

fn write_trades(out: &mut String, t: &TradeStats) -> std::fmt::Result {
    writeln!(out, "## Closed trades (one account, so rupee sums are real)\n\n| Metric | Value |\n| --- | --- |")?;
    let mut rows = trade_rows(t);
    rows.push(("Gross P&L before slippage / slippage / costs / net (Rs)".into(), format!("{} / {} / {} / {}", dec(t.gross_pnl + t.slippage), dec(t.slippage), dec(t.costs), dec(t.net_pnl))));
    rows.push(("Net P&L without the best 5 trades (Rs)".into(), dec(t.net_pnl_without_top_trades)));
    for (k, val) in rows {
        writeln!(out, "| {k} | {val} |")?;
    }
    writeln!(out)
}

/// A rejection message without its symbol or numbers, for counting by reason.
fn reason(msg: &str) -> String {
    let cut = msg.find(|c: char| c.is_ascii_digit()).unwrap_or(msg.len());
    let head = msg[..cut].trim_end();
    head.strip_suffix(" for").map_or_else(|| head.split(" for ").next().unwrap_or(head).to_string(), str::to_string)
}

fn write_funnel(out: &mut String, r: &RunReport) -> std::fmt::Result {
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for m in &r.rejections {
        *reasons.entry(reason(m)).or_default() += 1;
    }
    writeln!(out, "## Entry funnel\n\n| Step | Count |\n| --- | ---: |")?;
    writeln!(out, "| Entry signals | {} |", r.orders_proposed)?;
    writeln!(out, "| Ignored: already held or pending | {} |", r.duplicate_entries_ignored)?;
    for (k, n) in &reasons {
        writeln!(out, "| Rejected: {k} | {n} |")?;
    }
    writeln!(out, "| Skipped: not an index member on the fill date | {} |", r.entries_outside_universe)?;
    writeln!(out, "| Skipped: sized to zero shares | {} |", r.entries_sized_to_zero)?;
    writeln!(out, "| Cut to the 1% ADV cap (still filled unless zero) | {} |", r.entries_adv_capped)?;
    writeln!(out, "| Filled orders (entries + exits) | {} |\n", r.orders_filled)
}

/// Calendar-year returns from compounded daily returns.
fn by_year(returns: &[(Timestamp, f64)]) -> BTreeMap<i64, f64> {
    let mut growth: BTreeMap<i64, f64> = BTreeMap::new();
    for &(ts, r) in returns {
        *growth.entry(civil_date(ts).0).or_insert(1.0) *= 1.0 + r;
    }
    growth.into_iter().map(|(y, g)| (y, (g - 1.0) * 100.0)).collect()
}

fn write_years(out: &mut String, cash: Decimal, r: &RunReport, bench: Option<&Benchmark>) -> std::fmt::Result {
    let ours = by_year(&daily_returns(cash, &r.equity_curve));
    let theirs = bench.map(|b| by_year(&b.returns)).unwrap_or_default();
    writeln!(out, "## Calendar-year returns\n\n| Year | Strategy | Benchmark |\n| --- | ---: | ---: |")?;
    for (y, ret) in &ours {
        writeln!(out, "| {y} | {ret:.2}% | {} |", theirs.get(y).map_or_else(|| "-".into(), |b| format!("{b:.2}%")))?;
    }
    writeln!(out)
}

fn write_gates(out: &mut String, v: &PortfolioView, bench: Option<&Benchmark>) -> std::fmt::Result {
    writeln!(out, "## Gates (docs/validation-protocol.md), in-sample, this run only\n")?;
    writeln!(out, "| # | Gate | Threshold | Value | Verdict |\n| --- | --- | --- | --- | --- |")?;
    for g in portfolio_rows(&v.trades, &v.returns, &v.gate_t, bench.map(|b| (b.name.as_str(), &b.stats))) {
        writeln!(out, "| {} | {} | {} | {} | {} |", g.id, g.name, g.threshold, g.value, g.verdict.label())?;
    }
    writeln!(out, "\nNot assessable from one run: cost stress (5b, needs the 2x run), walk-forward OOS (2, 4), robustness (5), DSR/PBO (6), Monte Carlo (7). A PASS here is in-sample only.\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejection_reasons_drop_symbols_and_numbers() {
        assert_eq!(reason("insufficient cash for INFY"), "insufficient cash");
        assert_eq!(reason("risk: max open positions reached"), "risk: max open positions reached");
        assert_eq!(reason("risk at fill: risk per trade 5100 exceeds limit 5000"), "risk at fill: risk per trade");
    }

    #[test]
    fn yearly_returns_compound_within_each_year() {
        let jan2_2012 = 1_325_498_400_000;
        let y = by_year(&[(jan2_2012, 0.1), (jan2_2012 + 86_400_000, 0.1), (jan2_2012 + 400 * 86_400_000, -0.5)]);
        assert!((y[&2012] - 21.0).abs() < 1e-9 && (y[&2013] + 50.0).abs() < 1e-9);
    }
}

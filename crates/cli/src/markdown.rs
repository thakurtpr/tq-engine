//! Markdown rendering of a multi-symbol backtest.

use std::fmt::Write as _;

use rust_decimal::Decimal;
use tq_backtest::RunReport;
use tq_data::ist_date;

use crate::args::BacktestArgs;
use crate::backtest::Setup;
use crate::stats::{Pooled, SymbolResult, TOP_TRADES_DROPPED};

const NA: &str = "n/a";

fn dec(d: Decimal) -> String {
    d.round_dp(2).to_string()
}

fn opt_dec(d: Option<Decimal>) -> String {
    d.map_or_else(|| NA.to_string(), dec)
}

fn opt_f64(x: Option<f64>) -> String {
    x.map_or_else(|| NA.to_string(), |v| format!("{v:.2}"))
}

pub fn render(args: &BacktestArgs, setup: &Setup, results: &[SymbolResult], pooled: &Pooled) -> String {
    let mut out = String::new();
    // Writing to a String cannot fail; the results are ignored on purpose.
    let _ = write_header(&mut out, args, setup);
    let _ = write_pooled(&mut out, pooled);
    let _ = write_symbols(&mut out, results);
    out
}

fn write_header(out: &mut String, args: &BacktestArgs, setup: &Setup) -> std::fmt::Result {
    let l = setup.gate.limits();
    writeln!(out, "# Backtest: {:?}\n", args.strategy)?;
    writeln!(out, "- Window: {} to {} (earlier bars warm up indicators only)", args.start, args.end)?;
    writeln!(out, "- Cash per symbol: Rs {}; risk per trade {}% of equity", args.cash, args.risk_pct)?;
    writeln!(out, "- Slippage: {} bps per side; costs: Zerodha NSE delivery", args.slippage_bps)?;
    writeln!(
        out,
        "- Risk gate: max risk {}%, notional cap {}%, daily loss {}%, drawdown kill switch {}%",
        l.max_risk_per_trade_pct, l.max_position_notional_pct, l.max_daily_loss_pct, l.max_drawdown_pct
    )?;
    writeln!(out, "- Each symbol runs independently with fresh cash; strategy parameters are defaults.\n")
}

fn write_pooled(out: &mut String, p: &Pooled) -> std::fmt::Result {
    let share = p.top_symbol_share.as_ref().map_or_else(|| NA.to_string(), |(s, v)| format!("{s} {}%", dec(*v)));
    writeln!(out, "## Pooled closed trades\n\n| Metric | Value |\n| --- | --- |")?;
    let rows = [
        ("Symbols run / with trades / kill switch hit", format!("{} / {} / {}", p.symbols_run, p.symbols_with_trades, p.kill_switches)),
        ("Trades", p.trades.to_string()),
        ("Win rate %", dec(p.win_rate_pct)),
        ("Mean net trade %", dec(p.mean_net_trade_pct)),
        ("Mean R (net)", opt_dec(p.mean_r)),
        ("Profit factor (net)", opt_dec(p.profit_factor)),
        ("t-stat of net trade returns", opt_f64(p.t_stat)),
        ("Gross / costs / net P&L (Rs)", format!("{} / {} / {}", dec(p.gross_pnl), dec(p.costs), dec(p.net_pnl))),
        ("Costs as % of gross P&L", opt_dec(p.costs_pct_of_gross)),
        ("Median per-symbol Sharpe (annualised)", opt_f64(p.median_sharpe)),
        ("Median per-symbol max drawdown %", opt_dec(p.median_max_dd_pct)),
        ("Largest symbol share of net P&L", share),
        (&*format!("Net P&L without the best {TOP_TRADES_DROPPED} trades (Rs)"), dec(p.net_pnl_without_top_trades)),
    ];
    for (k, v) in rows {
        writeln!(out, "| {k} | {v} |")?;
    }
    writeln!(out)
}

fn write_symbols(out: &mut String, results: &[SymbolResult]) -> std::fmt::Result {
    writeln!(out, "## Per symbol\n")?;
    writeln!(out, "| Symbol | Bars | Trades | Win % | PF | Net P&L (Rs) | Return % | Max DD % | Sharpe | Kill switch | Rejected | Size 0 |")?;
    writeln!(out, "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |")?;
    let mut skipped = Vec::new();
    for r in results {
        match &r.outcome {
            Ok(rep) => writeln!(out, "{}", symbol_row(&r.symbol, rep))?,
            Err(why) => skipped.push(format!("- {}: {why}", r.symbol)),
        }
    }
    if !skipped.is_empty() {
        writeln!(out, "\n## Not run\n\n{}", skipped.join("\n"))?;
    }
    Ok(())
}

fn symbol_row(symbol: &str, r: &RunReport) -> String {
    let m = &r.metrics;
    let net: Decimal = r.trades.iter().map(|t| t.net_pnl).sum();
    let kill = r.kill_switch_engaged_at.and_then(ist_date).map_or_else(|| "-".to_string(), |d| d.to_string());
    format!(
        "| {symbol} | {} | {} | {} | {} | {} | {} | {} | {} | {kill} | {} | {} |",
        r.bars,
        m.trades,
        dec(m.win_rate_pct),
        opt_dec(m.profit_factor),
        dec(net),
        dec(m.total_return_pct),
        dec(m.max_drawdown_pct),
        opt_f64(m.sharpe),
        r.rejections.len(),
        r.entries_sized_to_zero,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn formats_two_decimals_and_missing_values() {
        assert_eq!(dec(dec!(1.23456)), "1.23");
        assert_eq!(opt_dec(None), NA);
        assert_eq!(opt_f64(Some(0.456)), "0.46");
    }

    #[test]
    fn symbol_row_shows_kill_switch_date_and_counts() {
        let r = RunReport { bars: 10, kill_switch_engaged_at: Some(1_325_412_000_000), ..RunReport::default() };
        let row = symbol_row("INFY", &r);
        assert!(row.starts_with("| INFY | 10 | 0 |") && row.contains("2012-01-01"), "{row}");
    }
}

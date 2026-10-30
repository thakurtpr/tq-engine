//! Markdown helpers shared by both modes, and the per-symbol (diagnostic) report.

use std::fmt::Write as _;

use rust_decimal::Decimal;
use tq_backtest::RunReport;
use tq_data::ist_date;

use crate::args::{BacktestArgs, Mode, PriceBasis, SlippageArg};
use crate::backtest::Setup;
use crate::gates::PER_SYMBOL_NOTE;
use crate::stats::{Pooled, SymbolResult, TradeStats, TOP_TRADES_DROPPED};
use crate::universe::Coverage;

pub const NA: &str = "n/a";

pub fn dec(d: Decimal) -> String {
    d.round_dp(2).to_string()
}

pub fn opt_dec(d: Option<Decimal>) -> String {
    d.map_or_else(|| NA.to_string(), dec)
}

pub fn opt_f64(x: Option<f64>) -> String {
    x.map_or_else(|| NA.to_string(), |v| format!("{v:.2}"))
}

/// Run settings, data manifest and (for per-symbol mode) the diagnostic warning.
pub fn write_header(out: &mut String, args: &BacktestArgs, setup: &Setup, manifest: &str) -> std::fmt::Result {
    let l = setup.gate.limits();
    let mode = match args.mode {
        Mode::Portfolio => "portfolio: ONE shared Rs account across all symbols",
        Mode::PerSymbol => "per-symbol: DIAGNOSTIC, NOT A PERFORMANCE CLAIM (each symbol has its own fresh account)",
    };
    writeln!(out, "# Backtest: {:?} ({mode})\n", args.strategy)?;
    writeln!(out, "- Window: {} to {} (earlier bars warm up indicators only)", args.start, args.end)?;
    if args.strategy.is_rebalancer() {
        writeln!(out, "- Starting cash: Rs {}; sized by month-end target weights (no stops; --risk-pct unused), 0.5% cash buffer, trims under 0.5% of equity skipped", args.cash)?;
    } else {
        writeln!(out, "- Starting cash: Rs {}; risk per trade {}% of equity", args.cash, args.risk_pct)?;
    }
    let model = match args.slippage_model {
        SlippageArg::Adopted if args.price_basis == PriceBasis::Unadjusted => "adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap, Rs 0.05 ticks)",
        SlippageArg::Adopted => "adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices",
        SlippageArg::Flat => "flat",
    };
    writeln!(out, "- Slippage: {} bps base per side x{}, {model}; costs: Zerodha NSE delivery", args.slippage_bps, args.slippage_mult)?;
    writeln!(
        out,
        "- Risk gate: max risk {}%, notional cap {}%, max open positions {}, daily loss {}%, drawdown kill switch {}%",
        l.max_risk_per_trade_pct, l.max_position_notional_pct, l.max_open_positions, l.max_daily_loss_pct, l.max_drawdown_pct
    )?;
    if let Some(m) = setup.config.slippage_model.as_ref().filter(|m| m.adv_cap_pct != Decimal::ONE) {
        writeln!(out, "- **ADV cap changed for a sensitivity run: entries cut at {}% of 20-day ADV (adopted: 1%)**", m.adv_cap_pct)?;
    }
    writeln!(out, "- Entry priority when slots or cash run out: {:?}", setup.config.priority)?;
    writeln!(out, "- Strategy parameters: defaults (not tuned)")?;
    writeln!(out, "- Data manifest: {manifest}\n")
}

pub fn write_coverage(out: &mut String, c: &Coverage) -> std::fmt::Result {
    let pct = if c.member_days == 0 { Decimal::ZERO } else { Decimal::from(c.covered_days) * Decimal::ONE_HUNDRED / Decimal::from(c.member_days) };
    writeln!(out, "## Point-in-time universe coverage\n")?;
    writeln!(out, "Entries only while an index member; exits always allowed.\n")?;
    writeln!(out, "Member-days with price data: **{} of {} ({}%)**.\n", c.covered_days, c.member_days, dec(pct))?;
    writeln!(out, "| Member symbol | Price file | Member-days missing | Member-days |\n| --- | --- | ---: | ---: |")?;
    for m in &c.missing {
        writeln!(out, "| {} | {} | {} | {} |", m.symbol, m.price_symbol, m.missing_days, m.member_days)?;
    }
    writeln!(out)
}

/// Trade-level rows common to both modes (no rupee sums: the caller adds those if meaningful).
pub fn trade_rows(t: &TradeStats) -> Vec<(String, String)> {
    let clustered = t.t_month_clustered.map_or_else(|| NA.to_string(), |(v, g)| format!("{v:.2} (G = {g} months)"));
    let share = t.top_symbol_share.as_ref().map_or_else(|| NA.to_string(), |(s, v)| format!("{s} {}%", dec(*v)));
    vec![
        ("Closed trades / symbols traded".into(), format!("{} / {}", t.trades, t.symbols_traded)),
        ("Win rate %".into(), dec(t.win_rate_pct)),
        ("Mean net trade %".into(), dec(t.mean_net_trade_pct)),
        ("Mean R (net)".into(), opt_dec(t.mean_r)),
        ("Profit factor (net)".into(), opt_dec(t.profit_factor)),
        ("t-stat of net trade returns, naive (assumes independent trades)".into(), opt_f64(t.t_stat)),
        ("t-stat, clustered by entry month (review tstat.py, CR1)".into(), clustered),
        ("Friction (costs + slippage) as % of pre-friction gross P&L".into(), opt_dec(t.friction_pct_of_gross)),
        ("Largest symbol share of net P&L".into(), share),
    ]
}

pub fn render(args: &BacktestArgs, setup: &Setup, results: &[SymbolResult], pooled: &Pooled, coverage: Option<&Coverage>, manifest: &str) -> String {
    let mut out = String::new();
    // Writing to a String cannot fail; the results are ignored on purpose.
    let _ = write_header(&mut out, args, setup, manifest);
    if let Some(c) = coverage {
        let _ = write_coverage(&mut out, c);
    }
    let _ = write_pooled(&mut out, pooled, args.cash);
    let _ = writeln!(out, "## Gates\n\n{PER_SYMBOL_NOTE}\n");
    let _ = write_symbols(&mut out, results);
    out
}

fn write_pooled(out: &mut String, p: &Pooled, cash: Decimal) -> std::fmt::Result {
    writeln!(out, "## Pooled closed trades (diagnostic)\n")?;
    writeln!(out, "Independent accounts summed or averaged here are not capital anyone has; rupee figures are shown only as a share of the {} accounts' combined starting capital.\n", p.symbols_run)?;
    writeln!(out, "| Metric | Value |\n| --- | --- |")?;
    let capital = cash * Decimal::from(p.symbols_run.max(1));
    let pct = |x: Decimal| format!("{}%", dec(x / capital * Decimal::ONE_HUNDRED));
    let mut rows = vec![("Symbols run / with trades / kill switch hit".to_string(), format!("{} / {} / {}", p.symbols_run, p.symbols_with_trades, p.kill_switches))];
    rows.extend(trade_rows(&p.trades));
    rows.push(("Net P&L, % of combined starting capital (whole window)".into(), pct(p.trades.net_pnl)));
    rows.push((format!("Net P&L without the best {TOP_TRADES_DROPPED} trades, % of combined capital"), pct(p.trades.net_pnl_without_top_trades)));
    rows.push(("Median per-symbol Sharpe (cash-diluted; NOT the protocol Sharpe)".into(), opt_f64(p.median_sharpe)));
    rows.push(("Median per-symbol max drawdown %".into(), opt_dec(p.median_max_dd_pct)));
    for (k, v) in rows {
        writeln!(out, "| {k} | {v} |")?;
    }
    writeln!(out)
}

fn write_symbols(out: &mut String, results: &[SymbolResult]) -> std::fmt::Result {
    writeln!(out, "## Per symbol\n")?;
    writeln!(out, "| Symbol | Bars | Trades | Win % | PF | Net P&L (Rs) | Return % | Max DD % | Sharpe | Kill switch | Rejected | Size 0 | Non-member | ADV-capped |")?;
    writeln!(out, "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |")?;
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
        "| {symbol} | {} | {} | {} | {} | {} | {} | {} | {} | {kill} | {} | {} | {} | {} |",
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
        r.entries_outside_universe,
        r.entries_adv_capped,
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

    #[test]
    fn pooled_section_reports_no_rupee_sums() {
        let mut out = String::new();
        write_pooled(&mut out, &Pooled { symbols_run: 2, ..Pooled::default() }, dec!(1000000)).unwrap();
        assert!(out.contains("% of combined starting capital") && !out.contains("(Rs)"), "{out}");
    }
}

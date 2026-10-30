//! Validation-protocol gates (`docs/validation-protocol.md`) that one run can assess.
//!
//! Only a shared-account portfolio run can assess edge gates: independent per-symbol accounts
//! take every signal at once, which the position cap forbids (review, section 5.1). Gate 4c follows
//! protocol v2: the Newey-West HAC t of PORTFOLIO DAILY returns, replaced by the lowest of HAC,
//! monthly-return t and per-trade month-clustered t when a cross-check is more than 0.5 below it
//! (`tq_backtest::tstats`). Everything here is in-sample (development window), not walk-forward OOS.

use rust_decimal::Decimal;
use tq_backtest::returns::ReturnStats;
use tq_backtest::tstats::GateT;

use crate::stats::TradeStats;

pub const PER_SYMBOL_NOTE: &str = "All edge gates (3a-3e, 4a-4f) are NOT-ASSESSABLE from independent per-symbol \
accounts: each account has its own cash and position cap, so the pooled trades could never all be taken in one \
account. Use a `--mode portfolio` run for gate verdicts.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    NotAssessable,
    NotMeasured,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS (in-sample)",
            Verdict::Fail => "**FAIL**",
            Verdict::NotAssessable => "NOT-ASSESSABLE",
            Verdict::NotMeasured => "NOT-YET-MEASURED",
        }
    }

    fn of(pass: Option<bool>) -> Self {
        match pass {
            Some(true) => Verdict::Pass,
            Some(false) => Verdict::Fail,
            None => Verdict::NotAssessable,
        }
    }
}

pub struct Gate {
    pub id: &'static str,
    pub name: &'static str,
    pub threshold: &'static str,
    pub value: String,
    pub verdict: Verdict,
}

fn gate(id: &'static str, name: &'static str, threshold: &'static str, value: String, pass: Option<bool>) -> Gate {
    Gate { id, name, threshold, value, verdict: Verdict::of(pass) }
}

fn f2(x: Option<f64>) -> String {
    x.map_or_else(|| "n/a".into(), |v| format!("{v:.2}"))
}

fn d2(x: Option<Decimal>) -> String {
    x.map_or_else(|| "n/a".into(), |v| v.round_dp(2).to_string())
}

/// Gates for a portfolio run. `bench` is the price-index benchmark, if one was given.
pub fn portfolio_rows(t: &TradeStats, r: &ReturnStats, g: &GateT, bench: Option<(&str, &ReturnStats)>) -> Vec<Gate> {
    let share = t.top_symbol_share.as_ref();
    let mut rows = vec![
        gate("3a", "Closed trades", ">= 200", t.trades.to_string(), Some(t.trades >= 200)),
        gate("3c", "Distinct symbols traded", ">= 10", t.symbols_traded.to_string(), Some(t.symbols_traded >= 10)),
        gate("3d", "Largest symbol share of net P&L", "<= 15%", share.map_or("n/a (net P&L <= 0)".into(), |(s, v)| format!("{s} {}%", v.round_dp(2))), share.map(|s| s.1 <= Decimal::from(15))),
        gate("3e", "Net P&L without best 5 trades", "> 0", format!("Rs {}", t.net_pnl_without_top_trades.round_dp(0)), Some(t.net_pnl_without_top_trades > Decimal::ZERO)),
        gate("4a", "Mean net trade %", ">= 0.5%", d2(Some(t.mean_net_trade_pct)), Some(t.mean_net_trade_pct >= Decimal::new(5, 1))),
        gate("4b", "Expectancy (mean R)", ">= +0.20R", d2(t.mean_r), t.mean_r.map(|m| m >= Decimal::new(2, 1))),
        gate("4c", "t, HAC daily portfolio returns (lowest-of-three rule)", ">= 3.0", gate_value(g), g.gate.map(|x| x >= 3.0)),
        gate("4d", "Profit factor (net)", ">= 1.3", d2(t.profit_factor), t.profit_factor.map(|p| p >= Decimal::new(13, 1))),
        gate("4e", "Sharpe, daily portfolio returns", ">= 0.8", f2(r.sharpe), r.sharpe.map(|s| s >= 0.8)),
    ];
    rows.push(benchmark_gate(r, bench));
    rows
}

fn gate_value(g: &GateT) -> String {
    format!("{} (HAC {}, L = {}; monthly {}; per-trade {})", f2(g.gate), f2(g.hac), g.lag, f2(g.monthly), f2(g.per_trade_clustered))
}

/// 4f: strategy Sharpe AND Calmar above buy-and-hold. Against the price index (NSEI) this is
/// necessary, not sufficient (the TRI is higher by the dividend yield).
fn benchmark_gate(r: &ReturnStats, bench: Option<(&str, &ReturnStats)>) -> Gate {
    let Some((name, b)) = bench else {
        return Gate { id: "4f", name: "Beats NIFTY 50 (Sharpe and Calmar)", threshold: "both higher", value: "no benchmark given".into(), verdict: Verdict::NotMeasured };
    };
    let value = format!("Sharpe {} vs {}; Calmar {} vs {} ({name})", f2(r.sharpe), f2(b.sharpe), f2(r.calmar), f2(b.calmar));
    let beats = |a: Option<f64>, b: Option<f64>| a.zip(b).map(|(a, b)| a > b);
    let pass = beats(r.sharpe, b.sharpe).zip(beats(r.calmar, b.calmar)).map(|(s, c)| s && c);
    let threshold = if name == "NSEI" { "both higher; necessary only (price index)" } else { "both higher" };
    gate("4f", "Beats buy-and-hold of the benchmark (Sharpe and Calmar)", threshold, value, pass)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn returns(sharpe: f64, calmar: f64, t: f64) -> ReturnStats {
        ReturnStats { sharpe: Some(sharpe), calmar: Some(calmar), t_month_clustered: Some(t), ..ReturnStats::default() }
    }

    #[test]
    fn edge_gates_use_portfolio_figures() {
        let t = TradeStats { trades: 300, symbols_traded: 40, mean_net_trade_pct: dec!(0.6), mean_r: Some(dec!(0.1)), profit_factor: Some(dec!(1.4)), net_pnl_without_top_trades: dec!(-1), ..TradeStats::default() };
        let g = GateT { hac: Some(3.2), monthly: Some(2.5), gate: Some(2.5), ..GateT::default() };
        let rows = portfolio_rows(&t, &returns(0.9, 0.5, 4.0), &g, None);
        let v = |id: &str| rows.iter().find(|g| g.id == id).map(|g| g.verdict);
        assert_eq!((v("3a"), v("3e"), v("4a"), v("4b")), (Some(Verdict::Pass), Some(Verdict::Fail), Some(Verdict::Pass), Some(Verdict::Fail)));
        assert_eq!((v("4c"), v("4d"), v("4e")), (Some(Verdict::Fail), Some(Verdict::Pass), Some(Verdict::Pass)));
        assert_eq!((v("3d"), v("4f")), (Some(Verdict::NotAssessable), Some(Verdict::NotMeasured)));
    }

    #[test]
    fn benchmark_gate_needs_both_sharpe_and_calmar() {
        let bench = returns(0.6, 0.4, 1.0);
        assert_eq!(benchmark_gate(&returns(0.7, 0.3, 0.0), Some(("NSEI", &bench))).verdict, Verdict::Fail);
        assert_eq!(benchmark_gate(&returns(0.7, 0.5, 0.0), Some(("NIFTYBEES", &bench))).verdict, Verdict::Pass);
    }
}

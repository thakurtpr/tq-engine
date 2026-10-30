//! Turnover, friction and liquidity-model counters for a portfolio or rebalancing run.
//! Shows whether the impact term and ADV cap were ever exercised (review-backtest-03, section 3).

use std::fmt::Write as _;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use tq_backtest::RunReport;

/// Friction and turnover, as fractions of capital, from one run.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Friction {
    /// One-way turnover per year: (buys + sells) / 2 / mean equity / years, percent.
    pub turnover_pct_per_year: Option<f64>,
    /// Total costs and slippage paid over the run, percent of starting capital.
    pub costs_pct_of_capital: Option<f64>,
    pub slippage_pct_of_capital: Option<f64>,
}

pub fn friction(r: &RunReport, cash: Decimal, years: f64) -> Friction {
    let pct = |x: Decimal| (!cash.is_zero()).then(|| (x / cash * Decimal::ONE_HUNDRED).to_f64()).flatten();
    let points = r.equity_curve.len();
    let mean_equity = (points > 0).then(|| r.equity_curve.iter().map(|p| p.1).sum::<Decimal>() / Decimal::from(points));
    let turnover = mean_equity
        .filter(|m| *m > Decimal::ZERO && years > 0.0)
        .and_then(|m| (r.turnover / Decimal::TWO / m).to_f64())
        .map(|t| t / years * 100.0);
    Friction { turnover_pct_per_year: turnover, costs_pct_of_capital: pct(r.metrics.total_costs), slippage_pct_of_capital: pct(r.total_slippage) }
}

fn p2(x: Option<f64>) -> String {
    x.map_or_else(|| "n/a".into(), |v| format!("{v:.2}%"))
}

pub fn write(out: &mut String, r: &RunReport, f: &Friction, rebalancer: bool) -> std::fmt::Result {
    writeln!(out, "## Turnover, friction and the liquidity model\n\n| Metric | Value |\n| --- | --- |")?;
    if rebalancer {
        writeln!(out, "| Rebalance dates traded | {} |", r.rebalances)?;
    }
    writeln!(out, "| Turnover, one-way, per year (% of mean equity) | {} |", p2(f.turnover_pct_per_year))?;
    writeln!(out, "| Costs paid over the run (% of starting capital) | {} |", p2(f.costs_pct_of_capital))?;
    writeln!(out, "| Slippage paid over the run (% of starting capital) | {} |", p2(f.slippage_pct_of_capital))?;
    writeln!(out, "| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | {} / {} |", r.fills_above_impact_threshold, r.orders_filled)?;
    writeln!(out, "| Entries cut to the 1% ADV cap | {} |", r.entries_adv_capped)?;
    if rebalancer {
        writeln!(out, "| Buys without an ADV estimate (zero or missing volume: base slippage, no cap) | {} |", r.fills_without_adv)?;
    }
    writeln!(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn friction_is_relative_to_capital_and_mean_equity() {
        let r = RunReport {
            equity_curve: vec![(0, dec!(100)), (1, dec!(300))],
            turnover: dec!(800),
            total_slippage: dec!(2),
            metrics: tq_backtest::Metrics { total_costs: dec!(5), ..Default::default() },
            ..RunReport::default()
        };
        // Mean equity 200; one-way turnover 400 / 200 = 2x over 2 years -> 100% a year.
        let f = friction(&r, dec!(100), 2.0);
        assert_eq!((f.costs_pct_of_capital, f.slippage_pct_of_capital), (Some(5.0), Some(2.0)));
        assert!((f.turnover_pct_per_year.unwrap_or(0.0) - 100.0).abs() < 1e-12);
        assert_eq!(friction(&RunReport::default(), dec!(100), 1.0).turnover_pct_per_year, None);
    }
}

//! Run summary for reports: trade counts, gross vs net, profit factor, per-trade bps, the
//! break-even cost, the HAC t of daily returns (protocol gate 4c, via `tq_backtest::tstats`),
//! and an optional simplified Indian-resident after-tax view for crypto.
//!
//! Statistics are `f64`; money totals stay `Decimal`.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::{json, Value};
use tq_backtest::tstats::{hac_t, protocol_lag};

use crate::journal::dec;
use crate::runner::{RunOutcome, TradeRecord};
use crate::strategies::Window;

const BPS: f64 = 10_000.0;
/// Intraday trades never span more than one session day.
const LONGEST_HOLD_DAYS: usize = 1;

/// Simplified s.115BBH reading: `rate_pct` on each winning trade's (sale - cost of acquisition),
/// fees not deductible, losses never offset. NOT tax advice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerTradeTax {
    pub rate_pct: Decimal,
}

fn bps(x: Decimal, base: Decimal) -> f64 {
    if base.is_zero() {
        return 0.0;
    }
    crate::strategies::to_f64(x / base) * BPS
}

fn mean(xs: &[f64]) -> Option<f64> {
    let mut w = Window::new(xs.len().max(1));
    xs.iter().for_each(|x| w.push(*x));
    w.mean()
}

fn profit_factor(trades: &[TradeRecord]) -> Option<f64> {
    let wins: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p > Decimal::ZERO).sum();
    let losses: Decimal = trades.iter().map(|t| t.net_pnl).filter(|p| *p < Decimal::ZERO).sum();
    (!losses.is_zero()).then(|| crate::strategies::to_f64(wins / -losses))
}

/// Daily simple returns from the end-of-day equity series, the first against `initial`.
pub fn daily_returns(initial: Decimal, daily: &[(i64, Decimal)]) -> Vec<f64> {
    let mut prev = initial;
    let mut out = Vec::with_capacity(daily.len());
    for &(_, e) in daily {
        if !prev.is_zero() {
            out.push(crate::strategies::to_f64(e / prev) - 1.0);
        }
        prev = e;
    }
    out
}

fn per_trade(trades: &[TradeRecord], f: impl Fn(&TradeRecord) -> Decimal) -> Option<f64> {
    mean(&trades.iter().map(|t| bps(f(t), t.notional())).collect::<Vec<_>>())
}

fn after_tax(trades: &[TradeRecord], tax: PerTradeTax) -> Value {
    let tax_of = |t: &TradeRecord| {
        let gain = (t.exit_fill - t.entry_fill) * t.qty;
        gain.max(Decimal::ZERO) * tax.rate_pct / Decimal::ONE_HUNDRED
    };
    let total_tax: Decimal = trades.iter().map(tax_of).sum();
    let net_after: Decimal = trades.iter().map(|t| t.net_pnl - tax_of(t)).sum();
    json!({
        "tax_rate_pct": dec(tax.rate_pct), "tax_total": dec(total_tax.round_dp(2)), "net_after_tax_total": dec(net_after.round_dp(2)),
        "mean_after_tax_bps": per_trade(trades, |t| t.net_pnl - tax_of(t)),
    })
}

fn tds_view(o: &RunOutcome, tds_pct: Decimal) -> Value {
    let turnover: Decimal = o.trades.iter().map(|t| t.exit_fill * t.qty).sum();
    let withheld = turnover * tds_pct / Decimal::ONE_HUNDRED;
    let years = o.daily.len() as f64 / 365.0;
    let pct_of_capital = bps(withheld, o.initial_cash) / 100.0;
    json!({
        "tds_pct": dec(tds_pct), "sell_turnover": dec(turnover.round_dp(2)), "tds_withheld": dec(withheld.round_dp(2)),
        "tds_withheld_pct_of_initial_capital": pct_of_capital,
        "tds_per_year_pct_of_initial_capital": if years > 0.0 { Some(pct_of_capital / years) } else { None },
    })
}

fn reasons(trades: &[TradeRecord]) -> BTreeMap<&'static str, usize> {
    let mut m = BTreeMap::new();
    for t in trades {
        *m.entry(t.reason.as_str()).or_insert(0) += 1;
    }
    m
}

/// The JSON summary of a run. `tax` adds the after-tax view; `tds_pct` the TDS cash drag.
pub fn summarize(o: &RunOutcome, tax: Option<PerTradeTax>, tds_pct: Option<Decimal>) -> Value {
    let t = &o.trades;
    let sum = |f: fn(&TradeRecord) -> Decimal| t.iter().map(f).sum::<Decimal>();
    let rets = daily_returns(o.initial_cash, &o.daily);
    let lag = protocol_lag(rets.len(), LONGEST_HOLD_DAYS);
    let mean_gross = per_trade(t, |x| x.gross_pnl);
    json!({
        "trades": t.len(),
        "win_rate": (!t.is_empty()).then(|| t.iter().filter(|x| x.net_pnl > Decimal::ZERO).count() as f64 / t.len() as f64),
        "gross_total": dec(sum(|x| x.gross_pnl).round_dp(2)), "fees_total": dec(sum(|x| x.fees).round_dp(2)),
        "slippage_total": dec(sum(|x| x.slippage).round_dp(2)), "net_total": dec(sum(|x| x.net_pnl).round_dp(2)),
        "profit_factor_net": profit_factor(t),
        "mean_gross_bps": mean_gross, "mean_fee_bps": per_trade(t, |x| x.fees),
        "mean_slippage_bps": per_trade(t, |x| x.slippage), "mean_net_bps": per_trade(t, |x| x.net_pnl),
        "breakeven_round_trip_cost_bps": mean_gross,
        "days": rets.len(), "hac_lag": lag, "hac_t_daily": hac_t(&rets, lag),
        "initial_cash": dec(o.initial_cash), "final_equity": dec(o.final_equity.round_dp(2)),
        "total_return_pct": bps(o.final_equity - o.initial_cash, o.initial_cash) / 100.0,
        "exit_reasons": reasons(t), "kill_switch_at": o.kill_switch_at, "open_at_end": o.open_at_end,
        "tds_receivable": dec(o.tds_receivable.round_dp(2)),
        "counters": counters(o),
        "after_tax_india": tax.map(|x| after_tax(t, x)),
        "tds_india": tds_pct.map(|p| tds_view(o, p)),
    })
}

fn counters(o: &RunOutcome) -> Value {
    let c = &o.counters;
    json!({
        "bars": c.bars, "out_of_session": c.out_of_session, "stale_bars": c.stale_bars, "gaps": c.gaps,
        "missing_bars": c.missing_bars, "entry_signals": c.entry_signals, "rejected": c.rejected,
        "suppressed": c.suppressed, "cancelled": c.cancelled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{Counters, ExitReason};
    use rust_decimal_macros::dec;

    fn trade(entry: Decimal, exit: Decimal, fees: Decimal) -> TradeRecord {
        TradeRecord {
            entry_ts: 0, exit_ts: 1, qty: dec!(10), raw_entry: entry, raw_exit: exit, entry_fill: entry,
            exit_fill: exit, gross_pnl: (exit - entry) * dec!(10), fees, slippage: dec!(0),
            net_pnl: (exit - entry) * dec!(10) - fees, tds: dec!(0), reason: ExitReason::Signal,
        }
    }

    fn outcome(trades: Vec<TradeRecord>) -> RunOutcome {
        RunOutcome {
            trades, daily: vec![(0, dec!(1010)), (1, dec!(1000)), (2, dec!(1020))], initial_cash: dec!(1000),
            final_equity: dec!(1020), tds_receivable: dec!(0), kill_switch_at: None, open_at_end: false,
            counters: Counters::default(),
        }
    }

    #[test]
    fn per_trade_bps_profit_factor_and_breakeven() {
        // +100 bps gross with 20 fee (20 bps), and -50 bps gross with 10 fee.
        let o = outcome(vec![trade(dec!(100), dec!(101), dec!(2)), trade(dec!(100), dec!(99.5), dec!(2))]);
        let s = summarize(&o, None, None);
        assert_eq!(s["trades"], 2);
        assert!((s["mean_gross_bps"].as_f64().unwrap() - 25.0).abs() < 1e-9);
        assert!((s["mean_net_bps"].as_f64().unwrap() - 5.0).abs() < 1e-9);
        assert!((s["profit_factor_net"].as_f64().unwrap() - 8.0 / 7.0).abs() < 1e-9);
        assert_eq!(s["breakeven_round_trip_cost_bps"], s["mean_gross_bps"]);
        assert_eq!(s["days"], 3);
    }

    #[test]
    fn after_tax_taxes_only_winners_and_ignores_fees() {
        let o = outcome(vec![trade(dec!(100), dec!(101), dec!(2)), trade(dec!(100), dec!(99), dec!(2))]);
        let s = summarize(&o, Some(PerTradeTax { rate_pct: dec!(30) }), Some(dec!(1)));
        // Winner gain 10 (fees not deductible) -> tax 3; net -4 -> after tax -7.
        assert_eq!(s["after_tax_india"]["tax_total"], "3");
        assert_eq!(s["after_tax_india"]["net_after_tax_total"], "-7");
        // Sell turnover 1010 + 990 = 2000 -> TDS 20 = 2% of 1000.
        assert_eq!(s["tds_india"]["tds_withheld"], "20");
    }

    #[test]
    fn daily_returns_start_from_initial_cash() {
        let r = daily_returns(dec!(100), &[(0, dec!(110)), (1, dec!(99))]);
        assert!((r[0] - 0.10).abs() < 1e-12 && (r[1] + 0.10).abs() < 1e-12);
    }
}

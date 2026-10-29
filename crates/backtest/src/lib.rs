//! Event-driven backtester (skeleton). Next: full Indian cost model (brokerage, STT, stamp,
//! exchange, SEBI, GST), slippage, mark-to-market equity, walk-forward and Monte Carlo.

use tq_broker::Broker;
use tq_core::{Bar, Symbol};
use tq_risk::{AccountState, RiskGate};
use tq_strategy::Strategy;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub bars: usize,
    pub orders_proposed: usize,
    pub orders_filled: usize,
    pub rejections: Vec<String>,
}

/// Runs `strategy` over `bars`. Orders proposed on bar N fill at bar N+1's open,
/// so a strategy can never trade on the bar it just saw (no look-ahead).
pub fn run<S: Strategy, B: Broker>(
    strategy: &mut S,
    broker: &mut B,
    gate: &RiskGate,
    acct: &AccountState,
    symbol: &Symbol,
    bars: &[Bar],
) -> RunReport {
    let mut report = RunReport { bars: bars.len(), ..Default::default() };
    for (bar, next) in bars.iter().zip(bars.iter().skip(1)) {
        for order in strategy.on_bar(symbol, bar) {
            report.orders_proposed += 1;
            let approved = match gate.check(order, acct) {
                Ok(a) => a,
                Err(v) => {
                    report.rejections.push(format!("risk: {v}"));
                    continue;
                }
            };
            match broker.submit(approved, next.ts, next.open) {
                Ok(fill) => {
                    strategy.on_fill(&fill);
                    report.orders_filled += 1;
                }
                Err(e) => report.rejections.push(e.to_string()),
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use tq_broker::Paper;
    use tq_core::{OrderRequest, Side};
    use tq_risk::RiskLimits;

    struct BuyEveryBar;

    impl Strategy for BuyEveryBar {
        fn name(&self) -> &str {
            "buy-every-bar"
        }
        fn on_bar(&mut self, s: &Symbol, b: &Bar) -> Vec<OrderRequest> {
            vec![OrderRequest::new(s.clone(), Side::Buy, dec!(1), b.close, b.close - dec!(1)).unwrap()]
        }
    }

    fn acct() -> AccountState {
        AccountState {
            equity: dec!(100000),
            start_of_day_equity: dec!(100000),
            peak_equity: dec!(100000),
            open_positions: 0,
            kill_switch: false,
        }
    }

    fn bars() -> Vec<Bar> {
        (0..3)
            .map(|i| Bar::new(i, dec!(100) + Decimal::from(i), dec!(110), dec!(90), dec!(100), dec!(1)).unwrap())
            .collect()
    }

    fn gate() -> RiskGate {
        RiskGate::new(RiskLimits::default())
    }

    #[test]
    fn fills_at_next_bar_open_never_same_bar() {
        let (mut s, mut p) = (BuyEveryBar, Paper::default());
        let r = run(&mut s, &mut p, &gate(), &acct(), &Symbol("X".into()), &bars());
        assert_eq!(r.orders_filled, 2);
        assert_eq!(p.fills()[0].price, dec!(101));
        assert_eq!(p.fills()[0].ts, 1);
    }

    #[test]
    fn kill_switch_blocks_every_order() {
        let (mut s, mut p) = (BuyEveryBar, Paper::default());
        let a = AccountState { kill_switch: true, ..acct() };
        let r = run(&mut s, &mut p, &gate(), &a, &Symbol("X".into()), &bars());
        assert_eq!(r.orders_filled, 0);
        assert_eq!(r.rejections.len(), 2);
    }
}

//! Helpers for strategy tests: synthetic bars and a tiny driver that mimics the engine.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tq_core::{Bar, Fill, Side, Symbol};
use tq_strategy::{Signal, Strategy};

pub(crate) fn sym() -> Symbol {
    Symbol("NSE:TEST".into())
}

/// A bar with open = close and a one-rupee range either side.
pub(crate) fn bar(ts: i64, close: Decimal) -> Bar {
    Bar::new(ts, close, close + dec!(1), close - dec!(1), close, dec!(1000)).expect("valid synthetic bar")
}

pub(crate) fn bars(closes: &[Decimal]) -> Vec<Bar> {
    closes.iter().enumerate().map(|(i, c)| bar(i as i64, *c)).collect()
}

fn fill(ts: i64, side: Side, price: Decimal) -> Fill {
    Fill { ts, symbol: sym(), side, qty: dec!(1), price, costs: Decimal::ZERO }
}

/// Feeds `bars` like the engine: every signal is filled before the next bar is seen.
/// Returns `(bar index, signal)` for each signal emitted.
pub(crate) fn drive<S: Strategy>(strategy: &mut S, bars: &[Bar]) -> Vec<(usize, Signal)> {
    let mut out = Vec::new();
    for (i, b) in bars.iter().enumerate() {
        for signal in strategy.on_bar(&sym(), b) {
            let side = match &signal {
                Signal::Enter(_) => Side::Buy,
                Signal::Exit(_) => Side::Sell,
            };
            strategy.on_fill(&fill(b.ts, side, b.close));
            out.push((i, signal));
        }
    }
    out
}

/// Indices and kinds only, for compact assertions: `('E', i)` entry, `('X', i)` exit.
pub(crate) fn kinds(signals: &[(usize, Signal)]) -> Vec<(char, usize)> {
    signals
        .iter()
        .map(|(i, s)| (if matches!(s, Signal::Enter(_)) { 'E' } else { 'X' }, *i))
        .collect()
}

/// No-look-ahead check: two runs sharing `history` but with different futures must emit
/// identical signals on every history bar.
pub(crate) fn assert_no_look_ahead<S: Strategy>(
    mut make: impl FnMut() -> S,
    history: &[Decimal],
    future_a: &[Decimal],
    future_b: &[Decimal],
) {
    let run = |future: &[Decimal], s: &mut S| drive(s, &bars(&[history, future].concat()));
    let (a, b) = (run(future_a, &mut make()), run(future_b, &mut make()));
    let upto = |v: &[(usize, Signal)]| v.iter().filter(|(i, _)| *i < history.len()).cloned().collect::<Vec<_>>();
    assert!(!upto(&a).is_empty(), "history must produce signals for the test to mean anything");
    assert_eq!(upto(&a), upto(&b));
    assert_ne!(a, b, "futures should diverge, proving they were actually fed");
}

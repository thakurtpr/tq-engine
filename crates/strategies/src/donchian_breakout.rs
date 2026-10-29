//! Classic turtle-style channel breakout.

use crate::book::{long_entry, Book, Position};
use crate::ConfigError;
use rust_decimal::Decimal;
use tq_core::{Bar, Fill, Symbol};
use tq_indicators::{Atr, RollingHigh, RollingLow};
use tq_strategy::{Signal, Strategy};

/// Parameters for [`DonchianBreakout`]. `Default` gives the textbook values.
#[derive(Debug, Clone, PartialEq)]
pub struct DonchianBreakoutConfig {
    /// Enter when the close exceeds the highest high of this many *prior* bars.
    pub entry_period: usize,
    /// Exit when the close falls below the lowest low of this many *prior* bars.
    pub exit_period: usize,
    pub atr_period: usize,
    /// Stop distance below entry, in ATRs.
    pub stop_atr_mult: Decimal,
}

impl Default for DonchianBreakoutConfig {
    fn default() -> Self {
        Self { entry_period: 55, exit_period: 20, atr_period: 20, stop_atr_mult: Decimal::TWO }
    }
}

/// Donchian channel breakout (Turtle System 2 style, long only).
///
/// Enter long when `close > highest high of the prior entry_period bars`; stop =
/// `close - stop_atr_mult * ATR`. Exit when `close < lowest low of the prior exit_period
/// bars`. The channel excludes the current bar, so a bar never breaks out of itself. Emits
/// nothing until `entry_period` prior bars and the ATR are available.
///
/// **Edge unproven.** This is a textbook strategy used to validate the engine. Do not trade
/// it until it passes walk-forward testing net of costs.
#[derive(Debug, Clone)]
pub struct DonchianBreakout {
    config: DonchianBreakoutConfig,
    book: Book<State>,
}

#[derive(Debug, Clone)]
struct State {
    high: RollingHigh,
    low: RollingLow,
    atr: Atr,
    /// Channel values as of the previous bar.
    prior_high: Option<Decimal>,
    prior_low: Option<Decimal>,
    position: Position,
}

impl DonchianBreakout {
    pub fn new(config: DonchianBreakoutConfig) -> Result<Self, ConfigError> {
        if config.stop_atr_mult <= Decimal::ZERO {
            return Err(ConfigError("stop_atr_mult must be positive"));
        }
        let template = State {
            high: RollingHigh::new(config.entry_period).ok_or(ConfigError("entry_period must be positive"))?,
            low: RollingLow::new(config.exit_period).ok_or(ConfigError("exit_period must be positive"))?,
            atr: Atr::new(config.atr_period).ok_or(ConfigError("atr_period must be positive"))?,
            prior_high: None,
            prior_low: None,
            position: Position::Flat,
        };
        Ok(Self { config, book: Book::new(template) })
    }
}

/// The channel as it stood before the current bar, plus the ATR including it.
struct Reading {
    prior_high: Option<Decimal>,
    prior_low: Option<Decimal>,
    atr: Option<Decimal>,
}

impl State {
    /// Reads the prior-bar channel, then rolls the channel forward to include `bar`.
    fn update(&mut self, bar: &Bar) -> Reading {
        let reading = Reading {
            prior_high: self.prior_high,
            prior_low: self.prior_low,
            atr: self.atr.update(bar.high, bar.low, bar.close),
        };
        self.prior_high = self.high.update(bar.high);
        self.prior_low = self.low.update(bar.low);
        reading
    }
}

impl Strategy for DonchianBreakout {
    fn name(&self) -> &str {
        "donchian-breakout"
    }

    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal> {
        let stop_atr_mult = self.config.stop_atr_mult;
        let state = self.book.state(symbol);
        let r = state.update(bar);
        match state.position {
            Position::Long { bars_held } => {
                if r.prior_low.is_some_and(|low| bar.close < low) {
                    state.position = Position::Flat;
                    return vec![Signal::Exit(symbol.clone())];
                }
                state.position = Position::Long { bars_held: bars_held + 1 };
                Vec::new()
            }
            Position::Flat => match (r.prior_high, r.atr) {
                (Some(high), Some(atr)) if bar.close > high => {
                    long_entry(symbol, bar.close, atr, stop_atr_mult).into_iter().collect()
                }
                _ => Vec::new(),
            },
        }
    }

    fn on_fill(&mut self, fill: &Fill) {
        if let Some(state) = self.book.existing(&fill.symbol) {
            state.position = Position::after_fill(fill.side);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_look_ahead, bars, drive, kinds};
    use rust_decimal_macros::dec;

    fn small() -> DonchianBreakoutConfig {
        DonchianBreakoutConfig { entry_period: 3, exit_period: 2, atr_period: 2, stop_atr_mult: dec!(2) }
    }

    fn strat(c: DonchianBreakoutConfig) -> DonchianBreakout {
        DonchianBreakout::new(c).unwrap()
    }

    /// Flat at 100 (range 99..101), breakout to 103, run to 105, pull back to 104, then 101.
    fn series() -> Vec<Decimal> {
        vec![dec!(100), dec!(100), dec!(100), dec!(103), dec!(105), dec!(104), dec!(101)]
    }

    #[test]
    fn rejects_invalid_configs() {
        let bad = [
            DonchianBreakoutConfig { entry_period: 0, ..small() },
            DonchianBreakoutConfig { exit_period: 0, ..small() },
            DonchianBreakoutConfig { atr_period: 0, ..small() },
            DonchianBreakoutConfig { stop_atr_mult: dec!(-1), ..small() },
        ];
        for c in bad {
            assert!(DonchianBreakout::new(c.clone()).is_err(), "{c:?}");
        }
        assert!(DonchianBreakout::new(DonchianBreakoutConfig::default()).is_ok());
    }

    #[test]
    fn enters_on_breakout_above_prior_high_with_atr_stop() {
        let out = drive(&mut strat(small()), &bars(&series()[..4]));
        assert_eq!(kinds(&out), [('E', 3)]);
        // ATR(2): TR 2, 2 => 2; TR 2 => 2; breakout TR = max(2, 104-100, 102-100) = 4 => 3.
        // Stop = 103 - 2*3 = 97.
        let Signal::Enter(o) = &out[0].1 else { unreachable!() };
        assert_eq!((o.entry, o.stop), (dec!(103), dec!(97)));
    }

    #[test]
    fn close_equal_to_prior_high_is_not_a_breakout() {
        // Prior 3-bar high is 101; a close of exactly 101 must not enter.
        let out = drive(&mut strat(small()), &bars(&[dec!(100), dec!(100), dec!(100), dec!(101)]));
        assert!(out.is_empty());
    }

    #[test]
    fn exits_below_prior_low_and_holds_until_then() {
        // Index 4: prior 2-bar low min(99, 102) = 99. Index 5: min(102, 104) = 102, close 104.
        // Index 6: min(104, 103) = 103, close 101 < 103 => exit.
        let out = drive(&mut strat(small()), &bars(&series()));
        assert_eq!(kinds(&out), [('E', 3), ('X', 6)]);
    }

    #[test]
    fn does_not_re_enter_while_long() {
        let rising: Vec<Decimal> = (0..10).map(|i| dec!(100) + Decimal::from(3 * i)).collect();
        let out = drive(&mut strat(small()), &bars(&rising));
        assert_eq!(kinds(&out), [('E', 3)]);
    }

    #[test]
    fn silent_during_warm_up_even_when_price_rises_every_bar() {
        let c = DonchianBreakoutConfig::default();
        let rising: Vec<Decimal> = (0..80).map(|i| dec!(100) + Decimal::from(3 * i)).collect();
        let out = drive(&mut strat(c.clone()), &bars(&rising));
        assert_eq!(kinds(&out), [('E', c.entry_period)]);
    }

    #[test]
    fn no_look_ahead() {
        assert_no_look_ahead(|| strat(small()), &series()[..5], &[dec!(110)], &[dec!(90)]);
    }
}

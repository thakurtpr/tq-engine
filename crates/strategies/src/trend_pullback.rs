//! Buy short, sharp dips inside an established uptrend.

use crate::book::{long_entry, Book, Position};
use crate::ConfigError;
use rust_decimal::Decimal;
use tq_core::{Bar, Fill, Symbol};
use tq_indicators::{Atr, Ema, Rsi, Sma};
use tq_strategy::{Signal, Strategy};

/// Parameters for [`TrendPullback`]. `Default` gives the textbook values.
#[derive(Debug, Clone, PartialEq)]
pub struct TrendPullbackConfig {
    /// Close must be above this SMA for the market to count as an uptrend.
    pub trend_sma_period: usize,
    /// Fast EMA: must be above the slow EMA to enter; a close above it exits.
    pub fast_ema_period: usize,
    /// Slow EMA of the trend filter.
    pub slow_ema_period: usize,
    pub rsi_period: usize,
    /// Enter when RSI is strictly below this level (0..=100).
    pub rsi_entry_below: Decimal,
    pub atr_period: usize,
    /// Stop distance below entry, in ATRs.
    pub stop_atr_mult: Decimal,
    /// Exit after holding this many bars even if the fast-EMA exit has not fired.
    pub max_hold_bars: usize,
}

impl Default for TrendPullbackConfig {
    fn default() -> Self {
        Self {
            trend_sma_period: 200,
            fast_ema_period: 20,
            slow_ema_period: 50,
            rsi_period: 2,
            rsi_entry_below: Decimal::TEN,
            atr_period: 14,
            stop_atr_mult: Decimal::TWO,
            max_hold_bars: 10,
        }
    }
}

/// Trend pullback (Connors-style RSI(2) dip buying).
///
/// Uptrend = `close > SMA(trend)` and `EMA(fast) > EMA(slow)`. Enter long when `RSI < level`;
/// stop = `close - stop_atr_mult * ATR`. Exit when `close > EMA(fast)` or after
/// `max_hold_bars`. Emits nothing until every indicator is warm (`trend_sma_period` bars by
/// default).
///
/// **Edge unproven.** This is a textbook strategy used to validate the engine. Do not trade
/// it until it passes walk-forward testing net of costs.
#[derive(Debug, Clone)]
pub struct TrendPullback {
    config: TrendPullbackConfig,
    book: Book<State>,
}

#[derive(Debug, Clone)]
struct State {
    sma: Sma,
    fast: Ema,
    slow: Ema,
    rsi: Rsi,
    atr: Atr,
    position: Position,
}

/// One bar's warmed-up indicator values.
struct Reading {
    sma: Decimal,
    fast: Decimal,
    slow: Decimal,
    rsi: Decimal,
    atr: Decimal,
}

impl TrendPullback {
    pub fn new(config: TrendPullbackConfig) -> Result<Self, ConfigError> {
        validate(&config)?;
        let template = State {
            sma: Sma::new(config.trend_sma_period).ok_or(ConfigError("trend_sma_period must be positive"))?,
            fast: Ema::new(config.fast_ema_period).ok_or(ConfigError("fast_ema_period must be positive"))?,
            slow: Ema::new(config.slow_ema_period).ok_or(ConfigError("slow_ema_period must be positive"))?,
            rsi: Rsi::new(config.rsi_period).ok_or(ConfigError("rsi_period must be positive"))?,
            atr: Atr::new(config.atr_period).ok_or(ConfigError("atr_period must be positive"))?,
            position: Position::Flat,
        };
        Ok(Self { config, book: Book::new(template) })
    }

    /// Exit rule while long: close above the fast EMA, or the holding period is used up.
    fn manage_long(&self, bar: &Bar, fast: Decimal, bars_held: usize) -> Position {
        let held = bars_held + 1;
        if bar.close > fast || held >= self.config.max_hold_bars {
            Position::Flat
        } else {
            Position::Long { bars_held: held }
        }
    }

    /// Entry rule while flat: uptrend plus an oversold RSI.
    fn entry(&self, symbol: &Symbol, bar: &Bar, r: &Reading) -> Option<Signal> {
        let uptrend = bar.close > r.sma && r.fast > r.slow;
        let oversold = r.rsi < self.config.rsi_entry_below;
        if !(uptrend && oversold) {
            return None;
        }
        long_entry(symbol, bar.close, r.atr, self.config.stop_atr_mult)
    }
}

fn validate(c: &TrendPullbackConfig) -> Result<(), ConfigError> {
    if c.fast_ema_period >= c.slow_ema_period {
        return Err(ConfigError("fast_ema_period must be below slow_ema_period"));
    }
    if c.rsi_entry_below <= Decimal::ZERO || c.rsi_entry_below > Decimal::ONE_HUNDRED {
        return Err(ConfigError("rsi_entry_below must be in (0, 100]"));
    }
    if c.stop_atr_mult <= Decimal::ZERO {
        return Err(ConfigError("stop_atr_mult must be positive"));
    }
    if c.max_hold_bars == 0 {
        return Err(ConfigError("max_hold_bars must be positive"));
    }
    Ok(())
}

impl State {
    /// Updates every indicator (always all of them, so none falls behind) and returns the
    /// values once all are warm.
    fn update(&mut self, bar: &Bar) -> Option<Reading> {
        let sma = self.sma.update(bar.close);
        let fast = self.fast.update(bar.close);
        let slow = self.slow.update(bar.close);
        let rsi = self.rsi.update(bar.close);
        let atr = self.atr.update(bar.high, bar.low, bar.close);
        Some(Reading { sma: sma?, fast: fast?, slow: slow?, rsi: rsi?, atr: atr? })
    }
}

impl Strategy for TrendPullback {
    fn name(&self) -> &str {
        "trend-pullback"
    }

    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal> {
        let state = self.book.state(symbol);
        let Some(reading) = state.update(bar) else { return Vec::new() };
        let position = state.position;
        match position {
            Position::Long { bars_held } => {
                let next = self.manage_long(bar, reading.fast, bars_held);
                self.book.state(symbol).position = next;
                if next == Position::Flat { vec![Signal::Exit(symbol.clone())] } else { Vec::new() }
            }
            Position::Flat => self.entry(symbol, bar, &reading).into_iter().collect(),
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
    use crate::test_support::{assert_no_look_ahead, bars, drive, kinds, sym};
    use rust_decimal_macros::dec;

    /// Small periods so the synthetic series stays short and hand-checkable.
    fn small() -> TrendPullbackConfig {
        TrendPullbackConfig {
            trend_sma_period: 20,
            fast_ema_period: 5,
            slow_ema_period: 10,
            rsi_period: 2,
            rsi_entry_below: dec!(10),
            atr_period: 3,
            stop_atr_mult: dec!(2),
            max_hold_bars: 3,
        }
    }

    /// 25 bars rising 2/day from 100 to 148, then three 4-point drops to 136. On index 27:
    /// SMA20 = 133.2 < 136, EMA5 = 140.44 > EMA10 = 139.21, RSI2 = 6.67 < 10 => enter.
    fn dip_history() -> Vec<Decimal> {
        let mut v: Vec<Decimal> = (0..25).map(|i| dec!(100) + Decimal::from(2 * i)).collect();
        v.extend([dec!(144), dec!(140), dec!(136)]);
        v
    }

    fn strat(c: TrendPullbackConfig) -> TrendPullback {
        TrendPullback::new(c).unwrap()
    }

    #[test]
    fn rejects_invalid_configs() {
        let bad = [
            TrendPullbackConfig { trend_sma_period: 0, ..small() },
            TrendPullbackConfig { fast_ema_period: 10, slow_ema_period: 10, ..small() },
            TrendPullbackConfig { rsi_entry_below: dec!(0), ..small() },
            TrendPullbackConfig { rsi_entry_below: dec!(101), ..small() },
            TrendPullbackConfig { stop_atr_mult: dec!(0), ..small() },
            TrendPullbackConfig { max_hold_bars: 0, ..small() },
            TrendPullbackConfig { atr_period: 0, ..small() },
        ];
        for c in bad {
            assert!(TrendPullback::new(c.clone()).is_err(), "{c:?}");
        }
        assert!(TrendPullback::new(TrendPullbackConfig::default()).is_ok());
    }

    #[test]
    fn enters_on_dip_with_atr_stop() {
        let out = drive(&mut strat(small()), &bars(&dip_history()));
        assert_eq!(kinds(&out), [('E', 27)]);
        let Signal::Enter(o) = &out[0].1 else { unreachable!() };
        let mut atr = Atr::new(small().atr_period).unwrap();
        let atr = bars(&dip_history()).iter().filter_map(|b| atr.update(b.high, b.low, b.close)).last().unwrap();
        assert_eq!((o.entry, o.stop, o.symbol.clone()), (dec!(136), dec!(136) - dec!(2) * atr, sym()));
        assert!(atr > dec!(3) && atr < dec!(5), "ATR {atr} should sit between rising-bar TR 3 and drop TR 5");
    }

    #[test]
    fn exits_when_close_crosses_above_fast_ema() {
        let mut closes = dip_history();
        closes.push(dec!(145)); // EMA5 ~ 141.96 < 145
        let out = drive(&mut strat(small()), &bars(&closes));
        assert_eq!(kinds(&out), [('E', 27), ('X', 28)]);
        assert_eq!(out[1].1, Signal::Exit(sym()));
    }

    #[test]
    fn exits_after_max_hold_bars() {
        let mut closes = dip_history();
        closes.extend([dec!(135), dec!(134), dec!(133), dec!(132)]); // stays below EMA5
        let out = drive(&mut strat(small()), &bars(&closes));
        assert_eq!(kinds(&out)[..2], [('E', 27), ('X', 30)]);
    }

    #[test]
    fn unfilled_entry_leaves_strategy_flat() {
        // No on_fill (e.g. risk rejected the entry): the close above EMA5 must not emit an exit.
        let mut s = strat(small());
        let mut closes = dip_history();
        closes.push(dec!(145));
        let signals: Vec<_> = bars(&closes).iter().flat_map(|b| s.on_bar(&sym(), b)).collect();
        assert_eq!(signals.len(), 1);
        assert!(matches!(signals[0], Signal::Enter(_)));
    }

    #[test]
    fn silent_during_default_warm_up() {
        // Rising 1/day with a 3-day, 3-point dip every 20 bars. RSI(2) dips below 10 from
        // bar 19 on, but the 200-bar SMA is only warm at index 199.
        let mut p = dec!(100);
        let closes: Vec<Decimal> = (0..260)
            .map(|i| {
                p += if i % 20 >= 17 { dec!(-3) } else { dec!(1) };
                p
            })
            .collect();
        let mut s = strat(TrendPullbackConfig::default());
        let signals: Vec<(usize, Signal)> = bars(&closes)
            .iter()
            .enumerate()
            .flat_map(|(i, b)| s.on_bar(&sym(), b).into_iter().map(move |x| (i, x)))
            .collect();
        assert_eq!(signals.first().map(|(i, _)| *i), Some(199));
    }

    #[test]
    fn keeps_separate_state_per_symbol() {
        let mut s = strat(small());
        let other = Symbol("NSE:OTHER".into());
        for b in bars(&dip_history()[..27]) {
            assert!(s.on_bar(&sym(), &b).is_empty());
            assert!(s.on_bar(&other, &b).is_empty());
        }
        let last = bars(&dip_history())[27].clone();
        assert_eq!(s.on_bar(&sym(), &last).len(), 1);
        assert_eq!(s.on_bar(&other, &last).len(), 1);
    }

    #[test]
    fn no_look_ahead() {
        assert_no_look_ahead(|| strat(small()), &dip_history(), &[dec!(145), dec!(150)], &[dec!(100), dec!(90)]);
    }
}

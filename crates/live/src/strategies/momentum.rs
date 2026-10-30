//! (c) 1m momentum burst (crypto). Pre-registered 2026-10-30; do not tune.
//!
//! r5 = close / close 5 bars ago - 1. Long when r5 > 3 x the sample stdev of the PREVIOUS 100
//! r5 values and volume > 2 x the mean volume of the PREVIOUS 100 bars (the current bar is not
//! in its own baseline). Exit after 10 bars held (fills at the next open); stop = signal close -
//! 1 x ATR(14).

use std::collections::VecDeque;

use rust_decimal::Decimal;
use tq_core::{Bar, Fill, Symbol};
use tq_indicators::Atr;
use tq_strategy::{Signal, Strategy};

use super::{long_entry, to_f64, Holding, Window};

pub const RET_BARS: usize = 5;
pub const BASELINE_BARS: usize = 100;
pub const RET_STDEVS: f64 = 3.0;
pub const VOLUME_MULT: f64 = 2.0;
pub const HOLD_BARS: u32 = 10;
pub const ATR_BARS: usize = 14;
pub const STOP_ATRS: Decimal = Decimal::ONE;

#[derive(Debug, Clone)]
pub struct MomentumBurst {
    closes: VecDeque<Decimal>,
    rets: Window,
    vols: Window,
    atr: Atr,
    hold: Holding,
}

impl MomentumBurst {
    pub fn new() -> Option<Self> {
        Some(Self {
            closes: VecDeque::with_capacity(RET_BARS + 2),
            rets: Window::new(BASELINE_BARS),
            vols: Window::new(BASELINE_BARS),
            atr: Atr::new(ATR_BARS)?,
            hold: Holding::default(),
        })
    }

    fn five_bar_return(&mut self, close: Decimal) -> Option<f64> {
        self.closes.push_back(close);
        if self.closes.len() > RET_BARS + 1 {
            self.closes.pop_front();
        }
        let first = *self.closes.front()?;
        (self.closes.len() == RET_BARS + 1 && first > Decimal::ZERO).then(|| to_f64(close / first) - 1.0)
    }

    fn is_burst(&self, r5: f64, volume: f64) -> bool {
        if !self.rets.is_full() || !self.vols.is_full() {
            return false;
        }
        let (Some(sd), Some(avg_vol)) = (self.rets.stdev(), self.vols.mean()) else { return false };
        sd > 0.0 && r5 > RET_STDEVS * sd && volume > VOLUME_MULT * avg_vol
    }
}

impl Strategy for MomentumBurst {
    fn name(&self) -> &str {
        "momentum_burst"
    }

    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal> {
        let atr = self.atr.update(bar.high, bar.low, bar.close);
        let r5 = self.five_bar_return(bar.close);
        let volume = to_f64(bar.volume);
        let signals = if self.hold.in_position {
            if self.hold.tick() >= HOLD_BARS { vec![Signal::Exit(symbol.clone())] } else { Vec::new() }
        } else {
            match (r5, atr) {
                (Some(r), Some(a)) if self.is_burst(r, volume) => long_entry(symbol, bar.close, bar.close - STOP_ATRS * a),
                _ => Vec::new(),
            }
        };
        if let Some(r) = r5 {
            self.rets.push(r);
        }
        self.vols.push(volume);
        signals
    }

    fn on_fill(&mut self, fill: &Fill) {
        self.hold.on_fill(fill.side);
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{bar, fill, sym};
    use super::*;
    use rust_decimal_macros::dec;
    use tq_core::Side;

    fn warm(s: &mut MomentumBurst) {
        for i in 0..120 {
            let p = if i % 2 == 0 { dec!(100) } else { dec!(100.1) };
            assert!(s.on_bar(&sym(), &bar(i, p, p + dec!(0.05), p - dec!(0.05), p, dec!(10))).is_empty());
        }
    }

    #[test]
    fn burst_with_volume_enters_with_one_atr_stop() {
        let mut s = MomentumBurst::new().unwrap();
        warm(&mut s);
        let sig = s.on_bar(&sym(), &bar(120, dec!(100), dec!(102), dec!(100), dec!(102), dec!(50)));
        let Some(Signal::Enter(o)) = sig.first() else { panic!("expected entry, got {sig:?}") };
        assert_eq!(o.entry, dec!(102));
        assert!(o.stop < dec!(102) && o.stop > dec!(100));
    }

    #[test]
    fn burst_without_volume_is_ignored() {
        let mut s = MomentumBurst::new().unwrap();
        warm(&mut s);
        assert!(s.on_bar(&sym(), &bar(120, dec!(100), dec!(102), dec!(100), dec!(102), dec!(15))).is_empty());
    }

    #[test]
    fn exits_after_ten_bars_held() {
        let mut s = MomentumBurst::new().unwrap();
        warm(&mut s);
        s.on_fill(&fill(Side::Buy, dec!(100)));
        for i in 0..9 {
            assert!(s.on_bar(&sym(), &bar(200 + i, dec!(100), dec!(100), dec!(100), dec!(100), dec!(10))).is_empty());
        }
        assert_eq!(s.on_bar(&sym(), &bar(209, dec!(100), dec!(100), dec!(100), dec!(100), dec!(10))), vec![Signal::Exit(sym())]);
    }
}

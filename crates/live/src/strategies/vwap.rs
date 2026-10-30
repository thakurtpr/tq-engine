//! (a) Session-VWAP reversion on 1m bars. Pre-registered 2026-10-30; do not tune.
//!
//! Session VWAP = cumulative (typical price x volume) / cumulative volume since the session day
//! began (UTC midnight for crypto, IST day for NSE), typical price = (high + low + close) / 3.
//! Long when close < VWAP - 2 x stdev(last 20 closes); exit when close >= VWAP or after 30 bars
//! held (the exit fills at the next open); stop = signal close - 1.5 x ATR(14).

use rust_decimal::Decimal;
use tq_core::{Bar, Fill, Symbol};
use tq_indicators::Atr;
use tq_strategy::{Signal, Strategy};

use super::{from_f64, long_entry, to_f64, Holding, Window};
use crate::session::Session;

pub const STDEV_BARS: usize = 20;
pub const ENTRY_STDEVS: f64 = 2.0;
pub const MAX_HOLD_BARS: u32 = 30;
pub const ATR_BARS: usize = 14;
pub const STOP_ATRS: Decimal = Decimal::from_parts(15, 0, 0, false, 1);
const THREE: Decimal = Decimal::from_parts(3, 0, 0, false, 0);

#[derive(Debug, Clone)]
pub struct VwapReversion {
    session: Session,
    day: Option<i64>,
    cum_pv: Decimal,
    cum_v: Decimal,
    closes: Window,
    atr: Atr,
    hold: Holding,
}

impl VwapReversion {
    pub fn new(session: Session) -> Option<Self> {
        Some(Self {
            session,
            day: None,
            cum_pv: Decimal::ZERO,
            cum_v: Decimal::ZERO,
            closes: Window::new(STDEV_BARS),
            atr: Atr::new(ATR_BARS)?,
            hold: Holding::default(),
        })
    }

    fn update_vwap(&mut self, bar: &Bar) -> Option<Decimal> {
        let day = self.session.day_key(bar.ts);
        if self.day != Some(day) {
            self.day = Some(day);
            self.cum_pv = Decimal::ZERO;
            self.cum_v = Decimal::ZERO;
        }
        let typical = (bar.high + bar.low + bar.close) / THREE;
        self.cum_pv += typical * bar.volume;
        self.cum_v += bar.volume;
        (self.cum_v > Decimal::ZERO).then(|| self.cum_pv / self.cum_v)
    }

    fn entry(&self, symbol: &Symbol, close: Decimal, vwap: Decimal, atr: Decimal) -> Vec<Signal> {
        let Some(sd) = self.closes.stdev().filter(|_| self.closes.is_full()) else { return Vec::new() };
        let Some(band) = from_f64(to_f64(vwap) - ENTRY_STDEVS * sd) else { return Vec::new() };
        if sd > 0.0 && close < band {
            return long_entry(symbol, close, close - STOP_ATRS * atr);
        }
        Vec::new()
    }
}

impl Strategy for VwapReversion {
    fn name(&self) -> &str {
        "vwap_reversion"
    }

    fn on_bar(&mut self, symbol: &Symbol, bar: &Bar) -> Vec<Signal> {
        let vwap = self.update_vwap(bar);
        let atr = self.atr.update(bar.high, bar.low, bar.close);
        self.closes.push(to_f64(bar.close));
        if self.hold.in_position {
            let held = self.hold.tick();
            let reverted = vwap.is_some_and(|v| bar.close >= v);
            return if reverted || held >= MAX_HOLD_BARS { vec![Signal::Exit(symbol.clone())] } else { Vec::new() };
        }
        match (vwap, atr) {
            (Some(v), Some(a)) => self.entry(symbol, bar.close, v, a),
            _ => Vec::new(),
        }
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

    const M: i64 = 60_000;

    fn flat(s: &mut VwapReversion, n: i64) {
        for i in 0..n {
            let p = if i % 2 == 0 { dec!(100) } else { dec!(100.2) };
            assert!(s.on_bar(&sym(), &bar(i * M, p, p + dec!(0.1), p - dec!(0.1), p, dec!(10))).is_empty());
        }
    }

    #[test]
    fn enters_below_lower_band_with_atr_stop() {
        let mut s = VwapReversion::new(Session::Crypto24x7).unwrap();
        flat(&mut s, 30);
        let sig = s.on_bar(&sym(), &bar(30 * M, dec!(100), dec!(100), dec!(98), dec!(98), dec!(10)));
        let Some(Signal::Enter(o)) = sig.first() else { panic!("expected entry, got {sig:?}") };
        assert_eq!(o.entry, dec!(98));
        assert!(o.stop < dec!(98));
    }

    #[test]
    fn exits_on_reversion_to_vwap() {
        let mut s = VwapReversion::new(Session::Crypto24x7).unwrap();
        flat(&mut s, 30);
        s.on_fill(&fill(Side::Buy, dec!(98)));
        let sig = s.on_bar(&sym(), &bar(31 * M, dec!(99), dec!(101), dec!(99), dec!(101), dec!(10)));
        assert_eq!(sig, vec![Signal::Exit(sym())]);
    }

    #[test]
    fn exits_after_thirty_bars_held() {
        let mut s = VwapReversion::new(Session::Crypto24x7).unwrap();
        flat(&mut s, 30);
        s.on_fill(&fill(Side::Buy, dec!(90)));
        let low = |i: i64| bar((31 + i) * M, dec!(90), dec!(90.1), dec!(89.9), dec!(90), dec!(10));
        for i in 0..29 {
            assert!(s.on_bar(&sym(), &low(i)).is_empty(), "bar {i}");
        }
        assert_eq!(s.on_bar(&sym(), &low(29)), vec![Signal::Exit(sym())]);
    }

    #[test]
    fn vwap_resets_each_session_day() {
        let mut s = VwapReversion::new(Session::Crypto24x7).unwrap();
        s.update_vwap(&bar(0, dec!(10), dec!(10), dec!(10), dec!(10), dec!(1)));
        let v = s.update_vwap(&bar(86_400_000, dec!(20), dec!(20), dec!(20), dec!(20), dec!(1)));
        assert_eq!(v, Some(dec!(20)));
    }
}

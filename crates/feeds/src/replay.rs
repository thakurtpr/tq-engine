//! Historical replay: feeds stored bars as if each arrived exactly at its close.

use std::collections::VecDeque;

use tq_core::{Bar, Symbol};

use crate::{events_for, BarEvent, BarStream, FeedError, FeedEvent, GapTracker, Source};

#[derive(Debug)]
pub struct ReplayStream {
    symbol: Symbol,
    interval_ms: i64,
    bars: std::vec::IntoIter<Bar>,
    gaps: GapTracker,
    queue: VecDeque<FeedEvent>,
}

impl ReplayStream {
    /// `bars` must be strictly time-ordered (`tq_data::IntradayCsv` guarantees it).
    pub fn new(symbol: Symbol, interval_ms: i64, bars: Vec<Bar>) -> Result<Self, FeedError> {
        if interval_ms <= 0 {
            return Err(FeedError::Config("interval must be positive".into()));
        }
        if bars.windows(2).any(|w| w[1].ts <= w[0].ts) {
            return Err(FeedError::Config("replay bars are not strictly time-ordered".into()));
        }
        let gaps = GapTracker::new(interval_ms);
        Ok(Self { symbol, interval_ms, bars: bars.into_iter(), gaps, queue: VecDeque::new() })
    }
}

impl BarStream for ReplayStream {
    fn next_event(&mut self) -> Result<Option<FeedEvent>, FeedError> {
        while self.queue.is_empty() {
            let Some(bar) = self.bars.next() else { return Ok(None) };
            let received_ms = bar.ts + self.interval_ms;
            let ev = BarEvent { symbol: self.symbol.clone(), bar, interval_ms: self.interval_ms, received_ms, source: Source::Replay };
            self.queue.extend(events_for(&mut self.gaps, ev));
        }
        Ok(self.queue.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MINUTE_MS;
    use rust_decimal_macros::dec;

    fn bar(min: i64) -> Bar {
        Bar::new(min * MINUTE_MS, dec!(10), dec!(11), dec!(9), dec!(10), dec!(1)).unwrap()
    }

    #[test]
    fn replays_bars_on_time_and_reports_gaps() {
        let mut s = ReplayStream::new(Symbol("X".into()), MINUTE_MS, vec![bar(0), bar(1), bar(4)]).unwrap();
        let mut out = Vec::new();
        while let Some(e) = s.next_event().unwrap() {
            out.push(e);
        }
        assert_eq!(out.len(), 4);
        assert!(matches!(&out[0], FeedEvent::Bar(b) if b.lateness_ms() == 0 && b.source == Source::Replay));
        assert_eq!(out[2], FeedEvent::Gap { expected_ts: 2 * MINUTE_MS, got_ts: 4 * MINUTE_MS, missing_bars: 2 });
    }

    #[test]
    fn rejects_unordered_bars() {
        assert!(ReplayStream::new(Symbol("X".into()), MINUTE_MS, vec![bar(1), bar(0)]).is_err());
    }
}

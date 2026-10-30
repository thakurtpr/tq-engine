//! Market-data bar streams for the paper runner. Public data only: no accounts, keys or orders.
//!
//! A [`BarStream`] yields [`FeedEvent`]s: closed bars, detected gaps and status lines. Two
//! sources implement it: [`ReplayStream`] (historical files) and [`binance::BinanceLive`]
//! (public kline websocket, REST polling fallback). Every bar carries the time it was received,
//! so the consumer can refuse to trade on a stale bar.

pub mod binance;
mod replay;

pub use replay::ReplayStream;

use thiserror::Error;
use tq_core::{Bar, Symbol, Timestamp};

pub const MINUTE_MS: i64 = 60_000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FeedError {
    #[error("network: {0}")]
    Network(String),
    #[error("cannot parse feed message: {0}")]
    Parse(String),
    #[error("invalid feed config: {0}")]
    Config(String),
}

/// Where a bar came from. Backfilled bars arrive late by construction and are never tradeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Replay,
    Websocket,
    Rest,
    Backfill,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Replay => "replay",
            Source::Websocket => "websocket",
            Source::Rest => "rest",
            Source::Backfill => "backfill",
        }
    }
}

/// One closed bar. `bar.ts` is the bar's OPEN time; it closes at `bar.ts + interval_ms`.
#[derive(Debug, Clone, PartialEq)]
pub struct BarEvent {
    pub symbol: Symbol,
    pub bar: Bar,
    pub interval_ms: i64,
    pub received_ms: Timestamp,
    pub source: Source,
}

impl BarEvent {
    pub fn close_ts(&self) -> Timestamp {
        self.bar.ts + self.interval_ms
    }

    /// How long after the bar closed it reached us.
    pub fn lateness_ms(&self) -> i64 {
        self.received_ms - self.close_ts()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FeedEvent {
    Bar(BarEvent),
    /// Bars between `expected_ts` (inclusive) and `got_ts` (exclusive) never arrived.
    Gap { expected_ts: Timestamp, got_ts: Timestamp, missing_bars: i64 },
    Status(String),
}

/// A source of bar events. `Ok(None)` means the stream has ended (replays only).
pub trait BarStream {
    fn next_event(&mut self) -> Result<Option<FeedEvent>, FeedError>;
}

/// What [`GapTracker::observe`] decided about a bar's timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// Not newer than the last bar: drop it (e.g. overlap after a reconnect).
    Duplicate,
    /// A new bar, with `(expected_ts, missing_bars)` when bars were skipped.
    New(Option<(Timestamp, i64)>),
}

/// Detects skipped and repeated bars on a fixed interval.
#[derive(Debug, Clone)]
pub struct GapTracker {
    interval_ms: i64,
    last: Option<Timestamp>,
}

impl GapTracker {
    pub fn new(interval_ms: i64) -> Self {
        Self { interval_ms, last: None }
    }

    pub fn last(&self) -> Option<Timestamp> {
        self.last
    }

    pub fn observe(&mut self, ts: Timestamp) -> Seen {
        let gap = match self.last {
            Some(last) if ts <= last => return Seen::Duplicate,
            Some(last) if ts > last + self.interval_ms => {
                let expected = last + self.interval_ms;
                Some((expected, (ts - expected) / self.interval_ms))
            }
            _ => None,
        };
        self.last = Some(ts);
        Seen::New(gap)
    }
}

/// Turns a bar into events, prefixed with a gap event when bars were skipped.
pub(crate) fn events_for(gaps: &mut GapTracker, ev: BarEvent) -> Vec<FeedEvent> {
    match gaps.observe(ev.bar.ts) {
        Seen::Duplicate => Vec::new(),
        Seen::New(None) => vec![FeedEvent::Bar(ev)],
        Seen::New(Some((expected_ts, missing_bars))) => {
            vec![FeedEvent::Gap { expected_ts, got_ts: ev.bar.ts, missing_bars }, FeedEvent::Bar(ev)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contiguous_bars_have_no_gap() {
        let mut g = GapTracker::new(MINUTE_MS);
        assert_eq!(g.observe(0), Seen::New(None));
        assert_eq!(g.observe(MINUTE_MS), Seen::New(None));
    }

    #[test]
    fn skipped_bars_are_reported_with_count() {
        let mut g = GapTracker::new(MINUTE_MS);
        g.observe(0);
        assert_eq!(g.observe(4 * MINUTE_MS), Seen::New(Some((MINUTE_MS, 3))));
    }

    #[test]
    fn repeated_or_older_bars_are_duplicates() {
        let mut g = GapTracker::new(MINUTE_MS);
        g.observe(MINUTE_MS);
        assert_eq!(g.observe(MINUTE_MS), Seen::Duplicate);
        assert_eq!(g.observe(0), Seen::Duplicate);
        assert_eq!(g.last(), Some(MINUTE_MS));
    }
}

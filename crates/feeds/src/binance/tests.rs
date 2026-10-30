//! Offline tests of the live feed's warm-up, dedup, reconnect, backfill and REST fallback.

use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::*;

const M: i64 = MINUTE_MS;

#[derive(Clone)]
struct FakeClock(Rc<Cell<i64>>);

impl WallClock for FakeClock {
    fn now_ms(&self) -> Timestamp {
        self.0.get()
    }
    fn sleep_ms(&mut self, ms: u64) {
        self.0.set(self.0.get() + ms as i64);
    }
}

fn kline_json(minute: i64, closed: bool) -> String {
    format!(
        r#"{{"e":"kline","k":{{"t":{},"T":{},"o":"100","h":"101","l":"99","c":"100","v":"1","x":{closed}}}}}"#,
        minute * M,
        minute * M + M - 1
    )
}

/// REST returns every minute from `start` up to and including the forming one.
struct FakeRest {
    clock: FakeClock,
    calls: usize,
    fail_from_call: usize,
}

impl RestKlines for FakeRest {
    fn klines(&mut self, _s: &str, _i: &str, start_ms: Timestamp, limit: u32) -> Result<String, FeedError> {
        self.calls += 1;
        if self.calls >= self.fail_from_call {
            return Err(FeedError::Network("HTTP 418".into()));
        }
        let now = self.clock.now_ms();
        let first = (start_ms + M - 1).div_euclid(M);
        let rows: Vec<String> = (first..=now.div_euclid(M))
            .take(limit as usize)
            .map(|m| format!(r#"[{},"100","101","99","100","1",{},"0",1,"0","0","0"]"#, m * M, m * M + M - 1))
            .collect();
        Ok(format!("[{}]", rows.join(",")))
    }
}

/// Each connect pops a script: `None` fails to connect; `Some` yields its messages (each first
/// moving the clock to the given time), then drops the connection. An empty message drops it
/// after moving the clock.
struct FakeWs {
    clock: FakeClock,
    scripts: VecDeque<Option<Vec<(i64, String)>>>,
}

struct FakeConn {
    clock: FakeClock,
    msgs: VecDeque<(i64, String)>,
}

impl WsConn for FakeConn {
    fn next_text(&mut self) -> Result<Option<String>, FeedError> {
        let (at, text) = self.msgs.pop_front().ok_or_else(|| FeedError::Network("eof".into()))?;
        self.clock.0.set(at);
        if text.is_empty() {
            return Err(FeedError::Network("connection reset".into()));
        }
        Ok(Some(text))
    }
}

impl WsConnector for FakeWs {
    fn connect(&mut self, _url: &str) -> Result<Box<dyn WsConn>, FeedError> {
        match self.scripts.pop_front().flatten() {
            Some(msgs) => Ok(Box::new(FakeConn { clock: self.clock.clone(), msgs: msgs.into() })),
            None => Err(FeedError::Network("refused".into())),
        }
    }
}

fn feed(now: i64, scripts: Vec<Option<Vec<(i64, String)>>>, fail_from_call: usize) -> BinanceLive<FakeWs, FakeRest, FakeClock> {
    let clock = FakeClock(Rc::new(Cell::new(now)));
    let mut cfg = LiveConfig::binance_1m("btcusdt").unwrap();
    cfg.warmup_bars = 5;
    let rest = FakeRest { clock: clock.clone(), calls: 0, fail_from_call };
    let ws = FakeWs { clock: clock.clone(), scripts: scripts.into() };
    BinanceLive::new(cfg, ws, rest, clock)
}

/// Collects events until `n` bars were seen.
fn run(f: &mut impl BarStream, n: usize) -> (Vec<(i64, Source)>, Vec<FeedEvent>) {
    let (mut bars, mut other) = (Vec::new(), Vec::new());
    for _ in 0..10_000 {
        match f.next_event().unwrap().unwrap() {
            FeedEvent::Bar(b) => bars.push((b.bar.ts / M, b.source)),
            e => other.push(e),
        }
        if bars.len() == n {
            break;
        }
    }
    (bars, other)
}

fn msg(minute: i64, closed: bool) -> (i64, String) {
    (minute * M + M + 500, kline_json(minute, closed))
}

#[test]
fn config_builds_public_stream_url_and_rejects_bad_symbols() {
    assert_eq!(LiveConfig::binance_1m("btcusdt").unwrap().ws_url(), "wss://stream.binance.com:9443/ws/btcusdt@kline_1m");
    assert!(LiveConfig::binance_1m("btc/usdt").is_err());
}

#[test]
fn warms_up_over_rest_then_streams_closed_ws_bars_without_duplicates() {
    let script = vec![msg(9, true), msg(10, false), msg(10, true), msg(11, true)];
    let mut f = feed(10 * M + 1_000, vec![Some(script)], usize::MAX);
    let (bars, _) = run(&mut f, 7);
    let want: Vec<(i64, Source)> = (5..10).map(|m| (m, Source::Backfill)).chain([(10, Source::Websocket), (11, Source::Websocket)]).collect();
    assert_eq!(bars, want);
}

#[test]
fn reconnect_backfills_missed_bars_as_untradeable_backfill() {
    let first = vec![msg(10, true), (14 * M + 500, String::new())];
    let second = vec![msg(14, true)];
    let mut f = feed(10 * M + 1_000, vec![Some(first), Some(second)], usize::MAX);
    let (bars, other) = run(&mut f, 10);
    assert_eq!(&bars[5..], &[(10, Source::Websocket), (11, Source::Backfill), (12, Source::Backfill), (13, Source::Backfill), (14, Source::Websocket)]);
    assert!(other.iter().any(|e| matches!(e, FeedEvent::Status(s) if s.contains("websocket dropped"))));
}

#[test]
fn ws_gap_without_backfill_is_reported() {
    let mut f = feed(10 * M + 1_000, vec![Some(vec![msg(10, true), msg(14, true)])], usize::MAX);
    let (_, other) = run(&mut f, 7);
    assert!(other.contains(&FeedEvent::Gap { expected_ts: 11 * M, got_ts: 14 * M, missing_bars: 3 }));
}

#[test]
fn repeated_ws_failures_fall_back_to_rest_polling() {
    let mut f = feed(10 * M + 1_000, vec![None, None, None], usize::MAX);
    let (bars, other) = run(&mut f, 6);
    assert_eq!(bars[5], (10, Source::Rest));
    assert!(other.iter().any(|e| matches!(e, FeedEvent::Status(s) if s.contains("polling REST"))));
}

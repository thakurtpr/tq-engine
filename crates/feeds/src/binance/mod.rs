//! Binance PUBLIC kline feed: no account, no API key, no order endpoints.
//!
//! Primary: the websocket `wss://stream.binance.com:9443/ws/<sym>@kline_<i>`, closed klines only
//! (`x == true`). On start, and after every reconnect, missing bars are backfilled from the
//! public REST endpoint `/api/v3/klines`; they are tagged [`Source::Backfill`] and arrive late,
//! so the runner never trades on them. Websocket failures back off exponentially; after
//! `ws_failures_before_rest` consecutive failures the feed polls REST for `rest_fallback_ms`,
//! then tries the websocket again. Errors never end the stream: they become status events.

mod net;
mod parse;
#[cfg(test)]
mod tests;

pub use net::{SystemClock, TungsteniteWs, UreqRest};
pub use parse::{rest_klines, ws_kline, Kline};

use std::collections::VecDeque;

use tq_core::{Symbol, Timestamp};

use crate::{events_for, BarEvent, BarStream, FeedError, FeedEvent, GapTracker, Source, MINUTE_MS};

/// Most rows Binance returns per REST klines request.
pub const REST_PAGE: u32 = 1000;
const MAX_BACKFILL_PAGES: usize = 5;
const FIRST_BACKOFF_MS: u64 = 1_000;

/// An open websocket. `Ok(None)` is a control frame (ping/pong) with no data.
pub trait WsConn {
    fn next_text(&mut self) -> Result<Option<String>, FeedError>;
}

pub trait WsConnector {
    fn connect(&mut self, url: &str) -> Result<Box<dyn WsConn>, FeedError>;
}

/// Raw body of `GET /api/v3/klines?symbol=..&interval=..&startTime=..&limit=..`.
pub trait RestKlines {
    fn klines(&mut self, symbol: &str, interval: &str, start_ms: Timestamp, limit: u32) -> Result<String, FeedError>;
}

pub trait WallClock {
    fn now_ms(&self) -> Timestamp;
    fn sleep_ms(&mut self, ms: u64);
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveConfig {
    /// Upper-case Binance symbol, e.g. BTCUSDT.
    pub symbol: String,
    pub interval: String,
    pub interval_ms: i64,
    pub ws_base: String,
    /// Bars fetched over REST before going live, to warm indicators (session VWAP needs the day).
    pub warmup_bars: i64,
    pub rest_poll_ms: u64,
    pub max_backoff_ms: u64,
    pub ws_failures_before_rest: u32,
    pub rest_fallback_ms: i64,
}

impl LiveConfig {
    pub fn binance_1m(symbol: &str) -> Result<Self, FeedError> {
        let symbol = symbol.trim().to_ascii_uppercase();
        if symbol.len() < 2 || !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(FeedError::Config(format!("invalid Binance symbol {symbol:?}")));
        }
        Ok(Self {
            symbol,
            interval: "1m".into(),
            interval_ms: MINUTE_MS,
            ws_base: "wss://stream.binance.com:9443".into(),
            warmup_bars: 1_500,
            rest_poll_ms: 5_000,
            max_backoff_ms: 60_000,
            ws_failures_before_rest: 3,
            rest_fallback_ms: 10 * MINUTE_MS,
        })
    }

    pub fn ws_url(&self) -> String {
        format!("{}/ws/{}@kline_{}", self.ws_base, self.symbol.to_ascii_lowercase(), self.interval)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Ws,
    Rest { until: Timestamp },
}

pub struct BinanceLive<W, R, C> {
    cfg: LiveConfig,
    ws: W,
    rest: R,
    clock: C,
    conn: Option<Box<dyn WsConn>>,
    queue: VecDeque<FeedEvent>,
    gaps: GapTracker,
    failures: u32,
    mode: Mode,
    warmed: bool,
    symbol: Symbol,
}

impl<W: WsConnector, R: RestKlines, C: WallClock> BinanceLive<W, R, C> {
    pub fn new(cfg: LiveConfig, ws: W, rest: R, clock: C) -> Self {
        let symbol = Symbol(cfg.symbol.clone());
        let gaps = GapTracker::new(cfg.interval_ms);
        Self { cfg, ws, rest, clock, conn: None, queue: VecDeque::new(), gaps, failures: 0, mode: Mode::Ws, warmed: false, symbol }
    }

    fn status(&mut self, msg: String) {
        self.queue.push_back(FeedEvent::Status(msg));
    }

    fn accept(&mut self, k: Kline, source: Source) {
        if !k.closed {
            return;
        }
        let ev = BarEvent { symbol: self.symbol.clone(), bar: k.bar, interval_ms: self.cfg.interval_ms, received_ms: self.clock.now_ms(), source };
        let events = events_for(&mut self.gaps, ev);
        self.queue.extend(events);
    }

    /// Fetches closed bars from `start` onward over REST, tagged `source`. Returns how many.
    fn fetch_rest(&mut self, mut start: Timestamp, source: Source) -> usize {
        let mut n = 0;
        for _ in 0..MAX_BACKFILL_PAGES {
            let fetched = self.rest.klines(&self.cfg.symbol, &self.cfg.interval, start, REST_PAGE);
            let rows = match fetched.and_then(|body| rest_klines(&body, self.clock.now_ms())) {
                Ok(rows) => rows,
                Err(e) => {
                    self.status(format!("REST klines failed: {e}"));
                    break;
                }
            };
            let full_page = rows.len() as u32 >= REST_PAGE;
            let next = rows.last().map(|k| k.bar.ts + self.cfg.interval_ms);
            let closed: Vec<Kline> = rows.into_iter().filter(|k| k.closed).collect();
            n += closed.len();
            closed.into_iter().for_each(|k| self.accept(k, source));
            match next {
                Some(s) if full_page => start = s,
                _ => break,
            }
        }
        n
    }

    fn warm_up(&mut self) {
        let iv = self.cfg.interval_ms;
        let start = (self.clock.now_ms() - self.cfg.warmup_bars * iv).div_euclid(iv) * iv;
        let n = self.fetch_rest(start, Source::Backfill);
        self.warmed = true;
        self.status(format!("warm-up: {n} closed bars backfilled over REST"));
    }

    fn backoff_ms(&self) -> u64 {
        let shift = self.failures.saturating_sub(1).min(16);
        (FIRST_BACKOFF_MS << shift).min(self.cfg.max_backoff_ms)
    }

    fn fail(&mut self, msg: String) {
        self.failures += 1;
        self.status(msg);
        if self.failures >= self.cfg.ws_failures_before_rest {
            let until = self.clock.now_ms() + self.cfg.rest_fallback_ms;
            self.mode = Mode::Rest { until };
            self.status(format!("{} websocket failures: polling REST until {until}", self.failures));
        } else {
            let wait = self.backoff_ms();
            self.status(format!("reconnecting in {wait} ms"));
            self.clock.sleep_ms(wait);
        }
    }

    fn connect_ws(&mut self) {
        let url = self.cfg.ws_url();
        match self.ws.connect(&url) {
            Ok(c) => {
                self.conn = Some(c);
                self.status(format!("websocket connected: {url}"));
                if let Some(last) = self.gaps.last() {
                    let n = self.fetch_rest(last + self.cfg.interval_ms, Source::Backfill);
                    self.status(format!("reconnect backfill: {n} bars"));
                }
            }
            Err(e) => self.fail(format!("websocket connect failed: {e}")),
        }
    }

    fn pump_ws(&mut self) {
        let Some(conn) = self.conn.as_mut() else { return self.connect_ws() };
        match conn.next_text() {
            Ok(Some(text)) => match ws_kline(&text) {
                Ok(Some(k)) => {
                    self.failures = 0;
                    self.accept(k, Source::Websocket);
                }
                Ok(None) => {}
                Err(e) => self.status(format!("ignored bad websocket message: {e}")),
            },
            Ok(None) => {}
            Err(e) => {
                self.conn = None;
                self.fail(format!("websocket dropped: {e}"));
            }
        }
    }

    fn pump_rest(&mut self, until: Timestamp) {
        let now = self.clock.now_ms();
        if now >= until {
            self.mode = Mode::Ws;
            self.failures = 0;
            return self.status("REST fallback over: retrying websocket".into());
        }
        let start = self.gaps.last().map_or(now - 2 * self.cfg.interval_ms, |l| l + self.cfg.interval_ms);
        self.fetch_rest(start, Source::Rest);
        self.clock.sleep_ms(self.cfg.rest_poll_ms);
    }
}

impl<W: WsConnector, R: RestKlines, C: WallClock> BarStream for BinanceLive<W, R, C> {
    /// Blocks until an event is available. A live feed never ends and never returns `Err`.
    fn next_event(&mut self) -> Result<Option<FeedEvent>, FeedError> {
        loop {
            if let Some(e) = self.queue.pop_front() {
                return Ok(Some(e));
            }
            if !self.warmed {
                self.warm_up();
                continue;
            }
            match self.mode {
                Mode::Ws => self.pump_ws(),
                Mode::Rest { until } => self.pump_rest(until),
            }
        }
    }
}

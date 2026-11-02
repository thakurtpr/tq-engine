//! Gate 1 (spec section 2): can the recording rebuild the book well enough to simulate on?
//! Feed every line in file order to [`Gate1::observe`], then render [`Gate1::report`].

use std::collections::{BTreeMap, VecDeque};

use chrono::DateTime;

use crate::record::{Line, Record};
use crate::sync::{DepthEv, DepthOutcome, SnapCheck, SnapOutcome, SymbolSync};

const HOUR_MS: i64 = 3_600_000;
const MINUTE_MS: i64 = 60_000;
const SILENCE_MS: i64 = 1_000;
const MAX_TICKERS_AHEAD: usize = 4_096;
/// Spec 2.4 [convention]: minutes whose p99 feed latency exceeds this are excluded downstream.
pub const LATENCY_P99_LIMIT_MS: i64 = 500;
/// Spec addendum 9a: an offset probe older than this does not correct a minute's latency.
const PROBE_MAX_AGE_MS: i64 = 20 * MINUTE_MS;
pub const PASS_VERIFIED_SHARE: f64 = 0.95;
pub const PASS_BREAKS_PER_6H: f64 = 1.0;
pub const PASS_SNAPSHOT_L20: f64 = 0.995;

#[derive(Default)]
struct HourStats {
    events: u64,
    breaks: u64,
    verified_ms: i64,
}

#[derive(Default)]
struct SymStats {
    sync: SymbolSync,
    events: u64,
    stale: u64,
    breaks: Vec<i64>,
    resets_on_disconnect: u64,
    sync_failed: u64,
    verified_ms: i64,
    last_depth_rx: Option<i64>,
    silences: u64,
    crossed: u64,
    snaps_checked: u64,
    snaps_match: [u64; 3],
    snaps_full_match: u64,
    snaps_compared_min: Option<usize>,
    snaps_uncheckable: u64,
    /// bookTickers with an update id ahead of the book, waiting for the depth event ending at it.
    /// WHY: bookTicker is real time and depth is batched per 100 ms, so the two books agree only
    /// at the same update id; comparing at arrival measures timing skew, not book correctness.
    tickers_ahead: VecDeque<(u64, i64, i64)>,
    tickers_checked: u64,
    tickers_price_match: u64,
    /// minute -> rx - E samples (local clock, uncorrected)
    latency: BTreeMap<i64, Vec<i64>>,
    hours: BTreeMap<i64, HourStats>,
}

#[derive(Default)]
pub struct Gate1 {
    syms: BTreeMap<String, SymStats>,
    first_rx: Option<i64>,
    last_rx: i64,
    connects: u64,
    disconnects: u64,
    other: u64,
    unparsable: u64,
    /// (probe rx, offset = server - midpoint(tx, rx), round trip)
    probes: Vec<(i64, i64, i64)>,
}

fn pct(v: &mut [i64], q: f64) -> i64 {
    v.sort_unstable();
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn share(n: u64, d: u64) -> f64 {
    if d == 0 {
        f64::NAN
    } else {
        n as f64 / d as f64
    }
}

fn hour_label(hour: i64) -> String {
    DateTime::from_timestamp_millis(hour * HOUR_MS).map_or_else(|| hour.to_string(), |d| d.format("%Y-%m-%dT%H").to_string())
}

impl SymStats {
    fn depth(&mut self, ev: DepthEv) {
        let (rx, e) = (ev.rx, ev.e);
        self.events += 1;
        let hour = self.hours.entry(rx.div_euclid(HOUR_MS)).or_default();
        hour.events += 1;
        self.latency.entry(rx.div_euclid(MINUTE_MS)).or_default().push(rx - e);
        if self.last_depth_rx.is_some_and(|prev| rx - prev > SILENCE_MS) {
            self.silences += 1;
        }
        let was_synced = self.sync.is_synced();
        let (out, check) = self.sync.on_depth(ev);
        match out {
            DepthOutcome::Applied => {
                if let (true, Some(prev)) = (was_synced, self.last_depth_rx) {
                    self.verified_ms += rx - prev;
                    hour.verified_ms += rx - prev;
                }
                if self.sync.book().is_some_and(|b| b.crossed()) {
                    self.crossed += 1;
                }
                let last = self.sync.last_update();
                while self.tickers_ahead.front().is_some_and(|t| t.0 < last) {
                    self.tickers_ahead.pop_front();
                }
                if let Some(&(u, bid, ask)) = self.tickers_ahead.front() {
                    if u == last {
                        self.tickers_ahead.pop_front();
                        self.ticker_check(bid, ask);
                    }
                }
            }
            DepthOutcome::Break => {
                self.breaks.push(rx);
                hour.breaks += 1;
            }
            DepthOutcome::Stale => self.stale += 1,
            DepthOutcome::Buffered => {}
        }
        if let Some(c) = check {
            self.snapshot_result(c);
        }
        self.last_depth_rx = Some(rx);
    }

    fn ticker(&mut self, update: u64, bid: i64, ask: i64) {
        if !self.sync.is_synced() {
            return;
        }
        let last = self.sync.last_update();
        if update == last {
            self.ticker_check(bid, ask);
        } else if update > last {
            if self.tickers_ahead.len() == MAX_TICKERS_AHEAD {
                self.tickers_ahead.pop_front();
            }
            self.tickers_ahead.push_back((update, bid, ask));
        }
    }

    fn ticker_check(&mut self, bid: i64, ask: i64) {
        let Some(book) = self.sync.book() else { return };
        self.tickers_checked += 1;
        if book.best_bid().map(|b| b.0) == Some(bid) && book.best_ask().map(|a| a.0) == Some(ask) {
            self.tickers_price_match += 1;
        }
    }

    fn snapshot_result(&mut self, r: Option<SnapCheck>) {
        let Some(c) = r else {
            self.snaps_uncheckable += 1;
            return;
        };
        self.snaps_checked += 1;
        for (i, ok) in c.top.iter().enumerate() {
            self.snaps_match[i] += u64::from(*ok);
        }
        self.snaps_full_match += u64::from(c.full);
        self.snaps_compared_min = Some(self.snaps_compared_min.map_or(c.compared, |m| m.min(c.compared)));
    }

    fn reset_link(&mut self) {
        if self.sync.is_synced() {
            self.resets_on_disconnect += 1;
        }
        self.sync.reset();
        self.tickers_ahead.clear();
        self.last_depth_rx = None;
    }
}

pub struct Verdict {
    pub sym: String,
    pub verified_share: f64,
    pub breaks_per_6h: f64,
    pub snapshot_l20: f64,
    pub snapshot_full: f64,
}

impl Verdict {
    /// The spec's L20 rule, and (review-gate1-19) the same bar for every untouched level.
    pub fn passes(&self) -> bool {
        self.verified_share >= PASS_VERIFIED_SHARE
            && self.breaks_per_6h <= PASS_BREAKS_PER_6H
            && self.snapshot_l20 >= PASS_SNAPSHOT_L20
            && self.snapshot_full >= PASS_SNAPSHOT_L20
    }
}

impl Gate1 {
    pub fn unparsable(&mut self) {
        self.unparsable += 1;
    }

    pub fn first_rx(&self) -> Option<i64> {
        self.first_rx
    }

    pub fn observe(&mut self, line: Line) {
        self.first_rx.get_or_insert(line.rx);
        self.last_rx = self.last_rx.max(line.rx);
        let rx = line.rx;
        match line.rec {
            Record::Depth { sym, e, first, last, prev, bids, asks } => {
                self.syms.entry(sym).or_default().depth(DepthEv { rx, e, first, last, prev, bids, asks });
            }
            Record::Snapshot { sym, last_update, bids, asks } => {
                let s = self.syms.entry(sym).or_default();
                match s.sync.on_snapshot(last_update, &bids, &asks) {
                    SnapOutcome::Synced => s.last_depth_rx = Some(rx),
                    SnapOutcome::SyncFailed => s.sync_failed += 1,
                    SnapOutcome::Checked(r) => s.snapshot_result(r),
                    SnapOutcome::Deferred => {}
                }
            }
            Record::Ticker { sym, update, bid, ask, .. } => self.syms.entry(sym).or_default().ticker(update, bid, ask),
            Record::ServerTime { tx, server } => self.probes.push((rx, server - (tx + rx) / 2, rx - tx)),
            Record::Link { event } => {
                if event == "connect" {
                    self.connects += 1;
                }
                // A new connection (or a stop) cannot be chained to the old book.
                if matches!(event.as_str(), "connect" | "disconnect" | "stop") {
                    self.disconnects += u64::from(event == "disconnect");
                    self.syms.values_mut().for_each(SymStats::reset_link);
                }
            }
            Record::Trade { .. } => {}
            Record::Other => self.other += 1,
        }
    }

    fn span_ms(&self) -> i64 {
        self.first_rx.map_or(0, |f| self.last_rx - f)
    }

    /// (offset, round trip) of the lowest-round-trip probe in the 20 min before `t`.
    /// WHY: the offset is only known to +/- RTT/2, so the tightest recent probe is the best estimate.
    fn offset_at(&self, t: i64) -> Option<(i64, i64)> {
        self.probes.iter().filter(|p| p.0 <= t && t - p.0 <= PROBE_MAX_AGE_MS).min_by_key(|p| p.2).map(|p| (p.1, p.2))
    }

    pub fn verdicts(&self) -> Vec<Verdict> {
        let span = self.span_ms().max(1) as f64;
        self.syms
            .iter()
            .map(|(sym, s)| Verdict {
                sym: sym.clone(),
                verified_share: s.verified_ms as f64 / span,
                breaks_per_6h: s.breaks.len() as f64 / (span / (6 * HOUR_MS) as f64),
                snapshot_l20: share(s.snaps_match[2], s.snaps_checked),
                snapshot_full: share(s.snaps_full_match, s.snaps_checked),
            })
            .collect()
    }

    pub fn report(&self) -> String {
        let mut o = vec![
            format!(
                "- span: {:.2} h of wall clock; connects {}, disconnects {}",
                self.span_ms() as f64 / HOUR_MS as f64,
                self.connects,
                self.disconnects
            ),
            format!("- non-market lines: {} other, {} unparsable", self.other, self.unparsable),
        ];
        if !self.probes.is_empty() {
            let mut off: Vec<i64> = self.probes.iter().map(|p| p.1).collect();
            let mut rtt: Vec<i64> = self.probes.iter().map(|p| p.2).collect();
            let med = pct(&mut off, 0.5);
            o.push(format!(
                "- clock probes: {}; offset (server - local) median {med} ms, range {}..{} ms; round trip median {} ms",
                self.probes.len(),
                off[0],
                off[off.len() - 1],
                pct(&mut rtt, 0.5)
            ));
        }
        o.extend(self.integrity_table());
        o.extend(self.latency_table());
        o.extend(self.hourly_table());
        o.join("\n")
    }

    fn integrity_table(&self) -> Vec<String> {
        let mut o = vec![
            String::new(),
            "| symbol | depth events | verified time | breaks (per 6 h) | resets on reconnect | sync failed | stale | silences > 1 s | crossed | snapshots L1 / L5 / L20 / all untouched (checked, uncheckable, fewest levels compared) | bookTicker at same update id: price match | PASS |".into(),
            "|---|---|---|---|---|---|---|---|---|---|---|---|".into(),
        ];
        for (v, s) in self.verdicts().iter().zip(self.syms.values()) {
            let m = |i: usize| share(s.snaps_match[i], s.snaps_checked) * 100.0;
            o.push(format!(
                "| {} | {} | {:.2}% | {} ({:.2}) | {} | {} | {} | {} | {} | {:.1}% / {:.1}% / {:.1}% / {:.1}% ({}, {}, {}) | {:.2}% of {} | {} |",
                v.sym,
                s.events,
                v.verified_share * 100.0,
                s.breaks.len(),
                v.breaks_per_6h,
                s.resets_on_disconnect,
                s.sync_failed,
                s.stale,
                s.silences,
                s.crossed,
                m(0),
                m(1),
                m(2),
                v.snapshot_full * 100.0,
                s.snaps_checked,
                s.snaps_uncheckable,
                s.snaps_compared_min.map_or("n/a".to_string(), |n| n.to_string()),
                share(s.tickers_price_match, s.tickers_checked) * 100.0,
                s.tickers_checked,
                if v.passes() { "yes" } else { "**no**" }
            ));
        }
        o
    }

    fn latency_table(&self) -> Vec<String> {
        let mut o = vec![
            String::new(),
            "Feed latency `rx - E` per minute (raw = local clock; corrected = raw + offset from a probe at most 20 min old):".into(),
            String::new(),
            "| symbol | minutes | median of minute medians (raw / corrected) | worst minute p99 (raw) | minutes p99 > 500 ms (corrected) | minutes without a fresh probe |".into(),
            "|---|---|---|---|---|---|".into(),
        ];
        for (sym, s) in &self.syms {
            let (mut meds, mut cmeds, mut rtts, mut worst, mut slow, mut unprobed) = (vec![], vec![], vec![], i64::MIN, 0, 0);
            for (minute, samples) in &s.latency {
                let mut v = samples.clone();
                let (med, p99) = (pct(&mut v, 0.5), pct(&mut v, 0.99));
                meds.push(med);
                worst = worst.max(p99);
                match self.offset_at(minute * MINUTE_MS) {
                    Some((off, rtt)) => {
                        cmeds.push(med + off);
                        rtts.push(rtt);
                        slow += u64::from(p99 + off > LATENCY_P99_LIMIT_MS);
                    }
                    None => unprobed += 1,
                }
            }
            let cm = if cmeds.is_empty() {
                "n/a".to_string()
            } else {
                format!("{} (probe RTT median {} ms, so +/- {} ms)", pct(&mut cmeds, 0.5), pct(&mut rtts, 0.5), pct(&mut rtts, 0.5) / 2)
            };
            o.push(format!("| {sym} | {} | {} / {cm} ms | {worst} ms | {slow} | {unprobed} |", meds.len(), pct(&mut meds, 0.5)));
        }
        o
    }

    fn hourly_table(&self) -> Vec<String> {
        let mut o = vec![
            String::new(),
            "Per UTC hour:".into(),
            String::new(),
            "| hour | symbol | events | breaks | verified share of the hour |".into(),
            "|---|---|---|---|---|".into(),
        ];
        for (sym, s) in &self.syms {
            for (h, hs) in &s.hours {
                o.push(format!(
                    "| {} | {sym} | {} | {} | {:.1}% |",
                    hour_label(*h),
                    hs.events,
                    hs.breaks,
                    hs.verified_ms as f64 / HOUR_MS as f64 * 100.0
                ));
            }
        }
        o
    }
}

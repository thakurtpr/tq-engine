//! Replays a recording through many [`Sim`]s at once (spec addendum 9b ordering rules):
//! - exchange effects are ordered by exchange time (`E` for depth, `T` for aggTrade), ties depth
//!   before aggTrade, then line order, using a reorder buffer of `REORDER_MS`;
//! - the book is rebuilt from depth in that order (identical to file order within the depth socket);
//! - each decision is stamped with the depth event's arrival (`rx + offset`), so it sees only what had
//!   arrived; a trade that arrived later but happened earlier still acts on resting quotes at `T`.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap};

use crate::record::{Line, Record};
use crate::sim::{imbalance, Sim};
use crate::sync::{DepthEv, DepthOutcome, SymbolSync};

/// Events are released once the newest arrival is this far past them.
/// WHY: median feed delay is ~50-150 ms, but this retail link delivers bursts 2-14 s late (first
/// dev-slice hour: worst 13.6 s), and a 2 s window released 264 events out of exchange-time order.
/// 60 s per review-sim-20 (cost is memory only); any inversion still marks the sample untrusted.
const REORDER_MS: i64 = 60_000;
const PROBE_MAX_AGE_MS: i64 = 20 * 60_000;

struct SymbolRun {
    sync: SymbolSync,
    sims: Vec<Sim>,
    /// Volume traded since the previous depth event, keyed by (resting side is bid, price).
    traded: HashMap<(bool, i64), i64>,
}

#[derive(Default)]
pub struct Counters {
    pub released: u64,
    /// An event released with an exchange time before the previous one (reorder window too short),
    /// by rank: depth, snapshot/link, trade.
    pub order_violations: [u64; 3],
    /// Largest such inversion, ms.
    pub worst_inversion_ms: i64,
    pub trades_while_unsynced: u64,
    pub book_resets: u64,
    pub trade_gaps: u64,
}

pub struct Driver {
    syms: BTreeMap<String, SymbolRun>,
    heap: BinaryHeap<Reverse<(i64, u8, u64, usize)>>,
    lines: HashMap<usize, Line>,
    seq: u64,
    newest_rx: i64,
    last_released: i64,
    /// (rx, offset, round trip) per clock probe.
    probes: Vec<(i64, i64, i64)>,
    pub counters: Counters,
}

impl Driver {
    pub fn new(sims: BTreeMap<String, Vec<Sim>>) -> Self {
        let syms = sims.into_iter().map(|(s, sims)| (s, SymbolRun { sync: SymbolSync::default(), sims, traded: HashMap::new() })).collect();
        Driver {
            syms,
            heap: BinaryHeap::new(),
            lines: HashMap::new(),
            seq: 0,
            newest_rx: i64::MIN,
            last_released: i64::MIN,
            probes: vec![],
            counters: Counters::default(),
        }
    }

    /// Lowest-round-trip probe in the last 20 min, else the latest; 0 before any probe.
    fn offset(&self, t: i64) -> i64 {
        let recent = self.probes.iter().filter(|p| p.0 <= t && t - p.0 <= PROBE_MAX_AGE_MS).min_by_key(|p| p.2);
        recent.or(self.probes.last()).map_or(0, |p| p.1)
    }

    pub fn push(&mut self, line: Line) {
        if let Record::ServerTime { tx, server } = line.rec {
            self.probes.push((line.rx, server - (tx + line.rx) / 2, line.rx - tx));
            return;
        }
        let off = self.offset(line.rx);
        let (t, rank) = match &line.rec {
            Record::Depth { e, .. } => (*e, 0),
            Record::Trade { t, .. } => (*t, 2),
            Record::Snapshot { .. } | Record::Link { .. } => (line.rx + off, 1),
            _ => return,
        };
        self.newest_rx = self.newest_rx.max(line.rx);
        self.seq += 1;
        let id = self.seq as usize;
        self.lines.insert(id, line);
        self.heap.push(Reverse((t, rank, self.seq, id)));
        let horizon = self.newest_rx + self.offset(self.newest_rx) - REORDER_MS;
        while self.heap.peek().is_some_and(|Reverse(top)| top.0 <= horizon) {
            self.release_one();
        }
    }

    /// Release everything left and close every simulation.
    pub fn finish(mut self) -> (BTreeMap<String, Vec<Sim>>, Counters) {
        while !self.heap.is_empty() {
            self.release_one();
        }
        let mut out = BTreeMap::new();
        for (sym, mut run) in self.syms {
            run.sims.iter_mut().for_each(Sim::finish);
            out.insert(sym, run.sims);
        }
        (out, self.counters)
    }

    fn release_one(&mut self) {
        let Some(Reverse((t, rank, _, id))) = self.heap.pop() else { return };
        let line = self.lines.remove(&id).expect("every heap entry has a line");
        self.counters.released += 1;
        if t < self.last_released {
            self.counters.order_violations[rank as usize] += 1;
            self.counters.worst_inversion_ms = self.counters.worst_inversion_ms.max(self.last_released - t);
        }
        self.last_released = self.last_released.max(t);
        let known_at = line.rx + self.offset(line.rx);
        match line.rec {
            Record::Depth { sym, e, first, last, prev, bids, asks } => {
                if let Some(run) = self.syms.get_mut(&sym) {
                    let ev = DepthEv { rx: line.rx, e, first, last, prev, bids, asks };
                    run.depth(t, known_at, ev, &mut self.counters);
                }
            }
            Record::Trade { sym, px, qty, buyer_maker, .. } => {
                if let Some(run) = self.syms.get_mut(&sym) {
                    run.trade(t, known_at, px, qty, buyer_maker, &mut self.counters);
                }
            }
            Record::Snapshot { sym, last_update, bids, asks } => {
                if let Some(run) = self.syms.get_mut(&sym) {
                    run.sync.on_snapshot(last_update, &bids, &asks);
                }
            }
            rec @ Record::Link { .. } => {
                if rec.is_book_link() {
                    self.counters.book_resets += 1;
                    self.syms.values_mut().for_each(|r| {
                        r.sync.reset();
                        r.sims.iter_mut().for_each(Sim::pull);
                    });
                } else if rec.is_trade_link() {
                    // Trades may be missing until reconnect: quotes could fill unseen, so pull them.
                    self.counters.trade_gaps += 1;
                    self.syms.values_mut().for_each(|r| r.sims.iter_mut().for_each(Sim::pull));
                }
            }
            _ => {}
        }
    }
}

impl SymbolRun {
    fn depth(&mut self, t: i64, known_at: i64, ev: DepthEv, c: &mut Counters) {
        if let Some(book) = self.sync.book() {
            for s in &mut self.sims {
                s.activate(t, book);
                s.pre_depth(book);
            }
        }
        let (out, _) = self.sync.on_depth(ev);
        match out {
            DepthOutcome::Applied => {
                let book = self.sync.book().expect("applied implies synced");
                let traded = &self.traded;
                let lookup = |is_bid: bool, px: i64| traded.get(&(is_bid, px)).copied().unwrap_or(0);
                for s in &mut self.sims {
                    s.post_depth(t, known_at, book, &lookup);
                }
            }
            DepthOutcome::Break => {
                c.book_resets += 1;
                self.sims.iter_mut().for_each(Sim::pull);
            }
            DepthOutcome::Stale | DepthOutcome::Buffered => {}
        }
        self.traded.clear();
    }

    fn trade(&mut self, t: i64, known_at: i64, px: i64, qty: i64, buyer_maker: bool, c: &mut Counters) {
        let Some(book) = self.sync.book() else {
            c.trades_while_unsynced += 1;
            return;
        };
        let imb = imbalance(book);
        for s in &mut self.sims {
            s.activate(t, book);
            s.on_trade(t, known_at, px, qty, buyer_maker, imb);
        }
        *self.traded.entry((buyer_maker, px)).or_default() += qty;
    }
}

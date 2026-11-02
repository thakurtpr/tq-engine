//! Binance USD-M diff-depth synchronisation for one symbol (spec section 2, Binance's own rule):
//! after a snapshot with `lastUpdateId` L, drop events with `u < L`; the first applied event must have
//! `U <= L <= u`; every later event must have `pu == previous u`, else resync from the next snapshot.

use std::collections::{BTreeSet, VecDeque};

use crate::book::Book;
use crate::record::Levels;

/// Depth events kept while unsynced (~30 min at 10/s): a snapshot older than this cannot sync anyway.
const MAX_BUFFER: usize = 20_000;
/// Applied events remembered for checking a snapshot whose L is already behind the book.
const MAX_RECENT: usize = 256;
pub const CHECK_LEVELS: [usize; 3] = [1, 5, 20];

#[derive(Debug, Clone, PartialEq)]
pub struct DepthEv {
    pub rx: i64,
    pub e: i64,
    pub first: u64,
    pub last: u64,
    pub prev: u64,
    pub bids: Levels,
    pub asks: Levels,
}

impl DepthEv {
    fn touched(&self) -> impl Iterator<Item = i64> + '_ {
        self.bids.iter().chain(&self.asks).map(|l| l.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthOutcome {
    Applied,
    /// Older than the snapshot that started this segment.
    Stale,
    /// The update-id chain broke: the book is dropped until the next snapshot.
    Break,
    Buffered,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SnapOutcome {
    /// The snapshot (re)started a verified segment.
    Synced,
    /// Unsynced and the buffer does not reach back to L (or has a hole): wait for the next one.
    SyncFailed,
    /// Synced: comparison result per `CHECK_LEVELS`, or `None` when it cannot be compared.
    Checked(Option<[bool; 3]>),
    /// Synced and L is ahead of the book: compared when the covering event arrives.
    Deferred,
}

struct Pending {
    last_update: u64,
    bids: Levels,
    asks: Levels,
}

#[derive(Default)]
pub struct SymbolSync {
    book: Option<Book>,
    last_u: u64,
    /// Synced from a snapshot with nothing buffered: the next event must satisfy U <= L <= u.
    awaiting_cover: Option<u64>,
    buffer: VecDeque<DepthEv>,
    recent: VecDeque<DepthEv>,
    pending: Option<Pending>,
}

/// First `n` levels of `side` after removing `skip` prices; `side` is best-first.
fn visible(side: &[(i64, i64)], skip: &BTreeSet<i64>, n: usize) -> Levels {
    side.iter().filter(|l| !skip.contains(&l.0)).take(n).copied().collect()
}

fn compare(book: &Book, bids: &Levels, asks: &Levels, skip: &BTreeSet<i64>) -> [bool; 3] {
    let (bb, ba) = book.top(CHECK_LEVELS[2] + skip.len());
    CHECK_LEVELS.map(|n| visible(&bb, skip, n) == visible(bids, skip, n) && visible(&ba, skip, n) == visible(asks, skip, n))
}

impl SymbolSync {
    pub fn book(&self) -> Option<&Book> {
        self.book.as_ref()
    }

    pub fn is_synced(&self) -> bool {
        self.book.is_some()
    }

    pub fn last_update(&self) -> u64 {
        self.last_u
    }

    /// Link dropped: whatever arrives next cannot be chained to the old book.
    pub fn reset(&mut self) {
        self.book = None;
        self.awaiting_cover = None;
        self.buffer.clear();
        self.recent.clear();
        self.pending = None;
    }

    fn chain_ok(&mut self, ev: &DepthEv) -> Result<bool, ()> {
        match self.awaiting_cover {
            Some(l) if ev.last < l => Ok(false),
            Some(l) if ev.first <= l => {
                self.awaiting_cover = None;
                Ok(true)
            }
            Some(_) => Err(()),
            None if ev.prev == self.last_u => Ok(true),
            None => Err(()),
        }
    }

    pub fn on_depth(&mut self, ev: DepthEv) -> (DepthOutcome, Option<[bool; 3]>) {
        if self.book.is_none() {
            if self.buffer.len() == MAX_BUFFER {
                self.buffer.pop_front();
            }
            self.buffer.push_back(ev);
            return (DepthOutcome::Buffered, None);
        }
        match self.chain_ok(&ev) {
            Ok(false) => return (DepthOutcome::Stale, None),
            Err(()) => {
                self.reset();
                self.buffer.push_back(ev);
                return (DepthOutcome::Break, None);
            }
            Ok(true) => {}
        }
        let book = self.book.as_mut().expect("checked above");
        book.apply(&ev.bids, &ev.asks);
        self.last_u = ev.last;
        let check = match &self.pending {
            Some(p) if ev.last >= p.last_update => {
                let r = compare(book, &p.bids, &p.asks, &ev.touched().collect());
                self.pending = None;
                Some(r)
            }
            _ => None,
        };
        if self.recent.len() == MAX_RECENT {
            self.recent.pop_front();
        }
        self.recent.push_back(ev);
        (DepthOutcome::Applied, check)
    }

    pub fn on_snapshot(&mut self, last_update: u64, bids: &Levels, asks: &Levels) -> SnapOutcome {
        if let Some(book) = &self.book {
            if self.awaiting_cover.is_some() || self.last_u < last_update {
                let n = CHECK_LEVELS[2] * 2;
                let top = |side: &Levels| side.iter().take(n).copied().collect::<Levels>();
                self.pending = Some(Pending { last_update, bids: top(bids), asks: top(asks) });
                return SnapOutcome::Deferred;
            }
            // L is already applied: the covering event and everything after it may differ.
            let after: Vec<&DepthEv> = self.recent.iter().filter(|e| e.last >= last_update).collect();
            if after.first().is_none_or(|e| e.first > last_update) {
                return SnapOutcome::Checked(None);
            }
            let skip = after.iter().flat_map(|e| e.touched()).collect();
            return SnapOutcome::Checked(Some(compare(book, bids, asks, &skip)));
        }
        self.buffer.retain(|e| e.last >= last_update);
        let mut book = Book::from_snapshot(bids, asks);
        self.recent.clear();
        self.pending = None;
        let Some(first) = self.buffer.front() else {
            self.awaiting_cover = Some(last_update);
            self.last_u = last_update;
            self.book = Some(book);
            return SnapOutcome::Synced;
        };
        if first.first > last_update {
            return SnapOutcome::SyncFailed;
        }
        if self.buffer.iter().zip(self.buffer.iter().skip(1)).any(|(a, b)| b.prev != a.last) {
            return SnapOutcome::SyncFailed;
        }
        for e in self.buffer.drain(..) {
            book.apply(&e.bids, &e.asks);
            self.last_u = e.last;
            self.recent.push_back(e);
        }
        self.awaiting_cover = None;
        self.book = Some(book);
        SnapOutcome::Synced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(first: u64, last: u64, prev: u64, bids: Levels) -> DepthEv {
        DepthEv { rx: 0, e: 0, first, last, prev, bids, asks: vec![] }
    }

    #[test]
    fn syncs_from_buffer_dropping_stale_events_then_chains_on_pu() {
        let mut s = SymbolSync::default();
        assert_eq!(s.on_depth(ev(1, 5, 0, vec![(10, 1)])).0, DepthOutcome::Buffered);
        assert_eq!(s.on_depth(ev(6, 9, 5, vec![(11, 2)])).0, DepthOutcome::Buffered);
        assert_eq!(s.on_depth(ev(10, 12, 9, vec![(12, 3)])).0, DepthOutcome::Buffered);
        // L = 7: the first event (u=5) is stale, the second covers 7.
        assert_eq!(s.on_snapshot(7, &vec![(9, 9)], &vec![]), SnapOutcome::Synced);
        assert_eq!(s.last_update(), 12);
        assert_eq!(s.book().unwrap().top(5).0, vec![(12, 3), (11, 2), (9, 9)]);
        assert_eq!(s.on_depth(ev(13, 14, 12, vec![])).0, DepthOutcome::Applied);
        assert_eq!(s.on_depth(ev(16, 17, 15, vec![])).0, DepthOutcome::Break);
        assert!(!s.is_synced());
    }

    #[test]
    fn snapshot_before_any_event_waits_for_the_covering_event() {
        let mut s = SymbolSync::default();
        assert_eq!(s.on_snapshot(7, &vec![(9, 9)], &vec![]), SnapOutcome::Synced);
        assert_eq!(s.on_depth(ev(3, 6, 2, vec![])).0, DepthOutcome::Stale);
        assert_eq!(s.on_depth(ev(5, 8, 6, vec![(10, 1)])).0, DepthOutcome::Applied);
        assert_eq!(s.on_depth(ev(9, 9, 8, vec![])).0, DepthOutcome::Applied);
        let mut gap = SymbolSync::default();
        gap.on_snapshot(7, &vec![], &vec![]);
        assert_eq!(gap.on_depth(ev(9, 10, 8, vec![])).0, DepthOutcome::Break);
    }

    #[test]
    fn refuses_a_snapshot_older_than_the_buffer() {
        let mut s = SymbolSync::default();
        s.on_depth(ev(10, 12, 9, vec![]));
        assert_eq!(s.on_snapshot(7, &vec![], &vec![]), SnapOutcome::SyncFailed);
        assert!(!s.is_synced());
    }

    #[test]
    fn checks_late_and_early_snapshots_ignoring_touched_levels() {
        let mut s = SymbolSync::default();
        s.on_depth(ev(1, 2, 0, vec![]));
        s.on_snapshot(1, &vec![(10, 1), (9, 1)], &vec![]);
        s.on_depth(ev(3, 4, 2, vec![(10, 5)]));
        // L=3 is behind the book; level 10 was touched by the covering event, so only 9 is compared.
        assert_eq!(s.on_snapshot(3, &vec![(10, 1), (9, 1)], &vec![]), SnapOutcome::Checked(Some([true; 3])));
        // L=6 is ahead: deferred, then compared when u >= 6 arrives. The snapshot disagrees at level 9.
        assert_eq!(s.on_snapshot(6, &vec![(10, 5), (9, 2)], &vec![]), SnapOutcome::Deferred);
        let (out, check) = s.on_depth(ev(5, 6, 4, vec![]));
        assert_eq!(out, DepthOutcome::Applied);
        assert_eq!(check, Some([true, false, false]));
    }
}

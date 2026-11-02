//! Passive quoting simulator for one symbol and one parameter set (spec sections 3, 4 and 6).
//!
//! Times are exchange-clock milliseconds: depth events use `E`, trades use `T`, and our own actions take
//! effect at `rx + clock_offset + L` (`E + d_feed + L` in the spec, with `d_feed = rx - E` measured).
//! The simulated orders are never in the recorded book (size ~150 USD, impact ignored). Fees are not
//! applied here: [`Block`] keeps gross P&L and maker/taker notional so every fee row is computed later.

use std::collections::{BTreeMap, VecDeque};

use crate::book::Book;
use crate::SCALE;

pub const MARKOUT_MS: [i64; 3] = [1_000, 10_000, 60_000];
/// Spec 4: re-quote at most once per 100 ms [convention, order-rate courtesy].
const REQUOTE_MS: i64 = 100;
const IMBALANCE_LEVELS: usize = 5;
const HOUR_MS: i64 = 3_600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    S0,
    S1,
    S2a,
    S2b,
}

impl Cell {
    pub const ALL: [Cell; 4] = [Cell::S0, Cell::S1, Cell::S2a, Cell::S2b];

    fn skew(self) -> bool {
        self != Cell::S0
    }

    fn imbalance_cut(self) -> Option<f64> {
        match self {
            Cell::S2a => Some(0.3),
            Cell::S2b => Some(0.5),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queue {
    /// Headline: the queue ahead shrinks only by volume traded at our price.
    Conservative,
    /// Upper bound: non-trade drops in displayed size shrink the queue ahead pro rata.
    Optimistic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub cell: Cell,
    pub queue: Queue,
    pub latency_ms: i64,
    /// Quote size q in fixed point.
    pub q: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillKind {
    TradeThrough,
    QueueExhaustion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub t: i64,
    /// +1 buy (our bid filled), -1 sell.
    pub side: i8,
    pub px: i64,
    pub qty: i64,
    pub kind: FillKind,
    pub imbalance: f64,
    pub markout_bps: [Option<f64>; 3],
}

/// One UTC hour of one run. Gross P&L is marked to mid at the hour's last book.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub hour: i64,
    pub gross: f64,
    pub maker_notional: f64,
    pub taker_notional: f64,
    pub fills: u32,
}

#[derive(Debug, Clone, Copy)]
struct Order {
    px: i64,
    remaining: i64,
    ahead: f64,
    filled_any: bool,
}

#[derive(Debug, Default)]
struct Side {
    live: Option<Order>,
    /// (effective time, new price or None = cancel only)
    pending: Option<(i64, Option<i64>)>,
    /// Displayed size at the live price just before the current depth event.
    pre_qty: i64,
}

impl Side {
    fn target(&self) -> Option<i64> {
        match self.pending {
            Some((_, px)) => px,
            None => self.live.map(|o| o.px),
        }
    }
}

pub fn notional(px: i64, qty: i64) -> f64 {
    (px as f64 / SCALE as f64) * (qty as f64 / SCALE as f64)
}

pub fn mid(book: &Book) -> Option<i64> {
    Some((book.best_bid()?.0 + book.best_ask()?.0) / 2)
}

pub fn imbalance(book: &Book) -> f64 {
    let (b, a) = book.top(IMBALANCE_LEVELS);
    let (sb, sa) = (b.iter().map(|l| l.1 as f64).sum::<f64>(), a.iter().map(|l| l.1 as f64).sum::<f64>());
    if sb + sa == 0.0 {
        0.0
    } else {
        (sb - sa) / (sb + sa)
    }
}

pub struct Sim {
    pub p: Params,
    bid: Side,
    ask: Side,
    inv: i64,
    cash: f64,
    flatten_at: Option<i64>,
    last_decision: i64,
    /// Fills whose trade message had not yet reached us: (arrival in exchange clock, signed qty).
    /// WHY (spec 9b.3): a decision may use only messages that arrived by then, inventory included.
    unseen_fills: VecDeque<(i64, i64)>,
    pub fills: Vec<Fill>,
    markouts: [VecDeque<(usize, i64)>; 3],
    pub blocks: Vec<Block>,
    block: Block,
    block_start_equity: f64,
    last_mid: Option<i64>,
    pub quotes_placed: u64,
    pub quotes_cancelled_unfilled: u64,
    pub post_only_rejects: u64,
    pub flattens: u64,
    /// Inventory in units of q after each fill.
    pub inv_hist: BTreeMap<i64, u64>,
}

impl Sim {
    pub fn new(p: Params) -> Self {
        Sim {
            p,
            bid: Side::default(),
            ask: Side::default(),
            inv: 0,
            cash: 0.0,
            flatten_at: None,
            last_decision: i64::MIN,
            unseen_fills: VecDeque::new(),
            fills: vec![],
            markouts: Default::default(),
            blocks: vec![],
            block: Block { hour: i64::MIN, ..Block::default() },
            block_start_equity: 0.0,
            last_mid: None,
            quotes_placed: 0,
            quotes_cancelled_unfilled: 0,
            post_only_rejects: 0,
            flattens: 0,
            inv_hist: BTreeMap::new(),
        }
    }

    fn equity(&self) -> f64 {
        self.cash + self.last_mid.map_or(0.0, |m| notional(m, self.inv))
    }

    fn roll_block(&mut self, t: i64) {
        let hour = t.div_euclid(HOUR_MS);
        if hour == self.block.hour {
            return;
        }
        if self.block.hour != i64::MIN {
            let eq = self.equity();
            self.block.gross = eq - self.block_start_equity;
            self.block_start_equity = eq;
            self.blocks.push(std::mem::take(&mut self.block));
        }
        self.block.hour = hour;
    }

    /// Close the open block (end of data).
    pub fn finish(&mut self) {
        self.roll_block(i64::MAX - HOUR_MS);
    }

    /// Apply pending actions effective at or before `t`, using the book as it stands.
    pub fn activate(&mut self, t: i64, book: &Book) {
        self.roll_block(t);
        for is_bid in [true, false] {
            let side = if is_bid { &mut self.bid } else { &mut self.ask };
            let Some((eff, px)) = side.pending else { continue };
            if eff > t {
                continue;
            }
            side.pending = None;
            if let Some(o) = side.live.take() {
                self.quotes_cancelled_unfilled += u64::from(!o.filled_any);
            }
            let Some(px) = px else { continue };
            // Post-only: a quote that would cross the current book is rejected by the exchange.
            let crosses = if is_bid {
                book.best_ask().is_some_and(|a| px >= a.0)
            } else {
                book.best_bid().is_some_and(|b| px <= b.0)
            };
            if crosses {
                self.post_only_rejects += 1;
                continue;
            }
            let shown = if is_bid { book.bids.get(&px) } else { book.asks.get(&px) };
            side.live = Some(Order { px, remaining: self.p.q, ahead: shown.copied().unwrap_or(0) as f64, filled_any: false });
            self.quotes_placed += 1;
        }
        if self.flatten_at.is_some_and(|eff| eff <= t) {
            self.flatten_at = None;
            self.flatten(book);
        }
    }

    fn flatten(&mut self, book: &Book) {
        let excess = self.inv.abs() - 2 * self.p.q;
        if excess <= 0 {
            return;
        }
        let (px, sign) = if self.inv > 0 { (book.best_bid().map(|b| b.0), -1) } else { (book.best_ask().map(|a| a.0), 1) };
        let Some(px) = px else { return };
        self.inv += sign * excess;
        self.cash -= sign as f64 * notional(px, excess);
        self.block.taker_notional += notional(px, excess);
        self.flattens += 1;
    }

    fn fill(&mut self, t: i64, known_at: i64, is_bid: bool, px: i64, qty: i64, kind: FillKind, imb: f64) {
        let sign: i8 = if is_bid { 1 } else { -1 };
        self.inv += sign as i64 * qty;
        self.unseen_fills.push_back((known_at, sign as i64 * qty));
        self.cash -= sign as f64 * notional(px, qty);
        self.block.maker_notional += notional(px, qty);
        self.block.fills += 1;
        let idx = self.fills.len();
        self.fills.push(Fill { t, side: sign, px, qty, kind, imbalance: imb, markout_bps: [None; 3] });
        for (h, q) in self.markouts.iter_mut().enumerate() {
            q.push_back((idx, t + MARKOUT_MS[h]));
        }
        *self.inv_hist.entry(self.inv / self.p.q).or_default() += 1;
        // The breach is noticed when the trade message arrives, then the taker order needs L.
        if self.inv.abs() >= 4 * self.p.q && self.flatten_at.is_none() {
            self.flatten_at = Some(known_at.max(t) + self.p.latency_ms);
        }
    }

    /// A trade at exchange time `t` that reached us at `known_at` (exchange clock).
    pub fn on_trade(&mut self, t: i64, known_at: i64, px: i64, qty: i64, buyer_maker: bool, imb: f64) {
        // buyer_maker: the taker sold, so resting bids were hit; otherwise asks were lifted.
        let is_bid = buyer_maker;
        let side = if is_bid { &mut self.bid } else { &mut self.ask };
        let Some(o) = side.live.as_mut() else { return };
        let through = if is_bid { px < o.px } else { px > o.px };
        let filled = if through {
            Some((o.remaining, FillKind::TradeThrough))
        } else if px == o.px {
            o.ahead -= qty as f64;
            (o.ahead < 0.0).then(|| {
                let q = (-o.ahead as i64).min(o.remaining);
                o.ahead = 0.0;
                (q, FillKind::QueueExhaustion)
            })
        } else {
            None
        };
        let Some((q, kind)) = filled.filter(|f| f.0 > 0) else { return };
        let fill_px = o.px;
        o.remaining -= q;
        o.filled_any = true;
        if o.remaining == 0 {
            side.live = None;
        }
        self.fill(t, known_at, is_bid, fill_px, q, kind, imb);
    }

    /// Before a depth event is applied: remember displayed size at our prices.
    pub fn pre_depth(&mut self, book: &Book) {
        self.bid.pre_qty = self.bid.live.and_then(|o| book.bids.get(&o.px).copied()).unwrap_or(0);
        self.ask.pre_qty = self.ask.live.and_then(|o| book.asks.get(&o.px).copied()).unwrap_or(0);
    }

    /// After a depth event at exchange time `t`: optimistic queue advance, markouts, then the
    /// quoting decision made at `decide_at` (our local arrival mapped to the exchange clock).
    /// `traded(is_bid, px)` is the volume traded at `px` against that side since the previous event.
    pub fn post_depth(&mut self, t: i64, decide_at: i64, book: &Book, traded: &dyn Fn(bool, i64) -> i64) {
        if self.p.queue == Queue::Optimistic {
            for is_bid in [true, false] {
                let side = if is_bid { &mut self.bid } else { &mut self.ask };
                let Some(o) = side.live.as_mut() else { continue };
                let now = if is_bid { book.bids.get(&o.px) } else { book.asks.get(&o.px) }.copied().unwrap_or(0);
                // Drop not explained by trades = cancellations, assumed uniform across the queue.
                let denom = side.pre_qty - traded(is_bid, o.px);
                if denom > 0 && now < denom {
                    o.ahead *= now as f64 / denom as f64;
                }
            }
        }
        self.last_mid = mid(book).or(self.last_mid);
        self.resolve_markouts(t);
        self.decide(decide_at, book);
    }

    fn resolve_markouts(&mut self, t: i64) {
        let Some(m) = self.last_mid else { return };
        for (h, q) in self.markouts.iter_mut().enumerate() {
            while q.front().is_some_and(|x| x.1 <= t) {
                let (idx, _) = q.pop_front().expect("checked");
                let f = &mut self.fills[idx];
                f.markout_bps[h] = Some(f.side as f64 * (m - f.px) as f64 / f.px as f64 * 1e4);
            }
        }
    }

    fn decide(&mut self, now: i64, book: &Book) {
        if now.saturating_sub(self.last_decision) < REQUOTE_MS {
            return;
        }
        let (Some((bb, _)), Some((ba, _))) = (book.best_bid(), book.best_ask()) else { return };
        while self.unseen_fills.front().is_some_and(|f| f.0 <= now) {
            self.unseen_fills.pop_front();
        }
        let known_inv = self.inv - self.unseen_fills.iter().map(|f| f.1).sum::<i64>();
        let imb = imbalance(book);
        let two_q = 2 * self.p.q;
        let cut = self.p.cell.imbalance_cut();
        let want_bid = !(self.p.cell.skew() && known_inv >= two_q) && !cut.is_some_and(|c| imb < -c);
        let want_ask = !(self.p.cell.skew() && known_inv <= -two_q) && !cut.is_some_and(|c| imb > c);
        let eff = now + self.p.latency_ms;
        let mut acted = false;
        for (side, want) in [(&mut self.bid, want_bid.then_some(bb)), (&mut self.ask, want_ask.then_some(ba))] {
            if side.target() != want {
                side.pending = Some((eff, want));
                acted = true;
            }
        }
        if acted {
            self.last_decision = now;
        }
    }

    /// Book lost (break or reconnect): assume we pull both quotes at once.
    pub fn pull(&mut self) {
        for side in [&mut self.bid, &mut self.ask] {
            if let Some(o) = side.live.take() {
                self.quotes_cancelled_unfilled += u64::from(!o.filled_any);
            }
            side.pending = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q: i64 = 1_000;

    fn book(bid: (i64, i64), ask: (i64, i64)) -> Book {
        Book::from_snapshot(&vec![bid], &vec![ask])
    }

    fn sim(queue: Queue) -> Sim {
        Sim::new(Params { cell: Cell::S0, queue, latency_ms: 100, q: Q })
    }

    fn no_trades(_: bool, _: i64) -> i64 {
        0
    }

    #[test]
    fn quote_joins_back_of_queue_and_fills_only_after_queue_is_traded_away() {
        let b = book((100 * SCALE, 5_000), (101 * SCALE, 5_000));
        let mut s = sim(Queue::Conservative);
        s.post_depth(0, 0, &b, &no_trades);
        s.activate(99, &b);
        assert!(s.bid.live.is_none(), "not live before latency");
        s.activate(100, &b);
        assert_eq!(s.bid.live.unwrap().ahead, 5_000.0);
        s.on_trade(200, 200, 100 * SCALE, 4_500, true, 0.0);
        assert!(s.fills.is_empty());
        s.on_trade(300, 300, 100 * SCALE, 900, true, 0.0);
        assert_eq!((s.fills[0].qty, s.fills[0].kind), (400, FillKind::QueueExhaustion));
        s.on_trade(400, 400, 99 * SCALE, 1, true, 0.0);
        assert_eq!((s.fills[1].qty, s.fills[1].kind), (600, FillKind::TradeThrough));
        assert_eq!(s.inv, Q);
    }

    #[test]
    fn optimistic_queue_advances_on_cancels_conservative_does_not() {
        let before = book((100 * SCALE, 4_000), (101 * SCALE, 1));
        let after = book((100 * SCALE, 1_000), (101 * SCALE, 1));
        for (queue, want) in [(Queue::Conservative, 4_000.0), (Queue::Optimistic, 1_000.0)] {
            let mut s = sim(queue);
            s.post_depth(0, 0, &before, &no_trades);
            s.activate(100, &before);
            s.pre_depth(&before);
            s.post_depth(150, 150, &after, &no_trades);
            assert_eq!(s.bid.live.unwrap().ahead, want, "{queue:?}");
        }
    }

    #[test]
    fn markouts_are_signed_so_adverse_moves_are_negative() {
        let b = book((100 * SCALE, 1), (101 * SCALE, 1));
        let mut s = sim(Queue::Conservative);
        s.post_depth(0, 0, &b, &no_trades);
        s.activate(100, &b);
        s.on_trade(200, 200, 99 * SCALE, 1, true, 0.0); // bought at 100 via trade-through
        let lower = book((98 * SCALE, 1), (99 * SCALE, 1)); // mid 98.5
        s.pre_depth(&b);
        s.post_depth(1_300, 1_300, &lower, &no_trades);
        assert_eq!(s.fills[0].markout_bps[0], Some(-150.0));
        assert_eq!(s.fills[0].markout_bps[1], None);
    }

    #[test]
    fn post_only_rejects_a_bid_that_would_cross() {
        let b = book((100 * SCALE, 1), (101 * SCALE, 1));
        let mut s = sim(Queue::Conservative);
        s.post_depth(0, 0, &b, &no_trades);
        let moved = book((99 * SCALE, 1), (100 * SCALE, 1));
        s.activate(100, &moved);
        assert!(s.bid.live.is_none());
        assert_eq!(s.post_only_rejects, 1);
    }

    #[test]
    fn breach_of_four_q_flattens_to_two_q_at_taker_after_latency() {
        let b = book((100 * SCALE, 1), (101 * SCALE, 1));
        let mut s = sim(Queue::Conservative);
        for i in 0..4 {
            s.fill(i, i, true, 100 * SCALE, Q, FillKind::TradeThrough, 0.0);
        }
        assert_eq!(s.flatten_at, Some(103));
        s.activate(103, &b);
        assert_eq!(s.inv, 2 * Q);
        assert_eq!(s.flattens, 1);
        assert!(s.block.taker_notional > 0.0);
    }

    #[test]
    fn skew_cell_stops_bidding_when_long() {
        let b = book((100 * SCALE, 1), (101 * SCALE, 1));
        let mut s = Sim::new(Params { cell: Cell::S1, queue: Queue::Conservative, latency_ms: 100, q: Q });
        s.inv = 2 * Q;
        s.post_depth(0, 0, &b, &no_trades);
        assert_eq!(s.bid.target(), None);
        assert_eq!(s.ask.target(), Some(101 * SCALE));
    }

    #[test]
    fn hourly_blocks_mark_inventory_to_mid() {
        let b = book((100 * SCALE, 1), (102 * SCALE, 1)); // mid 101
        let mut s = sim(Queue::Conservative);
        s.activate(0, &b);
        s.post_depth(0, 0, &b, &no_trades);
        s.fill(10, 10, true, 100 * SCALE, SCALE, FillKind::TradeThrough, 0.0); // buy 1.0 at 100
        s.activate(HOUR_MS, &b);
        assert_eq!(s.blocks.len(), 1);
        assert!((s.blocks[0].gross - 1.0).abs() < 1e-9, "1.0 unit marked from 100 to mid 101");
        assert!((s.blocks[0].maker_notional - 100.0).abs() < 1e-9);
    }
}

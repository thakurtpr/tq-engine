"""Task 19: independent replica of Gate 1 (docs/edge-hunt-02-mm-spec.md section 2), pure Python, written from the spec and
Binance's published USD-M diff-depth rule, not from crates/mm. Reads engine/data/l2/um read-only.
Usage: python3 gate1_replica.py [DIR] [--skip-hours H --hours N]      selftest: python3 gate1_replica.py --selftest"""
import collections, glob, heapq, json, os, re, statistics, sys, zlib

DIR = '/Users/thakur/Workspace/Trading/engine/data/l2/um'
LEVELS = (1, 5, 20)

def file_key(path):
    m = re.match(r'(\d{4}-\d{2}-\d{2}T\d{2})\.jsonl\.gz(?:\.(\d+))?$', os.path.basename(path))
    return (m.group(1), int(m.group(2) or 0)) if m else None

def read_lines(path, stats):
    d = zlib.decompressobj(31); buf = b''
    with open(path, 'rb') as f:
        while True:
            c = f.read(1 << 20)
            if not c: break
            try: out = d.decompress(c)
            except zlib.error: stats['undecodable_tails'] += 1; break
            buf += out; *ls, buf = buf.split(b'\n')
            yield from ls
    if not d.eof: stats['unfinished_gzip_files'] += 1

class Sym:
    def __init__(self, name):
        self.name = name; self.c = collections.Counter(); self.hist = collections.deque(maxlen=400)
        self.lat = collections.defaultdict(list)
        self.verified_ms = 0; self.first_rx = None; self.last_rx = None
        self.reset()
    def reset(self):
        self.close_segment(); self.state = 'WAIT_SNAP'; self.buf = []; self.L = None
        self.bids = {}; self.asks = {}; self.bh = []; self.ah = []; self.last_u = None; self.pending = []
        self.hist.clear(); self.seg_start = None; self.seg_last = None; self.prev_rx = None
        self.tick_ahead = collections.deque()
    def close_segment(self):
        if getattr(self, 'seg_start', None) is not None and self.seg_last is not None:
            self.verified_ms += self.seg_last - self.seg_start
        self.seg_start = self.seg_last = None
    # ---- book maintenance
    def _set(self, book, heap, sign, p, q):
        old = book.get(p)
        if q > 0:
            if not old: heapq.heappush(heap, sign * p)
            book[p] = q
        else:
            book.pop(p, None)
        return old
    def _best(self, book, heap, sign):
        while heap and book.get(sign * heap[0], 0) == 0: heapq.heappop(heap)
        return sign * heap[0] if heap else None
    def best(self):
        return self._best(self.bids, self.bh, -1), self._best(self.asks, self.ah, 1)
    def _compact(self):
        if len(self.bh) > 4 * len(self.bids) + 2000: self.bh = [-p for p in self.bids]; heapq.heapify(self.bh)
        if len(self.ah) > 4 * len(self.asks) + 2000: self.ah = list(self.asks); heapq.heapify(self.ah)
    def load_snapshot(self, snap):
        self.bids = {float(p): float(q) for p, q in snap['bids'] if float(q) > 0}
        self.asks = {float(p): float(q) for p, q in snap['asks'] if float(q) > 0}
        self.bh = [-p for p in self.bids]; heapq.heapify(self.bh); self.ah = list(self.asks); heapq.heapify(self.ah)
    def apply(self, ev, rx):
        bold = {}; aold = {}
        for p, q in ev['b']:
            p = float(p); o = self._set(self.bids, self.bh, -1, p, float(q)); bold.setdefault(p, o)
        for p, q in ev['a']:
            p = float(p); o = self._set(self.asks, self.ah, 1, p, float(q)); aold.setdefault(p, o)
        self.last_u = ev['u']; self.hist.append((ev['U'], ev['u'], bold, aold))
        self.c['applied'] += 1
        bb, ba = self.best()
        if bb is not None and ba is not None:
            if bb >= ba: self.c['crossed'] += 1
        if self.seg_start is None: self.seg_start = rx
        self.seg_last = rx; self._compact()
    def brk(self, why):
        self.c['breaks'] += 1; self.c['breaks_' + why] += 1; self.reset()
    # ---- events
    def on_depth(self, d, rx):
        if self.first_rx is None: self.first_rx = rx
        self.last_rx = rx
        self.lat[rx // 60000].append(rx - d['E'])
        if self.prev_rx is not None and rx - self.prev_rx > 1000: self.c['silences_over_1s'] += 1; self.c['max_silence_ms'] = max(self.c['max_silence_ms'], rx - self.prev_rx)
        self.prev_rx = rx; self.c['depth_events'] += 1
        if self.state == 'WAIT_SNAP': self.buf.append((d, rx)); return
        if self.state == 'WAIT_FIRST':
            if d['u'] < self.L: self.c['stale'] += 1; return
            if not (d['U'] <= self.L <= d['u']): self.brk('first_event'); return
            self.apply(d, rx); self.state = 'SYNCED'; self.after_apply(d); return
        if d['pu'] != self.last_u: self.brk('continuity'); return
        self.apply(d, rx); self.after_apply(d)
    def after_apply(self, d):
        self.resolve_tickers()
        keep = []
        for snap, rxs in self.pending:
            if d['u'] >= snap['lastUpdateId']: self.compare(snap, len(self.hist) - 1, 'deferred')
            else: keep.append((snap, rxs))
        self.pending = keep
    def on_snapshot(self, snap, rx):
        L = snap['lastUpdateId']
        if self.state == 'SYNCED':
            if self.last_u < L: self.pending.append((snap, rx)); return
            k = next((i for i, h in enumerate(self.hist) if h[1] >= L), None)
            if k is None: self.c['snap_uncheckable_old'] += 1; return
            self.compare(snap, k, 'behind'); return
        if self.state == 'WAIT_FIRST': return
        self.load_snapshot(snap); self.L = L
        stale = [b for b in self.buf if b[0]['u'] < L]; rest = [b for b in self.buf if b[0]['u'] >= L]
        self.c['stale'] += len(stale); self.buf = []
        if not rest: self.state = 'WAIT_FIRST'; return
        d0 = rest[0][0]
        if not (d0['U'] <= L <= d0['u']): self.brk('first_event'); return
        self.state = 'SYNCED'
        for d, rxe in rest:
            if self.last_u is not None and d['pu'] != self.last_u: self.brk('continuity'); return
            self.apply(d, rxe)
    def on_bookticker(self, d):
        # bookTicker is real time, depth is batched per 100 ms: a ticker can only be compared with the book once the
        # depth event whose final update id equals the ticker's update id has been applied.
        self.c['bt_total'] += 1
        if self.state != 'SYNCED': return
        t = (d['u'], float(d['b']), float(d['B']), float(d['a']), float(d['A']))
        if t[0] == self.last_u: self.bt_check(t)
        elif t[0] > self.last_u:
            if len(self.tick_ahead) >= 20000: self.tick_ahead.popleft()
            self.tick_ahead.append(t)
    def bt_check(self, t):
        bb, ba = self.best(); self.c['bt_checked'] += 1
        if (t[1], t[3]) != (bb, ba): self.c['bt_price_mismatch'] += 1
        if (t[1], t[2], t[3], t[4]) != (bb, self.bids.get(bb), ba, self.asks.get(ba)): self.c['bt_mismatch'] += 1
    def resolve_tickers(self):
        while self.tick_ahead and self.tick_ahead[0][0] < self.last_u: self.tick_ahead.popleft()
        while self.tick_ahead and self.tick_ahead[0][0] == self.last_u: self.bt_check(self.tick_ahead.popleft())
    # ---- snapshot comparison
    def qty_asof(self, side, p, later):
        for _, _, bold, aold in later:
            o = (bold if side == 'b' else aold)
            if p in o: return o[p] or 0.0
        return (self.bids if side == 'b' else self.asks).get(p, 0.0)
    def top_asof(self, side, n, later):
        book = self.bids if side == 'b' else self.asks
        touched = set()
        for _, _, bold, aold in later: touched |= set(bold if side == 'b' else aold)
        cand = set(heapq.nlargest(n + len(touched) + 2, book) if side == 'b' else heapq.nsmallest(n + len(touched) + 2, book)) | touched
        q = {p: self.qty_asof(side, p, later) for p in cand}
        live = [p for p in cand if q[p] > 0]
        live.sort(reverse=(side == 'b')); return live[:n], q
    def compare(self, snap, k, kind):
        later = list(self.hist)[k + 1:]; cover = self.hist[k]
        x_strict = set(cover[2]) | set(cover[3]); x_len = set(x_strict)
        for _, _, bo, ao in later: x_len |= set(bo) | set(ao)
        sb = [(float(p), float(q)) for p, q in snap['bids'] if float(q) > 0]; sa = [(float(p), float(q)) for p, q in snap['asks'] if float(q) > 0]
        smap = {'b': dict(sb), 'a': dict(sa)}
        self.c['snap_checked_' + kind] += 1; self.c['snap_checked'] += 1
        for mode, lat in (('strict', later), ('lenient', [])):
            for n in LEVELS:
                ok = True
                for side, sl in (('b', sb), ('a', sa)):
                    stop = {p for p, _ in sl[:n]}
                    bk, qk = self.top_asof(side, n, lat if mode == 'strict' else [])
                    excl = x_strict if mode == 'strict' else x_len
                    for p in (stop | set(bk)) - excl:
                        if abs(smap[side].get(p, 0.0) - qk.get(p, self.qty_asof(side, p, lat))) > 1e-9: ok = False
                self.c[f'snap_{mode}_ok_{n}'] += ok; self.c[f'snap_{mode}_n_{n}'] += 1
        # Full check: every level of the 1000-level snapshot that no event from the covering one onward touched must equal the book.
        for side, sl, book in (('b', snap['bids'], self.bids), ('a', snap['asks'], self.asks)):
            sn = {float(p): float(q) for p, q in sl if float(q) > 0}
            if not sn: continue
            lo, hi = min(sn), max(sn)
            keys = ({p for p in book if lo <= p <= hi} | set(sn)) - x_len
            bad = sum(1 for p in keys if abs(sn.get(p, 0.0) - book.get(p, 0.0)) > 1e-9)
            ordered = sorted(sn, reverse=(side == 'b'))
            self.c['full_levels_compared'] += len(keys); self.c['full_mismatched_levels'] += bad
            self.c['full_sides_checked'] += 1; self.c['full_sides_ok'] += (bad == 0)
            self.c['untouched_in_top40_min'] = min(self.c['untouched_in_top40_min'] or 99, sum(1 for p in ordered[:40] if p not in x_len))

def process(paths, skip_ms=0, window_ms=None):
    st = collections.Counter(); syms = {}; probes = []; t0 = None; files_used = []
    for path in paths:
        files_used.append(os.path.basename(path))
        for raw in read_lines(path, st):
            if not raw: continue
            try: o = json.loads(raw)
            except Exception: st['bad_json'] += 1; continue
            rx = o.get('rx')
            if t0 is None: t0 = rx
            if rx - t0 < skip_ms: continue
            if window_ms is not None and rx - t0 >= skip_ms + window_ms: return st, syms, probes, files_used
            if 'event' in o:
                st['link_' + o['event']] += 1
                for s in syms.values(): s.reset()
            elif 'snap' in o:
                syms.setdefault(o['snap'], Sym(o['snap'])).on_snapshot(o['m'], rx)
            elif 'servertime' in o:
                probes.append((o['rx'], o['tx'], o['servertime']['serverTime']))
            else:
                m = o['m']; stream = m['stream']; d = m['data']; name = d['s']; s = syms.setdefault(name, Sym(name))
                if stream.endswith('@depth@100ms'): s.on_depth(d, rx)
                elif stream.endswith('@bookTicker'): s.on_bookticker(d)
                else: st['other_' + stream.split('@')[1]] += 1
    return st, syms, probes, files_used

def report(st, syms, probes, files):
    out = {'files': files, 'file_stats': dict(st), 'symbols': {}}
    for name, s in sorted(syms.items()):
        s.close_segment()
        wall = (s.last_rx - s.first_rx) if s.first_rx is not None else 0
        c = dict(s.c)
        lat_by_min = {m: sorted(v) for m, v in s.lat.items()}
        meds = [statistics.median(v) for v in lat_by_min.values()]; p99s = [v[min(len(v) - 1, int(0.99 * len(v)))] for v in lat_by_min.values()]
        off = None; corr = []; unprobed = 0
        ps = sorted(probes)
        for m, v in s.lat.items():
            rxm = m * 60000 + 30000; pr = [p for p in ps if p[0] <= rxm and rxm - p[0] <= 20 * 60000]
            if not pr: unprobed += 1; continue
            p = pr[-1]; offset = p[2] - (p[1] + p[0]) / 2; corr.append(statistics.median(v) + offset); off = offset
        out['symbols'][name] = dict(counters=c, wall_ms=wall, verified_ms=s.verified_ms,
                                    verified_share=(s.verified_ms / wall if wall else None),
                                    lat_raw_median_of_minute_medians=statistics.median(meds) if meds else None, lat_p99_max_minute=max(p99s) if p99s else None,
                                    minutes_p99_over_500=sum(1 for x in p99s if x > 500), minutes=len(p99s), minutes_unprobed=unprobed,
                                    lat_corrected_median=statistics.median(corr) if corr else None, last_offset_ms=off)
    return out

def selftest():
    def ev(U, u, pu, b, a, E=0): return {'U': U, 'u': u, 'pu': pu, 'b': b, 'a': a, 'E': E}
    s = Sym('T'); L = 100
    snap = {'lastUpdateId': L, 'bids': [['10.0', '1'], ['9.9', '2']], 'asks': [['10.1', '1'], ['10.2', '3']]}
    s.on_snapshot(snap, 0)                                   # WAIT_FIRST
    s.on_depth(ev(90, 95, 80, [['10.0', '5']], []), 1)       # stale (u < L)
    assert s.c['stale'] == 1 and s.state == 'WAIT_FIRST'
    s.on_depth(ev(96, 101, 95, [['10.0', '2']], [['10.1', '4']]), 2)   # covers L
    assert s.state == 'SYNCED' and s.bids[10.0] == 2 and s.asks[10.1] == 4
    s.on_depth(ev(102, 103, 101, [['9.95', '1']], []), 3)
    # periodic snapshot L=101 arriving after event u=103 ("behind"): state as of event covering 101 = after event(96,101)
    snap2 = {'lastUpdateId': 101, 'bids': [['10.0', '2'], ['9.9', '2']], 'asks': [['10.1', '4'], ['10.2', '3']]}
    s.on_snapshot(snap2, 4)
    assert s.c['snap_strict_ok_20'] == 1, s.c      # 9.95 was added later; strict undoes it
    assert s.c['snap_lenient_ok_20'] == 1, s.c
    # an injected error must be caught by strict
    snap3 = {'lastUpdateId': 101, 'bids': [['10.0', '7'], ['9.9', '9']], 'asks': [['10.1', '4'], ['10.2', '3']]}
    s.on_snapshot(snap3, 5)
    # 10.0 was touched by the covering event (excluded by rule); 9.9 was not, so the error shows at levels 5 and 20 only
    assert s.c['snap_strict_ok_1'] == 2 and s.c['snap_strict_ok_5'] == 1 and s.c['snap_strict_n_5'] == 2, s.c
    # continuity break
    s.on_depth(ev(110, 112, 999, [], []), 6)
    assert s.c['breaks_continuity'] == 1 and s.state == 'WAIT_SNAP'
    # deferred snapshot: L ahead of the book
    s2 = Sym('U'); s2.on_snapshot({'lastUpdateId': 50, 'bids': [['5', '1']], 'asks': [['6', '1']]}, 0)
    s2.on_depth(ev(45, 50, 40, [['5', '2']], []), 1)
    s2.on_snapshot({'lastUpdateId': 60, 'bids': [['5', '2'], ['4', '1']], 'asks': [['6', '1']]}, 2)
    assert len(s2.pending) == 1
    s2.on_depth(ev(51, 61, 50, [['4', '1']], []), 3)
    assert s2.c['snap_checked_deferred'] == 1 and s2.c['snap_strict_ok_20'] == 1, s2.c
    # first event not covering L (gap) breaks
    s3 = Sym('V'); s3.on_snapshot({'lastUpdateId': 10, 'bids': [], 'asks': []}, 0)
    s3.on_depth(ev(20, 25, 19, [], []), 1)
    assert s3.c['breaks_first_event'] == 1
    print('selftest ok')

if __name__ == '__main__':
    if '--selftest' in sys.argv: selftest(); sys.exit(0)
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    d = args[0] if args and os.path.isdir(args[0]) else DIR
    def opt(name, default):
        return float(sys.argv[sys.argv.index(name) + 1]) if name in sys.argv else default
    skip = int(opt('--skip-hours', 0) * 3_600_000); hrs = opt('--hours', None); win = int(hrs * 3_600_000) if hrs is not None else None
    paths = sorted((p for p in glob.glob(os.path.join(d, '*.jsonl.gz*')) if file_key(p)), key=file_key)
    res = report(*process(paths, skip, win))
    json.dump(res, open('gate1_replica_summary.json', 'w'), indent=1, default=str)
    for name, v in res['symbols'].items():
        c = v['counters']
        print(f"{name}: depth events {c.get('depth_events',0)}, applied {c.get('applied',0)}, stale {c.get('stale',0)}, breaks {c.get('breaks',0)} "
              f"(continuity {c.get('breaks_continuity',0)}, first-event {c.get('breaks_first_event',0)}), silences>1s {c.get('silences_over_1s',0)} (max {c.get('max_silence_ms',0)} ms), crossed {c.get('crossed',0)}, "
              f"verified {v['verified_share']:.1%} of {v['wall_ms']/1000:.0f}s")
        print(f"   bookTicker: {c.get('bt_total',0)} msgs, {c.get('bt_checked',0)} compared at the same update id, price mismatches {c.get('bt_price_mismatch',0)}, price+qty mismatches {c.get('bt_mismatch',0)}")
        print(f"   snapshots checked {c.get('snap_checked',0)} (deferred {c.get('snap_checked_deferred',0)}, behind {c.get('snap_checked_behind',0)}, old/uncheckable {c.get('snap_uncheckable_old',0)})")
        print(f"   full untouched-level check: {c.get('full_sides_ok',0)}/{c.get('full_sides_checked',0)} snapshot sides exact, {c.get('full_levels_compared',0)} levels compared, {c.get('full_mismatched_levels',0)} mismatched; fewest untouched levels inside the snapshot's top 40: {c.get('untouched_in_top40_min','-')}")
        for mode in ('strict', 'lenient'):
            print('   ' + mode + ': ' + ', '.join(f"L{n} {c.get(f'snap_{mode}_ok_{n}',0)}/{c.get(f'snap_{mode}_n_{n}',0)}" for n in LEVELS))
        print(f"   latency rx-E: median of minute medians {v['lat_raw_median_of_minute_medians']} ms, worst minute p99 {v['lat_p99_max_minute']} ms, minutes with p99>500: {v['minutes_p99_over_500']}/{v['minutes']}, corrected median {v['lat_corrected_median']} (offset {v['last_offset_ms']}), unprobed minutes {v['minutes_unprobed']}")
    print('link events:', {k: x for k, x in res['file_stats'].items() if k.startswith('link_')}, '| unfinished gzip files:', res['file_stats'].get('unfinished_gzip_files', 0), '| undecodable tails:', res['file_stats'].get('undecodable_tails', 0), '| other streams:', {k: x for k, x in res['file_stats'].items() if k.startswith('other_')})

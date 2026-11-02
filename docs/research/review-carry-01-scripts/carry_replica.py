"""Task 17: independent replica of engine/research/carry_backtest.py (pure Python, no pandas), written from the
spec in docs/edge-hunt-01.md section 4 and the deviations described in reports/carry-01.md, NOT by copying the code.
Reads engine/data read-only. Run from this folder: python3 carry_replica.py"""
import csv, glob, math, json, sys, datetime as dt
from collections import defaultdict

E = '/Users/thakur/Workspace/Trading/engine/data'
SPOT_FEE, PERP_FEE, SLIP, MMR = 7.5e-4, 5e-4, 1e-4, 0.005
HOUR = 3_600_000

def norm(t):
    t = int(t)
    return t // 1000 if t > 10**14 else t          # spot archive is microseconds from 2025-01

def klines(sub, sym, cols):
    out = {}
    for f in sorted(glob.glob(f'{E}/{sub}/{sym}/*.csv')):
        for r in csv.reader(open(f)):
            if not r or not r[0].isdigit(): continue
            out[norm(r[0])] = tuple(float(r[c]) for c in cols)
    return out

def funding(sym):
    out = defaultdict(float); iv = defaultdict(int)
    for f in sorted(glob.glob(f'{E}/funding/{sym}/*.csv')):
        for r in csv.DictReader(open(f)):
            t = int(r['calc_time']); h = t - t % HOUR
            out[h] += float(r['last_funding_rate']); iv[r['funding_interval_hours']] += 1
    return out, iv

def load(sym):
    spot = klines('spot-1h', sym, (1, 4))             # open, close
    perp = klines('perp-1h', sym, (1, 2, 4))          # open, high, close
    mark = klines('mark-1h', sym, (1, 2))             # open, high
    rate, iv = funding(sym)
    hours = sorted(set(spot) & set(perp) & set(mark))
    return hours, spot, perp, mark, rate, iv

def year(ms): return dt.datetime.fromtimestamp(ms / 1000, dt.timezone.utc).year
def day(ms): return ms // 86_400_000

def simulate(data, m, band=(0.5, 1.5), liq='mark', rebal=True):
    hours, spot, perp, mark, rate, _ = data
    n_frac = 1 / (1 + m)
    s0 = spot[hours[0]][0]; p0 = perp[hours[0]][0]
    q = n_frac / s0                                   # equity 1.0 at start; N = n_frac
    wallet = (1 - n_frac) - q * s0 * (SPOT_FEE + SLIP) - q * p0 * (PERP_FEE + SLIP)   # perp wallet cash
    entry = p0; prev_s = s0; prev_p = p0
    rows = []                                         # (hour, equity, funding, spot_pnl, perp_pnl, sells)
    reb = 0; liq_hour = None
    for h in hours:
        so, sc = spot[h]; po, ph, pc = perp[h]; mo, mh = mark[h]
        f = q * mo * rate.get(h, 0.0)                 # short receives when rate > 0
        wallet += f
        hi = mh if liq == 'mark' else ph
        if wallet + q * (entry - hi) < MMR * q * hi:
            liq_hour = h; break
        sp = q * (sc - prev_s); prev_s = sc
        pp = q * (prev_p - pc); prev_p = pc           # short price PnL for the hour (close to close)
        perp_eq = wallet + q * (entry - pc)
        total = q * sc + perp_eq
        sells = 0.0
        if rebal and not (band[0] * m <= perp_eq / (q * pc) <= band[1] * m):
            nq = total * n_frac / sc; dq = abs(nq - q)
            if nq < q: sells = dq * sc
            wallet = total * (1 - n_frac) - dq * sc * (SPOT_FEE + SLIP) - dq * pc * (PERP_FEE + SLIP)
            q, entry = nq, pc; reb += 1
            total = q * sc + wallet
        rows.append([h, total, f, sp, pp, sells])
    if liq_hour is None:
        h = hours[-1]; sc = spot[h][1]; pc = perp[h][2]
        rows[-1][1] -= q * sc * (SPOT_FEE + SLIP) + q * pc * (PERP_FEE + SLIP)
        rows[-1][5] += q * sc
    return rows, reb, liq_hour

def daily(rows):
    d = {}
    for h, eq, *_ in rows: d[day(h)] = (h, eq)          # last hourly equity of each UTC day
    ks = sorted(d); return [(k, d[k][1]) for k in ks]

def ann(r): return math.prod(1 + x for x in r) ** (365 / len(r)) - 1 if r else float('nan')

def hac_t(r):
    n = len(r); lag = int(math.floor(4 * (n / 100) ** (2 / 9))); mu = sum(r) / n; d = [x - mu for x in r]
    var = sum(x * x for x in d) / n
    for k in range(1, lag + 1):
        var += 2 * (1 - k / (lag + 1)) * sum(d[i] * d[i - k] for i in range(k, n)) / n
    return mu / math.sqrt(var / n)

def maxdd(eqs):
    peak = -1e9; dd = 0
    for e in eqs: peak = max(peak, e); dd = min(dd, e / peak - 1)
    return dd

def summary(rows, liq_hour):
    dl = daily(rows); eqs = [e for _, e in dl]
    r = [eqs[i] / eqs[i - 1] - 1 for i in range(1, len(eqs))]
    cut = day(int(dt.datetime(2024, 1, 1, tzinfo=dt.timezone.utc).timestamp() * 1000))
    days = [k for k, _ in dl][1:]
    r_is = [x for x, k in zip(r, days) if k < cut - 1]; r_oos = [x for x, k in zip(r, days) if k >= cut]
    return dict(full=ann(r), t=hac_t(r), dd=maxdd(eqs), is_=ann(r_is), oos=ann(r_oos), ratio=ann(r_oos) / ann(r_is),
                n_days=len(r), liquidated=liq_hour)

def yearly(rows):
    dl = daily(rows)
    end = {}
    for k, e in dl: end[year(k * 86_400_000)] = e
    ys = sorted(end); start = {y: (end[ys[i - 1]] if i else 1.0) for i, y in enumerate(ys)}
    acc = {y: dict(f=0.0, s=0.0, p=0.0, sells=0.0) for y in ys}
    for h, eq, f, sp, pp, sl in rows:
        a = acc[year(h)]; a['f'] += f; a['s'] += sp; a['p'] += pp; a['sells'] += sl
    out = {}
    for y in ys:
        st = start[y]; a = acc[y]; pre = (end[y] - st) / st
        tax_b = 0.30 * (max(a['f'], 0) + max(a['s'], 0))                       # engine's definition
        tax_b2 = 0.30 * (max(a['f'], 0) + max(a['s'], 0) + max(a['p'], 0))     # + perp price gains taxed, no set-off
        out[y] = dict(pre=pre, fund=a['f'] / st, spot=a['s'] / st, perp=a['p'] / st,
                      b=(end[y] - st - tax_b) / st, b2=(end[y] - st - tax_b2) / st,
                      tds=(end[y] - st - a['sells'] * 0.01 * 0.08) / st)
    return out

if __name__ == '__main__':
    res = {}
    for sym in ('BTCUSDT', 'ETHUSDT'):
        data = load(sym); hours = data[0]
        gaps = sum(1 for a, b in zip(hours, hours[1:]) if b - a != HOUR)
        print(f'{sym}: hours {len(hours)}, first {dt.datetime.fromtimestamp(hours[0]/1000, dt.timezone.utc)}, last {dt.datetime.fromtimestamp(hours[-1]/1000, dt.timezone.utc)}, gaps {gaps}, funding intervals {dict(data[5])}')
        for m in (1.0, 0.5):
            rows, reb, lh = simulate(data, m)
            s = summary(rows, lh); y = yearly(rows)
            print(f"  M={m}: full {s['full']:+.2%} t {s['t']:.2f} DD {s['dd']:.2%} IS {s['is_']:+.2%} OOS {s['oos']:+.2%} ratio {s['ratio']:.2f} reb {reb} liq {s['liquidated']}")
            for yr, v in y.items():
                print(f"    {yr}: pre {v['pre']:+.2%} fund {v['fund']:+.2%} spot {v['spot']:+.2%} perp {v['perp']:+.2%} (b) {v['b']:+.2%} (b2 both legs) {v['b2']:+.2%} TDS {v['tds']:+.2%}")
            res[f'{sym} M={m}'] = dict(summary=s, years=y, rebalances=reb)
    json.dump(res, open('carry_replica_summary.json', 'w'), indent=1, default=str)

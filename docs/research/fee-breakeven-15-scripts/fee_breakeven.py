"""Task 15: fee break-even study. Imports the task-14 replicas (read-only on the engine data).
Constant notional, full 2022-2023 window for crypto, 60 sessions for NSE ORB.
Break-even round-trip cost (bps of notional) == mean gross bps per trade; 95% CI from the day-clustered SE.
Run from this folder with: python3 fee_breakeven.py   (the replica folder is added to the path below)."""
import sys, math, collections, importlib.util, json
sys.path.insert(0, '../review-intraday-01-scripts')
import intraday as I

MIN = 60000
COSTS = (0, 1, 2, 3, 4, 6, 8, 10)   # round-trip bps of notional (fee + slippage, both legs)

def clustered(vals, days):
    n = len(vals); m = sum(vals) / n
    by = collections.defaultdict(float)
    for v, d in zip(vals, days): by[d] += v - m
    G = len(by)
    se = math.sqrt(sum(x * x for x in by.values()) * G / (G - 1)) / n
    return m, se, G

def block(label, gross, days, hold_min, reasons):
    m, se, G = clustered(gross, days); n = len(gross)
    hs = sorted(hold_min)
    q = lambda p: hs[min(len(hs) - 1, int(p * len(hs)))]
    print(f"\n== {label}: n={n}, days={G}")
    print(f"   breakeven round-trip cost = mean gross {m:+.2f} bps, clustered SE {se:.2f}, 95% CI [{m-1.96*se:+.2f}, {m+1.96*se:+.2f}]")
    print(f"   hold minutes: median {q(.5):.0f}, p90 {q(.9):.0f}, max {hs[-1]:.0f}; stop exits {sum(1 for r in reasons if r=='stop')/n*100:.0f}%")
    byr = collections.defaultdict(list)
    for g, r in zip(gross, reasons): byr[r].append(g)
    print("   by exit reason: " + "; ".join(f"{r}: n={len(v)} mean {sum(v)/len(v):+.1f}" for r, v in sorted(byr.items())))
    print("   net bps after cost c (mean, 95% CI):  " + "  ".join(f"c={c}: {m-c:+.2f} [{m-c-1.96*se:+.2f},{m-c+1.96*se:+.2f}]" for c in COSTS))
    return dict(label=label, n=n, mean=m, se=se, lo=m - 1.96 * se, hi=m + 1.96 * se, hold_median=q(.5), hold_p90=q(.9))

out = []
for coin in ('BTCUSDT', 'ETHUSDT'):
    data = I.load(coin)
    for strat in ('vwap', 'momentum'):
        tr, st = I.run(coin, strat, data=data)
        gross = [t['gross_bps'] for t in tr]
        days = [t['entry_ts'] // I.DAY for t in tr]
        hold = [(t['exit_ts'] - t['entry_ts']) / MIN for t in tr]
        out.append(block(f"{coin} {strat}", gross, days, hold, [t['reason'] for t in tr]))
        for y in (2022, 2023):
            sel = [i for i, t in enumerate(tr) if 1970 + int(t['entry_ts'] / 31556952000) == y]
            g = [gross[i] for i in sel]; d = [days[i] for i in sel]
            m, se, G = clustered(g, d)
            print(f"   year {y}: n={len(g)} mean {m:+.2f} [{m-1.96*se:+.2f},{m+1.96*se:+.2f}]")

# ORB: patch the replica source to also record the exit timestamp, leaving the original file untouched
src = open('../review-intraday-01-scripts/orb.py').read()
src = src.replace("ts=pos['ts'],day=pos['day']", "ts=pos['ts'],xts=ts,day=pos['day']")
spec = importlib.util.spec_from_loader('orb_x', loader=None); orb = importlib.util.module_from_spec(spec)
exec(compile(src.split("if __name__")[0], 'orb_x', 'exec'), orb.__dict__)
allt = []
for sym in ('RELIANCE', 'HDFCBANK', 'ICICIBANK', 'INFY', 'TCS'):
    tr, ns = orb.run(sym)
    allt += tr
gross = [t['g'] for t in allt]; days = [t['day'] for t in allt]
hold = [(t['xts'] - t['ts']) / MIN for t in allt]
out.append(block("NSE ORB pooled (5 stocks, 60 sessions)", gross, days, hold, [t['reason'] for t in allt]))
same = [t for t in allt if orb.dkey(t['xts']) == orb.dkey(t['ts'])]
m, se, G = clustered([t['g'] for t in same], [t['day'] for t in same])
print(f"   intraday-only subset: n={len(same)} mean {m:+.2f} [{m-1.96*se:+.2f},{m+1.96*se:+.2f}]")
json.dump(out, open('fee_breakeven_summary.json', 'w'), indent=1)

"""Task 17 checks on top of carry_replica.py: funding sign, dropped funding hours, decomposition, deviations 1-3, E bound."""
import datetime as dt, json, math
import carry_replica as C

def iso(ms): return dt.datetime.fromtimestamp(ms / 1000, dt.timezone.utc).strftime('%Y-%m-%d %H:%M')

out = {}
for sym in ('BTCUSDT', 'ETHUSDT'):
    data = C.load(sym); hours, spot, perp, mark, rate, iv = data; hs = set(hours)
    print(f'\n===== {sym}')
    # 1) data gaps and funding hours falling in them
    gaps = [(a, b) for a, b in zip(hours, hours[1:]) if b - a != C.HOUR]
    lost = {h: r for h, r in rate.items() if h not in hs}
    print(f'hour gaps in the common index: {len(gaps)}; first {iso(gaps[0][0])} -> {iso(gaps[0][1])}, last {iso(gaps[-1][0])} -> {iso(gaps[-1][1])}')
    print(f'funding payments whose hour is missing from the common kline index (silently dropped): {len(lost)} of {len(rate)}, sum of rates {sum(lost.values()):+.5f}')
    for h, r in sorted(lost.items())[:6]: print('   ', iso(h), f'{r:+.6f}', 'in spot' if h in spot else '-', 'in perp' if h in perp else '-', 'in mark' if h in mark else '-')
    # 2) sign check on a known row
    h0 = hours[0]; print(f'sign: first funding {iso(h0)} rate {rate.get(h0):+.8f} -> short receives {"yes" if rate.get(h0) > 0 else "NO, short pays (rate < 0)"}')
    # 3) annual funding on notional straight from raw rates (3/day) vs the run
    by = {}
    for h, r in rate.items(): by.setdefault(C.year(h), []).append(r)
    print('raw funding rate sum per calendar year (annual yield on constant notional, %):',
          ', '.join(f'{y}: {sum(v)*100:.1f} ({len(v)} pays, {100*sum(1 for x in v if x>0)/len(v):.0f}% positive)' for y, v in sorted(by.items())))
    # 4) decomposition: pre-tax - (funding + spot + perp) = costs residual
    rows, reb, lh = C.simulate(data, 1.0); y = C.yearly(rows)
    print('decomposition M=1.0 (pre - fund - spot - perp = costs and compounding residual):')
    for yr, v in y.items(): print(f"   {yr}: pre {v['pre']:+.3%} fund {v['fund']:+.3%} spot+perp {v['spot']+v['perp']:+.3%} residual {v['pre']-v['fund']-v['spot']-v['perp']:+.3%}")
    # 5) sign flipped sensitivity
    flipped = {h: -r for h, r in rate.items()}
    rows_f, _, _ = C.simulate((hours, spot, perp, mark, flipped, iv), 1.0)
    print('sign-flipped funding, full-window ann.:', f"{C.summary(rows_f, None)['full']:+.2%} (should be about minus the base run)")
    # 6) deviation 1: last-price vs mark liquidation
    worst = max(((perp[h][1] / mark[h][1] - 1, h) for h in hours), key=lambda x: x[0])
    print(f'largest last-price-high / mark-high excess: {worst[0]:+.1%} at {iso(worst[1])} (last high {perp[worst[1]][1]}, mark high {mark[worst[1]][1]})')
    for m in (1.0, 0.5):
        for liq in ('mark', 'last'):
            r_, rb, lh = C.simulate(data, m, liq=liq)
            print(f'   M={m} liquidation on {liq}-price high: liquidated at {iso(lh) if lh else "never"}, rebalances {rb}')
    # 7) deviation 2: band sensitivity and no rebalance
    print('band sensitivity M=1.0 (full, OOS, DD, rebalances, liquidated):')
    for band in ((0.25, 1.75), (0.5, 1.5), (0.7, 1.3), (0.85, 1.15)):
        r_, rb, lh = C.simulate(data, 1.0, band=band); s = C.summary(r_, lh)
        print(f"   band {band}: full {s['full']:+.2%} OOS {s['oos']:+.2%} DD {s['dd']:.2%} reb {rb} liq {iso(lh) if lh else 'never'}")
    r_, rb, lh = C.simulate(data, 1.0, rebal=False); s = C.summary(r_, lh)
    print(f"   no rebalance: full {s['full']:+.2%} OOS {s['oos']:+.2%} DD {s['dd']:.2%} liq {iso(lh) if lh else 'never'}")
    # 8) deviation 3: tax-case alternatives (year-by-year b, b2)
    print('tax cases M=1.0 per year: (b) engine, (b2) both legs taxed, no set-off:')
    for yr, v in y.items(): print(f"   {yr}: b {v['b']:+.2%}  b2 {v['b2']:+.2%}")
    neg_b = [yr for yr, v in y.items() if v['b'] < 0]; neg_b2 = [yr for yr, v in y.items() if v['b2'] < 0]
    print('   years negative under (b):', neg_b, ' under (b2):', neg_b2)
    out[sym] = dict(gaps=len(gaps), lost_funding_payments=len(lost), lost_sum=sum(lost.values()))
json.dump(out, open('carry_checks_summary.json', 'w'), indent=1)

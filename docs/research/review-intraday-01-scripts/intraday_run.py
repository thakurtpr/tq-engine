import sys, math, statistics as S, collections, json; sys.path.insert(0,'.')
import intraday as I
def summ(tr,label):
    g=[t['gross_bps'] for t in tr]; n=len(g)
    m=sum(g)/n; sd=S.stdev(g); se=sd/math.sqrt(n)
    byday=collections.defaultdict(float)
    for t in tr: byday[t['entry_ts']//86400000]+=t['gross_bps']-m
    G=len(byday); secl=math.sqrt(sum(v*v for v in byday.values())*G/(G-1))/n
    yr=collections.defaultdict(list)
    for t in tr: yr[1970+int(t['entry_ts']/31556952000)].append(t['gross_bps'])
    stops=sum(1 for t in tr if t['reason']=='stop')/n
    print(f"{label:34} n={n:6d} mean gross {m:+.2f} bps  sd {sd:6.1f}  SE naive {se:.2f} (t {m/se:+.1f})  SE day-clustered {secl:.2f} (t {m/secl:+.1f}, {G} days)  stops {stops*100:.0f}%  by year "+', '.join(f"{y}: {sum(v)/len(v):+.2f} (n {len(v)})" for y,v in sorted(yr.items())))
    return m,secl
for coin in ('BTCUSDT','ETHUSDT'):
    data=I.load(coin)
    for strat in ('vwap','momentum'):
        tr,st=I.run(coin,strat,data=data)
        summ(tr,f'{coin} {strat} UNCONSTRAINED')
        print('     signals',st['signals'],'rejected at fill',st['rejected'])

import sys, math, random, statistics as S, collections; sys.path.insert(0,'.')
import replica as R, portfolio as P
from decimal import Decimal as D
def trades_for(kind, kp=D(5)):
    data,wins=P.load_universe('pit' if kind=='pit' else 'cur'); allt=[]
    for s,rows in data.items():
        tr,cv,st=R.run(rows,'2012-01-01',windows=(wins[s] if wins else None),kill_pct=kp)
        for t in tr: t['sym']=s
        allt+=tr
    return allt
def ret(t): return float(t['net']/(t['entry']*t['qty'])*100)
def naive_t(x):
    n=len(x); m=sum(x)/n; sd=math.sqrt(sum((v-m)**2 for v in x)/(n-1)); return m, sd, m/(sd/math.sqrt(n))
def cluster_t(trades,x,keyf):
    n=len(x); m=sum(x)/n; g=collections.defaultdict(float)
    for t,v in zip(trades,x): g[keyf(t)]+=v-m
    G=len(g); se=math.sqrt(sum(s*s for s in g.values())*G/(G-1))/n   # CR1 small-sample factor
    return m/se, G
def boot(trades,x,keyf,B=10000,seed=1):
    rnd=random.Random(seed); g=collections.defaultdict(list)
    for t,v in zip(trades,x): g[keyf(t)].append(v)
    keys=list(g); ms=[]
    for _ in range(B):
        samp=[v for k in (rnd.choice(keys) for _ in keys) for v in g[k]]
        ms.append(sum(samp)/len(samp))
    ms.sort(); return ms[int(.025*B)], ms[int(.975*B)], sum(1 for m in ms if m<=0)/B
for kind in ['cur','pit']:
    tr=trades_for(kind); x=[ret(t) for t in tr]
    m,sd,t0=naive_t(x); n=len(x)
    sk=sum(((v-m)/sd)**3 for v in x)/n; ku=sum(((v-m)/sd)**4 for v in x)/n
    print(f"== {kind}: n={n} mean {m:.3f}% sd {sd:.2f}% naive t {t0:.2f} | skew {sk:.2f} kurt {ku:.1f} | win {100*sum(v>0 for v in x)/n:.1f}%")
    for lab,kf in [('month',lambda t:t['entry_date'][:7]),('year',lambda t:t['entry_date'][:4]),('symbol',lambda t:t['sym'])]:
        ct,G=cluster_t(tr,x,kf); lo,hi,p=boot(tr,x,kf,B=4000)
        print(f"   cluster by {lab:6}: t {ct:.2f} (G={G}); effective n ~ {n*(ct/t0)**2:.0f}; block-bootstrap 95% CI for mean [{lo:.2f}, {hi:.2f}]%, P(mean<=0)={p:.3f}")
    # concurrency of trades: average number of other trades open during each trade's life
    ov=[]
    for a in tr:
        ov.append(sum(1 for b in tr if b is not a and not (b['exit_date']<a['entry_date'] or b['entry_date']>a['exit_date'])))
    print(f"   average number of other trades open during a trade's life: {sum(ov)/len(ov):.1f} (median {S.median(ov)})")
    hold=[(R.dt.date.fromisoformat(t['exit_date'])-R.dt.date.fromisoformat(t['entry_date'])).days if hasattr(R,'dt') else 0 for t in tr] if False else None

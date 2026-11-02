import sys, random, hashlib, statistics as S; sys.path.insert(0,'.')
import portfolio as P
from decimal import Decimal as D
data,wins=P.load_universe(sys.argv[1])
def rnd_order(seed):
    def f(syms,d):
        l=list(syms); random.Random(hashlib.md5(f'{seed}{d}'.encode()).hexdigest()).shuffle(l); return l
    return f
out=[]
for mp in (5,10):
    res=[]
    for seed in range(8):
        tr,cv,st,npd=P.simulate(data,wins,rnd_order(seed),kill_pct=D(100),max_pos=mp)
        res.append(P.summarize(tr,cv))
    tr,cv,st,npd=P.simulate(data,wins,lambda s,d:s,kill_pct=D(100),max_pos=mp,rank='strength')
    rk=P.summarize(tr,cv)
    q=lambda k:(round(min(r[k] for r in res),2),round(S.median(r[k] for r in res),2),round(max(r[k] for r in res),2))
    print(f"{sys.argv[1]} max_pos={mp} random-priority x8 (min/median/max): trades {q('n')} t {q('t')} CAGR% {q('cagr')} MDD% {q('mdd')} Sharpe {q('sharpe')} PF {q('pf')}")
    print(f"   strength-ranked: {rk}")

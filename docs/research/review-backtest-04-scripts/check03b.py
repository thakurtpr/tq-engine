import sys, json, multiprocessing as mp, statistics as S; sys.path.insert(0,'.')
import portfolio2 as P
from decimal import Decimal as D
P.USE_TICK=False
DATA,WINS=P.load_universe()
def job(a):
    mp_,kill,mult,seed=a
    tr,cv,st=P.simulate(DATA,WINS,max_pos=mp_,kill_pct=D(kill),mult=D(mult),seed=seed)
    r=P.summary(tr,cv); r['total']=round((r['final']/1e6-1)*100,2); r['kill_at']=st['kill_at']; r['trades']=len(tr); return a,r
def pct(v,q):
    v=sorted(v); k=(len(v)-1)*q; f=int(k); c=min(f+1,len(v)-1); return v[f]+(v[c]-v[f])*(k-f)
if __name__=='__main__':
    jobs=[(5,100,1,None),(10,100,1,None),(5,100,2,None),(10,100,2,None),(5,100,3,None),(5,10,1,None),(10,10,1,None)]+[(5,100,1,s) for s in range(20)]+[(10,100,1,s) for s in range(20)]
    with mp.get_context('fork').Pool(9) as pool: res=dict(pool.map(job,jobs,chunksize=1))
    g=lambda k:res[k]
    print('alphabetical, no tick, adopted slippage:')
    print(f"  D5 1x: CAGR {g((5,100,1,None))['cagr']:.2f} (03b says 1.27)  total {g((5,100,1,None))['total']}% (03b 13.43)  | 2x total {g((5,100,2,None))['total']}% (03b 9.45)  | 3x total {g((5,100,3,None))['total']}% (03b 5.53)")
    print(f"  D10 1x: CAGR {g((10,100,1,None))['cagr']:.2f} (03b 0.09)  total {g((10,100,1,None))['total']}% (03b 0.86) | 2x total {g((10,100,2,None))['total']}% (03b -5.47)")
    print(f"  kill 10%: D5 latch {g((5,10,1,None))['kill_at']} trades {g((5,10,1,None))['trades']} CAGR {g((5,10,1,None))['cagr']:.2f} MDD {g((5,10,1,None))['mdd']:.1f} (03b: 2015-10-05, 111, 0.64, 10.7)")
    print(f"            D10 latch {g((10,10,1,None))['kill_at']} trades {g((10,10,1,None))['trades']} CAGR {g((10,10,1,None))['cagr']:.2f} MDD {g((10,10,1,None))['mdd']:.1f} (03b: 2015-04-21, 171, 1.68, 11.6)")
    for p,exp in ((5,'0.90 (-0.37..2.07), MDD 26.2'),(10,'1.97 (0.33..3.56), MDD 39.1')):
        v=[res[(p,100,1,s)]['cagr'] for s in range(20)]; dd=[res[(p,100,1,s)]['mdd'] for s in range(20)]
        print(f"  seeds D{p}: CAGR median {S.median(v):.2f} (p5 {pct(v,.05):.2f}, p95 {pct(v,.95):.2f}); MDD median {S.median(dd):.1f}; positive {sum(x>0 for x in v)}/20   [03b: {exp}]")

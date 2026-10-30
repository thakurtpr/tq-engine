import sys, math, json, multiprocessing as mp, statistics as S, collections; sys.path.insert(0,'.')
import rebal as X, replica as R
from decimal import Decimal as D
SER,WIN=X.load_pit(); BEES={'NIFTYBEES':R.load(X.E+'/data/etf/NIFTYBEES.csv')}
def seedjob(sd):
    c,st,_=X.run(SER,WIN,X.momentum_targets(),seed=sd); s=X.summarize(c); return sd,(s['cagr'],s['mdd'],s['sharpe'])
def killjob(a):
    name,kp=a
    if name=='mom': c,st,_=X.run(SER,WIN,X.momentum_targets(),kill_pct=D(kp))
    elif name=='trend': c,st,_=X.run(BEES,None,X.Faber('NIFTYBEES'),cap_pct=D(100),kill_pct=D(kp))
    else: c,st,_=X.run(BEES,None,X.buyhold('NIFTYBEES'),cap_pct=D(100),kill_pct=D(kp))
    s=X.summarize(c); return name,(st['kill_at'],round(s['cagr'],2),round(s['mdd'],1))
if __name__=='__main__':
    with mp.get_context('fork').Pool(9) as pool:
        seeds=dict(pool.map(seedjob,range(20)))
        kills=dict(pool.map(killjob,[('mom',10),('trend',10),('bh',10)]))
    cg=[v[0] for v in seeds.values()]
    print(f"momentum 20 seeds CAGR min/median/max {min(cg):.2f}/{S.median(cg):.2f}/{max(cg):.2f} (report 14.02-14.08)")
    print('10% kill switch operating runs (replica): latch date, CAGR, MDD:',kills,"\n   report: mom 2013-08-07 3.29%/15.4 ; trend 2012-05-14 -0.94%/10.5 ; B&H 2012-05-04 14.31%/36.3")
    json.dump({'seeds':seeds},open('mom_seeds.json','w'))

import sys, math, csv, json, multiprocessing as mp, collections; sys.path.insert(0,'.')
import portfolio2 as P, replica as R
from decimal import Decimal as D
DATA,WINS=P.load_universe()
def job(a):
    mp_,kill,mult,seed=a
    tr,cv,st=P.simulate(DATA,WINS,max_pos=mp_,kill_pct=D(kill),mult=D(mult),seed=seed)
    full=[D(1000000)]+[c[1] for c in cv]; dates=[c[0] for c in cv]
    rets=[float(full[i+1]/full[i]-1) for i in range(len(full)-1)]
    cal={d:i for i,d in enumerate(dates)}
    trd=[(t['entry_date'],t['exit_date'],float(t['net']/(t['entry']*t['qty'])*100)) for t in tr]
    hold=[cal.get(t[1],0)-cal.get(t[0],0) for t in trd]
    return a,dict(rets=rets,dates=dates,trd=trd,hold=hold)
def cluster_t(vals,keys):
    n=len(vals); m=sum(vals)/n; g=collections.defaultdict(float)
    for v,k in zip(vals,keys): g[k]+=v-m
    G=len(g); se=math.sqrt(sum(s*s for s in g.values())*G/(G-1))/n
    return m/se
def nw_t(x,L):
    n=len(x); m=sum(x)/n; d=[v-m for v in x]
    g0=sum(v*v for v in d)/n; s=g0
    for l in range(1,L+1):
        w=1-l/(L+1); gl=sum(d[i]*d[i-l] for i in range(l,n))/n; s+=2*w*gl
    return m/math.sqrt(s/n)
def stats(o):
    r=o['rets']; d=o['dates']; T=len(r)
    mk=[x[:7] for x in d]
    t_cl=cluster_t(r,mk)
    L0=min(int(4*(T/100)**(2/9)), T//4); Lh=min(max(L0,max(o['hold'] or [0])),T//4)
    t_nw0=nw_t(r,L0); t_nwh=nw_t(r,Lh)
    # monthly compounded returns
    mon=collections.OrderedDict()
    for ret,k in zip(r,mk): mon[k]=mon.get(k,1.0)*(1+ret)
    mr=[v-1 for v in mon.values()]; n=len(mr); m=sum(mr)/n; sd=math.sqrt(sum((v-m)**2 for v in mr)/(n-1)); t_mon=m/(sd/math.sqrt(n))
    tt=[x[2] for x in o['trd']]; t_trade=cluster_t(tt,[x[0][:7] for x in o['trd']]) if len(tt)>2 else float('nan')
    return dict(t_month_cl_daily=t_cl,t_hac_L0=t_nw0,L0=L0,t_hac_Lhold=t_nwh,Lhold=Lh,t_monthly=t_mon,t_trade_cl=t_trade,
                maxhold=max(o['hold'] or [0]),med_hold=sorted(o['hold'])[len(o['hold'])//2] if o['hold'] else 0)
if __name__=='__main__':
    jobs=[(5,100,1,None),(10,100,1,None),(5,100,2,None),(5,100,3,None)]+[(5,100,1,s) for s in range(20)]+[(10,100,1,s) for s in range(20)]
    with mp.get_context('fork').Pool(9) as pool: res=dict(pool.map(job,jobs,chunksize=1))
    out={str(k):stats(v) for k,v in res.items()}
    json.dump(out,open('t_v2.json','w'))
    summ={r['label']:r for r in csv.DictReader(open('/Users/thakur/Workspace/Trading/engine/reports/backtest-03/summary.csv'))}
    print('engine t_month_clustered vs my recomputation (alphabetical, kill off):')
    for p in (5,10):
        for m in (1,2,3):
            if (p,100,m,None) in res or m==1: pass
    for key,lab in (((5,100,1,None),'donchian-p5-killoff-slip1x'),((10,100,1,None),'donchian-p10-killoff-slip1x'),((5,100,2,None),'donchian-p5-killoff-slip2x'),((5,100,3,None),'donchian-p5-killoff-slip3x')):
        s=out[str(key)]; print(f"  {lab:30} engine {float(summ[lab]['t_month_clustered']):6.3f} | mine {s['t_month_cl_daily']:6.3f}")
    print('\nv2 gate-4c inputs, alphabetical, kill off, 1x:')
    for key,lab in (((5,100,1,None),'Donchian 5'),((10,100,1,None),'Donchian 10')):
        s=out[str(key)]
        print(f"  {lab}: month-clustered daily {s['t_month_cl_daily']:.2f} | HAC L={s['L0']} {s['t_hac_L0']:.2f} | HAC L={s['Lhold']} (longest hold {s['maxhold']}d, median {s['med_hold']}d) {s['t_hac_Lhold']:.2f} | monthly-return t {s['t_monthly']:.2f} | per-trade month-clustered {s['t_trade_cl']:.2f}")
    import statistics as S
    for p in (5,10):
        rows=[out[str((p,100,1,s))] for s in range(20)]
        print(f"  random x20, p{p}: month-cl daily t min/med/max {min(r['t_month_cl_daily'] for r in rows):.2f}/{S.median(r['t_month_cl_daily'] for r in rows):.2f}/{max(r['t_month_cl_daily'] for r in rows):.2f} | HAC(L0) {min(r['t_hac_L0'] for r in rows):.2f}/{S.median(r['t_hac_L0'] for r in rows):.2f}/{max(r['t_hac_L0'] for r in rows):.2f} | monthly {min(r['t_monthly'] for r in rows):.2f}/{S.median(r['t_monthly'] for r in rows):.2f}/{max(r['t_monthly'] for r in rows):.2f}")

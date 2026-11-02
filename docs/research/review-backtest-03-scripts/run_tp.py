import sys, csv, json, multiprocessing as mp; sys.path.insert(0,'.')
import portfolio2 as P, replica as R, tp
R.Don=tp.TP
from decimal import Decimal as D
DATA,WINS=P.load_universe()
def job(a):
    mp_,kill,mult=a
    t,cv,st=P.simulate(DATA,WINS,max_pos=mp_,kill_pct=D(kill),mult=D(mult))
    r=P.summary(t,cv); r['net']=float(sum(x['net'] for x in t)); r['costs']=float(sum(x['costs'] for x in t)); return a,r
if __name__=='__main__':
    jobs=[(p,k,m) for p in (5,10) for k in (5,100) for m in (1,2,3)]
    with mp.get_context('fork').Pool(9) as pool: res=dict(pool.map(job,jobs,chunksize=1))
    summ={r['label']:r for r in csv.DictReader(open('/Users/thakur/Workspace/Trading/engine/reports/backtest-03/summary.csv'))}
    worst=0; bad=0
    for (p,k,m),r in sorted(res.items()):
        s=summ[f"trend-pullback-p{p}-kill{'on' if k==5 else 'off'}-slip{m}x"]
        ok=r['n']==int(s['trades']) and abs(r['final']-float(s['final_equity']))<0.05
        bad+=not ok; worst=max(worst,abs(r['final']-float(s['final_equity'])))
        print(f"p{p} kill{'on ' if k==5 else 'off'} {m}x: trades {r['n']:>5}/{s['trades']:>5}  CAGR {r['cagr']:>7.3f}/{float(s['cagr_pct']):>7.3f}  final {r['final']:>12.2f}/{float(s['final_equity']):>12.2f} {'OK' if ok else 'MISMATCH'}")
    print('rows not matching:',bad,'of',len(res),'; largest final-equity diff Rs',worst)

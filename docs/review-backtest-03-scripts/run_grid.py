import sys, csv, json, multiprocessing as mp; sys.path.insert(0,'.')
import portfolio2 as P
from decimal import Decimal as D
DATA,WINS=P.load_universe()
def job(a):
    mp_,kill,mult,seed=a
    tr,cv,st=P.simulate(DATA,WINS,max_pos=mp_,kill_pct=D(kill),mult=D(mult),seed=seed)
    r=P.summary(tr,cv); r['kill_at']=st['kill_at']; r['capped']=st['adv_capped']
    return a,r
if __name__=='__main__':
    jobs=[]
    for p in (5,10):
        for k in (5,100):
            for m in (1,2,3): jobs.append((p,k,m,None))
    for s in range(20): jobs.append((5,100,1,s)); jobs.append((10,100,1,s))
    ctx=mp.get_context('fork')
    with ctx.Pool(9) as pool: res=dict(pool.map(job,jobs,chunksize=1))
    json.dump({str(k):v for k,v in res.items()},open('grid_results.json','w'))
    print('done',len(res))

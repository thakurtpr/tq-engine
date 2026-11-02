import sys, math, csv, collections, statistics as S, multiprocessing as mp; sys.path.insert(0,'.')
import portfolio as OLD, portfolio2 as P, replica as R
from decimal import Decimal as D
import datetime as dt
E='/Users/thakur/Workspace/Trading/engine'
# (e) original portfolio.py (flat 5 bps, alphabetical), the four configs the engineer cites
data,wins=OLD.load_universe('pit')
def old(a):
    mp_,kp=a
    tr,cv,st,npd=OLD.simulate(data,wins,lambda s,d:s,kill_pct=D(kp),max_pos=mp_)
    s=OLD.summarize(tr,cv); return a,(s,len(tr))
def main():
    with mp.get_context('fork').Pool(4) as pool: res=dict(pool.map(old,[(5,100),(5,5),(10,100),(10,5)]))
    print('(e) portfolio.py (flat 5bps, alphabetical) vs report cross-check table (engine / reviewer)')
    for a,lab,exp in (((5,100),'5 off',(288,14.13,21.4,0.21)),((5,5),'5 on',(17,-3.13,6.3,-0.18)),((10,100),'10 off',(525,2.06,44.8,0.08)),((10,5),'10 on',(14,-1.40,7.0,-0.06))):
        s,n=res[a]; print(f"   {lab:7} replica: trades {n} total ret {s['ret_pct']}% CAGR(252d) {s['cagr']}% MDD {s['mdd']}% Sharpe {s['sharpe']}   | report engine: trades {exp[0]} ret {exp[1]}% MDD {exp[2]} Sharpe {exp[3]}")
main()

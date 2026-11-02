import sys, math, collections, statistics as S; sys.path.insert(0,'.')
import portfolio2 as P, replica as R
from decimal import Decimal as D
import datetime as dt
E='/Users/thakur/Workspace/Trading/engine'
DATA,WINS=P.load_universe()
tr,cv,st=P.simulate(DATA,WINS,max_pos=5,kill_pct=D(100),mult=D(1))
net=sum(t['net'] for t in tr); costs=sum(t['costs'] for t in tr)
print(f"closed trades {len(tr)}; closed net Rs {net:.0f} (report 43,197); costs Rs {costs:.0f} (report 69,701); final equity {cv[-1][1]:.2f}; unrealised+closed = {cv[-1][1]-1000000:.0f}")
bysym=collections.defaultdict(D)
for t in tr: bysym[t['sym']]+=t['net']
top=max(bysym,key=bysym.get); print(f"top symbol {top}: Rs {bysym[top]:.0f} = {100*bysym[top]/net:.0f}% of closed net (report: HCLTECH 134%); distinct symbols traded {len(bysym)} (report 58)")
srt=sorted((t['net'] for t in tr),reverse=True); print(f"closed net without best 5 trades: Rs {sum(srt[5:]):.0f} (report -2.05 lakh)")
wins=sum(t['net'] for t in tr if t['net']>0); loss=-sum(t['net'] for t in tr if t['net']<0); print(f"profit factor {wins/loss:.3f} (report 1.04); mean net trade % {sum(float(t['net']/(t['entry']*t['qty'])*100) for t in tr)/len(tr):.3f} (report 0.41)")
# calendar years strategy vs NSEI
def yearly(dates_equity,initial):
    ye={}
    for d,e in dates_equity: ye[d[:4]]=e
    out={}; prev=initial
    for y in sorted(ye): out[y]=float(ye[y]/prev-1)*100; prev=ye[y]
    return out
sy=yearly(cv,D(1000000))
rows=[l.strip().split(',') for l in open(E+'/data/index/NSEI.csv')][1:]
cl={r[0]:D(r[4]) for r in rows}
dates=sorted(cl); base=[d for d in dates if d<'2012-01-01'][-1]
ni=[(d,cl[d]) for d in dates if '2012-01-01'<=d<='2021-12-31']
ny=yearly(ni,cl[base])
print('calendar-year returns, strategy (Donchian p5 off, alphabetical) vs ^NSEI price index:')
for y in sorted(sy): print(f"  {y}: {sy[y]:6.1f}% vs {ny[y]:6.1f}%  {'<- beat' if sy[y]>ny[y] else ''}")
# benchmark stats
full=[cl[base]]+[c for _,c in ni]; rets=[float(full[i+1]/full[i]-1) for i in range(len(full)-1)]
m=sum(rets)/len(rets); sd=math.sqrt(sum((x-m)**2 for x in rets)/(len(rets)-1)); shp=m/sd*math.sqrt(252)
d0=dt.date.fromisoformat(ni[0][0]); d1=dt.date.fromisoformat(ni[-1][0]); yrs=((d1-d0).days+1)/365.25
pk=0; mdd=0
for x in full: x=float(x); pk=max(pk,x); mdd=max(mdd,(pk-x)/pk*100)
print(f"^NSEI: CAGR {((float(full[-1]/full[0]))**(1/yrs)-1)*100:.2f}% (report 14.15), Sharpe {shp:.2f} (0.88), max DD {mdd:.1f}% (38.4); first bar {ni[0][0]} base {base}")
# tick-rounding effect: avg slippage bps on entries/exits from the replica trades
ent=[];ext_sig=[];ext_stop=[]; lowpx=0
for t in tr:
    rows_=DATA[t['sym']]; idx={r[0]:r for r in rows_}
    o=idx[t['entry_date']][1]; ent.append(float((t['entry']/o-1)*10000))
    if float(o)<100: lowpx+=1
    b=idx[t['exit_date']]
    if t['why']=='stop': raw=min(b[1],t['stop']); ext_stop.append(float((1-t['exit']/raw)*10000))
    else: raw=b[1]; ext_sig.append(float((1-t['exit']/raw)*10000))
print(f"avg entry slippage {S.mean(ent):.1f} bps (median {S.median(ent):.1f}); signal exits {S.mean(ext_sig):.1f} bps; stop exits {S.mean(ext_stop):.1f} bps (n={len(ext_stop)}, base would be 5 and 10); entries at adjusted open price < Rs 100: {lowpx}/{len(tr)}")

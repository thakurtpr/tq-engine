"""Replica v2: shared-capital portfolio with the ADOPTED slippage model (docs/slippage-model.md section 5)
and the engine's SplitMix64 random priority. Written from docs + Rust source."""
import sys, os, math, datetime as dt; sys.path.insert(0,'.')
import replica as R, pit
from decimal import Decimal as D, ROUND_FLOOR, ROUND_CEILING, ROUND_HALF_UP
from collections import deque
E='/Users/thakur/Workspace/Trading/engine'
HUND=D(100); TICK=D('0.05'); M64=(1<<64)-1

def load_universe():
    by=pit.plan(); data={}; wins={}
    for s,w in by.items():
        p=f'{E}/data/daily-pit/{s}.csv'
        if os.path.exists(p): data[s]=R.load(p); wins[s]=w
    return data,wins

class Liq:
    def __init__(s): s.v=deque(maxlen=20); s.c=deque(maxlen=21)
    def push(s,bar): s.v.append(bar[5]); s.c.append(bar[4])
    def stats(s):
        adv=(sum(s.v)/D(20)) if len(s.v)==20 else None
        sig=None
        if len(s.c)>20:
            cl=[float(x) for x in s.c]; r=[cl[i+1]/cl[i]-1 for i in range(20)]
            m=sum(r)/20; sig=math.sqrt(sum((x-m)**2 for x in r)/19)
        return adv,sig

def bps(base,qty,adv,sig,stop):
    b=base
    if adv is not None and adv>0 and sig is not None and qty/adv*HUND>D('0.05'):
        imp=0.7*sig*math.sqrt(float(qty/adv))*1e4
        b=max(D(repr(round(3.5+imp,4))),base)
    return b*2 if stop else b
USE_TICK=True
def tick(price,side):
    if not USE_TICK: return price
    t=price/TICK
    if side=='B': return t.to_integral_value(rounding=ROUND_CEILING)*TICK
    return max(t.to_integral_value(rounding=ROUND_FLOOR)*TICK,TICK)
def fill_price(raw,side,qty,adv,sig,stop,mult):
    f=bps(D(5),qty,adv,sig,stop)*mult/D(10000)
    p=raw*(1+f) if side=='B' else raw*(1-f)
    return tick(p,side)

class SM64:
    def __init__(s,seed): s.x=seed&M64
    def next(s):
        s.x=(s.x+0x9E3779B97F4A7C15)&M64; z=s.x
        z=((z^(z>>30))*0xBF58476D1CE4E5B9)&M64; z=((z^(z>>27))*0x94D049BB133111EB)&M64
        return z^(z>>31)
def rotl(x,k): return ((x<<k)|(x>>(64-k)))&M64
def ts_of(date):  # ist_close_ts: date 00:00 UTC + 10:00 UTC (15:30 IST)
    d=dt.date.fromisoformat(date); return int(dt.datetime(d.year,d.month,d.day,tzinfo=dt.timezone.utc).timestamp()*1000)+36_000_000
def order_cands(cands,date,seed):
    cands.sort(key=lambda x:x[0])
    if seed is not None:
        g=SM64(seed ^ rotl(ts_of(date),17))
        for i in range(len(cands)-1,0,-1):
            j=g.next()%(i+1); cands[i],cands[j]=cands[j],cands[i]
    return cands

def simulate(data,wins,max_pos=5,kill_pct=D(100),mult=D(1),seed=None,cash0=D(1000000),risk_pct=D('0.5'),
             cap=D(20),daily_loss=D(2),start='2012-01-01',end='2021-12-31',want_curve=False):
    strat={s:R.Don() for s in data}; liq={s:Liq() for s in data}
    idx={s:{r[0]:i for i,r in enumerate(rows)} for s,rows in data.items()}
    for s,rows in data.items():
        for r in rows:
            if r[0]<start: strat[s].on_bar(r); liq[s].push(r)
    cal=sorted({r[0] for rows in data.values() for r in rows if start<=r[0]<=end})
    cash=cash0; eq=cash0; sod=cash0; peak=cash0; kill=False; kill_at=None
    pos={}; pend_e={}; pend_x=[]; trades=[]; curve=[]
    last_idx={s:len(rows)-1 for s,rows in data.items()}
    stats=dict(adv_capped=0,rej_cash=0,gap=0,zero=0,outside=0,kill_at=None)
    for k,d in enumerate(cal):
        today=[s for s in sorted(data) if d in idx[s]]
        bars={s:data[s][idx[s][d]] for s in today}
        # exits then entries
        px_exit=[]
        for s in list(pend_x):
            if s in bars:
                if s in pos:
                    adv,sig=liq[s].stats(); o=bars[s][1]
                    p=fill_price(o,'S',pos[s]['qty'],adv,sig,False,mult)
                    t=R.close_pos(pos[s],d,p,'signal',R.costs_one); t['sym']=s
                    trades.append(t); cash+=p*pos[s]['qty']-t['exit_costs']; del pos[s]; strat[s].long=False
                pend_x.remove(s)
        for s in list(pend_e):   # insertion order = priority order
            if s not in bars: continue
            sig_=pend_e.pop(s); stop=sig_['stop']
            if not any(a<=d<b for a,b in wins[s]): stats['outside']+=1; continue
            adv,sg=liq[s].stats(); o=bars[s][1]
            base_p=fill_price(o,'B',D(0),adv,sg,False,mult)
            def wanted(p):
                if p<=0: return D(0)
                per=(p-stop).copy_abs()
                br=D(0) if per==0 or eq<=0 else (eq*risk_pct/HUND/per).to_integral_value(rounding=ROUND_FLOOR)
                bc=(eq*cap/HUND/p).to_integral_value(rounding=ROUND_FLOOR)
                return max(min(br,bc),D(0))
            w=wanted(base_p)
            capq=(adv*D(1)/HUND).to_integral_value(rounding=ROUND_FLOOR) if adv is not None else D(0)
            q=w
            if w>capq: stats['adv_capped']+=1; q=capq
            p=fill_price(o,'B',q,adv,sg,False,mult)
            if p>base_p: q=min(wanted(p),q)
            if q==0: stats['zero']+=1; continue
            if p<=stop: stats['gap']+=1; continue
            if (p-stop)*q > eq*D(1)/HUND or p*q > eq*cap/HUND: stats['gap']+=1; continue   # gate.check_order
            if p*q>cash: stats['rej_cash']+=1; continue
            ec=R.costs_one('B',p*q); cash-=p*q+ec
            pos[s]=dict(qty=q,entry=p,stop=stop,entry_costs=ec,entry_date=d,last=p,sig=sig_); strat[s].long=True
        for s in today:
            if s in pos and bars[s][3]<=pos[s]['stop']:
                adv,sg=liq[s].stats(); raw=min(bars[s][1],pos[s]['stop'])
                p=fill_price(raw,'S',pos[s]['qty'],adv,sg,True,mult)
                t=R.close_pos(pos[s],d,p,'stop',R.costs_one); t['sym']=s
                trades.append(t); cash+=p*pos[s]['qty']-t['exit_costs']; del pos[s]; strat[s].long=False
        for s in today:
            if s in pos: pos[s]['last']=bars[s][4]
        e=cash+sum(p['qty']*p['last'] for p in pos.values())
        sod=eq; eq=e; peak=max(peak,eq)
        if not kill and (peak-eq)>=peak*kill_pct/HUND: kill=True; stats['kill_at']=d
        curve.append((d,eq))
        cands=[]
        for s in today:
            has_next = idx[s][d] < last_idx[s]
            sg=strat[s].on_bar(bars[s])
            if sg is None or not has_next: continue
            if sg[0]=='X':
                if s in pos and s not in pend_x: pend_x.append(s)
            else: cands.append((s,sg))
        cands=order_cands(cands,d,seed)
        for s,sg in cands:
            if s in pend_e or s in pos: continue
            if kill or (sod-eq)>=sod*daily_loss/HUND or (peak-eq)>=peak*kill_pct/HUND: continue
            if len(pos)+len(pend_e)>=max_pos: continue
            pend_e[s]=dict(stop=sg[2],signal_close=sg[1],prior_high=sg[3],atr=sg[4],signal_date=d)
        for s in today: liq[s].push(bars[s])
    return trades,curve,stats

def summary(trades,curve,cash0=D(1000000)):
    full=[cash0]+[c[1] for c in curve]; e0=float(cash0)
    dr=[float(full[i+1]/full[i]-1) for i in range(len(full)-1)]
    m=sum(dr)/len(dr); v=sum((x-m)**2 for x in dr)/(len(dr)-1); sh=m/math.sqrt(v)*math.sqrt(252) if v>0 else float('nan')
    pk=D(0); mdd=D(0)
    for x in full:
        pk=max(pk,x); mdd=max(mdd,(pk-x)/pk*HUND)
    d0=dt.date.fromisoformat(curve[0][0]); d1=dt.date.fromisoformat(curve[-1][0])
    yrs=((d1-d0).days+1)/365.25; g=float(full[-1]/cash0)
    return dict(n=len(trades),cagr=round((g**(1/yrs)-1)*100,4),mdd=round(float(mdd),4),sharpe=round(sh,4),final=round(float(full[-1]),2))

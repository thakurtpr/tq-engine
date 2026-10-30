"""Independent replica of tq-backtest `run_rebalance` (docs: crates/backtest/src/rebalance/{mod,orders}.rs,
risk/src/rebalance.rs) with the adopted slippage model, adjusted-price basis (no tick rounding)."""
import sys, os, math, csv, datetime as dt; sys.path.insert(0,'.')
import replica as R, pit
from decimal import Decimal as D, ROUND_FLOOR
from collections import deque
E='/Users/thakur/Workspace/Trading/engine'
HUND=D(100); BUF=D('0.5'); MINT=D('0.5')

class Liq:
    def __init__(s): s.v=deque(maxlen=20); s.c=deque(maxlen=21)
    def push(s,bar): s.v.append(bar[5]); s.c.append(bar[4])
    def stats(s):
        adv=(sum(s.v)/D(20)) if len(s.v)==20 else None; sig=None
        if len(s.c)>20:
            cl=[float(x) for x in s.c]; r=[cl[i+1]/cl[i]-1 for i in range(20)]
            m=sum(r)/20; sig=math.sqrt(sum((x-m)**2 for x in r)/19)
        return adv,sig
def bps(qty,adv,sig):
    if adv is not None and adv>0 and sig is not None and qty/adv*HUND>D('0.05'):
        return max(D(repr(round(3.5+0.7*sig*math.sqrt(float(qty/adv))*1e4,4))),D(5))
    return D(5)
def fill_price(raw,side,qty,adv,sig,mult):
    f=bps(qty,adv,sig)*mult/D(10000)
    return raw*(1+f) if side=='B' else raw*(1-f)
def costs(side,V):
    brk=D(0); stt=(V*D('0.001')).quantize(D(1),rounding='ROUND_HALF_UP')
    ex=V*D('0.00307')/HUND; sebi=V*D('0.0001')/HUND
    stamp=V*D('0.015')/HUND if side=='B' else D(0); dp=D(13) if side=='S' else D(0)
    return brk+stt+ex+sebi+stamp+dp+(brk+ex+sebi+dp)*D('0.18')

def month_key(d): return d[:7]
def run(series,windows,targets_fn,cash0=D(1000000),max_pos=10,cap_pct=D(20),mult=D(1),adv_cap_pct=D(1),
        start='2012-01-01',kill_pct=D(100),daily_loss=D(2),want_trades=False,cash_yield=D(0),seed=None,member_at_fill=False):
    idx={s:{r[0]:i for i,r in enumerate(rows)} for s,rows in series.items()}
    days=sorted({r[0] for rows in series.values() for r in rows})
    hist={}; liq={s:Liq() for s in series}
    cash=cash0; hold={}; pending=None; prev=cash0; peak=cash0; kill=False
    curve=[]; turnover=D(0); nsell=0; ntrade_days=0; rej=collections=None
    stats=dict(daily_loss_block=0,capped=0,noadv=0,posblock=0,sells=0,buys=0,kill_at=None)
    for i,d in enumerate(days):
        bars={s:series[s][idx[s][d]] for s in series if d in idx[s]}
        pend=pending; pending=None
        if d>=start:
            if pend is not None:
                ntrade_days+=1
                eq_open=cash+sum(h['qty']*(bars[s][1] if s in bars else h['last']) for s,h in hold.items())
                peak=max(peak,eq_open)
                sod=prev
                investable=eq_open*(1-BUF/HUND)
                wanted={}
                for s,w in pend:
                    wanted[s]=((w*investable/bars[s][1]).to_integral_value(rounding=ROUND_FLOOR) if s in bars else (hold[s]['qty'] if s in hold else D(0)))
                mint=eq_open*MINT/HUND
                for s in sorted(hold):
                    q=hold[s]['qty']; tgt=wanted.get(s,D(0)); ex=q-tgt
                    if ex<=0 or s not in bars: continue
                    o=bars[s][1]
                    if tgt==0 or ex*o>=mint:
                        adv,sg=liq[s].stats(); p=fill_price(o,'S',ex,adv,sg,mult); c=costs('S',p*ex)
                        cash+=p*ex-c; turnover+=p*ex; stats['sells']+=1
                        hold[s]['qty']-=ex
                        if hold[s]['qty']==0: del hold[s]
                buys=sorted(s for s,q in wanted.items() if q>(hold[s]['qty'] if s in hold else 0))
                if seed is not None:
                    import portfolio2 as P2
                    g=P2.SM64(seed ^ P2.rotl(P2.ts_of(d),17))
                    for k in range(len(buys)-1,0,-1):
                        j=g.next()%(k+1); buys[k],buys[j]=buys[j],buys[k]
                for s in buys:
                    if s not in bars: continue
                    o=bars[s][1]; held=hold[s]['qty'] if s in hold else D(0); short=wanted[s]-held
                    if not (held==0 or short*o>=mint): continue
                    adv,sg=liq[s].stats(); q=short
                    if adv is None or adv<=0: stats['noadv']+=1
                    else:
                        cp=(adv*adv_cap_pct/HUND).to_integral_value(rounding=ROUND_FLOOR)
                        if q>cp: stats['capped']+=1; q=cp
                    p=fill_price(o,'B',q,adv,sg,mult)
                    aff=max((cash*(1-BUF/HUND)/p).to_integral_value(rounding=ROUND_FLOOR),D(0)); q=min(q,aff)
                    if q==0: continue
                    # gate: kill switch / loss limits / positions / notional cap
                    if kill: continue
                    if (sod-eq_open)>=sod*daily_loss/HUND: stats['daily_loss_block']+=1; continue
                    if (peak-eq_open)>=peak*kill_pct/HUND: continue
                    if held==0 and len(hold)>=max_pos: stats['posblock']+=1; continue
                    if (held+q)*p > eq_open*cap_pct/HUND: continue
                    c=costs('B',p*q); cash-=p*q+c; turnover+=p*q; stats['buys']+=1
                    if s in hold: hold[s]['qty']+=q
                    else: hold[s]=dict(qty=q,last=p)
            for s,h in hold.items():
                if s in bars: h['last']=bars[s][4]
            if cash_yield!=0: cash=cash*((1+cash_yield)**(D(1)/D(252)))
            inv=sum(h['qty']*h['last'] for h in hold.values()); eq=cash+inv
            stats.setdefault('expo',[]).append(float(inv/eq))
            peak=max(peak,eq); prev=eq
            if not kill and (peak-eq)>=peak*kill_pct/HUND: kill=True; stats['kill_at']=d
            curve.append((d,eq))
        for s,b in bars.items(): hist.setdefault(s,[]).append(b[4])
        nxt=days[i+1] if i+1<len(days) else None
        if nxt is not None and month_key(nxt)!=month_key(d):
            md=nxt if member_at_fill else d
            members=[s for s in sorted(bars) if (windows is None or s not in windows or any(a<=md<b_ for a,b_ in windows[s]))]
            tg=targets_fn(d,members,hist)
            pending=[t for t in tg if t[0] in members]
        for s,b in bars.items(): liq[s].push(b)
    return curve,stats,turnover

def momentum_targets(lookback=252,skip=21,top=10):
    def f(d,members,hist):
        sc=[]
        for s in members:
            c=hist.get(s)
            if c is None or len(c)<lookback+1: continue
            last=len(c)-1; st=c[last-lookback]; en=c[last-skip]
            if st==0: continue
            sc.append((s,en/st-1))
        sc.sort(key=lambda x:(-x[1],x[0]))
        return [(s,D(1)/D(top)) for s,_ in sc[:top]]
    return f
class Faber:
    def __init__(s,sym,n=10): s.sym=sym; s.n=n; s.m=[]
    def __call__(s,d,members,hist):
        c=hist.get(s.sym)
        if not c: return []
        s.m.append(c[-1])
        if len(s.m)<s.n or s.sym not in members: return []
        sma=sum(s.m[-s.n:])/D(s.n)
        return [(s.sym,D(1))] if c[-1]>sma else []
def buyhold(sym):
    def f(d,members,hist): return [(sym,D(1))] if sym in members and hist.get(sym) else []
    return f

def summarize(curve,cash0=D(1000000)):
    full=[cash0]+[e for _,e in curve]; dr=[float(full[i+1]/full[i]-1) for i in range(len(full)-1)]
    m=sum(dr)/len(dr); sd=math.sqrt(sum((x-m)**2 for x in dr)/(len(dr)-1))
    pk=D(0); mdd=D(0)
    for x in full: pk=max(pk,x); mdd=max(mdd,(pk-x)/pk*HUND)
    d0=dt.date.fromisoformat(curve[0][0]); d1=dt.date.fromisoformat(curve[-1][0]); yrs=((d1-d0).days+1)/365.25
    return dict(final=float(full[-1]),cagr=((float(full[-1]/cash0))**(1/yrs)-1)*100,mdd=float(mdd),sharpe=m/sd*math.sqrt(252),rets=dr)
def load_pit():
    by=pit.plan(); ser={}; win={}
    for s,w in by.items():
        p=f'{E}/data/daily-pit/{s}.csv'
        if os.path.exists(p): ser[s]=R.load(p); win[s]=w
    return ser,win

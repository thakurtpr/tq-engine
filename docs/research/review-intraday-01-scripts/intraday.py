"""Independent replica of tq-live (crates/live) for the 1m crypto strategies: VWAP reversion and
momentum burst, runner fill rules (next-open fills, 5 bps slippage, resting stops, gap cancels pending
entry), optional compounding account with the engine's sizing. Floats, not Decimal: counts may differ
by a handful from the engine where a signal sits on a rounding boundary."""
import csv, glob, math, sys
from collections import deque
E='/Users/thakur/Workspace/Trading/engine'
MIN=60000; DAY=86400000

def load(coin, y0=2022, y1=2023):
    ts=[];o=[];h=[];l=[];c=[];v=[]
    for f in sorted(glob.glob(f'{E}/data/intraday/{coin}/{coin}-1m-*.csv')):
        yr=int(f.split('-1m-')[1][:4])
        if not (y0<=yr<=y1): continue
        for r in csv.reader(open(f)):
            if not r[0].isdigit(): continue
            ts.append(int(r[0])); o.append(float(r[1])); h.append(float(r[2])); l.append(float(r[3])); c.append(float(r[4])); v.append(float(r[5]))
    return ts,o,h,l,c,v

class Stat:
    """rolling window with exact sample stdev/mean recomputed cheaply"""
    def __init__(s,n): s.n=n; s.q=deque(); s.sum=0.0; s.sq=0.0; s.k=0
    def push(s,x):
        s.q.append(x); s.sum+=x; s.sq+=x*x
        if len(s.q)>s.n:
            y=s.q.popleft(); s.sum-=y; s.sq-=y*y
        s.k+=1
        if s.k%2000==0: s.sum=sum(s.q); s.sq=sum(x*x for x in s.q)
    def full(s): return len(s.q)==s.n
    def mean(s): return s.sum/len(s.q)
    def sd(s):
        n=len(s.q)
        if n<2: return None
        m=s.sum/n; var=(s.sq-n*m*m)/(n-1); return math.sqrt(var) if var>0 else 0.0

def run(coin, strat, y0=2022, y1=2023, account=False, cash0=10000.0, lot=1e-5, fee=0.001, slip=5e-4,
        max_risk=0.005, max_notional=0.20, const_notional=1000.0, data=None, live_from=None):
    ts,o,h,l,c,v=data if data else load(coin,y0,y1)
    n=len(ts)
    # strategy state
    in_pos=False; held=0
    atr=None; atr_seen=0; atr_seed=0.0; pc=None
    cum_pv=0.0; cum_v=0.0; day=None; closes=Stat(20)
    c6=deque(maxlen=6); rets=Stat(100); vols=Stat(100)
    # runner state
    pos=None; pend_entry=None; pend_exit=False; gap=False
    cash=cash0; equity=cash0
    trades=[]; sig=0; rej=0
    for i in range(n):
        t=ts[i]; O=o[i];H=h[i];L=l[i];C=c[i];V=v[i]
        if i>0 and t-ts[i-1]!=MIN: gap=True
        if gap:
            pend_entry=None; gap=False
        # fill pending: exit first, then entry
        if pend_exit:
            if pos is not None:
                trades.append(_close(pos,t,O,'signal',slip,fee)); 
                if account: cash+= trades[-1]['proceeds']
                pos=None; in_pos=False; held=0
            pend_exit=False
        if pend_entry is not None:
            stop=pend_entry; pend_entry=None
            price=O*(1+slip)
            if price<=stop: rej+=1
            else:
                if account:
                    per_unit=(price-stop)*lot
                    by_risk=math.floor(equity*max_risk/per_unit) if per_unit>0 else 0
                    by_not=math.floor(equity*max_notional/(price*lot))
                    q=min(by_risk,by_not)
                    if q<1 or price*q*lot*(1+fee)>cash: rej+=1; q=0
                    qty=q*lot
                else: qty=const_notional/price
                if (not account) or qty>0:
                    cost=price*qty
                    if account: cash-=cost*(1+fee)
                    pos=dict(raw_in=O,fill_in=price,stop=stop,qty=qty,entry_ts=t,fee_in=cost*fee)
                    in_pos=True; held=0
        # resting stop
        if pos is not None and L<=pos['stop']:
            trades.append(_close(pos,t,min(O,pos['stop']),'stop',slip,fee))
            if account: cash+=trades[-1]['proceeds']
            pos=None; in_pos=False; held=0
        equity = cash+(pos['qty']*C if pos else 0.0) if account else equity
        # ---- strategy on_bar (uses this bar's close) ----
        s_exit=False; s_entry=None
        tr=(H-L) if pc is None else max(H-L,abs(H-pc),abs(L-pc)); pc=C
        if atr is not None: atr=(atr*13+tr)/14
        else:
            atr_seen+=1; atr_seed+=tr
            if atr_seen>=14: atr=atr_seed/14
        if strat=='vwap':
            d=t//DAY
            if d!=day: day=d; cum_pv=0.0; cum_v=0.0
            cum_pv+=(H+L+C)/3*V; cum_v+=V
            vwap=cum_pv/cum_v if cum_v>0 else None
            closes.push(C)
            if in_pos:
                held+=1
                if (vwap is not None and C>=vwap) or held>=30: s_exit=True
            elif vwap is not None and atr is not None and closes.full():
                sd=closes.sd()
                if sd and sd>0 and C<vwap-2*sd:
                    stop=C-1.5*atr
                    if stop>0 and stop<C: s_entry=stop
        else:  # momentum burst
            c6.append(C); r5=None
            if len(c6)==6 and c6[0]>0: r5=C/c6[0]-1
            if in_pos:
                held+=1
                if held>=10: s_exit=True
            elif r5 is not None and atr is not None and rets.full() and vols.full():
                sd=rets.sd()
                if sd and sd>0 and r5>3*sd and V>2*vols.mean():
                    stop=C-1.0*atr
                    if stop>0 and stop<C: s_entry=stop
            if r5 is not None: rets.push(r5)
            vols.push(V)
        if live_from is not None and t<live_from: s_exit=False; s_entry=None   # stale/backfill bars: signals discarded
        # runner handling of signals
        if s_exit and pos is not None and not pend_exit: pend_exit=True
        if s_entry is not None and pos is None and pend_entry is None:
            sig+=1
            if account:
                ok = equity>0 and price_ok(C,equity,lot,max_notional)
                if ok: pend_entry=s_entry
                else: rej+=1
            else: pend_entry=s_entry
    return trades,dict(signals=sig,rejected=rej,bars=n,final_equity=(equity if account else None),open_at_end=pos is not None)

def price_ok(C,equity,lot,max_notional): return C*lot<=equity*max_notional   # gate.check on a one-unit order
def _close(pos,t,raw_out,reason,slip,fee):
    fill_out=raw_out*(1-slip); notional_out=fill_out*pos['qty']; fee_out=notional_out*fee
    return dict(entry_ts=pos['entry_ts'],exit_ts=t,raw_in=pos['raw_in'],raw_out=raw_out,reason=reason,qty=pos['qty'],
                gross_bps=(raw_out/pos['raw_in']-1)*1e4, net=(fill_out-pos['fill_in'])*pos['qty']-pos['fee_in']-fee_out,
                fees=pos['fee_in']+fee_out, proceeds=notional_out-fee_out, notional_in=pos['fill_in']*pos['qty'])

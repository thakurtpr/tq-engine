import csv, math, collections, statistics as S, datetime as dt
E='/Users/thakur/Workspace/Trading/engine'
IST=19_800_000; DAY=86_400_000; MIN=60_000
def load(sym):
    rows=[]
    for r in csv.reader(open(f'{E}/data/intraday-nse/5m/{sym}.csv')):
        if not r[0].isdigit(): continue
        rows.append((int(r[0]),float(r[1]),float(r[2]),float(r[3]),float(r[4]),float(r[5])))
    return rows
def mod(ts): return ((ts+IST)%DAY)//MIN
def dkey(ts): return (ts+IST)//DAY
def costs(side,V):
    brk=min(V*0.0003,20.0)
    stt=round(V*0.00025+1e-9) if side=='S' else 0.0           # whole rupee, half-up
    ex=V*0.0000307; sebi=V*0.000001; stamp=V*0.00003 if side=='B' else 0.0
    return brk+stt+ex+sebi+stamp+(brk+ex+sebi)*0.18
def run(sym,cash0=1_000_000.0,slip=5e-4,risk=0.005,notional=0.20):
    rows=load(sym); rows=[r for r in rows if 555<=mod(r[0])<930]
    cash=cash0; equity=cash0; pos=None; pend=None; pend_exit=False
    day=None; hi=lo=None; entered=False; in_pos=False; trades=[]; prev_ts=None; sess=set()
    for i,(ts,O,H,L,C,V) in enumerate(rows):
        m=mod(ts); d=dkey(ts); sess.add(d)
        gap = prev_ts is not None and ts-prev_ts!=5*MIN and dkey(prev_ts)==d
        prev_ts=ts
        if gap: pend=None
        if m>=555+360: pend=None   # flatten_due(ts): >=15:15 cancels entry
        # fill pending entry at this open
        if pend is not None:
            stop=pend; pend=None; price=O*(1+slip)
            if price>stop:
                q=min(math.floor(equity*risk/(price-stop)), math.floor(equity*notional/price))
                if q>=1 and price*q+costs('B',price*q)<=cash:
                    fee=costs('B',price*q); cash-=price*q+fee
                    pos=dict(raw_in=O,fill_in=price,stop=stop,q=q,fee_in=fee,ts=ts,day=d); in_pos=True; entered=True
        # flatten at/after 15:15 or overnight
        if pos is not None and (m>=915 or pos['day']!=d):
            fo=O*(1-slip); fee=costs('S',fo*pos['q']); cash+=fo*pos['q']-fee
            trades.append(dict(g=(O/pos['raw_in']-1)*1e4,net=(fo-pos['fill_in'])*pos['q']-pos['fee_in']-fee,fees=pos['fee_in']+fee,reason='flat',ts=pos['ts'],day=pos['day'],notional=pos['fill_in']*pos['q'],slipc=(pos['fill_in']-pos['raw_in']+O-fo)*pos['q']))
            pos=None; in_pos=False
        # stop
        if pos is not None and L<=pos['stop']:
            raw=min(O,pos['stop']); fo=raw*(1-slip); fee=costs('S',fo*pos['q']); cash+=fo*pos['q']-fee
            trades.append(dict(g=(raw/pos['raw_in']-1)*1e4,net=(fo-pos['fill_in'])*pos['q']-pos['fee_in']-fee,fees=pos['fee_in']+fee,reason='stop',ts=pos['ts'],day=pos['day'],notional=pos['fill_in']*pos['q'],slipc=(pos['fill_in']-pos['raw_in']+raw-fo)*pos['q']))
            pos=None; in_pos=False
        equity=cash+(pos['q']*C if pos else 0.0)
        # strategy
        if day!=d: day=d; hi=lo=None; entered=False
        if 555<=m<570:
            hi=H if hi is None else max(hi,H); lo=L if lo is None else min(lo,L)
        elif m>=570 and not in_pos and not entered and pend is None and hi is not None and C>hi:
            nxt_open_min=m+5
            if nxt_open_min<915 and lo>0 and lo<C: pend=lo
    return trades,len(sess)
if __name__=='__main__':
    allg=[]; byday=collections.defaultdict(float)
    for sym in ('RELIANCE','HDFCBANK','ICICIBANK','INFY','TCS'):
        tr,ns=run(sym); g=[t['g'] for t in tr]
        print(f"{sym:9} sessions {ns} trades {len(tr)} mean gross {sum(g)/len(g):+.1f} bps  win(net) {sum(1 for t in tr if t['net']>0)/len(tr)*100:.0f}%  net {sum(t['net'] for t in tr):.0f}  fees {sum(t['fees'] for t in tr):.0f}  stops {sum(1 for t in tr if t['reason']=='stop')}")
        allg+=[(t['g'],t['day']) for t in tr]
    g=[x[0] for x in allg]; n=len(g); m=sum(g)/n; sd=S.stdev(g); se=sd/math.sqrt(n)
    d=collections.defaultdict(float)
    for x,dy in allg: d[dy]+=x-m
    G=len(d); secl=math.sqrt(sum(v*v for v in d.values())*G/(G-1))/n
    print(f"POOLED n={n} mean gross {m:+.2f} bps sd {sd:.1f} SE naive {se:.2f} (t {m/se:+.2f}); SE clustered by day {secl:.2f} (t {m/secl:+.2f}, {G} days); 95% CI naive [{m-1.96*se:+.1f}, {m+1.96*se:+.1f}], clustered [{m-1.96*secl:+.1f}, {m+1.96*secl:+.1f}]")
    print(f"min detectable gross edge (t=2): naive {2*se:.1f} bps, clustered {2*secl:.1f} bps")

import csv, sys
sys.path.insert(0,'.')
import pit
from decimal import Decimal as D, ROUND_HALF_UP
E=pit.E
def rows(sym):
    with open(f'{E}/data/daily-pit/{sym}.csv') as f:
        r=csv.reader(f); next(r)
        return [(a,D(b),D(c),D(d),D(e),D(g)) for a,b,c,d,e,g in r]
def wilder_atr(R,t,n=20):
    trs=[]
    for i,(d,o,h,l,c,v) in enumerate(R[:t+1]):
        pc=R[i-1][4] if i>0 else None
        trs.append(h-l if pc is None else max(h-l,abs(h-pc),abs(l-pc)))
    atr=sum(trs[:n])/n
    for tr in trs[n:]: atr=(atr*(n-1)+tr)/n
    return atr, trs[-1]
def audit(sym, show=True):
    R=rows(sym); wins=pit.plan()[sym]; f=D('0.0005'); cash=D(1000000); out=[]
    t=56
    while t<len(R)-1:
        d,o,h,l,c,v=R[t]
        if d>='2012-01-01':
            hh=max(x[2] for x in R[t-55:t]); prior=R[t-55:t]
            if c>hh:
                atr,tr=wilder_atr(R,t); stop=c-2*atr
                fd,fo=R[t+1][0],R[t+1][1]
                member=any(a<=fd<b for a,b in wins)
                px=fo*(1+f)
                print(f"\n[{sym}] SIGNAL bar {d}: close {c} > max(high of prior 55 bars {prior[0][0]}..{prior[-1][0]}) = {hh}")
                print(f"   ATR(20, Wilder) at {d} = {atr:.6f} (that bar's TR {tr}); stop = close - 2*ATR = {stop:.6f}")
                print(f"   fill bar {fd}: open {fo}; member on {fd}? {member}; fill price = open*(1.0005) = {px:.6f}")
                if not member or px<=stop: t+=1; continue
                eq=cash  # fresh account, equity = cash before any trade
                by_risk=int((eq*D('0.005')/(px-stop)).to_integral_value(rounding='ROUND_FLOOR'))
                by_cap=int((eq*D('0.20')/px).to_integral_value(rounding='ROUND_FLOOR'))
                q=min(by_risk,by_cap); print(f"   size: risk 0.5% of {eq} = {eq*D('0.005')}; per-share risk {px-stop:.6f}; by_risk {by_risk}; by_cap(20% notional) {by_cap}; qty = {q}")
                buy=px*q
                def cost(side,turn):
                    stt=(turn*D('0.001')).quantize(D(1),rounding=ROUND_HALF_UP)
                    ex=turn*D('0.00307')/100; se=turn*D('0.0001')/100
                    stamp=turn*D('0.00015') if side=='B' else D(0); dp=D(13) if side=='S' else D(0)
                    gst=(ex+se+dp)*D('0.18'); return stt+ex+se+stamp+dp+gst, dict(stt=stt,ex=ex,se=se,stamp=stamp,dp=dp,gst=gst)
                ec,eb=cost('B',buy)
                # walk forward
                j=t+1; exit_info=None; pending=False
                while j<len(R):
                    dj,oj,hj,lj,cj,vj=R[j]
                    if pending:
                        exit_info=(dj,oj*(1-f),'signal exit: prior-bar exit queued, fills at open*(0.9995)',oj); break
                    if lj<=stop:
                        raw=min(oj,stop); exit_info=(dj,raw*(1-f),f'STOP: low {lj} <= stop {stop:.4f}; exit at min(open {oj}, stop)={raw:.4f} then *(0.9995)',raw); break
                    low20=min(x[3] for x in R[j-20:j])
                    if cj<low20: pending=True; print(f"   {dj}: close {cj} < lowest low of prior 20 bars {low20} -> exit queued for next open")
                    j+=1
                if exit_info is None: print('   still open at end'); return None
                xd,xp,why,raw=exit_info; sell=xp*q; xc,xb=cost('S',sell)
                total=(ec+xc).quantize(D('0.01'),rounding=ROUND_HALF_UP)
                gross=(xp-px)*q; net=gross-total
                print(f"   EXIT {xd}: {why}; exit price {xp:.6f}")
                print(f"   gross = ({xp:.6f}-{px:.6f})*{q} = {gross:.4f}; costs buy {ec:.4f} + sell {xc:.4f} -> round once {total}; NET = {net:.2f}")
                return net
        t+=1
for s in ['RPOWER','RCOM','HINDPETRO']:
    audit(s)

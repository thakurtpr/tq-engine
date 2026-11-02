import sys, os, random, math; sys.path.insert(0,'.')
import replica as R, pit
from decimal import Decimal as D, ROUND_FLOOR
E='/Users/thakur/Workspace/Trading/engine'
HUND=D(100)

def load_universe(kind):
    if kind=='pit':
        by=pit.plan(); data={}; wins={}
        for s,w in by.items():
            p=f'{E}/data/daily-pit/{s}.csv'
            if os.path.exists(p): data[s]=R.load(p); wins[s]=w
        return data,wins
    syms=[l.strip() for l in open(E+'/universe/nifty50-2026-10-30.txt') if l.strip()]
    data={}
    for s in syms:
        p=f'{E}/data/daily/{s}.csv'
        if os.path.exists(p): data[s]=R.load(p)
    return data,None

def simulate(data, wins, order_fn, cash0=D(1000000), risk_pct=D('0.5'), slip=D(5), kill_pct=D(5),
             max_pos=5, cap=D(20), daily_loss=D(2), start='2012-01-01', end='2021-12-31', rank=None):
    f=slip/D(10000)
    strat={s:R.Don() for s in data}
    idx={s:{r[0]:i for i,r in enumerate(rows)} for s,rows in data.items()}
    for s,rows in data.items():
        for r in rows:
            if r[0]<start: strat[s].on_bar(r)
    cal=sorted({r[0] for rows in data.values() for r in rows if start<=r[0]<=end})
    cash=cash0; eq=cash0; sod=cash0; peak=cash0; kill=False
    pos={}; pend_e={}; pend_x=set(); trades=[]; curve=[]
    stats=dict(signals=0, rej_maxpos=0, rej_cash=0, rej_other=0, outside=0, zero=0)
    npos_days=[]
    for k,d in enumerate(cal):
        today=[s for s in order_fn(sorted(data), d) if d in idx[s]]
        bars={s:data[s][idx[s][d]] for s in today}
        # fills: exits then entries (per symbol, per engine order)
        for s in today:
            if s in pend_x and s in pos:
                o=bars[s][1]; px=o*(1-f)
                t=R.close_pos(pos[s],d,px,'signal',R.costs_one); t['sym']=s
                trades.append(t); cash+=px*pos[s]['qty']-t['exit_costs']; del pos[s]; strat[s].long=False
            pend_x.discard(s)
        for s in today:
            if s in pend_e:
                sig=pend_e.pop(s)
                if wins is not None and not any(a<=d<b for a,b in wins[s]): stats['outside']+=1; continue
                o=bars[s][1]; px=o*(1+f); stop=sig['stop']
                if px<=stop: stats['rej_other']+=1; continue
                by_r=(eq*risk_pct/HUND/(px-stop)).to_integral_value(ROUND_FLOOR)
                by_c=(eq*cap/HUND/px).to_integral_value(ROUND_FLOOR)
                q=max(min(by_r,by_c),D(0))
                if q==0: stats['zero']+=1; continue
                if q*px>cash: stats['rej_cash']+=1; continue
                ec=R.costs_one('B',px*q); cash-=px*q+ec
                pos[s]=dict(qty=q,entry=px,stop=stop,entry_costs=ec,entry_date=d,last=px,sig=sig); strat[s].long=True
        for s in today:
            if s in pos and bars[s][3]<=pos[s]['stop']:
                px=min(bars[s][1],pos[s]['stop'])*(1-f)
                t=R.close_pos(pos[s],d,px,'stop',R.costs_one); t['sym']=s
                trades.append(t); cash+=px*pos[s]['qty']-t['exit_costs']; del pos[s]; strat[s].long=False
        for s in today:
            if s in pos: pos[s]['last']=bars[s][4]
        e=cash+sum(p['qty']*p['last'] for p in pos.values())
        sod=eq; eq=e; peak=max(peak,eq)
        if not kill and (peak-eq)>=peak*kill_pct/HUND: kill=True
        curve.append((d,eq)); npos_days.append(len(pos))
        if k+1<len(cal):
            cands=[]
            for s in today:
                sg=strat[s].on_bar(bars[s])
                if sg is None: continue
                if sg[0]=='X':
                    if s in pos: pend_x.add(s)
                else:
                    cands.append((s,sg))
            if rank=='strength': cands.sort(key=lambda x: -(x[1][1]/x[1][3]-1))
            for s,sg in cands:
                if True:
                    if s in pend_e or s in pos: continue
                    stats['signals']+=1
                    if kill or (sod-eq)>=sod*daily_loss/HUND or (peak-eq)>=peak*kill_pct/HUND: stats['rej_other']+=1; continue
                    if len(pos)+len(pend_e)>=max_pos: stats['rej_maxpos']+=1; continue
                    pend_e[s]=dict(stop=sg[2],signal_close=sg[1],prior_high=sg[3],atr=sg[4],signal_date=d)
        else:
            for s in today: strat[s].on_bar(bars[s])
    return trades,curve,stats,npos_days

def summarize(trades,curve,cash0=D(1000000)):
    rets=[float(t['net']/(t['entry']*t['qty'])*100) for t in trades]
    n=len(rets); mean=sum(rets)/n if n else 0
    sd=math.sqrt(sum((x-mean)**2 for x in rets)/(n-1)) if n>1 else 0
    tstat=mean/(sd/math.sqrt(n)) if sd>0 else float('nan')
    wins=sum(t['net'] for t in trades if t['net']>0); loss=-sum(t['net'] for t in trades if t['net']<0)
    eqs=[float(c[1]) for c in curve]; e0=float(cash0)
    full=[e0]+eqs; dr=[full[i+1]/full[i]-1 for i in range(len(full)-1)]
    m=sum(dr)/len(dr); v=sum((x-m)**2 for x in dr)/(len(dr)-1); sh=m/math.sqrt(v)*math.sqrt(252) if v>0 else float('nan')
    pk=0; mdd=0
    for x in full: pk=max(pk,x); mdd=max(mdd,(pk-x)/pk*100)
    yrs=len(curve)/252
    return dict(n=n,mean_pct=round(mean,3),t=round(tstat,2),pf=round(float(wins/loss),2) if loss>0 else None,
                net=round(float(sum(t['net'] for t in trades))), ret_pct=round((full[-1]/e0-1)*100,1),
                cagr=round(((full[-1]/e0)**(1/yrs)-1)*100,2), mdd=round(mdd,1), sharpe=round(sh,2))

if __name__=='__main__':
    kind=sys.argv[1]
    data,wins=load_universe(kind)
    print(kind,'series',len(data))
    alpha=lambda syms,d: syms
    for kp,lab in [(D(100),'kill switch OFF'),(D(5),'kill switch 5%')]:
        tr,cv,st,npd=simulate(data,wins,alpha,kill_pct=kp)
        print(lab,'alphabetical priority:',summarize(tr,cv),st)
    # unconstrained concurrency (no position cap, huge cash) just to count simultaneous positions
    tr,cv,st,npd=simulate(data,wins,alpha,kill_pct=D(100),max_pos=10**6,cash0=D(10)**9,cap=D(20))
    import collections
    c=collections.Counter(npd)
    print('UNCONSTRAINED (independent-accounts view): trades',len(tr),'max simultaneous',max(npd),'mean',round(sum(npd)/len(npd),2),
          'days>5:',round(100*sum(1 for x in npd if x>5)/len(npd),1),'% ; days>=10:',round(100*sum(1 for x in npd if x>=10)/len(npd),1),'%')

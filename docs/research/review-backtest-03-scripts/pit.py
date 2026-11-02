import sys, csv, os; sys.path.insert(0,'.')
import replica as R
from decimal import Decimal as D
E='/Users/thakur/Workspace/Trading/engine'
def snapshots():
    snaps={}
    with open(E+'/data-ref/nifty50_members_by_date.csv') as f:
        r=csv.reader(f); next(r)
        for d,s in r: snaps.setdefault(d,set()).add(s)
    return dict(sorted(snaps.items()))
def aliases():
    a={}
    with open(E+'/data-ref/nifty50_price_aliases.csv') as f:
        r=csv.reader(f); next(r)
        for row in r: a[row[0]]=row[1]
    return a
def windows(snaps):
    syms=set().union(*snaps.values()); out={}
    for s in syms:
        ws=[]; op=None
        for d,m in snaps.items():
            if s in m and op is None: op=d
            elif s not in m and op is not None: ws.append((op,d)); op=None
        if op is not None: ws.append((op,'9999-12-31'))
        out[s]=ws
    return out
def plan(start='2012-01-01', end='2021-12-31'):
    snaps=snapshots(); al=aliases(); ws=windows(snaps); by={}
    for s,w in ws.items():
        w=[x for x in w if x[0]<=end and x[1]>start]
        if w: by.setdefault(al.get(s,s),[]).extend(w)
    return by
def table(path):
    out={}
    for line in open(path):
        if line.startswith('|') and not line.startswith('| ---') and line.count('|')>=12:
            c=[x.strip() for x in line.strip().strip('|').split('|')]
            try: out[c[0]]=dict(trades=int(c[2]), net=D(c[5]), ks=c[9], nonmem=int(c[12]))
            except Exception: pass
    return out
if __name__=='__main__':
    by=plan()
    print('price series planned:',len(by))
    for label,rep,kp in [('no-killswitch','reports/backtest-02/donchian-no-killswitch.md',D(100)),('killswitch 5%','reports/backtest-02/donchian.md',D(5))]:
        t=table(E+'/'+rep); bad=0; tr_n=0; rep_n=0; netr=D(0); netm=D(0); ran=0
        for s,w in sorted(by.items()):
            p=f'{E}/data/daily-pit/{s}.csv'
            if not os.path.exists(p): continue
            rows=R.load(p); ran+=1
            tr,curve,st=R.run(rows,'2012-01-01',windows=w,kill_pct=kp)
            net=sum(x['net'] for x in tr); r=t.get(s)
            if r is None: print('  not in report:',s); continue
            tr_n+=len(tr); rep_n+=r['trades']; netr+=net; netm+=r['net']
            if len(tr)!=r['trades'] or abs(net-r['net'])>D('0.02') or st['outside']!=r['nonmem']:
                bad+=1; print(f"  DIFF {s}: replica {len(tr)} tr net {net:.2f} outside {st['outside']} | report {r['trades']} net {r['net']} nonmem {r['nonmem']}")
        print(f"[{label}] ran {ran}; differing {bad}; trades {tr_n} vs {rep_n}; net {netr:.2f} vs {netm}")

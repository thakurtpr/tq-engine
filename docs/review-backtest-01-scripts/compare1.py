import sys, re; sys.path.insert(0,'.')
import replica as R
from decimal import Decimal as D
E='/Users/thakur/Workspace/Trading/engine'
def table(path):
    out={}
    for line in open(path):
        if line.startswith('|') and not line.startswith('| ---') and not line.startswith('| Symbol') and line.count('|')>=12:
            c=[x.strip() for x in line.strip().strip('|').split('|')]
            if c[0] in ('Metric',): continue
            try: out[c[0]]=dict(trades=int(c[2]), net=D(c[5]), ks=c[9])
            except Exception: pass
    return out
syms=[l.strip() for l in open(E+'/universe/nifty50-2026-10-30.txt') if l.strip()]
for label,rep,kp in [('no-killswitch','reports/backtest-01/donchian-no-killswitch.md',D(100)),('killswitch 5%','reports/backtest-01/donchian.md',D(5))]:
    t=table(E+'/'+rep); bad=0; tot_r=0; tot_m=0; netr=D(0); netm=D(0)
    for s in syms:
        try: rows=R.load(f'{E}/data/daily/{s}.csv')
        except FileNotFoundError: continue
        tr,curve,st=R.run(rows,'2012-01-01',kill_pct=kp)
        net=sum(x['net'] for x in tr)
        rep_=t.get(s)
        if rep_ is None: print('missing in report',s); continue
        tot_r+=len(tr); tot_m+=rep_['trades']; netr+=net; netm+=rep_['net']
        if len(tr)!=rep_['trades'] or abs(net-rep_['net'])>D('0.02'):
            bad+=1; print(f"  DIFF {s}: replica {len(tr)} trades net {net:.2f} | report {rep_['trades']} net {rep_['net']} ks={rep_['ks']} replica_ks={st['kill_at']}")
    print(f"[{label}] symbols differing: {bad}; trades replica {tot_r} vs report {tot_m}; net replica {netr:.2f} vs report {netm}")

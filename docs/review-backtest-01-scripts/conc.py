import sys, os, collections, datetime as dt; sys.path.insert(0,'.')
import replica as R, pit, portfolio as P
from decimal import Decimal as D
for kind in ['current50','pit']:
    data,wins=P.load_universe('pit' if kind=='pit' else 'cur')
    alltr=[]
    for s,rows in data.items():
        tr,cv,st=R.run(rows,'2012-01-01',windows=(wins[s] if wins else None),kill_pct=D(100))
        for t in tr: t['sym']=s
        alltr+=tr
    cal=sorted({r[0] for rows in data.values() for r in rows if '2012-01-01'<=r[0]<='2021-12-31'})
    cnt={d:0 for d in cal}
    notion={d:D(0) for d in cal}
    for t in alltr:
        for d in cal:
            if t['entry_date']<=d<=t['exit_date']:
                cnt[d]+=1; notion[d]+=t['entry']*t['qty']
    vals=list(cnt.values())
    capdays=sum(1 for d in cal if notion[d]>D(1000000))
    print(kind,'trades',len(alltr),'| open positions per day: mean',round(sum(vals)/len(vals),1),'max',max(vals),
          '| days with >5:',round(100*sum(v>5 for v in vals)/len(vals)),'% | >=10:',round(100*sum(v>=10 for v in vals)/len(vals)),'%',
          '| days with total notional > Rs 10 lakh (one account):',round(100*capdays/len(cal)),'%; max notional Rs lakh',round(float(max(notion.values()))/1e5,1))

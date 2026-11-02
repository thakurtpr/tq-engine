import sys, math, statistics as S, json, collections; sys.path.insert(0,'.')
import intraday as I
rep={('BTCUSDT','vwap'):dict(trades=16681,fees=6747.33,final=0.82,win=0.130,sig=215288,rej=198607),
     ('ETHUSDT','vwap'):dict(trades=20694,fees=6595,final=None,win=0.16,sig=None,rej=None),
     ('BTCUSDT','momentum'):dict(trades=4035,fees=5822,final=None,win=0.08,sig=None,rej=None),
     ('ETHUSDT','momentum'):dict(trades=4017,fees=6091,final=None,win=0.11,sig=None,rej=None)}
for coin in ('BTCUSDT','ETHUSDT'):
    data=I.load(coin)
    for strat in ('vwap','momentum'):
        tr,st=I.run(coin,strat,account=True,data=data)
        n=len(tr); fees=sum(t['fees'] for t in tr); net=sum(t['net'] for t in tr)
        gm=sum(t['gross_bps'] for t in tr)/n; win=sum(1 for t in tr if t['net']>0)/n
        last=tr[-1]['exit_ts'] if tr else 0
        import datetime as dt
        print(f"{coin} {strat:8} ACCOUNT replica: trades {n} fees {fees:.0f} net {net:.0f} final equity {st['final_equity']:.2f} win {win*100:.1f}% mean gross {gm:+.2f} bps signals {st['signals']} rejected {st['rejected']} last trade {dt.datetime.utcfromtimestamp(last/1000).date()}")
        print(f"    report: trades {rep[(coin,strat)]['trades']} fees {rep[(coin,strat)]['fees']} win {rep[(coin,strat)]['win']} final {rep[(coin,strat)]['final']} signals {rep[(coin,strat)]['sig']} rejected {rep[(coin,strat)]['rej']}")

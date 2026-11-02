"""Scenario net bps from the break-even run: entry and signal exits as maker, stop exits as taker + extra slippage.
Mean gross and stop shares are copied from fee_breakeven.py output (BTC/ETH VWAP, momentum). All-optimistic: maker legs fill at the
simulated open with no adverse selection and no missed fills."""
G={'BTC vwap':(0.30,0.14,0.68),'ETH vwap':(0.34,0.16,0.69),'BTC momentum':(-0.83,0.45,0.75),'ETH momentum':(0.75,0.56,0.74)}
# (name, maker bps/side, taker bps/side, slippage bps on each market leg)
S=[('Regular + BNB, all market (7.5/7.5, slip 1)',7.5,7.5,1.0,True),
   ('VIP 9 + BNB, maker entry/exit, taker stops (0.825/1.725, slip 1 on stops)',0.825,1.725,1.0,False),
   ('LP Tier 4 rebate, taker VIP9 stops (-0.8/1.725, slip 1 on stops)',-0.8,1.725,1.0,False),
   ('Flat 2 bps round trip, zero slippage',1.0,1.0,0.0,True),
   ('Flat 0 bps (no fee, no slippage)',0.0,0.0,0.0,True)]
for k,(g,se,ps) in G.items():
    print(k, f'gross {g:+.2f} (SE {se})')
    for name,mk,tk,sl,allm in S:
        if allm: cost=2*(tk+sl) if 'Regular' in name else 2*mk+ (0 if sl==0 else 2*sl)
        else: cost=mk+(1-ps)*mk+ps*(tk+sl)
        print(f'   {name:75} cost {cost:+6.2f}  net {g-cost:+6.2f}  (95% CI +/-{1.96*se:.2f})')

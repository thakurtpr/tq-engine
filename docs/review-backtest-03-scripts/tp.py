"""TrendPullback replica (from crates/strategies/src/trend_pullback.rs and crates/indicators)."""
from decimal import Decimal as D
from collections import deque
class Sma:
    def __init__(s,n): s.n=n; s.w=deque(); s.sum=D(0)
    def update(s,v):
        s.w.append(v); s.sum+=v
        if len(s.w)>s.n: s.sum-=s.w.popleft()
        return s.sum/D(s.n) if len(s.w)==s.n else None
class Ema:
    def __init__(s,n): s.n=n; s.a=D(2)/D(n+1); s.seen=0; s.seed=D(0); s.v=None
    def update(s,x):
        if s.v is not None: s.v=s.v+s.a*(x-s.v); return s.v
        s.seen+=1; s.seed+=x
        if s.seen<s.n: return None
        s.v=s.seed/D(s.n); return s.v
class Rsi:
    def __init__(s,n): s.n=n; s.prev=None; s.seen=0; s.g=D(0); s.l=D(0)
    def update(s,x):
        if s.prev is None: s.prev=x; return None
        ch=x-s.prev; s.prev=x; g=max(ch,D(0)); l=max(-ch,D(0)); n=D(s.n)
        if s.seen<s.n:
            s.seen+=1; s.g+=g; s.l+=l
            if s.seen<s.n: return None
            s.g/=n; s.l/=n
        else:
            s.g=(s.g*(n-1)+g)/n; s.l=(s.l*(n-1)+l)/n
        t=s.g+s.l
        return D(50) if t==0 else D(100)*s.g/t
class Atr:
    def __init__(s,n): s.n=n; s.pc=None; s.seen=0; s.seed=D(0); s.v=None
    def update(s,h,l,c):
        tr=h-l if s.pc is None else max(h-l,abs(h-s.pc),abs(l-s.pc)); s.pc=c; n=D(s.n)
        if s.v is not None: s.v=(s.v*(n-1)+tr)/n; return s.v
        s.seen+=1; s.seed+=tr
        if s.seen<s.n: return None
        s.v=s.seed/n; return s.v
class TP:
    def __init__(s):
        s.sma=Sma(200); s.fast=Ema(20); s.slow=Ema(50); s.rsi=Rsi(2); s.atr=Atr(14)
        s._long=False; s.held=0
    @property
    def long(s): return s._long
    @long.setter
    def long(s,v): s._long=v; s.held=0
    def on_bar(s,bar):
        d,o,h,l,c,v=bar
        sm=s.sma.update(c); f=s.fast.update(c); sl=s.slow.update(c); r=s.rsi.update(c); a=s.atr.update(h,l,c)
        if None in (sm,f,sl,r,a): return None
        if s._long:
            held=s.held+1
            if c>f or held>=10:
                s._long=False; s.held=0; return ('X',None,None)
            s.held=held; return None
        if c>sm and f>sl and r<10:
            stop=c-a*2
            if stop>0 and stop<c: return ('E',c,stop,None,a)
        return None

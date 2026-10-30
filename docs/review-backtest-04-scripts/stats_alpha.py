import sys, math, collections, statistics as S; sys.path.insert(0,'.')
import rebal as X, replica as R
from decimal import Decimal as D
SER,WIN=X.load_pit(); B=R.load(X.E+'/data/etf/NIFTYBEES.csv'); BEES={'NIFTYBEES':B}
def curve_rets(c,cash0=D(1000000)):
    full=[cash0]+[e for _,e in c]; return [d for d,_ in c],[float(full[i+1]/full[i]-1) for i in range(len(full)-1)]
runs={}
c,st,_=X.run(SER,WIN,X.momentum_targets()); runs['Momentum']=curve_rets(c)
c,st,_=X.run(BEES,None,X.buyhold('NIFTYBEES'),cap_pct=D(100)); runs['Buy&hold (cost-paying)']=curve_rets(c)
c,st,_=X.run(BEES,None,X.Faber('NIFTYBEES'),cap_pct=D(100),cash_yield=D('0.06')); runs['Trend filter, 6% cash']=curve_rets(c)
c,st,_=X.run(BEES,None,X.Faber('NIFTYBEES'),cap_pct=D(100)); runs['Trend filter, 0% cash']=curve_rets(c)
# frictionless NIFTYBEES benchmark daily returns by date
cl={r[0]:float(r[4]) for r in B}; ds=sorted(cl); prev={ds[i]:cl[ds[i-1]] for i in range(1,len(ds))}
bench={d:cl[d]/prev[d]-1 for d in ds if d in prev}
def mean(x): return sum(x)/len(x)
def hac_t(x,L):
    n=len(x); m=mean(x); d=[v-m for v in x]; g0=sum(v*v for v in d)/n; s=g0
    for l in range(1,L+1): s+=2*(1-l/(L+1))*sum(d[i]*d[i-l] for i in range(l,n))/n
    return m/math.sqrt(s/n)
def monthly_t(dates,x):
    mon=collections.OrderedDict()
    for d,r in zip(dates,x): mon[d[:7]]=mon.get(d[:7],1.0)*(1+r)
    mr=[v-1 for v in mon.values()]; return mean(mr)/(S.stdev(mr)/math.sqrt(len(mr)))
def ols_hac(y,x,L):
    n=len(y); mx,my=mean(x),mean(y); sxx=sum((a-mx)**2 for a in x); b=sum((a-mx)*(c-my) for a,c in zip(x,y))/sxx; a0=my-b*mx
    e=[c-a0-b*a for a,c in zip(x,y)]
    # HAC for [a,b]: V=(X'X)^-1 S (X'X)^-1 ; X rows [1,x]
    sx=sum(x); Sxx=sum(a*a for a in x); det=n*Sxx-sx*sx; inv=[[Sxx/det,-sx/det],[-sx/det,n/det]]
    def G(l):
        g=[[0,0],[0,0]]
        for t in range(l,n):
            u=(e[t]*1.0,e[t]*x[t]); w=(e[t-l]*1.0,e[t-l]*x[t-l])
            for i in range(2):
                for j in range(2): g[i][j]+=u[i]*w[j]
        return g
    Sm=G(0)
    for l in range(1,L+1):
        g=G(l); w=1-l/(L+1)
        for i in range(2):
            for j in range(2): Sm[i][j]+=w*(g[i][j]+g[j][i])
    def mm(A,B_): return [[sum(A[i][k]*B_[k][j] for k in range(2)) for j in range(2)] for i in range(2)]
    V=mm(mm(inv,Sm),inv)
    return a0,b,math.sqrt(V[0][0]),math.sqrt(V[1][1])
print("1) Gate-4c HAC t of daily returns at different bandwidths (report used L=616 = T/4 for rebalancers):")
for name,(dts,x) in runs.items():
    T=len(x); L0=int(4*(T/100)**(2/9))
    print(f"  {name:24} naive {mean(x)/(S.stdev(x)/math.sqrt(T)):.2f} | HAC L={L0}: {hac_t(x,L0):.2f} | L=21: {hac_t(x,21):.2f} | L=63: {hac_t(x,63):.2f} | L=252: {hac_t(x,252):.2f} | L=616: {hac_t(x,616):.2f} | monthly t {monthly_t(dts,x):.2f}")
print("\n2) Strategy minus frictionless NIFTYBEES (the proposed test), TE and power; and beta-adjusted alpha (rf=0), HAC L=8:")
for name,(dts,x) in runs.items():
    pairs=[(r,bench[d]) for d,r in zip(dts,x) if d in bench]; y=[p[0] for p in pairs]; z=[p[1] for p in pairs]; ex=[a-b for a,b in pairs]
    T=len(ex); te=S.stdev(ex)*math.sqrt(252); mex=mean(ex)*252
    a,b,sa,sb=ols_hac(y,z,8)
    print(f"  {name:24} mean excess {mex*100:6.2f}%/yr TE {te*100:5.2f}% IR {mex/te:5.2f} | excess t: HAC8 {hac_t(ex,8):5.2f} HAC616 {hac_t(ex,616):5.2f} | alpha {a*252*100:6.2f}%/yr beta {b:.2f} alpha-t {a/sa:5.2f}")
    print(f"       power: with TE {te*100:.1f}% over {T/252:.1f} yrs the SE of mean excess is {te/math.sqrt(T/252)*100:.2f}%/yr, so t>=3 needs +{3*te/math.sqrt(T/252)*100:.1f}%/yr (t>=2 needs +{2*te/math.sqrt(T/252)*100:.1f}%)")
print("\n3) Sharpe with a risk-free rate (6%/yr) subtracted from every book:")
for name,(dts,x) in runs.items():
    rf=0.06/252; ex=[v-rf for v in x]; print(f"  {name:24} Sharpe(rf=0) {mean(x)/S.stdev(x)*math.sqrt(252):.2f} | Sharpe(rf=6%) {mean(ex)/S.stdev(ex)*math.sqrt(252):.2f}")
bx=[bench[d] for d in runs['Momentum'][0] if d in bench]; print(f"  {'NIFTYBEES frictionless':24} Sharpe(rf=0) {mean(bx)/S.stdev(bx)*math.sqrt(252):.2f} | Sharpe(rf=6%) {(mean(bx)-0.06/252)/S.stdev(bx)*math.sqrt(252):.2f}")

"""Independent re-implementation of tq-backtest + DonchianBreakout (long only, daily bars).
Written from the Rust source's documented behaviour, not by reading its output."""
import csv, sys, json
from decimal import Decimal as D, ROUND_HALF_UP, ROUND_FLOOR, getcontext
from collections import deque
getcontext().prec = 28
HUND = D(100)

def rnd(x, places):  # half-up (away from zero for positives)
    return x.quantize(D(1).scaleb(-places), rounding=ROUND_HALF_UP)

def costs_one(side, turnover):
    pct = lambda r: turnover * r / HUND
    brokerage = D(0)
    if side == 'B':
        stt, stamp, dp = pct(D('0.1')), pct(D('0.015')), D(0)
    else:
        stt, stamp, dp = pct(D('0.1')), D(0), D(13)
    stt = rnd(stt, 0)
    exch = pct(D('0.00307')); sebi = pct(D('0.0001'))
    gst = (brokerage + exch + sebi + dp) * D(18) / HUND
    return brokerage + stt + exch + sebi + stamp + dp + gst

def load(path):
    rows = []
    with open(path) as f:
        r = csv.reader(f); next(r)
        for d, o, h, l, c, v in r:
            rows.append((d, D(o), D(h), D(l), D(c), D(v)))
    return rows

class Don:
    def __init__(s, entry=55, exit_=20, atr_p=20, mult=D(2)):
        s.ep, s.xp, s.ap, s.mult = entry, exit_, atr_p, mult
        s.highs, s.lows = [], []
        s.prior_high = s.prior_low = None
        s.atr = None; s.atr_seen = 0; s.atr_seed = D(0); s.prev_close = None
        s.long = False
    def atr_update(s, h, l, c):
        pc = s.prev_close
        tr = h - l if pc is None else max(h - l, abs(h - pc), abs(l - pc))
        s.prev_close = c
        n = D(s.ap)
        if s.atr is not None:
            s.atr = (s.atr * (n - 1) + tr) / n
        else:
            s.atr_seen += 1; s.atr_seed += tr
            if s.atr_seen < s.ap: return None
            s.atr = s.atr_seed / n
        return s.atr
    def on_bar(s, bar):
        d, o, h, l, c, v = bar
        r_prior_high, r_prior_low = s.prior_high, s.prior_low
        atr = s.atr_update(h, l, c)
        s.highs.append(h); s.lows.append(l)
        s.prior_high = max(s.highs[-s.ep:]) if len(s.highs) >= s.ep else None
        s.prior_low = min(s.lows[-s.xp:]) if len(s.lows) >= s.xp else None
        if s.long:
            if r_prior_low is not None and c < r_prior_low:
                s.long = False
                return ('X', None, None)
            return None
        if r_prior_high is not None and atr is not None and c > r_prior_high:
            stop = c - atr * s.mult
            if stop > 0 and stop < c:
                return ('E', c, stop, r_prior_high, atr)
        return None

def run(rows, trade_from, windows=None, cash0=D(1000000), risk_pct=D('0.5'), slip=D(5),
        kill_pct=D(5), notional_cap=D(20), daily_loss=D(2), verbose=False):
    st = Don()
    cash = cash0; pos = None
    eq = cash0; sod = cash0; peak = cash0; kill = False
    pend_entry = None; pend_exit = False
    trades = []; curve = []
    f = slip / D(10000)
    n = len(rows)
    stats = dict(outside=0, rej=0, zero=0, kill_at=None, dup=0)
    for i, bar in enumerate(rows):
        d, o, h, l, c, v = bar
        if d < trade_from:
            st.on_bar(bar); continue
        # fill pending
        if pend_exit and pos:
            px = o * (1 - f)
            trades.append(close_pos(pos, d, px, 'signal', costs_one))
            cash += px * pos['qty'] - trades[-1]['exit_costs']
            pos = None; st.long = False
        pend_exit = False
        if pend_entry:
            sig = pend_entry; pend_entry = None
            ok = windows is None or any(a <= d < b for a, b in windows)
            if not ok:
                stats['outside'] += 1
            else:
                px = o * (1 + f); stop = sig['stop']
                if px <= stop:
                    stats['rej'] += 1
                else:
                    per = px - stop
                    by_risk = (eq * risk_pct / HUND / per).to_integral_value(ROUND_FLOOR)
                    by_cap = (eq * notional_cap / HUND / px).to_integral_value(ROUND_FLOOR)
                    qty = max(min(by_risk, by_cap), D(0))
                    if qty == 0:
                        stats['zero'] += 1
                    elif qty * px > cash:
                        stats['rej'] += 1
                    else:
                        ec = costs_one('B', px * qty)
                        cash -= px * qty + ec
                        pos = dict(qty=qty, entry=px, stop=stop, entry_costs=ec, entry_date=d, last=px, sig=sig)
                        st.long = True
        # stop check
        if pos and l <= pos['stop']:
            px = min(o, pos['stop']) * (1 - f)
            t = close_pos(pos, d, px, 'stop', costs_one)
            trades.append(t); cash += px * pos['qty'] - t['exit_costs']
            pos = None; st.long = False
        # mark
        if pos: pos['last'] = c
        e = cash + (pos['qty'] * pos['last'] if pos else D(0))
        sod = eq; eq = e; peak = max(peak, eq)
        if not kill and (peak - eq) >= peak * kill_pct / HUND:
            kill = True; stats['kill_at'] = d
        curve.append((d, eq))
        # signals
        if i + 1 < n:
            s = st.on_bar(bar)
            if s is None: continue
            if s[0] == 'X':
                if pos and not pend_exit: pend_exit = True
            else:
                _, ent, stop, ph, atr = s
                if pend_entry or pos:
                    stats['dup'] += 1; continue
                # gate at signal time
                if kill: stats['rej'] += 1; continue
                if (sod - eq) >= sod * daily_loss / HUND: stats['rej'] += 1; continue
                if (peak - eq) >= peak * kill_pct / HUND: stats['rej'] += 1; continue
                pend_entry = dict(stop=stop, signal_close=ent, prior_high=ph, atr=atr, signal_date=d)
    return trades, curve, stats

def close_pos(pos, d, px, why, cf):
    exit_c = cf('S', px * pos['qty'])
    total = rnd(pos['entry_costs'] + exit_c, 2)
    exit_costs = total - pos['entry_costs']
    gross = (px - pos['entry']) * pos['qty']
    return dict(entry_date=pos['entry_date'], exit_date=d, entry=pos['entry'], exit=px, qty=pos['qty'],
                stop=pos['stop'], gross=gross, costs=total, net=gross - total, exit_costs=exit_c if False else exit_costs,
                why=why, sig=pos['sig'])

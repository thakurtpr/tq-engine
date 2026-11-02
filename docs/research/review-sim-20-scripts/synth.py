"""Task 20: synthetic recordings with hand-computed answers for tq-mm-sim (BTCUSDT only, no clock probes so offset = 0).
Writes scenario folders into OUT (a scratch directory, never engine/data) and prints the expected rows.
Run the engine on each:  ./target/release/tq-mm-sim OUT/<scenario>   and compare with the expectations printed here.
q = 0.002 BTC; fees regular 2.0 bps maker, 5.0 bps taker; latencies 50 / 100 / 200 / 400 ms."""
import gzip, json, os, sys

OUT = sys.argv[1] if len(sys.argv) > 1 else 'synth_out'
T0 = 1_793_700_000_000          # exactly an hour boundary (498,250 x 3,600,000 ms)
STREAM_DEPTH = 'btcusdt@depth@100ms'

def depth(E, U, u, pu, b=(), a=(), delay=20):
    return {'rx': E + delay, 'm': {'stream': STREAM_DEPTH, 'data': {'e': 'depthUpdate', 'E': E, 'T': E - 1, 's': 'BTCUSDT', 'ps': 'BTCUSDT', 'U': U, 'u': u, 'pu': pu,
            'b': [list(x) for x in b], 'a': [list(x) for x in a]}}}

def trade(T, px, qty, m, delay=10):
    return {'rx': T + delay, 'm': {'stream': 'btcusdt@aggTrade', 'data': {'e': 'aggTrade', 'E': T + 1, 'T': T, 's': 'BTCUSDT', 'a': 1, 'p': px, 'q': qty, 'f': 1, 'l': 1, 'm': m}}}

def preamble():
    return [{'rx': T0, 'event': 'connect', 'info': 'wss://fstream.binance.com/public/stream?streams=btcusdt@depth@100ms'},
            {'rx': T0 + 1, 'snap': 'BTCUSDT', 'm': {'lastUpdateId': 100, 'E': T0, 'T': T0, 'bids': [['100.0', '5.000'], ['99.9', '5.000']], 'asks': [['100.1', '5.000'], ['100.2', '5.000']]}},
            depth(T0 + 10, 95, 105, 90)]            # first event covers L = 100; decision stamped at rx = T0 + 30

def moves(prev_u, steps):
    """Depth events that move the market so markouts can be read: steps = [(E_offset, bids, asks)]."""
    out = []; u = prev_u
    for off, b, a in steps: out.append(depth(T0 + off, u + 1, u + 5, u, b, a)); u += 5
    return out, u

def write(name, lines):
    d = os.path.join(OUT, name); os.makedirs(d, exist_ok=True)
    with gzip.open(os.path.join(d, '2026-11-02T18.jsonl.gz'), 'wt') as f:
        for ln in lines: f.write(json.dumps(ln, separators=(',', ':')) + '\n')

market = [(1_500, [('100.0', '0'), ('99.9', '0'), ('99.0', '5.000')], [('100.1', '0'), ('100.2', '0'), ('99.2', '5.000')]),          # mid 99.1
          (11_000, [('99.0', '0'), ('98.0', '5.000')], [('99.2', '0'), ('98.2', '5.000')]),                                         # mid 98.1
          (61_000, [('98.0', '0'), ('100.9', '5.000')], [('98.2', '0'), ('101.1', '5.000')])]                                       # mid 101.0

# --- S1: queue exhaustion, conservative; L = 400 ms not yet live at the trades
l = preamble()
l.append(depth(T0 + 130, 106, 110, 105)); l.append(depth(T0 + 230, 111, 115, 110))
l.append(trade(T0 + 250, '100.0', '4.000', True)); l.append(depth(T0 + 330, 116, 120, 115)); l.append(trade(T0 + 350, '100.0', '1.003', True))
mv, _ = moves(120, market); l += mv; write('s1_queue_exhaustion', l)
print('S1 expect (S0 cell, conservative AND optimistic, L=50/100/200): fills 1, trade-through 0%, markout 1s/10s/60s = -90.00 / -190.00 / +100.00 bps, '
      'net regular = +98.00 bps (gross 0.002 on 0.2 notional, maker fee 4e-5). L=400: 0 fills, markouts n/a, net n/a.')

# --- S2: optimistic denominator: displayed size at our price falls 5 -> 1 with no trade, then a 1.001 sell at our price
l = preamble()
l.append(depth(T0 + 130, 106, 110, 105)); l.append(depth(T0 + 230, 111, 115, 110, b=[('100.0', '1.000')]))
l.append(trade(T0 + 250, '100.0', '1.001', True))
mv, _ = moves(115, market); l += mv; write('s2_optimistic', l)
print('S2 expect: conservative rows: 0 fills (ahead 5 - 1.001 = 3.999). Optimistic rows L=50/100/200: ahead = 5 x 1/5 = 1.0, then 1.0 - 1.001 -> fill 0.001 (maker notional 0.1); '
      'markout 1s: trade at 250 so first event >= 1250 is E=1500, mid 99.1 -> -90.00 bps. L=400 optimistic: 0 fills.')

# --- S3: trade-through on both sides and markout sign for a sell
l = preamble()
l.append(depth(T0 + 130, 106, 110, 105)); l.append(depth(T0 + 230, 111, 115, 110))
l.append(trade(T0 + 250, '99.9', '0.001', True))     # sell strictly below our bid at 100.0 -> bid filled in full at 100.0
l.append(trade(T0 + 260, '100.2', '0.001', False))   # buy strictly above our ask at 100.1 -> ask filled in full at 100.1
mv, _ = moves(115, market); l += mv; write('s3_trade_through', l)
print('S3 expect (S0, L=50/100/200): fills 2, trade-through 100%, buy at 100.0 and sell at 100.1 each 0.002 (maker notional 0.4002 x ... see net). '
      'Markout 1s mean = (-90.00 [buy: (99.1-100.0)/100.0] + 99.90 [sell: -(99.1-100.1)/100.1]) / 2 = +4.95 bps; '
      '10s: buy -190.00, sell +199.80 -> +4.90; 60s: buy +100.00, sell -89.91 -> +5.05.')
# cash: -100.0 x 0.002 + 100.1 x 0.002 = +0.0002 ; inventory 0 ; maker notional 0.2 + 0.2002 = 0.4002 ; fee 2e-4 x 0.4002
print('   net regular = (0.0002 - 2e-4 x 0.4002) / 0.4002 x 1e4 = %.2f bps' % ((0.0002 - 2e-4 * 0.4002) / 0.4002 * 1e4))

# --- S4: no trade, nothing should happen (sanity: zero fills, no spurious P&L)
l = preamble(); l.append(depth(T0 + 130, 106, 110, 105)); mv, _ = moves(110, market); l += mv; write('s4_no_trades', l)
print('S4 expect: 0 fills in every row.')
# --- S5: falling market, four bid trade-throughs -> inventory 4q -> taker flatten to 2q; skew cells stop bidding after 2q
l = preamble()
l.append(depth(T0 + 130, 106, 110, 105)); l.append(depth(T0 + 230, 111, 115, 110))
l.append(trade(T0 + 250, '99.9', '0.001', True))                                                    # fill 1: buy 0.002 at 100.0
l.append(depth(T0 + 500, 116, 120, 115, b=[('100.0', '0'), ('99.9', '0'), ('99.0', '5.000')], a=[('100.1', '0'), ('100.2', '0'), ('99.1', '5.000')]))
l.append(trade(T0 + 800, '98.9', '0.001', True))                                                    # fill 2: buy 0.002 at 99.0
l.append(depth(T0 + 1000, 121, 125, 120, b=[('99.0', '0'), ('98.0', '5.000')], a=[('99.1', '0'), ('98.1', '5.000')]))
l.append(trade(T0 + 1300, '97.9', '0.001', True))                                                   # fill 3 (S0 only): buy 0.002 at 98.0
l.append(depth(T0 + 1500, 126, 130, 125, b=[('98.0', '0'), ('97.0', '5.000')], a=[('98.1', '0'), ('97.1', '5.000')]))
l.append(trade(T0 + 1800, '96.9', '0.001', True))                                                   # fill 4 (S0 only): buy 0.002 at 97.0 -> inventory 4q
l.append(depth(T0 + 2200, 131, 135, 130)); l.append(depth(T0 + 3000, 136, 140, 135)); write('s5_flatten_and_skew', l)
# S0 (L=50/100/200): maker notional 0.2+0.198+0.196+0.194 = 0.788; flatten sells 0.004 at best bid 97.0 = taker notional 0.388; inventory 0.004 at mid 97.05
g0 = -0.788 + 0.388 + 0.004 * 97.05
print('S5 expect S0 (conservative and optimistic, L=50/100/200): fills 4, flattens 1, gross %.4f, net regular = (gross - 2e-4 x 0.788 - 5e-4 x 0.388) / 0.788 x 1e4 = %.2f bps'
      % (g0, (g0 - 2e-4 * 0.788 - 5e-4 * 0.388) / 0.788 * 1e4))
g1 = -0.398 + 0.002 * 0 + 0.004 * 97.05
print('S5 expect S1/S2a/S2b (skew): the bid is pulled after fill 2 (known inventory 2q), so fills 2, flattens 0, maker notional 0.398, gross %.4f, net regular = %.2f bps'
      % (g1, (g1 - 2e-4 * 0.398) / 0.398 * 1e4))
print('Output folders written under', os.path.abspath(OUT))

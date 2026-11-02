# Task 15: fee break-even study

Reviewer: thakur-op2, 2026-11-02. Docs only: nothing under `engine/` was touched and no runner (live or paper) was started.
Scripts: `docs/fee-breakeven-15-scripts/` (`fee_breakeven.py`, `scenarios.py`, output `fee_breakeven_summary.json`). They import the
task-14 replicas read-only. Crypto: Binance 1m klines 2022-2023, constant notional, every signal measured (no compounding account).
NSE ORB: 153 trades, 5 stocks, 60 sessions of Yahoo 5m bars. NOT tax advice; not investment advice.

## Verdict

**No realistic fee level makes any tested strategy profitable. The gross edge is at most about 0.3 to 0.75 bps per trade
(and zero or negative for momentum on BTC and for ORB). Only a total round-trip cost below that, which no venue offers
to a non-institutional account, would leave a profit, and the maker-fill assumptions needed to even approach it are not
testable with this data.**

| Strategy | n | Break-even round-trip cost, bps (= mean gross) | 95% CI (day-clustered) |
| --- | --- | --- | --- |
| BTCUSDT VWAP | 36,130 | **+0.30** | [+0.02, +0.58] |
| ETHUSDT VWAP | 36,999 | **+0.34** | [+0.02, +0.65] |
| BTCUSDT momentum | 4,035 | **-0.83** | [-1.72, +0.05] |
| ETHUSDT momentum | 4,017 | **+0.75** | [-0.36, +1.86] |
| NSE ORB (pooled) | 153 | **-6.86** | [-21.48, +7.76] |
| NSE ORB, intraday-only subset | 139 | -10.97 | [-26.73, +4.78] |

Break-even cost equals mean gross because net = gross - cost per trade at constant notional. "Cost" here is all-in: fees on
both legs plus slippage on both legs. Even the upper end of each crypto CI is below 2 bps, and BTC momentum and ORB
cannot be distinguished from a loss before any cost at all.

## 1. Break-even by strategy

- **VWAP** is the only one with a gross edge whose interval excludes zero, and only just (lower bounds +0.02). It sits in 2023: +0.56 / +0.58 bps;
  2022 is -0.01 / +0.07 (not significant). One year of edge, tiny, and it needs a total cost under 0.6 bps to be positive.
- **Momentum** BTC is negative gross; ETH is +0.75 with an interval that includes zero (and 2022 is negative). Not a supported edge.
- **ORB** is -6.9 bps gross with a 95% interval of about [-21, +8]. The sample cannot rule out +8 bps nor -21 bps, so a
  break-even cost is not identifiable from 60 sessions. It does not support any fee conclusion beyond "no demonstrated edge".
- The payoff shape explains the thin edge: VWAP signal exits average +25.6 / +33.5 bps (BTC / ETH) on about 32% of trades,
  while stop exits average -11.7 / -14.4 bps on about 68%. Fee savings do not change that ratio; only gross-edge improvement does.

## 2. Re-run at lower cost levels

Net bps per trade (gross minus all-in round-trip cost; 95% CI half-width in the last column; `scenarios.py`):

| Scenario (per-side bps) | BTC VWAP | ETH VWAP | BTC mom | ETH mom | CI +/- |
| --- | --- | --- | --- | --- | --- |
| Regular + BNB, all market (7.5 / 7.5, 1 bp slip per leg) | -16.7 | -16.7 | -17.8 | -16.3 | 0.3-1.1 |
| VIP 9 + BNB: maker entry and signal exits, taker stops (0.825 / 1.725, 1 bp slip on stops) | -2.6 | -2.6 | -3.9 | -2.3 | 0.3-1.1 |
| Spot Liquidity Provider Tier 4 rebate (-0.8 maker), taker stops as VIP 9 (1 bp slip on stops) | -0.5 | -0.5 | -1.9 | -0.3 | 0.3-1.1 |
| Flat 2 bps round trip, zero slippage | -1.7 | -1.7 | -2.8 | -1.3 | 0.3-1.1 |
| Flat 0 bps, zero slippage (not achievable) | +0.3 | +0.3 | -0.8 | +0.8 | 0.3-1.1 |

**Plainly: at about 2 bps the edge does not survive; at about 0 bps with a rebate it still does not survive once stop
exits are charged as taker orders plus slippage (68-75% of trades are stop exits). Only a zero-cost fantasy is
positive, by 0.3 bps for VWAP.** Scenario rows use ETH momentum's CI of +/-1.1, so even its "-0.3" row cannot be called different from zero.

Why even these tables flatter the strategies (all unmodelled, all adverse):
- **Maker fills.** The replica fills at the next bar's open as a market-style fill. A resting limit order is filled mostly
  when price moves through it, i.e. adverse selection, and misses the trades that run away. The signal-exit winners
  (+25 to +37 bps) are exactly the ones a passive order would miss. This could erase the edge or reverse it; the 1m bars cannot
  show it (needs order-book / trade-level data).
- **Stops.** Stop exits are market orders and gap through the level; the replica assumes a fill at min(open, stop).
- **Queue priority, partial fills, rate limits** on a maker-only approach are not in the data.
- **Slippage** of 1 bp per market leg for BTC/ETH is my assumption, in line with `docs/intraday-costs-and-access.md`
  but not measured here; the runner itself uses 5 bps.
- **Taxes:** at 30% flat with no loss offset (s.115BBH) and 1% TDS, an Indian resident faces much more than the exchange fee
  (see `docs/review-intraday-01.md`, section 4). Unchanged by anything here.

## 3. Holding times and what real HFT would need

| Strategy | Median hold | 90th percentile | Maximum |
| --- | --- | --- | --- |
| BTC / ETH VWAP | 7 / 6 minutes | 30 minutes | 96 / 107 minutes |
| BTC / ETH momentum | 1 / 2 minutes | 10 minutes | 10 minutes |
| NSE ORB | 260 minutes | 340 minutes | 5,435 minutes (an overnight carry; 14 of 153 trades) |

These are minutes-to-hours strategies on **closed 1-minute and 5-minute bars** with a next-bar fill. They are not HFT.
Decisions happen at bar close and fills at the next open, so a 60-second (crypto) or 300-second (NSE) decision latency is built in.
The current runner receives one bar per interval and has no order book, no tick data and no order management beyond the bar open.

HFT, by definition, trades on quote and trade events with decision-to-order latency in microseconds to a few milliseconds, which requires
at least: a tick-by-tick or order-book feed (L2/L3), co-located servers next to the matching engine, a low-latency order gateway,
and an edge measured in fractions of a basis point per trade on very large volume. A retail connection (internet, a broker API with
rate limits) cannot do this. On India, SEBI's retail-algo framework requires registration of self-developed retail algos above an
orders-per-second threshold that secondary sources put at about 10 per exchange per second (unverified: I did not read the SEBI
circular), far below HFT rates. Nothing in our data shows a microstructure edge of the kind HFT needs. Co-location fees and latency
tiers for NSE and Binance were **not verified** here.

## 4. Venues and tiers (cited only; unsure items marked)

**Verified from primary pages fetched today:**
- **Binance spot fee schedule** (https://www.binance.com/en/fee/schedule): Regular 0.100% maker / 0.100% taker; 0.075% / 0.075% with
  the BNB 25% discount. VIP 1 0.090 / 0.100%. VIP 3 0.040 / 0.060% (needs 20,000,000 USD 30-day volume and 100 BNB). VIP 5 0.025 / 0.031%
  (150,000,000 USD, 1,000 BNB). VIP 9 0.011% / 0.023% (4,000,000,000 USD, 5,500 BNB); with BNB 0.00825% / 0.01725%. The page lists no zero-fee spot pairs.
  In bps per side: regular 10 (7.5 with BNB); VIP 9 about 1.1 maker, 2.3 taker. Reaching about 2 bps round trip needs roughly VIP 5 to VIP 9
  volume, i.e. hundreds of millions to billions of USD per month.
- **Binance Spot Liquidity Provider Program** (https://www.binance.com/en/support/announcement/6311b67d26804bf48c5d62f5beb91202,
  effective 2024-09-09 / 2024-09-17): Tier 1 0.0000% maker for 0.05% weekly maker-volume share or 25,000,000 USD weekly maker volume;
  Tier 2 -0.0040%, Tier 3 -0.0060%, Tier 4 -0.0080% (1.00% share). Taker fees stay at the user's VIP level. Scope is spot and margin pairs on Binance.com.
  That is a **-0.4 to -0.8 bps rebate** at the top, and entry needs volume far above any retail account. Whether it is still current in
  November 2026 is **unverified** (the page reflects the 2024 announcement).
- **Zerodha equity intraday** (https://zerodha.com/charges): brokerage lower of 0.03% or Rs 20 per executed order; STT 0.025% on the sell side;
  NSE exchange charge 0.00307%; SEBI Rs 10 per crore; stamp 0.003% on the buy side; GST 18% on brokerage + SEBI + exchange charges.
  No maker/taker split or rebate on NSE cash equities. `docs/costs-india.md` has the all-in figure used before (about 4 bps round trip at Rs 10 lakh
  with no slippage), so sub-2-bps is impossible there.

**Not verified (do not rely on these):**
- Binance USDT-M futures tiers and any futures maker rebate: the fee page I fetched returned no table; third-party sources conflict.
- Whether a user in India can trade on Binance.com spot, or whether the regulatory status has changed since the 2024 FIU-IND registration (see `docs/intraday-costs-and-access.md`).
- Any zero-fee pair promotions on spot, other exchanges' tiers (OKX, Bybit, Coinbase), and NSE/Kite co-location or latency options.
- The SEBI retail algo threshold and its effective date (secondary sources only).

## 5. Recommendations

1. Treat these three strategies as closed: gross edge is 0.3 bps or less in the best case and is not robust across years or coins.
2. Do not spend effort on fee negotiation or VIP volume: the cost needed (under 1 bp all-in) is out of reach, and a rebate does not fix 68-75% stop exits.
3. If the owner wants a production path, the next research steps are different in kind: record tick or L2 data for a few weeks and test a
   maker-side strategy with real fill and queue modelling, then paper trade it under a pre-registered plan and the same validation gates.
4. Keep the ORB question open only if real NSE intraday history can be obtained (Kite Connect history for a month, or recorded data); do not carry positions overnight in the test.
5. No real capital, credentials or leverage until a strategy passes the validation protocol on paper.

## Reproducing

```
cd docs/fee-breakeven-15-scripts && python3 fee_breakeven.py && python3 scenarios.py
```
`fee_breakeven.py` takes about 6 seconds. It reads `engine/data/intraday/{BTCUSDT,ETHUSDT}/*-1m-*.csv` (2022-2023) and `engine/data/intraday-nse/5m/*.csv`,
and writes only `fee_breakeven_summary.json` beside itself. `scenarios.py` uses the mean gross and stop-share figures printed by the first script.

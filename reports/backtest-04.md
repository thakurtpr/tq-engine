# Backtest 04: pre-registered rebalancers vs buy-and-hold NIFTYBEES

Date: 2026-10-30. Author: index research engineer. Development window only (2012-01-01 to 2021-12-31); the
holdout was not touched. Everything can be reproduced with [`backtest-04/run.sh`](backtest-04/run.sh). There is
one CSV line per run in [summary.csv](backtest-04/summary.csv) and [seeds.csv](backtest-04/seeds.csv).

## Verdict

**Neither pre-registered strategy beats simply holding NIFTYBEES. Both FAIL. Buy the index fund.**

- **MomentumRotation** (12-1 momentum, top 10 of NIFTY 50): CAGR **14.02%** against **14.31%** for cost-paying
  buy-and-hold and 14.86% for frictionless NIFTYBEES. It had the same drawdown (36%), a lower Sharpe (0.81 vs
  0.96) and **295% one-way turnover a year**, which cost 20% of starting capital in fees and slippage over the
  decade. The HAC t of its daily return in excess of NIFTYBEES is **-0.17**: no difference from the index,
  in either direction.
- **IndexTrendFilter** (Faber 10-month rule on NIFTYBEES): CAGR **7.14%**, max drawdown 21.2% (vs 36%), Sharpe
  0.71. It halves the return for a drawdown that is lower by 15 points, and in this bull decade it whipsawed
  (it lagged in 2012, 2013, 2016, 2018 and 2019). Excess HAC t **-3.12**. A 6% liquid-fund yield on its cash
  would add about 1.8 points a year (31% of the time in cash), which is not enough to close a 7-point gap.
- **The user's 10% kill switch cannot coexist with equity-index exposure.** Buy-and-hold itself breached 10% by
  2012-05-04. See section 7 below.

## Pre-registration (fixed before any run, from the literature; nothing was tuned)

| Strategy | Rule | Parameters | Source |
| --- | --- | --- | --- |
| MomentumRotation | After the last close of each month, rank point-in-time NIFTY 50 members by `close[t-21] / close[t-252] - 1`; hold the top 10 at equal weight from the next open. A member needs 253 closes (252 bars of history) to be ranked. | lookback 252, skip 21, top 10 | Jegadeesh & Titman (1993), 12-1 formation |
| IndexTrendFilter | Hold NIFTYBEES while its month-end close is above the mean of the last 10 month-end closes (current month included); otherwise hold cash at **0%** (conservative; a liquid fund would have earned about 6%). | 10 months | Faber (2007) |
| BuyAndHold | 100% NIFTYBEES. The benchmark, not a trial. | - | - |

Ties in momentum go to the earlier ticker. The engine executes each book (`run_rebalance`, new in this round)
in one Rs 10 lakh account:

- Targets are decided after the month's last close and filled at the **next open**. Sells come first, then buys
  in priority order.
- **Every buy goes through the risk gate.** These positions have no stop, so the per-trade risk check does not
  apply; the gate instead enforces the kill switch, the daily-loss and drawdown limits, the position count and
  the notional cap of the resulting position.
- Fills pay Zerodha delivery costs and the adopted slippage model (no tick rounding on adjusted prices; see
  [backtest-03b](backtest-03b.md)).
- Weights apply to equity at the open, less a 0.5% cash buffer for costs.
- Trims or top-ups under 0.5% of equity are skipped, so that fixed fees do not dominate small trades. These two
  constants were also set before any run.
- Risk gate: momentum uses the default 20% notional cap and 10 positions. The single-fund strategies use a 100%
  notional cap (`--max-notional-pct 100`, unlevered): the cap is a single-stock concentration rule, and
  NIFTYBEES is a 50-stock basket.

## Data

- **NIFTYBEES:** fetched through the existing venv with `scripts/fetch_daily.py NIFTYBEES --start 2010-01-01
  --end 2021-12-31 --out data/etf` (Yahoo, dividend-adjusted). No ^NSEI proxy was needed. Two defects:
  1. **2019-12-19 and 2019-12-20 are printed at one tenth of the price** (12.92 and 13.49 between 129.25 and
     131.20, around the fund's unit split). This would show as a fake 90% crash. I dropped those two rows and kept
     the raw file as `data/etf/NIFTYBEES-raw.csv` (SHA-256 `85dc3592…`). The repaired file has SHA-256
     `80bcfaa005f1…`, and `tq quality` then reports no jumps.
  2. **Oct 2012 to Jun 2013: 161 of 164 bars have zero volume and o=h=l=c.** The closes still track the
     market, so I kept them. The engine counts any buy made without an ADV estimate; there were none.
- The NIFTY 50 members, prices and aliases are the same as backtest-03. Universe coverage is 95.43% of
  member-days, and HDFC Ltd is still missing.
- Manifests: momentum runs `924622021a2591f2e56827f5ac6189ecac6ab6eeef61333a677b9fd12270289b` (78 files);
  NIFTYBEES runs `4010b78e052f590f425e33f48a0647f78acc8a6ef610c73c4b938bd5c7664d63`.
- **Benchmark:** frictionless buy-and-hold of NIFTYBEES's adjusted closes. Over the decade it returned 14.86% a
  year, against 14.15% for the ^NSEI price index. That is 0.7 points of dividends net of the fund's expense
  ratio and tracking, below the TRI's estimated 1.2-1.5 points, so this is still not the TRI. All excess-return
  t-statistics are against this frictionless series, which is 0.1-0.5 points a year harder to beat than the
  cost-paying buy-and-hold run.

## Headline (kill switch off, slippage 1x; momentum: alphabetical, which equals the 20-seed median)

| | CAGR | Max DD | Sharpe | Calmar | Turnover, one-way per year | Costs, % of capital (10 y) | Slippage, % of capital (10 y) | Gate-4c t | **HAC t of excess over NIFTYBEES** |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| MomentumRotation | 14.02% | 36.0% | 0.81 | 0.39 | 295% | 14.13% | 6.04% | 2.81 | **-0.17** |
| IndexTrendFilter | 7.14% | 21.2% | 0.71 | 0.34 | 91% | 2.63% | 2.62% | 0.62 | **-3.12** |
| **BuyAndHold NIFTYBEES** (cost-paying run) | **14.31%** | 36.3% | **0.96** | **0.39** | 2.5% | 0.12% | 0.12% | 2.85 | -1.81 |
| NIFTYBEES, frictionless closes (benchmark) | 14.86% | 36.3% | 0.98 | 0.41 | - | - | - | - | - |
| ^NSEI price index (for reference) | 14.15% | 38.4% | 0.88 | 0.37 | - | - | - | - | - |

Notes on the table:

- **Gate-4c t for MomentumRotation:** HAC 4.13 (L = 616, the T/4 cap), monthly 2.81, per-trade 5.46. The monthly
  t is more than 0.5 below HAC, so the gate uses 2.81.
- **Gate-4c t for IndexTrendFilter:** HAC 1.75, monthly 2.16, per-trade 0.62, so the gate uses 0.62.
- **Buy-and-hold's -1.81 excess t** is pure friction: the 0.5% cash buffer, one round of costs, and a first buy
  cut by the 1% ADV cap. That costs it 5.7 points in 2012 (21.7% vs 27.4%).

**Gate 4c against zero is not a test of edge for a fully invested long-only book.** Buy-and-hold scores 2.85 on
it. The test that matters here is the excess-return t, and it is **-0.17** for momentum and **-3.12** for the
trend filter.

### Calendar years, % (strategy vs frictionless NIFTYBEES)

| Year | 2012 | 2013 | 2014 | 2015 | 2016 | 2017 | 2018 | 2019 | 2020 | 2021 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| MomentumRotation | 24.1 | 13.0 | 28.1 | -4.5 | 3.4 | 43.4 | -3.2 | 6.6 | 2.0 | 38.6 |
| IndexTrendFilter | 2.3 | -2.6 | 32.0 | -4.2 | -2.7 | 18.5 | -6.1 | -2.3 | 18.8 | 25.9 |
| NIFTYBEES | 27.4 | 6.5 | 31.6 | -4.3 | 4.0 | 29.9 | 4.8 | 13.6 | 15.4 | 26.0 |

Momentum beat the index in 2013, 2017 and 2021 and lost by 8-13 points a year in 2018-2020. Concentrated wins
and long dry spells are what a real but noisy premium looks like too. Ten years of one 50-stock index cannot tell
the two apart (excess t -0.17).

## Gate table (development window, in-sample; kill switch off; 1x)

| # | Gate | Threshold | MomentumRotation | IndexTrendFilter |
| --- | --- | --- | --- | --- |
| 0.1 | PIT, survivorship-safe universe | required | FAIL, as before (95.43% member-days) | n/a (one fund); data repaired as above |
| 0.2 / 0.3 | Next-open fills; costs + adopted slippage | required | PASS | PASS |
| 0.4 | Trials ledger | required | PASS (rows appended) | PASS |
| 0.5 | Holdout untouched | required | PASS | PASS |
| 0.6 | Median of >= 20 random-priority seeds; p5 CAGR > 0 | required | PASS: 20 seeds give CAGR 14.02-14.08%, gate t 2.81-2.82 (order barely matters) | n/a (one fund) |
| 3a | Closed trades (every sell, including trims, at average cost) | >= 200 | PASS (476) | **FAIL** (10) |
| 3c | Distinct symbols | >= 10 | PASS (63) | **FAIL** (1) |
| 3d | No symbol > 15% of net P&L | <= 15% | PASS (HCLTECH 12.0%) | **FAIL** (100%) |
| 3e | Net P&L > 0 without best 5 trades | > 0 | PASS (Rs 15.6 lakh) | **FAIL** (-Rs 2.70 lakh) |
| 4a | Mean net trade | >= 0.5% | PASS (13.0%; long holds) | PASS (2.4%) |
| 4b | Expectancy in R | >= +0.20R | NOT-ASSESSABLE (no stops) | NOT-ASSESSABLE |
| **4c** | **Gate-4c t (HAC; lowest-of-three)** | **>= 3.0** | **FAIL (2.81)** | **FAIL (0.62)** |
| 4d | Profit factor | >= 1.3 | PASS (1.90) | PASS (1.85) |
| 4e | Sharpe, daily portfolio | >= 0.8 | PASS, barely (0.81) | **FAIL** (0.71) |
| **4f** | **Sharpe and Calmar above buy-and-hold** | **both higher** | **FAIL** (0.81 vs 0.98; 0.39 vs 0.41) | **FAIL** (0.71 vs 0.98; 0.34 vs 0.41) |
| - | Excess return over NIFTYBEES, HAC t | (reported) | -0.17 | -3.12 |
| 5b | 2x slippage: still net-profitable, t > 0 | required | PASS (CAGR 13.70%; 3x 13.37%); not fragile | PASS (6.92%; 3x 6.70%) |
| 7 | Monte Carlo 95th-percentile MDD <= 0.8 x kill switch (8%) | required | **FAIL** (realised MDD alone is 36%) | **FAIL** (21%) |
| 2, 5a, 6 | Walk-forward, parameter neighbourhood, DSR/PBO | | NOT-MEASURED: fixed parameters, and 4c/4f already fail | |

The trade diagnostics (3a, 4a, 4d) were designed for stopped swing trades. For a monthly rebalancer a "trade" is a
sale at average cost, often after a long hold, so these passes say little. The deciding gates are 4c, 4f and the
excess t.

## Execution sensitivity: the ADV cap on an index ETF

The adopted model cuts entries at 1% of 20-day ADV, a rule written for single stocks. NIFTYBEES's Yahoo volume
averaged only about Rs 2.8 crore a day in 2012-14, so the cap cut 19 trend-filter entries and buy-and-hold's first
purchase. Real ETF liquidity also comes from market makers and creation/redemption. With the cap lifted
(`--adv-cap-pct 100`; the square-root impact term still charges every order above 0.05% of ADV):

| | CAGR (cap 1% / cap 100%) | Sharpe | Max DD | Excess HAC t |
| --- | --- | --- | --- | --- |
| IndexTrendFilter | 7.14% / 7.22% | 0.71 / 0.68 | 21.2 / 21.4% | -3.12 / -3.15 |
| BuyAndHold | 14.31% / 14.72% | 0.96 / 0.98 | 36.3 / 36.3% | -1.81 / -1.36 |

No verdict changes. An independent Python replica of the Faber rule (frictionless, full investment) gives a
**7.79% CAGR with 21 switches**, consistent with the engine's 7.1-7.2% after costs, slippage and the cash buffer.

## Capital scaling (MomentumRotation, kill switch off, 1x)

| Capital | Fills priced by impact | Entries cut to 1% ADV | CAGR | Max DD | Sharpe | Excess HAC t |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rs 10 lakh | 14 / 951 | 0 | 14.02% | 36.0% | 0.81 | -0.17 |
| Rs 1 crore | 484 / 956 | 0 | 13.93% | 36.1% | 0.81 | -0.21 |
| Rs 10 crore | 900 / 1,029 | 204 | 12.48% | 35.2% | 0.76 | -1.12 |

At Rs 10 crore the momentum book is liquidity-bound: 204 entries are cut, exposure falls to 92%, and costs plus
slippage reach 25% of capital over the decade.

## Section 7: the 10% kill switch (user decision) and an index-like book

Operating runs with the switch at 10% (1x):

| | Latches on | CAGR to 2021 | Max DD | What happens after |
| --- | --- | ---: | ---: | --- |
| MomentumRotation | 2013-08-07 | 3.29% | 15.4% | Sells continue, buys are blocked, so the book decays into cash |
| IndexTrendFilter | 2012-05-14 | -0.94% | 10.5% | Exits to cash and can never re-enter |
| BuyAndHold | 2012-05-04 | 14.31% | 36.3% | Never trades again, so the latch changes nothing; the account rides a 36% drawdown |

The kill switch blocks new exposure; it does not liquidate. A fully invested equity book will breach 10% in
most years: NIFTY's drawdowns this decade reached 36-38%. **Any strategy with index-like exposure fails section
7 at full size.** Passing would take roughly 20-25% gross exposure, a wider switch, or a switch that liquidates
and resets under human review. That is the user's decision; raising the limit after seeing these drawdowns is not
allowed under the protocol.

## Trials ledger and multiple testing

36 rows were appended to `research/trials.csv`:

- 2 new independent ideas: MomentumRotation and IndexTrendFilter, at kill switch off and 1x.
- 34 re-measurements: stress, kill-switch, seed and scaling runs, each labelled in the result column.
- Buy-and-hold rows are marked BENCHMARK.

**Independent trials so far: N = 4** (6 if the swing strategies' 5 or 10 position limit counts as a choice).
`per_period_sharpe` is the portfolio daily Sharpe; momentum's is 0.051. No deflated Sharpe is computed, because
nothing passes 4c or 4f.

## Recommendations

1. **Hold the index fund.** Neither idea beats NIFTYBEES net of costs over 2012-2021. Do not tune either one: a
   different lookback, top-N or SMA length is a new trial.
2. **Protocol amendment to consider** (the user's decision, in `docs/`): for fully invested long-only strategies,
   gate 4c should test the **excess return over the benchmark**, not the return against zero. Buy-and-hold
   scores 2.85 on the current 4c.
3. **The user must reconcile the 10% kill switch with any equity-index allocation** (section 7) before another
   index-like strategy is worth testing.
4. **Data still open:** NIFTY 50 TRI; HDFC Ltd prices; a clean NIFTYBEES series from NSE bhavcopy (volumes for
   2012-13, the split dates).

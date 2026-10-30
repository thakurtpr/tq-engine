# Backtest 03b: addendum to backtest-03 (review fixes)

Date: 2026-10-30. Author: index research engineer. Responds to [the review of backtest-03](../docs/review-backtest-03.md).
Same strategies (default parameters, nothing tuned), same point-in-time universe, same Rs 10 lakh account and
development window (2012-01-01 to 2021-12-31). The holdout was not touched. Everything below can be reproduced
with [`backtest-03b/run.sh`](backtest-03b/run.sh). One CSV line per run is in
[summary.csv](backtest-03b/summary.csv), [seeds.csv](backtest-03b/seeds.csv) and
[scaling.csv](backtest-03b/scaling.csv). All runs read data manifest `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`
(the same 78 files as backtest-03).

## Verdict (unchanged, wording corrected)

**No demonstrated edge, for either strategy. Both FAIL every edge gate on the median of 20 random-priority
seeds, and neither comes close to the index.** The review's corrections are accepted.

- **Donchian, 5 positions:** median CAGR **0.90%** over 20 seeds (5th-95th percentile -0.37% to 2.07%), gate-4c
  t **0.39** (-0.02 to 0.80). NIFTY 50 price index: 14.15%.
- **Donchian, 10 positions: it does NOT "lose money"; that was an ordering artefact.** The median CAGR is
  **1.97%** (5th-95th percentile 0.33% to 3.56%; 19 of 20 seeds positive), with a gate-4c t of **0.59**
  (0.22 to 0.85), a Sharpe of 0.23 and a max drawdown of 39%. That is still not distinguishable from zero, and
  still about 12 points a year behind the index. backtest-03's alphabetical figure (-0.41%) was below every seed
  the review drew. Without tick rounding the alphabetical order gives +0.09%.
- **TrendPullback** loses money in all 80 seed runs (median CAGR -3.30% at 5 positions, -4.00% at 10).

## What changed in the engine

1. **Gate 4c now follows protocol v2 to the letter** (`tq_backtest::tstats`). Every report prints all of
   these, and the gate uses the last one:
   - the primary: a Newey-West (Bartlett) HAC t of the daily portfolio returns, with bandwidth
     `L = max(floor(4 (T/100)^(2/9)), longest holding period)`, capped at T/4;
   - the monthly-return t;
   - the per-trade t, clustered by entry month;
   - the gate value: HAC, or the lowest of the three when a cross-check is more than 0.5 below it.

   The HAC t of the strategy-minus-benchmark daily return is printed too. The engine reproduces the review's
   `t_v2.py` figures: Donchian 5, alphabetical, with the tick: HAC 0.60 (L = 198), monthly 0.59, per-trade 0.49.
2. **Tick rounding: dropped on adjusted prices** (decision). Every price file here is dividend-adjusted
   Yahoo data. Those prices are off the exchange's Rs 0.05 grid and often far below the traded price, so a tick
   charge on them is friction that real fills would not pay. `--price-basis adjusted` (the default) skips
   the tick. `--price-basis unadjusted` keeps it for raw traded prices; with it, backtest-03 reproduces to the
   paisa (Donchian 5: final equity Rs 11,26,048.63). The effect on Donchian 5 is small: CAGR 1.19% with the
   tick, 1.27% without, and slippage falls from 4.36% to 3.74% of capital over ten years. Tick rounding comes
   back when we use unadjusted data with explicit corporate-action adjustment.
3. **Liquidity counters:** fills priced by the impact term (above 0.05% of 20-day ADV), entries cut to the 1%
   ADV cap, turnover, and costs and slippage as % of capital. They are printed in every report and in the CSV.
4. **Risk defaults (user decision, 2026-10-30):** 0.5% risk per trade and a **10%** drawdown kill switch.
   The other limits are unchanged and still unconfirmed for live use. The kill-switch-on runs below use 10%.
5. `--adv-cap-pct` is a labelled sensitivity knob (the adopted model keeps 1%). It is used only in backtest-04.

## Gate 4c, all statistics (kill switch off, 1x, adjusted prices, alphabetical order)

| Configuration | HAC t (L) | Monthly-return t | Per-trade clustered t | **Gate-4c t** | Month-clustered daily t (old) | HAC t of excess over ^NSEI |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Donchian 5 | 0.63 (198) | 0.62 | 0.51 | **0.63** | 0.61 | -4.06 |
| Donchian 10 | 0.18 (218) | 0.21 | 0.16 | **0.18** | 0.20 | -5.28 |
| TrendPullback 5 | -1.90 (10) | -1.80 | -1.88 | **-1.90** | -1.86 | -3.93 |
| TrendPullback 10 | -1.89 (10) | -1.84 | -1.74 | **-1.89** | -1.83 | -4.45 |

No cross-check falls more than 0.5 below the HAC t in any configuration, so the lowest-of-three rule does not
bind here. Note that the excess-return t is far more negative under HAC (-4.06) than month-clustered (-2.89,
backtest-03). Either way the strategies trail the index decisively.

## Rule 0.6: judged on the median of 20 random-priority seeds (kill switch off, 1x)

| Configuration | CAGR median (p5 to p95) | Max DD median (p5 to p95) | Sharpe median | Gate-4c t median (p5 to p95) | CAGR > 0 | Trades median |
| --- | --- | --- | ---: | --- | ---: | ---: |
| Donchian 5 | **0.90%** (-0.37 to 2.07) | 26.2% (20.4 to 30.2) | 0.16 | **0.39** (-0.02 to 0.80) | 17 / 20 | 300 |
| Donchian 10 | **1.97%** (0.33 to 3.56) | 39.1% (33.1 to 45.8) | 0.23 | **0.59** (0.22 to 0.85) | 19 / 20 | 507 |
| TrendPullback 5 | **-3.30%** (-4.38 to -2.73) | 34.1% (29.5 to 38.7) | -0.62 | **-1.87** (-2.55 to -1.56) | 0 / 20 | 1,545 |
| TrendPullback 10 | **-4.00%** (-4.48 to -3.36) | 38.9% (36.7 to 42.7) | -0.48 | **-1.55** (-1.75 to -1.28) | 0 / 20 | 2,350 |

Percentiles are linearly interpolated over 20 values, so "p5" is roughly the second-worst seed. Rule 0.6
also requires net CAGR > 0 at the 5th percentile. That fails for Donchian 5 (-0.37%) and both TrendPullback
configurations, and holds for Donchian 10 (0.33%), which fails on every edge gate anyway.

## Gate table, corrected (seed medians; in-sample, development window)

| # | Gate | Threshold | Donchian 5 | Donchian 10 | TrendPullback 5 | TrendPullback 10 |
| --- | --- | --- | --- | --- | --- | --- |
| 0.6 | Median of >= 20 seeds; p5 CAGR > 0 | required | median used; **p5 FAIL** (-0.37%) | median used; p5 0.33% | **p5 FAIL** | **p5 FAIL** |
| 4c | Gate-4c t (HAC, lowest-of-three) | >= 3.0 | **FAIL** (0.39) | **FAIL** (0.59) | **FAIL** (-1.87) | **FAIL** (-1.55) |
| 4e | Sharpe, daily portfolio | >= 0.8 | **FAIL** (0.16) | **FAIL** (0.23) | **FAIL** (-0.62) | **FAIL** (-0.48) |
| 4f | Sharpe and Calmar above NIFTY 50 | both higher | **FAIL** (0.16 vs 0.88) | **FAIL** (0.23 vs 0.88) | **FAIL** | **FAIL** |
| 5b | 2x slippage: still net-profitable, t > 0 | required | PASS on net return (alphabetical: 9.45%, t 0.48) | **FAIL** (alphabetical: -5.47%) | **FAIL** | **FAIL** |
| 3a-3e, 4a-4d | Trade diagnostics | | as backtest-03 (fail) | as backtest-03 (fail) | fail | fail |

The 2x and 3x stress runs use the alphabetical order only. Seeds were run at 1x.

## Wording corrections to backtest-03

1. **"With 10 positions it loses money"** (verdict) and the 10-position column: withdrawn. The correct statement is
   above: CAGR 0.3% to 3.6% across orderings (median 2.0%), not distinguishable from zero.
2. **"Twice the positions, twice the stop exits"** as the explanation of the gap to the review's 2.3%: mostly wrong.
   The ordering explains most of it (the seed spread is about 3.2 points of CAGR at 10 positions). The adopted
   slippage model explains about 0.3-0.6 points.
3. **"Fragile"** must use **net return**, as `slippage-model.md` defines it ("net return falls by more than half
   from 1x to 2x"). backtest-03 used closed-trade P&L, which excludes about Rs 83,000 of open gains. On total
   net return, Donchian 5 falls from **13.43% (1x) to 9.45% (2x)**, a 30% fall, so it is **not fragile at 2x**. It
   falls 59% by 3x (5.53%). Donchian 10 goes negative at 2x (0.86% to -5.47%), which is a 5b failure rather than
   fragility.
4. **"Rejected 6,102 of 6,707 entry signals"** counts **signal-days**, not signals. A stock that stays above its
   channel while every slot is full re-signals each day, so distinct blocked opportunities are fewer. The
   conclusion (capacity-bound, mean 4.15 of 5 slots in use) stands.
5. **Ledger N.** The trials ledger now has 212 rows, but **N for the deflated Sharpe is the number of independent
   ideas, not rows.** Through backtest-04 that is **N = 4**: Donchian, TrendPullback, MomentumRotation and
   IndexTrendFilter, all at fixed parameters. It is 6 if the 5- versus 10-position limit counts as a choice for
   the two swing strategies. Seeds, slippage stress, kill-switch variants, reproductions and capital-scaling runs
   are re-measurements of the same idea and are labelled as such in the ledger's result column. Buy-and-hold is a
   benchmark, not a trial.
6. **5b with doubled fees** is still an estimate. A fee multiplier needs a change in the cost model, which belongs
   to the costs owner, so it is not run here.

## The impact term is dormant at Rs 10 lakh

At the backtest-03 capital, the adopted model is in practice "5 bps, 2x on stops". Only **5 of 580 fills**
(Donchian 5) and **14 of 1,052** (Donchian 10) exceed 0.05% of 20-day ADV, and the 1% ADV cap never binds.
Read "the ADV cap never bound" as "the liquidity model was not exercised", not as evidence that liquidity is fine.

### Capital scaling: Rs 1 crore and Rs 10 crore (Donchian, kill switch off, 1x, alphabetical)

| Capital | Positions | Fills priced by impact | Entries cut to 1% ADV | CAGR | Max DD | Sharpe | Slippage (% of capital) | Costs (% of capital) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Rs 10 lakh | 5 | 5 / 580 | 0 | 1.27% | 21.8% | 0.21 | 3.74% | 7.06% |
| Rs 1 crore | 5 | 351 / 580 | 0 | 1.17% | 22.1% | 0.19 | 4.86% | 6.63% |
| Rs 10 crore | 5 | 568 / 580 | **95** | 0.65% | 18.8% | 0.13 | 7.26% | 5.40% |
| Rs 10 lakh | 10 | 14 / 1,052 | 0 | 0.09% | 45.5% | 0.07 | 6.43% | 12.01% |
| Rs 1 crore | 10 | 583 / 1,062 | 1 | -0.36% | 45.8% | 0.03 | 8.25% | 11.06% |
| Rs 10 crore | 10 | 1,057 / 1,082 | **194** | -1.25% | 47.8% | -0.07 | 12.16% | 8.93% |

The impact term starts to price most fills at Rs 1 crore, and the ADV cap binds at Rs 10 crore. There it cuts
95 of 288 entries (5 positions) and lowers exposure from 43% to 37%. Slippage roughly doubles from Rs 10 lakh to
Rs 10 crore. The model now responds to size as designed, but these are model outputs, not measurements: the
square-root coefficient is from the literature (`slippage-model.md`) and has not been calibrated on our fills.

## Kill switch at the user's 10% (operating runs, 1x, alphabetical)

| Configuration | Latches on | Trades before latching | CAGR to 2021 | Max DD |
| --- | --- | ---: | ---: | ---: |
| Donchian 5 | 2015-10-05 | 111 | 0.64% | 10.7% |
| Donchian 10 | 2015-04-21 | 171 | 1.68% | 11.6% |
| TrendPullback 5 | 2013-06-11 | 217 | -0.89% | 10.2% |
| TrendPullback 10 | 2013-02-07 | 266 | -0.94% | 10.9% |

At 10% the switch trades for 1-4 years instead of a few months (at 5% it latched within months). Each run still
latches within the first 40% of the window and sits in cash afterwards. A backtest kill switch never resets; in
live use a human reviews and resets it, so these runs describe the latch, not a strategy.

## Caveats (carried over)

- The benchmark is the price index (^NSEI); the TRI is about 1.2-1.5% a year higher. backtest-04 uses
  dividend-adjusted NIFTYBEES as a closer proxy.
- The universe misses 4.57% of member-days (HDFC Ltd for the whole decade, among others).
- 120 monthly clusters cannot resolve a small true edge.

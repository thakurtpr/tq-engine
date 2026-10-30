# Backtest 03: one shared account, adopted slippage, NIFTY 50 benchmark

Date: 2026-10-30. Author: portfolio engineer. Responds to [the independent review](../docs/review-backtest-01.md).
Same strategies with **default parameters (nothing tuned)**, same point-in-time universe, costs and risk per
trade as [backtest-02](backtest-02.md). What changed: every result below is **one Rs 10 lakh account**
holding at most 5 or 10 positions at once; fills pay the adopted slippage model; returns are compared
with the NIFTY 50. Development window only (2012-01-01 to 2021-12-31); the holdout was not touched.

## Verdict

**No demonstrated edge, for either strategy. Both FAIL, and neither beats buying the index.**

- **Donchian**, max 5 positions, kill switch off (its best configuration): CAGR **1.19%**, Sharpe **0.20**,
  max drawdown **22.0%**, month-clustered t of daily returns **0.58**. The NIFTY 50 *price* index made
  **14.15%** a year (Sharpe 0.88) over the same dates. Across 20 random priority orders the CAGR ranges
  from -0.67% to 2.05% (median 0.82%). With 10 positions it loses money (CAGR -0.41%, max DD 45.6%).
- **TrendPullback** loses money in every configuration: CAGR -3.7% (5 positions) and -5.3% (10 positions)
  at 1x slippage. Its total friction (costs plus slippage) is about three times its gross profit.
- **The 5% kill switch and a shared portfolio cannot coexist** (review, point 3c): with it on, every run
  trips the switch between February and May 2012, after 14-75 trades, and then sits in cash for nearly ten years.
- The review's shared-account numbers are confirmed: the engine reproduces the reviewer's `portfolio.py` to
  the trade (section "Cross-check").

## What is new in the engine

1. **Portfolio mode** (`tq backtest --mode portfolio`, now the default): one cash balance and one
   `AccountState` across all symbols on a unified date calendar. Same bar order as before (exits, then
   entries at the open; stops; mark to market at the close; signals for the next open), so the position cap,
   the 2% daily loss limit and the drawdown kill switch bind across the portfolio. The per-symbol engine is
   now the one-symbol case of the same code. Per-symbol mode remains (`--mode per-symbol`) and its report is
   headed "DIAGNOSTIC, NOT A PERFORMANCE CLAIM".
2. **Priority rule** when there are more same-day entry signals than slots or cash: **alphabetical by
   ticker** (deterministic, unrelated to the signal, so it neither helps nor hurts the strategy). Candidates
   are offered to the gate in that order and fill at the next open in that order. `--priority random --seed N`
   shuffles each day's candidates (SplitMix64, seed and date), for the sensitivity analysis below.
3. **Adopted slippage model** ([slippage-model.md](../docs/slippage-model.md), section 5): 5 bps per side up to
   0.05% of 20-day ADV, square-root impact above it (half-spread 1.5 + open noise 2.0 + 0.7 x sigma x
   sqrt(q/ADV), in bps, never below the 5 bps base), 2x on stop exits, entries cut to 1% of 20-day ADV and
   counted, filled prices rounded to the Rs 0.05 tick against the trader, and ADV and sigma from bars before
   the fill bar only. `--slippage-mult` scales all of it. At Rs 10 lakh the ADV cap never bound (0 entries cut).
4. **Portfolio statistics**: CAGR (calendar years), Sharpe of daily returns (sqrt 252, rf 0), max drawdown,
   Calmar, exposure (mean invested value / equity), and a **month-clustered t of daily portfolio returns**:
   the review's `tstat.py` estimator (`cluster_t`, CR1 factor G/(G-1)), with clusters = calendar months
   (G = 120). Trade-level stats also show the review's entry-month clustered t.
5. **Report template fixes** (review, section 5): friction is (costs + slippage) / pre-slippage gross; the
   per-symbol diagnostic shows no rupee sums across accounts (only % of combined capital); and per-symbol
   runs mark all edge gates NOT-ASSESSABLE.
6. **Benchmark**: ^NSEI fetched with the existing yfinance script (`fetch_daily.py` now accepts `^` index
   symbols). **This is the price index, not the TRI.** It understates the total-return index by the dividend
   yield (roughly 1.2-1.5% a year), so beating it is necessary, not sufficient.
7. **Data archive**: every run writes `reports/backtest-03/manifests/manifest-<sha12>.csv` (file, data rows,
   SHA-256 of each input CSV) and cites the manifest's SHA-256 in its report. All portfolio runs below share
   manifest `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf` (78 files: 75 price
   series, membership, aliases, NSEI). `data/` is still not committed; the manifest proves which bytes a
   report used.

Money stays `Decimal`. `f64` is used only for statistics (sigma and the square root in the impact term,
Sharpe, CAGR, t-statistics), as documented in `slippage.rs` and `returns.rs`.

## Commands

```sh
.venv/bin/python scripts/fetch_daily.py '^NSEI' --start 2011-01-01 --end 2021-12-31 --out data/index
U="--data-dir data/daily-pit --universe-file data-ref/nifty50_members_by_date.csv \
   --aliases data-ref/nifty50_price_aliases.csv --start 2012-01-01 --end 2021-12-31 --risk-pct 0.5 \
   --benchmark data/index/NSEI.csv"
# Grid: strategy x max positions x kill switch (5% = on, 100% = off) x slippage multiple
for st in donchian trend-pullback; do for mp in 5 10; do for k in on off; do for m in 1 2 3; do
  kp=$([ $k = on ] && echo 5 || echo 100); name=$st-p$mp-kill$k-slip${m}x
  ./target/release/tq backtest --strategy $st $U --max-positions $mp --max-drawdown-pct $kp \
      --slippage-mult $m --summary-csv reports/backtest-03/summary.csv --label $name \
      --out reports/backtest-03/$name.md
done; done; done; done
# Priority sensitivity: Donchian, 5 positions, 1x slippage, seeds 0-19, kill switch off and on
for k in 100 5; do for seed in $(seq 0 19); do
  ./target/release/tq backtest --strategy donchian $U --max-positions 5 --max-drawdown-pct $k \
      --priority random --seed $seed --summary-csv reports/backtest-03/seeds.csv --label donchian-p5-kill$k-seed$seed
done; done
# Cross-check against the review (flat 5 bps, as portfolio.py): --slippage-model flat, files crosscheck-*.md
```

Per-run reports: `reports/backtest-03/<strategy>-p<N>-kill<on|off>-slip<M>x.md`; one line per run in
[summary.csv](backtest-03/summary.csv) and [seeds.csv](backtest-03/seeds.csv).

## Headline: one shared account, adopted slippage 1x

| Strategy | Max positions | Kill switch (5% DD) | Trades | CAGR | Max DD | Sharpe | t (month-clustered, daily) | t of excess over ^NSEI | Calmar |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Donchian | 5 | on (trips 2012-05-16) | 17 | -0.33% | 6.4% | -0.19 | -1.07 | -2.83 | -0.05 |
| Donchian | 5 | off | 288 | **1.19%** | 22.0% | 0.20 | 0.58 | -2.89 | 0.05 |
| Donchian | 10 | on (trips 2012-02-27) | 14 | -0.15% | 7.0% | -0.06 | -0.36 | -2.80 | -0.02 |
| Donchian | 10 | off | 530 | -0.41% | 45.6% | 0.02 | 0.07 | -3.22 | -0.01 |
| TrendPullback | 5 | on (trips 2012-05-09) | 46 | -0.40% | 5.5% | -0.42 | -1.01 | -2.86 | -0.07 |
| TrendPullback | 5 | off | 1,541 | -3.67% | 35.8% | -0.69 | -1.98 | -4.17 | -0.10 |
| TrendPullback | 10 | on (trips 2012-05-16) | 75 | -0.49% | 6.0% | -0.35 | -0.99 | -2.89 | -0.08 |
| TrendPullback | 10 | off | 2,359 | -5.29% | 44.0% | -0.67 | -2.05 | -4.73 | -0.12 |
| **^NSEI buy and hold** (price index) | - | - | - | **14.15%** | 38.4% | **0.88** | 2.77 | - | 0.37 |

Exposure (mean invested / equity, kill switch off): Donchian 43% (5 positions) and 71% (10); TrendPullback
23% and 34%. Mean open positions: Donchian 4.15 of 5 and 7.19 of 10; TrendPullback 2.45 and 3.68. Donchian
is capacity-bound: at 5 positions it rejected 6,102 of 6,707 entry signals for "max open positions".

Per-trade view of Donchian, 5 positions, kill off: 288 trades, mean net trade 0.41%, +0.05R, profit factor
1.04, entry-month clustered t 0.49. Closed trades net only Rs 43,197; the 12.6% total return also includes
about Rs 83,000 of unrealised gain in positions open on 2021-12-31. Without its best 5 trades the closed-trade
P&L is -Rs 2.05 lakh. Friction (costs Rs 69,701 + slippage Rs 43,416) is 72% of the pre-friction gross.

Calendar years (Donchian, 5, off): the strategy beat the index in 2013 only (8.9% vs 6.8%). In 2012, 2014,
2017 and 2021 it trailed by 14-29 points. Full year tables are in each run's report.

## Slippage stress (kill switch off)

| Strategy, positions | CAGR 1x / 2x / 3x | Sharpe 1x / 2x / 3x | Max DD 1x / 3x | Mean net trade 1x / 2x / 3x | PF 1x / 2x / 3x |
| --- | --- | --- | --- | --- | --- |
| Donchian, 5 | 1.19 / 0.85 / 0.50% | 0.20 / 0.15 / 0.11 | 22.0 / 24.6% | 0.41 / 0.29 / 0.16% | 1.04 / 1.01 / 0.97 |
| Donchian, 10 | -0.41 / -1.23 / -1.37% | 0.02 / -0.05 / -0.06 | 45.6 / 50.2% | -0.01 / -0.16 / -0.20% | 0.94 / 0.90 / 0.89 |
| TrendPullback, 5 | -3.67 / -5.21 / -6.65% | -0.69 / -1.00 / -1.30 | 35.8 / 50.9% | -0.25 / -0.36 / -0.47% | 0.83 / 0.76 / 0.70 |
| TrendPullback, 10 | -5.29 / -7.61 / -9.90% | -0.67 / -1.00 / -1.33 | 44.0 / 65.7% | -0.20 / -0.32 / -0.43% | 0.84 / 0.77 / 0.70 |

Donchian at 5 positions is **fragile** by the slippage doc's definition. Closed-trade net P&L falls from
Rs 43,197 (1x) to Rs 9,142 (2x), more than half, and to -Rs 24,874 at 3x. Doubling fees as well as
slippage (protocol 5b) would make 2x negative (Rs 9,142 - Rs 67,917 of extra costs). So **5b FAILS** for
Donchian. TrendPullback is negative before any stress.

## Priority sensitivity: Donchian, 5 positions, 1x, 20 random seeds

| Kill switch | Trades | CAGR | Max DD | Sharpe | t (month-clustered) |
| --- | --- | --- | --- | --- | --- |
| off: min / median / max | 282 / 300 / 316 | -0.67 / **0.82** / 2.05% | 19.6 / 26.5 / 31.8% | -0.05 / 0.15 / 0.30 | -0.16 / 0.44 / 0.93 |
| on: min / median / max | 13 / 16.5 / 19 | -0.56 / -0.27 / -0.19% | 5.1 / 5.9 / 6.7% | -0.53 / -0.18 / -0.11 | -1.44 / -0.72 / -0.51 |

The alphabetical baseline (CAGR 1.19%, Sharpe 0.20, max DD 22.0%) sits above the seed median: alphabetical
was mildly lucky, not unlucky. Which signals get the five slots moves CAGR by about 2.7 points and max
drawdown by about 12 points. That is larger than any plausible edge, so a single-order backtest of this
strategy cannot be read to better than about +/-1.5 points of CAGR. No seed reaches a Sharpe of 0.31 or a t of 1.

## Cross-check against the review's `portfolio.py`

Both run Donchian on the point-in-time universe with flat 5 bps, alphabetical priority and one Rs 10 lakh
account (reports `backtest-03/crosscheck-flat5bps-*.md`):

| Max positions, kill switch | Trades, engine / reviewer | Total return, engine / reviewer | CAGR, engine / reviewer | Max DD | Sharpe |
| --- | --- | --- | --- | --- | --- |
| 5, off | 288 / 288 | 14.13% / 14.1% | 1.33% / 1.36% | 21.4 / 21.4% | 0.21 / 0.21 |
| 5, on | 17 / 17 | -3.13% / -3.1% | -0.32% / -0.32% | 6.3 / 6.3% | -0.18 / -0.18 |
| 10, off | 525 / 525 | 2.06% / 2.1% | 0.20% / 0.21% | 44.8 / 44.8% | 0.08 / 0.08 |
| 10, on | 14 / 14 | -1.40% / -1.4% | -0.14% / -0.14% | 7.0 / 7.0% | -0.06 / -0.06 |

No difference reaches 1 pp of CAGR; the largest is 0.03 pp. That gap is only the annualisation convention.
The reviewer divides by trading days / 252 (2,465 / 252 = 9.78 years); this engine uses calendar time
(10.0 years). Trade counts, total return, drawdown and Sharpe agree. The per-symbol diagnostic also
reproduces backtest-02 and the review: 940 trades, naive t 2.41, entry-month clustered t **1.35** (G = 116).

The adopted model costs Donchian (5, off) 0.14 pp of CAGR against flat 5 bps (1.33% to 1.19%). The 2x
stop-exit charge drives most of it.

## Gates (docs/validation-protocol.md), in-sample, portfolio runs

Configuration assessed: 5 and 10 positions, kill switch **off**, 1x adopted slippage. With the kill switch
on, every run stops within months; every edge gate then fails trivially (3a fails on 14-75 trades). Gate 4c
uses the review's amendment: the month-clustered t of daily portfolio returns, not the naive per-trade t.

| # | Gate | Threshold | Donchian 5 | Donchian 10 | TrendPullback 5 | TrendPullback 10 |
| --- | --- | --- | --- | --- | --- | --- |
| 0.1 | PIT, survivorship-safe universe | required | FAIL, much improved (95.43% of member-days; HDFC Ltd missing) | same | same | same |
| 0.2a / 0.3 | Next-open fills; costs + adopted slippage | required | PASS | PASS | PASS | PASS |
| 0.4 | Trials ledger | required | PASS (69 rows appended) | PASS | PASS | PASS |
| 0.5 | Holdout untouched | required | PASS | PASS | PASS | PASS |
| 3a | Closed trades | >= 200 | PASS (288) | PASS (530) | PASS (1,541) | PASS (2,359) |
| 3c | Distinct symbols | >= 10 | PASS (58) | PASS (69) | PASS (71) | PASS (72) |
| 3d | No symbol > 15% of net P&L | <= 15% | **FAIL** (HCLTECH 134%) | n/a (net < 0) | n/a (net < 0) | n/a (net < 0) |
| 3e | Net P&L > 0 without best 5 trades | > 0 | **FAIL** (-Rs 2.05 lakh) | **FAIL** | **FAIL** | **FAIL** |
| 4a | Mean net trade | >= 0.5% | **FAIL** (0.41%) | **FAIL** (-0.01%) | **FAIL** (-0.25%) | **FAIL** (-0.20%) |
| 4b | Expectancy | >= +0.20R | **FAIL** (+0.05R) | **FAIL** | **FAIL** | **FAIL** |
| 4c | t, month-clustered daily returns | >= 3.0 | **FAIL** (0.58) | **FAIL** (0.07) | **FAIL** (-1.98) | **FAIL** (-2.05) |
| 4d | Profit factor | >= 1.3 | **FAIL** (1.04) | **FAIL** (0.94) | **FAIL** (0.83) | **FAIL** (0.84) |
| 4e | Sharpe, daily portfolio | >= 0.8 | **FAIL** (0.20) | **FAIL** (0.02) | **FAIL** (-0.69) | **FAIL** (-0.67) |
| 4f | Sharpe and Calmar above NIFTY 50 | both higher | **FAIL** vs price index (0.20 vs 0.88; 0.05 vs 0.37), so also vs TRI | **FAIL** | **FAIL** | **FAIL** |
| 5b | Cost stress (2x fees and slippage) | expectancy > 0 | **FAIL** (see stress) | **FAIL** | **FAIL** | **FAIL** |
| 2, 5, 6, 7 | Walk-forward, robustness, DSR/PBO, Monte Carlo | | NOT-YET-MEASURED | | | |

Since 4a-4f fail in-sample on the development window, walk-forward, DSR and Monte Carlo would not change
the verdict, and I did not run them. Per-symbol runs (diagnostic) assess none of these gates.

## Where I disagree with, or add to, the review

1. **Drawdown risk is wider than the review's range.** The review gave max DD 20.7-24.7% for Donchian at 5
   positions over 8 random seeds (flat 5 bps). Over 20 seeds with the adopted model it is 19.6-31.8%
   (median 26.5%). CAGR spans -0.67% to 2.05% (median 0.82%), consistent with the review's -0.3% to 2.7%
   (median 1.0%).
2. **Donchian at 10 positions is now negative** (CAGR -0.41% at 1x, -1.23% at 2x). The review's 10-position
   runs (median 2.3% CAGR over random seeds) had flat 5 bps and no 2x stop charge. With twice the positions
   it has twice the stop exits.
3. **Minor convention difference**: CAGR over calendar years vs trading days / 252 (0.03 pp here). I kept
   calendar years, and the benchmark uses the same convention, so the comparison is like for like.
4. **Agreed and now measured**: the review's "about 1-2.5% CAGR, Sharpe about 0.2, no demonstrated edge"; the
   kill switch latching after about 17 trades; and that a 2012-2021 NIFTY 50 buy and hold returned "low double
   digits" (measured: 14.15% on the price index; the TRI is higher still).

## Caveats

- **Benchmark is the price index (^NSEI).** The TRI (dividends reinvested) was about 1.2-1.5% a year higher
  over the decade (approximate; not fetched). Both strategies fail even against the lower bar.
- The universe still misses 4.57% of member-days (HDFC Ltd for the whole decade, Cairn, IDFC, Jaiprakash,
  Ranbaxy, Sterlite), and 41 of 45 membership change dates come from a secondary source.
- Positions in a series whose data ends early (e.g. merged companies) are marked at their last close until
  the end. Results are on dividend-adjusted Yahoo prices; volume is split-adjusted only.
- The kill switch (5% drawdown) and 0.5% risk per trade are still placeholders. At portfolio level they are
  incompatible: a decision for the user, not something to tune here (review, recommendation 6).
- Power: 120 monthly clusters cannot resolve a small true edge. A real but small edge would also have
  looked like this. That is not evidence for one.

## Recommendation

Do not paper trade either strategy, and do not tune them on this window (every variant adds to N in the
ledger). The engine is now ready to evaluate a genuinely different idea at portfolio level against the index.
The next infrastructure steps are the NIFTY 50 TRI series, the missing member price histories, and
walk-forward with the portfolio engine.

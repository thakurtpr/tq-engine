# Independent review of backtest-03b and backtest-04

Reviewer: research-review session, 2026-10-30. Scope: `engine/reports/backtest-03b.md`,
`reports/backtest-04.md` (+ `backtest-04/`), the new `rebalance/` engine, `momentum_rotation.rs`,
`index_trend_filter.rs`, `buy_and_hold.rs`, the NIFTYBEES data files, against protocol v2. I did not edit
anything under `engine/`. Scripts: `docs/review-backtest-04-scripts/`.

## Verdict

**Every number reproduces exactly, there is no look-ahead in the rankings or the membership filter, and the
FAIL verdicts stand. But one start-up artefact biases the headline momentum comparison against momentum, and
protocol v2's gate 4c needs a different amendment from the one proposed.**

1. **Reproduction: exact.** My independent replica of the new rebalancing engine matches MomentumRotation to
   the paisa (final equity Rs 37,13,501.61, CAGR 14.02%, max DD 36.01%, Sharpe 0.81, 476 sells), the trend
   filter (7.14% / 21.2% / 0.71, 19 capped entries), buy-and-hold (14.31% / 36.3% / 0.96), momentum's 20 seeds
   (14.02-14.08%), all three 10%-kill-switch runs (latch dates and results), and every 03b figure I checked
   (section 1).
2. **No look-ahead (your 1 and 2).** The 12-1 score uses closes up to the month-end close only, with the skip
   applied as `close[t-21] / close[t-252] - 1`. Membership at the rank date costs or gains nothing material
   (section 2).
3. **Start-up artefact (new, important).** The momentum book is **entirely in cash for January 2012**: the data
   begins 2011-01-03, so only 246 closes exist at the first month-end and the rule needs 253. NIFTYBEES gained
   **+11.9% that month**. Like-for-like, momentum **beats** the index by 0.45 points a year (14.15% vs 13.70%),
   and gate 4f still fails, but on Sharpe alone (section 3).
4. **NIFTYBEES repair (your 3a): fine.** Cash yield (your 3b): the engineer's estimate is right; the trend filter
   still fails 4f on Sharpe even at 8% (section 4).
5. **Gate 4c (your 4): the diagnosis is right, the proposed remedy has almost no power, and v2's HAC bandwidth
   rule breaks for rebalancers.** I recommend a v3 amendment with different content (section 5).

## 1. Reproduction

Replica: `rebal.py` (new, no shared code with Rust; written from `rebalance/{mod,orders}.rs` and
`risk/src/rebalance.rs`): month-end detection, sells then buys in ticker order, 0.5% cash buffer, 0.5% trim
threshold, 1% ADV cap, adopted slippage without tick rounding, gate checks (kill switch, 2% daily loss, position
count, notional cap), delivery costs per fill. Swing checks use `portfolio2.py` with `USE_TICK = False`.

| Check | Report | Replica |
| --- | --- | --- |
| MomentumRotation (kill off, 1x): final equity / CAGR / max DD / Sharpe / sells | Rs 37,13,501.61 / 14.02% / 36.01% / 0.81 / 476 | **identical** |
| Momentum 20 random-priority seeds, CAGR | 14.02-14.08% | 14.02 / 14.05 / 14.08 (min / median / max) |
| IndexTrendFilter | 7.14% / 21.2% / 0.71; 19 entries cut | 7.140 / 21.17 / 0.708; **19** cut |
| BuyAndHold NIFTYBEES (cost-paying) | 14.31% / 36.3% / 0.96; first buy cut | 14.307 / 36.27 / 0.957; **1** cut |
| 10% kill switch: momentum / trend / B&H | 2013-08-07, 3.29%; 2012-05-14, -0.94%; 2012-05-04, 14.31% | **identical** latch dates, CAGR and MDD |
| 03b, alphabetical, no tick: D5 total 13.43% / 2x 9.45% / 3x 5.53%; D10 0.86% / -5.47% | as listed | **identical** |
| 03b, kill 10%: D5 latch 2015-10-05, 111 trades, 0.64%; D10 2015-04-21, 171, 1.68% | as listed | **identical** |
| 03b seeds, D5 and D10: median CAGR and p5-p95 (0.90, -0.37..2.07; 1.97, 0.33..3.56), positive 17/20 and 19/20, MDD medians 26.2 / 39.1 | as listed | **identical** |

Not replicated: the 80 TrendPullback seed runs, capital scaling at Rs 1 and 10 crore, the `--adv-cap-pct 100`
variants, and the Yahoo data itself against NSE. The replica shares the spec with the engine, so a misreading of
a rule common to both would not show.

## 2. Look-ahead and the membership filter (your 1 and 2)

- **Month-end detection** uses the *next calendar day's month* from the union trading calendar. That reads the
  calendar (published in advance), not prices, so it is not a look-ahead.
- **Order of events** per day: fill pending targets at the open, mark to the close, append the close to the
  history, *then* decide. `history` therefore ends at the rank date's close; targets fill at the next open.
- **Score:** `closes[last-21] / closes[last-252] - 1` with `last` = the rank date. This is 12-1 as pre-registered.
  A symbol needs 253 closes. Ties go to the earlier ticker. Unit tests cover skip, ties and short history.
- **Membership at the rank date, not the fill date.** `decide` keeps symbols that are members on the rank date
  (`from <= date < to`), so an addition effective the next day is ignored until the following month-end and a
  removal effective the next day can still be bought. Neither uses future information. I re-ran with
  membership taken on the fill date (also known in advance): CAGR **14.11%** vs 14.02%, max DD unchanged. The
  two sets differ at only 5 of 120 month-ends (one of them the empty first month-end, section 3). Immaterial.
- **Daily-loss gate evaluated at the fill-day open:** 7 rebalance buys were blocked when the open gapped down
  2% or more. That conditions on the fill-day price, but it works *against* the strategy: with the gate off,
  CAGR is **14.12%** (+0.10). Not a flattering bias, but note the principle for later runs.
- **Adjusted prices (small, not a look-ahead of prices):** dividend adjustment rescales history with later
  dividends, so the 12-1 ratio includes dividends in the window, and Faber's SMA compares today's close with a
  slightly lowered history. Effects are a few basis points.

## 3. The January 2012 start-up artefact (new)

`data/daily-pit` starts 2011-01-03. At the first month-end (2011-12-30) the longest series has **246 closes**;
the rule needs 253. No symbol is eligible, the first targets are empty, and the book is **fully in cash for 21
trading days**. The first fill is 2012-02-01. NIFTYBEES (adjusted) gained **+11.87% from 2011-12-30 to
2012-01-31**, so the comparison from 2012-01-02 hands the index a free month.

| From the 2012-01-31 close (first fill 2012-02-01) | CAGR | Max DD | Sharpe | Calmar |
| --- | --- | --- | --- | --- |
| MomentumRotation | **14.15%** | 36.0% | 0.817 | **0.393** |
| NIFTYBEES, frictionless | 13.70% | 36.3% | 0.918 | 0.377 |
| Cost-paying buy-and-hold (CAGR only) | 13.65% | | | |

- Momentum is **+0.45 points a year ahead** of the frictionless index; the report's window shows it 0.84 points
  behind (14.02% vs 14.86%; 14.31% for the cost-paying buy-and-hold). In 2012 February-December momentum made
  +24.1% against +13.9% for the index; the report's full-year 24.1% vs 27.4% is the missing January.
- Gate **4f**: Sharpe 0.817 vs 0.918 still **fails**; Calmar 0.393 vs 0.377 now **passes** (the report has it
  failing, 0.39 vs 0.41). Both must pass, so 4f still fails, but "same drawdown, lower return" is not the right
  description.
- Excess return over NIFTYBEES (HAC): full window **-0.31%/yr, t -0.09** (the report's -0.17 at L = 616); from
  the first invested day **+0.86%/yr, t +0.24**. Either way indistinguishable from zero.
- This is a *comparison* bias, not an engine error: the engine did what it was told. **Fix:** fetch price
  history from at least 2010-11 so the book can rank on 2011-12-30, or define the evaluation window from the
  first invested day for both books, and state it. Add a protocol rule: warm-up data must cover the longest
  lookback, or strategy and benchmark are compared from the strategy's first fill.
- The same family of start-up effect affects the *cost-paying* buy-and-hold (its first purchase is cut to 1% of
  ADV), which is why the report correctly benchmarks against the frictionless series.

## 4. NIFTYBEES data and cash fairness (your 3)

**Dropping the two rows (2019-12-19, 12-20): correct.** In `NIFTYBEES-raw.csv` they print at 12.92-13.49
(0.1007 and 0.1008 of the previous close) with volume about 8x normal (2.7M and 2.5M vs a median of 338k),
the signature of a split adjustment applied twice. I compared two repairs: dropping the rows (engineer) and
multiplying prices by 10 and dividing volume by 10 (closes 130.2 and 130.3, consistent with the index). Results:
B&H 14.307% / 36.27% / 0.957 and trend filter 7.140% / 21.17% / 0.708 are **identical to three decimals** under
both. Dropping also keeps a high-volume print out of the 20-day ADV. Other checks: the 161 zero-volume flat bars
(Oct 2012-Jun 2013) track ^NSEI with a correlation of 0.984 and a price ratio between 1.001 and 1.016, so the
closes are usable. NIFTYBEES' daily returns differ from ^NSEI's by a median of 10 bps (up to 445 bps on 2020-03-19);
this is same-day noise (near-zero correlation with the lagged index), not a systematic shift. The cumulative gap
(14.86% vs 14.15%) is dividends net of fund costs.

**Cash at 0% in the trend filter: yes, it handicaps the comparison; the engineer's estimate is right but it
does not change the verdict.** The filter holds cash about **31%** of days. Rerun in the replica with cash earning
a yield (costs and slippage included):

| Trend filter | CAGR | Max DD | Sharpe (rf = 0) | Calmar |
| --- | --- | --- | --- | --- |
| cash 0% (report) | 7.14% | 21.2% | 0.71 | 0.34 |
| cash 4% | 8.33% | 19.8% | 0.82 | 0.42 |
| **cash 6%** | **8.92%** | 19.2% | **0.87** | **0.46** |
| cash 8% | 9.52% | 18.5% | 0.92 | 0.51 |
| *Buy-and-hold (cost-paying) / frictionless* | 14.31% / 14.86% | 36.3% | 0.96 / 0.98 | 0.39 / 0.41 |

- At 6% the gain is +1.78 points, matching the engineer's 1.8. Calmar then **passes** (0.46 vs 0.41) but Sharpe
  still **fails** at every yield up to 8% (0.92 < 0.96). 4f stays FAIL, narrowly, not by 0.27 as the headline suggests.
- Sharpe with rf = 0 flatters cash: crediting 6% to cash lifts the filter's Sharpe from 0.71 to 0.87. With 6%
  subtracted from every book, the Sharpes are 0.57 (B&H), 0.49 (momentum) and **0.31** (filter). On excess return
  over cash the filter is clearly worse.
- A risk-matched constant mix (69% NIFTYBEES + 31% cash at 6%) gives 12.25% / 25.1% DD / Sharpe 1.17: it beats the
  filter by 3.3 points of CAGR for 5.9 points more drawdown. Timing did not beat simple de-risking here.
- The 6% figure is the engineer's and the user's; Indian liquid-fund yields were higher earlier and lower in
  2020-21, so treat 4-8% as the range.

## 5. Gate 4c: the proposed amendment (your 4)

**Agreed:** an absolute-return t is not a test of skill for a fully invested long-only book. Buy-and-hold scores
2.85-2.86 on it.

**Problems with testing the excess return over the benchmark:**
1. **No power.** Momentum's tracking error is 11.1% a year; over 9.8 years the standard error of the mean excess
   is 3.6% a year. **t >= 3 needs +10.7% a year of alpha; t >= 2 needs +7.1%.** The published momentum premium is
   far smaller, so a genuine premium could not pass. A failure here means "not detectable", not "absent". For a
   low-tracking-error fund (buy-and-hold, TE 0.6%) the same test just measures costs.
2. **Beta.** The trend filter's beta to NIFTYBEES is **0.49**; most of its excess return is simply 49% market
   exposure in a bull market (-6.1%/yr at 6% cash, t -1.6 HAC). The regression alpha (rf = 0) is +1.8%/yr, t 0.6,
   but a proper CAPM alpha at rf = 6% subtracts `rf (1 - beta)`, about **-1.3%/yr**. Beta-adjusted alpha is the
   right quantity, with rf stated.
3. **Start date and cash** (sections 3-4) move the excess return by 1.2 and 1.8 points, more than any effect
   being tested.

**v2's HAC bandwidth rule breaks for rebalancers (new).** v2 sets `L = max(NW rule, longest holding period)`,
capped at T/4. For a book that holds names for years the floor is the cap (L = 616 of 2,465 days), which
inflates the t. Momentum's daily-return t by bandwidth: **2.67 (L = 8), 2.75 (21), 3.13 (63), 3.23 (252),
4.13 (616)**; monthly t 2.81. Buy-and-hold: 2.85 at L = 8 but **4.59** at L = 616. The report reaches the right
number (2.81) only through the lowest-of-three fallback. Portfolio returns have little autocorrelation, so the
holding-period floor is not justified.

**Recommended v3 amendment (the user decides thresholds; I can apply it on request):**
1. **Bandwidth:** `L = floor(4 (T/100)^(2/9))` (about 8 here); report the t also at 2L and 4L; drop the
   holding-period floor (or cap it at 63 days). Keep "lowest of HAC and monthly t".
2. **Split 4c by strategy type.**
   - *Absolute-return books* (stop-based swing trades, market-neutral): as v2, absolute t >= 3.
   - *Benchmark-relative long-only books* (average equity exposure >= 50% or beta >= 0.5): **4c-rel = HAC t of the
     intercept of daily net returns regressed on the pre-registered benchmark's returns, both net of the
     stated cash rate.** Threshold t >= 3.0 for strategies found by search; **t >= 2.0** for a single
     pre-registered rule taken from the published literature before any data was loaded (N <= 6). Harvey, Liu &
     Zhu's t > 3 targets newly discovered factors [cited]; a pre-registered replication of a known effect
     justifies a lower bar [convention].
   - **Always report the minimum detectable alpha** (2x and 3x the standard error) next to the result.
   - Absolute t is printed for information only for these books.
3. **4f stays** (Sharpe and Calmar both above the benchmark), with three clarifications: computed like-for-like
   from the strategy's first fill; cash credited at a stated yield; Sharpe reported with rf = 0 and with rf =
   the cash yield.
4. **New rule 0.8 (warm-up):** history must cover the longest lookback before the evaluation window starts, or
   strategy and benchmark are compared from the strategy's first fill.
5. **Power note in every benchmark-relative report:** with ten years of one index, only alphas above about 7%
   a year are detectable, so "fails" means "no detectable alpha".

## 6. Other observations

- **Kill switch (section 7 of the report): confirmed.** Buy-and-hold latches 2012-05-04 and rides a 36% drawdown;
  the switch blocks buying, it does not liquidate. Agreed that any equity-index book fails a 10% switch at full size.
- **N = 4** independent trials is right; buy-and-hold is a benchmark.
- **Report wording:** "same drawdown, lower Sharpe, 295% turnover costing 20% of capital" is accurate (costs 14.13%
  and slippage 6.04% of *starting* capital over ten years), but the CAGR comparison (14.02% vs 14.31% and 14.86%)
  depends on the January cash month. "Buy the index fund" remains reasonable advice on risk-adjusted terms.
- **Gate 3a-3e and 4a/4d** for rebalancers are meaningless as the report says (sales at average cost).
- 03b's corrections of backtest-03 are all accepted: every number I checked reproduces.

## 7. Recommendations

1. Refetch prices from 2010-11 (or start the comparison at the first invested day) and re-run momentum; expect CAGR
   about 14.2% vs 13.7% and still FAIL on Sharpe.
2. Apply the v3 amendment above if the user agrees (bandwidth, 4c-rel with minimum detectable alpha, rule 0.8).
3. Report Sharpe with rf = 0 and with the cash yield; give the trend filter a stated cash yield; include the
   constant-mix comparator.
4. Open items still: the NIFTY 50 TRI, HDFC Ltd history, a clean NIFTYBEES series from NSE.

## Reproducing

`docs/review-backtest-04-scripts/` (read-only on the engine's data and reports; run with `python3 -I` from that
folder): `rebal.py` (rebalancing engine replica: momentum, Faber, buy-and-hold, cash yield, seeds, fill-date
membership), `rebal_checks.py` (momentum seeds and 10% kill switch), `stats_alpha.py` (HAC bandwidths, alpha and
power, Sharpe at rf), `check03b.py` (03b swing numbers), plus `portfolio2.py`, `replica.py`, `pit.py`.

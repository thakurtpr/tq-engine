# Independent review of backtest-03

Reviewer: research-review session, 2026-10-30. Scope: `engine/reports/backtest-03.md`,
`reports/backtest-03/{summary.csv,seeds.csv,manifests/}`, the new engine code (`slippage.rs`, `engine/`,
`portfolio.rs`, `priority.rs`, `returns.rs`, `gates.rs`, `benchmark.rs`) against `docs/validation-protocol.md`
v2 and `docs/slippage-model.md`. I did not edit anything under `engine/`. Scripts: `docs/review-backtest-03-scripts/`.

## Verdict

**The conclusions (no demonstrated edge, both strategies fail, neither beats the index) are right, and the
engine reproduces exactly. But the report overclaims in one place that matters: it says Donchian at 10
positions "loses money", and that is an artefact of one unlucky ordering.**

1. **Exact-match claim: confirmed, and stronger than claimed.** My independent replica reproduces all 12
   Donchian and all 12 TrendPullback grid rows (trade counts and final equity to the paisa), all 20 seeds at
   5 positions, and the four `portfolio.py` cross-check configs (288 / 17 / 525 / 14 trades).
2. **Slippage implementation: matches `slippage-model.md`** (prior-bar ADV and sigma, tick rounding against
   the trader, 2x on stops). Two things to state: the impact term almost never binds at Rs 10 lakh, and the
   Rs 0.05 tick is applied to dividend-adjusted prices (section 3).
3. **Gate 4c does not follow protocol v2 to the letter.** v2 requires a Newey-West (HAC) t as the primary,
   with a monthly-return t and a month-clustered per-trade t as cross-checks, and takes the lowest. The report
   gives only the month-clustered t of daily returns. Numerically it makes no difference here (section 4).
4. **Overclaiming (section 5):** the "10 positions loses money" claim and its explanation; pass/fail judged on
   one alphabetical ordering instead of the median of 20 seeds (v2 rule 0.6); the "fragile" label; and a few
   smaller items.

## 1. Does the engine reproduce? (your question 1)

Replica: `portfolio2.py` (new, shares no code with Rust). It implements the engine's day order, shared cash,
position cap, risk sizing at the fill, the **adopted slippage model** (ADV and sigma from prior bars, impact
above 0.05% of ADV, 2x stops, tick rounding, `--slippage-mult`), the gate checks, and a bit-for-bit port of the
engine's SplitMix64 priority shuffle. It also contains a TrendPullback replica (`tp.py`: Wilder RSI(2), EMA
20/50, SMA 200, ATR 14).

| Check | Result |
| --- | --- |
| 12 Donchian grid rows (max 5 / 10 x kill on / off x slippage 1x / 2x / 3x) | **12 of 12 identical** (trades, CAGR, max DD, Sharpe, final equity to the paisa) |
| 12 TrendPullback grid rows | **12 of 12 identical** (largest final-equity difference Rs 0) |
| Donchian 5 positions, kill off, 20 random seeds (`seeds.csv`) | **20 of 20 identical** trade count and CAGR; confirms the shuffle |
| Original `portfolio.py` (flat 5 bps, alphabetical): 5 off, 5 on, 10 off, 10 on | trades **288 / 17 / 525 / 14**; returns 14.1% / -3.1% / 2.1% / -1.4%, max DD 21.4 / 6.3 / 44.8 / 7.0%: matches the report's table |
| Closed-trade net Rs 43,197; costs Rs 69,701; slippage Rs 43,416; friction 72% of pre-slippage gross | all reproduced exactly |
| HCLTECH 134% of closed net; 58 symbols; net without best 5 trades -Rs 2.05 lakh; PF 1.04; mean trade 0.41% | reproduced |
| TrendPullback (5, off): friction (costs + slippage) about 3x gross | 299%: reproduced |
| ^NSEI: CAGR 14.15%, Sharpe 0.88, max DD 38.4%; calendar-year comparison (beat only in 2013) | reproduced from `data/index/NSEI.csv` |
| Data manifests | `manifest-405ea8789715.csv` and `manifest-69e6b5be26ec.csv` hash to their filename prefixes; **all 155 listed file hashes match the files on disk** |
| `t_month_clustered` in `summary.csv` (0.580 / 0.071 / 0.446 / 0.310 on four rows I checked) | reproduced exactly |

Limit: the replica is written from the same docs and Rust source, so a rule misread shared by both would not
show. Its value is that the *code* is independent and agrees to the paisa.

The one disagreement in the report (Donchian at 10 positions) is not an engine disagreement. See section 2.

## 2. The 10-position "loss" is an ordering artefact (your disagreement)

The report says Donchian with 10 positions **loses money** (CAGR -0.41% at 1x), attributes the gap to the
review's 2.3% median to the 2x stop charge, and ran random-priority seeds **only at 5 positions**. I ran the
same 20 seeds at 10 positions (adopted slippage, 1x, kill switch off):

| Donchian, 10 positions, 1x | CAGR | Max DD | Sharpe | Trades |
| --- | --- | --- | --- | --- |
| Alphabetical (the report's number) | **-0.41%** | 45.7% | 0.02 | 530 |
| 20 random orderings: min / median / max | **+0.11% / +1.94% / +4.27%** | 32.8 / 40.1 / 46.4% | 0.07 / 0.22 / 0.42 | 485 / 507 / 529 |

**All 20 random orderings are positive; the alphabetical order is lower than every one.** Decomposing the
distance from the review's 2.3% median: the slippage model vs flat 5 bps costs about **0.6 points** for the
alphabetical order (flat +0.20% -> adopted -0.41%), and about 0.3 points at the random-order median (2.25% ->
1.94%, approximate because the review's shuffles used a different random-number generator), while the choice
of ordering moves CAGR by about **2.3 points**. So the report's explanation ("twice the positions, twice the
stop exits") is mostly wrong: the ordering explains most of the difference.

This does not rescue the strategy: median CAGR 1.9% against 14.15% for the index, max DD 40%, Sharpe 0.22,
t about 0.7. It is still a FAIL on every edge gate. But "loses money" in the verdict and the 10-position
columns of the gate table should read "CAGR 0.1% to 4.3% (median 1.9%), not distinguishable from zero".
For the report's own sentence "a single-order backtest ... cannot be read to better than about +/-1.5 points
of CAGR": at 10 positions the spread is about +/-2 points.

## 3. Slippage implementation vs `docs/slippage-model.md` (your question 2)

| Requirement (doc section 5) | Implementation | Verdict |
| --- | --- | --- |
| 5 bps per side up to 0.05% of 20-day ADV | `bps()` returns the base when `qty/adv*100 <= 0.05` | Matches |
| Above that, `1.5 + 2.0 + Y*sigma*sqrt(q/ADV)*10,000`, Y = 0.7 | `impact_bps()`: same formula, Y 0.7, rounded to 4 dp | Matches (test reproduces the doc's 14.0 bps at 1% ADV, sigma 1.5%) |
| ADV and sigma from bars **before** the fill bar | `Liquidity::push` runs in `observe()` after the day's fills; 20 volumes, 20 returns from 21 closes | Matches. My replica uses the same prior-bar rule and ties to the paisa |
| 2x on stop exits (fill at the worse of stop and a gapped open) | `bps * 2` for `StopLoss`; raw price is `min(open, stop)` | Matches |
| Orders capped at 1% of 20-day ADV, cut and recorded | `adv_cap`, `entries_adv_capped`; zero when ADV is unknown | Matches. Cap bound **0 times** in every run I replicated |
| Round to the Rs 0.05 tick **in the worse direction** | buys `ceil`, sells `floor`, never below one tick; unit tests | Matches |
| Stress 1x / 2x / 3x scales the total | `bps * slippage_mult`; so a stop exit at 2x stress pays 4x base | Matches the doc ("scale the total"); worth stating |

Implementation choices the doc did not specify: the impact figure is floored at the base (a bigger order never
pays less than 5 bps), which is conservative and reasonable.

Two points the report should state:
1. **The impact term is dormant at this capital.** Entry size is a median 0.006% of 20-day ADV (95th
   percentile 0.032%, max 0.069%); only 7 of 530 fills (1.3%) exceed the 0.05% threshold. In practice the
   adopted model here is "5 bps + 2x on stops + tick rounding". The liquidity dimension is untested until
   capital grows; do not read "ADV cap never bound" as evidence the impact formula was exercised.
2. **The Rs 0.05 tick is applied to dividend-adjusted prices**, which are not on the real tick grid and are
   often far below the traded price. Effect on the Donchian 5-position run: entries pay 6.2 bps on average
   (9.9 bps when the adjusted open is under Rs 100, 33 of 288 entries; 5.2 bps at Rs 500 and above), stop exits
   11.5 bps, signal exits 6.0 bps. Small and conservative, but it is friction that real prices would not have.

## 4. Gate 4c vs protocol v2 (your question 3)

v2 (section 4): the primary is the **t of portfolio daily net returns with Newey-West (HAC) standard errors**
(bandwidth `max(floor(4*(T/100)^(2/9)), longest holding period)`); cross-checks are the monthly-return t and the
month-clustered per-trade t; if either is more than 0.5 below the HAC t, use the lowest.

The report computes only the **month-clustered t of daily returns** and says it "uses the review's
amendment". That was my recommendation ("cluster-robust or portfolio-daily-return t"), but v2 is stricter.
Recomputed on the same return series (kill off, 1x, alphabetical):

| | Month-clustered daily (report) | HAC, L = 8 | HAC, L = longest hold | Monthly-return t | Per-trade month-clustered |
| --- | --- | --- | --- | --- | --- |
| Donchian 5 | 0.58 | 0.59 | 0.60 (L = 198) | 0.59 | 0.49 |
| Donchian 10 | 0.07 | 0.07 | 0.06 (L = 218) | 0.08 | -0.01 |

Over 20 random seeds the HAC and monthly t agree with the month-clustered t within about 0.03 at 5 positions
(-0.16 to 0.91 versus -0.16 to 0.93) and at 10 positions (0.21 to 1.23 versus 0.22 to 1.26). **No v2 gate verdict
changes**, but the report should print the HAC and monthly t and apply "lowest of three". Also, the
longest-holding-period bandwidth floor in v2 (about 200 days here) is heavy; it gave the same answer, but I
would amend v2 to a high percentile of holding periods if it ever disagrees.

Other v2 compliance points:
- **Rule 0.6 (judge on the median of >= 20 random-priority seeds):** not followed. The gate table uses the
  alphabetical single order for all four configurations; seeds exist only for Donchian at 5 positions.
  For D5 the seed median (CAGR 0.82%, t 0.44) is below the alphabetical value (1.19%, 0.58); for D10 it is
  above (section 2). The verdict is unchanged either way, but the table should show medians and the 5th percentile.
- **5b cost stress:** v2 asks for 1x/2x/3x slippage and 2x fees at 2x. The 2x-fees case was not run: the report
  subtracts an estimated "Rs 67,917 of extra costs" from the 2x closed-trade result. The sign is robust, but
  label it an estimate or add a fee multiplier and run it.
- Section 7 kill-switch note: followed (on and off both reported, latch dates given). Rule 0.7 (return on
  capital, friction counted, manifest, ledger Sharpe from portfolio returns): followed; I verified the manifest
  hashes and the ledger's per-period Sharpe is portfolio-based.

## 5. Overclaiming and smaller items (your question 4)

1. **"With 10 positions it loses money" (verdict) and the 10-position gate column.** Ordering artefact
   (section 2): 20 of 20 random orders are positive. Re-run the seeds for D10 and state the median.
2. **Explanation of the disagreement with the review** ("twice the stop exits"): ordering explains most of it.
3. **Pass/fail on a single ordering** for D10 and both TrendPullback configurations (v2 rule 0.6).
4. **"Fragile by the slippage doc's definition":** the doc says net return falls by more than half between 1x
   and 2x. The report uses closed-trade P&L (Rs 43,197 to Rs 9,142, a 79% fall). Total return / CAGR falls from
   1.19% to 0.85% (29%), so by the doc's wording it is not fragile at 2x (CAGR does fall 58% by 3x). The
   closed-trade figure also excludes about Rs 83,000 of unrealised gain. Say which metric and why.
5. **"Rejected 6,102 of 6,707 entry signals"** counts signal-days: a stock above its channel re-signals every day
   it stays flat and blocked, so distinct blocked opportunities are fewer. The capacity-bound conclusion holds
   (mean 4.15 of 5 slots in use), but "signals" overstates.
6. **TRI:** the benchmark is the price index and the report says so clearly, and the inference (failing vs the
   price index implies failing vs the TRI) is sound. Gate 4f cannot be formally passed until the TRI exists.
7. **Ledger N:** 69 rows appended (stress variants, seeds, cross-checks). The trials ledger's N for the deflated
   Sharpe is the number of *independent* ideas (here 2 strategies at fixed defaults, plus the 5 / 10 position
   limit if you count it), not rows. Define N in the report so DSR is not computed with 73.
8. Minor: a pending entry whose symbol has no bar on the next day waits for that symbol's next bar (stale
   signal, still counted toward the position cap). With Yahoo data this is rare, but it is worth a guard.

What the report did well: it measured the benchmark, ran the kill-switch on/off both ways, reproduced the
per-symbol diagnostic, labelled the TRI gap, gave honest power caveats, archived data with verified hashes,
and its headline numbers are all correct.

## 6. Recommendations

1. Re-run D10 and TrendPullback with >= 20 random seeds; report median and 5th-95th percentile; fix the
   verdict wording for D10.
2. Print HAC, monthly-return and per-trade clustered t next to the current one; apply "lowest of three".
3. State that the impact term and ADV cap did not bind at Rs 10 lakh; consider a capital-scaling run (for
   example Rs 1 crore and Rs 10 crore) to exercise the liquidity model before relying on it.
4. Apply the tick to unadjusted prices or drop it on adjusted data.
5. Define N for the DSR; run 5b with a fee multiplier.
6. Still open from before: TRI series, HDFC Ltd (keep the gap, report coverage), 41 of 45 membership dates
   from a secondary source.

## Reproducing

`docs/review-backtest-03-scripts/` (read-only on the engine's data and reports; run with `python3 -I` from
that folder): `portfolio2.py` (replica with adopted slippage and SplitMix64), `run_grid.py` (Donchian grid and
seeds, writes `grid_results.json`), `tp.py` + `run_tp.py` (TrendPullback), `t_v2.py` (clustered, HAC and monthly
t), `checks2.py` (original `portfolio.py` cross-check), `checks3.py` (closed P&L, benchmark, yearly returns,
slippage by price level), plus `replica.py`, `pit.py`, `portfolio.py`.

# Master pre-registration research-v2 (revision 2, after adversarial review and owner decision D-003)

Which strategy on which instrument is profitable after costs and tax, on the maximum free history, with seen data treated as development data and forward paper as the final judge?

Registered: 2026-11-16 (decisions D-002 and D-003), **before any strategy has been run on research-v2 data.** Paper and backtest only; no broker, orders or credentials; no engine code is pushed.
Revision 2 applies `docs/review-prereg-research-v2.md` (verdict FIX THEN TAG; 3 CRITICAL, 9 HIGH, 11 MEDIUM, 6 LOW) and D-003. **No threshold of the frozen v3 protocol and no threshold of revision 1 was lowered**; where a fix needed a number, the stricter of the candidates was kept (marked "stricter").
**Edit rule (L3).** Before the tag, any edit is allowed provided no strategy result exists. From the tag onward an edit may only tighten, needs a dated written reason, and makes the changed test a new trial. **The tag is created by the coordinator, not by the author of this file; this file is NOT tagged yet.** The tag annotation records the tag date, which is the start of forward paper (section 5.4).
Ledger family: **`research-v2`** (section 4). Revision 1 had a draft marker pinned to the text of revision 1; it is closed, and a new marker pinned to this final text is registered after the last edit (section 4.6).

## 0. Rules of the game

1. The frozen `docs/validation-protocol.md` **v3 gates are UNAMENDED.** Section 2.1 restates their numbers so that no agent guesses; if a number here ever differs from v3 the STRICTER governs and the difference is reported.
2. The owner's extra gates (2.2) and the integrity gates (2.4) are ADDED, never substitutes.
3. **Failing any gate = KILL** (INCONCLUSIVE-SAMPLE where V2 says so; INCONCLUSIVE is never a pass). **Gates are never relaxed after any result.**
4. Vocabulary (D-003): a candidate that passes every development gate (and, for classes with a clean holdout, the single-touch class holdout) is **PROMOTED TO FORWARD PAPER**. It is **never called "PASSED"** until the paper gate (V11) is met. Otherwise the report says **"nothing passed; the default stays Nifty ETF + BTC"**.
5. No candidate runs before: (a) the harness of section 10 is committed AND independently reviewed; (b) its `docs/prereg-r2-<id>.md` is tagged; (c) its sweep is opened in the ledger by the single ledger writer (4.5). No parameter, grid value, window, instrument, cost, gate or seed may be chosen after looking at any return, P&L, Sharpe, drawdown or equity curve. Rules come from cited literature or from the catalogue in section 6.
6. Contact disclosure is in the per-class seen-window table (3.1). The owner and agents also know the general history of Nifty and BTC through 2026-11; this cannot be erased and is why only forward paper counts for a claim.

## 1. Question, hypotheses and null

**H1.** At least one candidate in section 6, with rules and grid fixed in advance, earns net of costs, slippage, funding and tax, on development data, on the clean class holdout where one exists and on forward paper, a positive return that passes every gate in section 2 including beating buy-and-hold Nifty ETF, BTC and the fixed 50/50 blend.
**Null.** After costs and tax every candidate is no better than buy-and-hold; any development winner is selection among N trials. Prior: every earlier candidate was KILL or INCONCLUSIVE. KILL or INCONCLUSIVE for all candidates is the expected outcome and a valid, useful result.

## 2. Gates (all numeric, all frozen)

### 2.1 Frozen v3 gates (restated, UNAMENDED unless marked "stricter")

| ID | v3 ref | Requirement |
| --- | --- | --- |
| V1 | s.0 rule 6, s.3 | one shared-capital portfolio account; headline metrics from it only; random-priority sensitivity >= 20 seeds, median judged, 5th percentile net CAGR > 0 |
| V2 | s.3 | **Trade counts, with pooling (D-003; v3 numbers unchanged).** Counts are **effective independent closed trades `N_eff`** (2.5): **>= 200 in the development window, >= 100 in the stitched walk-forward OOS, >= 30 in the class holdout.** Diversification clauses apply only to books with >= 10 eligible symbols at registration: >= 10 distinct symbols with a closed trade and no symbol > 15% of net P&L (2.5). Dropping the 5 best trades leaves net P&L > 0 (all books). A book with < 10 eligible symbols, a single instrument, or `N_eff` below a threshold is **INCONCLUSIVE-SAMPLE with the shortfall stated** (not KILL, not promoted). The revision-1 clause "no two counted trades share an entry day" was not in v3 and is deleted |
| V3 | 4c-abs / 4c-rel | HAC t (Newey-West, bandwidth `L = floor(4 (T/100)^(2/9))`; the engine's `tstats.rs` still has the v2 holding-period floor and MUST be fixed in the harness, section 10) of daily net portfolio returns >= **3.0** on the development window AND the stitched walk-forward OOS; class holdout >= **1.65**. Long-only benchmark-relative books (invested share >= 50% or beta >= 0.5) use the 4c-rel alpha t at 3.0 (every candidate is "searched"). 4c-x: the lowest of the HAC, monthly and clustered t governs |
| V4 | 4e | annualised Sharpe (daily, rf = 0, net) >= **0.8** OOS, >= **0.5** holdout |
| V5 | 4f | like-for-like vs the class benchmark (first invested day): Sharpe at rf = cash rate (6%, sensitivity 4-8%) AND Calmar both exceed buy-and-hold |
| V6 | 4g | holdout net return > 0 and holdout Sharpe >= 50% of walk-forward OOS Sharpe |
| V7 | s.5 | neighbourhood: >= 70% of neighbours net-profitable and median neighbour Sharpe >= 70% of the base; 2x slippage and 2x brokerage keep net profit and a positive t; >= 60% of walk-forward folds net-positive (rounded up) and worst-fold max drawdown <= the kill-switch limit (30%, ASSUMPTION below); random-entry baseline (1,000 draws) net return above its 95th percentile |
| V8 | s.6 | **DSR >= 0.95** at N_true (4.2) on daily AND monthly stitched D-OOS returns (the more conservative governs), variance rule 4.3. **PBO <= 0.20** by CSCV (16 blocks) wherever the candidate's cell set (family within a class) has >= 20 cells (v3's scope); below 20 it is reported only. Trimming rule: drop the OLDEST observations so T is a multiple of 16 |
| V9 | s.7 | month-block Monte Carlo (10,000 runs) on stitched OOS portfolio returns: 95th-percentile MDD <= **24%** (= 0.8 x 30%); >= 95% of runs profitable; P(kill switch within 2 years) <= 5%. Kill switch OFF in the assessment run, ON in a reported operating run. **Sizing is fixed in the prereg; a drawdown-only failure is KILL (or a new counted cell), never a silent resize (M1)** |
| V10 | s.0 rules 2, 5 | shift test (an extra one-bar delay must not improve net results beyond noise); single-touch class holdout; warm-up rule 0.8; ledger complete |
| V11 | s.8, STATUS rule 4 | **Paper gate (the promotion-to-pass criteria, 5.4):** >= **60 sessions AND >= 100 closed paper trades (counted as N_eff of the paper window)**, results inside the Monte Carlo 5-95% band, net expectancy positive, counted ONLY from the later of the tag date and the end of the class holdout batch. (v3 s.8 text says 20 trades, STATUS rule 4 says 100; the stricter 100 is used) |

**ASSUMPTIONS the owner may override (kill switch and sizing; none is owner-confirmed).** Kill-switch drawdown limit **30%** (v3 placeholder was 5% and `UNSET-BY-USER`; 5% makes every active book fail V9 by construction, so 30% is an assumption, labelled as such). Risk per trade **0.5% of equity**, notional cap **20%** per position (the engine's `RiskLimits` defaults), max open positions **= the book's pre-registered maximum, <= 20** (declared per candidate). An owner override may only make these STRICTER before the first scored run. Where the 30%-derived numbers and v3 numbers conflict, the stricter is used: V7 worst-fold <= 30% (v3 says <= limit), V9 p95 <= 24% (0.8 x limit), and the single MDD threshold of OG3 is **24%** (stricter than revision 1's 25%).

### 2.2 Owner gates (added)

All owner-gate returns are **net of costs, slippage, funding and tax, in INR, with NO idle-cash interest** (7.4). Candidates and benchmarks are all on one **total-return** basis (7.1), and candidates' open positions at the window end are **taxed as if liquidated** exactly like the benchmarks (the engine's `unrealised_at_end` path, as `rerun-with-tax-2026-11.md`). USDT/USD legs convert at the `USDINR=X` daily close; **dual reporting**: INR and USD; for hedged or market-neutral books (pairs, carry) the **less favourable** of the two governs (INR depreciation is not free alpha). Cross-calendar Sharpe comparisons use **weekly** returns with a fixed cut: NSE series at the Friday 15:30 IST close, crypto series at Friday 10:00 UTC (the same instant; where only a 00:00 UTC daily bar exists the Friday daily bar is used and the <= 15.5 h mismatch is reported); last available price on or before the cut.

| ID | Gate | Value |
| --- | --- | --- |
| OG1 | Positive after cost and tax | cumulative net-of-everything return > 0 and net CAGR > 0 in D-OOS, V-OOS and the holdout |
| OG2 | Beats buy-and-hold | for EACH of B1, B2, B3 (where it exists on the overlapping dates): weekly Sharpe (rf = 0) of the candidate strictly above the benchmark's AND the paired month-block bootstrap (10,000 draws, seed 20261116) one-sided 90% lower bound of (candidate Sharpe - benchmark Sharpe) > 0, in D-OOS **and** V-OOS; in the holdout the same two conditions. Benchmarks 2.3 (all three apply to every class) |
| OG3 | Max drawdown | daily-equity max drawdown, kill switch OFF, after cost and tax, **<= 24%** in each of D-OOS, V-OOS and holdout AND not deeper than B1's over the same window. (BTC buy-and-hold fails this, so crypto passes only with real risk control; the number is one threshold for all classes so it cannot be tuned per class.) |
| OG4 | Sub-period stability | **D-OOS only** (D-OOS plus the holdout at holdout stage; V-OOS is part of D-OOS, M4): (a) **>= 60%** of full calendar years (partial years >= 6 months count) net-of-everything positive, no single year > 50% of cumulative net log return; (b) four regimes from the class benchmark's own data, lagged one day: trend = benchmark close above/below its 200-day SMA; volatility = benchmark 60-day realised volatility above/below its **expanding median computed as of t-1 starting 252 trading days after the benchmark's first date**; **>= 3 of 4 regimes** with mean daily net return > 0, except that a regime in which the book's exposure is < 10% of days must show mean >= 0 (a flat book cannot earn); regimes with < 60 days are excluded; fewer than 3 evaluable regimes = FAIL. Class benchmark: NIFTYBEES (India), BTC (crypto) |
| OG5 | Parameter robustness | grids have **>= 3 levels on every tuned numeric axis with the base at the centre** (section 6), so a base cell has >= 4 ring-1 neighbours; **ring-1** = cells differing in exactly one parameter by exactly one step; **>= 80% of ring-1 neighbours** AND **>= 60% of all cells of the book's grid** net-profitable after cost and tax in D-OOS. Categorical choices (the rule type) are separate books outside OG5. A book without >= 4 ring-1 neighbours is "untestable" = cannot reach GO |
| OG6 | Not interest in disguise | the gating series carries no idle-cash interest; invested share >= 20% of days; V5 is also evaluated cash-free for BOTH strategy and benchmark; a candidate passing V5 only with cash credited FAILS |

### 2.3 Benchmarks (precise; all in INR, same dates, same cost and tax engines, tax as if liquidated on the window's last day)
* **B1 Nifty ETF:** NIFTYBEES.NS bought on the first date, held; Yahoo adjusted close (dividends reinvested); `tq-costs` delivery schedule, slippage 5 bps per side, `TaxModel::equity_cg()`.
* **B2 BTC:** BTCUSDT spot bought on the first date, held. Before 2017-08-17 the series is BTC-USD Yahoo **chained by returns** (the 2017-08-17 return is taken from BTCUSDT only; the level is never spliced). Taker 10 bps + slippage 5 bps per side (**doubled for proxy-era dates before 2017-08-17**); `TaxModel::vda_statutory()` with `with_tds()`.
* **B3 fixed blend: 50% B1 + 50% B2** (owner-assumed, stated assumption), restored to target on the first trading day of each calendar quarter; rebalance sells realise gains under the respective tax models and pay costs. Report-only sensitivities 80/20 and 20/80.
* Frictionless and cost-paying versions both reported; gates use the cost-and-tax-paying versions.

### 2.4 Integrity gates (added)
* **IG1 Second multiple-testing check (SPA/White, code with tests required before first use).** For the candidate's OWN base cell: Hansen's SPA with the studentized loss differential against its benchmark (zero return for absolute books; B1 for India relative books; B2 for crypto relative books), stationary bootstrap (mean block 10 days daily / 4 weeks weekly, 10,000 draws, seed 20261117), **model universe = all counted cells of the same asset class whose D-OOS window contains the candidate's D-OOS window, evaluated on the candidate's window** (cells excluded for a shorter window are counted and reported); consistent p-value **<= 0.05**; White's reality-check p reported. Each cell's daily net D-OOS series and `returns_hash` are stored in the ledger CLOSE.
* **IG2** the harness passes prefix-invariance (`tq bias lookahead`-style), a byte-identical re-run, a content-level maximum-date check on every loaded bar, and the shift test.
* **IG3 Holdout multiplicity.** If K_H candidates reach a class holdout, the holdout HAC t >= 1.65 is additionally subject to Holm-Bonferroni across the K_H one-sided p-values at family-wise 0.05.

### 2.5 Pooled trade counts and diversification (D-003 item 2)
**Pooling is allowed within ONE pre-registered strategy family:** one book, one rule, **one parameter vector for every instrument (no per-instrument tuning)**, one shared-capital account. A **closed trade** is one flat-to-flat position round trip in one instrument (a partial reduction does not close it). Let `N_raw` be the closed trades in the window.
```
DE    = 1 + (k_bar - 1) * rho_bar                  (design effect, DE >= 1)
N_eff = N_raw / DE
k_bar   = time-average number of simultaneously open positions over days with >= 1 open position in the window (>= 1)
rho_bar = mean pairwise Pearson correlation of DAILY close-to-close returns of the book's pre-registered universe instruments
          over the same window (pairs with >= 60 overlapping days; negative mean floored at 0; capped at 1)
```
V2 thresholds are applied to `N_eff`; `N_raw`, `k_bar`, `rho_bar` and the shortfall `threshold - N_eff` are always reported. The correlation is from underlying prices, not from strategy results, so it cannot be tuned. Example: 10 concurrent positions with `rho_bar` 0.4 gives DE 4.6, so 300 raw trades are 65 effective: such a book is INCONCLUSIVE-SAMPLE on a 10-year window, as it should be.
**Diversification clause:** "eligible symbols" = instruments in the pre-registered universe with >= 252 days of history at the date of first possible entry. A book with >= 10 eligible symbols must show >= 10 distinct symbols with a closed trade and no symbol > 15% of net P&L (measured as the share of gross positive P&L). A book with < 10 eligible symbols cannot satisfy it and is **INCONCLUSIVE-SAMPLE (symbol shortfall stated)**.

## 3. Data splits (set from data availability, not from returns)

### 3.1 Per-class seen-window table (D-003 item 1) and windows

Last complete data month: **2026-10-31**. **Seen data is development data only.** Boundaries are fixed here; the inventory may only SHORTEN a start date.

| Class | Window | Read by (which prior work) | Status |
| --- | --- | --- | --- |
| **IN-EQ** PIT NIFTY 50 daily (`data-ref/nifty50_members_by_date.csv`, truncated at the dev end for every input; membership past 2021-12-31 is read only in the holdout stage) | 2012-01-01 to 2021-12-31 | backtest-01..04, rerun-after-hardening, rerun-with-tax, council-v1 dev (2015-2019), SUE-0a diagnostic, options spot closes | **SEEN = development.** 2020-01-01 to 2021-12-31 was council-v1's reserved, never-run equity holdout; research-v2 consumes it as development, which voids council-v1's holdout |
| | 2022-01-01 to 2026-10-31 | none (data on disk end 2021-12-31) | **CLEAN: class holdout H** |
| | 2026-11-01 onward | paper benchmark, heartbeat (NIFTYBEES +5.30% to 2026-11-16) | SEEN, non-gating, and outside H |
| **IN-OPT** Nifty/BankNifty EOD options | 2012-01-01 to 2021-12-31 | options-v1 dev (structures V1-V6, KILL, PBO 0.849) | **SEEN = development** |
| | 2022-01-01 to 2026-10-31 | 2025-10-27..31 sessions downloaded for feasibility (declared burned for any holdout trade, as options-v1; cycles touching them still count); 2019-03-01..07 sample | **CLEAN except those 5 sessions: class holdout H** (contract regime changed in 2024: SEBI weekly-expiry and STT/lot changes, so H is a different instrument, stated) |
| **IN-IDX** index/ETF daily (^NSEI, ^NSEBANK, NIFTYBEES, BANKBEES) | 2007-09-17 to 2011-12-31 | calendar and gap detection only (no strategy return) | development |
| | 2012-01-01 to 2021-12-31 | backtest-04 buy-and-hold and index-trend-filter, momentum rotation vs NIFTYBEES | **SEEN = development** |
| | 2022-01-01 to 2026-10-31 | none | **CLEAN: class holdout H** |
| **CRYPTO** spot/perp/funding/1h (BTC, ETH, alts) | 2014-09-17 to 2019-12-31 | none by any strategy result | development (unseen) |
| | 2020-01-01 to 2026-10-31 | carry-01 (2020-01 to 2026-10 incl. its OOS), basis-0b (2021-2026), intraday-01 and fee-breakeven-15 (1m klines 2022-2023), council-v1 crypto-burst dev (2023), edge-hunt-02 recordings | **SEEN = development; NO clean holdout exists** |
| | 2026-11-01 onward | paper benchmark, heartbeat (BTC -14.78% to 2026-11-16) | SEEN, non-gating |

**Windows (T = train, V = validation = the later part of development, H = class holdout).** D = T union V. Where no clean holdout remains (CRYPTO) the true holdout is **FORWARD PAPER from the tag date** and no historical H exists.

| Class | Train T | Validation V | Class holdout H |
| --- | --- | --- | --- |
| IN-IDX | start (expected 2007-09-17) to 2017-12-31 | 2018-01-01 to 2021-12-31 | 2022-01-01 to 2026-10-31 |
| IN-EQ | 2012-01-01 to 2017-12-31 | 2018-01-01 to 2021-12-31 | 2022-01-01 to 2026-10-31 |
| IN-OPT | 2012-01-01 to 2019-12-31 | 2020-01-01 to 2021-12-31 | 2022-01-01 to 2026-10-31 |
| CRYPTO (BTC-LONG from 2014-09-17, chained by returns to BTCUSDT 2017-08-17; alts by listing) | start to 2021-12-31 | 2022-01-01 to 2026-10-31 | **none: forward paper from the tag date** |

* **Validation is not out-of-sample** in any class except for the unseen crypto years; it is a development check labelled "validation (seen)". Only a clean class holdout or forward paper counts toward a claim.
* **Class holdout H is one-shot per class per program** (section 5.2). No tool may read an H-dated bar in development (content-level date check). H-dated bars are written at ingestion to a sealed store (10.4).
* **Carry/funding/basis lineage** and **any crypto candidate** have no clean history: they can be at most PROMOTED TO FORWARD PAPER after development gates.
* GLOBAL ETFs and foreign-ETF classes are **dropped** (tax, access and costs unverified); revision 1's GLOBAL class is not run.
* **Data-inventory dependency.** `docs/data-inventory-2026-11.md` (wt/rdata) may only (a) confirm start dates, (b) TRUNCATE a start date, (c) mark a source unusable (candidates relying on it are dropped and not counted). It may not move T/V/H boundaries, and may report only row counts and date ranges **for development windows**; no row counts, gaps or quality statistics are computed on H-dated data before the holdout stage (H6).

### 3.2 Walk-forward (positions CARRY across fold boundaries; folds only partition evaluation, L5)

| Class | Train window | Test | Step | First test | Purge | Embargo |
| --- | --- | --- | --- | --- | --- | --- |
| IN-IDX, IN-EQ, IN-OPT | 3 years rolling | 6 months | 6 months | first 1 Jan / 1 Jul on or after start + 3 years | gap >= max(max holding period, 10 trading days; 21 for monthly rebalancing; one option cycle for IN-OPT) | 21 trading days |
| CRYPTO | 12 months rolling | 3 months | 3 months | first quarter start on or after start + 12 months | max(hold, 24 h) | 7 days |

Fold counts (rounded up for V7): IN-EQ first test 2015-01-01: 14 folds (6 in T, 8 in V); IN-IDX first test 2011-01-01: 22 folds (14 + 8); IN-OPT: 14 (10 + 4); CRYPTO from 2017-08-17: first test 2018-10-01, 33 quarterly folds to 2026-09-30 (13 in T, 20 in V); BTC-LONG from 2014-09-17: first test 2015-10-01. **D-OOS** = stitched test segments over D; **V-OOS** = those inside V. The v3 development-window t is also computed on the full D from the first invested day.

### 3.3 Candidate selection
Mode F only: every candidate nominates ONE base cell, **cited to a literature reference** and, where the base cell equals a cell already run on the same data in an earlier family (for example momentum 252/21/10, Donchian 55/20), the prereg declares "same cell, same data: no new information" (M11); there is no argmax over the grid. Validation is an out-of-selection check: V-OOS must satisfy OG1-OG6, net > 0, HAC t >= 1.65 and Sharpe >= 50% of the T-part OOS Sharpe.

## 4. Trial accounting

### 4.1 Unit of a trial
A **trial = one cell** = (book) x (one full parameter vector). Neighbour cells, bug re-runs (4.7), per-book cells, the master markers, and class-holdout touch markers are all trials. Sensitivities of the SAME cell (slippage 2x/3x, brokerage 2x, tax variants, cash rate, T+2 shift, priority seeds, INR/USD) are not trials. B1-B3 and random baselines are controls, not trials.

### 4.2 TRUE total N (the DSR input; program-wide, conservative upper bound; D-003 item 3)
```
N_true      = N_new_final + L_total                       (the same for every candidate)
N_new_final = ledger.family_trials("research-v2") read after the LAST cell is opened
              (all candidates incl. killed and diagnostic, both markers, holdout-touch markers, re-runs)
L_total     = 364 (4.4)
```
Reported for information only: N_own, N_new_final alone, and N_all = N_new_final + 304 (226 + 48 + 30 as in revision 1). No existing function computes N_true: the harness calls `deflated_sharpe(obs, sr_var, N_true)` directly (`deflated_sharpe_for_sweep` deflates with `family_trials` only, no L).

### 4.3 DSR window and variance set (H2)
DSR is computed on the **stitched D-OOS** daily and monthly net returns of the candidate's base cell, never on V-OOS or H alone. `sr_var` = variance of the per-period Sharpes of **all counted cells of the candidate's asset class on D-OOS** (the SPA set), floored at `1/T` (the honesty layer's floor); the pooled family-wide maximum (`family_sr_var`) is also computed and reported, and a gap larger than 2x is flagged for the reviewer. Required net Sharpe, reviewer's arithmetic with kurtosis 8-10 and skew -0.3 (to be recomputed by the harness; **owner acknowledgement required: a net D-OOS Sharpe of about 1.8 to 2.1 after Indian costs and tax, about 2.3 to 2.6 if the pooled-maximum variance governs**):

| Window | N=100 | N=300 | N=600 | N=800 |
| --- | ---: | ---: | ---: | ---: |
| IN-IDX D-OOS 11 y | 1.27 | 1.38 | 1.45 | 1.47 |
| IN-EQ / IN-OPT D-OOS 7 y | 1.59 | 1.73 | 1.82 | 1.85 |
| CRYPTO D-OOS 5.25 y (more now: 8 y) | 1.84 | 2.00 | 2.10 | 2.13 |
| monthly series, 7 y | | | 1.92 | 1.95 |

So V3 and V4 (t >= 3 needs about 1.1-1.3) are not the binding gates; DSR is. This is accepted: KILL is the expected outcome.

### 4.4 Lineage L_total, recounted from the ledgers and git history (cells, upper bounds)

| Lineage | Evidence | Cells counted |
| --- | --- | ---: |
| donchian-breakout | trials.csv rows 114 (reviewer reports these are sensitivities of ONE parameter vector; no evidence beyond that reading, so the larger row count is kept) | 114 |
| trend-pullback | rows 62 | 62 |
| momentum-rotation | rows 26 | 26 |
| index-trend-filter | rows 5 | 5 |
| buy-and-hold | rows 5 (labelled benchmark; kept, larger) | 5 |
| sentiment-scaling-family | rows 6 | 6 |
| sue-drift-0a | rows 3 | 3 |
| council-v1 | ledger OPEN n = 48 (cells, not the 1 summary row) | 48 |
| options-v1 | ledger OPEN n = 30 | 30 |
| intraday (documented) | intraday-01: BTC and ETH x {VWAP, momentum} = 4 runs, NSE ORB on 5 stocks = 5 runs: **9 cells actually run**; edge-hunt-02 market-making: 4 trials x 2 symbols = 8 tests (spec s.8-9). Fee-breakeven scenario rows are sensitivities of the same cells | 17 |
| funding/basis carry (**documented UPPER bound**) | carry-01: 2 instruments x 2 margins (M = 0.5N, 1.0N) x 3 tax cases x an un-pre-registered margin band of 3 settings = 36; edge-hunt-01 dated-futures E variant 1; basis-0b: 2 instruments x 2 margins x 2 tax scenarios = 8; ledger rows 3 | 48 |
| **L_total** | 114+62+26+5+5+6+3 = 221; +48 +30 +17 +48 | **364** |

The 226 rows of trials.csv contain one summary row each for council-v1 and options-v1 (dropped to avoid double counting; their 48 and 30 cells are counted) and 3 carry rows (replaced by the 48). Row-based counts are kept where cells are unknown because undercounting is the dangerous direction. L_total may be raised by addendum, never lowered.

### 4.5 Caps and the single ledger writer
**Total NEW trials `N_new <= 450`.** Family caps (section 6): GO-capable families 234 cells + DIAGNOSTIC families 84 = 318; **reserve 132** (markers, class-holdout touches, re-runs). The harness refuses `open_sweep` beyond a family cap or the total (caps live in a committed `research/caps.json`). **Single ledger writer (H7):** the coordinator opens all sweeps serially in the main worktree from tagged preregs; parallel strategy agents only evaluate a given sweep id and never write the ledger file; no per-candidate ledger files.

### 4.6 Registration procedure and the markers
1. Candidate prereg from `docs/prereg-r2-TEMPLATE.md` (every field filled), committed, tagged `prereg-r2-<id>`, tagged commit an ancestor of HEAD.
2. The ledger writer opens `Ledger::open_sweep(Manifest{ family_id "research-v2", strategy_hash = sha256(prereg at tag), data_hash, cost_model_hash, split_spec_hash = sha256(the TAGGED master), axes, seed, code_git_sha = harness build, parent_strategy_hash for lineage })` through `LedgerStore::beside(research/ledger-research-v2.jsonl)`, records the sweep id and ledger tip in the prereg's registration block, and commits ledger + anchor mirror + `trials.csv` row.
3. All sweeps share `family_id = "research-v2"`; the sweep is closed by `close_open_sweep`, which stores each cell's `returns_hash`.
4. **Markers.** Revision 1's marker (sweep bee0a44e...) pins revision 1 and is **closed** (cells = 1, sr_var 0). A **new marker** pinned to the sha256 of this final text is opened after the last edit and counts as one more trial. The ledger commit comes after the final prereg commit.

### 4.7 Freeze and bug re-run protocol (H4, H5, H9)
* **The candidate list in section 6 (ids, books, K) is FROZEN at the tag.** No candidate is added or altered after the first run; no later batch's design may reference an earlier batch's results; the council bases are fixed now, independent of any result.
* Phase 1 closes only when every listed candidate has run or is declared not-run; if any cell is opened after the first run, every candidate's DSR is **recomputed with the larger final N**, before and after the holdout; reserve cells are released only by a dated addendum.
* **Bug re-runs (symmetric):** allowed only if (i) a failing unit test reproducing the bug was committed before the corrected run, (ii) ALL cells of the sweep are re-run under the fixed code, (iii) both runs' `returns_hash` stay in the ledger and the report shows both, (iv) the gate uses the LESS favourable run unless an independent replica confirms the corrected one. Every re-run counts as new trials and changes nothing about selection.

## 5. Phases, holdout, promotion

**5.1 Phase 1 (development + validation).** Every frozen candidate runs D (walk-forward) and V once. No H-dated bar is read. N_new_final then fixes every DSR/PBO/SPA.
**5.2 Phase 2 (class holdout, one shot, sealed batch).** Only for classes IN-EQ, IN-IDX, IN-OPT. H is touched **once per class per program**, in **one sealed batch** for all GO-capable survivors of that class; outcomes are sealed until the batch ends. The harness (i) requires `--holdout-ack=<candidate-id>` and checks the class has not been batched, (ii) checks the prereg tag, a development-GO record and the ledger, (iii) writes a class holdout-touch sweep (1 cell, counted) and commits ledger + mirror BEFORE (iv) unsealing and reading any H bar. **A burned candidate's descendants (same `parent_strategy_hash`, rule family or book) are restricted to forward paper data.** There is no retry and no new-id retry; a crash after (iii) leaves the class spent. One-touch is a **procedural control enforced by reviewed code (section 10), not a claim of impossibility.** Holdout gates: V2 (N_eff >= 30), V3 (t >= 1.65 with IG3), V4 (>= 0.5), V6, V5, OG1-OG6.
**5.3 Promotion.** A candidate passing all development gates (and the class holdout where it exists) is **PROMOTED TO FORWARD PAPER**. CRYPTO candidates and all carry lineage are promoted directly after development gates because no clean holdout exists.
**5.4 Promotion to PASSED (the exact criteria).** A promoted candidate is **PASSED** only when forward paper trading, counted from the **later of the tag date and the end of its class holdout batch** (never earlier; weeks since 2026-11-01 are seen and non-gating), shows ALL of: (a) >= **60 sessions** (trading days; calendar days for crypto) with the strategy running unattended on the pre-registered rules; (b) >= **100 closed paper trades as `N_eff`** of the paper window (2.5); (c) paper results inside the Monte Carlo 5th-95th percentile band of the development OOS; (d) net expectancy per trade > 0 after the full cost and tax model; (e) kill switch never latched in a way the Monte Carlo band does not allow; (f) OG1-OG6 hold on the paper window where the window is long enough (>= 1 year) to evaluate them, otherwise reported. A slow candidate that cannot reach 100 effective trades stays **INCONCLUSIVE-PAPER** with the shortfall stated. Live trading is a separate human decision outside this document.

## 6. Candidate catalogue (FROZEN list; every grid has >= 3 levels per tuned numeric axis, base at the centre)

Universe "books" are shared-capital portfolios. **Long-only** (no shorting of Indian cash equities); sizing <= 1.0 gross (no leverage) except pairs/carry/options where margin is stated. "Top-12 crypto" = the 12 USDT spot pairs with the highest trailing 90-day dollar volume at each monthly selection date among pairs listed >= 365 days, using only data before the date; PIT-NIFTY = the 50 point-in-time members with prices.

### 6.1 Can-reach-GO table

| Family (ids) | Cells (cap) | L basis | Eligible symbols | Can reach GO? | Reason / likely outcome |
| --- | ---: | --- | ---: | --- | --- |
| **r2-trend-lc-{sma,don,tsmom}** | 3 x 9 = 27 | trend lineage (re-test of donchian, index-trend) | 50 | **Yes, structurally** | N_eff limited: slow rules give few effective trades; INCONCLUSIVE-SAMPLE likely |
| **r2-trend-cr-{sma,don,tsmom}** | 27 | idem | 12 | **Yes**; forward paper only | no clean holdout |
| **r2-xsmom-lc** | 27 | re-test of momentum-rotation | 50 | **Yes, structurally** | monthly rebalance, k up to 15: DE large, N_eff short on 10 years: INCONCLUSIVE-SAMPLE likely |
| **r2-xsmom-cr** | 27 | new | 12 | Yes; forward paper only | same |
| **r2-mr-lc-{rsi2,zscore,ibs}** | 27 | re-test of trend-pullback (rsi2) | 50 | **Yes; most plausible on trade counts** | short holds, small k_bar |
| **r2-mr-cr-{rsi2,zscore,ibs}** | 27 | new | 12 | Yes; forward paper only | |
| **r2-pairs-{lc,cr}** | 2 x 27 = 54 | new | pairs among 50 / 12 | **Yes** (many pairs pooled) | futures-short costs unverified (lc) |
| **r2-intraday-nse-{orb,vwap}** | 2 x 9 = 18 | re-test of intraday | 50 | Yes, **conditional** on >= 3 years of free 5m bars in the inventory; else not run, not counted | |
| **GO-capable total** | **234** | | | | |
| r2-diag-volt-blend | 9 | scaling | 1 (50/50 blend) | **No**: always-invested overlay, < 10 symbols, V2 not attainable | diagnostic: measures benefit vs B3 |
| r2-diag-trend-idx | 12 | trend | 4 single instruments (SMA, L {150,200,250} on NIFTYBEES, BANKBEES, BTC-LONG, ETHUSDT; 1 axis so ring-1 = 2) | **No**: single instruments, OG5 untestable, V2 shortfall | diagnostic, INCONCLUSIVE-SAMPLE expected |
| r2-diag-carry | 9 | carry upper bound | 2 (BTC, ETH perps pooled) | **No**: 2 symbols, contaminated, no clean holdout | diagnostic |
| r2-diag-intraday-cr | 9 | intraday | 2 | **No**: 2 symbols | diagnostic |
| r2-diag-options-weekly | 36 | options-v1 | 2 underlyings | **No**: < 10 symbols; 2024 contract regime break | diagnostic (monthly re-run DROPPED: it would repeat options-v1 on identical data) |
| r2-diag-council | 8 | council-v1 | fixed bases donchian, trend-pullback, momentum-rotation, crypto-burst x {full, minimal} | **No**: no ordered axis (OG5 vacuous); bases themselves are KILL or INCONCLUSIVE | diagnostic |
| **Diagnostic total** | **83 (cap 84)** | | | | Diagnostic outcomes are capped at INCONCLUSIVE-SAMPLE or KILL and can never be promoted; they count in N honestly |

Per-family caps equal the cell counts (diagnostic cap 84). Nothing structurally unable to reach GO is run as a GO-claim candidate.

### 6.2 Grids (axis: levels; base cell in bold) and signals

* **Trend** (long when the rule is on, else flat; inverse 60-day-vol weights, gross <= 1). SMA book: price > SMA(L), L {150, **200**, 250} x vol window {40, **60**, 90}. Donchian book: close > prior N-day high, exit on N/2 low, N {40, **55**, 70} x vol window {40, **60**, 90}. TSMOM book: sign of L-day return, L {189, **252**, 315} x vol window {40, **60**, 90}. (Source: Moskowitz-Ooi-Pedersen 2012; Faber 2007; Donchian.)
* **XSMOM** rank by L-day return skipping s days, hold top k equal weight, monthly rebalance: LC: L {189, **252**, 315} x skip {0, **21**, 42} x k {5, **10**, 15}; crypto: L {60, **90**, 120} x skip {0, **7**, 14} x k {3, **5**, 8} (Jegadeesh-Titman 1993; Liu-Tsyvinski-Wu 2022).
* **Mean reversion** (above SMA200 filter fixed; exit when close > SMA(5) or max hold): RSI(2) book: threshold {5, **10**, 15} x max hold {3, **5**, 10} days; z-score(20) book: entry z {-1.5, **-2.0**, -2.5} x hold {3, **5**, 10}; IBS book: IBS < {0.1, **0.2**, 0.3} x hold {3, **5**, 10} (Connors-Alvarez; daily bars).
* **Pairs** (formation top 10 pairs by ADF p < 0.05 within sector (lc) or among top 12 (cr), OLS hedge ratio, trade the spread z, stop z 3.5 fixed): formation window {90, **180**, 270} x entry z {1.5, **2.0**, 2.5} x exit z {0, **0.5**, 1.0} (Gatev-Goetzmann-Rouwenhorst 2006).
* **Intraday NSE** (only with >= 3 y of 5m bars): ORB: range minutes {5, **15**, 30} x target R {1.0, **1.5**, 2.0}; VWAP reversion: deviation {1.0, **1.5**, 2.0} sigma x hold {30, **60**, 90} minutes.
* **Diagnostics:** volt-blend: target vol {10, **15**, 20}% x window {10, **20**, 60}; carry (BTC+ETH pooled): margin {0.5N, **0.75N**, 1.0N} x entry funding threshold {0, **5**, 10}% annualised; crypto intraday: momentum burst BTC/ETH, z {2, **3**, 4} x hold {5, **10**, 20} bars; options weekly (NIFTY and BANKNIFTY x {bull put spread, iron condor}): short offset {2.4, **3.0**, 3.6}% x width {1.6, **2.0**, 2.4}% = 4 books x 9 = 36; council: 4 bases x {full, minimal} = 8.

## 7. Costs, tax and cash

### 7.1 Rates and tax
Date-keyed `RateTable::india()` (`crates/costs/src/rates.rs`, `rates_data.rs`); `TaxModel::equity_cg()` (STCG/LTCG by exit date, 4% cess, 2024-07-23 change in the table, netted per financial year, no loss carry-forward, paid 31 March, **open positions at the window end taxed as if sold**); `TaxModel::vda_statutory()` (30% + 4% cess on the fee-free gain, no loss offset, fees not deductible; `with_tds()`, `with_gst_on_fees()`); options/F&O business income 31.2% flat (loss netting in year, 8-year carry-forward); equity intraday speculative income 31.2% (secondary). **Tax asymmetry warning:** the VDA no-loss-offset rule taxes active crypto books on every winning trade but never credits losing ones, whereas B2 is taxed once; a crypto KILL must not be read as "no edge" (report the pre-tax and after-tax numbers). **Total-return basis:** NIFTYBEES adjusted close, equities adjusted for splits, bonuses and dividends; no price-only index series (^NSEI, ^NSEBANK) is ever the P&L of a tradable (they serve calendar and regime labels only). Gold ETFs are NOT in the counted catalogue (non-equity-oriented tax treatment unverified). Every result prints the engine's `unverified rates: k of m` line.

### 7.2 Costs and slippage
* India equity/ETF delivery: `tq-costs` schedule (about 24 bps round trip on a Rs 1,000 stock) + slippage 5 bps per side up to 0.05% of 20-day ADV (impact formula above), 2x on stop exits, orders <= 1% ADV. ETFs: NIFTYBEES 5, BANKBEES 8 bps. ETF-specific STT is not modelled (delivery schedule, conservative).
* Futures leg (pairs-lc): table STT/charges by date; slippage 10 bps per side single-stock futures; lot sizes from the table (older rows unverified).
* Options: options-v1 schedule (Rs 20 per order, STT by date, exchange charge, stamp, GST), slippage `max(1% of premium, Rs 1)`, double on loss cuts.
* Crypto: spot taker 10 bps per side, slippage 5 bps BTC/ETH, 10 bps other top-12, 20 bps beyond; perp taker 5 bps per side; **all crypto costs and slippage doubled for dates before 2017-08-17** (proxy era); funding: actual 8-hourly Binance history, received funding taxed at 31.2%, paid funding not deductible (adverse scenario governs); 1% TDS as cash flow; Indian-resident access legally unverified (paper only). **USDINR prints** with |daily move| > 3% are filtered and replaced by the previous valid print.
* Stress: 1x/2x/3x slippage and 2x brokerage; 2x must stay net-positive.

### 7.3 Unverified (listed in every result)
Equity STCG/LTCG and the 2024-07-23 change (secondary); crypto 30%/4% cess/1% TDS/GST and derivative treatment; pre-2024 STT/exchange tables and lot sizes; Binance fee tiers and funding completeness; Yahoo adjusted prices; NIFTYBEES dividend treatment; no Section 112A grandfathering (B1 tax is an upper bound); USDINR conversion and USDT = USD; the PIT NIFTY 50 membership notes marked approximate (STER, RANBAXY, CAIRN, SESAGOA alias, MCDOWELL-N).

### 7.4 Idle cash (the options-v1 lesson)
The gating series (V3, V4, V8, OG1-OG6, IG1) has idle cash earning **0** (options-v1 addendum item 19). The v3 4f like-for-like line credits cash for strategy AND benchmark and is reported beside the cash-free line; V5 uses the cash-credited Sharpe as v3 requires, OG6 additionally requires the cash-free comparison to hold. Crypto USDT cash earns 0 unless a prereg states a rate (sensitivity 0-6%).

### 7.5 Survivorship and coverage (M10)
Each universe reports `coverage = member-days with a usable price / member-days total`; **coverage < 95% makes the book diagnostic-only** (cannot be promoted). A member with no usable price on a date stays in the universe as unfillable (never silently dropped). The same rule applies to delisted crypto pairs.

## 8. Assumptions and known risks

**Assumptions.** (1) Free data are accurate enough; (2) fills at next-bar open; (3) PIT membership file correct (secondary source); (4) B3 is 50/50 quarterly (owner); (5) kill switch 30%, risk 0.5%, notional cap 20%, max positions per prereg are ASSUMPTIONS the owner may only tighten; (6) the inventory will not move boundaries; (7) INR/USDT conversion at `USDINR=X`.
**Known risks.** R1 power: t >= 3 needs Sharpe 1.1-1.3, DSR needs about 1.8-2.1 (4.3). R2 trade counts: `N_eff` after the correlation haircut is short for slow and many-position books: INCONCLUSIVE-SAMPLE is likely for trend and XSMOM. R3 survivorship (secondary PIT file, delisted pairs). R4 holdouts are 2-5 years; benchmark history is publicly known. R5 holdout reuse mitigated by class-level one-shot sealed batches. R6 selection among about 450 trials is covered only by DSR/SPA with the program-wide L; L counts rows where cells are unknown. R7 tax models unverified. R8 crypto access unverified. R9 weekly vs daily Sharpe can disagree; both must hold. R10 the idle-cash exclusion penalises low-exposure books on purpose. R11 PBO at >= 20 cells under mode F can be near 0.5 for a true edge on a flat plateau, working against OG5; kept because v3 requires it.
**Ambiguities for the owner/reviewer.** A1 small books (< 10 symbols) cannot satisfy v3's >= 10 symbols clause and are INCONCLUSIVE; an owner waiver would be a relaxation and is NOT assumed. A2 OG2 requires beating all of B1, B2, B3 with a bootstrap lower bound (strict for India-only books against BTC). A3 kill switch 30% is an assumption. A4 start dates provisional until the inventory lands. A5 paper gate 100 trades (v3 s.8 says 20). A6 `N_eff` design-effect formula is mine (the owner asked for "an" overlap haircut). A7 gate variance rule 4.3 follows the review (class-cell variance, floored), with the pooled maximum reported. A8 L is row-based upper bound for the equity lineages.

## 9. What counts as a claim
Only a candidate PROMOTED TO FORWARD PAPER (section 5.3) that then meets the paper gate (5.4) under the unamended v3 gates may be called **PASSED**. Everything else is reported with equal prominence as KILL or INCONCLUSIVE; a program where nothing passes reports "nothing passed; the default stays Nifty ETF + BTC".

## 10. Harness specification (a harness build agent implements this after the tag; NO scored run may start until it is committed and independently reviewed)

1. **Caps from code:** a committed `research/caps.json` (family caps 234 / 84 / reserve 132 / total 450); `open_sweep` through the harness refuses beyond any cap.
2. **Open-gate:** the harness opens a sweep only if tag `prereg-r2-<id>` exists, its commit is an ancestor of HEAD, and `strategy_hash == sha256(prereg file at the tag)`; `code_git_sha` must be the committed harness build; commit ordering is the run-after-register evidence (ledger-open commit strictly precedes any result file in `git log`), and every result file carries the ledger tip it was produced under.
3. **Single ledger writer** (4.5): one process opens/closes sweeps serially; a lock file refuses concurrent writers; no agent worktree holds a writable ledger copy.
4. **Holdout guard:** keyed by candidate id AND class; windows come from the tagged prereg (never hard-coded); reads the ledger; writes the class holdout-touch marker (fsync, committed) BEFORE unsealing; refuses any `--holdout-ack` after the class batch ran; refuses lineage descendants of a burned candidate.
5. **Sealed holdout store:** at ingestion H-dated bars are written to a directory outside the repository and outside the development loader path (permissions 000, manifest hash only); membership, VIX, ADV and volume-rank inputs are truncated at the class development end; no quality statistics, row counts or last-bar dates beyond D are computed or printed in development; every loaded bar passes a **content-level maximum-date check** (not file-name based; equity data are one CSV per symbol).
6. **v3 statistics:** HAC bandwidth `floor(4 (T/100)^(2/9))` (fix `tstats.rs`; the holding-period floor is removed) with t at 2L and 4L shown; N_true DSR function (4.2-4.3) with the stated window and variance set; PBO with the trimming rule; SPA/White with the stationary bootstrap and studentized loss differential (IG1) with unit tests before first use; `N_eff` calculator (2.5); weekly-Friday alignment; regime labelling as of t-1; benchmark engine B1-B3 with liquidation tax.
7. **Walk-forward and integrity:** positions carry across folds; prefix-invariance, byte-identical re-run and shift tests wired as IG2; the bug re-run protocol (4.7) enforced by refusing a second run of a sweep id.
8. **Review:** the harness is tested test-first, committed, and independently reviewed (a reviewer agent that did not write it) before the first scored run; the reviewer's report is committed.

## 11. Disclosure of changes from revision 1 (for the reviewer)
Loosening-looking edits, all justified by non-v3 origin: the "no two trades share an entry day" clause was deleted (not in v3); the >= 10 symbols clause was scoped to books with >= 10 eligible symbols and small books are INCONCLUSIVE (not relaxed to pass); PBO is a gate from 20 cells (v3's scope) instead of 8; OG3 tightened to 24%; OG2 gained per-benchmark bootstraps; OG4 limited to D-OOS with a flat-regime rule; families that cannot reach GO moved to a capped diagnostic set; GLOBAL class and monthly options re-run dropped; the council conditional rule deleted. L rose from 119/26/62/11/10/3/30 per family to a program-wide 364.

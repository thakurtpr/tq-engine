# options-v2: protocol design and calibration (feasibility first)

Status: DRAFT design rationale, written 2026-11-17 on branch `hardening/o2proto` (base `hardening/integration` 09b7371). Paper and backtest only. **No strategy has been run.** (Amended by addendum A1: the HIGH benchmark scenario (ETF 0.8, BTC 0.9, blend about 1.3) is a measured research-v2 number and the BTC leg of the sealed window is already seen through the crypto development side; both are disclosed in A1.6. No candidate return exists in this document.) Every number below comes from SYNTHETIC returns placed on the REAL NSE trading calendar (`crates/r2stats/src/o2cal`, run with `tq r2stats calibrate-o2 ...`, seeds 20270000.., 500 seeds per cell, 400 bootstrap draws, agreement with 10,000 draws checked in section 13). The only real-data input is the SHAPE of weekly Nifty ETF returns (the development-side NIFTYBEES file, bad prints repaired by the addendum-4 rule, read through a plain file path, data to 2021-12-31 only); no sealed-side file and no key was touched. The companion files are `docs/prereg-options-v2.md` (the DRAFT gates, untagged) and `docs/options-v2-harness-dryrun-spec.md` (harness pins and the Study-profile design).

## 0. The finding in five lines (read this first)

1. **A modest edge (net Sharpe about 1) cannot be detected with five years of data by ANY design that keeps the false-pass rate near 5%.** The best calibrated design reaches power of 4-7% at planted net Sharpe 1, 17-28% at 1.5 (25-37% for the bare multiplicity design without quality gates), 41-62% at 2 and 75-94% at 3 (tables in section 12). The binding constraint is **data length, not the gates**: a Sharpe of 1 over the 3.5-year development window has t about 1.9, and over the 1.5-year sealed slice about 1.2, so a test that must also separate 36 hypotheses has little to work with.
2. The research-v2 gate design applied to the same windows has power 0.2-0.6% at Sharpe 2 and 11-14% at Sharpe 3 for weekly structures with Phase-2 data, **0% at every Sharpe for weekly structures without Phase-2 data** (V2 needs 200 N_eff in development) and **0% for monthly structures** (the trade-count gates cannot be met). Zero false passes in 500 seeds. So the old design was not "strict but fair"; it was blind below Sharpe 3.
3. The proposed design (section 4) targets a family-wise false-pass rate of 5% through a union bound over 36 pre-registered hypotheses spent over data stages. **That bound is CONDITIONAL on a named null family** (A1.1): under the design-pool null (negatively skewed, iid) it measures 0.0-1.2% (0.0-2.2% when a grid-best cell is picked by argmax, which the design forbids); under rare-tail and persistent-regime zero-edge nulls the staged t-test alone does NOT hold its bound, and addendum A1 reports the measured rates and the gate (G9, INCONCLUSIVE-TAIL) that is the only available remedy. It lifts power at Sharpe 2 from about 0% to **41-62%** and at Sharpe 3 to **75-94%**.
4. The deflation that hurt before (the plateau counted as variance in the DSR, neighbour gates that demanded a wide profitable plateau) is removed: neighbours are a sign-only robustness check, are never in any deflation or variance set, and the multiplicity budget is spent on **candidates**, not cells.
5. What would change the picture is more independent data, not looser gates: forward paper is the only source (about 12 months per 1.5 Sharpe-years of extra sealed evidence), or paid intraday/vendor data that adds independent observations per year. Section 14 lists the owner decisions.

## 1. What blocked research-v2 and how this design avoids each blocker

| # | Previous structural blocker (evidence: `docs/research-v2-final-report.md`, D-005, STATUS) | Effect | How options-v2 avoids it |
|---|---|---|---|
| B1 | DSR deflated by the variance of ALL cell Sharpes of the class, so a genuine plateau counted as dispersion; N_true = 537 cells | required net Sharpe 3-4; power 0/0/3% at 1/2/3 | deflation counts CANDIDATES (m = 36 hypotheses), ex-ante variance, plateau cells never enter (section 4.4); ablation A1 shows the effect |
| B2 | OG5/V7 demanded >= 80% of ring-1 and >= 60% of all cells profitable | wide plateau needed on top of B1 | one a-priori base cell; robustness = both one-step neighbours have a positive net mean (section 9) |
| B3 | 24% absolute drawdown limit below the Nifty ETF's own worst (34.3%) | the default ETF plan fails | limit relative to the benchmark's drawdown in the same window (section 7) |
| B4 | V10 "ledger complete" required no unclosed sweep, including NOT-RUN families | no GO possible | NOT-RUN catalogue entries consume no cell, are excluded from every derivation set, and a closed-sweep requirement applies to RUNNABLE candidates only (spec item 9) |
| B5 | deflation sets over not-run families (addendum 7) | V8-DSR/IG1 NOT-EVALUATED for all | candidate-level deflation needs no cell set at all; the SPA cross-check set is the registered candidates' base cells |
| B6 | tax subtracted as a lump from the pre-tax curve, no loss offset (crypto): after-tax equity <= 0 | series ruined, verdict INCOMPLETE | provisional tax on positive YTD net business income only; fail closed on any equity <= 0 (section 11) |
| B7 | bad benchmark prints (NIFTYBEES 2019-12-19/20) found after kills | late addenda | every input series is scanned and repaired BEFORE the freeze; the calibration itself repairs the benchmark file the same way |
| B8 | trade-count gates (200/100/30 N_eff) unreachable for monthly structures | monthly baseline power 0 at every Sharpe | minimum N_eff 30 (D1) and 10 (sealed), justified from the tail-event arithmetic (section 10); per-family reach stated; families that cannot reach are labelled DIAGNOSTIC up front |
| B9 | one-shot class holdout with a single window and no design for the power it leaves | sealed result could not be interpreted | sealed slice is per candidate, single touch, with its own alpha and its own N_eff requirement (section 5) |
| B10 | gates fitted to nothing: they were written before any evaluation of their power | infeasible by construction | this document: gates are chosen from the calibration BEFORE any strategy runs, and the calibration code and seeds are committed |

## 2. Unit of analysis, K, selection mode

* **A candidate is ONE pre-registered base structure** (structure x underlying class x expiry type x one numeric parameter vector, written before any run) with a **tiny grid of 3 cells on ONE parameter axis** (base in the middle, two one-step neighbours). The neighbours exist only for the robustness check and for the ledger count.
* **Selection mode: a-priori base cell, no argmax, no best-of-grid.** The base cell is the only cell eligible for any gate other than robustness. Calibration shows why: picking the best of the 3 cells by Sharpe raises the null false-pass rate 2-3x (0.8 -> 2.0% weekly, 0.4 -> 0.6% monthly in the `P proposed` row, columns "base" vs "argmax").
* **K budget.** Initial wave = the 17 candidates of the capability audit (`docs/options-v2-capability-and-catalogue.md` section 7: C1..C17, one axis of 3 levels each; the naked stress cell S1 is a labelled stress, not a candidate) = 17 x 3 = 51 cells + 1 stress cell = 52, plus a reserve of 8 cells for dated re-runs = **60 cells cap**. The multiplicity budget is fixed at **K_cap = 30 candidate slots** (17 planned, 13 unfilled), so m = K_cap + 6 (options-v1 lineage) = **36 hypotheses** whether or not the slots are used (unfilled slots cannot be used to loosen later; a new candidate needs an addendum BEFORE its first run and takes a slot).
* Per family (audit families): 2-3 candidates each: short premium weekly (C1-C4, C16), short premium monthly (C5, C9, C10), long premium (C6, C11), calendar/diagonal (C7), butterflies/ratio (C3, C8, C17), expiry-day (C13), event (C14, C15), delta-hedged (C12). No family has more than 5.
* Smaller and larger designs are tested too (section 12.3): K = 17 (m = 23) changes power at Sharpe 2 by 0 points for type-A weekly and by about +1 to +3 for type B; K = 60 (m = 66) costs 1-7 points (e.g. weekly type B, LOW: 61.2 -> 54.2). K is therefore not what binds; data length is.


## 3. Evidence standard

A candidate is a **development GO** only if its base cell passes every gate of every stage that applies to it, in the fixed order below, on NET-of-cost-and-tax weekly (or, for monthly-cycle structures, calendar-monthly) after-tax returns. A development GO is then **PROMOTED TO FORWARD PAPER**, never "passed" (STATUS rule 4 and D-006 item 6 are unchanged).

| Stage | Data | What it tests | Alpha (one-sided) |
|---|---|---|---|
| S1 development | D1 = 2021-11-01 .. 2025-04-30 (42 months, 182 weekly periods; train 2021-11..2023-10 = 24 months, validation 2023-11..2025-04 = 18 months) | edge t-test + quality gates | alpha_1 (derived below) |
| S2 Phase-2 replication | Phase 2 = 2012-01-01 .. 2021-10-31 where the structure's contracts exist for >= 24 months (**type A** candidates); development data, already seen by options-v1 (6 monthly structures only) | independent replication of the sign and size of the edge | 0.10 |
| S3 sealed slice | 2025-05-01 .. 2026-10-31 (18 months, 79 weekly periods), single touch | independent confirmation | 0.10 |

A candidate with fewer than 24 months of Phase-2 data (**type B**, e.g. recent weeklies, FinNifty/Midcap) skips S2 and gets a stricter alpha_1. The type is fixed at registration from the data manifest and cannot change after a result.

### 3.1 The family-wise error target and where it is spent

Target: **P(any false development GO among all registered candidates) <= 5%** under the global null (no candidate has a net edge).

* hypotheses m = 36 (section 2). Each candidate gets the share 0.05/36 = 0.00139.
* The stages use disjoint calendar periods, so under a zero-edge null their tests are independent and the probability that a null candidate passes all of them is the PRODUCT of its stage alphas (the union bound over candidates then gives the family-wise bound; the bound needs no assumption about the dependence between candidates):
  * type A: alpha_1 x 0.10 x 0.10 = 0.00139, so **alpha_1 = 0.139**;
  * type B: alpha_1 x 0.10 = 0.00139, so **alpha_1 = 0.0139**.
* **Why this spends the budget better than one big test.** A single stage over the whole 5 years at the Bonferroni level 0.05/36 needs t >= 3.0; splitting the evidence across independent stages lets each stage use a modest hurdle and the product does the work. The ablation rows `P-single` (two stages only, Bonferroni 0.00139 on D1) and `P-core` (the staged design with no quality gate) in section 12 show the trade: the staged design is more powerful at every Sharpe with the same bound.
* **Honest size.** The HAC t-statistic of a premium-selling series is anti-conservative (negative skew, fat tails): measured rejection rate of a nominal 1.4% test is 2.9% (lag auto, 182 periods), 4-5% with 4 overlapping positions (section 5). The hurdles are therefore **size-corrected for the design-pool null only** (see A1.1: for rarer-tail nulls no finite hurdle restores the size): the (1 - alpha) quantile of the t under a negatively skewed fat-tailed zero-mean null, for the stage's n, H (table in 3.2). Measured family-wise false-pass rate of the full design under the design-pool null, 500 seeds: 0.2-0.8% (weekly), 0.2-0.4% (monthly) with a-priori base cells. The claim "at most 5%" holds for that null; it is not claimed for others (A1.1).

### 3.2 Pre-registered hurdles (HAC t of the mean; lag = max(auto, 2H); worst-case-skew null; 20,000 simulations, `calibrate-o2 hac`)

| n periods | H (periods held) | z at alpha 1.39% (type B S1) | z at 13.9% (type A S1) | z at 10% (S2, S3) | normal values |
|---|---|---|---|---|---|
| 182 (D1 weekly) | 1 | 2.60 | 1.18 | 1.42 | 2.20 / 1.08 / 1.28 |
| 182 (D1 weekly ladder) | 4 | 2.92 | 1.33 | 1.57 | |
| 142 (Phase 2, weekly from 2019-02) | 1 | 2.69 | 1.21 | 1.43 | |
| 118 (Phase 2, monthly) | 1 | 2.72 | 1.23 | 1.46 | |
| 79 (sealed weekly) | 1 | 2.89 | 1.29 | 1.55 | |
| 42 (D1 monthly) | 1 | 3.33 | 1.42 | 1.73 | |
| 18 (sealed monthly) | 1 | 3.91 | 1.76 | 2.12 | |

For any other n the harness recomputes the quantile with the same routine (`emp_crit`, fixed seed, deterministic) and the result is written into the sweep record. The hurdle table is part of the freeze.

### 3.3 Why not "DSR >= 0.95 at the true N"

The DSR gate is kept as a REPORTED statistic (candidate-level N = 36, ex-ante variance 1/T, plus N_all in section 8) but is not a gate, for three measured reasons. (a) DSR >= 0.95 at N = 36 means observed Sharpe >= SR0 + 1.645 sd, i.e. z about 3.8 on D1; that is a far stricter standard than the 5% family-wise target it is meant to serve (a union bound at 5% needs z about 3.0 on a single stage). (b) Replacing the staged hurdle by the DSR in the proposed design (row `P-DSR`) drops power at Sharpe 2 from 61.6% to 26.0% (weekly) and 50.2% to 16.4% (monthly) while the measured null false-pass is 0.0% in both: the extra strictness buys nothing the target asks for. (c) D-006 item 3 asks for DSR plus White's reality check or SPA: the SPA cross-check of IG1 is kept (section 4.5) as a **reported** second multiple-testing statistic over the candidates' base cells; the staged union bound is the gate. Owner ruling (2026-11-17): DSR (N = 36 and N_all) and SPA p are printed on every verdict line, a DSR at N = 36 below 0.50 is printed with a WEAK flag (not a gate, but a WEAK candidate needs the owner's confirmation before forward paper), SPA is NOT-EVALUATED and non-blocking when a registered candidate has no returns, and no options-v2 verdict ever uses the words "passed" or "profitable". **This is a deliberate departure from the letter of D-006 item 3 and an owner decision (section 14, Q1).**

## 4. Multiple testing, deflation and the neighbour problem

### 4.1 Which design: hierarchical, candidate-level

* Level 1 (gate): staged candidate-level union bound over m hypotheses (3.1).
* Level 2 (non-deflating checks): robustness of the base cell to its two neighbours (section 9), regime and year stability (section 8), drawdown (section 7), benchmark dominance (section 6).
* Level 3 (reported, not gating): DSR at N = 36 candidate-level and N_all, SPA p-value across base cells, PBO is **not computed** (3 cells per candidate is below the 20-cell scope of v3 and neighbours are not trials).

### 4.2 Parameter neighbours never inflate the deflation set

Neighbour cells are written to the ledger (they cost a reserve-free count: every opened cell is a trial for the honest counter N_cells) but they are NOT hypotheses: they cannot be the evaluated cell (no argmax), they enter no variance set, no SPA model universe, no PBO matrix and no DSR. The calibration enforces this: the planted plateau decays 0.9 per grid step (and 0.7 in section 12.4); the design's power does not depend on how wide the plateau is, because only the SIGN of the two neighbours is read.

### 4.3 Trial accounting (running counter; N_true formula)

* **Ledger counter (every opened cell, never reduced):** N_cells = 30 (options-v1 K, family `options-v1`, 6 variants x 5 params) + 52 planned cells (51 + 1 stress) + reserve cells actually opened (<= 8) + markers and sealed-touch markers (1 per touch). The program-wide figure for information (N_all) adds research-v2's 173 opened cells and the 364 lineage rows of `docs/prereg-research-v2.md` 4.4 (those are different books on different classes: reported, not used to deflate options candidates).
* **Earlier options sweeps in `research/trials.csv`:** the only options rows are the options-v1 summary row (K = 30, 6 variants). No earlier options sweep exists in the ledgers (research-v2's OPTIONS60 and `r2-diag-options-weekly` were NOT-RUN and opened no cell). This was checked in `research/trials.csv` (2 rows mention options: options-v1 and the research-v2 master marker) and `research/ledger-options-v1.jsonl` (30 cells).
* **Deflation count used by the design:** `N_true(options-v2) = m = K_cap + 6 = 36` hypotheses (the 6 are the options-v1 base variants V1-V6, each of which was eligible for GO on its base cell). Sensitivity (reported): N = 30 + 52 + 8 + markers (about 95) with ex-ante variance.
* Pre-registration order is enforced by the ledger exactly as in research-v2 (no sweep opens before its prereg tag).

### 4.4 What the cell-level deflation did (ablation)

Section 12 rows A1..A4 move the research-v2 design one change at a time: A1 replaces the plateau-variance DSR at N_true = 807 by a candidate-level DSR (N = 36, ex-ante variance): power at Sharpe 3 rises from 14% to 48% (weekly); A2 adds sign-only neighbours: no further change (the plateau gate was not binding once the DSR was fixed in this plateau-0.9 world); A3 the relative drawdown limit: none (drawdown is not binding for a 10%-volatility book); A4 quarters and N_eff 30: +5 points. The staged design then adds the largest step (+40 points at Sharpe 3 relative to A4, and the monthly profile becomes testable). The binding gates of every row are listed in section 12.2.

### 4.5 SPA / White (reported)

IG1 stays as a REPORTED statistic: Hansen's SPA with the studentised loss differential versus B1 over the model universe of all registered candidates' base cells on D1 (stationary bootstrap, mean block max(4, 2H) periods, 10,000 draws, seed 20261117 as in research-v2). It cannot reduce the false-pass rate below the staged union bound by more than a correlation effect (candidates share a crash factor: calibration correlation 0.09 across families, 0.29 within), so it is not a gate.

## 5. Overlapping and serially dependent structures

* **Unit of analysis.** The weekly account-level return series on the Friday-cut calendar (last NSE session of each ISO week; the real holiday table is used, e.g. Thursday when Friday is a holiday). Monthly-cycle structures are analysed on **calendar-monthly** returns (the months of the same grid), NOT on weekly marks of a monthly structure: a monthly position marked weekly has serially dependent, smoothed returns and an annualised Sharpe computed on them is inflated (the weekly series of a 4-week position is a moving average; the naive Sharpe overstates by about sqrt(4)). The weekly mark-to-market series is still used for drawdown.
* **Where correlation comes from.** (i) A structure held H periods and opened every period (a ladder) makes trade-level returns an MA(H-1) process: adjacent trades share H-1 of H periods of market move. (ii) Consecutive weekly outcomes share a volatility regime (autocorrelation of per-trade returns). (iii) Several underlyings in one candidate share the index factor (harness rho_bar).
* **Calibrated size under the null** (`calibrate-o2 hac`, n = 182, 20,000 sims; rejection of t >= 2.20, nominal 1.4%):

| skew | H | lag 0 | lag auto | lag max(auto, 2H) |
|---|---|---|---|---|
| negative | 1 | 2.6 | 2.9 | 2.9 |
| negative | 4 | 15.7 | 4.9 | 4.3 |
| symmetric | 1 | 1.5 | 1.6 | 1.6 |
| symmetric | 4 | 14.0 | 3.4 | 3.0 |
| positive | 1 | 0.8 | 1.0 | 1.0 |
| positive | 4 | 12.9 | 2.3 | 1.9 |

A naive t is 10x too liberal with a 4-period overlap; HAC with lag max(auto, 2H) brings it to 2-4% and the size-corrected hurdles (3.2) remove the rest. **HAC lag = max(floor(4 (n/100)^(2/9)), 2H)**, H = periods a position is held. **Bootstrap block** (paired Sharpe bootstrap, drawdown Monte Carlo): mean block = max(4, 2H) weekly periods (2H monthly periods for monthly cycles).
* **N_eff (independent trades), stricter than the harness.** `neff.rs` falls back to rho_bar = 1 for a single underlying, so non-overlapping weeklies get N_eff = N_raw. The protocol therefore uses
  `N_eff = N_raw / max(DE_harness, DE_time)`, `DE_harness = 1 + (k_bar - 1) rho_bar`, `DE_time = max(1, 1 + 2 sum_{k=1..4} (1 - k/5) rho_k)` with rho_k the autocorrelations of the per-cycle net returns of the window. Both counts are reported; the gate uses the lower N_eff.

## 6. Benchmarks and Sharpe above benchmark

* **B1** Nifty ETF NIFTYBEES total return (dividends reinvested), **B3** fixed 50/50 B1 + BTC-in-INR, weekly rebalanced in the calibration (quarterly in production, as research-v2 2.3); same window, same cost engine, same tax-as-if-liquidated convention as the candidate. B2 (BTC alone) is reported, not gated. Both benchmarks are built on the candidate's own period grid (weekly or monthly) over each stage window.
* **Test.** Annualised Sharpe (rf = 0) difference `SR_candidate - SR_benchmark` of the stage window with a paired stationary bootstrap of (candidate, benchmark) period pairs (mean block max(4, 2H) periods, seed fixed per stage, 10,000 draws in production).
  * S1 (D1): point estimate > 0 AND the one-sided **80% lower bound > 0 against B1**; against B3 the point estimate must be > 0 (no bound).
  * S2 (Phase 2): point estimate > 0 against B1 (B3 does not exist for the whole phase).
  * S3 (sealed): point estimate > 0 against B1 and B3 (18 months cannot carry a bound).
* **Why this level.** The staged t-test already controls the family-wise error; the benchmark gate is a "would I rather hold the default" screen. Calibration (section 12, rows "bound 90% / 80% / point estimates") at the same family-wise bound: weekly (Phase-2, LOW benchmarks) power at Sharpe 2 is 51.2% with a 90% bound on both benchmarks, 58.2% with an 80% bound on both, 61.6% with the proposed 80%-on-B1 + point-on-B3 rule and 64.6% with point estimates only (table 12.1 has every row); the measured null false-pass is 0.0-0.8% in every row.
* **The benchmark's own Sharpe in the window is a fact of the window, not of the design.** The calibration brackets it: LOW (ETF 0.5, BTC 0.4, blend about 0.7) and HIGH (ETF 0.8, BTC 0.9, blend about 1.3, the value research-v2 measured over its crypto window). The harder the blend did over the window, the lower the power: at Sharpe 2 the proposed design has 61% (LOW) and 51% (HIGH) power for the weekly shape without Phase 2 data. This is economically right (a book that did not beat the blend should not displace it) and is stated to the owner as a dependency the design cannot remove.
* **Data dependency.** NIFTYBEES for 2022-2026 is not on the development side (it ends 2021-12-31): B1 for D1 and the sealed slice comes from the o2data pull, loaded under the same bad-print repair rules (addenda 4 and 6), never from the sealed store.

## 7. Maximum drawdown limit

* **Rule.** For each stage window w (D1, P2-available, sealed) the base cell's maximum drawdown of its after-cost, after-tax equity (kill switch OFF) must satisfy
  `MDD_candidate(w) <= clamp(1.5 x MDD_B1(w), 25%, 40%)`.
  The Nifty ETF's own worst drawdown in every window (and the blend's) is **reported in every result** (the research-v2 figure was 34.3% in 2015-2021; the 2022-2026 figures are not read here because the data are on the sealed side).
* **Reasoning.** A limit below the benchmark's own drawdown makes the default plan fail (B3 of section 1). The limit is relative so it moves with the regime of the window; the floor (25%) stops a calm window from setting an unreachable limit; the cap (40%) stops a violent window from licensing a catastrophic book (40% is above the ETF's 34.3% worst but below a loss at which a defined-risk options account is in margin distress). The kill-switch operating run at 30% (assumption, D-005) is reported separately and does not gate.
* **How many ETF-like books pass** (synthetic ETF-shaped books: 0.9 x benchmark + independent noise, same volatility, real weekly shape; 1,000 seeds; `calibrate-o2 dd`):

| window | absolute 24% (research-v2) | absolute 35% | relative 1.0x | 1.25x clamp(20,35) | **1.5x clamp(25,40) (proposed)** | mean benchmark MDD in the synthetic window |
|---|---|---|---|---|---|---|
| D1, 42 months | 46.7% | 80.6% | 46.9% | 69.3% | **85.9%** | 26.1% |
| sealed, 18 months | 77.7% | 94.8% | 48.8% | 86.0% | **96.6%** | 18.5% |
| Phase 2, 9.8 years | 9.7% | 43.2% | 43.7% | 39.2% | **58.4%** | 37.4% |

  So the research-v2 24% rule would reject more than half of ETF-like books in D1 and 90% in Phase 2; the proposed rule passes 86%, 97% and 58% (Phase 2 contains a 2020-type crash for which a 40% cap is deliberately not generous). The synthetic window MDDs are larger than the real ETF's in recent years because the benchmark pool contains the 2020 crash; the proposed numbers are therefore conservative for D1 and sealed.
* The weekly equity path understates intra-week drawdown; the production gate uses the daily MTM account series (spec item 23) with the same limits.

## 8. Stability across years and regimes

* **Quarters, not years.** D1 contains only 3 full calendar years, so "60% of years" is "2 of 3". Gate: **>= 60% of full calendar quarters (>= 10 weekly periods, or 3 monthly) have a positive compounded net return** (at least 5 evaluable quarters, otherwise not applicable). Plus: the net mean is positive in BOTH the 24-month train and the 18-month validation sub-windows, and dropping the best 1% of periods (at least 2) leaves total net return > 0.
* **Regimes, causal.** Trend = benchmark close of the previous period above its 40-week SMA of the previous period; volatility = 13-week realised volatility of the benchmark's weekly returns as of the previous period above its expanding median (as of the previous period, starting after 52 weeks). The regime of a period is the label of the week BEFORE it starts. Four regime cells; cells with fewer than 20 weekly periods (5 monthly) are excluded; **at least 75% of evaluable cells (rounded up, minimum 2 evaluable, else not applicable) have a positive mean net return.** (research-v2: 3 of 4, fewer than 3 evaluable = FAIL; here not-applicable is not a fail, so a short window cannot make GO impossible.)
* Calibrated cost: quarters + regimes + halves fail 3-6% of true-Sharpe-2 candidates (section 12.2).

## 9. Parameter robustness without touching the deflation set

* The two one-step neighbours of the base cell (same structure, one numeric parameter moved one pre-registered step each way) must both have a **positive net after-tax mean on D1**. That is all. No share-of-plateau, no median-Sharpe ratio, no second-order neighbours.
* Neighbours are ledger cells (counted) but are never hypotheses, never evaluated cells, and never enter a variance set, the SPA universe or the DSR. Under the planted plateau (0.9 per step, and the harsher 0.7) the gate removed 0-2 points of power (not in the binding lists).
* Categorical choices (structure type, underlying class, expiry type) are separate candidates, not axes.

## 10. Enough independent trades

* Minimum **N_eff >= 30 in D1** and **>= 10 in the sealed slice** (N_eff by 5). Basis: for a premium-selling structure with a 1-in-10 loss-cycle rate, P(no loss cycle in N independent cycles) <= 5% needs N >= 29, so 30 is the smallest sample in which the book has probably seen its tail at all; the sealed minimum of 10 is the smallest at which the sealed t-test is defined with a non-degenerate bootstrap. Below the minimum the verdict is **INCONCLUSIVE-SAMPLE (not KILL, not promoted)**.
* **Which structures can reach it** (counts from the real calendar: D1 = 182 weeks / 42 months, sealed = 79 weeks / 18 months, Phase 2 = 142 weeks of Nifty weeklies from 2019-02 or 118 months; trade-count expectations per family from the capability audit section 7.2; weekly-expiry dates are RECOLLECTION until the o2data inventory verifies them):

| Family | D1 raw cycles | sealed raw | N_eff D1 >= 30 | N_eff sealed >= 10 | Verdict on reach |
|---|---|---|---|---|---|
| Weekly Nifty (single index, non-overlapping) | 182 | 79 | yes (DE_time ~1-2) | yes | **can reach GO** (type A) |
| Weekly ladder of 4 (monthly structure entered weekly) | 182 | 79 | 182/4 = 45 | 20 | can reach |
| Weekly BankNifty / FinNifty / Midcap | about 100-180 | **0 if the weekly contracts were discontinued 2024-11-20 (SEBI one-weekly-expiry rule, recollection)** | yes | **no contracts in the sealed slice** | **DIAGNOSTIC: cannot reach GO by construction (sealed slice empty)**; kept as development evidence |
| Monthly, one index | 42 | 18 | only if DE_time <= 1.4 (about 94% of iid draws pass) | 18/DE >= 10 needs DE <= 1.8 | borderline; reaches only with a pooled Nifty + BankNifty candidate (84 cycles, harness DE about 1.85 gives 45) or with Phase 2 |
| Monthly stocks, pooled about 40 names | 1,680 | 720 | 1680/16.6 = 101 | 43 | can reach (rho_bar 0.4) |
| Expiry-day open-to-settle (weekly) | 182 | 79 | yes | yes | as weekly Nifty |
| Event: Budget | about 4 | about 1 | no | no | **DIAGNOSTIC: cannot reach GO by construction** |
| Event: RBI MPC | about 21 | about 9 | no (< 30) | no (< 10) | **DIAGNOSTIC: cannot reach GO by construction** |
| Event: results (stocks, pooled) | about 600 | about 250 | clustered: rho_bar by same-week results | | can reach after the clustering haircut |
| Daily-hedged straddle (weekly) | 182 | 79 | yes | yes | as weekly |

* Also unreachable for a structural, not statistical, reason: any candidate whose contracts do not exist in a stage window cannot pass that stage; stage availability is declared in the registration (the data manifest) and the verdict is DIAGNOSTIC for the empty stage rather than a pass or a fail.

## 11. Windows, sealed-slice rule, costs, tax and the after-tax series

### 11.1 Window plan

| Window | Dates | Length | Role |
|---|---|---|---|
| Phase 2 | 2012-01-01 .. 2021-10-31 | 9.8 y (structure availability varies) | development-only out-of-time check; already seen by options-v1 for its 6 monthly variants: any candidate classified TYPE B by the type rule (A1.2: v1-lineage, cousin, or otherwise seen) runs S2 as a replication/consistency check that spends no alpha and cannot count as an independent confirmation |
| D1 train | 2021-11-01 .. 2023-10-31 | 24 months | development (no parameter is fitted: base cells are a priori; the split is used for the halves-consistency gate) |
| D1 validation | 2023-11-01 .. 2025-04-30 | 18 months | development |
| Sealed slice | 2025-05-01 .. 2026-10-31 | 18 months | single touch |

* **Why 42 + 18 and not 30 + 30.** Power is almost flat in the sealed length for a fixed family-wise bound (arithmetic on normal approximations: for Sharpe 1.5, sealed 1.0 / 1.5 / 2.0 years give 46% / 52% / 52% for type B), so the sealed slice is kept at 18 months (one volatility cycle plus the 2025-26 regime: the 2024-10 and 2026-04 STT increases are inside the data) and the longer window goes to S1.
* The Phase-1 window is the options-v1 never-opened holdout window; D-006 declares it consumed as development data. **The only clean holdout is forward paper from 2026-11-17.** Dates are fixed by this draft and by the calendar of the sealed side (the cut 2025-05-01 is a month boundary and not a function of any return).
* **Sealed-slice single-touch rule.** (1) A candidate's sealed slice is read at most once, only after S1 and S2 are PASS and the integrity gates (look-ahead prefix test, byte-identical re-run) are PASS; (2) the touch is a ledger marker written BEFORE the read, on its own cap axis `o2-sealed-touch` (A1.4), not a reserve cell; (3) after the touch the candidate's verdict for the sealed stage is final whatever it is: there is NO re-run after the sealed batch (A1.4 replaces the per-candidate touch by ONE sealed batch and deletes the new-slot re-run clause); (4) the five sessions 2025-10-27..31 burned by options-v1 are part of the slice and do not affect this rule (they were never used for a return); (5) nothing at all may read a file dated on or after 2025-05-01 during development (the harness fails closed on a sealed-dated file content check, as `DevReader` does today).

### 11.2 Costs and slippage

As `docs/costs-tax-options-india.md` and the date-keyed rates table (`crates/costs/src/rates_data.rs`; every row carries a `verified` flag; options STT 0.15% from 2026-04-01 and NSE transaction charge 3.553 bp from 2026-03-01 are verified from primary text; the earlier STT steps 0.017% / 0.05% / 0.0625% / 0.10% and the pre-2024 exchange charge are secondary and flagged in every result). Brokerage Rs 20 per executed order. Slippage assumption: each leg, each side, fill at the day's OPEN moved against us by `s = max(1% of premium, Rs 1)` (2s for stop/loss-cut exits), stress at 1x, 2x, 3x; **2x slippage and 2x brokerage must keep the base cell's net mean positive** (a sign check on the S1 series, not a new test). Liquidity filter and margin model as the capability audit sections 2.2 and 3 (defined-risk margin = maximum loss; SPAN-like only for futures legs and calendars; naked only as a labelled stress test with a hard loss cap, never a candidate).

### 11.3 Tax and the after-tax series

* Income class: **non-speculative business income, 31.2% flat (30% + 4% cess; surcharge ignored)** on positive annual net business income, STT deductible, losses carried forward 8 financial years against future business income, tax paid out of the account on 31 March. This is the orchestrator default (carry-forward only; a same-year symmetric offset against other income is a reported sensitivity, 39% and 20% likewise, per audit OQ1).
* **Assumption stated:** the account is standalone (the owner's other income is not modelled); losses offset only against this account's future F&O profit. If the owner has other income, same-year relief is more favourable than modelled.
* **After-tax series definition (fail-closed, never ruined by tax).** Weekly after-tax equity = pre-tax equity - provisional tax, where the provision is `31.2% x max(0, brought-forward-loss-adjusted YTD net business income)` computed on the CUMULATIVE financial-year figure (marked to market, open positions included). The provision rises and falls with the YTD figure, so there is no asymmetry between a gain taxed and a later loss ignored (the research-v2 crypto failure), the after-tax equity can never fall below pre-tax equity minus 31.2% of a profit, and tax is never "paid" in excess of equity. At each 31 March the provision crystallises (cash out) and a net loss is carried forward. **Any period return <= -100% or non-finite is non-physical: the consuming gates are NOT-EVALUATED and the verdict INCOMPLETE, never PASS** (the guard exists in `r2stats/src/physical.rs`; `metrics.rs:32` returns 0 when equity <= 0 and hides ruin: the options runner must not call it for equity <= 0). The calibration enforces the same rule (gate `PHYS`).
* Account size: Rs 1 crore FIXED capital; returns are P&L on that fixed capital with NO cash credit (A1.3); sizing is the fixed-notional rule of A1.5; Rs 25 lakh is a reported sensitivity with the same rule. The calibration assumes a book of about 10% annual volatility, inside the 6-14% band that A1.3 requires of every candidate.

## 12. Calibration results (synthetic returns on the real calendar)

### 12.0 Method (what was simulated, stated assumptions)

* **Calendar:** real NSE sessions from `tq-calendar` (holiday table 2012-2026), Friday-cut weeks (774 weeks 2012-01..2026-10), calendar months; windows as section 11. Periods per window:

## Periods per window on the real NSE calendar

| shape | Phase 2 available | D1 | D1 train | D1 validation | sealed | three-stage |
|---|---|---|---|---|---|---|
| weekly (Phase-2 data from 2019-02) | 142 | 182 | 104 | 78 | 79 | true |
| weekly (no Phase-2 data) | 0 | 182 | 104 | 78 | 79 | false |
| monthly (Phase-2 data from 2012) | 118 | 42 | 24 | 18 | 18 | true |

weeks in 2012-01..2026-10: 774

* **Candidate returns:** per period `mu + sigma (sqrt(.09) b + sqrt(.20) f + sqrt(.55) c + sqrt(.16) x)`; `b` = standardised benchmark return (real weekly Nifty-ETF shape, bad prints repaired, stationary bootstrap block 4), `f` family shock, `c` candidate shock, `x` cell shock, all from a standardised NEGATIVELY skewed fat-tailed pool (6% crash periods of mean -3.2 sd, skew about -2): premium-selling shape. Correlation within candidate 0.84, within family 0.29, across 0.09. Annual volatility 10%. Planted NET Sharpe `S` (after cost and tax by definition) in the base cell, neighbours decay 0.9 per step (0.7 in 12.4). Null = S 0. K = 30 candidates in 8 families, candidate 0 carries the edge in power runs; null runs evaluate all 30 and count a **family-wise false pass = any candidate passes**.
* **Shapes:** weekly with Phase-2 data from 2019-02 (type A), weekly without Phase-2 data (type B), monthly cycles with Phase-2 data from 2012 (type A).
* **Benchmark scenarios:** LOW (ETF Sharpe 0.5, BTC 0.4) and HIGH (0.8, 0.9; blend about 1.3).
* **Designs.** R2 = research-v2 gates reduced to these windows (HAC t >= 3, Sharpe >= 0.8, DSR >= 0.95 weekly and monthly at N_true = 30 x 9 + 537 = 807 with the class-cell Sharpe variance, paired 90% bound vs B1 and B3 in D1 and the sealed slice, 24% MDD and not deeper than B1, MC p95 MDD <= 24%, years >= 60%, regimes 3 of 4, OG5/V7 neighbour gates on a 3 x 3 grid, N_eff 200/100/30, holdout t >= 1.65 and Sharpe >= 0.5 and >= half). A1-A4 change one thing at a time (A1 candidate-level DSR N = 36 with ex-ante variance; A2 sign-only neighbours on 3 cells; A3 relative drawdown; A4 quarters, no Sharpe floor, N_eff 30/10). P-single = Bonferroni t on D1 with a sealed stage only. P-DSR = P with the DSR gate in place of the t hurdle. P-core = staged t hurdles only. "bound 90/80, point" = benchmark rule variants. P-loose = alpha_P2 = alpha_seal = 0.20. **P proposed** = section 3-11.
* Every power figure in this section is an UPPER BOUND under a constant planted edge on the design-pool null; addendum A1 re-states the headline numbers with the added gates. Not simulated (stated limits): serial dependence beyond MA(H) and the 4-lag design effect; edge that is lumpy in time (the planted edge is uniform); regime-dependent edge; time-varying volatility of the book; slippage/margin/tax mechanics (net Sharpe is planted after them); weekly MDD (intra-week understated); the integrity gates; real option-chain mechanics. Planted Sharpe is stated per year with sqrt(52) (weekly) or sqrt(12) (monthly) annualisation of the period series.
* 500 seeds per cell (null and each planted Sharpe), 400 bootstrap draws; binomial standard error of a power figure is up to 2.2 points; null rates are shown to 0.2.

### 12.1 Power and false-pass tables (K = 30, m = 36, plateau 0.9)

Columns: family-wise null false-pass % with the a-priori base cell; with the grid-best (argmax) cell; per-candidate null %; power % at planted net Sharpe 0.5 / 1 / 1.5 / 2 / 3.

## Calibration: K=30 candidates (m=36 hypotheses), plateau 0.9, 500 seeds from 20270000, 400 bootstrap draws, assumed account vol 10%

### weekly (Phase-2 data from 2019-02) | benchmarks low (ETF 0.5, BTC 0.4)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.6 / 14.4 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.2 / 1.8 / 10.8 / 48.0 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.0 / 10.8 / 48.2 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.0 / 10.8 / 48.8 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.2 / 12.8 / 54.4 |
| P-single (Bonferroni t on D1, 2 stages) | 0.6 | 0.8 | 0.0 | 0.6 / 4.4 / 21.0 / 48.4 / 84.0 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 2.4 / 23.6 / 80.4 |
| P-core (t hurdles only) | 1.0 | 1.4 | 0.0 | 0.6 / 10.2 / 36.2 / 68.2 / 94.0 |
| P (bound 90%, B3 bound too) | 0.0 | 0.0 | 0.0 | 0.2 / 3.0 / 20.2 / 51.2 / 90.6 |
| P (bound 80% on B1 and B3) | 0.0 | 0.4 | 0.0 | 0.4 / 4.6 / 24.6 / 58.2 / 92.4 |
| P (point estimates only) | 0.6 | 0.8 | 0.0 | 0.6 / 8.8 / 32.0 / 64.6 / 93.4 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 0.4 | 1.0 | 0.0 | 0.4 / 7.2 / 36.0 / 68.2 / 95.8 |
| P proposed | 0.2 | 0.4 | 0.0 | 0.6 / 6.6 / 28.4 / 61.6 / 93.2 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: DEFL 98.2%, BENCH S 50.4%, YEARS 32.4%, BENCH D1 25.6%, E1 t(D1) 21.8%
- P proposed: E3 t(S) 19.6%, BENCH S 13.6%, BENCH D1 8.6%, E2 t(P2) 4.6%, REGIMES 4.0%

### weekly (Phase-2 data from 2019-02) | benchmarks high (ETF 0.8, BTC 0.9)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.2 / 11.2 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.6 / 6.4 / 36.6 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.0 / 7.0 / 35.8 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.0 / 7.0 / 36.2 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.2 / 8.0 / 39.0 |
| P-single (Bonferroni t on D1, 2 stages) | 0.2 | 0.6 | 0.0 | 0.2 / 3.2 / 15.2 / 37.4 / 72.2 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 2.2 / 21.4 / 78.8 |
| P-core (t hurdles only) | 0.4 | 0.4 | 0.0 | 0.4 / 6.4 / 24.8 / 56.6 / 92.6 |
| P (bound 90%, B3 bound too) | 0.0 | 0.0 | 0.0 | 0.0 / 0.8 / 7.8 / 33.8 / 82.4 |
| P (bound 80% on B1 and B3) | 0.0 | 0.0 | 0.0 | 0.0 / 2.2 / 13.6 / 42.4 / 88.2 |
| P (point estimates only) | 0.0 | 0.2 | 0.0 | 0.4 / 5.4 / 21.4 / 54.0 / 91.8 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 0.0 | 0.2 | 0.0 | 0.2 / 3.0 / 21.8 / 54.0 / 92.6 |
| P proposed | 0.0 | 0.2 | 0.0 | 0.2 / 4.0 / 17.8 / 49.2 / 91.0 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: DEFL 98.2%, BENCH S 64.6%, BENCH D1 45.6%, YEARS 32.4%, REGIMES 25.8%
- P proposed: BENCH S 22.6%, BENCH D1 19.6%, E3 t(S) 19.6%, SR>B1 P2 6.8%, E2 t(P2) 4.6%

### weekly (no Phase-2 data) | benchmarks low (ETF 0.5, BTC 0.4)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.2 / 1.8 / 10.8 / 48.0 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.0 / 10.8 / 48.2 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.0 / 10.8 / 48.8 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.2 / 12.8 / 54.4 |
| P-single (Bonferroni t on D1, 2 stages) | 0.6 | 0.8 | 0.0 | 0.6 / 4.4 / 21.0 / 48.4 / 84.0 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.2 / 0.6 / 4.2 / 26.0 / 81.0 |
| P-core (t hurdles only) | 1.2 | 2.8 | 0.0 | 1.0 / 9.2 / 31.6 / 65.0 / 94.4 |
| P (bound 90%, B3 bound too) | 0.4 | 1.2 | 0.0 | 0.2 / 4.8 / 22.0 / 52.8 / 90.8 |
| P (bound 80% on B1 and B3) | 0.6 | 2.0 | 0.0 | 0.6 / 5.8 / 25.6 / 58.6 / 92.8 |
| P (point estimates only) | 1.2 | 2.6 | 0.0 | 1.0 / 9.0 / 30.6 / 63.6 / 93.8 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 1.2 | 2.0 | 0.0 | 0.8 / 5.2 / 27.2 / 59.8 / 95.8 |
| P proposed | 0.8 | 2.0 | 0.0 | 0.6 / 7.0 / 28.4 / 61.2 / 93.6 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: N_eff dev 100.0%, DEFL 98.2%, BENCH S 50.4%, YEARS 32.4%, BENCH D1 25.6%
- P proposed: E3 t(S) 19.6%, BENCH S 13.6%, E1 t(D1) 11.6%, BENCH D1 8.6%, REGIMES 4.0%

### weekly (no Phase-2 data) | benchmarks high (ETF 0.8, BTC 0.9)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.6 / 6.4 / 36.6 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.0 / 7.0 / 35.8 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.0 / 7.0 / 36.2 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.2 / 8.0 / 39.0 |
| P-single (Bonferroni t on D1, 2 stages) | 0.2 | 0.6 | 0.0 | 0.2 / 3.2 / 15.2 / 37.4 / 72.2 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.2 / 0.6 / 3.8 / 23.8 / 79.4 |
| P-core (t hurdles only) | 0.8 | 1.6 | 0.0 | 0.8 / 7.2 / 25.4 / 56.2 / 93.0 |
| P (bound 90%, B3 bound too) | 0.2 | 0.4 | 0.0 | 0.2 / 2.2 / 12.0 / 36.4 / 82.8 |
| P (bound 80% on B1 and B3) | 0.2 | 1.0 | 0.0 | 0.2 / 4.4 / 17.6 / 45.0 / 88.4 |
| P (point estimates only) | 0.6 | 1.6 | 0.0 | 0.8 / 7.2 / 25.0 / 54.6 / 92.2 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 0.4 | 1.6 | 0.0 | 0.2 / 3.8 / 20.0 / 50.2 / 92.6 |
| P proposed | 0.2 | 1.2 | 0.0 | 0.2 / 5.0 / 21.2 / 51.2 / 91.4 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: N_eff dev 100.0%, DEFL 98.2%, BENCH S 64.6%, BENCH D1 45.6%, YEARS 32.4%
- P proposed: BENCH S 22.6%, BENCH D1 19.6%, E3 t(S) 19.6%, E1 t(D1) 11.6%, REGIMES 4.6%

### monthly (Phase-2 data from 2012) | benchmarks low (ETF 0.5, BTC 0.4)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.4 | 0.8 | 0.0 | 0.0 / 0.4 / 1.8 / 8.4 / 40.0 |
| P-single (Bonferroni t on D1, 2 stages) | 1.8 | 2.2 | 0.1 | 0.4 / 3.2 / 15.6 / 38.0 / 69.6 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.0 / 0.6 / 3.2 / 16.4 / 62.0 |
| P-core (t hurdles only) | 0.8 | 1.4 | 0.0 | 1.6 / 12.4 / 36.4 / 60.0 / 88.0 |
| P (bound 90%, B3 bound too) | 0.0 | 0.4 | 0.0 | 0.0 / 5.0 / 16.8 / 41.0 / 75.2 |
| P (bound 80% on B1 and B3) | 0.2 | 0.4 | 0.0 | 0.6 / 6.4 / 23.6 / 47.4 / 79.2 |
| P (point estimates only) | 0.6 | 0.8 | 0.0 | 1.2 / 9.6 / 30.8 / 54.8 / 81.2 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 0.4 | 1.0 | 0.0 | 0.4 / 5.6 / 28.0 / 56.0 / 84.4 |
| P proposed | 0.4 | 0.6 | 0.0 | 0.8 / 6.8 / 26.2 / 50.2 / 80.6 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: N_eff D1 100.0%, N_eff S 100.0%, N_eff dev 100.0%, DEFL 97.4%, BENCH S 51.4%
- P proposed: E3 t(S) 31.8%, BENCH S 16.6%, BENCH D1 15.2%, N_eff D1 5.6%, REGIMES 3.8%

### monthly (Phase-2 data from 2012) | benchmarks high (ETF 0.8, BTC 0.9)

| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A2 = A1 + sign-only neighbours (3 cells) | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A3 = A2 + relative drawdown limit | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 / 0.0 |
| A4 = A3 + quarters, no SR floor, N_eff 30 | 0.2 | 0.4 | 0.0 | 0.0 / 0.0 / 0.8 / 4.4 / 29.6 |
| P-single (Bonferroni t on D1, 2 stages) | 0.4 | 0.8 | 0.0 | 0.2 / 1.8 / 10.4 / 29.4 / 59.2 |
| P-DSR (P with DSR>=0.95 at N=m instead of the t hurdle) | 0.0 | 0.0 | 0.0 | 0.0 / 0.4 / 2.8 / 15.6 / 59.8 |
| P-core (t hurdles only) | 0.2 | 0.4 | 0.0 | 0.8 / 7.2 / 28.0 / 52.2 / 83.8 |
| P (bound 90%, B3 bound too) | 0.0 | 0.0 | 0.0 | 0.0 / 1.2 / 8.0 / 25.8 / 66.6 |
| P (bound 80% on B1 and B3) | 0.0 | 0.0 | 0.0 | 0.0 / 2.0 / 13.4 / 36.2 / 70.6 |
| P (point estimates only) | 0.2 | 0.0 | 0.0 | 0.4 / 5.4 / 23.4 / 45.8 / 77.2 |
| P-loose (alpha_P2 = alpha_seal = 0.20, same FWER 5%) | 0.2 | 0.4 | 0.0 | 0.2 / 2.6 / 18.8 / 44.8 / 77.6 |
| P proposed | 0.2 | 0.0 | 0.0 | 0.0 / 4.0 / 17.2 / 41.0 / 74.6 |

Binding gates at S=2 (share of seeds in which the gate failed; the target is evaluated on every gate):

- R2 baseline: N_eff D1 100.0%, N_eff S 100.0%, N_eff dev 100.0%, DEFL 97.4%, BENCH S 62.8%
- P proposed: E3 t(S) 31.8%, BENCH S 25.8%, BENCH D1 24.0%, N_eff D1 5.6%, REGIMES 5.2%

### 12.2 Reading the tables: the chosen design and what binds

* **Chosen: `P proposed`.** Null false-pass (family-wise) 0.0-1.2% against a 5% target; power at planted Sharpe 1 / 2 / 3: weekly type A 6.6 / 61.6 / 93.2 (LOW) and 4.0 / 49.2 / 91.0 (HIGH); weekly type B 7.0 / 61.2 / 93.6 and 5.0 / 51.2 / 91.4; monthly 6.8 / 50.2 / 80.6 and 4.0 / 41.0 / 74.6. At Sharpe 1.5 (the target band 1.5-2): 28.4 / 17.8 / 28.4 / 21.2 / 26.2 / 17.2.
* **Does it meet BOTH requirements?** Null <= 5%: yes with a wide margin. Power at 1.5-2 "meaningfully above chance": yes against chance (5%): 17-28% at 1.5 and 41-62% at 2 (3-12 times chance), **but NOT the conventional 80%.** At net Sharpe 1.5 no design reaches 40% (best: the bare multiplicity design `P-core`, 25-37%); at Sharpe 1 none exceeds 15%. This is a finding for the owner, section 0.
* **Which constraint binds.** (1) *Data length.* The sealed stage alone (79 weeks) fails a true Sharpe-2 candidate 19.6% of the time and monthly 31.8% (18 observations): 1.5 years cannot carry a confirmation. (2) *Benchmark dominance.* The benchmark gates fail 13.6-25.8% (sealed) and 8.6-24.0% (D1) of true Sharpe-2 candidates, more when the blend did well. (3) *The quality gates* (benchmark, stability, robustness) cost about 7 points at Sharpe 2 between `P-core` and `P proposed`, and the choice of alpha split costs nothing (`P-loose`, same 5% bound, is +/-3 points). (4) The old DSR and plateau gates dominated research-v2 (A-ladder). Binding gates of the two key rows at S = 2 are listed under each table above.
* **The 5% bound is conditional, and there is no unused error budget to spend** (addendum A1): it holds on the iid loss-week null family and is exceeded under a persistent calm/stress regime null. Owner decision Q2 (spend the unused budget) is withdrawn: the owner ruled not to spend it, and the null-family run shows that it is not unused.
* **Staged vs single.** `P-single` (Bonferroni 0.00139 on D1, then sealed): 4.4 / 48.4 / 84.0 at Sharpe 1 / 2 / 3 (weekly B, LOW) vs 7.0 / 61.2 / 93.6 for `P proposed` with the same bound.
* **Research-v2 ladder (weekly type A, LOW):** R2 0.0 / 0.6 / 14.4 (Sharpe 1 / 2 / 3); A1 (candidate-level DSR) 0.2 / 10.8 / 48.0; A4 0.4 / 12.8 / 54.4; P 6.6 / 61.6 / 93.2. The DSR with ex-ante variance is itself the next-largest constraint (64% of true Sharpe-2 candidates fail it).

### 12.3 Sensitivity to K (500 seeds; Sharpe 1 / 1.5 / 2 / 3)

## Calibration: K=17 candidates (m=23 hypotheses), plateau 0.9, 500 seeds from 20270000, 400 bootstrap draws, assumed account vol 10%
### weekly (Phase-2 data from 2019-02) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 4.0 |
| P-core (t hurdles only) | 1.0 | 1.4 | 0.1 | 10.6 / 36.8 / 68.4 / 94.0 |
| P proposed | 0.2 | 0.4 | 0.0 | 6.6 / 28.4 / 61.6 / 93.2 |
### weekly (Phase-2 data from 2019-02) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 2.4 |
| P-core (t hurdles only) | 0.4 | 0.4 | 0.0 | 6.4 / 25.2 / 56.6 / 92.6 |
| P proposed | 0.0 | 0.2 | 0.0 | 4.0 / 17.8 / 49.2 / 91.0 |
### weekly (no Phase-2 data) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 1.4 | 3.0 | 0.1 | 12.2 / 34.8 / 66.8 / 94.4 |
| P proposed | 1.2 | 2.2 | 0.1 | 9.2 / 30.8 / 62.2 / 93.6 |
### weekly (no Phase-2 data) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 1.2 | 2.0 | 0.1 | 9.0 / 27.6 / 57.6 / 93.0 |
| P proposed | 0.6 | 1.4 | 0.0 | 5.8 / 22.4 / 51.8 / 91.4 |
### monthly (Phase-2 data from 2012) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 0.6 | 1.2 | 0.0 | 14.6 / 37.2 / 60.8 / 88.0 |
| P proposed | 0.2 | 0.6 | 0.0 | 6.8 / 26.2 / 50.4 / 80.6 |
### monthly (Phase-2 data from 2012) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 0.0 | 0.2 | 0.0 | 7.2 / 28.0 / 52.6 / 83.8 |
| P proposed | 0.0 | 0.0 | 0.0 | 4.0 / 17.2 / 41.0 / 74.6 |

## Calibration: K=60 candidates (m=66 hypotheses), plateau 0.9, 500 seeds from 20270000, 400 bootstrap draws, assumed account vol 10%
### weekly (Phase-2 data from 2019-02) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.8 / 23.6 |
| P-core (t hurdles only) | 0.4 | 1.0 | 0.0 | 8.4 / 34.4 / 67.0 / 94.0 |
| P proposed | 0.2 | 0.6 | 0.0 | 5.8 / 28.2 / 61.2 / 93.2 |
### weekly (Phase-2 data from 2019-02) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 1.2 / 19.0 |
| P-core (t hurdles only) | 0.2 | 0.2 | 0.0 | 5.2 / 24.4 / 55.8 / 92.6 |
| P proposed | 0.0 | 0.2 | 0.0 | 3.6 / 17.8 / 49.0 / 91.0 |
### weekly (no Phase-2 data) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 1.4 | 2.2 | 0.0 | 6.0 / 26.2 / 56.6 / 93.6 |
| P proposed | 1.0 | 1.6 | 0.0 | 4.8 / 24.0 / 54.2 / 93.0 |
### weekly (no Phase-2 data) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 1.0 | 1.6 | 0.0 | 5.2 / 22.2 / 49.6 / 92.2 |
| P proposed | 0.2 | 1.0 | 0.0 | 3.8 / 18.2 / 46.0 / 90.8 |
### monthly (Phase-2 data from 2012) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 0.6 | 1.2 | 0.0 | 9.8 / 32.8 / 57.4 / 88.0 |
| P proposed | 0.4 | 0.6 | 0.0 | 6.2 / 24.6 / 49.6 / 80.6 |
### monthly (Phase-2 data from 2012) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 / 0.0 / 0.0 |
| P-core (t hurdles only) | 0.2 | 0.2 | 0.0 | 5.6 / 25.2 / 50.8 / 83.8 |
| P proposed | 0.2 | 0.0 | 0.0 | 3.6 / 16.4 / 40.4 / 74.6 |

K = 17 (m = 23) versus K = 30 (m = 36) versus K = 60 (m = 66): P proposed weekly type B LOW at Sharpe 2: 62.2 / 61.2 / 54.2; weekly type A: 61.6 / 61.6 / 61.2; monthly LOW: 50.4 / 50.2 / 49.6. Multiplicity is not what limits detection.

### 12.4 Sensitivity to a narrow plateau (0.7 per step; weekly type A, Sharpe 1 / 2 / 3)

## Calibration: K=30 candidates (m=36 hypotheses), plateau 0.7, 500 seeds from 20270000, 400 bootstrap draws, assumed account vol 10%
### weekly (Phase-2 data from 2019-02) | benchmarks low (ETF 0.5, BTC 0.4)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 1.2 / 14.8 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.2 / 7.8 / 28.2 |
| P-core (t hurdles only) | 1.0 | 1.4 | 0.0 | 10.2 / 68.2 / 94.0 |
| P proposed | 0.2 | 0.4 | 0.0 | 6.6 / 61.6 / 93.2 |
### weekly (Phase-2 data from 2019-02) | benchmarks high (ETF 0.8, BTC 0.9)
| design | null FWER % (base) | null FWER % (argmax) | per-cand null % | power S=1 / 2 / 3 |
|---|---|---|---|---|
| R2 baseline | 0.0 | 0.0 | 0.0 | 0.0 / 0.8 / 12.2 |
| A1 R2 + candidate-level DSR (N=m, ex-ante var) | 0.0 | 0.0 | 0.0 | 0.0 / 4.8 / 22.6 |
| P-core (t hurdles only) | 0.4 | 0.4 | 0.0 | 6.4 / 56.6 / 92.6 |
| P proposed | 0.0 | 0.2 | 0.0 | 4.0 / 49.2 / 91.0 |

`P proposed` is unchanged (the design reads only the sign of the neighbours); the research-v2 ladder falls further (A1 at Sharpe 3: 48.0 -> 28.2).

### 12.5 HAC size and hurdles

## HAC size under the null: n=182 periods, 20000 sims; rejection rate of t >= 2.20 and t >= 1.28 (nominal 1.4% and 10%)

| skew | hold H | lag 0 | lag auto | lag max(auto,2H) | lag 2H+2 |
|---|---|---|---|---|---|
| Neg | 1 | 2.6 / 12.4 | 2.9 / 12.8 | 2.9 / 12.8 | 2.9 / 12.8 |
| Neg | 4 | 15.7 / 28.1 | 4.9 / 15.9 | 4.3 / 14.7 | 4.3 / 14.5 |
| Sym | 1 | 1.5 / 10.1 | 1.6 / 10.5 | 1.6 / 10.5 | 1.6 / 10.5 |
| Sym | 4 | 14.0 / 26.3 | 3.4 / 14.2 | 3.0 / 13.1 | 3.0 / 12.9 |
| Pos | 1 | 0.8 / 8.8 | 1.0 / 9.3 | 1.0 / 9.3 | 1.0 / 9.3 |
| Pos | 4 | 12.9 / 25.8 | 2.3 / 13.1 | 1.9 / 11.9 | 1.9 / 11.8 |

## Empirical critical values of the HAC t under the worst-case null (negative skew), lag max(auto, 2H)

| n | H | z at 1.39% | z at 13.9% | z at 10% | normal: 2.20 / 1.08 / 1.28 |
|---|---|---|---|---|---|
| 182 | 1 | 2.60 | 1.18 | 1.42 | |
| 182 | 4 | 2.92 | 1.33 | 1.57 | |
| 142 | 1 | 2.69 | 1.21 | 1.43 | |
| 118 | 1 | 2.72 | 1.23 | 1.46 | |
| 104 | 1 | 2.80 | 1.25 | 1.50 | |
| 79 | 1 | 2.89 | 1.29 | 1.55 | |
| 78 | 1 | 2.90 | 1.29 | 1.55 | |
| 42 | 1 | 3.33 | 1.42 | 1.73 | |
| 24 | 1 | 3.67 | 1.59 | 1.93 | |
| 18 | 1 | 3.91 | 1.76 | 2.12 | |


### 12.6 400 versus 10,000 bootstrap draws

## 400 vs 10,000 bootstrap draws (planted S=2, weekly, 10 seeds)

| seed | paired bound vs B1: 400 | 10,000 | MC p95 MDD: 400 | 10,000 | P verdict 400 | P verdict 10,000 |
|---|---|---|---|---|---|---|
| 20270000 | 0.305 | 0.320 | 0.2478 | 0.2551 | false | false |
| 20270001 | 2.262 | 2.277 | 0.1228 | 0.1181 | true | true |
| 20270002 | 2.195 | 2.236 | 0.1166 | 0.1255 | true | true |
| 20270003 | 2.294 | 2.323 | 0.0777 | 0.0812 | true | true |
| 20270004 | 0.309 | 0.348 | 0.1309 | 0.1331 | false | false |
| 20270005 | 0.651 | 0.624 | 0.1265 | 0.1266 | true | true |
| 20270006 | 0.606 | 0.655 | 0.1093 | 0.1122 | true | true |
| 20270007 | 1.189 | 1.219 | 0.1369 | 0.1329 | true | true |
| 20270008 | 0.558 | 0.582 | 0.1955 | 0.1955 | false | false |
| 20270009 | 0.307 | 0.247 | 0.1126 | 0.1149 | false | false |

verdicts agree on 10/10 seeds; largest |difference| of the annualised Sharpe-difference bound: 0.060

The bound moves by at most 0.06 annualised Sharpe points between 400 and 10,000 draws and no verdict changes on the 10 seeds tested (planted Sharpe 2, weekly type B). Production uses 10,000. The tables above use 400.

### 12.7 Drawdown rule on ETF-like books

See section 7 (same run, 1,000 seeds).

## 13. Reproduction

`cargo build --release -p tq-r2stats` (CARGO_TARGET_DIR=/Users/thakur/Workspace/Trading/shared-target), then `tq-r2stats calibrate-o2 power --seeds 500 --k 30 --bench-csv <dev-side NIFTYBEES.NS.csv>`, `... hac --seeds 20000`, `... dd --seeds 1000`, `... agree`, `... counts`; K and plateau sensitivities `--k 17|60`, `--plateau 0.7`. Seeds 20270000.. Code: `crates/r2stats/src/o2cal/` (tests: 21 unit tests). The first-pass of gen.rs preceded its tests (disclosed); the calendar and statistics modules were written test-first (RED seen, 12 failures).

## 14. Open questions for the owner (genuine decisions only)

1. **Deflation form (departure from D-006 item 3).** Accept the staged candidate-level union bound as the multiplicity gate, with DSR at N = 36 and SPA reported but not gating? (A DSR >= 0.95 gate cuts power at Sharpe 2 from 61.6% to 26.0%.)
2. **Spend the unused error budget?** Withdrawn (owner ruling 2026-11-17: do not spend it; addendum A1 shows the bound is exceeded under a regime null).
3. **Accept that the five-year window cannot see Sharpe 1.** If a net Sharpe of 1 matters, the only fix is more independent data: forward paper (the clean holdout) or paid data. Otherwise the realistic claim is "a net Sharpe of about 2 or more is detectable about half the time".
4. **Benchmark rule:** 80% lower bound on B1, point estimate on B3 and in the sealed slice (proposed), versus a 90% bound on both (power -10 points).
5. **Weekly BankNifty/FinNifty/Midcap contracts discontinued in 2024-11 (recollection):** confirm from the data inventory; they become DIAGNOSTIC for the sealed stage.

## Addendum A1 (2026-11-17, protocol fix after the adversarial review; all changes definitional, made before any strategy result)

The authoritative statement is `docs/prereg-options-v2.md` (sections 1-9, addendum A1 table, Appendices A-C). Summary of what changes in this design:

* **Claim.** The family-wise bound is conditional on a named null family. Measured with the frozen hurdles, K = 30, 1,000 seeds, 10,000 bootstrap draws: <= 3.1% on the design pool and the iid credit nulls; **10.5% (weekly A), 20.7% (weekly B), 4.1% (monthly A), 13.6% (monthly B) under the persistent calm/stress regime null**. No finite hurdle repairs rare-tail nulls (z needed 20-55 when P(no loss event in window) exceeds the stage alpha). A promotion means PROMOTED TO FORWARD PAPER only.
* **Gates added or tightened:** G9 stress exposure and the adverse-regime-evaluable requirement (INCONCLUSIVE-TAIL), G10 absolute floor 6% on fixed capital, G11 volatility band 6-14% (too quiet = INCONCLUSIVE-SIZE), second drawdown leg 3 x volatility, skip cap 15%, quarter gate from 3 evaluable quarters, frozen hurdle table. Cost: 2-12 points of power at Sharpe 2 and 9-12 at Sharpe 3.
* **Type rule** (A/B, signed table): Phase 2 spends no alpha for type B; monthly type B power at Sharpe 2 is 25-33%.
* **Definitions:** returns on fixed capital with no cash credit, realised-cycle series for t and Sharpe, one sealed batch, own cap axis for markers, orphan rule, UNSELECTED state, fixed-notional sizing, Nifty-only weeklies, expiry-day settle rule, calibration B3 rebalanced quarterly, `--max-date` guard, `draw_bench` fails closed.
* **Disclosure:** the HIGH benchmark scenario and the BTC leg of S3 are seen real numbers. Real benchmark stress weeks: D1 8 (one <= -5%), S3 exactly 1.
* **Code defect found and fixed during A1:** the `emp_crit` cache was keyed without the pool, so the first draft of the 'z needed' table repeated the design-pool value; fixed with a killing test. The frozen hurdles and all nullfam/power runs were unaffected (they use the design pool).

The full tables follow (copied from the prereg).

## Appendix A. Frozen hurdle table
Source: `FROZEN_HURDLES` in `crates/r2stats/src/o2cal/design.rs` (identical numbers; a test asserts frozen = max(routine, draft)). z for the HAC t of the mean, lag max(auto, 2H), design-pool null, m = 36. A = type-A S1 alpha 0.1389, B = type-B S1 alpha 0.01389, P = 0.10 (S2 and S3).
| n | H | kind | routine z | MC se | draft z | FROZEN z |
|---|---|---|---|---|---|---|
| 182 | 1 | A | 1.223 | 0.013 | 1.18 | 1.223 |
| 182 | 1 | B | 2.601 | 0.061 | 2.6 | 2.601 |
| 182 | 1 | P | 1.457 | 0.015 | 1.42 | 1.457 |
| 182 | 4 | A | 1.322 | 0.015 | 1.33 | 1.330 |
| 182 | 4 | B | 2.839 | 0.117 | 2.92 | 2.920 |
| 182 | 4 | P | 1.570 | 0.018 | 1.57 | 1.570 |
| 142 | 1 | A | 1.234 | 0.013 | 1.21 | 1.234 |
| 142 | 1 | B | 2.720 | 0.084 | 2.69 | 2.720 |
| 142 | 1 | P | 1.486 | 0.016 | 1.43 | 1.486 |
| 118 | 1 | A | 1.251 | 0.013 | 1.23 | 1.251 |
| 118 | 1 | B | 2.758 | 0.093 | 2.72 | 2.758 |
| 118 | 1 | P | 1.498 | 0.016 | 1.46 | 1.498 |
| 104 | 1 | A | 1.251 | 0.013 | 1.25 | 1.251 |
| 104 | 1 | B | 2.840 | 0.117 | 2.8 | 2.840 |
| 104 | 1 | P | 1.486 | 0.016 | 1.5 | 1.500 |
| 79 | 1 | A | 1.307 | 0.014 | 1.29 | 1.307 |
| 79 | 1 | B | 2.929 | 0.151 | 2.89 | 2.929 |
| 79 | 1 | P | 1.558 | 0.018 | 1.55 | 1.558 |
| 78 | 1 | A | 1.298 | 0.014 | 1.29 | 1.298 |
| 78 | 1 | B | 2.932 | 0.153 | 2.9 | 2.932 |
| 78 | 1 | P | 1.555 | 0.018 | 1.55 | 1.555 |
| 42 | 1 | A | 1.418 | 0.017 | 1.42 | 1.420 |
| 42 | 1 | B | 3.321 | 0.515 | 3.33 | 3.330 |
| 42 | 1 | P | 1.728 | 0.024 | 1.73 | 1.730 |
| 24 | 1 | A | 1.607 | 0.022 | 1.59 | 1.607 |
| 24 | 1 | B | 3.613 | 1.418 | 3.67 | 3.670 |
| 24 | 1 | P | 1.938 | 0.035 | 1.93 | 1.938 |
| 18 | 1 | A | 1.742 | 0.028 | 1.76 | 1.760 |
| 18 | 1 | B | 3.784 | 2.667 | 3.91 | 3.910 |
| 18 | 1 | P | 2.092 | 0.047 | 2.12 | 2.120 |

MC se = approximate standard error of the quantile from 20,000 draws (normal-density approximation, an UNDERESTIMATE for heavy tails). It is large for the rare-tail type-B rows at small n (n = 42, 24, 18); those frozen values are the draft value (a larger, rounded number) wherever it exceeds the routine value, so the noise cannot lower a hurdle. The routine was NOT raised to 100,000 draws (the frozen-table test calls it 30 times in debug); the error is stated instead.

## Appendix C. Calibrated power (upper bounds; constant planted edge; design-pool null; K = 30, m = 36; 10,000 bootstrap draws; seeds 20270000..)
Power = share of seeds in which the planted candidate reaches every gate, at planted net Sharpe 0.5 / 1 / 1.5 / 2 / 3. Null FWER = family-wise base-cell false pass on the DESIGN-POOL null (K = 30). Type A = Phase-2 data exist (S2 spends alpha); type B = no S2 alpha. LOW benchmarks = ETF Sharpe 0.5, BTC 0.4; HIGH = 0.8 / 0.9 (a research-v2 measured value, seen). Upper bounds: constant planted edge, iid design-pool null, no G7/G8, no lumpy edge.

| shape | benchmarks | design | null FWER % | power S = 0.5 / 1 / 1.5 / 2 / 3 |
|---|---|---|---|---|
| weekly (Phase-2 data from 2019-02) | low | P-core (t hurdles only) | 0.6 | 0.8 / 9.4 / 35.4 / 66.2 / 94.6 |
| weekly (Phase-2 data from 2019-02) | low | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 0.2 / 6.6 / 28.8 / 59.6 / 93.4 |
| weekly (Phase-2 data from 2019-02) | low | P proposed | 0.2 | 0.2 / 6.2 / 27.2 / 55.8 / 84.6 |
| weekly (Phase-2 data from 2019-02) | high | P-core (t hurdles only) | 0.4 | 0.4 / 6.4 / 25.6 / 55.0 / 90.2 |
| weekly (Phase-2 data from 2019-02) | high | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 0.2 / 3.0 / 19.6 / 47.4 / 88.2 |
| weekly (Phase-2 data from 2019-02) | high | P proposed | 0.2 | 0.2 / 3.0 / 18.4 / 43.0 / 76.2 |
| weekly (no Phase-2 data) | low | P-core (t hurdles only) | 1.8 | 1.0 / 8.8 / 33.8 / 62.8 / 94.4 |
| weekly (no Phase-2 data) | low | P before A1 (no G9/floor/vol/dd-vol) | 1.0 | 1.0 / 6.6 / 31.4 / 58.4 / 93.4 |
| weekly (no Phase-2 data) | low | P proposed | 1.0 | 1.0 / 6.4 / 29.8 / 55.0 / 85.8 |
| weekly (no Phase-2 data) | high | P-core (t hurdles only) | 1.0 | 1.0 / 7.0 / 28.8 / 55.6 / 90.8 |
| weekly (no Phase-2 data) | high | P before A1 (no G9/floor/vol/dd-vol) | 0.6 | 0.8 / 4.0 / 23.6 / 50.2 / 89.0 |
| weekly (no Phase-2 data) | high | P proposed | 0.6 | 0.8 / 4.0 / 22.2 / 46.0 / 78.2 |
| monthly (Phase-2 data from 2012) | low | P-core (t hurdles only) | 1.0 | 1.8 / 11.2 / 35.8 / 57.0 / 87.2 |
| monthly (Phase-2 data from 2012) | low | P before A1 (no G9/floor/vol/dd-vol) | 0.6 | 0.6 / 7.2 / 26.8 / 48.2 / 80.2 |
| monthly (Phase-2 data from 2012) | low | P proposed | 0.6 | 0.6 / 6.8 / 25.0 / 44.6 / 73.6 |
| monthly (Phase-2 data from 2012) | high | P-core (t hurdles only) | 0.2 | 0.6 / 7.8 / 26.2 / 50.0 / 83.6 |
| monthly (Phase-2 data from 2012) | high | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.0 / 4.2 / 15.8 / 38.2 / 75.2 |
| monthly (Phase-2 data from 2012) | high | P proposed | 0.0 | 0.0 / 4.2 / 14.4 / 33.0 / 62.0 |
| monthly (no Phase-2 data, type B) | low | P-core (t hurdles only) | 0.8 | 0.4 / 4.8 / 17.0 / 39.4 / 83.6 |
| monthly (no Phase-2 data, type B) | low | P before A1 (no G9/floor/vol/dd-vol) | 0.8 | 0.4 / 4.0 / 14.6 / 36.0 / 78.2 |
| monthly (no Phase-2 data, type B) | low | P proposed | 0.8 | 0.4 / 4.0 / 13.6 / 32.8 / 71.6 |
| monthly (no Phase-2 data, type B) | high | P-core (t hurdles only) | 0.8 | 0.4 / 4.6 / 15.0 / 36.4 / 80.4 |
| monthly (no Phase-2 data, type B) | high | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 0.2 / 3.6 / 11.6 / 30.0 / 73.8 |
| monthly (no Phase-2 data, type B) | high | P proposed | 0.2 | 0.2 / 3.6 / 10.6 / 25.4 / 60.6 |

Headline (A1 design, 'P proposed'), power at net Sharpe 1 / 2 / 3, LOW then HIGH benchmarks:

| shape | type | LOW | HIGH |
|---|---|---|---|
| weekly (Phase-2 data from 2019-02) | A | 6.2 / 55.8 / 84.6 | 3.0 / 43.0 / 76.2 |
| weekly (no Phase-2 data) | B | 6.4 / 55.0 / 85.8 | 4.0 / 46.0 / 78.2 |
| monthly (Phase-2 data from 2012) | A | 6.8 / 44.6 / 73.6 | 4.2 / 33.0 / 62.0 |
| monthly (no Phase-2 data, type B) | B | 4.0 / 32.8 / 71.6 | 3.6 / 25.4 / 60.6 |

The added gates cost power: 'P before A1' to 'P proposed' loses about 2-12 points at Sharpe 2 and about 9-12 points at Sharpe 3 (weekly A LOW 59.6 -> 55.8 and 93.4 -> 84.6; monthly A HIGH 38.2 -> 33.0 and 75.2 -> 62.0), mostly from the adverse-regime-evaluable requirement (fails 8-18% of true Sharpe-2 candidates in these bootstrap worlds) and the sealed floor (about 5%). **A net Sharpe of about 1 is undetectable: 3-7% power.** Monthly type B (no S2) at Sharpe 2 is only 33% (LOW) / 25% (HIGH); monthly candidates are therefore reported with this calibrated power in the registration table. Power numbers come from 500 seeds (binomial se up to 2.2 points).

Binding gates at planted Sharpe 2 for the A1 design (share of seeds in which the gate failed):
* weekly (Phase-2 data from 2019-02) | benchmarks low (ETF 0.5, BTC 0.4): E3 t(S) 19.0%, BENCH S 14.6%, BENCH D1 11.2%, G9 adverse 7.8%, FLOOR S 4.8%
* weekly (Phase-2 data from 2019-02) | benchmarks high (ETF 0.8, BTC 0.9): BENCH S 23.8%, BENCH D1 22.2%, E3 t(S) 19.0%, G9 adverse 13.4%, SR>B1 P2 8.6%
* weekly (no Phase-2 data) | benchmarks low (ETF 0.5, BTC 0.4): E3 t(S) 19.0%, BENCH S 14.6%, BENCH D1 11.2%, E1 t(D1) 11.2%, G9 adverse 7.8%
* weekly (no Phase-2 data) | benchmarks high (ETF 0.8, BTC 0.9): BENCH S 23.8%, BENCH D1 22.2%, E3 t(S) 19.0%, G9 adverse 13.4%, E1 t(D1) 11.2%
* monthly (Phase-2 data from 2012) | benchmarks low (ETF 0.5, BTC 0.4): E3 t(S) 33.6%, BENCH S 16.8%, BENCH D1 13.6%, G9 adverse 9.2%, FLOOR S 5.2%
* monthly (Phase-2 data from 2012) | benchmarks high (ETF 0.8, BTC 0.9): E3 t(S) 33.6%, BENCH D1 28.0%, BENCH S 27.0%, G9 adverse 18.0%, FLOOR S 5.2%
* monthly (no Phase-2 data, type B) | benchmarks low (ETF 0.5, BTC 0.4): E1 t(D1) 34.0%, E3 t(S) 33.6%, BENCH S 16.8%, BENCH D1 13.6%, G9 adverse 9.2%
* monthly (no Phase-2 data, type B) | benchmarks high (ETF 0.8, BTC 0.9): E1 t(D1) 34.2%, E3 t(S) 33.6%, BENCH D1 28.0%, BENCH S 27.0%, G9 adverse 18.0%

## Real benchmark stress weeks per window (weekly returns of the supplied price series)

| window | weeks | weeks <= -3% | weeks <= -5% | worst week | G9 rule met |
|---|---|---|---|---|---|
| Phase 2 | 513 | 22 | 8 | -12.59% | yes |
| D1 | 182 | 8 | 1 | -5.43% | yes |
| D1 train | 104 | 6 | 1 | -5.43% | yes |
| D1 validation | 78 | 2 | 0 | -4.32% | NO |
| S3 sealed | 79 | 1 | 1 | -5.25% | yes |

**Caveat (plain):** G9 is met in the real benchmark windows, but S3 holds exactly ONE benchmark week at or below -3% (the same week, -5.25%, is also the only severe one). A short-convexity candidate's sealed verdict therefore rests on a single stress week; G9 is not non-evaluable, and no threshold was changed to compensate. The D1 validation half alone (2 weeks <= -3%) would not meet the rule, but G9 is defined on D1 as a whole.

## Appendix B. Null-family calibration (frozen hurdles; 10,000 bootstrap draws)
Null-family calibration: K=30 (m=36), 1000 seeds from 20270000, 10000 bootstrap draws; hurdles FROZEN from the design pool, never recomputed from the null under test

Columns: per-candidate pass %, family-wise pass % (any of K candidates), INCONCLUSIVE-TAIL % and INCONCLUSIVE-SIZE % of candidates (no failed gate). 'core' = staged t hurdles only, 'before A1' = the design without G9/floor/vol/dd-vol, 'A1' = the frozen design.

### weekly (Phase-2 data from 2019-02) (type A)

| null | design | per-cand % | family-wise % | tail-inconclusive % | size-inconclusive % |
|---|---|---|---|---|---|
| design pool (control) | P-core (t hurdles only) | 0.0 | 0.7 | 0.0 | 0.0 |
| design pool (control) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.3 | 0.0 | 0.0 |
| design pool (control) | P proposed | 0.0 | 0.3 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P-core (t hurdles only) | 0.1 | 1.9 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.8 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P proposed | 0.0 | 0.6 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P-core (t hurdles only) | 0.1 | 3.0 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P before A1 (no G9/floor/vol/dd-vol) | 0.1 | 1.5 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P proposed | 0.0 | 1.3 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P-core (t hurdles only) | 7.6 | 15.5 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P before A1 (no G9/floor/vol/dd-vol) | 5.5 | 12.5 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P proposed | 4.3 | 10.5 | 0.8 | 0.2 |
| losses in benchmark stress | P-core (t hurdles only) | 0.0 | 0.1 | 0.0 | 0.0 |
| losses in benchmark stress | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.1 | 0.0 | 0.0 |
| losses in benchmark stress | P proposed | 0.0 | 0.1 | 0.0 | 0.0 |

### weekly (no Phase-2 data) (type B)

| null | design | per-cand % | family-wise % | tail-inconclusive % | size-inconclusive % |
|---|---|---|---|---|---|
| design pool (control) | P-core (t hurdles only) | 0.0 | 1.2 | 0.0 | 0.0 |
| design pool (control) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.8 | 0.0 | 0.0 |
| design pool (control) | P proposed | 0.0 | 0.8 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P-core (t hurdles only) | 0.1 | 3.8 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P before A1 (no G9/floor/vol/dd-vol) | 0.1 | 3.2 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P proposed | 0.1 | 3.1 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P-core (t hurdles only) | 0.3 | 7.3 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 5.2 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P proposed | 0.1 | 2.3 | 0.0 | 0.1 |
| regime calm/stress 5%/40% | P-core (t hurdles only) | 13.4 | 24.7 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P before A1 (no G9/floor/vol/dd-vol) | 12.5 | 22.9 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P proposed | 10.5 | 20.7 | 1.1 | 0.6 |
| losses in benchmark stress | P-core (t hurdles only) | 0.1 | 0.4 | 0.0 | 0.0 |
| losses in benchmark stress | P before A1 (no G9/floor/vol/dd-vol) | 0.1 | 0.3 | 0.0 | 0.0 |
| losses in benchmark stress | P proposed | 0.1 | 0.1 | 0.0 | 0.0 |

### monthly (Phase-2 data from 2012) (type A)

| null | design | per-cand % | family-wise % | tail-inconclusive % | size-inconclusive % |
|---|---|---|---|---|---|
| design pool (control) | P-core (t hurdles only) | 0.0 | 1.0 | 0.0 | 0.0 |
| design pool (control) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.5 | 0.0 | 0.0 |
| design pool (control) | P proposed | 0.0 | 0.4 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P-core (t hurdles only) | 0.1 | 1.4 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 1.0 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P proposed | 0.0 | 0.3 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P-core (t hurdles only) | 0.4 | 11.4 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 6.9 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P proposed | 0.0 | 0.0 | 0.0 | 0.1 |
| regime calm/stress 5%/40% | P-core (t hurdles only) | 1.2 | 9.0 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P before A1 (no G9/floor/vol/dd-vol) | 0.8 | 5.8 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P proposed | 0.4 | 4.1 | 0.0 | 0.2 |
| losses in benchmark stress | P-core (t hurdles only) | 0.0 | 0.0 | 0.0 | 0.0 |
| losses in benchmark stress | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.0 | 0.0 | 0.0 |
| losses in benchmark stress | P proposed | 0.0 | 0.0 | 0.0 | 0.0 |

### monthly (no Phase-2 data, type B) (type B)

| null | design | per-cand % | family-wise % | tail-inconclusive % | size-inconclusive % |
|---|---|---|---|---|---|
| design pool (control) | P-core (t hurdles only) | 0.0 | 0.6 | 0.0 | 0.0 |
| design pool (control) | P before A1 (no G9/floor/vol/dd-vol) | 0.0 | 0.6 | 0.0 | 0.0 |
| design pool (control) | P proposed | 0.0 | 0.5 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P-core (t hurdles only) | 0.3 | 7.7 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P before A1 (no G9/floor/vol/dd-vol) | 0.2 | 6.3 | 0.0 | 0.0 |
| credit 1:19 (loss 5%) | P proposed | 0.0 | 0.7 | 0.0 | 0.2 |
| credit 1:49 (loss 2%) | P-core (t hurdles only) | 2.6 | 47.2 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P before A1 (no G9/floor/vol/dd-vol) | 2.3 | 42.5 | 0.0 | 0.0 |
| credit 1:49 (loss 2%) | P proposed | 0.0 | 0.0 | 0.3 | 1.6 |
| regime calm/stress 5%/40% | P-core (t hurdles only) | 3.0 | 17.3 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P before A1 (no G9/floor/vol/dd-vol) | 2.5 | 16.1 | 0.0 | 0.0 |
| regime calm/stress 5%/40% | P proposed | 1.5 | 13.6 | 0.1 | 0.9 |
| losses in benchmark stress | P-core (t hurdles only) | 0.8 | 1.8 | 0.0 | 0.0 |
| losses in benchmark stress | P before A1 (no G9/floor/vol/dd-vol) | 0.7 | 1.6 | 0.0 | 0.0 |
| losses in benchmark stress | P proposed | 0.1 | 0.5 | 0.2 | 0.4 |



## Hurdle that would restore the nominal tail under an iid credit null (t-only, one stage)

P(no loss event in the window) = (1-p)^n; when it exceeds the stage alpha the sample variance of a quiet window is tiny and t is huge, so no finite hurdle restores the nominal size: that is the INCONCLUSIVE-TAIL route (G9), not a hurdle.

| null | window | n | stage alpha | frozen z (design pool) | z needed | P(no event) | finite hurdle exists |
|---|---|---|---|---|---|---|---|
| credit 1:19 (loss 5%) | D1 weekly (A) | 182 | 0.1389 | 1.22 | 1.31 | 0.0% | yes |
| credit 1:19 (loss 5%) | D1 weekly (B) | 182 | 0.0139 | 2.60 | 3.53 | 0.0% | yes |
| credit 1:19 (loss 5%) | D1 weekly (P) | 182 | 0.1000 | 1.46 | 1.73 | 0.0% | yes |
| credit 1:19 (loss 5%) | S3 weekly (A) | 79 | 0.1389 | 1.31 | 1.43 | 1.7% | yes |
| credit 1:19 (loss 5%) | S3 weekly (B) | 79 | 0.0139 | 2.93 | 32.51 | 1.7% | NO |
| credit 1:19 (loss 5%) | S3 weekly (P) | 79 | 0.1000 | 1.56 | 1.54 | 1.7% | yes |
| credit 1:19 (loss 5%) | D1 monthly (A) | 42 | 0.1389 | 1.42 | 1.23 | 11.6% | yes |
| credit 1:19 (loss 5%) | D1 monthly (B) | 42 | 0.0139 | 3.32 | 34.21 | 11.6% | NO |
| credit 1:19 (loss 5%) | D1 monthly (P) | 42 | 0.1000 | 1.73 | 22.92 | 11.6% | NO |
| credit 1:19 (loss 5%) | S3 monthly (A) | 18 | 0.1389 | 1.74 | 20.84 | 39.7% | NO |
| credit 1:19 (loss 5%) | S3 monthly (B) | 18 | 0.0139 | 3.78 | 31.18 | 39.7% | NO |
| credit 1:19 (loss 5%) | S3 monthly (P) | 18 | 0.1000 | 2.09 | 22.50 | 39.7% | NO |
| credit 1:19 (loss 5%) | Phase 2 weekly (A) | 142 | 0.1389 | 1.23 | 1.48 | 0.1% | yes |
| credit 1:19 (loss 5%) | Phase 2 weekly (B) | 142 | 0.0139 | 2.72 | 3.66 | 0.1% | yes |
| credit 1:19 (loss 5%) | Phase 2 weekly (P) | 142 | 0.1000 | 1.49 | 1.61 | 0.1% | yes |
| credit 1:49 (loss 2%) | D1 weekly (A) | 182 | 0.1389 | 1.22 | 2.31 | 2.5% | yes |
| credit 1:49 (loss 2%) | D1 weekly (B) | 182 | 0.0139 | 2.60 | 52.41 | 2.5% | NO |
| credit 1:49 (loss 2%) | D1 weekly (P) | 182 | 0.1000 | 1.46 | 2.43 | 2.5% | yes |
| credit 1:49 (loss 2%) | S3 weekly (A) | 79 | 0.1389 | 1.31 | 33.46 | 20.3% | NO |
| credit 1:49 (loss 2%) | S3 weekly (B) | 79 | 0.0139 | 2.93 | 43.24 | 20.3% | NO |
| credit 1:49 (loss 2%) | S3 weekly (P) | 79 | 0.1000 | 1.56 | 35.46 | 20.3% | NO |
| credit 1:49 (loss 2%) | D1 monthly (A) | 42 | 0.1389 | 1.42 | 29.00 | 42.8% | NO |
| credit 1:49 (loss 2%) | D1 monthly (B) | 42 | 0.0139 | 3.32 | 37.92 | 42.8% | NO |
| credit 1:49 (loss 2%) | D1 monthly (P) | 42 | 0.1000 | 1.73 | 30.55 | 42.8% | NO |
| credit 1:49 (loss 2%) | S3 monthly (A) | 18 | 0.1389 | 1.74 | 22.98 | 69.5% | NO |
| credit 1:49 (loss 2%) | S3 monthly (B) | 18 | 0.0139 | 3.78 | 32.64 | 69.5% | NO |
| credit 1:49 (loss 2%) | S3 monthly (P) | 18 | 0.1000 | 2.09 | 24.47 | 69.5% | NO |
| credit 1:49 (loss 2%) | Phase 2 weekly (A) | 142 | 0.1389 | 1.23 | 1.72 | 5.7% | yes |
| credit 1:49 (loss 2%) | Phase 2 weekly (B) | 142 | 0.0139 | 2.72 | 50.22 | 5.7% | NO |
| credit 1:49 (loss 2%) | Phase 2 weekly (P) | 142 | 0.1000 | 1.49 | 1.76 | 5.7% | yes |

**Reading (honest).** Hurdles are frozen from the design pool; these are the rates when the truth is another zero-edge null. Family-wise = any of K = 30 independent-ish candidates passes; per-candidate rates are the number to multiply by the number of real candidates (16-17).
* The 5% family-wise target is MET in these runs for the design pool and the iid credit nulls (0.0-3.1% with the A1 design; the t-only core reaches 3.0-47.2%, with 47.2% for monthly type B under credit 1:49 and 11.4% for monthly type A) and for the benchmark-linked null (0.0-0.5%).
* It is NOT met under the persistent calm/stress regime null: A1 family-wise 10.5% (weekly A), 20.7% (weekly B), 4.1% (monthly A), 13.6% (monthly B). The regime null is not repaired by any finite hurdle, and G9 does not catch it, because the regime's stress is independent of the benchmark's stress in this generator. **So the 5% claim is conditional: it does not hold if a candidate's true net returns are a persistent calm/stress mixture with a positive calm mean and a rare, long negative stress mean.** That is a realistic description of a premium seller with zero net expectancy, and the protocol cannot rule it out with this data. The residual false-promotion risk under that null is reported here, not hidden; the only mitigations are forward paper and the owner's reading of a WEAK/borderline candidate.
* Where the iid rare tail would give a huge t (P(no loss event in window) above the stage alpha), no finite hurdle exists (z needed 20-55 in the table above). The A1 route is INCONCLUSIVE (G9, the volatility band) instead of a hurdle: it removes the monthly 2%-tail false passes (47.2% -> 0.0%, 11.4% -> 0.0%) and shows up as tail- or size-inconclusive candidates (at most 1.6% of candidates).
* Per-candidate tail/size-inconclusive shares are small in these synthetic worlds because the bootstrap benchmark almost always contains stress weeks. In the REAL benchmark the D1 window has 8 stress weeks and one week of -5.4%, and S3 has exactly ONE stress week (-5.25%): G9 is satisfied there, with no margin.


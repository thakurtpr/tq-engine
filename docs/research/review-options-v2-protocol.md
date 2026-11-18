# Adversarial review of the options-v2 protocol (staged union-bound design, DRAFT prereg)

Reviewer: independent agent a59a8485 (did not write the protocol). Date 2026-11-17. Branch `hardening/o2review` off `hardening/integration` (1751763). Paper and backtest only; docs and read-only analysis; no strategy backtest, no ledger/tag/caps touched, no sealed data, no key. Nothing pushed.

Reviewed: `hardening/STATUS.md` (hard rules, the options-v2 directive), `docs/ai/DECISIONS.md` D-003/D-005/D-006, `docs/research-v2-final-report.md`, `docs/options-v2-protocol-design.md` (550 lines), `docs/prereg-options-v2.md` (DRAFT), `docs/options-v2-harness-dryrun-spec.md`, `docs/options-v2-capability-and-catalogue.md`, `crates/r2stats/src/o2cal/*` (stats, gen, design, run, cli, calendar, 21 tests), `docs/validation-protocol.md` v3, `docs/prereg-options-v1.md`, `docs/result-options-v1-dev.md`, `docs/data-inventory-2026-11.md`, `research/trials.csv`.

What I ran: `cargo test -p tq-r2stats --lib o2cal` (21 passed, baseline); a single-edit mutation sample on a scratch copy of the repo (section 6: 28 clean mutants plus 38 indicative; the 60-mutant target was not reached because of lock contention and one spoiled stretch); my own probe program (`rustc -O`, no dependencies, own xoshiro256++ RNG, own HAC t, own seeds, written from the documented parameters, NOT from the repo code) in the scratchpad: `/private/tmp/claude-501/-Users-thakur-Workspace-Trading/6f39fe22-10ec-4c71-9269-df8de83872d5/scratchpad/probe/probe.rs` (throwaway, not committed). Probe sub-commands: `size`, `chain`, `fwer`, `sel`, `power`, `crit`, `neff`, `tax`. Cargo always with `CARGO_TARGET_DIR=/Users/thakur/Workspace/Trading/shared-target`; no `target/` in the worktree; disk stayed above 10 GB free.

## 0. Verdict in one paragraph

**FIX THEN FREEZE.** The arithmetic of the union bound is right and the structure is a real improvement over research-v2 (candidate-level count, one a-priori cell, sign-only neighbours, benchmark-relative drawdown, fail-closed series). But the headline guarantee, "family-wise false-GO <= 5% by construction", is **not established** for the return shapes option-selling books actually have, and three definitions are open that make the gates either trivially passable or silently censored. (1) The size-corrected hurdles and the null controls come from the same synthetic pool, so the calibration is circular; under rare-tail or persistent-regime zero-edge nulls the per-candidate chain false-pass is 2x to 80x the 0.139% target and **no finite hurdle can repair it** when P(no tail event in the window) exceeds the stage alpha. (2) The Phase-2 stage (S2) is declared "development data only" by D-003/D-006 yet is spent as an independent alpha stage, is not independent for the v1-lineage candidates (selection inflates its effective alpha 1.5x to 5x), and the prereg does not even carry the rule that demotes those candidates to type B. (3) The return series is undefined with respect to idle cash and the sizing rule is not pinned, so a near-cash book can pass the Sharpe/t gates and premium-dependent lot rounding censors the sample. None of this needs a redesign: it needs about ten edits to the prereg and design, one extra gate (stress exposure), one batch rule for the sealed slice and a re-run of the calibration with a null family. The minimal list is in section 8.

## 1. Findings ranked

| # | Rank | Finding (short) | Where |
|---|---|---|---|
| F1 | CRITICAL | Size correction and null controls share one synthetic pool (circular). Rare-tail and regime nulls: chain false-pass 2x to 80x target; no finite hurdle exists when P(no tail event in window) > alpha. "5% by construction" is not established | design 3.1-3.2, 12; `stats.rs::emp_crit`, `gen.rs`, `design.rs::evaluate` |
| F2 | CRITICAL | S2 (Phase 2) spends alpha as an independent replication although D-003/D-006 call 2012-2021 development-only; v1-lineage and informed candidates are selected on it (alpha inflated 1.5x to 5x); the type-B demotion exists only in design 11.1, not in the prereg; monthly type-B power was never calibrated | design 3, 11.1; prereg 2-3 |
| F3 | CRITICAL | The series is undefined for idle cash and interest; if a cash yield is credited, a zero-edge near-cash book passes every t/Sharpe gate. No excess-over-cash or minimum-return gate either | design 11.3, prereg 5 |
| F4 | HIGH | Shared sealed slice: per-candidate touches plus candidates added by addendum after earlier touches leak S3 information; "re-run after touch" contradicts "verdict final" | design 11.1(3); prereg 1-2 |
| F5 | HIGH | Ledger arithmetic: sealed-touch markers and dated re-runs share an 8-cell reserve; stage windows may or may not count as cells; orphaned markers have no rule; a third catalogue state (RUNNABLE but unselected) is undefined | prereg 1; spec items 9, 12, 14, 15 |
| F6 | HIGH | Sizing is not pinned. Lot rounding against a 1% risk budget made v1 skip 36-53% of straddle cycles; skips are premium-dependent (a hidden IV filter that favours long-vol candidates); the MDD gate depends on the unpinned sizing | design 11.3, prereg 5; v1 result V5/V6 |
| F7 | HIGH | Power claims are reproducible but misapplied: my numbers match as upper bounds; omitted gates; constant planted edge; the monthly rows assume Phase 2 counts (type A) although the monthly candidates are exactly the v1-lineage (type B); the "unused error budget" is mostly benchmark gates, not t-gates | design 12; `run.rs`, `design.rs` |
| F8 | MEDIUM | DSR/SPA leaving the gate: acceptable only after F1/F2; conflicts in letter with D-006 item 3 and STATUS rule 4 unless wording is fixed | design 3.3, 14 Q1 |
| F9 | MEDIUM | MTM on EOD settles (stale/carried marks) can smooth the series (Sharpe and t overstated); the gate series should be realised cycle P&L | design 5, 11 |
| F10 | MEDIUM | Tax is nearly Sharpe-neutral (measured), the benchmark's tax convention differs, window-boundary tax state undefined | design 11.3, 6 |
| F11 | MEDIUM | Stability/regime gates exclude the adverse regime when it is "not evaluable"; no stress-exposure requirement; S2/S3 have no stability gate | design 8, prereg G5 |
| F12 | MEDIUM | Benchmark rule: the B3 point estimate is a coin flip near equality, S3 has no bound; BTC returns in the sealed window are already seen | design 6 |
| F13 | MEDIUM | 2024-10/11 SEBI changes, Nifty expiry weekday change, lot changes: structural breaks inside D1/S3 break the H = 1 assumption and the "independent replication" reading | design 5, 10 |
| F14 | MEDIUM | Drawdown clamp: floor 25% and cap 40% are sizing-dependent and the cap admits a 40% drawdown book | design 7 |
| F15 | MEDIUM | Contamination bookkeeping: v1 probes inside D1/S3, scans over S3 by the data agent, HIGH-benchmark values taken from research-v2 results, `--bench-csv` has no date guard | design header, 6 |
| F16 | MEDIUM/LOW | Code quality: spec-code drift (quarters >= 3 vs >= 5), gates not simulated, B1 series cycles, runtime-simulated hurdles are not frozen, weak tests (mutation survivors, section 6) | `o2cal` |
| F17 | LOW | N_eff / DE_time, HAC lag rule, 400 vs 10,000 draws: sound, with the caveats in section 5 | design 5, 12.6 |

## 2. CRITICAL findings

### F1. The size correction is circular and does not survive realistic null shapes

**Evidence.**
* `stats.rs::emp_crit` computes the (1 - alpha) quantile of the HAC t from iid draws of `pool(Skew::Neg)` (6% crash weeks at -3.2 sd). `gen.rs` draws the null candidates from the same pool (as four mixed components, which attenuates the skew: the skew weight is 0.55, so about -1.1 instead of -1.8). `design.rs::evaluate` passes `pool(Skew::Neg)` into `crit`. So the null control tests the hurdle against the very distribution the hurdle was built from; the measured 0.0-1.2% family-wise rate is then guaranteed to be near or below nominal.
* I replicated the pool from the documented parameters and the hurdles agree (mine vs doc: 1.20/1.18, 2.60/2.60, 1.44/1.43, 1.54/1.55, 3.27/3.33, 1.48/1.46, 2.09/2.12). Under that same pool the per-candidate chain false-pass is 0.121% (target 0.139%). So the arithmetic is right **for that null**.
* Now change only the zero-edge null. All generators are zero-mean in expectation with unit-scale shocks; win/loss payoffs get N(0, 0.25) noise so no window is degenerate; 40,000 sims; the doc's hurdles applied unchanged. Rejection rate of the doc hurdle, % (nominal in the header):

| generator | skew | kurt | S1 A wk (13.9) | S1 B wk (1.39) | S2 wk (10) | S3 wk (10) | S1 B mo (1.39) | S2 mo (10) | S3 mo (10) |
|---|---|---|---|---|---|---|---|---|---|
| design pool (control) | -1.8 | 8 | 13.7 | 1.31 | 10.5 | 9.5 | 1.31 | 10.5 | 9.6 |
| normal | 0 | 3 | 12.4 | 0.63 | 8.7 | 7.0 | 0.27 | 8.0 | 4.4 |
| defined-risk credit 1:4 (loss in 20% of weeks) | -1.5 | 3 | 14.0 | 1.27 | 10.1 | 9.4 | 1.42 | 9.7 | 9.8 |
| credit 1:9 (loss 10%) | -2.6 | 8 | 14.9 | 1.98 | 11.4 | 10.9 | 3.92 | 11.3 | 14.8 |
| credit 1:19 (loss 5%) | -4.1 | 18 | 17.8 | 3.71 | 14.4 | 10.8 | 11.8 | 14.6 | 39.3 |
| credit 1:49 (loss 2%) | -6.8 | 48 | 21.6 | 10.6 | 22.1 | 20.3 | 42.7 | 11.3 | 69.8 |
| GARCH(.10,.85) + skewed pool | -2.2 | 18 | 16.0 | 1.66 | 12.3 | 11.0 | 1.29 | 12.2 | 9.7 |
| GARCH(.15,.83) + skewed pool | -3.9 | 106 | 17.6 | 1.80 | 13.4 | 12.0 | 1.12 | 13.3 | 9.5 |
| 2-state regime, loss rate 5% / 40%, mean durations 78w / 26w | -2.1 | 5 | 37.2 | 22.3 | 35.8 | 39.8 | 19.0 | 37.0 | 25.1 |
| 2-state regime 3% / 50%, durations 150w / 20w | -3.0 | 10 | 46.2 | 32.7 | 46.4 | 48.7 | 19.1 | 48.3 | 46.3 |

* Per-candidate t-only chain false-pass on ONE contiguous timeline (Phase 2 = 142 weeks, D1 = 182, S3 = 79; target 0.139%), 100,000 sims, and the implied family-wise rate for 17 and 30 independent null candidates:

| null | A chain % | product of marginals % | B chain % | FWER 17 A % | FWER 30 A % | FWER 17 B % |
|---|---|---|---|---|---|---|
| design pool | 0.121 | 0.142 | 0.154 | 2.04 | 3.57 | 2.59 |
| normal | 0.072 | 0.076 | 0.041 | 1.22 | 2.14 | 0.69 |
| credit 1:4 | 0.144 | 0.125 | 0.106 | 2.42 | 4.23 | 1.79 |
| credit 1:9 | 0.178 | 0.186 | 0.212 | 2.98 | 5.20 | 3.54 |
| credit 1:19 | 0.287 | 0.286 | 0.422 | 4.77 | 8.26 | 6.94 |
| credit 1:49 | 1.011 | 0.978 | 2.174 | 15.86 | 26.28 | 31.18 |
| GARCH(.10,.85) | 0.202 | 0.224 | 0.205 | 3.38 | 5.89 | 3.43 |
| GARCH(.15,.83) | 0.280 | 0.287 | 0.240 | 4.65 | 8.07 | 4.00 |
| regime 5/40 | 6.06 | 5.27 | 9.79 | 65.4 | 84.7 | 82.7 |
| regime 3/50 | 11.60 | 10.41 | 16.98 | 87.7 | 97.5 | 95.8 |

  (The FWER columns assume independent candidates; they are an upper-style illustration.) With the design's own candidate-correlation structure (common .09, family .20, own .55 + cell .16; 17 candidates, 4,000 sims, probe `fwer`) the family-wise t-only rate is 2.4% (type A, base cell) and 3.2% (A, argmax of 3 cells) on the design pool, 3.1% / 4.5% for credit 1:19, 5.2% / 7.5% for credit 1:49, and 43% to 68% for the regime nulls.
* **No finite hurdle repairs the rare-tail nulls.** The z that restores the nominal tail (probe `crit`): credit 1:9 at S3 monthly n = 18 needs z = 17.3; credit 1:19 at S1 B monthly n = 42 needs 35; credit 1:49 at S1 B weekly n = 182 needs 55 and at S3 weekly n = 79 needs 37. The reason is arithmetic: when P(no loss event in the window) = (1 - p)^n exceeds the stage alpha, the sample variance in those windows is tiny and t is enormous. A weekly seller with a 2%-per-week tail has P(no tail in 79 weeks) = 20%, above the 10% stage alpha; with a 0.5% weekly tail it is 67%. D1 is 3.5 years and S3 is 1.5 years, so such windows exist.
* The N_eff >= 30 / >= 10 gate does not address this: its stated basis (design 10) is "P(no loss cycle in N cycles) <= 5% needs N >= 29" for a 1-in-10 loss rate; for a 1-in-50 tail the same N has a 55% chance of seeing no tail at all.
* Independence across stages is fine under an iid null (joint 0.121% vs product 0.142%). Under shared regimes the joint exceeds the product (6.06 vs 5.27; type B 9.79) and, more importantly, each marginal is already 3x to 5x its nominal alpha. Volatility clustering alone (GARCH) costs 1.5x to 2x; the persistent mean-regime is what destroys the bound. A regime null is not exotic for a premium seller whose net expectancy is about zero: long calm stretches with positive means and rare multi-month stress with large negative means is the shape of the thing being tested.
* Real-shape ingredients named in the task: negative skew and fat tails (credit rows), volatility clustering (GARCH rows), expiry-day gaps (tail events: credit rows), regime breaks such as 2024-10/11 SEBI changes (regime rows). Only the first is in the generator.

**Failure scenario.** A weekly short-premium candidate with true zero net expectancy and a 2%-per-week tail event. The probability it passes S1 (type B, z = 2.60) and S3 (z = 1.55) is 2.2% per candidate (design target 0.139%, 15.6x); 17 such candidates give 31% family-wise (independent) instead of 2.4%. Under a persistent calm/stress regime the per-candidate rate is 6% to 17%.

**Fix.**
1. Do not claim "5% by construction". State the bound as conditional on a stated null family and report the t-only family-wise rate on that family.
2. Re-run the calibration with a null FAMILY: design pool, credit 1:9, 1:19, 1:49, GARCH(.15,.83), and at least one persistent-regime null, with hurdles frozen from the design pool (not from the null under test). Report per-candidate and family-wise rates for each. This is a few lines in `o2cal/gen.rs` and `run.rs`; my probe numbers are the expected result.
3. Because a hurdle cannot fix a "no tail in the window" sample, add a **stress-exposure gate (G9)** for any structure that is net short convexity (credit spreads, condors, butterflies, covered calls, cash-secured puts, calendars, ratio, expiry-day sellers): the stage window must contain at least `N_s` stress periods of the BENCHMARK (suggested: >= 3 weeks with a benchmark weekly return <= -3% or a weekly India VIX rise >= 25%, or >= 1 week <= -5%) in D1 and at least 1 in S3; otherwise the verdict is INCONCLUSIVE-TAIL (not GO, not KILL). Report the worst period and its loss in units of the average credit. The thresholds are suggestions; set them from the o2data benchmark inventory only, never from candidate returns, before the freeze.
4. Add a **stress replay** to the reported set and to G4: the candidate's structure re-priced under the Phase-2 worst benchmark week (2020-03 type) with a fixed vol shock; its loss must fit inside the drawdown limit. It is a labelled stress, not a backtest of a real week.
5. Freeze the hurdle table numerically (F16) and add a rule that hurdles are never recomputed from a modified pool.

### F2. Phase 2 (S2) is not an independent stage, contradicts D-003/D-006 and is mis-specified in the prereg

**Evidence.**
* D-003 (Accepted): "Already-seen data is NOT holdout; it is development data only." D-006 item 2: Phase 2 is "DEVELOPMENT data only (already seen by options-v1)". The design spends 10% alpha on it as an independent replication stage (type A: 0.139 x 0.10 x 0.10 = 0.00139). The design text (11.1) knows the problem and says any parameter variant of an options-v1 structure "is flagged and its Phase-2 evidence is reported but does not count for S2". **`docs/prereg-options-v2.md` sections 2-3 contain no such rule**: type A needs only "contracts for >= 24 months". As written, C5 (v1-lineage monthly spread/condor), C6 (long straddle, "v1 base"), C1/C2 (weekly spread and condor whose offset axis is justified by "v1's 3% monthly offset is about 1 sigma") are type A with S2 counted.
* If the flagged candidates are processed as type A with S2 not counting, alpha is 0.139 x 0.10 = 1.39% per candidate, 10x over the design share; if they are demoted to type B the hurdle becomes 2.60 (weekly) or 3.33 (monthly n = 42) and the power falls. The calibration has no monthly type-B shape (`Shape::monthly()` is type A with Phase 2 from 2012). The candidates that are monthly AND v1-lineage (C5, C9, C10 per the catalogue) are the ones missing from the table. My independent monthly type-B power (t-only; planted net Sharpe 0.5 / 1 / 1.5 / 2 / 3): 1.8 / 8.7 / 24.0 / 46.5 / 83.2 on the design pool, versus 5.1 / 24.6 / 46.8 / 65.8 / 88.7 for type A. A flagged monthly candidate therefore has 35% (Sharpe 1) to 71% (Sharpe 2) of the type-A t-only power, before the benchmark and stability gates.
* **Selection effect on S2** (probe `sel`; null candidate, variant chosen as the best of k by its Phase-2 HAC t, hurdle z = 1.43, n = 142). v1 left a result for every one of its 30 cells and the v2 base cells are chosen with that table, the T+2 shift result and the 3% / 2% parameters known:

| k variants screened | rho between variants | P(pass S2) | x nominal 10% | type-A chain % (0.139 x P x 0.10) |
|---|---|---|---|---|
| 1 | any | 8-9% | 0.8-0.9 | 0.11-0.12 |
| 3 | 0.50 | 19.2% | 1.9 | 0.27 |
| 6 | 0.50 | 27.8% | 2.8 | 0.39 |
| 10 | 0.50 | 35.8% | 3.6 | 0.50 |
| 30 | 0.50 | 53.4% | 5.3 | 0.74 |
| 6 | 0.84 | 17.8% | 1.8 | 0.25 |
| 30 | 0.84 | 27.6% | 2.8 | 0.38 |
| 30 | 0.00 | 92.6% | 9.3 | 1.29 |

  The within-variant correlation of v1's cells is high (the generator's own 0.84) and across the six v1 variants lower; the realistic range is 1.5x to 5x. With 17 candidates all informed at 3x to 5x, the t-only family-wise rate of 2.4% on the design pool becomes up to 7-12%, before any shape problem.
* The informed set is wider than the six v1 variants: every candidate built on v1's machinery or printed results (C1, C2, C5, C6, C10 directly; C3, C4, C13, C16 as condor/butterfly/spread cousins) and, honestly, every candidate designed by agents who know the 2012-2021 Nifty path (2013 taper, 2015-16, 2018, 2020). The last form of contamination cannot be bounded; it is the reason D-003 says development only.

**Failure scenario.** C5 monthly credit spread: its offset axis was fixed knowing the v1 V1/V3 results on exactly the same 118 monthly cycles; it passes S2 at t >= 1.46 in about 28% of null worlds instead of 10%, then S1 (z = 1.42 monthly, 14% of nulls) and S3 (z = 2.12, 10%): 0.28 x 0.139 x 0.10 = 0.39% vs 0.139%, before the shape problem of F1, and with a power figure computed for a design it is not in.

**Candidates that cannot use S2 honestly:** C1, C2, C5, C6, C10 (direct v1 lineage), C3, C4, C13, C16 (same family and machinery), and any candidate whose base parameters were screened on any Phase-2 return. Candidates that can, if registered blind with a written rule: C7 (calendar), C8 (broken-wing), C9 (covered call), C11 (trend overlay), C12 (delta-hedged), C14/C15 (events), C17, with the general-knowledge caveat above.

**Fix.**
1. Prereg section 3: add the type rule verbatim: a candidate is **type B** if its structure x underlying class x expiry type matches, or is a parameter, entry or exit variant of, any options-v1 variant V1-V6, or if its base parameters were chosen or screened using any Phase-2 return, or if its design used the printed v1 result tables. The classification table C1..C17 is part of the freeze and is signed before the first run. Where there is doubt the candidate is type B.
2. Type A only for structures with no v1 counterpart and a base parameter vector written from a stated rule (for example volatility-unit offsets) before any Phase-2 return is computed. For everyone else S2 is run and reported but spends no alpha.
3. Add the calibration shape `monthly_new` (monthly, no Phase 2) and report it. Re-state the power of the v1-lineage monthly candidates from it.
4. State in the design that Phase 2 is development data (D-003) and that S2 is a pre-registered blind replication, not a holdout.

### F3. The series is undefined for idle cash; a near-cash book can pass every statistical gate

**Evidence.** The design and prereg say "NET-of-cost-and-tax weekly after-tax returns" of an account of Rs 1 crore with positions "sized by a capped fraction of margin". Nothing says whether margin collateral or idle cash earns interest, what the return denominator is, or that Sharpe is on excess over cash. Gates: t of the mean > 0 (rf = 0), Sharpe above the ETF and the 50/50 blend (rf = 0, scale-free), drawdown, stability. v1 used rf = 6% for its like-for-like line and rf = 0 for the gate; v2 prereg G3 uses rf = 0 on both sides.
* If any cash yield (about 6.5% a year in India) is credited to the account, a book with a zero-edge options overlay at 3% volatility has mean 6.5% and sd 3%: Sharpe about 2.2 and a t of about 4 over 3.5 years. It beats an ETF Sharpe of 0.5-0.8, passes the drawdown clamp, is "stable" every quarter and its neighbours are all positive. The staged t-test then passes every stage with probability near 1, for any structure.
* If no cash yield is credited, the book is a sub-scale overlay on Rs 1 crore and Sharpe is scale-free: a Sharpe-0.8 book at 1% volatility beats the 0.5 ETF while earning 0.8% a year. "Sharpe above benchmark" and "mean > 0" are met, the owner directive's "positive after-cost after-tax OOS return" is technically met, and the book is economically worthless. There is no minimum return, no CAGR gate and no excess-over-cash gate.

**Fix.** Pin in prereg 3 and design 11.3: (a) returns are options P&L (plus futures hedge P&L) net of costs and tax divided by the FIXED initial account capital; no interest, margin yield or collateral income is credited to the numerator; (b) an **absolute floor**: annualised after-tax net return on the account >= a stated cash rate (a constant fixed before freeze, for example 6.0%) over D1 and over S3, or alternatively the book is scaled to a pre-registered target volatility (for example 10% annual) and the gates run on the scaled series with costs scaled; (c) the o2cost/o2eng builders get a test that credits 6.5% to idle cash and shows the verdict does not change.

## 3. HIGH findings

### F4. The sealed slice is shared; per-candidate touches plus later candidates leak it

**Evidence.** D-006 says the sealed slice is "touched at most once per candidate". The design implements that as a candidate-by-candidate read (11.1 (1)-(3)) with the verdict "final whatever it is". But S3 is one fixed 18-month window common to all candidates. Each touch reveals the window's character (for example that weekly short premium did or did not work in 2025-26). The prereg allows new candidates by dated addendum "before the first run" up to the 30 slots (13 unfilled), so a candidate can be designed after earlier candidates' S3 results are known; its S3 alpha of 10% is then not a fresh test. The same section allows "a re-run with a fixed bug ... as a new counted candidate slot": that re-run's S3 is a second read of the same window for the same strategy, which contradicts "verdict final" and "single touch". research-v2 handled the class problem with one sealed BATCH whose outcomes stay sealed until the batch ends; options-v2 spec item 12 drops that for per-candidate touches to avoid the structural block.

**Failure scenario.** C1 (weekly spread) dies at S3; the team adds C1' (delta-hedged variant) by addendum, informed by the S3 character; C1' passes S3 at a conditional probability far above 10%. The union bound is spent honestly on paper, the evidence is not independent.

**Fix.** Keep "once per candidate" (D-006) but make it a **single sealed batch**: all candidates register before the batch; after S1 (and S2 where applicable) and G8 pass for the last registered candidate, ONE touch writes all markers, runs all S3 evaluations and publishes all outcomes together. No candidate may be registered after the batch (it can only go to forward paper). A re-run after the batch has no S3. Remove the "new counted slot re-run after touch" sentence.

### F5. Ledger and cap arithmetic can make S3 unreachable or leave the counter ambiguous

**Evidence.** Prereg 1: reserve 8 cells for "dated re-runs, sealed-touch markers, each counted"; cap 60. With 17 candidates, if more than 8 reach the sealed stage (S1 passes about 14% of nulls and far more real candidates), later touches are refused by the cap, and every re-run needs a reserve cell too. Design 11.1: the marker is written BEFORE the read; a crash or refusal leaves an unclosed marker. Spec item 15 starts the orphan list "empty: no orphan may be declared before a result exists", and V10-style completeness (spec item 9) demands closed sweeps for RUNNABLE candidates. The spec never says whether the P2 and S3 evaluations of an already-registered base cell add ledger cells (the harness counts cells by candidate x parameter vector, but design 11.1 reads like per-stage markers). The catalogue has 100 RUNNABLE cells of which 17 candidates are selected; the other RUNNABLE rows are neither NOT-RUN nor candidates, a third state the harness does not know (research-v2 B4 was exactly this kind of state mismatch).

**Fix.** (a) A cell is (candidate, parameter vector); stage windows consume no cell; only the sealed batch consumes ONE marker per candidate, taken from its own cap axis (`o2-sealed-touch`, cap = K_cap) and not from the 8-cell reserve. (b) Orphan rule: a marker or sweep with zero stored results may be declared orphan by a dated addendum written before the candidate's next evaluation; a marker with results may never be. (c) Introduce the catalogue state UNSELECTED (RUNNABLE, no candidate, no cell, never a failure), excluded from completeness and from every set. (d) The dry run must include: 9 candidates reaching the sealed stage at the design cap; a crash between marker and read; an UNSELECTED catalogue row.

### F6. Sizing is not pinned and lot rounding censors the sample

**Evidence.** v1 at Rs 25 lakh with a 1% risk budget skipped 43 of 118 Nifty straddle cycles and 63 of 118 BankNifty straddle cycles ("risk budget below one lot": V5, V6 in `docs/result-options-v1-dev.md`). The v2 prereg says only "size by a capped fraction of margin" with Rs 1 crore and Rs 25 lakh as a stress; the fraction, the cap and the skip rule are not numeric, and v1's gate "skip rate <= 15%" was not carried over. Arithmetic at Rs 1 crore (spot and lot sizes are ASSUMPTIONS from recollection, flagged unverified in the catalogue; I use Nifty 24,000 x 75 and BankNifty 52,000 x 30): a weekly bull put spread of width 1% of spot has max loss about Rs 18,000 per lot, so a 1% risk budget (Rs 1,00,000) buys 5 lots (20% quantisation) and a 0.5% budget buys 2; a monthly ATM Nifty straddle costs about 3.2% of spot, about Rs 57,600 per lot, so the 1% budget buys one lot (100% quantisation) and the 0.5% budget buys zero (skip). At Rs 25 lakh every structure is 0 or 1 lot and monthly straddles are never traded.
* The skip is premium-dependent: a long-premium structure is skipped when IV (and premium) is high and traded when it is cheap. That is an unregistered IV filter that gives long-premium candidates (C6, C11, C12, C14, C15) a favourable sample by construction (buys only cheap vol). For short-premium defined-risk structures the max loss is set by width, not IV, so skipping is rarer.
* The drawdown clamp and the cost drag (fixed Rs 20 per order) depend on size; the design calibrated at 10% volatility, a number the prereg does not pin.

**Fix.** Pre-register a **fixed-lot** rule: every cycle trades the same NOTIONAL rounded to whole lots (not the same lot count: Nifty lots went 50 to 25 on 2024-04-26, 25 to 75 on 2024-11-20 and 75 to 65 on 2025-10-28, all inside D1/S3; set per candidate from the structure's max loss per unit at the START of the window, never from premium) with the return denominator fixed at the account capital; no cycle is skipped for budget; skips only for the liquidity filter, counted and capped at 15% (otherwise INCONCLUSIVE-SAMPLE as in v1); the volatility of the book is reported and a target band (for example 6-14% annual on D1) is required so that the drawdown clamp and the cost drag are comparable across candidates. Run Rs 25 lakh as a reported sensitivity with the same lot rule, not as a different sizing.

### F7. Power claims: reproducible, but misapplied

**Evidence (own simulation; different code path and seeds).** t-only chain (S1, S2, S3 hurdles of the design table, planted net Sharpe 0.5 / 1 / 1.5 / 2 / 3 per year, 20,000 sims, no benchmark, stability, drawdown or N_eff gate):

| shape (design pool) | 0.5 | 1 | 1.5 | 2 | 3 |
|---|---|---|---|---|---|
| weekly type A (mine) | 3.2 | 19.4 | 49.6 | 76.2 | 96.3 |
| weekly type B (mine) | 1.9 | 12.8 | 37.4 | 68.0 | 96.0 |
| monthly type A (mine) | 5.1 | 24.6 | 46.8 | 65.8 | 88.7 |
| monthly type B, S2 void (mine; not in the design) | 1.8 | 8.7 | 24.0 | 46.5 | 83.2 |
| doc P-core weekly A LOW | 0.6 | 10.2 | 36.2 | 68.2 | 94.0 |
| doc P-core weekly B LOW | 1.0 | 9.2 | 31.6 | 65.0 | 94.4 |
| doc P-core monthly A LOW | 1.6 | 12.4 | 36.4 | 60.0 | 88.0 |
| doc P proposed weekly A LOW | 0.6 | 6.6 | 28.4 | 61.6 | 93.2 |

My t-only numbers are above the doc's P-core by 8 to 9 points at Sharpe 1-2 because P-core still carries the B1/B3 point-estimate gates; that gap is the expected size of those gates. With a credit 1:9 return shape the t-only power is within 1 to 3 points of the pool shape for weekly (my sim: 19.4 / 73.6 at Sharpe 1 / 2, weekly A). So the doc's power table is a fair upper bound for a CONSTANT planted edge on the iid pool, and the headline "Sharpe 1 is not detectable by any design" is correct.
* Misapplications: (a) the planted edge is uniform in time; real edges are lumpy and regime-dependent (design 12.0 says so), so real power is lower; (b) the calibration omits gates that are in the prereg: G5 drop-best-1%, G7 cost stress, G8, the S2/S3 regime tests, and the harness design effect of N_eff (`env.shape.de` is 1.0): reported power is an upper bound by those gates; (c) the monthly rows are type A (F2); (d) the "unused error budget" of Q2 is mostly the benchmark gates: on the design pool my t-only family-wise rate for 17 candidates is 2.4% (A base) and 3.2% (argmax), about 3.6% for 30 (independent-candidate approximation), against the doc's P-core 0.2-1.2%. The benchmark gates depend on the assumed benchmark Sharpe (LOW 0.5, HIGH 0.8) and are nearly free in a window where the ETF Sharpe is near zero, so the real margin to 5% is small and spending it (Q2) would be reckless.
* Argmax / best-of-candidates: the doc forbids argmax. If it happened, the t-only family-wise rate rises from 2.4% to 3.2% (design pool) and from 3.1% to 4.5% (credit 1:19). The base-cell-only rule must therefore be enforced by the harness (`Pick::Base` only in the evaluation path), not by convention.
* 400 vs 10,000 draws: the bootstrap lower bound at 400 draws has Monte Carlo SE about sqrt(0.2 x 0.8 / 400) / f(q) = 0.057 annualised Sharpe (f = N(0, 0.8) density at the 20% quantile), matching the observed maximum difference 0.060; there is no bias. The agreement check uses only 10 seeds at planted S = 2, where the verdict is mostly decided by other gates; it does not test near-boundary cases. LOW: use 10,000 in any calibration row that reports a benchmark-bound gate, or at least 2,000.

**Fix.** Re-run section 12 with: the omitted gates implemented; `monthly_new`; a null family (F1); a lumpy-edge variant (for example 70% of the Sharpe earned in 50% of the periods); 10,000 draws for the benchmark gates. State every power as "upper bound under a constant planted edge".

## 4. MEDIUM findings

### F8. DSR and SPA leaving the gate (the orchestrator's ruling): risk assessment

The ruling is defensible on its merits (DSR >= 0.95 at N = 36 needs z about 3.8, stricter than a 5% family-wise target; my probes agree with the design's P-DSR row direction) and a false DEV-GO only buys forward paper observation, not capital. Three things must hold and none is written today:
1. The staged bound must actually be a bound (F1, F2). The DSR/SPA were the only second line that does not depend on the synthetic null; removing them while the first line is circular raises the real false-GO risk.
2. Wording: D-006 item 3 asks for deflated Sharpe on the true total N plus White/SPA, and STATUS hard rule 4 reserves "profitable" for unamended v3 gates (DSR >= 0.95, PBO <= 0.20). The prereg already says "PROMOTED TO FORWARD PAPER, never passed"; add that "profitable" or "passed" is never used for an options-v2 verdict and that DSR (N = 36 and N_all), SPA p and PBO-n/a are printed on every verdict line.
3. One cheap, shape-independent hedge: flag DSR at N = 36 below 0.50 as "WEAK" and require the owner to confirm promotion of a WEAK candidate (not a gate).

Risk: MEDIUM after F1/F2 are fixed, HIGH if they are not. SPA must be defined as NOT-EVALUATED-and-non-blocking when a registered candidate has no returns.

### F9. Marks and the gate series

`docs/result-options-v1-dev.md`: V4 had 41 carried marks and 5 unfillable exit legs; EOD settles of illiquid strikes are stale or model-derived. A series built from daily MTM of overlapping positions is smoothed: Sharpe and t are overstated; autocorrelation is partly absorbed by DE_time/HAC but not the level. Design 5 keeps weekly MTM for drawdown and uses it for the weekly series. **Fix:** the Sharpe/t/benchmark series is the REALISED cycle P&L (entry fill to exit fill, flat-to-flat, with the period-allocation rule written down); MTM is used for drawdown only; the count of carried marks and unfillable legs is a gate input (carried marks above 10% of leg-days = INCONCLUSIVE-QUALITY).

### F10. Tax

Measured (probe `tax`: fixed capital, 52-week FY, 182 weeks, design pool, provisional tax 31.2% on max(0, YTD - brought-forward loss)): mean after-tax Sharpe / pre-tax Sharpe = 0.93 / 0.88 / 0.80 (planted 2 / 1 / 0.5); mean HAC t 3.74 vs 3.96 (S = 2), 1.79 vs 2.02 (S = 1). So the provisional-tax series is nearly Sharpe-neutral: tax barely moves any scale-free gate and "net of tax" is not a meaningful filter. Second, the provision falls with YTD profit, so the state shares intra-year losses, which dampens the candidate's drawdown and volatility; the benchmark is taxed "as if liquidated" (design 6, 11.3), a different convention, which favours the candidate in G3 and G4. Third, stage windows start and end mid-financial-year (04-30/05-01, 10-31): the YTD figure and the carry-forward at the cut are undefined (one continuous account path sliced by stage makes S3 depend on D1 losses; a restart makes S3 an account that never existed). **Fix:** state one convention: a single continuous account path 2021-11 to 2026-10 with one carry-forward, sliced for statistics; the benchmark uses the same provisional business-income convention for option-like legs and capital gains for the ETF, both written; report the pre-tax Sharpe next to the after-tax one. The ruin guard (fail closed at <= -100%) is sound and tested; add a test that a profitable year followed by a large loss cannot produce an after-tax return below the pre-tax return minus one provision.

### F11. Stability and regime gates

G5 applies to S1 only. Regime cells with fewer than 20 weekly periods (5 monthly) are "not evaluable" and ignored, and fewer than 2 evaluable cells make the gate "not applicable": the adverse regime (down-trend and high volatility) is exactly the cell most likely to be short, so a short-premium candidate can dodge its worst regime. G5 does not exist in S2/S3. The quarter gate counts positive quarters (60%): a candidate that earns 12 small quarters and loses one is "stable". **Fix:** G9 stress exposure (F1); require the adverse-regime cell to be evaluable in D1 (at least 10 weekly periods) or the verdict is INCONCLUSIVE-TAIL; report the loss in the worst cell.

### F12. Benchmark rule

* B1: 80% lower bound > 0 in S1; B3: point estimate only. A point estimate on B3 is a coin flip near equality: with SE of the annualised Sharpe difference about sqrt((2 - 2 rho) / T) = 0.63 for T = 3.5 years and rho = 0.3, a candidate 0.5 Sharpe BELOW B3 passes the point test with probability 21% and one 1.0 below 6%; in S3 (T = 1.5, SE about 0.97) 30% and 15%. It cannot cause a false pass by itself (all gates are ANDed with the t hurdles), so it adds no false-pass risk, but it deletes true candidates near the benchmark at about 50%: label it a "screen", not a "gate".
* B3 (BTC leg) over the sealed window comes from the crypto development side, which research-v2 has already evaluated through 2026-10 (research-v2 final report: BTC weekly Sharpe 0.83 over its window; the design's HIGH scenario takes 0.9 from it). So the benchmark side of S3 is seen; the design header claims "no real-data performance statistic exists in this document" while the HIGH scenario is a measured research-v2 number. LOW, but disclose it.
* B3 is rebalanced weekly in the calibration and quarterly in production (design 6): a mismatch the calibration cannot absorb; make them equal.

### F13. Structural breaks and the "independent replication" reading

D1 and S3 straddle the 2024-10/11 changes (lot sizes, expiry rules, upfront premium, expiry-day margin) and the Nifty weekly expiry weekday move (recollection, unverified: Thursday to Tuesday in 2025-09). A trade held Wednesday to Tuesday spans the Friday cut, so weekly period returns are MA(1) for every trade and H is 2, not 1; the HAC lag and N_eff use H = 1 in S3 after the shift. A candidate that worked under the older contract rules (Phase 2, early D1) and not after is KILLED by S3 although the S1/S2 evidence is real; a candidate whose edge exists only after the changes cannot pass S2 by construction. These are not false-pass risks but S2 and S3 are not replications of the same contract. **Fix:** record H and the expiry weekday per stage in the manifest; compute lag and N_eff with the stage's own H; label S3 results with the contract-rule regime; do not read a failed S3 of a candidate that passed S1/S2 as evidence against the pre-change edge, and say so in the report.

### F14. Drawdown clamp

`clamp(1.5 x MDD_B1, 25%, 40%)`. Does it let a bad book through? Yes, two ways: the 25% floor allows MDD up to 25% when the benchmark window was calm (benchmark MDD 10% gives a limit of 25%, 2.5x the benchmark), and the 40% cap allows 40% in a Phase-2-type window (benchmark 38%): the level the design itself calls "margin distress". Does it fail a good one? A Sharpe-1.5 book sized at 25% volatility exceeds 25% MDD in a calm window and fails while a Sharpe-0.5 book at 8% volatility passes: the gate measures sizing, not quality (F6). The design's own table (ETF-like books pass 86% / 97% / 58%) shows it is a loose screen. **Fix:** tie it to the volatility target of F6, add a second leg `MDD <= 3 x annualised volatility of the book`, keep the cap, and report the loss as a multiple of Rs 1 crore and of the average credit.

### F15. Contamination bookkeeping (D1 and S3 versus options-v1 and the data inventory)

* options-v1 prereg (disclosure section) and the data inventory: v1 downloaded nine sessions (2019-03-01..07 and 2025-10-27..31) and probed single dates 2001-06-04, 2005-01-03, 2012-01-02, 2016-10-03, **2023-01-02, 2024-07-05, 2024-08-01, 2026-01-05**; one printed row each for 2012, 2019, **2023, 2024-08, 2026-01**. Of these, 2023-01-02, 2024-07-05 and 2024-08-01 lie inside D1, and 2025-10-27..31 and 2026-01-05 inside S3. Only row counts, field names, zero-volume shares and one printed row were examined; no return was computed. Contamination by v1 or the inventory is therefore format-level and negligible, but the five burned sessions and these probes must be listed in the freeze as "format knowledge only". The data inventory states no F&O or `ind` file after 2021-12-31 was fetched or read; index levels past 2021 came from Yahoo (levels only).
* The Phase-1 window was declared consumed by D-006; the S3 re-seal is self-imposed and holds only as long as nothing reads the files. The o2data task (not merged at this commit) downloads 2021-11 to latest and runs `scripts/o2_dq_scan.py` over bad prints, gaps and non-positive values: any liquidity share, strike count, zero-volume share or price-level statistic over files dated on or after 2025-05-01 is exposure to S3, and the design requires "no sealed-dated file read". **Fix:** the data manifest states per file whether it is D1 or S3; the DQ scan runs on D1 only until the sealed batch, or a dated addendum written before the freeze declares exactly which S3 statistics it read (row counts and schema only).
* Benchmarks: NIFTYBEES for 2022-2026 is not on the development side (it ends 2021-12-31). It must come from o2data and be split by window at load time (G8 forbids sealed-dated content in development): the D1 drawdown clamp needs B1 over D1 only and S3 needs it over S3 only; one 2021-2026 file read in development violates G8.
* `o2cal --bench-csv` takes any path; `weekly_pool_from_csv` has no date cut. The "development side only" statement is by convention; enforce `--max-date 2021-12-31` in code.
* D1 is not a clean window either: the data are free and public and the agents designing the candidates know the 2021-2025 Nifty path. D-006 already treats it as development; the design should stop describing D1 as independent of the designer.

## 5. Definitions: remaining checks

* **N_eff = N_raw / max(DE_harness, DE_time) and the minimums.** Sound: on iid series DE_time has mean 1.08 (n = 42), 1.09 (n = 18), 1.07 (n = 79), 1.05 (n = 182); P(N_eff < 30) for monthly D1 (n = 42) is 6.6% under iid, P(N_eff < 10) for S3 monthly (n = 18) is 2.1%, weekly 0%. So the gate costs about 7 power points for monthly D1 (the design reports 5.6%). It cannot be passed by luck (the floor at 1 means negative autocorrelation never helps). Two caveats: DE_time uses lags 1-4, so a ladder held more than 5 periods is under-counted; and N_eff counts periods, not tail exposures (F1).
* **HAC lag rule** `max(floor(4 (n/100)^(2/9)), 2H)`: standard; n = 18 gives lag 2 and n = 182 gives 4. The test `auto_lag(182) = 4` pins it; the exponent mutant survived (section 6) because the tested n values do not distinguish 2/9 from 1/9; harmless for n >= 42, different for n <= 24 (the sealed monthly stage).
* **Drawdown, benchmark, stability, tax, series rules:** F9-F14. **Robustness (sign-only neighbours):** a single lucky cell cannot pass, because the base cell must pass S1-S3 itself and both neighbours must be positive. But the gate is a weak statistical filter by construction: neighbour cells share 84% of the variance with the base, so given a base that passed S1 (z >= 1.18, mean about +1.6 sd) each neighbour is positive with probability about 99%; the gate removes about 1-3% of passers (the design measures 0-2 points). Its value is bookkeeping (neighbours are counted), not statistical.
* **Account size and lot granularity:** F6.
* **Candidates that fail stage 1: counted?** The bound is per candidate chain and m is fixed in advance, so failing S1 spends nothing further. Unfilled slots (13 of 30) and the 6 v1 hypotheses are never tested but reserve 19 of 36 shares (53%) of the alpha budget: conservative and safe, and makes the 17 real candidates' alpha_1 smaller than needed (the K = 17 sensitivity in the design shows no power change because the budget lives in the S1 hurdle, already as low as 1.18).

## 6. Code quality (`crates/r2stats/src/o2cal`)

* Baseline: 21 tests, all pass (31 s, dominated by the rayon planted-power test).
* `gen.rs` and `design.rs` were written before their tests (disclosed in design 13). The tests that exist for them are thin: `planted_power_rises_with_sharpe_and_zero_edge_rarely_passes` runs only `Cfg::core` on `weekly_new` with a 60-seed null and a threshold of 6; `the_staged_alphas_follow_the_fwer_budget` checks the nominal z values although `size_corrected` replaces them at run time with `emp_crit`; no test covers `quarters_ok`, `regimes_ok`, `robust_ok`, `dd_ok`, `bench_ok`, `years_ok`, `Env::has_phase2` boundaries, `Pick::Argmax`, the `full` flag, or the planted/null mode split of `one_seed`.
* Spec-code drift: `share_positive` uses `MIN_QUARTERS.min(3)` (3 evaluable quarters) where the spec says >= 5; the spec's G5 "drop best 1%", G7 cost stress and G8 integrity are not simulated; the N_eff harness design effect is the constant `env.shape.de = 1.0`.
* Hidden leakage into the generator: the only real-data input is the weekly shape of the development NIFTYBEES (bootstrap block 4) with the addendum-4 repair; no sealed path is opened by the code. Two defects: `--bench-csv` has no date guard (F15), and `draw_bench` does `idx.iter().cycle().take(n)` on a bootstrap sequence of the pool's own length. With a real pool shorter than the 774-week timeline (about 520 weekly returns for 2012-2021) the benchmark series REPEATS the first weeks, so D1 and S3 benchmark returns equal Phase-2 ones exactly and the stage benchmark windows are not independent (weight 0.09 in the candidate shocks: small). With the fallback pool (2,000 draws) there is no cycle. I could not tell which pool the published tables used; add a test that the benchmark series has no period-n repeat.
* Determinism and seeds: `SplitMix64` seeded through `cell_seed`, rayon `par_iter` over seeds with an order-preserving collect, `emp_crit` cached under a mutex with a fixed seed: results are deterministic. Seeds for different (n, hold) in `emp_crit` overlap (`s + 1_000_000 hold + 7 n`), so hurdles for different n share draws: harmless. Hurdles are re-simulated at run time from code constants (`CRASH_P`, `POOL_LEN`, seeds): a code change silently changes a gate; freeze the numeric table in the prereg (F1.5).
* A degenerate series (zero variance) gets t = 0 (`mean_t`, tested): conservative, but it means a window with no loss and identical wins fails rather than passes; irrelevant for real data, and the reason my generators add noise.

### Mutation sample

Method: single-edit mutants on a scratch copy of the repo (never the worktree), each run with `cargo test -p tq-r2stats --lib o2cal` (fast tests first, then the full 31 s suite for survivors); 99 mutants were prepared (stats.rs 40, design.rs 48, gen/run/calendar 11), every pattern verified to match exactly once.

**Honest limit.** The shared target directory was under heavy contention from other agents' `cargo test --workspace` runs (load average above 20; single mutants took 4 minutes), and one stretch of the run was spoiled because two copies of my mutation driver ran at the same time (an old `pkill` pattern missed the process), so their file edits overlapped. I discarded that stretch as unreliable. **28 mutants were run cleanly (one driver at a time): 17 killed, 11 survived (61% kill rate). The target of 60 clean mutants was NOT reached.** A further 38 mutants from the overlapped stretch are listed as indicative only: kills there may be spurious and survivors may be false (mutant 39, `ALPHA_P2` 0.10 to 0.50, "survived" there although `the_staged_alphas_follow_the_fwer_budget` asserts `z_dev_a` = 1.08 and must kill it, which proves the stretch is not trustworthy). Total attempted: 66 distinct mutants. The reviewer's recommendation is to re-run all 99 once the machine is quiet; the harness is `scratchpad/mut/mutate.py`.

Clean results (authoritative):

| # | file | mutant | result | assessment |
|---|---|---|---|---|
| 0,1,3,4 | stats | auto_lag base/constant, lag_rule 2H to H, max to min | killed | `the_auto_lag_and_the_hold_rule` |
| 2 | stats | auto_lag exponent 2/9 to 1/9 | SURVIVED | near-equivalent at the tested n (42, 182); differs for n <= 24 (sealed monthly) |
| 5 | stats | Newey-West Bartlett weight denominator lag+1 to lag | SURVIVED | genuine gap: HAC variance weights untested (only lag 0 and a qualitative AR(1) check) |
| 6 | stats | NW autocovariance weight 2 to 1 | SURVIVED | genuine gap, same test |
| 7-12 | stats | gamma divisor, mean_t guards/sqrt//n, Sharpe annualisation | killed | |
| 13 | stats | paired_lb minimum n 8 to 2 | SURVIVED | genuine gap (small-sample guard) |
| 14-17 | stats | paired_lb quantile/point/diff, alpha_dev cap | killed | |
| 18 | stats | alpha_dev `m.max(1)` to `m` | SURVIVED | equivalent for m >= 1 |
| 40,43,44 | design | ALPHA_SEAL to 0.5; type A/B alpha ignoring a later stage | killed | only by the nominal-z test |
| 41 | design | P_NEFF_DEV 30 to 3 | SURVIVED | genuine gap: the N_eff gate value is never tested |
| 42 | design | P_NEFF_SEAL 10 to 1 | SURVIVED | genuine gap |
| 45 | design | benchmark lower-bound level 0.8 to 0 | SURVIVED | genuine gap: BENCH gate level untested |
| 46 | design | B3 bound switched on | SURVIVED | genuine gap |
| 47 | design | drawdown multiplier 1.5 to 15 | SURVIVED | genuine gap: the drawdown clamp parameters are untested (cap still applies) |
| 48 | design | drawdown cap 40% to 400% | SURVIVED | genuine gap: the 40% cap, the number the design defends, is not pinned by any test |

Indicative only (overlapped stretch, 38 mutants: 19 "killed", 19 "survived"): the survivors concentrate on `de_time` lags/weights/factor, `mc_dd_profit` (unused by the proposed design), `emp_crit` simulations count / MA averaging / hold-aware lag (the hurdle routine is never tested directly), regime/quarter/robustness switches, drawdown floor, N_eff D1 gate removal, size-correction off, type-A switches. Structure confirms the pattern: reading `tests_a.rs` and `tests_b.rs`, no test exercises `evaluate`'s individual gates (`bench_ok`, `dd_ok`, `quarters_ok`, `regimes_ok`, `robust_ok`, `n_eff`, `emp_crit`), so the gate logic that the freeze hash will pin is essentially untested; the only integration test (`planted_power_rises...`) checks one ordering of two counts. **Survivors: at least 11 verified (9 genuine, 2 equivalent) plus the unverified list; the verified rate of 61% is below a usual 80% bar for a statistic that will be frozen.**


## 7. Structural blockers (new ways the design can be unpassable or silently permissive)

| previous blocker | status in options-v2 | new risk |
|---|---|---|
| V10 ledger-complete with NOT-RUN families | fixed by spec items 9/14 | UNSELECTED catalogue state undefined; cap arithmetic of the reserve (F5) |
| deflation sets over NOT-RUN candidates | removed (candidate-level count) | the SPA universe is "registered candidates' base cells": registered-but-not-run candidates must be dropped from it, and an SPA NOT-EVALUATED must never block a verdict (reported statistic); not stated |
| tax-series ruin | fail closed at <= -100%, provisional tax | start-of-window tax state and benchmark convention undefined (F10) |
| bad benchmark prints | scan before freeze | the 2022-2026 benchmark and BTC leg are new inputs from o2data: scan and declare repairs before the tag; B1 loaded per window (F15) |
| benchmark inputs supplied by the framework, not re-derived | unchanged | B1/B3 on each stage grid is a new derivation (spec item 25 lists it as a data dependency); add it to the dry run: a planted weak benchmark cannot flip a verdict |
| derived-evidence binding | unchanged (hash-pinned prereg, forged controls) | hurdle table recomputed at run time from code constants (F16); the new `o2gates.rs` constants must be inside the hash |
| catalogue/status files | new catalogue from the capability audit | RUNNABLE-but-unselected rows (F5) |
| declared orphans | start empty | marker orphan rule (F5) |
| sealed-slice single touch per class | per candidate | shared-window leakage (F4) |
| (new) type A/B availability | fixed from the data manifest at registration | S2 not evaluable for a type-A candidate after S1 passes (too few liquid strikes or trades in Phase 2): verdict undefined; define it as DIAGNOSTIC, not GO and not KILL |
| (new) tail evidence | none | F1, F11 |
| (new) cycle straddling a stage cut (monthly cycle entered 2025-04-29, expiring 2025-05-27) | v1's rule "entry >= start and expiry <= end" is not copied | state the rule: a cycle belongs to the stage of its entry; a position carried across a cut is closed at the cut mark for the earlier stage's statistics |
| (new) low power by construction for monthly families | emp_crit hurdles at n = 18 (z = 2.12 at 10%) | a true Sharpe-2 monthly candidate passes S3 alone about 62% of the time (design: 31.8% fail) and the quality gates cut more: reachable but label the monthly families with their calibrated power in the registration table |

Gates unsatisfiable by construction were checked: G1 for Budget/RBI is declared DIAGNOSTIC (honest); weekly BankNifty/FinNifty/Midcap with S3 empty is DIAGNOSTIC (honest); a type-A candidate with a thin Phase 2 (24 months, about 100 periods) takes its S2 hurdle from `emp_crit(n)`, fine.

## 8. Verdict, required edits and the minimal change set

**Verdict: FIX THEN FREEZE.** Not REDESIGN: the stage structure, candidate-level counting, one a-priori cell, sign-only neighbours and fail-closed series are right and the power table is reproducible. Not SAFE TO FREEZE: the 5% guarantee is unproven for the shapes that matter, S2 is mis-specified in the prereg, and the cash and sizing definitions are open.

### 8.1 Required edits to `docs/prereg-options-v2.md`

1. Section 3 (alpha budget): add the type rule (F2.1), the classification table C1..C17 signed before any run, and "S2 is a blind pre-registered replication on development data (D-003), not a holdout".
2. Section 3 (gates): add **G9 stress exposure** and the adverse-regime-evaluable requirement; verdict INCONCLUSIVE-TAIL (F1.3, F11).
3. Section 3 (G2): the HAC t is computed on the realised-cycle series, MTM for drawdown only (F9); hurdles are the frozen numbers of an attached table (F16).
4. Section 3 (G3/G4/new G10): returns are P&L on fixed initial capital with no cash credit; absolute floor (annualised after-tax return >= the stated cash rate) over D1 and S3, or volatility-target scaling; drawdown second leg 3 x annual vol (F3, F14).
5. Section 5 (sizing): fixed-lot rule, no budget skips, liquidity-only skips capped at 15%, volatility band, numeric margin fraction and cap (F6).
6. Section 2 (windows): one continuous account/tax path sliced by stage; the cycle-at-the-cut rule; H and expiry weekday per stage; the data manifest states D1/S3 per file (F10, F13, F15).
7. Section 2 (sealed): replace sequential per-candidate touches by ONE sealed batch after all candidates are registered; no registration after the batch; delete the "new-slot re-run after touch" clause (F4).
8. Section 1 (caps): sealed-touch marker on its own cap axis; reserve only for dated re-runs of non-sealed stages; stage windows consume no cell; orphan rule; UNSELECTED catalogue state (F5).
9. Section 3 (outcomes): print DSR (N = 36 and N_all), SPA p, DSR < 0.5 = WEAK; forbid "passed/profitable" in any options-v2 verdict (F8).
10. Add a frozen-numbers appendix: hurdle table, null-family results, the omitted-gate power table.

### 8.2 Required edits to `docs/options-v2-protocol-design.md`

* Sections 0 and 3.1: remove "by construction"; state the bound is conditional on the stated null family and give the t-only family-wise rate on it (F1, F7).
* Section 12: re-run with (a) the null family, (b) `monthly_new`, (c) omitted gates implemented, (d) a lumpy-edge variant, (e) 10,000 draws for the benchmark-gate rows; label every power "upper bound, constant planted edge".
* Section 11.1: promote the v1-lineage flag into the type rule; resolve the contradiction between "verdict final" and "re-run" (F2, F4).
* Sections 11.3 and prereg 5: cash/return definition, tax convention for the benchmark, window-boundary state (F3, F10).
* Section 6: relabel the B3 point estimate a "screen"; equal rebalancing convention in calibration and production; disclose that the BTC leg of S3 is seen (F12).
* Section 12.0: delete or qualify "no real-data performance statistic" (the HIGH scenario is a research-v2 number).
* Section 7: second drawdown leg tied to volatility (F14).

### 8.3 Exact minimal changes before tagging (smallest set that makes the freeze defensible)

1. **Type rule + classification table** in the prereg (F2): prevents a 10x alpha overspend on the v1-lineage candidates.
2. **Null-family calibration run and the honest bound statement** (F1.1-F1.2): changes no hurdle, only the claim and the evidence.
3. **G9 stress-exposure gate** (F1.3): the only change that addresses the rare-tail failure no hurdle can fix.
4. **Return/cash definition and absolute floor** (F3): one paragraph and one test.
5. **Fixed-lot sizing, numeric** (F6): one paragraph.
6. **Single sealed batch, no registrations after it** (F4) and **marker/cap/orphan/UNSELECTED rules** (F5).
7. **Realised-cycle series for t/Sharpe** (F9).
8. **Frozen numeric hurdle table** (F16) and a `--max-date` guard on `--bench-csv`.
9. **Data manifest per-window flags and the D1/S3 benchmark split** (F15).
10. The mutation survivors of section 6: add the killing tests before the freeze hash, because the hash freezes the code that computes the gates.

11. **Data-agent facts (section 11):** weekly BankNifty/FinNifty/Midcap become HISTORICAL-ONLY rows outside m; weekly candidates defined on NIFTY only; expiry-day marks = intrinsic from the final settlement price (never SETTLE_PR, with a planted-zero test); liquidity filter on signal-day data with a per-candidate skip-rate cap; the 2026-11-01..17 buffer excluded and forward paper starting after the tag; lot-table sensitivity for stock candidates.

After the edits re-run `cargo test --workspace` and clippy (integration baseline: 1,934 tests green, clippy clean per STATUS).

## 9. What the protocol gets right

* The union-bound algebra and `alpha_dev` are correct (checked, tested, mutated).
* Candidate-level counting, sign-only neighbours, no argmax in the evaluation path, type fixed at registration, DIAGNOSTIC rather than FAIL for empty stages, the fail-closed physical-series guard, the benchmark-relative drawdown idea, quarterly stability, causal regimes, HAC lag with 2H, DE_time floored at 1, determinism.
* The design says openly that Sharpe 1 is undetectable in five years and that more independent data (forward paper) is the only fix; my power re-derivation supports that.
* The departure from DSR gating is disclosed as an owner decision (Q1).


## 11. Addendum: facts from the finished data agent (branch `hardening/o2data`)

Read from `docs/data-inventory-options-v2.md`, `docs/regulatory-timeline-options-v2.md`, `docs/data/lot-sizes-options-v2.csv` (not merged; read with `git show`). Five points from the coordinator, each assessed.

1. **Weekly discontinuations (data: BANKNIFTY weeklies 2016-06-02 to 2024-11-13, FINNIFTY 2021-01-14 to 2024-11-19, MIDCPNIFTY 2022-02-01 to 2024-11-18; NIFTY only after, Tuesday expiries from 2025-09-02).** The design's recollection (BankNifty weekly from 2016-10, discontinued about 2024-11) is corrected by data: BankNifty weeklies start 2016-06, so its Phase 2 is about 5.4 years (type A) and NIFTY weeklies 2019-02-14 (consistent with the 142 weeks used). Consequences: (a) weekly BankNifty/FinNifty/Midcap candidates have an EMPTY S3 and also no forward-paper path (no contract exists to paper-trade): they are **HISTORICAL-ONLY**, not "DIAGNOSTIC pending S3". They can never be PROMOTED, so they must not hold hypothesis slots: remove C16 and every BankNifty weekly leg from m (frees alpha for the real candidates) and label their S1/S2 results "historical evidence, no forward test". (b) Candidates pooled over Nifty+BankNifty weekly (C1, C2, C3) change composition between stages: S1 (to 2024-11) pools two underlyings, S3 has Nifty alone, so DE_harness, rho_bar, volatility and N_eff are not like-for-like and the type A/B and hurdle logic assume a constant candidate. Define weekly candidates on NIFTY only; BankNifty weekly becomes a separate historical-only row. (c) The alpha split stays coherent for the remaining Nifty-weekly candidates, but S3 contains a structural break: NIFTY expiry moves Thursday to Tuesday from 2025-09-02 (confirmed, circular read), so inside S3 the trade timing, the Friday-cut period alignment and H change (F13 is now a fact, not a recollection); and D1 contains the 2024-11-20 lot tripling, the +2% expiry-day ELM and the 2025-02 premium and calendar-spread rules.
2. **Expiry-day settle is 0 through 2020-01-30 and equals the underlying final settlement price (not a premium) from 2020-02-06.** A loader that reads SETTLE_PR as a premium values every option at 0 on expiry day (or at the index level): all short options become full wins, a silent huge false positive for any hold-to-expiry or expiry-day structure (C9 covered call, C10 cash-secured put, C13 expiry-day butterfly, any monthly hold to expiry). Required: expiry-day mark = intrinsic from the underlying final settlement price, never SETTLE_PR; a G8 test with a planted zero and a planted index-level settle must make the series INCOMPLETE; C13 has expiry-day marks only from 2020-02-06 (about 21 months in Phase 2 to 2021-10-31, below the 24-month type-A rule), so C13 is type B by availability; before 2020-02 an expiry-day mark does not exist (not a zero).
3. **Zero-volume rows 85% (2012-2018) falling to about 51-65% by 2026.** The v1 filter (volume > 0 and open > 0 on the FILL day) conditions on same-day information that is unknown at the entry decision: skips are informative (far wings trade on volatile days), a mild look-ahead and a regime filter. Use signal-day OI/volume only. Gate arithmetic with realistic skips: weekly D1 182 periods at 30% skips leaves 127 trades, N_eff gate 30 is safe; monthly D1 has 42 cycles and N_eff >= 30 needs about 33 trades at DE 1.08, so **any monthly candidate with more than about 23% skips cannot pass G1 in D1** (v1 skipped 12.7% for BankNifty condors with 7 far-wing strikes at OI < 500); sealed monthly needs >= 11 of 18. Phase-2 skip rates will be far higher (85% zero volume): the S2 series is mostly flat for wings, which depresses t and N_eff; state per candidate the expected skip rate from the data manifest at registration and set INCONCLUSIVE-SAMPLE (cap 15% as in v1, or a pre-registered per-candidate cap), and say whether skipped periods enter the HAC series as zeros (they should, with the skip count reported, because dropping them selects liquid days).
4. **Windows end 2026-10-31 but the store runs to 2026-11-17.** Sessions 2026-11-01..17 are downloaded and were processed by the data agent (row counts only), and the freeze/tag will be later still. They belong to no stage. Rule: they are a declared dead buffer, never evaluated; forward paper starts at the first session AFTER the tag date (not "2026-11-17" as prereg 2 and design 11.1 say) and the files of the buffer are excluded from every manifest used by the harness. Any DQ statistic computed over 2026-11-01..17 and over S3 must be listed (F15).
5. **Lot sizes before 2025 are data-derived (value / volume / close within 2%), not circular-verified.** For index underlyings the lots are confirmed by circulars (NIFTY 25 to 75 on 2024-11-20, to 65 on 2025-10-28; BANKNIFTY 15 to 30, 35, 30) and by the `lot_size` column from 2024-07-05; for stock options the series are inferred. Sizing consequence: a wrong lot changes costs and the integer rounding, not the sign per unit; with the F6 notional-based sizing the effect is the rounding only. Required: a sensitivity row (lot table perturbed by one regime step) for stock candidates (C9, C15) and for any trade with an unverified lot, the count of such trades reported (v1: 100% unverified), and stock candidates flagged lot-uncertain in the registration table.

## 10. Reproduction

`rustc -O probe.rs -o probe`, then `./probe size 40000`, `./probe chain 100000`, `./probe fwer 4000 17`, `./probe sel 20000`, `./probe power 20000`, `./probe crit 40000`, `./probe neff 20000`, `./probe tax 3000`; source at `/private/tmp/claude-501/-Users-thakur-Workspace-Trading/6f39fe22-10ec-4c71-9269-df8de83872d5/scratchpad/probe/probe.rs` (not committed, throwaway). Mutation harness `.../scratchpad/mut/mutate.py` on a scratch copy of the repo (never the review worktree); results `.../scratchpad/mut/results.jsonl`.

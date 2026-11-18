# Pre-registration options-v2 (FROZEN at the annotated tag `prereg-options-v2`, 2026-11-18; no threshold fitted to any real result)

Status: FROZEN 2026-11-18 at the annotated tag `prereg-options-v2` (the tag is never moved; after it a gate may only be tightened, by a dated addendum, and the changed test is a new counted trial). History: DRAFT of 2026-11-17 from branch `hardening/o2proto`, amended by addendum A1 (protocol fix after the adversarial review `docs/review-options-v2-protocol.md`, verdict FIX THEN FREEZE) and by the freeze record A2 (2026-11-18). No strategy has been run at the freeze; every number was fixed from SYNTHETIC calibration (`docs/options-v2-protocol-design.md`, addendum A1) before any result exists. Paper and backtest only. Frozen under the rule: after the tag, a gate may only be tightened, by a dated addendum, and the changed test is a new counted trial. **Never lowered after a result.** research-v2's gates and prereg are untouched.

Question. Does any pre-registered, defined-risk (or stress-labelled) Indian NSE options structure earn, net of costs, slippage and business-income tax, a Sharpe above the Nifty ETF and above the 50/50 ETF+BTC blend? The error target is a family-wise false-promotion probability of at most 5% **conditional on the iid loss-week null family named in section 9** (A1.1): measured <= 3.1% for K = 30 under the final design. It is **NOT controlled under a persistent calm/stress regime null** (measured family-wise 10.5% weekly type A, 20.7% weekly type B, 4.1% monthly type A, 13.6% monthly type B; per candidate 0.4% to 10.5%; Appendix B). A promotion therefore means only PROMOTED TO FORWARD PAPER, the real clean test, never evidence of profitability. A net Sharpe of about 1 is undetectable (power 3-7%); power at Sharpe 2 is 25-56% and at Sharpe 3 61-86% depending on shape and benchmark scenario (Appendix C). Null: no candidate has a net edge. Wording rule: no options-v2 outcome is ever called "passed" or "profitable"; the only positive outcome is PROMOTED TO FORWARD PAPER.

## 1. Candidates, K and caps
* Ledger family `options-v2`. A candidate = one structure x underlying class x expiry type x one numeric axis of 3 levels (base in the middle); candidates C1..C17 of `docs/options-v2-capability-and-catalogue.md` section 7, classified in section 3.1 (final list fixed in the registration commit; S1 naked strangle = labelled stress, not a hypothesis, not eligible for promotion).
* A **cell** is (candidate, parameter vector). Stage windows (D1, Phase 2, sealed) consume NO cell. Planned cells: 16 in-m candidates x 3 = 48 + 1 stress = 49 GO-wave cells (C16 is historical-only, section 6.1, and holds no slot); **cap 60 cells** (11 spare for dated re-runs of non-sealed stages, each counted).
* The sealed batch (section 2) consumes ONE marker per candidate from its own cap axis `o2-sealed-touch` (cap = K_cap = 30); markers never draw on the 60-cell axis.
* Hypothesis slots K_cap = 30 candidates; **m = 36 = 30 + 6 (options-v1 base variants)** whatever number is used. Removing C16 (historical-only) does NOT lower m: unused alpha is not spent (owner ruling 2026-11-17). A new candidate needs a dated addendum before the first run, and none may be registered after the sealed batch (section 2).
* Selection: the base (middle) cell only. No argmax (the harness evaluation path takes `Pick::Base` only). Neighbours are robustness cells (counted in the ledger, never hypotheses, never in a variance/SPA/PBO/DSR set).
* Catalogue states: RUNNABLE-and-selected (a candidate), NOT-RUN (consumes no cell, never a failure), and **UNSELECTED** (RUNNABLE, no candidate, no cell, never a failure, excluded from completeness and from every derivation set).
* Orphan rule: a sealed-touch marker or a sweep with ZERO stored results may be declared orphan by a dated addendum written before that candidate's next evaluation; a marker or sweep with stored results may never be declared orphan.

## 2. Windows
Phase 2: 2012-01-01..2021-10-31, **development data** (D-003: already-seen data is development only; options-v1 looked at its 6 monthly structures). D1: 2021-11-01..2025-04-30 (train 2021-11-01..2023-10-31, validation 2023-11-01..2025-04-30). Sealed S3: 2025-05-01..2026-10-31. Dead buffer: 2026-11-01 to the tag date, never evaluated, files excluded from every manifest the harness reads; the clean holdout is forward paper starting at the first session AFTER the tag date.
* **Single sealed BATCH.** Every candidate is registered before the batch. After S1 (and S2 for type A) and G8 pass for the last registered candidate, ONE touch writes all markers, runs all S3 evaluations and publishes all outcomes together. No candidate may be registered after the batch (it can only go to forward paper). There is no re-run after the batch and no S3 for a re-run. A per-candidate sealed verdict is final.
* **One continuous account and tax path** 2021-11 to 2026-10 with one loss carry-forward, sliced by stage for statistics; the benchmark uses the same provisional business-income convention for option-like legs and capital gains for the ETF; the pre-tax Sharpe is reported next to the after-tax one.
* A cycle belongs to the stage of its ENTRY date; a position carried across a stage cut is closed at the cut mark for the earlier stage's statistics.
* Per stage the manifest records the contract-rule regime: H (periods a position is held), the expiry weekday (Nifty weekly expiry moves Thursday to Tuesday from 2025-09-02), lot size history, the 2024-10/11 SEBI changes. Lag and N_eff use the stage's own H. A failed S3 of a candidate that passed S1/S2 is not read as evidence against an edge that existed under the older contract rules, and the report says so.
* The data manifest states per file whether it is D1, Phase 2 or S3; the DQ scan runs on D1 and Phase 2 only until the sealed batch, or a dated addendum written before the freeze declares exactly which S3 statistics (row counts and schema only) were read. B1 and B3 are loaded per window.

## 3. Gates (all numeric; base cell; weekly series, calendar-monthly for monthly cycles)

### 3.1 Type rule and classification (A1.2)
A candidate is **type B** if its structure x underlying class x expiry type matches, or is a parameter, entry or exit variant of, any options-v1 variant V1-V6 (monthly Nifty/BankNifty bull put spread, iron condor, straddle), or its base parameters were chosen or screened using any Phase-2 return, or its design used the printed v1 result tables, or it is a structural cousin of those. **Where there is doubt the candidate is type B.** **Type A** = no prior look at that window or structure, a base parameter vector written from a stated rule (volatility-unit offsets) before any Phase-2 return is computed, and at least 24 months of the structure's contracts in Phase 2. Phase 2 is a **replication/consistency check that spends no alpha for type B**; for type A it is a pre-registered blind replication on development data (not a holdout) and spends alpha_2. The type is fixed at registration and cannot change after a result.

| Cand. | Structure | Type | Reason |
|---|---|---|---|
| C1 | weekly bull put spread, Nifty | B | v1 V1 lineage (offset axis justified by v1's 1-sigma monthly offset); weekly BankNifty leg removed (6.1) |
| C2 | weekly iron condor, Nifty | B | v1 V3 lineage |
| C3 | weekly iron butterfly | B | cousin of V3 (condor/butterfly machinery) |
| C4 | conditional defined-risk VRP, weekly | B | cousin of V1/V3 (same structures, entry filter) |
| C5 | monthly spread / condor | B | direct v1 V1-V4 lineage |
| C6 | long straddle/strangle | B | direct v1 V5/V6 lineage |
| C7 | calendar | A | no v1 counterpart; offsets in sigma units |
| C8 | broken-wing butterfly, weekly | A | no v1 counterpart |
| C9 | covered call, monthly | A | no v1 counterpart (future + short call); lot-uncertain for stocks, sensitivity row required |
| C10 | cash-secured put, monthly | B | short put leg of V1 (cousin), doubt rule |
| C11 | trend-filter overlay | A | no v1 counterpart |
| C12 | delta-hedged straddle | A | no v1 counterpart (hedge leg), tenor axis |
| C13 | expiry-day iron butterfly | B | cousin of V3, and only about 21 months of expiry-day marks exist in Phase 2 (below 24): type B by availability, never type A |
| C14 | RBI MPC straddle | A | no v1 counterpart; DIAGNOSTIC for G1 (too few events) |
| C15 | results straddle, stocks | A | no v1 counterpart; lot-uncertain, sensitivity row required |
| C16 | FinNifty / Midcap weekly spread | B | HISTORICAL-ONLY, outside m (6.1) |
| C17 | back-ratio / jade lizard | A | no v1 counterpart; low priority, run last |

Classification entered by the protocol agent 2026-11-17. **Signed: orchestrator, 2026-11-18, assumption recorded** (under the owner's standing instruction to make reasonable assumptions and write them down; the assumption is that the doubt rule is applied conservatively and that the table is a classification of knowledge, not a result). Any later change of a type is a new addendum and cannot be made after a result.

### 3.2 Alpha budget and hurdles
FWER 0.05, m = 36: type A alpha_1 = 0.139 (S1), alpha_2 = 0.10 (S2), alpha_3 = 0.10 (S3); type B alpha_1 = 0.0139, alpha_3 = 0.10, no S2 alpha. Hurdles are the **frozen numeric table** of Appendix A (HAC t of the mean, lag max(auto, 2H), the larger of the deterministic routine value and the draft value; never recomputed from a modified pool; any other n uses the same routine and the value is written to the sweep record).

### 3.3 Gate table
| Gate | S1 (D1) | S2 (Phase 2; gates for type A, reported for type B) | S3 (sealed) |
|---|---|---|---|
| G1 independent trades | N_eff >= 30 (N_eff = N_raw / max(DE_harness, DE_time)); skips (signal-day liquidity) <= 15% of scheduled cycles else INCONCLUSIVE-SAMPLE | N_eff reported | N_eff >= 10; same skip cap |
| G2 edge | HAC t >= frozen z(alpha_1) on the REALISED cycle P&L series (entry fill to exit fill, flat to flat; MTM is used for drawdown only) | t >= frozen z(0.10) | t >= frozen z(0.10); mean > 0 |
| G3 benchmark | SR > SR_B1 with paired 80% lower bound > 0; SR > SR_B3 (point screen) | SR > SR_B1 (point) | SR > SR_B1 and SR > SR_B3 (point screens) |
| G4 drawdown | MDD <= clamp(1.5 x MDD_B1, 25%, 40%) AND MDD <= 3 x annualised volatility of the book | same two legs on the window | same two legs |
| G5 stability | >= 60% of full quarters positive (applies from 3 evaluable quarters, else n/a); train and validation means both > 0; dropping best 1% (>= 2) periods leaves net > 0; >= 75% of evaluable causal regime cells positive (>= 2 evaluable else n/a); **the adverse regime (benchmark down-trend and high volatility) must be evaluable (>= 10 weekly periods, >= 3 monthly) in D1 or the verdict is INCONCLUSIVE-TAIL** | none | none |
| G6 robustness | both one-step neighbours have a positive net mean | none | none |
| G7 cost stress | 2x slippage and 2x brokerage keep the net mean > 0 | | |
| G8 integrity | prefix-invariance (no look-ahead), byte-identical re-run, no sealed-dated file read, series physical (no period return <= -100%, fail closed), data scan, carried marks <= 10% of leg-days (else INCONCLUSIVE-QUALITY) | | |
| **G9 stress exposure** (ADDED) | for net-short-convexity structures: the window holds >= 3 benchmark weeks <= -3% OR >= 1 week <= -5%; else INCONCLUSIVE-TAIL | same rule | >= 1 benchmark week <= -3%; else INCONCLUSIVE-TAIL |
| **G10 absolute floor** (ADDED) | annualised after-tax net return on the FIXED initial capital >= 6.0% (arithmetic mean x periods per year; no cash credit) | | same on the sealed window |
| **G11 volatility band** (ADDED, sizing validity) | annualised volatility of the book in [6%, 14%]; **a book that is too quiet (below 6%) is INCONCLUSIVE-SIZE, not a pass and not a kill**, and so is one above 14% | | |

Net short convexity = credit spreads, condors, butterflies, covered calls, cash-secured puts, calendars, ratios, expiry-day sellers, jade lizards. G9, G10, G11, the adverse-regime requirement and the second drawdown leg are **additions or tightenings** (A1.1, A1.3); the draft had none of them. The India-VIX leg of the reviewer's G9 suggestion is not used (VIX is not a benchmark series of the o2data inventory); a VIX leg could only be added later as a tightening.
Reported, not gating: DSR at N = 36 and at N_all (**printed WEAK when DSR(N = 36) < 0.50; a WEAK candidate needs the owner's confirmation before forward paper**), SPA p (NOT-EVALUATED and non-blocking when a registered candidate has no returns; SPA universe = base cells of registered candidates that have returns), PBO not computed, MC drawdown percentiles, B2 comparison, kill-switch operating run, the ETF's and the blend's worst drawdown in every window, the pre-tax Sharpe, the stress replay (the candidate's structure re-priced under the Phase-2 worst benchmark week with a fixed vol shock, labelled a stress; its loss must fit inside the drawdown limit and is reported as a multiple of Rs 1 crore and of the average credit), the worst benchmark period and the candidate's loss in it in units of average credit, the volatility of the book, lot-table sensitivity for stock candidates.
Outcomes: PROMOTED TO FORWARD PAPER (all gates; then >= 60 sessions and >= 100 N_eff trades where the structure trades that often, results inside the Monte Carlo band, positive expectancy); INCONCLUSIVE-SAMPLE (G1 short or skips above the cap); INCONCLUSIVE-TAIL (G9 or adverse regime not evaluable, with no gate failed); INCONCLUSIVE-SIZE (G11 or a lot that cannot be sized); INCONCLUSIVE-QUALITY (carried marks); DIAGNOSTIC (a stage has no contracts); KILL otherwise. A failed gate is a KILL and takes precedence over any INCONCLUSIVE flag. Never "passed", never "profitable".

## 4. Trial accounting
Counter of every opened cell (options-v1 30 + options-v2 cells + sealed markers on their own axis); `N_true = m = 36` for deflation reporting; N_all adds research-v2 (173 cells) and the 364 lineage rows for information. Earlier options sweeps: only options-v1 (K = 30) exists. Contamination bookkeeping: section 8.

## 5. Costs, tax, returns and sizing
`docs/costs-tax-options-india.md` and the date-keyed rates table (unverified rows flagged in every result). Slippage s = max(1% premium, Rs 1), stress 1x/2x/3x. Tax: business income 31.2%, loss carry-forward 8 years, provisional tax on positive cumulative FY net income, one continuous standalone account path (section 2); sensitivities 20%/39% and same-year symmetric offset.
* **Returns** (A1.3): options P&L (plus futures hedge P&L) net of costs and tax divided by the FIXED initial account capital of **Rs 1 crore**. No interest, margin yield or collateral income is credited to any numerator; a test credits 6.5% to idle cash and shows that no verdict changes. Sharpe, HAC t and the benchmark test use the realised-cycle series, never stale EOD marks.
* **Sizing** (A1.5): every cycle trades the SAME NOTIONAL rounded to whole lots; lots = max(1, floor(budget x capital / max loss per lot at the START of the window)), budget = **3.0% of capital per weekly cycle, 5.0% per monthly cycle**; if one lot alone risks more than **6.0%** of capital the candidate is INCONCLUSIVE-SIZE. Premium and implied volatility are never inputs, so no cycle is skipped for budget (`rules::fixed_lots`). Lot-size history (Nifty 25/75/65, BankNifty 15/30) is taken per date; the max loss per lot is set at the window start. The only skip is the signal-day liquidity filter (signal-day OI/volume only, never fill-day data); skipped periods enter the HAC series as zero P&L with the skip count reported; above 15% skips the verdict is INCONCLUSIVE-SAMPLE (not a pass). The expected skip rate per candidate is stated at registration from the manifest. Rs 25 lakh is a reported sensitivity with the SAME rule. Defined-risk margin = maximum loss; naked only as a labelled stress.
* Expiry-day marks (A1.7): intrinsic value from the underlying final settlement price, never SETTLE_PR (0 through 2020-01-30, the settlement price from 2020-02-06); a G8 test plants a zero and a planted index-level settle and the series must be INCOMPLETE; before 2020-02 an expiry-day mark does not exist (not a zero).

## 6. NOT-RUN and data rules
Catalogue of 130 cells: 100 RUNNABLE, 30 NOT-RUN (15 data unavailable: intraday option prices, vendor IV surface, quotes; 15 other: stock weeklies not listed, stock naked/jade lizard, physical-settlement holds). NOT-RUN entries consume no cell and are excluded from every derivation set; they are never failures. Every series is scanned (bad prints, gaps, non-positive values) and repairs are declared in a dated addendum BEFORE the freeze. Private copy of the tq binary for every sweep. Fail closed on any sealed-dated read.
### 6.1 Weekly contracts (data facts)
Weekly candidates are defined on **NIFTY only** (weeklies from 2019-02; Tuesday expiry from 2025-09-02 handled by the per-stage manifest). BankNifty (to 2024-11-13), FinNifty (to 2024-11-19) and Midcap Nifty (to 2024-11-18) weeklies are **HISTORICAL-ONLY** rows: outside m, no hypothesis slot, labelled "historical evidence, no forward test", never promotable (no contract exists to paper-trade). C16 and every BankNifty weekly leg are historical-only.

## 7. "Can reach promotion" per gate family (upper bounds under a constant planted edge; Appendix C)
| Gate | Can reach it? |
|---|---|
| G2 edge | yes for a net Sharpe of about 2 or more (Appendix C); **a net Sharpe of about 1 is undetectable in this data (3-7% power, Appendix C) and the report says so; at Sharpe 2 power is 55.8% / 55.0% (weekly type A / B, LOW benchmarks) and 44.6% / 32.8% (monthly type A / B)** |
| G3 benchmark | yes; harder when the blend's window Sharpe is high |
| G4/G10/G11 | yes for a book sized into the band |
| G9 | a short-convexity book can be PROMOTED only if D1 and S3 contain benchmark stress; otherwise INCONCLUSIVE-TAIL |
| G1 trades | weekly Nifty, ladders, pooled stocks, expiry-day, daily-hedged, results (clustered): yes. Monthly single index: borderline. **No by construction: Budget, RBI MPC (DIAGNOSTIC)** |
| S2 | type A only; type B reports it |

## 8. Contamination bookkeeping
Format-level exposure, listed and negligible: options-v1 downloaded nine sessions (2019-03-01..07 and 2025-10-27..31) and probed single dates 2001-06-04, 2005-01-03, 2012-01-02, 2016-10-03, 2023-01-02, 2024-07-05, 2024-08-01, 2026-01-05 (row counts, field names, zero-volume shares, one printed row each; no return computed); three probes lie in D1 and 2025-10-27..31 and 2026-01-05 lie in S3. The o2data agent processed sessions 2026-11-01..17 (row counts only; dead buffer) and scanned data for bad prints; any S3-dated statistic it read is listed in its dated addendum. **Seen real numbers used in the calibration**: the HIGH benchmark scenario (ETF Sharpe 0.8, BTC 0.9, blend about 1.3) is a research-v2 measured value, and the BTC leg of the sealed window is already seen through the crypto development side (research-v2 evaluated BTC through 2026-10; its weekly Sharpe was 0.83 over its window). Designers know the 2013-2025 Nifty path in general terms; D1 is development data, not independent of the designers (D-006).

## 9. Null family (A1.1)
The error claim in the question is conditional on these zero-edge nulls, each stated in `crates/r2stats/src/o2cal/nullfam.rs` and reported in Appendix B: the design pool (negatively skewed iid, the control), iid defined-risk credit with loss in 5% and in 2% of periods, a persistent calm/stress regime (loss 5% / 40%, mean durations 78 / 26 periods), and a null whose losses coincide with benchmark stress periods. For rare-tail and regime nulls the staged t-test alone does not hold the bound and no finite hurdle restores it (Appendix B); G9 and INCONCLUSIVE-TAIL are the route, and the residual family-wise rate is reported there as measured: **10.5-20.7% family-wise under the persistent-regime null for weekly books and 13.6% for monthly type B, which the protocol does not control**.

## 9.1 Open for owner (proposals only; NOT implemented, no gate was added to chase the regime null after seeing these numbers)
* The regime null (stress independent of the benchmark) is not controlled by any gate. A possible additional gate would require the candidate's worst losses to be explainable by benchmark stress, or would cap promotion to a smaller K. Either is a NEW gate and would be a counted addendum; it is not part of this draft.
* Whether to accept a conditional 5% claim (iid null family) with the regime null disclosed as uncontrolled, as written here.

## 10. Previous blockers and their avoidance: see `docs/options-v2-protocol-design.md` section 1 (B1-B10).

## A1. Addendum 2026-11-17 (protocol fix; every change is definitional, made before any strategy result)
Reason: skeptic review FIX THEN FREEZE. **No gate was lowered.** "Tighten" = a gate made stricter, "add" = a new gate, "define" = an open definition pinned, "restate" = a claim re-worded.
| # | Change | Kind | Reason (review finding) |
|---|---|---|---|
| A1.1 | FWER claim conditional on the named null family (section 9); null family added to calibrate-o2; G9 stress exposure (INCONCLUSIVE-TAIL); adverse regime must be evaluable; "by construction" removed | restate + add | F1, F11 |
| A1.2 | type rule A/B and the signed C1..C17 table; Phase 2 spends no alpha for type B | tighten | F2 |
| A1.3 | returns = P&L on fixed capital, no cash credit; G10 floor 6%; G11 volatility band 6-14%; second drawdown leg 3 x volatility; t/Sharpe on realised cycle P&L; hurdles a frozen numeric table | add + tighten + define | F3, F9, F14, F16 |
| A1.4 | ONE sealed batch, nothing registered after; sealed-touch marker on its own cap axis; orphan rule; UNSELECTED state; "re-run as new slot after touch" deleted | define | F4, F5 |
| A1.5 | fixed-notional sizing (3% / 5% budget, 6% single-lot cap, min one lot), skip cap 15% (INCONCLUSIVE-SAMPLE), lot rounding is not an IV filter, lot-table sensitivity for stock candidates | define + add | F6 |
| A1.6 | contamination note (section 8); DSR(N = 36) < 0.5 WEAK flag; no "passed"; B3 point test relabelled "screen"; BTC leg and HIGH values disclosed as seen; calibration B3 rebalanced quarterly as in production | restate | F8, F12, F15 |
| A1.7 | weekly candidates Nifty-only; BankNifty/FinNifty/Midcap weeklies historical-only outside m; Tuesday-expiry break; expiry-day settle rule with a planted-zero test; forward paper starts after the tag; signal-day liquidity filter; C13 type B | define | data-agent facts |
| A1.9 | orchestrator rulings 2026-11-18: quarter gate stays at >=3 evaluable quarters (stricter than the spec's 5), sizing numbers and the 6-14% band accepted, G9 thresholds accepted as written (VIX leg dropped), frozen hurdle = max(routine, draft) with the Monte Carlo error stated per row, C16 removal keeps m = 36, classification signed | define | decisions 1-6 |
| A1.8 | code: `--max-date` guard (default 2021-12-31, fail closed) on `--bench-csv`; `draw_bench` fails closed instead of cycling a short pool; quarter gate applies from 3 evaluable quarters (the spec text said 5, a looser gate: the code value is the stricter and stands, with a killing test); mutation-survivor killing tests | tighten | F15, F16 |

## A2. Freeze record 2026-11-18 (orchestrator, under the owner's standing instruction to make reasonable assumptions and write them down; every item is definitional and made before any strategy result; no gate was lowered)
| # | Ruling or assumption | Effect |
|---|---|---|
| A2.1 | DSR(N = 36) and DSR(N_all) and SPA are printed, never gating; WEAK flag at DSR(36) < 0.5 (accepted departure from D-006 item 3, disclosed) | reporting |
| A2.2 | Section 9.1 closed as written: the conditional 5% claim is accepted with the persistent calm/stress regime null disclosed as UNCONTROLLED (10.5% / 20.7% / 4.1% / 13.6% family-wise); NO gate was added and K was not cut after seeing these numbers; any such gate is a new counted addendum | claim wording |
| A2.3 | Printed power (Appendix C) is an UPPER bound: the calibration excludes drop-best, G6, G7, G8, the all-candidate adverse-regime rule and the audit/size gates; real power is a little lower. A net Sharpe of about 1 is undetectable | disclosure |
| A2.4 | B1 dividend yield: primary = NSE Indices NIFTY 50 dividend-yield series `nifty50_div_yield.csv` (sha256 0d269ba2ad425a027f5bf0ee64ea98b3e4a61fedaa82895cbd4d2093ebf97217; latest published on or before the session, at most 4 days old, else fail closed). 167 sessions (2026-02-04..2026-10-08, inside the sealed window) are RESTATED by +0.05..+0.29 pp against the as-printed daily file: the restated value is higher, so the benchmark accrual is larger, the STRICTER choice. The as-printed daily-file series is a report-only sensitivity. A constant yield is never primary. The index yield approximates the ETF's distributions (flagged assumption) | benchmark |
| A2.5 | Cells: planned GO-wave 49 (16 in-m candidates x 3 levels + 1 stress cell), cap 60, spare 11 for dated re-runs, sealed-touch cap 30 on its own axis `o2-sealed-touch`. The 52+8 of the pre-A1.7 design is superseded | caps |
| A2.6 | NOT-RUN rows (consume no cell, never a failure): C14 RBI MPC straddle (no verified primary-source decision-date list; also DIAGNOSTIC for G1), C15 results straddle (no free point-in-time results-date source), S1 naked-strangle stress cell (no policy family registered). C16 is HISTORICAL-ONLY, outside m. Unlock: RBI press-release dates with URLs, a vendor corporate-event calendar, a registered stress policy | catalogue |
| A2.7 | C9 (future + short call) and C10 (net short put) run through the SPAN-like stress path, which refuses a zero loss cap: their stress loss cap is the monthly risk share, 5% (`SPAN_LOSS_CAP_MONTHLY`). Their candidate preregs state it | sizing |
| A2.8 | Policy definitions: trailing realised vol = 20 returns of closes dated on or before the decision day, scaled by sqrt(calendar days to expiry / 365); C12 tenors weekly = (weekly, min_dte 3), two_week = (weekly, 8), monthly = (monthly, 15); C4 signal = ATM implied minus trailing realised vol, thresholds zero / median / p75 over the prior history_sessions (60..504) strictly before the decision day, min_history 20..252; hedge futures margin for C12 is NOT modelled (disclosed assumption; the C12 candidate prereg adds a margin-utilisation sensitivity) | policy |
| A2.9 | Exit rules: premium-based exits (profit_target, stop_loss) are FORBIDDEN for C9 and C12 (Position premium base includes futures notional); C13 and C14 enter daily self-gated (every 1 session) and C13 holds to expiry; exchange-holiday eve cycles for C13/C14 are LOST, never guessed | engine |
| A2.10 | Parameter levels of every candidate grid must be strictly ascending in canonical STRING order (the ledger sorts levels as text), for example 0.5, 1, 1.5 or 2, 3, 4; other level sets are refused at spec load | grid |
| A2.11 | Stricter readings confirmed: adverse-regime evaluability for every candidate; drop-best-1% = max(2, ceil(1%)) periods; S2 SR > B1 strict; G6 needs exactly 2 neighbours, each net-profitable with Sharpe > 0; S3 needs mean > 0 as well as t; G8 settle flag and non-physical series are INCOMPLETE (both block promotion); a ruined fixed-capital account is a KILL; hurdle for an n outside the table is the routine value; type A needs at least 24 months of Phase-2 periods or is INCOMPLETE; development facts must be committed and clean | gates |
| A2.12 | Real-data findings accepted before any result: a lot printed on the contract wins over the date-keyed table (the table mis-sized about 11% of 2025 NIFTY cycles); contracts straddling a lot revision with no printed lot are UNUSABLE; expiry-day settlement comes from the index close or the underlying settlement, never an option row's SETTLE_PR; weekly families need the index close series in the run inputs or fail closed | data |
| A2.13 | Attested inputs: the 2x cost-stress mean, the skips, N_raw, DE_harness, carried marks and expiry bookings are DERIVED from the engine output through the adapter (nothing hand-supplied); only registered constants (type, hold, short_convex, data_scan_clean, benchmarks) are attested, bound to the CLOSE root | evidence |
| A2.14 | Forward paper starts after the tag date (sessions 2026-11-01..2026-11-17 are a dead buffer); the real clean holdout is forward paper only | holdout |
| A2.15 | Verified at the freeze commit: clippy -D warnings rc 0 and workspace tests rc 0 (2,401 passed, 0 failed); the 14 research-v2 results are byte-identical on the same build | verification |

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


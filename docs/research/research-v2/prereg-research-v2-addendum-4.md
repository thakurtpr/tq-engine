# Addendum 4 to `prereg-research-v2` (benchmark bad-print repair and two evaluator conformance fixes)

Dated 2026-11-17. Written by the calibration branch `hardening/r2calib` on the orchestrator's ruling.

**Timing, stated plainly.** This addendum is written AFTER three candidates (`r2-trend-lc-sma`, `r2-trend-lc-don`, `r2-trend-lc-tsmom`) had been run and closed in the real ledger. The orchestrator states that those three kills do not depend on benchmark B1 (they fail gates that do not read B1); this is to be re-verified by re-evaluating their saved cells through the fixed evaluator (no cell is re-run). All 14 candidates of the frozen catalogue are evaluated identically under this addendum: no candidate is singled out.

**Scope.** No gate, threshold, cap, split boundary, grid or catalogue entry changes. Every item below makes the evaluator STRICTER or makes it follow the TAGGED text more faithfully. Per the edit rule of the master (L3) a post-tag edit may only tighten and needs a dated written reason.

## 1. Benchmark bad prints (master 2.3: B1, B2, B3 built from price levels)

**Finding.** The development store holds a bad print in `NIFTYBEES.NS` and `BANKBEES.NS`: the close is about 13 instead of about 129 on 2019-12-19 and 2019-12-20 (a -90% day), reverting to about 129.9 on 2019-12-23 (+897%). The D-OOS window of every IN-EQ candidate contains it. Built as stored, B1 has a -90% day followed by a +897% day: its weekly Sharpe collapses and OG2-B1, V5, OG6 and the 4c-rel alpha regression become far easier to pass. That is a false-pass path through a self-supplied input.

**Rule (one shared function, `tq_r2stats::bench_clean::repair_bad_prints`, applied inside `bench::run`, so the tq-r2stats benchmark engine and the tq-r2run benchmark builder behave identically).**
* A run of at most **3** consecutive bars is a bad print when every bar's close deviates from the close before the run by a factor of **>= 5 or <= 1/5** AND the bar after the run is back within **20%** of the pre-run close. It is replaced by carrying the pre-run close.
* Every repaired bar is listed in the output (`Benchmarks.repairs`, printed in the report's `self-supplied` block).
* **Fail closed:** if, after repair, any move of **>= 50%** remains that the series reverses within 5 bars, the benchmark is refused and the gates that need it are not evaluated.
* A persistent move (a real crash) is never touched.

**Effect.** B1 is harder to beat; nothing becomes easier. The class benchmark CLOSE levels used for the OG4 regime labels are not repaired by this rule (the single bad print moves a 200-day average by well under 1% and no label is expected to change; not verified here).

## 2. Ring-1 neighbours are one NUMERIC step (master 2.2 OG5, tagged text)

The evaluator sorted axis levels as strings, so mixed-width numeric levels ({5,10,15}, {3,5,10}, {60,90,120}, {0,7,14}) lost numeric neighbours. Ring-1 is now one numeric step on one parameter, as tagged. Cell indices, ledger hashes and stored cells are unchanged. This restores neighbours that the tagged text includes; it affects `r2-mr-*-rsi2` (both axes), `r2-mr-*-{zscore,ibs}` (hold), `r2-xsmom-lc` (k) and `r2-xsmom-cr` (lookback, skip). The three trend candidates are unaffected (equal-width levels).

## 3. The candidate must be the nominated base (master 3.3, "there is no argmax over the grid")

The evidence binder now refuses any cell that is not the centre (numeric median level on every numeric axis of 3 or more levels) of the candidate's grid. This only tightens.

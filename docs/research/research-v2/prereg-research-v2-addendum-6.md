# Addendum 6 to `prereg-research-v2` (benchmark-side data validity: regime closes, USDINR, VIX, loading boundary)

Dated 2026-11-17. Written by the branch `hardening/r2fix5` on the orchestrator's ruling after the read-only data-quality scan of the development store.

**Timing, stated plainly.** This addendum is written AFTER seven candidates (`r2-trend-lc-sma`, `r2-trend-lc-don`, `r2-trend-lc-tsmom`, `r2-xsmom-lc`, `r2-mr-lc-rsi2`, `r2-mr-lc-zscore`, `r2-mr-lc-ibs`) had been run, closed and killed in the real ledger. Those kills do not depend on the inputs repaired here: each of them also fails gates that read neither the USDINR series, nor VIX, nor the class-benchmark closes (V4 Sharpe, VAL, V9 and others, as recorded in `hardening/STATUS.md`), and none is a development GO. This is to be re-verified by re-evaluating their saved cells through the fixed evaluator (no cell is re-run, no trial is added). **No development GO exists** at the time of writing. The decision is taken before any candidate has one and applies **identically to all 14 candidates** of the frozen catalogue; no candidate is singled out and no candidate's outcome was a reason for it.

**Scope.** No gate, threshold, cap, split boundary, grid or catalogue entry changes. Every item below only TIGHTENS the evaluator or makes it follow the tagged text (benchmarks built from honest price levels) more faithfully. Per the edit rule of the master (L3) a post-tag edit may only tighten and needs a dated written reason; this is it. No crypto, funding, perp or equity candidate return series is repaired: the scan found no bad print in them (flagged moves are real events with consistent volume).

## 1. Class benchmark closes used for the OG4 regime labels (A)

**Finding.** Addendum 4 repaired the NIFTYBEES/BANKBEES bad print (2019-12-19/20: about 13 instead of about 129, back on 2019-12-23) in the benchmark RETURNS B1 only. The close LEVELS that feed the regime labels (`class_benchmark_closes`: trend = close vs its 200-day SMA, volatility = 60-day realised vol vs its expanding median) still carried it. Addendum 4 stated that no label was expected to change and that this was not verified. It is now verified: it is false.

**Rule.** The evaluator passes the class benchmark closes through the SAME shared function `tq_r2stats::bench_clean::repair_bad_prints` (one code path; thresholds unchanged: run of at most 3 bars, factor >= 5, back within 20%, fail closed on a >= 50% reverting move) before labelling. Every repaired bar is listed in the report's `self-supplied` block. An unrepaired reverting spike leaves OG4b NOT-EVALUATED (never computed on such labels).

**Effect (development store, NIFTYBEES.NS, IN-EQ D-OOS 2015-01-01 to 2021-12-31).** 64 daily labels change, all between 2019-12-20 and 2021-12-22 (6 trend flips, 60 volatility flips, from the 60-day window and the expanding median). Nothing else is touched. BTCUSDT closes (crypto regime series) have no repair. Bars repaired: 2019-12-19 (printed 13.02, carried 129.254) and 2019-12-20 (printed 13.034, carried 129.254).

## 2. USDINR validity (B): converts BTC to INR for B2 and B3

**Finding.** `CRYPTO/USDINR.csv`: closes on 2012-01-27 (52.49) and 2012-01-31 (52.49) lie outside their own bar range (49.3 to 49.8) and revert; 10 bars close more than 0.5% outside their own [low, high]; reverting moves of 2% or more on 2013-09-03, 2020-04-01/02/05, 2026-04-12/13 and 2026-08-25. The weekly aligner's USDINR filter acts only above 3%.

**Rules (`tq_r2stats::aux_clean::repair_usdinr`, applied at the loading boundary in `tq_r2run::auxload`, before the existing 3% filter, which stays).**
* **F1.** A bar whose close lies outside its own [low, high] by more than **0.5%** is an invalid print; its close is replaced by the previous valid close, and listed.
* **F2.** A move of **>= 2%** from the previous valid close that reverts to within **1%** of the pre-move level within **3 bars** is a bad print; every bar before the reversion is carried from the pre-move close, and listed.
* **Fail closed.** After repair, a move of >= 2% that still reverts to within 1% within **5 bars** refuses the series. A non-positive price, or an invalid first row (no previous value to carry), refuses it.

**List of repaired bars (all 17, the whole development store).** 2009-02-06, 2009-02-20, 2009-03-27, 2010-11-08, 2011-10-27, 2012-01-27, 2012-01-31, 2012-04-03, 2018-10-18, 2023-11-02 (F1); 2013-09-03, 2020-04-01, 2020-04-02, 2020-04-05, 2026-04-12, 2026-04-13, 2026-08-25 (F2). The report lists each with printed and carried value. Unrepaired after the rules: nothing (the refusal check passes on the real series; the remaining 2% moves of 2008, 2012-07/09, 2013-07/08 and 2019-08 are persistent, not reverting).

**Effect on the benchmarks (crypto D-OOS window 2018-10-01 to 2026-09-30, costs and tax as built; weekly Sharpe = mean/sd x sqrt(52), Friday cut).** B1: unchanged (not built from USDINR). B2 (BTC in INR): weekly Sharpe 0.8322 before, 0.8324 after; total return 10.5904 before and after to four decimals; 17 daily returns differ. B3 (50/50, window limited by the NIFTYBEES series to 2018-10 to 2021-12): weekly Sharpe 1.3143 before, 1.3109 after; total return 2.7630 before, 2.7561 after; 438 daily returns differ (rebalance path). The 2012 and 2013 repairs lie before BTCUSDT exists and do not enter B2/B3. The repair makes B3 marginally harder to beat and B2 marginally easier by 0.0002 Sharpe: immaterial, and the repair is declared because it is the honest series, not because of its direction. No gate changes.

## 3. India VIX (C)

Rule V1: a VIX row with a non-positive open, high, low or close is an invalid print; its close is replaced by the previous valid close and listed; a series that starts on an invalid row is refused. Real rows: `NSE-INDIAVIX` 2021-02-12 and 2021-03-30 (open = high = low = 0, a stale close). The integration branch has no VIX consumer (the options study that read it is closed); the function is provided and tested so that any VIX input uses it.

## 4. Generic fail-closed validation at the loading boundary (D)

`tq_r2run::auxload::load_aux` is the one place where the B1 series, the BTC series, the USDINR series and the class-benchmark series enter a run. Each is refused, naming the series and the date (development window only; the store holds nothing later), when it has: a non-positive open, high, low or close; a close outside its own [low, high] by more than 0.5%; or an unrepaired reverting spike (the addendum-4 function is the arbiter, so the declared NIFTYBEES repair still loads). USDINR is repaired under section 2 instead of refused for out-of-range closes. Checked on the real development store: NIFTYBEES.NS, BANKBEES.NS, BTCUSDT and BTC-USD pass with no row above 0.5%.

## 5. Zero-volume scan (E), report only

Completed over all 144 development CSV files (the 20 `FUND-*` files and `MEMBERS` have no volume column and are skipped by header). No crypto spot or perp series has a zero-volume row. Zero-volume rows are structural for USDINR and the VIX and index files (volume is not recorded there) and for 78 of the IN-EQ files (about 0.1% to 7% of rows, 35.6% for NESTLEIND, 16.7% for TMPV), almost all dated before 2014-10-15 (a few to 2015-08-12 and 2021-11-04). NIFTYBEES.NS has 171 and BANKBEES.NS 348 zero-volume rows, all before 2015-03 and all flat open = high = low = close fills; one falls in the IN-EQ D-OOS window (2015-02-28). Volume is not an input of any benchmark and no gate reads it; no rule is added.

## 6. Non-physical series guard (G)

**Finding.** Sweep `821e20a6` (`r2-xsmom-cr`, 27 cells; read-only inspection of its saved `.f64` files) has 18 of 27 cells with 297 daily net returns of -100% or worse (worst about -259x). The after-tax series reaches NEGATIVE EQUITY: the VDA tax (30% plus cess on gains, no loss offset) is subtracted from the pre-tax curve and exceeds what is left after a large gain followed by a crash (the base cell shows a net D-OOS return of +3407% with a -314% day). Everything computed from such a series (V1 seeds, V9 p95 drawdown, Sharpe, t-statistics) is an artefact, and returns around a zero crossing can be absurdly positive: a false-PASS path.

**Rule.** If a candidate series (D-OOS, full development window, holdout), one of its own cells, or a peer cell of its class contains a non-finite value or a daily return <= -100% (equity <= 0), every gate that consumes that series is NOT-EVALUATED with the reason `non-physical series: equity <= 0 (after-tax ruin or engine artefact)`, listing the series or cell ids and the first offending date (development dates only), and the overall verdict is INCOMPLETE: never PASS, never a silent KILL. A candidate-level or own-cell ruin blocks every gate except the three that read no return series (V2 sample counts, V10, IG2); a ruined peer cell of the class blocks only IG1, V8-DSR and V8-PBO. The pre-tax and tax-drag-only views of the same cell are NOT derivable from the stored inputs (cell files carry the after-tax series only); they are therefore not reported, and in no case gate.

**Calibration.** All 12 saved sweeps in the ledger's cell store were scanned: the 11 others have no return <= -100% (worst -0.316) and are unchanged by the guard. Only `821e20a6` is affected.

**Consequence, stated plainly.** The evaluation of `r2-xsmom-cr` is affected: with this guard its verdict is INCOMPLETE, so any kill recorded from the artefact series is NOT a clean kill. Other candidates' kills are unchanged.

**Open owner/protocol question (NOT decided here).** How the after-tax series should be defined for ruin cases: floor the return at -100% and stop (a ruined account), or let the account pay tax out of pocket (negative equity continuing). The guard only refuses to evaluate an artefact; it does not choose.

## What stays absolute

Every gate, threshold, cap, split boundary, grid and catalogue entry; the trial count; the ledger. Saved cell files are not touched. A re-evaluation of saved cells through the fixed evaluator repairs the regime closes (section 1); benchmark series stored in an `inputs.json` written before this addendum keep the USDINR conversion they were built with until they are rebuilt through `tq r2run`, and the report then lists the repairs (section 2).

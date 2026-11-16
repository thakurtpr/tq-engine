# Addendum 5 to `prereg-research-v2` (V10 "ledger complete": declared orphans)

Dated 2026-11-17. Written by the branch `hardening/r2fix4` on the orchestrator's ruling.

**Timing, stated plainly.** This addendum is written AFTER four candidates (`r2-trend-lc-sma`, `r2-trend-lc-don`, `r2-trend-lc-tsmom`, `r2-mr-lc-rsi2`) had been run, closed and killed in the real ledger. Those kills do not depend on V10: each of them also fails other gates (V4, VAL, V9 and others), none of them is a development GO, and none is changed by this addendum. **No development GO exists** at the time of writing. The decision is taken before any candidate has one and applies identically to all 14 candidates of the frozen catalogue; no candidate is singled out and no candidate's outcome was a reason for it.

**Finding.** The evaluator derived `ledger_complete = ledger.verify().is_ok() && ledger.unclosed_sweeps().is_empty()`. The research-v2 ledger contains sweeps that can never be closed normally: the master marker (OPEN by design, no evaluator) and sweep `20969dc0...` of `r2-trend-lc-sma`, opened with a stale binary (data_hash mismatch) before any cell ran. So V10 failed for EVERY candidate regardless of its returns and no development GO could ever exist. That is not what the tagged text means.

**Alignment with the tagged text.** The tag (V10, s.0 rules 2 and 5) and validation-protocol v3 rule 4 (the trials ledger records EVERY variant tried, including discarded ones) require that nothing is hidden: every trial recorded and counted in N. They do not require that every sweep was closed. The implementation is aligned with that definition; no gate, threshold, cap, split boundary, grid or catalogue entry changes.

## Rule

`ledger_complete` is true iff:
1. the ledger hash chain verifies; AND
2. every sweep that was OPENed and never CLOSEd is DECLARED, that is either (a) a master marker (`role_of` = Marker), or (b) listed in the committed, git-clean registry `research/declared-orphans.json`.

Registry schema: a JSON list of `{sweep_id, candidate_id, reason, no_cell_ran: true, addendum}`, unknown fields denied. For the registry to be honoured: the file must be committed and byte-identical to HEAD; every entry's sweep must exist in the ledger, be unclosed, belong to a candidate sweep of `candidate_id`, with `candidate_id` present in `research/caps.json`; the reason must be non-empty; `no_cell_ran` must be true; the named addendum must be a plain file name that exists in `research/` and is committed and clean; no cell file may be stored for the sweep; a sweep may be listed once. Any failure makes `ledger_complete` false for every candidate (a declared sweep that is closed or has stored cells is an inconsistency and is refused).

## What stays absolute

* **No hidden trial.** Any unclosed sweep that is not declared (a crashed run, a hidden sweep, a sweep absent from the ledger) still makes V10 fail.
* A declared orphan still counts in N (`n_new_final`, the cap usage, DSR) exactly as before; the registry changes only how V10 reads it.
* Broken hash chains, dirty or uncommitted registries, missing addenda and closed or cell-bearing declared sweeps fail.
* Every other V10 component (shift test, warm-up rule 0.8, single-touch holdout, kill switch OFF) and all other gates are untouched.

## Declared now, and later

The registry declares exactly one sweep now: `20969dc0d8da938825c90f5967b0950f303507cf9c5d45e83b28da4ca0416c42` (`r2-trend-lc-sma`, re-opened under `caps-addendum-2026-11-17.json`). The z-score sweep `feafa8ed...` of `r2-mr-lc-zscore` is NOT declared: after the cell-matching fix it is a valid open sweep that is run to completion and closed normally. Any further operational orphan (a sweep opened and abandoned before any cell ran, with a dated caps addendum) is declared the same way, by a committed registry entry, BEFORE the candidate's gate evaluation, and never after seeing its results.

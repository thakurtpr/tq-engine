# Addendum 3 to `prereg-research-v2` (IG2 prefix-invariance semantics)

Dated 2026-11-17. This addendum is dated **AFTER the first scored result was read**: the development sweep of `r2-trend-lc-sma` had already been run and closed, and its saved per-symbol bias verdicts (83 clean, 2 inconclusive, 0 look-ahead) were seen before this decision was taken. The orchestrator reports that candidate as already KILL on other gates; nothing in this document touches those gates. **No candidate has reached a development GO**: no DEV-GO exists, so the decision below cannot have been steered by a pass or a fail of any candidate on IG2.

**Scope.** Like addenda 1 and 2, this addendum changes **no gate, threshold, cap, split boundary, catalogue entry or spec K**. It does not edit `prereg-research-v2.md`, `research/caps.json` or any `prereg-<id>.md`. It aligns the *implementation* of one integrity flag with the *tagged text* (below) and is stated before any candidate can use it to reach a GO.

## 1. The tagged text and what the harness did

The tag `prereg-research-v2` defines IG2 as: "the harness passes prefix-invariance (`tq bias lookahead`-style), a byte-identical re-run, a content-level maximum-date check on every loaded bar, and the shift test."

`tq bias lookahead` exits non-zero **only on a look-ahead**. A symbol with too little history for the rule's warm-up (or too few signals) gets the verdict `Inconclusive`: no mismatch was found, but nothing meaningful could be exercised.

`r2run` (`crates/r2run/src/integrity.rs`) set `prefix_invariance = every per-symbol verdict is Clean`. That is **stricter than the tagged definition**, and it is unsatisfiable by construction for any IN-EQ candidate: a point-in-time NIFTY 50 universe contains symbols with fewer development bars than a 150 to 250 bar warm-up (for example ETERNAL, 111 development bars). Every such candidate would fail IG2 for a reason unrelated to look-ahead.

## 2. Decision (D12)

`prefix_invariance` is true if and only if ALL hold, evaluated over the candidate's pre-registered universe (one verdict per symbol, base cell):

1. **Zero bias verdicts.** No symbol has a `Lookahead` verdict, and none has `Unstable` (the other verdict `tq-bias` treats as a failure). **No tolerance: this requirement is absolute and is not diluted by the percentage below.** An unrecognised saved label counts as a bias verdict (fail closed).
2. **At least 90% of the universe is Clean** (named constant `MIN_CLEAN_PCT = 90` in `crates/r2run/src/ig2.rs`, integer arithmetic: `clean * 100 >= universe * 90`). A vacuous result (all symbols inconclusive, or an empty universe) therefore fails.
3. **Every Inconclusive symbol is listed in the saved artifact with its reason**: `history_shorter_than_warmup` (the symbol has no more bars than the rule's warm-up for the cell, so the rule can never signal on it) or `too_few_signals` (history exceeds the warm-up but fewer signals than the harness minimum occurred). The list is written to `suite.json` under `prefix_invariance.inconclusive` (symbol, reason, bar count) next to the counts.

Why 90%: a symbol that cannot be judged does not weaken a result on symbols that can, but a result that is mostly unjudged says nothing. 90% is stricter than the tagged text (which has no clean-share requirement) and is the smallest round share that admits the PIT universe's genuinely short-history symbols while keeping the check non-vacuous. It is fixed here, before any candidate can reach a GO, and is never tuned afterwards.

## 3. What this is not

* It is **not a relaxation of the "no look-ahead" requirement**: one Lookahead (or Unstable) verdict fails the flag even if 99% of the universe is Clean.
* It does not change `Inconclusive` handling anywhere else (the bias harness, CI gates).
* It applies **identically to all 14 runnable candidates** (`r2-trend-{lc,cr}-{sma,don,tsmom}`, `r2-xsmom-{lc,cr}`, `r2-mr-{lc,cr}-{rsi2,zscore,ibs}`); no candidate is treated differently.
* **It changes no existing verdict.** The first scored candidate's saved verdicts (83 clean, 2 inconclusive, 0 look-ahead) re-evaluate to a pass of this flag, but its outcome on the other gates is unchanged and no DEV-GO is created or implied.

## 4. Re-evaluating candidates already run

`ig2::recompute_from_saved` recomputes the flag from the saved `bias_verdicts` labels of a result (optionally with development bar counts and the cell warm-up to classify each inconclusive symbol), so an already-run candidate can be re-evaluated without re-running any cell. Applying it to a ledger or result file is a separate act of the orchestrator; the function writes nothing.

## 5. Limit of the per-symbol check (disclosure)

The per-symbol check drives only the signal stream of one symbol at a time. It does not exercise fills, sizing, the cost and tax stages, membership windows, alias mapping, coverage or the rebalancer's cross-sectional step. `tq r2run engine-prefix` (a diagnostic, not a gate; `docs/r2run-notes.md`) re-runs the whole engine on data truncated at cut dates and compares everything dated on or before the cut. It adds no gate and changes no threshold.

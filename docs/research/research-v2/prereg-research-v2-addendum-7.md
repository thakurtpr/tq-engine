# Addendum 7 to `prereg-research-v2` (NOT-RUN families and the multiple-testing derivation sets)

Dated 2026-11-17. Written by the branch `hardening/r2fix6` on the orchestrator's ruling.

**Timing, stated plainly.** This addendum is written AFTER all 14 runnable candidates of the frozen catalogue (`r2-trend-lc-sma`, `-don`, `-tsmom`, `r2-xsmom-lc`, `r2-mr-lc-rsi2`, `-zscore`, `-ibs`, `r2-trend-cr-sma`, `-don`, `-tsmom`, `r2-xsmom-cr`, `r2-mr-cr-rsi2`, `-zscore`, `-ibs`) had been run, closed and KILLED in the real ledger. **No kill depends on V8-DSR or IG1**: each candidate also fails other gates (V3 alpha t, V4 Sharpe, VAL, V9 drawdown and others, as recorded in `hardening/STATUS.md` and in each `research/r2run/results/<id>/gate-eval.txt`), and none of those verdicts is changed by this addendum. **No development GO exists** at the time of writing. The decision is taken before any candidate has one and applies **identically to all 14 candidates**; no candidate is singled out and no candidate's outcome was a reason for it.

**Finding.** The evaluator derived the deflation/variance set (V8-DSR), the PBO cell matrix (V8-PBO), the IG1 model universe and N_new_final from EVERY GO candidate in `research/caps.json` and refused (NOT-EVALUATED, "candidates without a closed sweep") unless every one had a closed sweep. Four GO candidates are NOT-RUN by design (addendum 2 D4: `r2-pairs-lc`, `r2-pairs-cr`, `r2-intraday-nse-orb`, `r2-intraday-nse-vwap`, status NOT-RUN in `research/r2run/catalogue-status.json`) and can never close a sweep. So V8-DSR, V8-PBO and IG1 were NOT-EVALUATED for every candidate and no development GO could ever exist. That is not what the tagged text means.

**Alignment with addendum 2 D4.** D4 states that a family that cannot run is NOT-RUN: it stays in the catalogue and consumes NO trial cells. The derivation sets are therefore the sets of candidates that CAN run. The tagged `N_true = N_new_final + L_total` is untouched: N_new_final still counts every cell of every sweep in the ledger (a NOT-RUN family contributes none, because it has no sweep) and `L_total` stays pinned. No gate, threshold, cap, split boundary, grid or catalogue entry changes; nothing is loosened.

## Rule

1. `research/r2run/catalogue-status.json` is read by the evaluator. The candidates (per `research/caps.json`) of every family whose status is `NOT-RUN` form the NOT-RUN set. The file must be committed and byte-identical to HEAD (the same mechanism as the declared-orphans registry of addendum 5). An ABSENT file means an empty set (the strict behaviour before this addendum). A present file that is uncommitted, dirty, malformed, has an unknown status or lists a family twice makes the derivation fail closed (every bound gate NOT-EVALUATED).
2. The deflation/variance set, the PBO matrix and the IG1 universe (all three uses of the GO-candidate list) exclude the NOT-RUN candidates.
3. **Hard cross-check (anti-gaming).** If the ledger holds ANY sweep, open or closed, whose role is `Candidate(<id>)` of a NOT-RUN candidate, the derivation refuses: NOT-EVALUATED with the reason "a candidate declared NOT-RUN has a sweep in the ledger". A candidate cannot be declared NOT-RUN to leave the sets after it was run.
4. **The evaluate-after-Phase-1 safeguard stays.** A RUNNABLE candidate without a closed sweep blocks exactly as before ("candidates without a closed sweep"). N_new_final is derivable only after every runnable candidate is closed (or is a declared orphan, addendum 5), so early evaluation gives no advantage.
5. The report lists the excluded candidates in its self-supplied block: "derivation sets exclude NOT-RUN candidates (addendum 7): ...".

## Effect (read-only run on the committed ledger and catalogue, no cell re-run, no trial added)

All 14 candidates now derive the class-cell variance set, the PBO matrix and the IG1 universe (81 cells per class), with N_new_final 173 and N_true 537 (L_total 364). Sweep `20969dc0...` stays a declared orphan (addendum 5) and the master marker stays open by design. Both counted in N as before.

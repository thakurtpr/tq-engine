# research-v2 governance harness (tq-r2gov): notes, ambiguities, gaps

Crate `crates/r2gov` (tq-r2gov) and `tq r2 ...` (crates/cli/src/r2_cmd.rs). No prereg, addendum or threshold was changed. Spec ambiguities are resolved here, not in the prereg.

## Ambiguities and how the harness resolves them (reviewer: confirm or overrule)
1. **Tag and file naming.** Template says tag `prereg-r2-<id>` with candidate id `r2-...`. Taken literally that is `prereg-r2-r2-...`. Resolved: tag = `prereg-` + candidate id (`prereg-r2-trend-lc-sma`), file = `docs/prereg-<id>.md`.
2. **Per-family vs per-candidate caps.** Master 6.1 gives family caps equal to the cell counts (a family such as r2-trend-lc is 3 books x 9). `research/caps.json` carries family caps plus a frozen K per candidate (the book); an open must match K exactly. Because K is frozen and each candidate opens once, family and group caps cannot be exceeded through the harness; the binding caps in practice are the total (450) and the reserve (132) when sweeps were written around the harness or re-run.
3. **What draws on the reserve.** Master markers, class holdout-touch sweeps and re-runs. 4.7 says reserve cells are released by a dated addendum; the existing two markers were opened without one. Resolved: markers and touches draw on the reserve up to its cap with no addendum; a RE-RUN (second sweep of a candidate id) needs a committed `research/caps-addendum-YYYY-MM-DD.json` releasing >= its cells. An addendum can never raise a cap.
4. **Classification of sweeps.** The ledger has one family (`research-v2`), so candidate / touch / marker are told apart by single-valued manifest axes added by the harness (`r2-candidate`, `r2-holdout-touch`, `r2-batch`, `master`). They are hash-chained, and do not change N. Sweeps written around the harness are counted as unclassified reserve and in the total.
5. **`--holdout-ack=<candidate-id>` vs one class batch.** 5.2 says a single ack names a candidate but the touch is one sealed batch per class. Resolved: `--holdout-ack` takes the comma-separated ids of the WHOLE class batch; any later ack for that class, with any id, is refused (no new-id retry).
6. **"Descendant".** Same candidate, same family, same `rule_family` (caps.json) as a burned candidate, or a `parent_strategy_hash` chain reaching a burned candidate's strategy_hash. `rule_family` assignments in caps.json are mine (trend, xsmom, mr, pairs, intraday, volt, carry, options, council).
7. **Committing from code.** The guard does NOT run `git commit`. It writes ledger + anchors + marker (fsync, atomic) and `unseal` refuses until those paths are committed and clean at HEAD.
8. **Dev-GO record.** 5.2(ii) names "a development-GO record" without a format. Resolved: committed `research/dev-go/<id>.json` `{candidate_id, sweep_id, verdict:"GO"}` naming a CLOSED sweep of that candidate. Who writes it is the coordinator's call.
9. **Classes with no GO-capable families.** In caps.json only IN-EQ and CRYPTO have GO families; IN-IDX and IN-OPT have only diagnostics, which can never reach a holdout. The guard is class-general, but IN-IDX/IN-OPT acks will always be refused as `not_go_capable`.
10. **Post-dev-end rows of a class without a holdout (CRYPTO)** are sealed too (nobody can unseal them: no historical holdout).
11. **Yahoo `close` vs `adjclose` for non-index series** is not specified by the addendum (it says only close for NSE indices). `normalise_yahoo` takes the column as an explicit argument; the CLI uses `close`.

## Gaps (not done, or only partly)
* Run-after-register evidence beyond the open gate (10.2 "every result file carries the ledger tip", "ledger-open commit strictly precedes any result file in git log") is not enforced: `tq r2 open-sweep` prints the tip and the files to commit, but nothing checks result-file commits.
* The open gate checks the harness paths are clean and `code_git_sha` is an ancestor of HEAD; it cannot prove the running binary was built from that sha.
* A coordinator-only convention remains: nothing stops a process from writing the ledger with the raw honesty API (the existing anchor/extend checks still catch rewrites; caps count such sweeps as unclassified reserve and total, so they cannot be hidden, only refused afterwards).
* Sealing is file permissions (mode 000) outside the repo: a speed bump for agents, not an OS boundary (same user can chmod). The only strategy-facing API is `DevReader`; no reader takes a raw path.
* `SealedStore::read_full` returns merged CSV text; a typed holdout bar loader needs a public CSV parser in tq-data (not changed here).
* Addendum rules implemented as library functions with tests (`datarules`): close-not-adjclose, zero-open and inconsistent-OHLC fill rule, 95% coverage on full window AND D-OOS, manifest first dates. They are NOT yet wired into the backtest engine fill path (no engine code was touched); the benchmark engine inside the sealed process (addendum rule 2) and the adjustment-level leak rule (rule 3) are not implemented.
* Prereg 10 items 6-8 (v3 statistics: HAC fix, N_true DSR, SPA, N_eff; IG2 tests; independent review) are the OTHER half of the harness, out of scope here.
* The audit log lives at research/r2-audit.jsonl and is not auto-committed.
* Ledger/anchor/mirror and trials.csv commits after open-sweep are manual by design.

# r2fix: fixes for the independent harness review (docs/review-harness-r2.md, branch hardening/r2review c698fbf)

Branch `hardening/r2fix` off `hardening/integration` (f57a780). Two parts: governance (`tq-r2gov`, `tq r2`, one
additive `tq-backtest` module; full detail in `docs/r2fix-gov-notes.md`) and statistics (`tq-r2stats`, this file).
No prereg, addendum, tag, threshold or cap was edited; no backtest, real sweep, real sealed store or `data-raw`
mutation was run (every ledger/raw-cut test uses tempdirs).

Test counts: tq-r2stats lib 184 -> 230; tq-r2gov lib 78 -> 138; tq-backtest `honesty` filter 93 -> 96; tq-cli 78.

## Finding status

| Finding | Status | Where |
| --- | --- | --- |
| F-C1 evaluator not bound to ledger | fixed, with named residuals | `bind.rs`, `bind_repo.rs`, `evaluate.rs` |
| F-H1 trust anchors | fixed | gov notes |
| F-H2 sealing / raw data | partially (procedural + defence in depth; same-user adversary can still read the key) | gov notes |
| F-H3 V3 lowest-t | fixed (departure from v3 documented below) | `gates_edge::lowest_t` |
| F-H4 missing inputs make gates easier | fixed | `inputs::Window::check_exposure`, `neff`, `gates_edge::og3`, `gates_og::og2` |
| F-H5 dev-GO and close | fixed | gov notes; `GateReport.input_sha256` |
| F-H6 close vs adjclose | fixed (splits after dev end partial) | gov notes |
| F-M1 holiday weeks | fixed | `weekly::weekly_returns_from_daily` |
| F-M2 windowed rho_bar | fixed | `evaluate::universe_in` |
| F-M3 dev-stage verdict | fixed (`Verdict::DevGo`) | `gate::overall_staged` |
| F-M4 holdout batch protocol | (a),(b) fixed, (c) partially | gov notes |
| F-M5 repo-level lock | fixed | gov notes |
| F-M6 series-to-class allowlist | fixed (crypto B1/B3 leak accepted per addendum s.4) | gov notes |
| F-M7 DSR inputs and `dsr` subcommand | fixed | ledger-derived variance set; `dsr` prints no verdict |
| F-M8 V9 horizon | fixed | `McConfig::for_class` (730 for CRYPTO) |
| F-L1 USDINR filter | fixed | `weekly::filter_usdinr` re-anchors after 3 consistent rejections |
| F-L2, L3, L4, L5, L6, L7 | fixed | gov notes |
| F-L8 V5 benchmark start; dev exposure | not fixed | V5 still uses B1's series from the first invested day (small gap, documented); the dev-window exposure is still the D-OOS mean (only used for the V3-dev classification) |
| F-L9 V11 omits OG1-OG6 | partially | `paper-gate` now prints `V11-MET ... NOT yet PASSED` and names OG1-OG6; the OG paper-window evaluation is not built |
| F-L10 Sharpe annualisation 365 for crypto | not fixed | sqrt(252) everywhere is the stricter convention (A-V5-1); threading a per-class factor through V4/V5/V6/OG6 is not cheap. Addendum s.4 says 365: owner decision (an addendum can allow it; the harness must not loosen on its own) |
| survivors 1-8 | killed | `r2fix_tests.rs` (each verified to fail under its mutation) |
| survivors 9-14 | killed / equivalent | gov notes |

## F-C1: what the evaluator now derives (`bind::derive`, `bind_repo::derive_from_repo`)

`tq r2stats gate-eval --input c.json --repo <checkout>` loads, read-only: the verified ledger, `research/caps.json`
(sha256 must equal `tq_r2gov::gate::CAPS_SHA256`), the stored cells `research/cells/<sweep_id>/<index:05>.f64`,
the class development end from the pinned master prereg, `research/holdout/<class>.touch.json` and
`research/integrity/<candidate>.json`. From these it derives, and the caller's JSON values are IGNORED for:

* **N_new_final** = `ledger.family_trials(caps.ledger_family)`, only when every GO candidate in caps has a CLOSED sweep, the ledger
  verifies and no sweep is unclosed ("after the last cell", prereg 4.2); otherwise V8-DSR is NOT-EVALUATED.
* **K / cell counts**: the candidate's stored cell count must equal the caps count; cell lists are built from the ledger, so a
  cell list of the wrong length cannot exist. PBO matrix = cells of the candidate's caps family (applicable iff >= 20, derived);
  IG1 universe and the class-cell variance set (daily and monthly Sharpes) = every cell of every GO candidate of the class;
  pooled daily variance = `ledger.family_sr_var`. `all_cells_profitable`, ring-1 neighbours (one axis, one step), base Sharpe and
  candidate net return come from the candidate's own stored cells and the manifest axes.
* **Merkle check**: every sweep used is re-hashed with `close_record` and must reproduce the ledger's CLOSE merkle root and
  `sr_var` bits; a tampered, missing or reordered cell makes everything that depends on it NOT-EVALUATED.
* **Base cell**: the candidate's D-OOS series must equal (exactly) one stored cell of its own sweep; that cell is the IG1 candidate.
* **K_H**: the batch length of the class touch marker (the caller's `holdout_peer_p_values` is ignored). Peer p-values are sealed
  until the batch ends, so IG3 passes iff own p <= 0.05 / K_H (passes under Holm whatever the peers did), fails iff own p > 0.05,
  and is NOT-EVALUATED in between ("Holm needs the peers' p-values"). This is stricter than Holm's full step-down.
* **Integrity flags**: `ledger_complete` (ledger verifies, no unclosed sweep), `holdout_single_touch` (at most one touch sweep
  for the class), `max_date_check` (D-OOS last date <= class dev end from the pinned master) are computed. The other flags
  (`prefix_invariance`, `byte_identical_rerun`, `warmup_ok`, `kill_switch_off_assessment`, shift-test Sharpes) can only come from
  an `IntegrityArtifact` JSON written by the harness runner (r2run), which counts only if its `candidate_id`, `sweep_id` and
  `merkle_root` equal this candidate's sweep and its CLOSE root. With no artifact V10 and IG2 are NOT-EVALUATED, so no DEV-GO is possible.

**Contract for r2run** (cells and artifact): the stored cell series of a sweep are the stitched D-OOS daily net returns of each cell,
**all of the same length as, and dated like, the candidate D-OOS window** (undated f64 files cannot be dated otherwise: a cell
of another length makes the class-level gates NOT-EVALUATED). The integrity artifact schema is `bind::IntegrityArtifact`.

**Residual (self-supplied) inputs.** The report lists them in `self_supplied` and the text output prints them: benchmarks
B1/B2/B3 and the class benchmark closes (not outputs of `bench::run` on the stores; the sealed benchmark engine is not built),
trade books and `universe_returns`, V1 seed outcomes, V7 stress/folds/random-entry, `eligible_symbols`, `full_dev`, the per-trade
clustered t. These are still the caller's word; a weak benchmark still passes OG2/OG3/V5/OG6. Not fixed (needs a bench artifact
pipeline); do not call a DEV-GO evidence until those are produced by the runner.

## Other statistics fixes

* **F-H3.** `gates_edge::lowest_t`: gate t = min(HAC, monthly, clustered), for V3 4c-x and for the 4c-rel monthly alpha t. This is the
  TAGGED prereg text ("the lowest of the HAC, monthly and clustered t governs", rule 0.1: the stricter text wins). It differs from
  v3 (`tq_backtest::tstats::judge`, "lowest only when a cross-check is more than 0.5 below the HAC t"), which is left UNAMENDED and
  still used by the legacy paths and `validation` (VAL). Concretely a book with HAC t 3.01 and monthly t 2.99 passed under v3 and fails now.
* **F-H4.** Exposure must be present, the same length as returns and in [0, 1]: otherwise V3, V5, OG6, OG4b are NOT-EVALUATED (no panic).
  `rho_bar` falls back to 1 when fewer than two series (or no qualifying pair) are supplied, so a missing universe cannot lower DE.
  OG3 with B1 absent is NOT-EVALUATED (the B1 clause is dropped only when B1 provably has no date in the window); OG2 with a benchmark
  absent is NOT-EVALUATED, NOT-APPLICABLE only when the benchmark's own range has no overlap with the window.
* **F-M1.** A week ending before its Friday is kept when a later week follows (Thursday's close ends it); only the last group must end on its
  Friday, the first group must start on the Monday (a Tuesday start after a Monday holiday at the very start of a series is still dropped).
* **F-M2.** `universe_in`: rho_bar uses the universe sliced to each window's own dates (dev, D-OOS, holdout).
* **F-M3.** `Verdict::DevGo` (serialised `DevGo`) at the Development stage of a class with a clean holdout; `Promote` only at the Holdout
  stage or for CRYPTO. The old `evaluate` test that asserted Promote for an IN-EQ development candidate was rewritten.
* **F-M7.** `dsr` prints `FLOOR VARIANCE ONLY (indicative, not a gate)` and never PASS/FAIL.
* **F-M8.** `McConfig.horizon_days` (504 default, 730 for CRYPTO through `for_class`). The V9 seed 20261118 is still author-chosen (A-V9-1):
  record it in every candidate prereg.

## API changes (tq-r2stats)

Additive unless noted.
* NEW modules `bind` (`Binding`, `Ev`, `Sources`, `derive`, `GridFacts`, `DerivedIntegrity`, `IntegrityArtifact`, `CellReader`),
  `bind_repo` (`derive_from_repo`, `RepoPins`), `evaluate_render` (`render_text` is re-exported from `evaluate`).
* NEW `evaluate::evaluate_with(inp, &Binding, cfg)`. **BEHAVIOUR CHANGE:** `evaluate(inp, cfg)` now equals `evaluate_with` with an
  unbound `Binding`: V8-DSR, V8-PBO, IG1, V10, IG2 are NOT-EVALUATED and the verdict can never be DevGo or Promote.
  The `CandidateInputs` fields `n_new_final`, `variance_set`, `pbo_cells`, `ig1`, `integrity`, `holdout_peer_p_values` and the
  grid part of `robustness` are kept for JSON compatibility but are IGNORED by the evaluator.
* `GateReport` gains `input_sha256: Option<String>` and `self_supplied: Vec<String>` (serde default). The dev-GO record binds to the
  sha256 of the report JSON and its `input_sha256` (see gov notes).
* `gate::Verdict` gains `DevGo` (exhaustive matches downstream must add the arm); NEW `gate::overall_staged`.
* `Window::check_exposure`; `AssetClass::caps_name`.
* `gates_og::integrity_flags` + `IntegrityFlags` (tri-state); `gates_og::integrity` kept as a wrapper over typed-in booleans, NOT used by the evaluator.
* `gates_stat::ig3_bound` (K_H from the marker); `gates_stat::ig3` kept (caller peers), not used by the evaluator.
* `gates_edge::lowest_t`. `neff::rho_bar` now returns rho_bar = 1 (`fallback_to_one`) for fewer than 2 series (was 0).
* `og3` with `None`, `og2` with `None`, `og6`/`v5`/`v3` with an empty exposure: NOT-EVALUATED (were Pass / NotApplicable / treated as invested).
* `McConfig` gains `horizon_days` (construct with `..McConfig::prereg()`); `McConfig::for_class`; `og2::{lower_bound_rank, decide}` (crate-private).
* CLI `tq r2stats gate-eval` gains `--repo <checkout>`; without it nothing multiple-testing is evaluated and the output says
  `NO LEDGER EVIDENCE`. `tq r2stats paper-gate` output `PASSED-ELIGIBLE` is now `V11-MET ... NOT yet PASSED`.
* `Cargo.toml`: `tq-r2stats` now depends on `tq-r2gov` (and dev-depends on `tempfile`).

## TDD evidence (honest)

Real runtime REDs (assertion failures against stubs of the new functions or the old behaviour) were observed for: all twelve
`bind` derivation tests, the integrity-artifact binding, P1, P2, P3, P4, the OG2 absent benchmark, P6, P7, F-H3 (both tests), F-M8,
the exposure check, and the four CLI tests. The evaluator-level tests added after the implementation (`n_true_comes_from_the_binding...`,
DevGo, exposure at evaluator level, report hash, `universe_in`) were written green-first; the DevGo test was verified to fail when the
staging is reverted. Survivor tests 1-8 pass on the correct code by design; each was verified to FAIL under its reviewer mutation
(MIN_INVESTED_SHARE 0.10, V7 70% -> 50%, FLAT_EXPOSURE 0.90, V5 Calmar clause removed, p95 -> p50, lower-bound rank +100, `pass` ignoring
the strict inequality, VDA net basis). Not re-run under a mutation tool: survivors 9, 11, 12, 13, 14 (r2gov, see gov notes).

## Residual risks

* Benchmarks, books and V7/V1 runner outputs are still self-supplied (listed in every report). A DEV-GO from this evaluator is evidence only
  to the degree those inputs were produced by the reviewed runner; the walk-forward machinery (r2run) must write the cells and the integrity artifact.
* N_new_final needs every GO candidate closed: no candidate can be evaluated before the whole GO catalogue has been run (prereg 4.2 "after the
  last cell"); an abandoned candidate blocks all others until the owner records a dated addendum or runs it.
* IG3 stays NOT-EVALUATED for p between 0.05 / K_H and 0.05 until peer p-values can be released after `finish_batch` and verified.
* Crypto B1/B3 sealed-benchmark leak (addendum s.4) accepted; Sharpe annualisation for crypto stays sqrt(252) (F-L10).
* The integrity artifact and `research/integrity/*.json` are plain repo files: anyone who can commit can write one (procedural control,
  bound to the CLOSE root so it cannot be reused for a different sweep).
* A same-user process can read the sealed-store key (gov notes): the OS is the real boundary.

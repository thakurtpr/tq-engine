# State
Updated: 2026-11-18 (orchestrator)

## Right now
T-002 (D-006) options-v2, pre-freeze. Groundwork merged and verified in hardening/integration d96785a (clippy rc=0, 2191 tests, 14 research-v2 results byte-identical). Protocol DRAFT amended after skeptic review; NOT tagged. Next: engine real-data fixes + harness items, synthetic dry run, then freeze + tag.

## Done (latest 10)
- Data pulled and scanned (NSE F&O EOD 2012-2026, indices, VIX, benchmarks repaired); capability catalogue; cost/margin/tax crate (tq-opt2cost); options engine core (tq-opt2, 21 structures, no-look-ahead harness).
- Study profile generalises the r2 harness (options_v2 pins UNSET until freeze); research-v2 re-evaluation byte-identical.
- Skeptic review (FIX THEN FREEZE) applied: G9-G11, null family, frozen hurdles, type A/B rule, one sealed batch, fixed-notional sizing. Calibration: iid-loss nulls <= 3.1% family-wise; persistent-regime null NOT controlled (4-21%); power at Sharpe 1/2/3 = 3-7% / 25-56% / 61-86%.
- prereg-options-v2.md DRAFT sha256 77e137a8...ce565c4 (no placeholders, untagged).
- research-v2 CLOSED: all 14 killed; default stays Nifty ETF + BTC.

## In progress
- Wave 2 agents (engine real-data fixes; harness evaluator + dry run) about to launch.

## Next (ordered)
1. Engine: mixed-lot fail-closed per contract, index-close keyed on file_date, real-data smoke load (no statistics printed).
2. Harness: staged options evaluator E1-E3 + G7-G11, idle-cash 6.5% test, planted-zero expiry-settle test, o2bench wiring, forged-input controls, synthetic end-to-end dry run.
3. Freeze: final hash, annotated tag prereg-options-v2, register ledger/caps/writer; then strategy agents per family (K <= 60), Phase 1 then Phase 2, sealed slice only for development-GO.

## Blockers and open questions
- Owner items (non-blocking): D-005 gates near-infeasible, after-tax series when tax > account, 30% kill-switch assumption, key backup on same disk, real Nifty/BTC split unknown.
- Orchestrator ruling: no gate added/cut to chase the regime null; claim stays conditional; promotion = forward paper only.

## Failed approaches (do not retry)
- Free end-of-day data produced no edge: intraday, momentum rotation, basis carry, council, defined-risk options (v1) all killed or inconclusive. Do not tune or revisit.
- Shared-target rlib collision between worktrees (E0463/stale binaries): use CARGO_PROFILE_DEV_DEBUG=1 per worktree and cargo clean -p after.

# research-v2 run framework (tq-r2run): workflow, decisions, harness findings, gaps

Crate `crates/r2run` (tq-r2run), CLI `tq r2run ...`, branch `hardening/r2run` off `hardening/integration`.
Backtest only. No prereg, addendum, threshold, cap or harness-crate semantics was changed. Nothing was run on real
data, no sweep was opened in the real ledger, the data step was NOT run on `data-raw`.

## Strategy-author workflow (exact commands)

0. Once, by the coordinator (not yet run). The sealed store is AES-256-GCM (`.enc` files) with a 0400 key file outside the repo and
   data-raw (create it on the first run, back it up separately):
   `tq r2run data-step --repo . --raw-root /Users/thakur/Workspace/Trading/data-raw --sealed-root /Users/thakur/Workspace/Trading/data-sealed --key-file <KEYFILE> --create-key --exclude ADANIENT`
   (`--exclude ADANIENT` is the orchestrator ruling: its adjclose is negative on 47 development rows, so total-return ingestion refuses it; it was not
   a NIFTY 50 member before 2022, so IN-EQ development is unaffected; it is the default of `--exclude`). Yahoo equities and ETFs are ingested on the
   total-return basis (`normalise_yahoo_total_return`, addendum 2 rule 3), NSE indices, VIX and FX on `close`; the series-to-class allowlist applies
   to every raw stem; each sealed copy is verified by decrypt-and-hash; after ALL classes are done the consumed raw compact files are cut to the
   development end (the LARGEST end among the classes that read a file; all-or-nothing, atomic per file). Use `--no-cut` for a dry look. Prints only
   series and development-row counts, writes `docs/data/r2-dev-manifest-<CLASS>.csv`, and checks that the sealed side is ciphertext only. Commit the
   manifests before any candidate is registered. Strategy code only reads `data-sealed/dev/<CLASS>` through `DevReader` (`DevData::load`).
   Naming in the dev store is listed in `datastep.rs` (raw stems where the harness allows: `ACC`, `NIFTYBEES.NS`, `NSEI`; `NSE-NIFTY50`, `BTCUSDT`,
   `BTCUSDT-PERP`, `FUND-BTCUSDT`, `USDINR`, `BTC-USD`, `MEMBERS`). The allowlist keeps ETFs and indices out of IN-EQ and FX and crypto out of the Indian
   classes, so a spec reads such series from the other class store with `aux.ext` (cut at its own development end), e.g.
   `"ext": [{"class":"IN-IDX","series":"NIFTYBEES.NS"}, {"class":"CRYPTO","series":"BTCUSDT"}, {"class":"CRYPTO","series":"USDINR"}]`.
1. Write the prereg `docs/prereg-r2-<id>.md` from the template; get it tagged `prereg-r2-<id>` (coordinator).
2. Write the rule (the only code a family writes): implement `tq_r2run::rule::Rule` (`build(&BuildCtx) -> Built`,
   `Signals(Box<dyn Strategy>)` for daily signals or `Rebalance(Box<dyn Rebalancer>)` for month-end weights; optional
   `warmup_bars`) in `crates/r2run/src/rules/<family>.rs` and register it in `rule::standard_registry`. Rules see
   closed bars only, one parameter vector for every instrument, no per-instrument tuning.
3. Write the spec JSON (copy `docs/r2run-spec-TEMPLATE.json`): id, class, universe, grid axes (ordered, >= 3 levels,
   base at the centre), base cell, seed, sizing, `aux` series. The cell count must equal the frozen K in `research/caps.json`.
4. `tq r2run plan --spec spec.json --store /Users/thakur/Workspace/Trading/data-sealed --write-axes`
   prints the grid, ring-1 neighbours, walk-forward folds (purge, embargo), the data manifest hash, the cost model hash,
   whether the sweep is open, and otherwise the exact `tq r2 --repo . open-sweep --candidate <id> --data-hash ...` command.
   `--write-axes` writes `research/r2run/<id>.axes.json` for `--axes-json`.
5. The single ledger writer (registered once with `tq r2 register-writer`, from the coordinator worktree and branch) runs that printed command in
   the main worktree and commits ledger, anchors, mirror and `trials.csv` BEFORE any result file.
6. `tq r2run run-sweep --spec spec.json --store /Users/thakur/Workspace/Trading/data-sealed [--window dev]`
   refuses unless the sweep is open, unclosed and bound to this grid, data hash, cost hash and the tagged prereg hash; refuses `--window holdout` unless
   the harness guard unseals the class (and even then reads no holdout bar); refuses dirty harness paths. It runs every cell IN THE LEDGER'S CELL ORDER
   (`enumerate_cells` sorts axes by name and values as strings), the robustness suite and benchmarks, and writes `research/r2run/results/<id>/`:
   `cells-f64/<index:05>.f64` (each cell's stitched D-OOS daily NET returns, one length, the candidate's D-OOS dates), `cells/*.json`, `suite.json`,
   `inputs.json` (`CandidateInputs`; the evaluator now ignores its self-supplied n_new_final, variance_set, pbo_cells, ig1, integrity, peers and grid
   robustness), `integrity.json`, plus `research/integrity/<id>.json` (`bind::IntegrityArtifact`: candidate, sweep id, the CLOSE merkle root computed
   with `close_record`). Exposure is clamped to [0, 1] and present for every window. It prints the exact follow-ups:
   `tq r2 --repo . close-sweep --sweep-id <S> --cells-dir research/r2run/results/<id>/cells-f64` (the registrar then stores
   `research/cells/<S>/<index:05>.f64` itself and refuses to overwrite: that is why the cells are staged in `cells-f64`, not written there by run-sweep),
   then `tq r2stats gate-eval --input .../inputs.json --repo .` (commit `research/integrity/<id>.json` first). Flags: `--priority-seeds 20 --random-draws 1000`.
7. Every run is appended to `research/r2run-audit.jsonl` (candidate id, cell id, code sha, HEAD, data manifest hash,
   cost model hash, sweep id, ledger tip, returns hash, outcome); `tq r2run audit-tail --n 20`.
8. Peer cells: the evaluator derives the IG1 universe and the DSR class variance from the ledger and the stored cells of every closed GO candidate
   of the class, so no run needs the others' files; N_new_final exists only once every GO candidate is closed (prereg 4.2). r2run still reads
   `research/r2run/results/*/cells/*.json` of other candidates for its own `inputs.json` (informational).

## What the framework does, mapped to the task

| Task item | Where |
| --- | --- |
| (1) data step, sealed store outside repo and data-raw | `datastep.rs`, `tq r2run data-step` (class recipes, naming, tested on a synthetic tree) |
| (2) DevReader loading, addendum rules | `data.rs` (content-level date check twice, lenient rows kept and counted, member-day coverage on full window, D-OOS and per year, 95% rule), `datastep.rs` (close for indices, adjclose for tradables, funding timestamps), engine `fill_trust` (zero-open and inconsistent-OHLC bars cannot fill, orders move to the next usable bar, stops on such a bar use the close; both engine paths), `funding.rs` (Binance UTC day), weekly Friday alignment is in tq-r2stats |
| (3) walk-forward as the prereg specifies | `folds.rs` (first test per class, 6m/3m, purge max(hold,10; 21 monthly; option cycle), embargo 21 td / 7 d, positions carry: ONE continuous run cut into test segments), `cellrun.rs` (stitched D-OOS, fold outcomes, k_bar input = open positions per day, trades by exit date, tax via `TaxModel` with open positions taxed as if sold, slippage per class, perp funding inside every run variant, idle cash 0) |
| (4) robustness suite | `robust.rs` (grid, ring-1, 2x slippage and 2x charges, >= 20 priority seeds, kill-switch-ON operating run), `baselines.rs` (seeded random-entry p95, shift test), `integrity.rs` (prefix invariance through `tq-bias` StreamSource/RebalanceSource, byte-identical re-run, max-date, warm-up) |
| (5) B1/B2/B3 and CandidateInputs | `bench.rs` (via `tq_r2stats::bench`), `assemble.rs`, `sweep.rs` |
| (6) registration glue | `guard.rs` (open-sweep requirement and command, K, hashes, holdout refusal) |
| (7) determinism and audit | `sweep.rs` result files, `audit.rs`, tests |
| toy strategy | `toy.rs`, feature `toy`, tests only |

## Decisions (ambiguities resolved here, not in the prereg; reviewer: confirm or overrule)

1. Walk-forward with fixed-parameter rules is one continuous run over the whole development window starting at the
   class train start (or first bar); folds select which days are scored (L5). The "full D from the first invested day"
   series (V3-dev) is the same run. Rules that fit something per fold get the `FoldPlan` (train ranges minus embargo) in `BuildCtx`.
2. Yahoo equities and tradable ETFs are ingested with `adjclose` (total-return basis, prereg 7.1); NSE index and VIX
   series with `close` (addendum 5). r2gov note 11 says the CLI uses `close` for non-indices; the data step overrides that.
3. 2x-cost stress doubles every pct charge, the DP charge and the brokerage cap (not GST) and the slippage; the engine's
   flat or ADV slippage has one bps value per run, so per-symbol ETF values (5 / 8 bps) are the spec's single `slippage_bps`.
4. Random-entry baseline: entry probability = own entries / symbol-bars, holding = median holding in bars, stop = median
   stop distance, seeded per (seed, symbol, bar), same engine, costs and tax; compared on stitched D-OOS net return.
5. Ring-1 and the grid are numeric axes only (categorical books are separate candidates).
6. `V2-oos` counts trades by exit date inside a test segment; open positions at the window end are taxed as sold but
   are not closed trades. Trade P&L is net of costs and slippage, before tax.
7. DSR per-period Sharpe inputs are daily mean/sd (not annualised); pooled variance is over every supplied cell of any class.
8. Kill-switch OFF = drawdown limit 100%; the engine's 2% daily-loss limit (RiskLimits default) is left as is.
9. A second scored run of the same sweep id is allowed (the IG2 byte-identical re-run) while code and data hashes match;
   a bug re-run is a new sweep through the harness (caps addendum), as r2gov enforces.

## Harness findings, re-checked against the fixed harness (hardening/integration 2b39e76)

* H-1 STANDS. `DevReader::bars` still goes through `CsvDaily` (`Bar::new` rejects zero-open and non-bracketing rows that addendum 5 says are kept and
  counted: BANKBEES, USDINR, HDFCBANK, TMPV ...). Workaround: `data::parse_lenient` over `load_csv_text` (date check first). Proposed patch (r2gov
  `sealed.rs`, `DevReader::bars`): build `Bar { .. }` literals from `load_csv_text` with a strictly-increasing-ts check instead of `CsvDaily`, or export a
  lenient parser; then r2run can drop its copy.
* H-2 PARTLY FIXED (key, `.enc`, allowlist, exclude, total-return, RFC 3339 dates, raw cut), but `tq r2 seal-ingest` still cannot ingest the real compact tree:
  (a) `ingest_selected` runs `classmap::check` over EVERY file name of the directory before `exclude` is applied, and the compact `yahoo/` directory mixes
  indices, ETFs, FX, crypto proxies and foreign series, so any class ingest of that directory fails (`ClassMismatch`) whatever `--exclude` says; (b) `valid_symbol`
  rejects `=` so `USDINR=X` (and `CL=F`) can never be a store name; (c) epoch-millisecond first columns (Binance funding) are not dates for `row_date`; (d) the
  NSE archive `INDIAVIX` and the Yahoo `INDIAVIX` share a stem. Proposed patch: apply `exclude` (and an `include` list) before the allowlist check, accept an
  alias for `=` names, and accept epoch-ms in `row_date`. r2run works around this by checking `classmap::check` per raw stem itself, converting funding to ISO,
  renaming (`USDINR`, `NSE-<stem>`) and calling `SealedStore::ingest` + `verify_sealed` directly, with its own raw cut (`datastep::cut_raw`); the coordinator
  runs `tq r2run data-step`, not `tq r2 seal-ingest`.
* H-3 STANDS (`windows::parse_master` returns only the development end; `splits.rs` parses the validation start).
* H-4 FIXED (`universe_in` windows rho_bar per window).
* H-5 PARTLY: sweeps now record the master and candidate tag object ids, but `data_hash` and `cost_model_hash` are still opaque strings to the registrar;
  `guard::require_open_sweep` still binds them to the run's own hashes.
* H-6 FIXED (code sha must equal HEAD for harness paths, caps and master pinned); r2run keeps its dirty-harness refusal and records both shas.
* H-7 STANDS (ledger, anchor and `trials.csv` commit ordering is manual).
* NEW H-8. The allowlist forbids ETFs and indices in IN-EQ, so the IN-EQ OG4 regime benchmark (NIFTYBEES), B1 and (FX, crypto) B2/B3 are not in an IN-EQ
  store; r2run reads them across class stores (`aux.ext`, cut at the reading class's development end). B2/B3 for India windows and B1/B3 for crypto stay
  caller-supplied numbers to the evaluator (listed in its `self_supplied`): the sealed benchmark engine is not built (addendum 2 rule 2, accepted leak).
* NEW H-9. `close-sweep` computes the CLOSE root from the series in `enumerate_cells` order; any producer must follow that order (string-sorted values,
  name-sorted axes, harness axes included in the cell index). r2run takes the order from the OPEN manifest.

## Gaps (not done)

* The data step was not run on real data (the coordinator runs it). ADANIENT is excluded by ruling. IN-OPT `fo` bhavcopy files (15 GB, no date in column 1) are not ingested; only index
  and VIX series are. `r2-diag-options-weekly` needs a typed bhavcopy loader.
* Holdout (phase 2) is not implemented: a holdout window is refused or reported unsupported even when unsealed.
* B1/B3 for CRYPTO windows beyond 2021-12-31 need NIFTYBEES 2022+ inside the sealed benchmark process (addendum 2 rule 2, not
  built); B1 is absent there (OG2-B1 not applicable). The BTC-USD proxy chain before 2017-08-17 (A-BENCH-2 splice input) is not wired;
  B2/B3 start at the first BTCUSDT date inside the window.
* The engine is long-only, one account, daily bars: pairs (short leg), options, intraday and short carry books cannot run in it.
  Funding is charged long-side (the `Side::Short` path exists in `funding.rs` but no engine path produces shorts).
* The shift test is not available for the rebalancing engine (month-end decisions cannot be delayed one bar from outside): the
  integrity input is `None`, so the evaluator reports INCOMPLETE. Prefix invariance of a cross-sectional rebalancer is checked one
  symbol at a time (degenerate universe).
* No T+1 settlement calendar (`RunConfig.calendar` off); no per-symbol slippage; 1,000 random-entry draws and 20 seeds re-run the
  engine each time (slow on 50 symbols x 10 years; sequential and deterministic by design).
* Per-trade month-clustered t (4c-x) is not supplied (r2stats GAP-2). Sector tables for pairs formation are not provided.
* IG3 and the holdout-peer gates, the V1/V7 outputs, books, benchmarks and `eligible_symbols` remain self-supplied inputs to the evaluator (its `self_supplied` list).
* Reviewer should audit `fill_trust` in `tq-backtest` (opt-in; default runs are unchanged, all 140+ backtest tests pass).

## TDD log (real runtime RED before implementation, assertion failures)

* engine fill trust (`crates/backtest/tests/fill_trust.rs`): 5 of 5 RED with the flags present but ignored (e.g. entry_ts none vs
  fill at the usable open); rebalance path: RED by division-by-zero panic at a zero open, then the target-wait rule was added.
* data 9/9 RED on a stub loader; cellrun 9/9; robust 6/6; baselines 3/4 (the rebalance None case passes on the stub by design);
  integrity 4/6; guard 7/7. One RED exposed a real design bug: `DevReader::bars` rejecting zero-open rows (H-1).
* Written test-and-code together, then proven RED by a mutation of the code under test (mutation, runtime assertion failure):
  spec (ring-1 emptied), splits (validation start taken from the end), folds (train 24 not 36 months: 3 tests), setup (kill-switch
  off 50% not 100%), wrappers (delay removed), funding (received-funding tax 0), bench (frictions off), assemble (PBO cells dropped),
  audit (nothing written: 2 tests), sweep (suite.json not written), datastep (close instead of adjclose), cli (dirty-harness check removed).
  The wrapper random-entry test also caught a real bug (a refused entry left the symbol stuck).

## Reconciliation with the fixed harness (this branch after merging integration)

Adapted: sealed store (key file, `.enc`, verify-before-cut, allowlist, total-return basis, exclude, raw cut), repo-level writer registration and annotated tags
in the test fixtures, the REAL pinned caps in tests (candidate `r2-trend-lc-sma`; the pins make synthetic caps impossible), `close_sweep_with_cells` and
`close-sweep --cells-dir`, ledger-order cells and `<index:05>.f64` series, the integrity artifact bound to the CLOSE root, `gate-eval --repo`, exposure in
[0, 1], the harness-added tag-object axes in the guard's axes comparison. RED evidence: the 20 failing tests the merge produced were the RED; the new
behaviours (ledger order, external cut, exclusion, raw-cut maximum) were proven by mutation (each mutation made its test fail).

## Catalogue status (owner requirement)

`tq r2run catalogue [--write]` records, for all 15 families of the frozen catalogue (mirrored by `research/caps.json`; a test fails if the two drift),
whether this engine can run them: `research/r2run/catalogue-status.json` and `docs/r2run-catalogue-status.md` are generated and a test compares the committed
files with the generator. NOT-RUN is a status, never a verdict about the strategy: it is distinct from KILL, INCONCLUSIVE-SAMPLE and DEV-GO, it opens no sweep and
consumes no trial cell. `plan` and `run-sweep` (and `require_open_sweep`) refuse a NOT-RUN family with the status message. `tq r2run final-report` prints the
status table with the ledger cells per family and flags any NOT-RUN family that consumed cells as VIOLATION (the harness `open-sweep` itself does not know the
status: the coordinator must not open such a sweep; the final report catches it). The `tq r2stats` evaluator is not changed: the NOT-RUN verdict lives in the
catalogue and final report, not in `gate-eval`.
Runnable: trend-lc, trend-cr, xsmom-lc, xsmom-cr (long-only top-bucket rotation), mr-lc, mr-cr. NOT-RUN engine limitation: pairs-lc, pairs-cr, diag-carry,
diag-options-weekly. NOT-RUN data unavailable: intraday-nse, diag-intraday-cr. NOT-RUN other: diag-volt-blend (two calendars and currencies in one account),
diag-trend-idx (spans two classes, BTC-LONG proxy chain not wired), diag-council (councileval has its own loader and guard outside the sealed store).

## Real-data readiness fixes (hardening/r2fix2)

### A. Member aliases and unfillable members (addendum 1 section 3)

The raw PIT membership file lists symbols as they were at the time; the store holds one series per current name. `docs/data/member-aliases.csv`
(columns `member_symbol,status,store_symbol,verified,source,note`) is the committed, documented table that ties them. It is part of the data manifest
hash. Only a genuine rename or merger where the SAME listed instrument continues is an `alias`; a member with no usable series is an `unfillable` row,
never replaced by another company and never given a proxy price. `verified` is `unverified` for everything the repo cannot confirm from a primary source
(the sources are `nifty50_symbol_notes.csv`, addendum 1 sections 3 and 5, the data-step record in STATUS.md).

Found by comparing every symbol of the members file with the committed `r2-dev-manifest-IN-EQ.csv` (15 symbols have no store series under their own name):
aliases SESAGOA and SESASTERLITE to VEDL, INFRATEL to INDUSTOWER, IBULHSGFIN to SAMMAANCAP, MCDOWELL-N to UNITDSPR, TATAMOTORS to TMPV, LTIM to LTM (all
unverified); unfillable CAIRN, HDFC, IDFC, JPASSOCIAT, RANBAXY, STER (no history, merged or delisted; unverified) and ADANIENT, JIOFIN (data-step
exclusions, verified; both join the membership after the development end). Of the 15, 6 aliases and 6 unfillable occur in the development membership.

Mechanics (`aliases.rs`, `members.rs`, `data.rs`): `load_membership` maps member symbols through the table, so a renamed instrument keeps one continuous entry
window (before this, SESAGOA/INFRATEL/... never matched a series and VEDL/TMPV started their windows late). `resolve_members` classifies every development
member as DIRECT (a loaded series of that name), ALIAS (target loaded) or UNFILLABLE (declared) and REFUSES (`unclassified_members`) when a member is none
of these, an alias target is not loaded (the spec universe is missing it), or a declared-unfillable member has a loaded series. It runs in `load_ctx`
(so `plan` and `run-sweep` refuse before printing an open command or running a cell) and again inside `coverage`. Coverage now counts every member in the
denominator (unfillable ones included) whatever the spec universe lists; previously the universe filter silently dropped them. `plan` prints the member
listing (`aliased`, `UNFILLABLE`, flag UNVERIFIED), `suite.json` carries it (`members`), and `run-sweep` prints it. Gap: the harness `tq r2 open-sweep` itself
does not know the table; the refusal is in `tq r2run plan` and `run-sweep` (the coordinator opens the sweep with the command `plan` prints, and `plan` refuses
first).

### B. Closed trades are flat-to-flat (master 2.5)

The rebalancing engine records every sale, trims and top-up sales included, as a closed `Trade` (all with the position's first-buy timestamp). Counting them
inflated `N_raw`. `roundtrip::flat_to_flat` folds the sales into round trips per symbol (symbol plus first-buy date = one position lifetime; P&L summed; a
lifetime still open at the end is not a closed trade). `CellRun.trades` stays the raw sale list (report-only `N_raw`), `CellRun.round_trips` is the
flat-to-flat list, `open_at_end` is separate and never a closed trade. `full_book()` and `stitched().book` (the evaluator's `TradeBook`s, so V2, `N_eff`,
the best-trade drop and the diversification clause) are built from the round trips; `assemble` refuses (`trade_count_mismatch`) unless the books carry
`min(raw, flat_to_flat)`. `suite.json` has `trade_counts` (dev and D-OOS; raw, flat_to_flat, open_at_end). `k_bar` is the open-position count per day, which
trims and top-ups do not change; per-trade P&L is the round trip's. The random-entry baseline calibrates on round trips. The signals path is unchanged (one
trade per position).

### C. Engine-level prefix invariance (`tq r2run engine-prefix`, diagnostic)

`tq r2run engine-prefix --spec F --store DIR --cut D1[,D2..] [--cell base|ID] [--level-stress] [--scratch DIR]` runs ONE cell on the whole development data, then again
on data RECOMPUTED as of each cut (`DevData::truncated_at`: every series, the funding side input and the membership snapshots cut at the date; the rule is rebuilt from
that data, so any normalisation or universe the rule or loader derives is recomputed from rows on or before the cut). It compares the engine equity curve, exposure and
open-position counts, daily pre-tax and net returns (bit for bit), closed trades (all fields), reconstructed positions (signals engine) and the rule's own decisions
(signals per bar, targets per rebalance, so the last bar's orders are compared too). Any difference is a look-ahead finding (first date, symbol, field). It opens no sweep,
reads and writes no ledger, reads the development store only, refuses a cut after the class development end, and writes only an optional scratch report.

Comparison horizon (documented artefacts, not leaks): the engine knows a series' last bar, so in the truncated data the cut is an artificial last bar (entries on it are
dropped, exits close at its close, month-end decisions are impossible, and the unfinished tax year is paid at the last point). Day series, trades and positions are therefore
compared strictly before the earliest such bar, decisions through it (signals engine) or before the last day (rebalancer), net returns before the last day.

Price LEVEL: the development store keeps one dividend-adjusted OHLC series (`adjclose` rescaled to the development end); the raw close and the adjustment factor are NOT kept,
so the point-in-time adjusted series as of a cut cannot be rebuilt. `--level-stress` stands in: every price of the i-th series (sorted) is multiplied by 1.00 + 0.02 (i mod 10),
and decisions, trade dates and positions are compared, with the numeric drift reported. Crypto prices are raw Binance prices (no dividend adjustment), so level stress does not
apply to crypto. Controls (planted leaks: next-bar peek, absolute price level, whole-sample dollar-volume rank, whole-sample normalisation, end-of-period membership universe)
are in `engine_prefix_tests.rs`; the absolute-level control is invisible to slicing and caught only by the level stress.

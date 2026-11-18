# options-v2: harness pin list and Study-profile design (spec for the harness-generalisation agent)

Status: DRAFT, docs only, written 2026-11-17 on `hardening/o2proto`; gate list and sealed-batch rules amended by prereg addendum A1 (see section 4 below). No strategy has been run. All `file:line` references are to `hardening/integration` at 09b7371 (the worktree base). The goal: everything research-v2-specific that the options-v2 study must replace is listed with its place, and a minimal `Study` config makes research-v2 the default so that all existing tests stay green.

## 1. Proposed design: one `Study` struct, `Study::research_v2()` as default

A new module `crates/r2gov/src/study.rs` (r2gov is the lowest crate of the three, so r2run and r2stats can use it):

```rust
pub struct Study {
    pub id: &'static str,                 // "research-v2" | "options-v2"
    pub ledger_family: &'static str,      // gate.rs:40
    pub master_tag: &'static str,         // gate.rs:14
    pub master_file: &'static str,        // gate.rs:39
    pub master_sha256: &'static str,      // gate.rs:17  (options-v2: hash of the TAGGED prereg-options-v2.md)
    pub caps_rel: &'static str,           // gate.rs:20, bind_repo / registrar paths
    pub caps_sha256: &'static str,        // gate.rs:19
    pub ledger_file: &'static str,        // registrar.rs:49
    pub lock_file: &'static str,          // registrar.rs:52 and :249
    pub catalogue_status_rel: &'static str, // not_run.rs:14, cli.rs:191
    pub orphans_rel: &'static str,        // orphans.rs:13
    pub candidate_prefix: &'static str,   // "r2-" -> "o2-"
    pub classes: &'static [ClassName],    // classmap.rs, guard.rs:25, inputs.rs:30
    pub lineage_cells: u64,               // dsr.rs:13 L_TOTAL (364 for research-v2)
    pub windows: WindowSource,            // Table(master prereg) | Fixed(StudyWindows)
}
pub fn research_v2() -> &'static Study   // default; every existing call site passes it
```
Every function that today reads a `const` takes `&Study` (or reads `Study::current()`, a `OnceLock` set by the CLI `--study`, default research-v2). Tests keep calling the old signatures through thin wrappers that pass `research_v2()`, so the existing 1,913 tests do not change.

Seam principle: **no gate number moves**. The options-v2 gate constants live in a separate `crates/r2stats/src/o2gates.rs` (new, from `docs/prereg-options-v2.md`), selected by `Study`; the research-v2 constants in `gates_*.rs` stay untouched.

## 2. Pin and assumption list (file:line, what is pinned, what must change)

| # | Place | Pin / assumption | options-v2 change |
|---|---|---|---|
| 1 | `r2gov/src/gate.rs:14` | `MASTER_TAG = "prereg-research-v2"` | `Study.master_tag = "prereg-options-v2"` |
| 2 | `gate.rs:17` | `MASTER_SHA256 = 0f36d2a5...` compiled in; moved tag refused (`gate.rs:87-89`) | new hash AFTER the options-v2 freeze; compile in the options-v2 constant, never edit the research-v2 one |
| 3 | `gate.rs:19-20` | `CAPS_SHA256`, `CAPS_REL = research/caps.json` | `research/caps-options-v2.json` + its own hash (caps changes need a reviewed build) |
| 4 | `gate.rs:39-40`, `:145-146`, `:162-163` | `MASTER_FILE`, `LEDGER_FAMILY = "research-v2"`; a manifest with another family id is refused (`BadInput`) | `ledger_family = "options-v2"`; ledger file `research/ledger-options-v2.jsonl` |
| 5 | `gate.rs:43-50` | candidate tag `prereg-<id>`, file `docs/prereg-<id>.md`; `valid_id` lower-case ids | keep (`o2-...` ids pass `valid_id`) |
| 6 | `gate.rs:54` | `default_harness_paths` = crates, Cargo.*, `research/caps.json` must be clean at sweep open | add the options-v2 caps file and the new `crates/opt2` |
| 7 | `registrar.rs:49-52`, `:249`, `:289` | ledger path `ledger-research-v2.jsonl`, lock names, master-tag-object axis | derive from `Study` |
| 8 | `r2gov/src/caps.rs:17-20` | axis names `r2-candidate`, `r2-holdout-touch`, `r2-batch`, `master` | keep (internal axis names, not study-specific); `caps.rs:58` parses `ledger_family` from the caps JSON (already data) |
| 9 | `caps.rs:187-230`, `r2stats/src/bind.rs:312-348` | cap check by caps.json families; `go_candidates()` requires a CLOSED sweep for EVERY GO candidate (the structural blocker of research-v2), N_new_final = `family_trials(caps.ledger_family)` | NOT-RUN candidates excluded from the derivation sets (addendum 7 logic) must be the default for options-v2; keep N counting every opened cell |
| 10 | `r2gov/src/windows.rs:54-81`, `r2run/src/splits.rs:32-53` | class windows PARSED from a 4-column table (`Class | Train T | Validation V | Class holdout H`) in the tagged master prereg; classes IN-EQ/IN-IDX/IN-OPT/CRYPTO; "holdout" = one class-wide single touch | options-v2 has THREE windows (Phase-2 development-only, Phase-1 development D1, sealed S) per structure, plus per-candidate "Phase-2 available from" date. Either add a second table to the prereg with its own parser (`StudyWindows`) or a `Fixed` source (dates compiled from the tagged prereg and hash-pinned). Recommended: `WindowSource::Fixed` with the dates in a JSON committed with the prereg and covered by the master hash |
| 11 | `r2run/src/folds.rs:14`, `:58-91` | walk-forward folds (`in_validation = test_start >= v_start`), purge `max_hold_days`; IN-OPT purge = 21 sessions | options-v2 has NO walk-forward folds (stages D1 / P2 / S replace them); the fold plan becomes a plan of three windows with purge = the structure's holding period (weekly 5, monthly 21) |
| 12 | `r2gov/src/holdout.rs` (whole), `sealed.rs`, `seal_crypto.rs` | single touch per CLASS batch (`AXIS_TOUCH` = class); `holdout.rs:118` family id | **ONE sealed batch** (A1.4): every candidate registered first; after S1/S2/G8 pass for the last registered candidate ONE touch writes one marker per candidate on its own cap axis `o2-sealed-touch` (NOT the 60-cell axis and NOT a reserve cell), runs every S3 evaluation and publishes all outcomes together; no registration after the batch, no re-run after the batch; the sealed-store layout (`dev/`, `sealed/`) is already class-named: add class `IN-OPT-V2` or reuse IN-OPT with the new split date 2025-05-01 |
| 13 | `r2gov/src/classmap.rs:40-43`, `r2run/src/guard.rs:25-28`, `r2stats/src/inputs.rs:30-33`, `datastep.rs:165-183` | file-stem to class allow-list; IN-OPT = `OPTION_SPOT_STEMS` or `fo*`/`opt*` stems; data step puts NIFTYBEES/BANKBEES/NSEI/NSEBANK/INDIAVIX | add the new bhavcopy store stems (from the o2data manifest) to IN-OPT; options-v2 needs the NSE trading-day calendar (not the benchmark's dates) as the master calendar |
| 14 | `r2run/src/catalogue.rs:40-88`, `r2stats/src/not_run.rs:14` | catalogue of research-v2 families with RUNNABLE/NOT-RUN; `research/r2run/catalogue-status.json`; the line 88 `r2-diag-options-weekly` NOT-RUN row | new catalogue `o2-...` from the capability audit (100 runnable cells, 30 NOT-RUN); status JSON path from `Study` |
| 15 | `r2stats/src/orphans.rs:13`, `bind_repo.rs:58` | `research/declared-orphans.json` | `Study.orphans_rel` (start empty: no orphan may be declared before a result exists). A1.4 rule: a marker or sweep with ZERO stored results may be declared orphan by a dated addendum written before that candidate's next evaluation; one with results never. Catalogue state **UNSELECTED** (RUNNABLE, no candidate, no cell) is excluded from completeness and every derivation set and is never a failure |
| 16 | `r2stats/src/dsr.rs:13-24`, `gates_stat.rs:16` | `L_TOTAL = 364`, `n_true = N_new_final + L`; DSR >= 0.95 with the class-cell variance floor `1/T`; daily and monthly | options-v2: `N_cand = candidates registered (<= K + reserve) + lineage 6`, ex-ante variance, see the protocol design (and `Study.lineage_cells` kept for N_all reporting) |
| 17 | `gates_edge.rs:12-20` | `T_ABS_DEV_OOS 3.0`, `T_HOLDOUT 1.65`, `SHARPE_OOS_MIN 0.8`, `SHARPE_HOLDOUT_MIN 0.5`, `MDD_LIMIT 0.24`, invested share >= 20% | replaced by the o2 staged hurdles (`o2gates.rs`, numbers = the FROZEN hurdle table `FROZEN_HURDLES` of `crates/r2stats/src/o2cal/design.rs` and the constants of `o2cal/rules.rs`, both inside the freeze hash); invested-share gate OG6 is meaningless for an options book (exposure is margin/equity) and must be dropped or redefined |
| 18 | `gates_sample.rs:17-22` | `N_EFF_DEV 200 / OOS 100 / HOLDOUT 30`, distinct symbols >= 10, drop-5-best | o2: `N_eff` D1 >= 30, sealed >= 10 with the autocorrelation design effect (item 19) |
| 19 | `neff.rs:56-83` | `rho_bar` fallback = 1 for a single underlying; `N_eff = N_raw / (1 + (k_bar-1) rho_bar)`: non-overlapping weeklies get N_eff = N_raw | add `DE_time = max(1, 1 + 2 sum_{k=1..4} (1-k/5) rho_k)` of the per-cycle net returns, `N_eff = N_raw / max(DE_harness, DE_time)`; report both |
| 20 | `gates_og.rs:11-17` (OG5/V7), `bind.rs` ring-1 derivation | ring-1 >= 80%, all cells >= 60%, V7 70%/70%; neighbours enter the class variance set | options-v2: sign-only robustness on the 2 one-step neighbours of the single parameter axis; neighbours NEVER enter DSR/SPA/PBO sets (they are counted in the ledger only) |
| 21 | `og4.rs:14-19`, `regime.rs:16-19` | calendar years >= 60% (>= 6 months), regimes by 200-day SMA / 60-day vol over DAILY bars, expanding median from day 252 | weekly/period regimes: 40-week SMA, 13-week vol, warm-up 52 weeks; quarters instead of years (D1 has only 3 full years) |
| 22 | `og2.rs:12-16`, `weekly.rs:39-44` | B1/B2/B3 weekly Sharpe, paired MONTH-block bootstrap, 90% lower bound, Friday 15:30 IST cut | options-v2: periods = weekly (or monthly for monthly cycles), stationary bootstrap block max(4, 2*hold); lower bound level and B3 treatment per the protocol (`lb 80%` on B1, point estimate on B3, point estimates in S) |
| 23 | `mc.rs:13-24` | MC month-block, p95 MDD <= 24%, >= 95% profitable, kill-in-2y <= 5% (crypto 730 d) | replace by the benchmark-relative drawdown limit (observed MDD of the stage window); keep the MC as a REPORTED statistic |
| 24 | `r2stats/src/evaluate.rs:249`, `:250-299` | `class_bench = B2 if Crypto else B1`; gate list; holdout stage `Stage::Holdout && class.has_clean_holdout()`; `inputs.rs:24` `has_clean_holdout` = not Crypto | options-v2 class benchmark = B1 for all; the staged gate list (E1..E3 + quality gates) is a new function `gates_o2()`; `has_clean_holdout` true for options (sealed slice) |
| 25 | `r2run/src/auxload.rs:41-52`, `bench.rs:40`, `datastep.rs:174-189` | aux loading: NIFTYBEES (B1), BTC-USD/BTCUSDT + USDINR (B2, B3), class benchmark validated; benchmark builders `bench.rs:40-73` (costs-and-tax paying, liquidation basis); `bench_costs`, `bench_clean` repairs (addenda 4 and 6) | options-v2 needs B1 and B3 on the WEEKLY/period grid of the candidate over EACH stage window (D1, P2 when available, S) with the same costs; BTC data is on the crypto dev side (all windows through 2026-10); NIFTYBEES dev side ends 2021-12-31: the 2022-2026 B1 requires the o2data pull (the sealed side is NOT to be read for it): this is a DATA DEPENDENCY, not a harness one |
| 26 | `r2run/src/spec.rs:16-21`, `cellrun.rs:169-180`, `rule.rs`, `rules/` | `EngineKind {Signals, Rebalance}` and `execute()` matches `(spec.engine, built)` over `run_portfolio` / `run_rebalance` on `Series` (daily bars of a symbol universe) | the options runner plugs in as a THIRD engine kind `EngineKind::Options` (serde `options`) with `Built::Options(Box<dyn OptionsBook>)` and an arm in `execute()` that calls the new `crates/opt2` runner and returns the same `RunReport` (daily net/pre-tax/exposure series, trade list with `symbol = underlying`, `net_pnl` after costs before tax). `tq-options` stays frozen at v1 (old bhavcopy format only): `opt2` is a new crate. `data.series(&spec.universe)` (`cellrun.rs:161`) must be bypassed: the options book reads its own option-chain store; `spec.universe` lists the underlyings |
| 27 | `r2run/src/cellrun.rs` after-tax series, `physical.rs`, `evaluate.rs:203-232` | non-physical-series guard (return <= -100% or non-finite blocks the consuming gates, verdict INCOMPLETE; a real failure stays KILL) | KEEP and make options-v2's series rule fail closed (`metrics.rs:32` returns 0 when equity <= 0 and hides ruin: the options runner must refuse to return a zero for equity <= 0, and the guard must see it) |
| 28 | `r2run/src/ig2.rs`, `engine_prefix.rs` | prefix-invariance / look-ahead test replays the engine over truncated data | options runner must implement the same trait: signals on day t use data through t; entry at the t+1 open |
| 29 | `r2run/src/sweep.rs`, `cli_run.rs:43` | sweep open/close, `n_now = ledger.family_trials(LEDGER_FAMILY)` | use `Study.ledger_family`; sweep manifest carries `data_hash` (o2data manifest), `cost_model_hash` (`guard.rs:34-45` includes class, slippage, risk pct; add margin model and tax mode) |
| 30 | `r2run/src/setup.rs`, `RunSetup` (risk 0.5%, notional cap 20%, kill switch 30% assumption) | equity-book sizing | options sizing = pre-registered capped fraction of margin on a Rs 1 crore account (Rs 25 lakh stress), kill switch OFF in assessment (as v3 section 7); NOT the equity RiskLimits |
| 31 | tests | `r2gov/src/testutil.rs:58-59`, `r2run/src/gitfx.rs:54-65` embed `docs/prereg-research-v2.md` and `research/caps.json` by `include_str!` | unchanged (default Study); add parallel fixtures for options-v2 behind the new Study only |

## 3. What a dry run (synthetic end to end) must show before the freeze

1. A synthetic options world (planted net Sharpe 0 and 2, weekly and monthly shapes, the real NSE calendar) goes through `opt2`'s runner stub -> sweep open/close -> store cells -> evaluate (staged gates) -> verdict, and a planted S=3 candidate reaches DEV-GO and a null never does. Nothing may be NOT-EVALUATED for a structural reason: every input a gate reads is derivable (ledger completeness without unclosed NOT-RUN sweeps; deflation set = candidates only; benchmark inputs present for D1/P2/S; after-tax series physical).
2. The NOT-RUN catalogue entries consume no cell and block no derivation (addendum-7 behaviour as the default).
3. A forged input (weak benchmark, inflated trade count) cannot flip a verdict (the `forged` controls of `tq r2stats calibrate forged` re-run on the o2 gates).
4. The after-tax series of a losing book stays above -100% by construction (provisional tax on the positive YTD net business income only, see the prereg) and any series that is not is INCOMPLETE, never PASS.
5. The sealed-slice touch is refused before every development stage has passed, refused twice, and counted as a reserve cell.

## 4. Gate-list changes of addendum A1 (2026-11-17; the harness agent implements these)

The o2 gate list is now (all on the base cell, evaluation path `Pick::Base` only): G1 N_eff + skip cap (15% skips = INCONCLUSIVE-SAMPLE); G2 HAC t on the REALISED cycle P&L series vs the frozen hurdle table; G3 benchmark (80% paired bound on B1, point screens on B3); G4 drawdown = clamp(1.5 x MDD_B1, 25%, 40%) AND 3 x annualised volatility; G5 stability (quarters from 3 evaluable, regimes, halves, drop-best-1%, adverse regime must be evaluable); G6 sign-only neighbours; G7 cost stress; G8 integrity (+ expiry-day settle planted-zero test, carried marks <= 10%); **G9 stress exposure** (INCONCLUSIVE-TAIL); **G10 absolute floor 6% on fixed capital**; **G11 volatility band 6-14%** (INCONCLUSIVE-SIZE). Verdict precedence: any failed gate = KILL; else INCONCLUSIVE-TAIL, then -SIZE, -SAMPLE, -QUALITY; else PROMOTED TO FORWARD PAPER (the word "passed" is never printed). DSR(N = 36), DSR(N_all) and SPA p are printed on every verdict line with WEAK when DSR(N = 36) < 0.5; SPA is NOT-EVALUATED and non-blocking when a registered candidate has no returns.

Additional dry-run items (section 3 above is extended):
6. Nine candidates reaching the sealed batch at the design cap (30 on the `o2-sealed-touch` axis) are all served; a crash between marker and read leaves an unclosed marker that is refused as an orphan if results exist and may be declared orphan if none exist; an UNSELECTED catalogue row neither blocks nor counts.
7. A near-cash overlay credited 6.5% on idle cash gives the SAME verdict (returns are P&L on fixed capital, no cash credit); a Sharpe-2 book at 2% volatility is KILL by G10 and G4, never promoted.
8. A planted calm window with no benchmark stress week gives INCONCLUSIVE-TAIL for a short-convexity book, not a pass and not a kill; a failed gate in the same window is KILL.
9. Expiry-day SETTLE_PR = 0 (planted) and an index-level settle (planted) make the series INCOMPLETE; intrinsic from the final settlement price is used.
10. `--bench-csv` with any row dated after the cut fails closed; a benchmark pool shorter than the timeline fails closed (no cycling).

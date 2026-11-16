# Adversarial review of `docs/prereg-research-v2.md` and `docs/prereg-r2-TEMPLATE.md`

Reviewer role: hostile statistician / quant-risk auditor. Reviewed 2026-11-16 at branch head `a9d23ff`. Read-only: nothing was run, edited, tagged or pushed. Line references (`M:nn`) are to `docs/prereg-research-v2.md` as committed (sha256 `5a801475...`, the same hash the master marker already carries).

## Verdict: FIX THEN TAG

The skeleton (cell-level trial unit, shared ledger family, caps, additive owner gates, sealed batch holdout) is sound and mostly honest about its own weaknesses. But as written it has three design-level defects that make it either unenforceable or mechanically unpassable, plus a cluster of HIGH leakage and accounting holes. None needs a rewrite; all are edits to the text before the tag. Several items also need code that does not exist yet (see "Minimal edits", item 2), and the tag text must commit to that code being committed and reviewed before the first run.

Tally: 3 CRITICAL, 9 HIGH, 11 MEDIUM, 6 LOW.

## What I verified (counts and arithmetic)

- `research/trials.csv`: 227 data rows = 226 prior + 1 research-v2 row. Per strategy: donchian-breakout 114, trend-pullback 62, momentum-rotation 26, sentiment-scaling-family 6, index-trend-filter 5, buy-and-hold 5, sue-drift-0a 3, basis-carry-0b 2, funding-carry 1, council-v1 1, options-v1 1. Master's L numbers (119 / 26 / 62 / 11 / 10 / 30; council 162/110/74/51) reproduce exactly (M:128-136; council precedent matches `prereg-council-v1.md:75`).
- Ledgers: council-v1 = 1 OPEN (n=48) + 1 CLOSE; options-v1 = 1 OPEN (n=30) + 1 CLOSE; research-v2 = 1 OPEN (n=1, master marker, never closed). Anchors and `hardening/ledger-anchor.txt` agree. `sha256(docs/prereg-research-v2.md)` equals the marker's `strategy_hash`, `data_hash` AND `split_spec_hash` (one value used three times).
- Caps: family caps sum to 420 and the catalogue's max grids fit under them (TREND 60/72, XSMOM 54/60, MEANREV 72/72, VOLT 24/24, PAIRS 48/48, CARRY 20/24, INTRADAY 36/36, OPTIONS 48/60, COUNCIL <=6 per base). 420 + 80 reserve = 500.
- Fold counts in M:95 reproduce (IN-IDX 14+8, IN-EQ 6+8, IN-OPT 10+4, CRYPTO 13+8, BTC-LONG 25).
- v3 gate numbers restated in M:26-37 match `validation-protocol.md` (200/100/30 trades, t 3.0/1.65, Sharpe 0.8/0.5, DSR 0.95, PBO 0.20, 70% neighbours, 60% folds, MC 0.8 x limit).

---

## CRITICAL

### C1. The holdout is not single-touch at program level; a "burned" candidate returns under a new id (M:84, M:157)
M:157 says a crash after the holdout marker "burns" a candidate and "any change is a NEW candidate id". A new id gets its own `--holdout-ack`, its own HOLDOUT-TOUCH check, and the same H. v3 rule 0.5 (`validation-protocol.md:73-76`) says the opposite: after a holdout leads to a change, "its next out-of-sample is forward/paper data only." Same hole in M:84 ("touched at most once per candidate"). Because K_H candidates all see the same H, and any variant of a failed candidate can be re-registered, H is worn out serially. The batch/sealing in M:157 only protects within one batch.
Fix: H is touched at most once per class (IN-IDX, IN-EQ, IN-OPT, CRYPTO) per program, in one sealed batch. A candidate whose lineage (`parent_strategy_hash`, same rule family, same book) touches a burned id may only use forward paper data. State it in M:84 and M:157 and make the harness refuse any `--holdout-ack` once the class batch has run.

### C2. Pre-registration order, caps and one-touch are enforced by promise and by uncommitted code (M:84, M:142, M:147, M:157, TEMPLATE s.3 "Holdout guard: yes")
- M:147 says the sweep is opened by "a throwaway crate (scratchpad)". That code is not in git, not reviewed, not hashed. `code_git_sha` in the manifest is a free string; nothing checks it. M:142 "the harness refuses `open_sweep` beyond a family cap": `Ledger::open_sweep` only enforces `MAX_CELLS = 1_000_000` (`ledger.rs:21`); the 500/420/80 caps exist nowhere in code.
- Existing guards are not what the doc claims. `tq options-eval` / `councileval` take a bare boolean (`options/src/window.rs:29`, `cli/src/options_eval.rs:31`), not `--holdout-ack=<candidate-id>`; they check no ledger; windows are hard-coded to old boundaries (options dev end 2021-12-31; `councileval/src/window.rs:21-26`: equity dev end **2019-12-31**, crypto 2023-12-31). Any dev run can pass the flag.
- Ledger entries carry no timestamp (no wall clock by design). Ordering "prereg tagged -> sweep opened -> run" rests only on git history; nothing stops an agent running `tq backtest` directly first and opening the sweep afterwards.
- M:84 says loaders "fail closed on the date in the file name". Equity data on disk are one CSV per symbol (`engine/data/daily/ADANIENT.csv`, `index/NSEI.csv`, `etf/NIFTYBEES.csv`), not dated files; councileval truncates by window end. When `wt/rdata` pulls 2022-2026 into those single files, a filename rule is vacuous.
Fix: (a) the tag text must require a committed, reviewed research-v2 harness (tests first) that implements: cap enforcement from a committed `research/caps.json`; `open_sweep` refusing unless tag `prereg-r2-<id>` exists, its commit is an ancestor of HEAD and `strategy_hash == sha256(file at tag)`; a holdout guard keyed by candidate id AND class that reads the ledger and writes the HOLDOUT-TOUCH marker (fsync) before any H-dated bar is opened; a content-level max-date check on every loaded bar (not filename). (b) No candidate runs until that harness is reviewed. (c) Say in M:84 that one-touch is a procedural control backed by those checks, not a claim of impossibility.

### C3. The catalogue is inconsistent with OG5/V2: most families are KILL "by construction" and burn most of the 420-cell budget
- OG5 (M:49) kills a candidate with fewer than 4 ring-1 neighbours and requires >= 3 levels per tuned parameter. Grids as written (M:166-174): XSMOM `skip {0,21}` has 2 levels and base 252/21/10 sits at the edge of lookback and k, so at most 3 ring-1 neighbours (M:167). VOLT `window {20,60}` has 2 levels (M:169). PAIRS has four binary axes (M:170). MEANREV `threshold 2` (M:168). CARRY has 2-level margin/entry axes and a categorical instrument axis (M:171). OPTIONS "monthly 6 base" cells have no neighbours at all (M:173; options-v1 used 30 cells for this reason). COUNCIL's full/minimal/leave-one-out has no ordered axis (M:174). Even TREND's categorical `rule` axis has no defined order, so whether the base has 3 or 4 neighbours depends on how the author orders SMA / Donchian / sign-return.
- V2 (M:28) needs >= 10 distinct symbols for "multi-symbol books" and no symbol > 15% of net P&L. B-IN-ETF and B-IN-ETF-ROT (3 instruments), B-CRYPTO (BTC, ETH, 5 alts), CARRY (4 perps), OPTIONS (2 underlyings) cannot satisfy 15% (minimum possible maximum share for 3 symbols is 33%). A1 (M:202) flags only trade counts.
- V2's clause "no two counted trades share an entry day" (M:28) is not in v3 (`validation-protocol.md:56-62`) yet is presented as restated/UNAMENDED. For monthly-rebalanced books it caps counts at about 12 entry days a year: B-IN-LC XSMOM over D (10 y) <= 120 < 200 and over D-OOS (7 y) <= 84 < 100, so INCONCLUSIVE-SAMPLE is guaranteed whatever the edge.
- Net: XSMOM, VOLT, PAIRS, CARRY, OPTIONS-monthly, COUNCIL and every 3-7 instrument book cannot reach GO, but their cells count in N_new and raise N for the one or two viable books (TREND on PIT large caps, parts of MEANREV and OPTIONS-weekly).
Fix: (1) redesign every grid to >= 3 levels on each tuned numeric axis with the base at the centre (e.g. skip {0,21,42} base 21; VOLT window {10,20,60}); treat each categorical rule as its own book, outside OG5. (2) State that V2's >= 10 symbols / <= 15% tests apply only to books with >= 10 eligible symbols; smaller books are capped at INCONCLUSIVE-SAMPLE and run (if at all) as diagnostics with cells not counted. (3) Delete the entry-day clause or restrict it to the same instrument. (4) Print a "can reach GO: yes/no" table per family from the final rules before the tag and drop families that cannot; do not spend N on them.

---

## HIGH

### H1. "Validation" is not out-of-sample in any class; the seen-window disclosure understates contact (M:14, M:85, M:99)
- Equity: T = 2012-2017 and V = 2018-2021 both lie inside 2012-2021, already read by backtest-01..04, council-v1 dev, options-v1 dev and SUE-0a. The council-v1 equity holdout was 2020-2021 and never touched (`councileval/window.rs:21`; STATUS council entries); research-v2 now consumes it as V, undisclosed, ending council-v1's sealed holdout.
- Crypto: V = 2022-2023 was read by intraday-01 / fee-breakeven-15 (1m klines 2022-2023, with year-split findings), council-v1 crypto dev (score year 2023) and carry-01 / basis-0b (2020-2026). T 2020-2021 likewise. Only BTC-LONG 2014-2019 is unseen by any strategy.
- Options: T and V (2012-2021) are exactly options-v1's dev window and structures V1-V6 (M:173 "re-run"): no new information, up to 60 cells of N, PBO 0.849 already on file. Only weekly variants and H are new.
- So V (M:99) is not an unseen check; only H is. M:85 omits crypto 2020-2023 beyond carry and the consumed council holdout.
Fix: add a per-class table of windows read by which prior work; relabel V "validation (seen in 4 of 5 classes)"; make H the only evidence that counts toward a claim; drop the R2-OPTIONS monthly re-run (keep weekly only).

### H2. DSR attainability: the doc's arithmetic is wrong and the pooled-max variance can make it infeasible (M:35, M:121, M:142)
- M:142 assumes cross-trial Sharpe std 0.3 annualised, SR0 about 1.0, required Sharpe about 1.6 on 5-7 years. The code floors the variance at 1/T (`sweep.rs:208`: `recorded.max(1.0 / t)`), i.e. 1/sqrt(years) annualised: 0.38 at 7 y, 0.44 at 5.25 y, 0.5 at 4 y. Using `expected_max_sharpe` and the PSR form (`psr.rs:37-55`) with skew -0.3 and kurtosis 8-10 (my arithmetic), the annualised Sharpe needed for DSR >= 0.95:

| Window | N=30 | N=100 | N=300 | N=619 (500+119) | N=804 (N_all) |
| --- | ---: | ---: | ---: | ---: | ---: |
| IN-IDX D-OOS 11 y | 1.13 | 1.27 | 1.38 | 1.45 | 1.47 |
| IN-EQ / IN-OPT D-OOS 7 y | 1.42 | 1.59 | 1.73 | **1.82** | 1.85 |
| CRYPTO D-OOS 5.25 y | 1.64 | 1.84 | 2.00 | **2.10** | 2.13 |
| V-OOS 4 y (if DSR were evaluated there) | 1.88 | 2.12 | 2.30 | 2.42 | 2.45 |
| monthly series, 7 y | | | | 1.92 | 1.95 |

  So the real requirement is about 1.8-2.1 net of Indian costs and tax, not 1.6; V3/V4 (t >= 3 needs about 1.1-1.3) are not the binding gates. Acceptable as "KILL is expected", but the doc must say so.
- `Ledger::family_sr_var` returns the MAX recorded variance over every closed sweep in the family component (`ledger.rs:531-544`), and all research-v2 sweeps share one family. One high-dispersion sweep (crypto alts, carry cells with Sharpe 0-3 across margins, any family with wiped-out accounts) sets the variance for every candidate in every class: annual std 0.5 gives SR0 about 1.55 at N=619 and a required Sharpe of 2.3-2.6 for everybody. M:35 just says "from the honesty layer".
- `deflated_sharpe_for_sweep` deflates with `family_trials` only (`sweep.rs:215`): no L. N_true = N_new + L (M:114) is computed by no existing function; new code must call `deflated_sharpe(obs, sr_var, N_true)` directly.
- M:35/M:121 never say which window/series the DSR uses (D, D-OOS, V-OOS, H): a free choice of the best-looking window.
Fix: DSR on stitched D-OOS daily and monthly returns only; sr_var = variance of cell Sharpes over the candidate's asset-class cells on the common D-OOS window (the SPA set), floored at 1/T, with the pooled-max value also reported; new N_true code; replace M:142's arithmetic with the table; owner acknowledgement of "required net Sharpe about 1.8-2.1".

### H3. Kill-switch 30% is agent-chosen, 6x looser than the only written value, and owner-unconfirmed (M:35, M:202 A3)
`validation-protocol.md:156-160` names a 5% drawdown kill switch as the placeholder; v3 says limits are the user's and `UNSET-BY-USER` until confirmed. The master hard-codes 30% in V7 (worst-fold MDD), V9 (p95 MDD <= 24%, P(kill in 2y) <= 5%) and OG3 (25%); setting it before runs honours "limits first" but it is not the owner's number and loosens V9 materially against a document it claims to leave unamended. 24% (V9) and 25% (OG3) are called "consistent" (M:35) while differing. Risk per trade and max positions are also never set (M1).
Fix: tag blocked on an owner-signed kill-switch, risk per trade and max positions written into M:35 and the template; one MDD threshold.

### H4. Adaptive registration and selection on development results (M:5, M:146-150, M:162, M:174)
Candidate preregs are written later by strategy agents; nothing forbids registering B after seeing A's D/V results. M:174 makes it explicit: R2-COUNCIL bases are chosen "only from catalogue candidates with net-positive D-OOS" (selection on development returns, contradicting M:13 and not counted for the unrun alternatives).
Fix: (a) all Phase-1 candidate preregs tagged and sweeps opened before ANY candidate runs, or declared batches where no later batch's design may reference earlier results; (b) delete the conditional council rule or fix council bases now, independent of results.

### H5. Bug re-runs are an asymmetric free option (M:109, M:150, M:157)
A bug re-run is counted from the reserve, but one extra trial adds almost nothing to N while the researcher keeps the favourable draw; re-running only when a result disappoints is a stopping-rule leak.
Fix: allowed only if (i) a failing unit test reproducing the bug was committed before the corrected run, (ii) ALL cells of the sweep re-run under the fixed code, (iii) both runs' return hashes stay in the ledger and the report shows both, (iv) the gate uses the less favourable run unless an independent replica confirms the corrected one.

### H6. Holdout contact through the data pipeline (M:84, M:103, STATUS D-002 item 2)
- The inventory may read "row counts and date ranges" (M:103); row counts over 2022-2026 reveal halts/gaps and quality checks (outlier and split detection) need H returns. India VIX is on the pull list and is a natural regime feature; its 2022-2026 rows will sit in the same store.
- `data-ref/nifty50_members_by_date.csv` already runs to 2026-09-30 (H-period index changes) in one file; engine data are single CSVs per symbol.
- Run logs and reports may print last bar date / last close.
Fix: at ingestion write H-dated bars into a sealed directory outside the repo and the dev loader path (permissions 000, manifest hash only); D-dated bars in their own files; quality checks on H deferred to the holdout stage; truncate membership/VIX/ADV/volume-rank inputs at the class dev end; content-level max-date check (C2); dev logs must not print dates beyond D.

### H7. Parallel strategy agents vs a single hash-chained ledger file (M:147-149, STATUS D-002 item 3)
`family_trials` is computed per ledger file. Agents in separate worktrees each hold their own copy of `research/ledger-research-v2.jsonl`; each append forks the chain at tip `bee0a44e...`; a conflict "resolved" by keeping one side drops the other's OPEN (and its N), and the anchor refuses rollback. Per-candidate files sever the family link and hence N.
Fix: one ledger writer: the coordinator opens all sweeps serially in the main worktree from tagged preregs; agents only evaluate a given sweep id. Say so in M:146-149.

### H8. N undercount in unledgered prior sweeps; L counts rows not cells; L is family-specific (M:133-134, M:114, M:138)
- INTRADAY L=3 (M:134). intraday-01 / fee-breakeven-15 ran at least BTC and ETH x VWAP and momentum (4 runs, year-split 2022/2023) plus NSE ORB on 5 stocks (`review-intraday-01.md:17`) plus maker-rebate scenario variants: at least 9 cells, none ledgered. council-v1's 12 crypto-burst cells are absent from INTRADAY's L (only 3 enter COUNCIL's 51).
- CARRY L=10 (M:133): carry-01 ran BTC/ETH x M 0.5N/1.0N (4 runs), 3 tax cases and an un-pre-registered margin band; "floor of 7" is an estimate. edge-hunt-02 market-making simulations are in no L.
- Conversely donchian/trend-pullback/momentum "rows" are sensitivities of ONE default parameter vector (seeds x slippage x kill-switch x positions; verified by reading `params` in `trials.csv`) and B&H rows are labelled "BENCHMARK (not a trial)". By M:109 these are about 3-6 cells, not 119/62/26/5. L is therefore conservative for equity and too low for crypto/intraday. Since M:138 forbids lowering L the net error is conservative, but it is presented as evidence-based when it is a convention.
- N_true is family-specific: PAIRS gets L=0 although the claim tested is "some strategy x instrument works"; the honest deflator is program-wide (about 800). The SR0 difference is only about 5%, but the choice should not be family-lenient.
Fix: program-wide L for every candidate (N_new_final + 226 + 48 + 30), raise INTRADAY >= 12 and add MM >= 3 with a stated derivation, label the rows convention, drop the two summary-row double counts.

### H9. Timing rule can be gamed (M:122, M:142, M:156)
N_new_final is read at Phase 1 close, but Phase 1 close is declared by the coordinator after seeing results, and reserve cells (80) can be released afterwards without recomputing DSRs already published. Keep registering until one candidate looks good, then close Phase 1.
Fix: freeze the candidate list (ids and K) by a dated commit before the first run; Phase 1 closes only when that list is exhausted; any later cell forces every candidate's DSR to be recomputed with the larger N (including after holdout).

---

## MEDIUM

- **M1 Sizing option.** `validation-protocol.md:149-151` allows scaling risk down and re-running when only the drawdown gate fails; the master does not forbid it and leaves sizing to candidate preregs (TEMPLATE s.4): an uncounted free parameter tuned after seeing MDD. Fix: sizing fixed in the prereg; a drawdown-only failure is KILL or a new counted cell.
- **M2 SPA/White (IG1, M:64).** No SPA, reality check or stationary bootstrap exists in `crates/` (searched). "p <= 0.05 for the best candidate" tests whether ANY class cell beats the benchmark, so a candidate whose base cell is not SPA-best passes on another cell's strength. Mixed benchmarks in one test and the "common D-OOS window" (shortest-history cell truncates everyone) are unspecified. Fix: studentized loss differential for THIS candidate's base cell against its benchmark, class cells as the model universe, window per class, code with tests before first use.
- **M3 PBO from 8 cells under mode F (M:34).** PBO measures overfitting of an argmax procedure; mode F has none (M:97). With 8-15 correlated neighbour cells, a real edge with a flat plateau (what V7/OG5 reward) has IS-best cells ranked about randomly OOS, so PBO is near 0.5 for true edges: V7/OG5 and V8 pull opposite ways. `pbo.rs:119` requires T to be a multiple of 16 (`RemainderDropped`); the trimming rule is unspecified (a free choice). Fix: report but do not gate PBO unless K >= 20 (v3's own scope), or specify trimming.
- **M4 OG4(a) double-counts V (M:48, M:95).** V-OOS is a subset of D-OOS, yet OG4(a) pools "D-OOS plus V-OOS" calendar years. Fix: D-OOS only; at holdout D-OOS plus H.
- **M5 OG4(b) is structurally hard for long-only trend books (M:48, M:194).** A book flat in down regimes has mean exactly 0, not > 0, so it must be positive in both up regimes plus one down regime. "Expanding-window median" does not say where the expansion starts; causal as written but a coder can implement a full-sample median. Fix: ">= 0 with minimum exposure" for flat regimes, state expansion start and as-of-t-1 computation.
- **M6 OG2 (M:46).** V-OOS and H compare weekly Sharpe "strictly above" B1/B2/B3 with no noise margin (a null equal to a benchmark passes about half the time); only D-OOS gets a bootstrap and only against B3. Requiring an equity candidate to beat BTC's weekly Sharpe in 2018-2021 can wrongly kill a real equity edge. Weekly series mix NSE 15:30 IST Friday closes with crypto 00:00 UTC. Fix: bootstrap lower bound vs EACH of B1/B2/B3 in D-OOS and V-OOS; state which benchmarks apply per class; define Friday alignment per calendar.
- **M7 Crypto data and FX (M:41, M:58, M:81).** Splicing `BTC-USD` to `BTCUSDT` by level on 2017-08-17 creates a spurious return (USDT premium/discount in 2017); chain returns. Pre-2017 Yahoo BTC-USD has gaps/stale prints and 10+5 bps costs are optimistic for 2014-2017; BTC-LONG adds about 3 y of the most favourable BTC trend sample to T. Gating in INR via `USDINR=X` credits every USD-hedged book (carry) with INR depreciation (about 2-4% a year) as free "return" and adds USDINR vol to weekly Sharpe; USDT = USD assumed. Fix: chain returns, inflate proxy-era costs, report INR and USD and gate on the less favourable for hedged books, filter bad USDINR prints.
- **M8 Tax and cost asymmetries (M:57-60, M:181-184).** (a) Only benchmarks are "taxed as if liquidated on the last day"; candidates' unrealised gains are untaxed unless the engine's `unrealised_at_end` path (used in `rerun-with-tax-2026-11.md`) is mandated. (b) GOLDBEES is not an equity-oriented fund: STCG at slab and LTCG 12.5% after 12 months post 2024-07-23 (20% with indexation before) (my knowledge, unverified), while M:57/M:184 apply `equity_cg()` to all ETFs; tax understated. (c) VDA no-loss-offset tax punishes active crypto books versus B2 taxed once; state the expected effect so a KILL is not read as "no edge". (d) B1 uses dividend-reinvested adjusted close while ^NSEI is a price index (M:191): index-priced P&L is handicapped about 1.3% a year. Fix: same liquidation rule for all; gold-ETF tax model; all series on one total-return basis.
- **M9 R2-OPTIONS (M:173).** See H1; also H = 2022-2026 spans the 2024 SEBI changes (weekly-expiry reduction, STT increases, lot-size changes) so a structure fit on 2012-2021 faces a different contract at H, and rates rows are verified only "where marked". Fix: weekly-only plus an explicit regime-break statement.
- **M10 Survivorship (M:79, M:166-167, M:176, R3).** `data-ref/nifty50_symbol_notes.csv` marks several membership dates "my knowledge", "approx" or "unverified" (STER, RANBAXY, CAIRN, SESAGOA alias, MCDOWELL-N), and Yahoo lacks history for several delisted members; the doc has no rule for members without prices (silent drop = survivorship) and no coverage threshold. Same for Binance delisted pairs in top-N crypto. Fix: per-class coverage report (member-days with price / member-days total); < 95% means diagnostic only.
- **M11 Priors and post-H weeks (M:14).** Agents propose "literature" base cells (SMA 200, 12-1 momentum 252/21/10, which is the exact backtest-04 cell already seen as a KILL, Donchian 55/20) with training knowledge covering much of 2022-mid 2026 index history; base-cell nomination is a promise with no ordering evidence. The paper benchmark (`paper/journal.csv`) starts 2026-11-05, after H ends (2026-10-31), so it does NOT overlap H; but 2026-11-01..11-16 is now seen (STATUS: NIFTYBEES +5.3%, BTC -14.8%), BTC's level at H's end is therefore effectively known, and V11 paper sessions must be counted only after the holdout batch. Fix: base cell cited to a literature reference; overlap with previously seen cells disclosed as "same cell, same data = no new information"; mark 2026-11-01 onward as seen and non-gating.

## LOW

- **L1** Master marker pins the draft text (`5a801475...`) and will not equal the tagged text after these fixes; it also uses one hash for strategy, data and split fields. M:152 says "the committed text". Fix: after tagging either append a second marker pinned to the tagged blob (+1 trial) or state that the marker pins draft v0 and the tag is authoritative; candidates' `split_spec_hash` = sha256 of the TAGGED file.
- **L2** The master marker stays OPEN (M:152). `options/study.rs:94` and `councileval/study.rs:143` use "first unclosed sweep of the family"; a harness copied from them would grab the marker and close it with a candidate's eval. Fix: close it now (cells=1, sr_var=0, merkle = sha256 of tagged text); it adds no variance.
- **L3** M:4 "edited by new commits that only TIGHTEN" contradicts the loosening edits this review requires (entry-day clause, OG5 grids). Pre-tag edits with no result seen are unrestricted; the tightening-only rule starts at the tag.
- **L4** M:121 N_all = N_new + 226 + 48 + 30 double counts the council-v1 and options-v1 summary rows inside the 226; B&H rows (5) are in L while labelled "not a trial".
- **L5** Fold boundaries for fixed-rule candidates: unspecified whether positions carry across folds or reset flat (resets create artificial entries/exits and costs). Fix: carry positions; folds only partition evaluation.
- **L6** TEMPLATE s.5 "price-free count of signal events" is not price-free and can be used to tune L until V2 clears. Fix: computed on T only, cannot change the grid, recorded as a flag.

## Template (`docs/prereg-r2-TEMPLATE.md`) gaps
Missing fields: kill-switch limit, risk per trade and max positions (H3, M1); DSR window and sr_var set (H2); fold-boundary position rule (L5); treatment of missing/delisted symbols and corporate actions, coverage threshold (M10); open-position tax at window end and TR basis of P&L (M8); declaration "this base cell equals prior cell X on the same data" (M11); class-level holdout burn and "no lineage descendant may use H" (C1). s.3 "Holdout guard: yes" should read "enforced by harness build `<git sha>`, test `<name>` passes". s.5 must require >= 3 levels per axis with centred base and the explicit ring-1 count. s.7 should record the sweep id and ledger tip hash at open.

## Answers to the specific attack lines
1. **Leakage.** No direct look-ahead in the regime/benchmark definitions as written (expanding median and quarterly B3 rebalance are causal if coded as of t-1; B3 weights are calendar-driven). Real routes: H1 (V is not unseen), H4 (adaptive/conditional selection), H6 (pipeline), C1 (holdout reuse), M7 (splice), M10 (survivorship), M11 (priors). The paper benchmark does not overlap H.
2. **Trial count.** L numbers verified. Issues: H2 (N_true not computed by the honesty layer; pooled-max variance), H8 (intraday/carry/MM undercount, row vs cell, family-specific L), H9 (timing), H5 (re-run option). Caps: DSR >= 0.95 needs a net D-OOS Sharpe of about 1.8-2.1 (2.3-2.6 with pooled-max variance). Not literally infeasible, but with C3 the jointly attainable set is almost empty; only low-vol carry could clear the Sharpe bars, and it cannot be claimed (contaminated holdout), fails OG5/V2, and V5 (Sharpe at rf 6%).
3. **Gates.** Weaker than v3 or v3-placeholder: kill-switch 30% (H3). Stricter without saying so: entry-day clause and 10-symbol rule for small books (C3). PBO at 8 cells is stricter and mis-specified (M3). v3 numbers otherwise restated accurately. Engine `tstats.rs:29` still implements the v2 holding-period bandwidth floor, so V3 needs new code to match the master's `floor(4 (T/100)^(2/9))` (validation-protocol changelog says so).
4. **Holdout contact.** None today (equity data end 2021-12-31; funding files to 2026-10 are carry-only and declared contaminated); C1, C2, H6, M11 are the paths that open once the data pull and harness exist.
5. **Other.** M4, M5, C3, L3, and the tagged-text/marker mismatch (L1).

## Minimal edits required before tagging

1. M:84, M:157: H touched once per class per program in one sealed batch; lineage descendants of a burned candidate get forward paper only (C1).
2. M:84, M:142, M:147, M:157, TEMPLATE s.3: require a committed, reviewed harness before any run: committed cap config, tag-ancestor and `strategy_hash` check at OPEN, candidate-id and class-keyed holdout guard with ledger marker before any H read, content-level max-date check, N_true DSR function, v3 HAC bandwidth, SPA/White with tests (C2, H2, M2).
3. M:28: delete the entry-day clause or restrict it per instrument; apply the 10-symbol and 15% tests only to books with >= 10 eligible symbols, otherwise cap at INCONCLUSIVE-SAMPLE (C3).
4. M:166-174: redesign grids to >= 3 levels per numeric axis with centred base; remove categorical axes from OG5 scope; add the per-family "can reach GO" table and drop families that cannot (C3).
5. M:35 and template: owner-signed kill-switch, risk per trade, max positions; one MDD threshold (H3, M1).
6. M:35, M:121, M:142: DSR on stitched D-OOS daily and monthly; sr_var defined as class-cell variance floored at 1/T plus pooled-max reported; replace the arithmetic with the H2 table; owner acknowledgement (H2).
7. M:14, M:85, M:99: per-class seen-window table including crypto 2020-2023, council-v1's consumed equity holdout and options T/V identical to options-v1; relabel V (H1).
8. M:146-150, M:174: freeze the candidate list before the first run or per batch with no cross-batch design dependence; delete the D-OOS-conditional council rule (H4, H9).
9. M:109, M:150, M:157: bug re-run protocol (red test first, whole sweep, both runs kept, less favourable governs) (H5).
10. M:103, M:84: sealed H store at ingestion; truncate membership/VIX/ADV/rank inputs at dev end; defer H quality checks (H6).
11. M:146-149: single ledger writer (H7).
12. M:114, M:133-138: program-wide L for every candidate; raise INTRADAY and CARRY floors with derivation; drop summary-row double counts (H8).
13. M:41, M:46, M:48: OG2 bootstrap vs each benchmark; OG4(a) on D-OOS only; OG4(b) flat-regime rule; Friday alignment (M4-M6).
14. M:57-60, M:81, M:184: liquidation tax for candidates; gold-ETF tax; common TR basis; return-chained BTC splice with proxy-era cost inflation; INR and USD dual reporting (M7, M8).
15. M:64, M:34: IG1 for the candidate's own base cell; PBO trimming rule or not a gate for mode F (M2, M3).
16. M:4, M:152: pre-tag vs post-tag edit rule; marker pins draft v0 and the tag is authoritative; close the marker (L1-L3).
17. TEMPLATE: add the missing fields above.

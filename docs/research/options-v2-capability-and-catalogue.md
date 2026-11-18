# options-v2: capability audit, strategy catalogue, margin model, cost/tax spec, engineering plan, grid design

Written 2026-11-17 (task T-002, decision D-006). Docs and read-only analysis only: no engine code changed, no backtest run, no ledger written, no sealed data read, nothing pushed. Branch `hardening/o2cap` off `hardening/integration` (09b7371). Nothing here is a result or evidence of profit.

Evidence convention: `path:line` refers to this worktree at 09b7371. "Unverified" means not read from a primary source in this task; every such item is listed again in section 9.

---------------------------------------------------------------------------------------------------

## 1. What exists today and what it assumes

### 1.1 `crates/options` (tq-options, the options-v1 engine)

It is a hard-wired replication of the options-v1 pre-registration, not a general options engine. Verdict: three structures, two underlyings, monthly only, development window only.

| Topic | What it does | Evidence |
|---|---|---|
| Underlyings | NIFTY and BANKNIFTY only; any other symbol row is silently skipped | `model.rs:8-12`, `bhav.rs:86` |
| Instruments parsed | `OPTIDX` rows only; index FUTURES rows ignored; stock options (`OPTSTK`) ignored | `bhav.rs:79`, doc comment `bhav.rs:68-69` |
| File format | Old-format bhavcopy header only (`FO_HEADER`); the UDiFF format (2024-07-05 onward) is rejected with a header error | `bhav.rs:24`, `bhav.rs:73-75` |
| Index closes | Names "Nifty 50"/"CNX Nifty"/"S&P CNX Nifty" and "Nifty Bank"/"CNX Bank" only; no FinNifty, Midcap Nifty, India VIX | `bhav.rs:131-135` |
| Expiries | Monthly only: retained rows are the latest expiry of each calendar month; weeklies are dropped at load | `store.rs:138-153`, signal-day logic `cycle.rs:37-54` |
| Cycle | One cycle per month: signal on the previous monthly expiry day, entry at next open, scheduled exit at the open of the session before expiry; no holding through expiry | `cycle.rs:63-90`, `cycle.rs:81-84` |
| Structures | `BullPutSpread`, `IronCondor`, `LongStraddle` (enum has exactly three); six hard-coded variants V1-V6; five hard-coded parameter cells | `structure.rs:14-19`, `29-36`, `54-81` |
| Strike selection | percent-of-spot offset and width; nearest listed strike with signal-day OI >= 500 rounded away from spot; no delta, no sigma, no IV | `structure.rs:12`, `117-146` |
| Fill rule | open of the fill day, adverse slippage `max(1% of premium, Rs 1)`, 2x on loss cuts; a leg is fillable only if fill-day volume > 0 and open > 0 | `fill.rs:9-21`, `model.rs:58-61`, `engine.rs:184-185`, `199` |
| Exit rule | take-profit and loss-cut evaluated on the daily settle, executed at the next open; an unfillable exit leg falls back to the last mark with 2x slippage | `engine.rs:114-142`, `196-210` |
| Valuation | daily mark at `SETTLE_PR` only for legs that traded; otherwise the last mark is carried and counted | `engine.rs:236-246`, `model.rs:63-66` |
| Sizing | 1% of closed-trade equity against structure max loss plus 2x one-way costs; hard cap 10 lots; `ZeroLots` skip | `engine.rs:18`, `152-175` (`166`), account Rs 25 lakh `engine.rs:48` |
| Margin | margin = max loss x lots (no SPAN); cap 60% of equity; no margin calls, no MTM funding | `engine.rs:19-20`, `170-173` |
| Position count | one position per variant at a time; a second signal inside a live position is an `Overlap` skip | `engine.rs:256`, `268-271` |
| Expiry handling | none: no settlement, no exercise STT, no physical settlement; positions never reach expiry by construction | `cycle.rs:81-84` |
| Lot sizes | own CSV table for NIFTY/BANKNIFTY only; 100% of v1 trades used unverified lots | `lots.rs:9-23`, `45`; result-options-v1-dev.md V1 |
| Costs | STT-by-date, exchange, SEBI, stamp, GST, Rs 20 per order, hard-coded in the crate (NOT read from the `tq-costs` RateTable); flat 0.003% stamp on all dates; no futures, exercise STT or physical settlement | `costs.rs:26-66`, `51` |
| Tax | own flat-rate model: 31.2% on positive FY net, in-year netting, 8-year carry-forward, paid on the last session of the FY | `tax.rs:32-69` |
| After-tax series | `(pnl - tax)/equity_prev`, with a silent `0.0` return when equity <= 0 (ruin is hidden, not flagged) | `metrics.rs:25-37` (line 32) |
| Window | constants 2012-01-01..2021-12-31; any later file is refused by `guard_date` | `window.rs:8-14`, `29-37` |
| Folds | 14 half-year folds 2015-H1..2021-H2 hard-coded | `metrics.rs:39-53` |
| Study | ledger family constant `options-v1`, 6 x 5 cells, gates O1-O8 constants, DSR at N = 30 | `study.rs:27`, `gates.rs:6-21` |
| Idle cash | optional interest helper (6%); the gating series in research-v2 uses 0 | `metrics.rs:114-125`, prereg-research-v2.md 7.4 |
| Integrity | `tq-bias` options source is bound to the v1 store and v1 `select_legs` | `crates/bias/src/options.rs:11-14`, `38-60` |
| Synthetic data | one toy generator, 760 sessions, monthly expiries only | `synth.rs:11-14` |

Worth keeping as design: point-in-time signal days (judged from each day's own file), the prefix-invariant `ChainStore::truncate_to` horizon, the fail-closed holdout guard, the price-free structural check, skip accounting by reason, deterministic runs, Decimal money.

What v1 cannot do: weeklies; any underlying but two; stocks; futures legs (covered call, protective put, delta hedge); settlement at expiry; more than three structures; calendar and diagonal (`plan_cycle` has ONE expiry per plan, `cycle.rs:14-24`); IV and Greeks; SPAN-like margin; event calendars; concurrent positions; date-keyed costs from the shared rate table; the 2022+ data format; windows other than 2012-2021.

### 1.2 `crates/costs` (rates table, TaxModel)

* `RateTable::india()` is date-keyed with `verified` flags (`rates.rs:68-75`). It carries F&O options STT only from 2024-10-01 (`rates_data.rs:38-43`: futures 0.02% then 0.05%; options sell 0.10% then 0.15%; exercise 0.125% then 0.15%) and index lots only from 2024-11-20 (`rates_data.rs:47-55`). A lookup for 2021-11..2024-09 returns `None` (`rates.rs:69-71`), so the table cannot price Phase 1 before 2024-10 and cannot price Phase 2 at all. `tq-options` carries its own 2013..2026 STT steps instead (`costs.rs:28-38`): two sources of truth.
* `TaxModel` has `None`, `EquityCg`, `Vda` only (`taxmodel.rs:24-28`). There is no business-income model; `equity_fy_tax` is per-FY with no carry-forward (`taxmodel.rs:95-99`). The options 31.2% business-income model lives only in `tq-options/tax.rs`.
* `CostSchedule` has the cash-equity shape (brokerage cap, STT buy/sell, DP) (`lib.rs:41-53`); no per-leg futures/options/exercise variants.

### 1.3 The research-v2 harness (r2gov, r2stats, r2run) and what it assumes

| Assumption | Evidence |
|---|---|
| The engine is a daily-bar, LONG-ONLY, one-account event engine; signals are only `Enter`/`Exit` of one symbol | `crates/backtest/src/lib.rs:1`, `crates/strategy/src/lib.rs:12-17` |
| A candidate is a `Rule` building either a `Signals` strategy or a month-end `Rebalance` weights strategy; `EngineKind` has exactly those two | `rule.rs:22-27`, `spec.rs:16-21` |
| A "universe" is a list of bar series in the sealed development store; the run fails if any is missing | `cellrun.rs:169-180` |
| Costs are the cash-equity delivery schedule or crypto spot fees, scaled for the 2x stress | `setup.rs:133-138` |
| Tax is `equity_cg` or `vda_statutory` only | `setup.rs:144-150`; applied via `equity_net_returns_with`/`crypto_net_returns_with` `cellrun.rs:199-204` |
| Sizing is `Sizing::RiskPct` against a stop; slippage is one bps figure per class | `setup.rs:108-120` |
| The evaluator wants daily NET returns (after costs, slippage, funding, tax; liquidation-taxed; no idle-cash interest) | `inputs.rs:1-2` |
| Exposure per day is REQUIRED, finite, in [0, 1], aligned with returns; empty means NOT-EVALUATED | `inputs.rs:57-77` |
| A trade is a flat-to-flat round trip `{symbol, net_pnl}`; `open_positions_per_day` feeds k_bar | `inputs.rs:80-90`, `cellrun.rs:122-127`, r2run-notes.md "B. Closed trades are flat-to-flat" |
| N_eff = N_raw / DE, DE = 1 + (k_bar - 1) rho_bar; rho_bar from the universe's own daily underlying returns; with fewer than two series it falls back to 1, so DE = k_bar | `neff.rs:1-31`, `56`; prereg-research-v2.md 2.5 |
| Benchmarks B1/B2/B3 are cost-and-tax-paying buy-and-hold series; the class benchmark is NIFTYBEES or BTC close levels | `inputs.rs:92-101`, `176-177`; `r2stats/src/bench.rs:21-45` |
| Folds: 3-year train, 6-month test (India), purge max(hold,10) or "one option cycle = 21 trading days" for IN-OPT, embargo 21 days; positions carry across folds (one continuous run) | `folds.rs:10-16`, `cellrun.rs:1-2` |
| A non-physical series (any daily return <= -100%) makes dependent gates NOT-EVALUATED | `r2stats/src/physical.rs:1-12`, `29-32` |
| IN-OPT exists as an `AssetClass` but has only diagnostic families (36 cells, `r2-diag-options-weekly`) that can never reach a holdout; the catalogue marks it NOT-RUN (engine limitation) | `inputs.rs:14-19`, `harness-r2-notes.md` item 9, `r2run/src/catalogue.rs:88` |
| The data step ingests index and VIX only; the 15 GB `fo` bhavcopy is NOT ingested and there is no typed bhavcopy loader in the sealed store | r2run-notes.md "Gaps" |
| Ledger family, caps and the registrar are keyed to `research-v2` (450 total, 132 reserve); an options-v2 family needs its own caps/ledger per D-006 | `research/caps.json`, `harness-r2-notes.md` items 2-4 |
| Statistics reusable as is: DSR with N_true, PBO/CSCV, SPA/White, HAC t, N_eff, MC drawdown, calibration generators | `r2stats/src/lib.rs` modules, `r2stats/src/calib/*` |

### 1.4 Gap list (each with evidence)

| # | Gap | Evidence | Closed by (section 6) |
|---|---|---|---|
| G1 | No weekly expiries (dropped at load) | `store.rs:138-153` | M0, M1 |
| G2 | Two underlyings only; no FinNifty, Midcap Nifty, stocks | `model.rs:8-12`, `bhav.rs:86`, `131-135` | M0, M2 |
| G3 | Old-format parser only; Phase 1 (2021-11..2026-10) includes the UDiFF format | `bhav.rs:24`, `73-75` | M0 |
| G4 | No futures rows (covered call, protective put, delta hedge, forward for IV) | `bhav.rs:79` | M0 |
| G5 | Three structures; one expiry per plan (no calendar/diagonal) | `structure.rs:14-19`, `cycle.rs:14-24` | M4 |
| G6 | No IV, no Greeks, no delta-based selection | none in crate | M3 |
| G7 | No expiry settlement, exercise STT, physical settlement | `cycle.rs:81-84` | M1, M7 |
| G8 | Margin = max loss; no SPAN-like model, no naked, no peak margin | `engine.rs:170` | M6 |
| G9 | One position at a time; no overlapping weeklies, no hedge book | `engine.rs:256`, `268-271` | M9 |
| G10 | Date window and folds are constants | `window.rs:8-14`, `metrics.rs:39-53` | M0 (config), M10 |
| G11 | Costs duplicated and incomplete: shared table lacks pre-2024-10 STT and all pre-2024-11 lots; options crate hard-codes; flat stamp; no futures/exercise legs | `rates_data.rs:38-55`, `costs.rs:26-66` | M7 |
| G12 | No business-income tax model in `tq-costs`; r2 setup cannot select one | `taxmodel.rs:24-28`, `setup.rs:144-150` | M8 |
| G13 | After-tax return hides ruin by returning 0.0 when equity <= 0 | `metrics.rs:32` | M8 |
| G14 | r2 engine seam is bar-and-signal only: cannot accept an external daily series, exposure and trade book | `cellrun.rs:169-180`, `spec.rs:16-21` | M10 |
| G15 | r2 costs/tax/slippage by class cannot express options legs | `setup.rs:133-150` | M7, M10 |
| G16 | Sealed store has no chain reader; the allowlist does not know option files | r2run-notes.md Gaps; harness-r2-notes.md H-2 | M0, M10 |
| G17 | Caps, ledger family, gate thresholds are research-v2 only | `research/caps.json` | protocol agent (wt/o2proto) |
| G18 | Integrity (prefix invariance) source bound to v1 | `bias/src/options.rs:11-14` | M10 |
| G19 | No event calendars; no VIX loader | grep: none | M12 |
| G20 | v1 skip rule uses fill-day volume > 0 (a mild outcome-dependent skip) | `model.rs:58-61`, `engine.rs:184` | M5 |
| G21 | `retain_liquid_monthlies` discards zero-volume rows before it is known they are needed for IV | `store.rs:140-153` | M0 |

Runs TODAY: NIFTY and BANKNIFTY bull put spread, iron condor, long ATM straddle with the registered parameters, monthly, dev window 2012-2021, old-format files. Nothing else.

---------------------------------------------------------------------------------------------------

## 2. EOD feasibility rules (these define RUNNABLE)

### 2.1 Timing
Signal on the close of session t using only data dated <= t. Entry at the OPEN of t+1 (as v1). Decisions from settle/close marks execute at the next open. Prefix-invariance is testable through `ChainStore::truncate_to`.

### 2.2 Conservative fill rule and liquidity filter
* Fill price = open of the fill day, adverse by `s` (4.4). An open outside [low, high], or a row with high/low > 3 on a contract under Rs 5, is a bad print: no fill, counted.
* Eligibility is decided on SIGNAL-DAY data (v1 used fill-day volume, G20): each leg needs signal-day volume >= Vmin and OI >= Omin, tiered per 4.4 (index weekly legs: volume >= 100 and OI >= 1,000 minimum; v1 used OI >= 500). Fill-day `open > 0` is still required; a non-fill is a skip and the skip rate is reported (gate <= 15% as v1).
* Sensitivity fill rules (same trials, not new): fill at (O+H+L+C)/4 plus slippage; fill at settle plus slippage; T+2 shift (v1 found V1 flips sign under it: result-options-v1-dev.md "Shift test").
* Why: bhavcopy has no quotes; `CLOSE != SETTLE_PR` on 91% of 2019 NIFTY rows and 90.6% of those rows had zero volume (`data-feasibility-options.md` section 3). The open of an illiquid strike is a poor price, so tiering makes the assumption explicit.

### 2.3 Exits and valuation
Daily mark = settle for contracts with volume > 0, else carry the last mark and count it; a trade whose carried marks exceed 20% of leg-days is flagged mark-quality-low (reported, not gated). Exits: scheduled (D-1 open for monthly, a fixed holding rule for weekly), plus take-profit/loss-cut on settle executed at next open with 2x slippage on loss cuts.

### 2.4 Weekly availability (UNVERIFIED, from memory; the o2data inventory is the authority)
Nifty weekly from 2019-02 (Thursday; Tuesday from 2025-09); BankNifty weekly 2016-10 to about 2024-11; FinNifty weekly about 2021-01 to 2024-11; Midcap Nifty weekly about 2022 to 2024-11; from 2024-11-20 one weekly benchmark per exchange (`tax-india-fno.md` section 2). Stocks: monthly only. Expiry dates are inferred from each day's file, never from a calendar rule (`data-feasibility-options.md` item 5).

### 2.5 Expiry settlement
* Index options are cash settled at the final settlement price (index close on expiry day). Exercise STT is paid by the buyer on intrinsic: 0.125% before 2026-04-01, 0.15% from (`tax-india-fno.md` section 1). The engine can model hold-to-expiry exactly: intrinsic from the index close, cross-checked with `SETTLE_PR` on expiry day (whether they match is unverified). A long ITM wing of a spread held to expiry pays exercise STT on its intrinsic even though the position is economically hedged; v1 avoided this by never holding. v2 offers both rules as a registered structure parameter ("exit D-1 open" or "hold to settlement").
* Stock options are physically settled (phased in from about 2019; cash settled earlier: unverified). Obligation = lot x strike notional, delivery STT, a margin ramp in the final days. NOT modelled: every stock structure exits before the final session (F-X2 NOT-RUN, other).
* Expiry-day structures: EOD gives that day's open, high, low, close, settle per contract, so only "enter at the expiry-day open, exit at settlement" can be tested, with a conservative intrabar order (a stop is assumed hit whenever the day's range crosses it, before any target). Entry at 9:20, intraday stops, VWAP rules and true 0DTE path dependence are NOT supported (NR-D).

### 2.6 Delta-hedged variants (daily futures hedge)
Possible: the bhavcopy carries index and stock FUTURES rows (OHLC, settle, OI). Hedge ratio from derived delta on day-t data; executed at the futures open of t+1; costs: futures STT (sell) 0.02% from 2024-10-01, 0.05% from 2026-04-01 (`rates_data.rs:38-39`; earlier steps unverified), exchange 0.00183%, stamp 0.002%, Rs 20 brokerage, one tick of slippage; futures margin from the SPAN-like model (3.3). Cannot support: intraday gamma scalping, intraday hedge bands, hedging at better-than-open prices. Realised-versus-implied P&L is measured at daily frequency and discrete-hedging error is large: a stated limitation.

### 2.7 IV derivation
Black-76 on the option SETTLE (`CLOSE` is last trade and often stale) with the same-expiry future's settle as forward F; cross-check F from put-call parity at the strike with smallest |C-P| among strikes with volume > 0 on both legs: F = K + e^{rT}(C-P). r = 6% flat (second-order for T under a month). T in calendar days/365 to 15:30 IST of expiry. Bracketed Brent on [0.01, 5].
Failure modes (each returns `None` and a counted reason, never an imputed value): price below intrinsic or above its bound; premium at the Rs 0.05 tick floor (IV is a bucket); zero-volume row (the settle is an exchange-computed price, not a trade); parity forward and future disagree by more than 0.5% (stale leg); expiry day (T -> 0); stock options with dividends before expiry (use the future-implied F, which embeds them); deep ITM (time value inside tick noise); close-versus-settle timing mismatch (index closes at 15:30, option settle is a last-half-hour VWAP). India VIX (index files, from 2014-10; zero rows earlier, `research-v2-final-report.md` data section) is a 30-day interpolated OTM measure: use only as a sanity check of the derived 30-day ATM IV, never as an input to weekly pricing.

### 2.8 Event calendars
* Budget: public dates, about one a year (2012-2026 roughly 16 sessions including interim, special and weekend sessions). Exact dates and citations: unverified; to be committed as a CSV with a source per row.
* RBI MPC: scheduled meeting calendar published in advance; announcement is intraday (about 10:00), so only close-to-close or overnight windows are testable.
* Company results: NSE corporate filings list board-meeting dates (free). Whether the archive records WHEN each date was announced (point-in-time) and rescheduling is not established; without an intimation timestamp a date can leak the future. F18c is RUNNABLE with a restriction (only meetings whose intimation precedes entry); if the source has no intimation date, F18c becomes NOT-RUN (data unavailable) at the data step.

---------------------------------------------------------------------------------------------------

## 3. Margin model design

### 3.1 Principle
Exact SPAN is not reproducible from bhavcopy. Margin matters for the feasibility cap (deployed margin <= 60% of equity, as v1), for sizing of naked stress books, for the required exposure series, and for capital-efficiency reporting. The gating return series is account-level with idle cash at 0, so margin precision does not enter defined-risk returns; it does enter the naked stress test, which therefore gets the conservative model.

### 3.2 Defined-risk structures (documented approximation)
margin = maximum loss of the expiry payoff over all underlying levels (exact for verticals, condors, butterflies, BWBs, back-ratios; premium paid for long options), found by scanning the piecewise-linear payoff so any leg list works, plus round-trip costs; no exposure margin on fully hedged legs. This generalises v1 (`engine.rs:170`; prereg-options-v1.md section 3). It is conservative against broker SPAN where far wings give scenario loss below width. Calendar and diagonal: max loss = net debit (credit versions use 3.3). Covered call and protective put: futures margin from 3.3 with the option as a scenario leg.

### 3.3 SPAN-like scenario engine (futures legs, calendars, naked stress)
Revalue the whole same-underlying position with Black-76 under the standard SPAN 16-scenario array: price moves 0, +-1/3, +-2/3, +-1 of the price scan range (PSR), each with volatility up and down by the volatility scan range (VSR) (14 scenarios), plus two extreme moves of +-3 PSR counted at 35% of the loss. Scan risk = largest scenario loss. Inter-month spread charge comes from revaluing each expiry at its own derived IV (the calendar benefit emerges from the scenarios, no separate table). Add a short-option minimum charge and exposure margin (ELM).
All parameters are UNVERIFIED approximations stored in a date-keyed table with `verified = false`: PSR = max(floor, k x trailing 21-day EWMA daily vol x sqrt(2)) with floor and k calibrated if 3.5 delivers parameters; VSR floor 4 vol points; ELM: index options 2% of notional on short positions, stock options max(5%, 1.5 sigma of 3-month return); short-option minimum charge: percent of notional (index about 3%) [all recollection]. The stress book is sized from the model margin x 1.25 (sensitivities 1.0 and 1.5).

### 3.4 Rules by date (UNVERIFIED unless cited)
| Effective | Rule | Source / status |
|---|---|---|
| to 2020 | SPAN + exposure margin on end-of-day positions; full spread benefit | recollection |
| 2020-12 to 2021-09 | peak margin phased 25% / 50% / 75% / 100%; 100% from 2021-09-01 (intraday snapshots) | SEBI circular July 2020, recollection |
| 2021-09-01 onward | peak margin 100%. EOD cannot see the intraday peak: apply a peak uplift to naked stress books (1.10 base, 1.25 sensitivity) | assumption |
| 2024-11-20 | minimum contract value Rs 15 lakh (lot revisions); one weekly index per exchange; extra extreme-loss margin on short options on expiry day (+2%; one source says 12%) | `tax-india-fno.md` section 2 (rate unsure) |
| 2025-02-01 | calendar-spread benefit removed on expiry day (near leg margins standalone); upfront premium from option buyers (no effect: premium is paid at entry in our cash model) | `tax-india-fno.md` section 2 (single source) |
| 2025-04 / 2025-10 | intraday position-limit monitoring | single source; no EOD effect |
| stock options, physical-settlement era | margin ramps in the last days before expiry (not modelled: stock structures exit earlier) | recollection |

### 3.5 NSE SPAN risk parameter files: free? point-in-time?
NSE Clearing publishes SPAN risk parameter files (begin-day, intraday and end-of-day sets) on public NSE pages ("equity-derivatives-span-risk-parameter-files" and "all-reports-derivatives", the latter listing a historical SPAN entry dated 2019-01-01, per a web search in this task). Access is therefore plausibly FREE. NOT verified: the download URL pattern, how far back the archive goes, whether each day's files are retained (point-in-time) or only the latest set, and the terms of use. A direct download probe was declined by the permission system and not retried. Status: UNVERIFIED.
If the archive is daily and deep: parse the per-contract risk arrays and PSR/VSR per underlying, and replace the approximations in 3.3 with the exchange's own scan risk. If only recent files exist: use them to calibrate PSR/VSR/minimum charge on recent years and keep the approximation for older years, reported. Either way broker margin adds its own multiples, so the model stays an approximation. This is a data-agent (o2data) follow-up, not an engineering dependency.

### 3.6 Naked selling: labelled stress test only
* Registered as `STRESS-NAKED`, fixed rule (no tuning), ineligible for development GO and promotion, reported in a separate table titled "stress test, not a strategy".
* Margin = 3.3 model x 1.25. The hard loss cap is enforced EX ANTE by sizing: lots = floor(cap_frac x equity / stress_loss_per_lot), stress loss = largest scenario loss at 2 x PSR with vol +50%, cap_frac = 2% of equity; deployed margin <= 25% of equity.
* EX POST: a stop is evaluated on the settle and executed at the next open with 2x slippage; any loss beyond the cap (a gap) is booked as it happened and reported as a cap breach (count and size). The cap is never an accounting floor (that would be a look-ahead fudge). After a breach larger than 2x the cap the stress book halts for the rest of the window, reported.
* No stocks (earnings and corporate-action gaps make a cap meaningless).

---------------------------------------------------------------------------------------------------

## 4. Cost model spec for weekly (and all) option structures

### 4.1 Per-leg charges (source: one date-keyed table, extended; every row carries `verified`)
| Charge | Rule | Dates / status |
|---|---|---|
| Brokerage | Rs 20 per executed order, each leg an order (stress 2x; early years understated) | `costs.rs:49`; costs-tax-options-india.md section 1 |
| STT, option SELL premium | 0.017% to 2016-05-31; 0.05% from 2016-06-01; 0.0625% from 2023-04-01; 0.10% from 2024-10-01; 0.15% from 2026-04-01 | `costs.rs:26-38`; only 2026-04-01 verified (NSE/FATAX/73524, `tax-india-fno.md` section 8), the rest secondary |
| STT, option exercised (index, cash settled, buyer, on intrinsic) | 0.125% to 2026-03-31; 0.15% from 2026-04-01; the table (`rates_data.rs:42-43`) has 2024-10-01 onward only | earlier history secondary/unsure |
| STT, physical settlement (stock) | not modelled (F-X2) | n/a |
| STT, futures SELL | 0.0125% to 2024-09-30 (memory), 0.02% from 2024-10-01, 0.05% from 2026-04-01 | `rates_data.rs:38-39`; only the last verified |
| Exchange charge, options, % of premium | 0.05% before 2024-01-01 (conservative stand-in), 0.03553% (incl. IPFT) from 2024-01-01 | `costs.rs:41-47`; table verified from 2026-03-01 only (`rates_data.rs:46`) |
| Exchange charge, futures | 0.00183% (verified from 2026-03-01; earlier unverified) | `tax-india-fno.md` section 1 |
| SEBI fee | Rs 10 per crore of turnover | `costs.rs:50` |
| Stamp duty | 0.003% options buy, 0.002% futures buy, from 2020-07-01; earlier state-wise (v1 applies 0.003% to all dates, conservative) | `costs.rs:51`; date unverified |
| GST | 18% on brokerage + exchange + SEBI (not STT, not stamp) | `costs.rs:60` |
| Clearing, DP, platform fees | none modelled | unverified |

Weekly consequence: a 4-leg condor is 8 orders (Rs 160 + GST = Rs 188.8) per round trip regardless of size, about 50 times a year per underlying. On a 75-unit lot with Rs 20-60 of weekly credit (Rs 1,500-4,500 gross) fixed cost alone is 4-12% of gross before STT and slippage. Costs must be computed from actual lots, never as a notional percentage.

### 4.2 Rounding and structure
Per order unrounded, total rounded once per trade to paise (`costs.rs:1-2`, `lib.rs:7-11`). One `CostBook` keyed by (date, instrument kind, side, leg type) is the only source; the `tq-options` constants and `tq-costs` rows are reconciled into it (M7).

### 4.3 Stress
1x, 2x, 3x slippage and 2x brokerage (as v1, `study.rs:182-189`), plus a bound run with every unverified rate set to its upper plausible value. Every result prints `unverified rates: k of m` (r2 convention).

### 4.4 Slippage per leg as a function of premium and liquidity (assumption)
No quotes exist, so this is a stated convention, stress-tested. Half-spread proxy by tier on SIGNAL-DAY data:

| Tier | Signal-day volume (contracts) and OI | Slip per unit per side `s` |
|---|---|---|
| A (ATM and near-ATM of Nifty/BankNifty) | volume >= 2,000 and OI >= 5,000 | max(Rs 0.50, 0.5% of premium) |
| B | volume 100-2,000 or OI 1,000-5,000 | max(Rs 1.00, 1.5% of premium) |
| C (illiquid) | below B | not tradable: skip, counted |
| Stock options | volume thresholds one tenth of the index ones | one tier worse than the index equivalent |

x2 on stop/loss-cut exits and x2 on expiry-day open fills (wide opening spreads). Justification: the NSE tick is Rs 0.05 and liquid ATM spreads are a few ticks (about 0.05-0.2% of a Rs 100-200 premium), but far-OTM wings of Rs 5-20 trade 0.5-1.0 wide (5-10% of premium); a pure percent rule overcharges ATM and undercharges cheap wings, the floor corrects this. Tier thresholds are guesses covered by the 1x/2x/3x stress; the real spread distribution is unknown (`costs-tax-options-india.md` section 2).

### 4.5 Tax and the after-tax series
* Business (non-speculative) income at slab rate; modelled as flat 31.2% (30% + 4% cess; surcharge and new-regime slabs ignored) on positive FY net after charges and STT (STT deductibility unverified); losses netted within the FY; carry-forward 8 assessment years against future business income only (if the return is filed on time); tax paid at FY end (`tax.rs`, `tax-india-fno.md` section 5).
* DEFINITION of the gating after-tax series (recommended): liquidation-basis equity. Daily return = change in (pre-tax account equity - tax liability accrued on realised-plus-marked YTD net business income after using the loss pool) divided by prior-day equity. The liability is >= 0 (no deferred-tax asset is booked, conservative) and falls as losses mount, so tax cannot exceed equity unless pre-tax equity is itself ruined. The research-v2 crypto problem (VDA: no loss offset, tax on every winner, negative equity) does not arise for business income because losses offset gains inside the year and carry forward.
* Ruin: a day with after-tax equity <= 0 is a TERMINAL EVENT flagged in the output (`ruin_date`), the candidate is a KILL by definition, the series is truncated. The v1 behaviour of emitting 0.0 silently (`metrics.rs:32`) is a defect to fix (M8); without the fix the r2 guard (`physical.rs:29-32`) would mark gates NOT-EVALUATED, which is wrong for a genuine ruin.
* Sensitivities (same trials): 20% and 39% rates; no loss carry-forward (worst case); symmetric tax (loss refunded at 31.2% in the year it arises, an upper bound representing set-off against other income); the pre-tax series is always reported beside.
* Open owner question OQ1 (section 8): other income against which F&O losses can be set off. Not blocking: both are computed.
* Not modelled: tax-audit threshold and CA fee, advance-tax interest, turnover basis (`tax-india-fno.md` section 4).

---------------------------------------------------------------------------------------------------

## 5. Strategy catalogue (family x underlying class x expiry type)

Definitions. RUNNABLE = free EOD data is sufficient AND the section 6 plan implements it; the table gives the endpoint of the plan, not today's state (today's state is the end of 1.4). Codes: `R` runnable; `R-L` runnable but liquidity-limited (the filter will skip many cycles; skip rate reported against the 15% gate, and for FinNifty/Midcap weekly only a window of years exists, 2.4); `R-S` runnable ONLY as a labelled stress test; `NR-D` not run, data unavailable; `NR-O` not run, other (reason in notes); `NR-E` not run, engine limitation (none remain after the plan; used only if a module is cut). Underlying classes: Nifty/BankNifty, FinNifty/Midcap Nifty, most liquid stock options. NSE lists no stock weeklies, so the stock column is monthly only. Weekly-only and expiry-day behaviour for stocks are `NR-O`.

| ID | Strategy family | Nifty/BankNifty monthly | Nifty/BankNifty weekly | FinNifty/Midcap monthly | FinNifty/Midcap weekly | Stocks monthly | Notes |
|---|---|---|---|---|---|---|---|
| F1 | Long call / long put (outright) | R | R | R-L | R-L | R | liquidity filter binds on FinNifty/Midcap; weekly FinNifty/Midcap exist only in a window (see 2.4) |
| F2 | Covered call (long future or ETF + short call) | R | R | R | R-L | R | needs index/stock FUTURES rows (M0); stock: exit before expiry (physical settlement) |
| F3 | Protective put (long future/ETF + long put) | R | R | R | R-L | R | as F2 |
| F4 | Cash-secured put (short put, strike notional reserved) | R | R | R | R-L | R | defined risk to zero; capital heavy; stock hold-to-expiry assignment is F-X2 NOT-RUN |
| F5 | Vertical spreads: bull/bear call and put spreads (4 variants) | R | R | R-L | R-L | R | stock strike grids are coarse, skips expected |
| F6 | Calendar spread (same strike, two expiries) | R | R | R-L | R-L | R-L | far-leg liquidity filter; calendar margin benefit modelled (3.4) |
| F7 | Diagonal spread | R | R | R-L | R-L | R-L | as F6 |
| F8 | Long straddle / long strangle | R | R | R-L | R-L | R | v1 lot-granularity skips (36-53%) fixed by sizing change (5.3) |
| F9 | Short straddle / short strangle (NAKED): STRESS TEST ONLY | R-S | R-S | R-S | R-S | NR-O | labelled stress, hard loss cap, never a candidate for GO (3.6); stocks: not registered, earnings and corporate-action gaps make a loss cap meaningless |
| F10 | Iron condor | R | R | R-L | R-L | R-L |  |
| F11 | Iron butterfly | R | R | R-L | R-L | R-L |  |
| F12 | Broken-wing butterfly | R | R | R-L | R-L | R-L | needs 3 liquid strikes on one side |
| F13 | Ratio spreads (back-ratio defined risk; front-ratio = stress label) | R | R | R-L | R-L | R-L | front ratio has undefined tail: F9 rules apply |
| F14 | Jade lizard (short put + short call spread) | R-S | R-S | R-S | R-S | NR-O | put side is undefined risk: stress label, margin per 3.6; stocks not registered |
| F15 | VRP selling, DEFINED risk (conditional entry on derived IV vs realised vol) | R | R | R-L | R-L | R-L | depends on IV derivation (2.7); entry filter uses only signal-day data |
| F16 | VRP selling, NAKED: stress test | R-S | R-S | R-S | R-S | NR-O | same as F9 |
| F17a | Expiry-day structures: open-to-settlement only (conservative intrabar order) | R-L | R-L | R-L | R-L | NR-O | EOD supports only open entry and close/settlement exit; stocks: no weekly, monthly expiry day is physical settlement |
| F17b | Expiry-day / 0DTE with intraday entry (9:20), intraday stop/target, VWAP rules | NR-D | NR-D | NR-D | NR-D | NR-D | no intraday option prices |
| F18a | Event: Budget (overnight/close-to-close windows) | R-L | R-L | R-L | R-L | NR-O | about 1 event a year: sample cannot be reached (6.3); stocks: not an event for single names |
| F18b | Event: RBI MPC (overnight/close-to-close windows) | R-L | R-L | R-L | R-L | NR-O | 6 a year; announcement is intraday so only close-to-close windows |
| F18c | Event: company results | NR-O | NR-O | NR-O | NR-O | R-L | index options have no results event; stock dates need a point-in-time source (2.8) |
| F19 | Directional overlay on a trend filter (long call/put or debit spread) | R | R | R-L | R-L | R | filter uses underlying closes only |
| F20 | Delta-hedged variants (daily futures hedge, long or short gamma) | R | R-L | R | R-L | R-L | daily hedge only; intraday gamma P&L is not observable (2.6) |
| F-X1 | IV-surface / skew / term-structure trading needing vendor-quality IV | NR-D | NR-D | NR-D | NR-D | NR-D | derived-IV is noisy and EOD-timed |
| F-X2 | Stock options held through physical settlement (assignment, delivery STT, margin ramp) | NR-O | NR-O | NR-O | NR-O | NR-O | not modelled: all stock structures exit before expiry (2.5) |
| F-X3 | Market making, spread capture, tick or quote-based fills | NR-D | NR-D | NR-D | NR-D | NR-D | no bid/ask, no depth |

Cell counts (130 cells = 26 rows x 5 columns): RUNNABLE 100 (R 40, R-L 48, R-S 12); NOT-RUN 30 (data unavailable 15, other 15, engine limitation 0). RUNNABLE TODAY with the unmodified engine: 6 cells (bull put spread, iron condor, long straddle on Nifty and BankNifty, monthly, development window 2012-2021).

### 5.1 NOT-RUN detail and the paid data that would unlock it (UNVERIFIED and UNPRICED; no vendor contacted)
| Item | Why not run | Paid data that would unlock it |
|---|---|---|
| F17b intraday expiry-day / 0DTE, F-X3 market making | EOD gives one observation a day: no intraday path, no bid/ask | 1-minute or tick option data with quotes; vendors named in `data-feasibility-options.md` section 5 (TrueData, Global Datafeeds, NSE data products); broker APIs need accounts (excluded by hard rule 1) and usually lack expired contracts |
| F-X1 vendor-quality IV, skew, term structure | derived IV is EOD, timing-noisy, stale on thin strikes | historical IV surface / Greeks product from the same vendor class |
| F18c if no point-in-time source | announcement timestamps unknown | vendor corporate-event calendar with announcement dates |
| F-X2 physical settlement | not modelled by design (stock structures exit before expiry) | none needed: an engineering decision |
| F9/F14/F16 on stocks; F17a/F18a/F18b on stocks; F18c on index | not meaningful, not registered, or not listed | n/a |

Every owner-listed strategy appears; none is called a failure. Defined-risk VRP selling runs first; naked stays a labelled stress test, never a default (CURRENT_TASK non-goals, D-006 item 8).

---------------------------------------------------------------------------------------------------

## 6. Engineering plan

### 6.1 Placement
Keep `tq-options` FROZEN (its code sha, manifest and lot hashes are cited by the registered v1 result). Add a NEW crate `crates/opt2` (`tq-opt2`). Acceptance test of the new loader and engine: the 30 v1 cells reproduce exactly on the 2012-2021 data.

### 6.2 Modules
| Module | Content | Reuses |
|---|---|---|
| M0 loader | typed chain/futures/index-close/VIX reader over o2data's compact schema (both bhavcopy eras), all underlyings, weeklies kept, window guard driven by config (Phase 1 train/validation/sealed; Phase 2), prefix view `truncate_to` | `tq-options/store.rs` design; r2gov sealed store and `DevReader` (new chain reader) |
| M1 calendar and expiry | expiry classes from file contents, expiry-weekday regimes, settlement day, exit-before-expiry vs hold-to-settlement, cash settlement from index close, expiry-day sessions | `cycle.rs` ideas |
| M2 contracts and lots | lot table for all underlyings with verified flags, reconciled with RateTable `lot.*` keys | `lots.rs`, `rates_data.rs:47-55` |
| M3 pricing | Black-76, Brent IV, forward from futures/parity, Greeks, failure counters, VIX cross-check | none |
| M4 structures | generic leg list with payoff scanner (max loss, max gain, breakevens, defined/undefined flag); strike selection by percent, sigma or delta; families F1-F20 | v1 selection as regression |
| M5 fills and liquidity | 2.2 and 4.4 rules, tiers, bad-print rules, sensitivity fill modes | `fill.rs` |
| M6 margin | payoff-scan margin, 16-scenario engine, ELM, minimum charge, date-keyed rules, peak uplift, stress sizing | none |
| M7 costs | `CostBook` by date, leg type, side; add historical STT/stamp/exchange rows and lots to the shared table with `verified` flags; futures and exercise legs | `costs.rs`, `tq-costs` RateTable |
| M8 tax | business-income `TaxModel` variant in `tq-costs`, liquidation-basis accrual series, ruin flag, sensitivities | `tax.rs`, `TaxModel` |
| M9 book engine | multi-position overlapping book, daily MTM of all legs and hedges, triggers, scheduling, event entries, daily futures hedge; outputs `dates, pre_tax, net, exposure, open_positions, round_trips, skips by reason` | `engine.rs` |
| M10 r2 plug-in | an external-runner seam (`EngineKind`, `spec.rs:16-21`; `cellrun.rs:169-180`) returning a `CellRun`-shaped result so folds, assemble, robustness, integrity, audit and sweep run unchanged; IN-OPT chain reader in the sealed store; extend the `tq-bias` options source; move the catalogue entry from NOT-RUN | r2run `folds`, `assemble`, `robust`, `sweep`, `guard`, `integrity`; r2gov registrar/caps/sealed; r2stats `gate-eval` |
| M11 benchmarks and baselines | B1 Nifty ETF (cost and tax paying), B3 50/50 ETF+BTC via `r2stats::bench::run`; options baselines: random-entry options book, cash | `r2stats/bench.rs`, r2run `baselines.rs` |
| M12 event calendars | committed CSVs with citations: Budget, RBI MPC, results (if point-in-time), loader with a point-in-time check | none |
| M13 synthetic world | seeded chain generator with planted VRP edge and null, weekly and monthly, for the protocol agent's dry run and calibration | `synth.rs`, `r2stats/calib/*` |

### 6.3 How options books plug into the r2 harness
* Returns series: DAILY mark-to-market of the whole account across all open structures (overlapping weeklies included): `r_t = (sum of leg MTM changes - costs_t - tax_t) / equity_{t-1}`; equity = cash + open MTM; idle cash 0; flat days 0 (as v1). The gating series is the after-tax liquidation series (4.5); the pre-tax series is stored beside.
* Exposure: `margin_t / equity_{t-1}` clipped to [0, 1] (`inputs.rs:73`); a naked stress margin above equity is itself a breach flag.
* Trade definition: one structure lifecycle from first leg fill to last leg exit, flat-to-flat; `symbol` = the underlying (so concurrency counts per underlying); `net_pnl` after slippage and costs, before tax. Hedge futures belong to the same trade. Stock books: one trade per (stock, cycle).
* Independent-trade counting and the N_eff haircut: use the harness formula (`neff.rs`) but note its behaviour. For a single underlying rho_bar falls back to 1, so DE = k_bar and N_eff = N_raw / (mean concurrent open structures). Non-overlapping weeklies give k_bar = 1 and N_eff = N_raw (about 50 a year), which OVERSTATES independence: consecutive weekly outcomes share a volatility regime. Proposed additional, stricter count (for the protocol agent to adopt or not): N_eff = N_raw / DE_time with DE_time = 1 + 2 sum_{k=1..4}(1 - k/5) rho_k from the autocorrelation of per-trade net returns; where structures overlap on one underlying, use k_bar as the harness does. For pooled Nifty + BankNifty the harness rho_bar is high (about 0.8-0.9, recollection), so a second index adds little. Report both counts.
* Benchmarks: options books use the absolute-return kind (as v1, prereg-options-v1.md section 7); compare to B1/B3 on the same dates and add the risk-matched baseline "cash + beta-matched ETF".
* Folds: harness purge for IN-OPT is one option cycle = 21 trading days (`folds.rs:14`); weekly structures purge 5 sessions; the fold plan comes from the protocol agent. Positions carry across folds as in r2.

### 6.4 Dependency-ordered task list (parallel agents, separate worktrees, one ledger writer)
Wave 1 (parallel, independent): T1 M2+M7 rate and lot table extension (docs first, one citation per row, unverified flagged); T2 M3 pricing library (parity and known-value tests); T3 M12 event calendars; T5 schema freeze with o2data (the compact schema belongs to the data agent).
Wave 2: T6 M0 loader (needs T5); T7 M8 tax model (independent, early); T4 M13 synthetic world (needs T5); T8 M6 margin (needs T2 and the payoff scanner).
Wave 3: T9 M4 structures + M5 fills (needs T6, T2); T10 M1 calendar/settlement (needs T6).
Wave 4: T11 M9 book engine (needs T1, T7, T8, T9, T10); acceptance: v1 regression exact.
Wave 5: T12 M10 r2 plug-in and bias source (needs T11 and the protocol agent's Study profile); T13 M11 benchmarks/baselines.
Wave 6: synthetic end-to-end dry run (no gate structurally unpassable), then family agents (section 7 cells), one ledger writer.
Each task: tests first, clippy clean, `CARGO_TARGET_DIR=/Users/thakur/Workspace/Trading/shared-target`, no `target/` in a worktree, disk > 8 GB, local commits only.

### 6.5 Top engineering risks
1. Data volume and schema drift: two bhavcopy formats, weekly rows multiply row counts, corporate-action-adjusted stock contracts, lot revisions; a wrong lot or expiry inference silently changes sizing and costs (v1: 100% unverified lots).
2. The r2 seam (M10): the harness is long-only and bar-engine-centred; retrofitting an external runner risks determinism, sealing and ledger cell-order assumptions (byte-identical re-run, `close-sweep` order).
3. IV and Greek quality from EOD settles; any rule conditioned on derived IV inherits the noise.
4. Overlapping weekly books: concurrency, MTM on carried marks, and a defensible N_eff; double-counted independence inflates DSR.
5. Margin and cost rate history unverified (STT before 2024, stamp before 2020, peak margin, ELM); every unverified row must be flagged in every result.
6. Tax and ruin series definition (OQ1) and the silent 0.0 return (`metrics.rs:32`).
7. Look-ahead through expiry inference, event dates and fill-day filters; prefix-invariance tests must cover opt2 and event inputs.
8. Phase 1 is the former options-v1 holdout, consumed as development data (D-006), with 2025-10-27..31 burned; the clean holdout is forward paper only.

---------------------------------------------------------------------------------------------------

## 7. Grid design (smallest meaningful)

Principle: one parameter axis of 3 levels per candidate (K = 3), documented base at the centre, ring-1 neighbours counted as trials. Structure, underlying class and expiry type are separate candidates, not axes. Underlyings of one class are pooled in a candidate with one parameter vector (r2 rule). Offsets are in trailing-realised-volatility units (sigma over the holding horizon, from 20 sessions of closes), which removes the IV dependency from selection and adapts to regimes; widths are fixed at the base.

| Cand. | Family / expiry / class | Axis (3 levels, base centred) | Base-cell rationale | K |
|---|---|---|---|---|
| C1 | F5 bull put credit spread, weekly, Nifty+BankNifty | short-strike offset 0.75 / 1.0 / 1.25 sigma | about 1 sigma short strike is the usual 16-delta practice; v1's 3% monthly offset is about 1 sigma; VRP literature (Bondarenko; Coval-Shumway; Israelov-Nielsen) | 3 |
| C2 | F10 iron condor, weekly | same offset axis | same | 3 |
| C3 | F11 iron butterfly, weekly | wing width 0.75 / 1.0 / 1.25 sigma | ATM short, wings at 1 sigma | 3 |
| C4 | F15 defined-risk VRP, conditional, weekly | entry threshold on (derived IV - trailing realised vol): 0 / median / 75th percentile | sell only when premium is rich | 3 |
| C5 | F5/F10 monthly | offset 0.75 / 1.0 / 1.25 sigma | v1 lineage, expiry-type contrast | 3 |
| C6 | F8 long straddle/strangle, weekly+monthly | take-profit 40 / 50 / 60% | v1 base | 3 |
| C7 | F6 calendar | far-leg tenor next weekly / second weekly / monthly (ATM) | theta and vega carry | 3 |
| C8 | F12 broken-wing butterfly, weekly | far-wing width 1.0 / 1.5 / 2.0 x near width | skew-harvest convention | 3 |
| C9 | F2 covered call (future + short call), monthly, index and stocks | call offset 0.5 / 1.0 / 1.5 sigma | buy-write literature (Whaley BXM) | 3 |
| C10 | F4 cash-secured put, monthly, index | put offset 0.5 / 1.0 / 1.5 sigma | put-write literature (PUT index) | 3 |
| C11 | F19 trend-filter overlay (long call/put or debit spread), monthly | trend lookback 50 / 100 / 200 sessions | standard filters | 3 |
| C12 | F20 delta-hedged straddle, daily futures hedge | tenor weekly / 2-week / monthly | isolates VRP at daily hedging | 3 |
| C13 | F17a expiry-day open-to-settle iron butterfly | wing width 0.5 / 0.75 / 1.0 sigma (1-day) | restricted EOD form | 3 |
| C14 | F18b RBI MPC straddle | entry T-1 / T-2 / T-3 close | few events, K kept at 3 | 3 |
| C15 | F18c results straddle/strangle, stocks | entry 1 / 2 / 3 days before | point-in-time dependent | 3 |
| C16 | FinNifty+Midcap weekly credit spread (window-limited) | offset 0.75 / 1.0 / 1.25 sigma | thin sample | 3 |
| C17 | F13 back-ratio / F14 jade lizard (jade = stress label) | one axis, 3 levels | low priority, run last | 3 |
| S1 | F9/F16 naked short strangle, weekly: STRESS | none (fixed 1.0 sigma) | not a candidate | 1 |

Total proposal: 17 x 3 + 1 = 52 cells, plus a reserve of 8 for dated re-runs = 60 cap. F1 (outright long call/put) is covered by C6 and C11; F3 protective put, F7 diagonal and F18a Budget get no cells (cheap to add if the owner wants; Budget cannot reach a sample, 7.2). Run in waves and stop early where candidates die; the protocol agent's calibrated power decides the final K.

### 7.1 Why K must stay small (my arithmetic; assumes iid daily returns, zero skew, independent trials: all optimistic)
Expected maximum annualised Sharpe of N zero-edge trials: SR0 = SE x ((1-g) Phi^-1(1-1/N) + g Phi^-1(1-1/(N e))), g = 0.5772, SE = sqrt(1/T) with T in years. A DSR >= 0.95 needs observed SR >= SR0 + 1.645 SE.

| N trials | T = 5 y (Phase 1): SR0 / needed | T = 12 y (Phases 1+2 pooled): SR0 / needed |
|---|---|---|
| 50 | 1.02 / 1.76 | 0.66 / 1.13 |
| 537 (research-v2 N_true) | 1.37 / 2.11 | 0.89 / 1.36 |

At about 52 cells an options book needs an observed net Sharpe near 1.8 on five years; short-premium returns are skewed and fat-tailed, which raises SE further. The protocol agent's calibration gives the real power; this only motivates K <= 60 and candidate-level deflation.

### 7.2 Trade-count and independent-trade expectations per family (calendar facts from 2.4 are UNVERIFIED)
| Family | Raw trades per year | Phase 1 (5 y) raw | Phase 2 (2012-2021) raw | Reaches 100 N_eff? |
|---|---|---|---|---|
| Weekly, Nifty | about 50 | about 250 | about 150 (weeklies from 2019-02) | borderline; not under the stricter serial-correlation count |
| Weekly, BankNifty | about 50 | about 150 (weekly to about 2024-11) | about 270 (from 2016-10) | Phase 2 raw yes, Phase 1 no |
| Weekly, FinNifty / Midcap | about 50 | about 150 / about 130 | none | NO (thin) |
| Monthly, any index | 12 | 60 per index | 120 per index (v1: 118) | NO in Phase 1; v1 needed ten years to reach about 100-118 |
| Stock monthly pooled (about 40 names) | up to 480 | up to 2,400 | up to 4,800 | raw yes; N_eff low: 40 concurrent at rho 0.4 gives DE 16.6, about 145 N_eff in Phase 1 |
| Expiry-day structures | 50 | 250 | 150 / 270 | as weekly |
| Event: Budget | 1 | about 6 | about 10 | NO: cannot reach a meaningful sample |
| Event: RBI MPC | 6 | about 30 | about 60 | NO |
| Event: results (stocks, pooled) | about 160 | about 800 | about 1,600 | raw yes; point-in-time and same-week clustering haircuts |
| Daily-hedged straddle | about 50 | about 250 | about 150-270 | as weekly |

Families that cannot reach a meaningful sample under an honest count: all monthly index structures in Phase 1, Budget, RBI MPC, FinNifty/Midcap weekly. They stay RUNNABLE (a shortfall is INCONCLUSIVE-SAMPLE, not a failure) but run after the weekly families, and their cells are the first to drop if the owner wants a smaller K.

---------------------------------------------------------------------------------------------------

## 8. Open questions for the owner (only what genuinely needs a decision)
OQ1 Tax: is there other income against which F&O losses could be set off in the same year? Default is carry-forward only; symmetric tax is computed as a sensitivity (4.5). Not blocking.
OQ2 Account size for the options book: v1 assumed Rs 25 lakh at 1% risk and lost 36-53% of straddle cycles to "risk budget below one lot" (result-options-v1-dev.md). Proposed default: size by a capped fraction of margin on a Rs 1 crore account, with Rs 25 lakh as a stress case. Not blocking.
The research-v2 questions (D-005 and the rest) are unchanged.

## 9. Items not verified in this task (re-check in dated addenda before any scored run)
Weekly-expiry start/end dates per index; Budget dates; RBI MPC date source; results-date point-in-time availability; NSE SPAN file URL pattern, archive depth and point-in-time retention (a direct probe was blocked by the permission system; only search-result page titles were seen); SPAN parameters (PSR, VSR, ELM, short-option minimum charge); peak-margin phase-in dates; options STT history before 2024-10 and the 2016-06 step; futures STT 0.0125%; exercise STT 0.125% history; stamp duty before 2020-07-01; exchange charge before 2026-03; physical-settlement start dates for stock options; European/American style of early stock options; that option `SETTLE_PR` on expiry day equals intrinsic; FinNifty and Midcap Nifty index close names in `ind_close_all`; stock cash close source for stock options; lot-size history for all underlyings before 2024-11; identification of corporate-action-adjusted contracts; STT deductibility against business income; vendor products and prices (none contacted).

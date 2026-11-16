# Pre-registration options-v1: do mechanical, defined-risk Nifty and BankNifty option structures earn a net-of-cost, net-of-tax edge on end-of-day data?

Registered: 2026-11-06, **before any option price, return, P&L, Sharpe or trade from any strategy below has been computed.** Paper and backtest only; no broker, orders or credentials.
Frozen under `docs/validation-protocol.md` v3: after a result is seen, changes may only tighten, need a dated written reason and make the changed test a new trial.
Data and cost basis: `docs/data-feasibility-options.md`, `docs/costs-tax-options-india.md`. Ledger family **`options-v1`**, **K = 30 cells** (section 6), registered in the same commit
through the honesty ledger API: `research/ledger-options-v1.jsonl` (+ `.anchor`, mirror line in `hardening/ledger-anchor.txt`, row in `research/trials.csv`).

Disclosure of contact with data before registration: for feasibility only I downloaded 9 sessions (2019-03-01 to -07; 2025-10-27 to -31) and probed 8 other single dates (2001-06-04, 2005-01-03, 2012-01-02, 2016-10-03, 2023-01-02, 2024-07-05, 2024-08-01, 2026-01-05; one sample NIFTY row was printed for each of 2012, 2019, 2023, 2024-08 and 2026-01).
Only row counts, field names, zero-volume shares and one printed row were looked at. No strategy was run. The 2025-10-27..31 sessions lie inside the holdout window and are declared **burned for any holdout trade** (a cycle whose entry or exit day falls in them still counts; the data were never used to compute a return).

## 1. Hypotheses

**H1.** At least one structure in section 2, fixed rules, earns out of sample, net of the full cost model, assumed slippage and tax, a positive expectancy that passes every gate in section 8.
**Null.** Option prices are fair after costs: short-premium structures collect a risk premium that is smaller than costs plus tail losses, long premium loses to decay, and any positive dev result is selection among 30 cells.
Known weakness: the base rate is poor (retail F&O studies report most traders lose; unverified here), costs include Rs 20 per order on a small-notional spread, and slippage is a guess because there are no quotes. KILL or INCONCLUSIVE is the expected outcome and a useful one.

## 2. Structures (frozen; defined risk only; no naked short option at any time)

Underlyings: `NIFTY` (spot = Nifty 50 close) and `BANKNIFTY` (spot = Nifty Bank close), from the NSE index close file. Expiry = the **monthly** expiry: the latest expiry date in the calendar month that appears in the option file on the signal date.
Calendar of one cycle: signal on the close of trading day `t` = the last trading day on or before the previous monthly expiry's day (that is, the previous monthly expiry day itself, or its prior trading day if it was not a session). **Entry executes at the open of `t+1`.**
**Scheduled exit** executes at the open of the trading day before the expiry day (`D-1`); so no position is held through expiry and exercise STT is never paid. Next cycle's signal is the previous expiry day. No overlap; at most one position per variant at a time. All other decisions (profit take, stop) are evaluated on the **daily settle at close of a day and executed at the next open**.

| Variant | Underlying | Structure | Strikes (nearest listed strike with OI >= 500 on `t`, rounded away from spot) |
| --- | --- | --- | --- |
| V1, V2 | NIFTY, BANKNIFTY | **A. Bull put credit spread** | sell put 3.0% below spot; buy put `w` further down, `w` = 2.0% of spot |
| V3, V4 | NIFTY, BANKNIFTY | **B. Iron condor** | A plus bear call spread: sell call 3.0% above spot, buy call `w` = 2.0% further up |
| V5, V6 | NIFTY, BANKNIFTY | **C. Long ATM straddle** (long-premium, defined risk = premium paid) | buy call and put at the strike nearest spot |

Exits for A and B: take profit when the mark (sum of leg settles, long legs valued at settle, short legs at settle) shows >= 50% of the opening credit captured; loss cut when the mark shows a loss >= 1.0x the opening credit; otherwise `D-1`. Exits for C: take profit at +50% of premium paid, cut at -50%, otherwise `D-1`.
A loss cut or stop exit uses double slippage (costs doc). Both A and B are always entered (no trend or vol filter), so the trade count is not shrunk by conditioning.
Liquidity filter: each leg must have `CONTRACTS > 0` and `OPEN > 0` on the fill day `t+1` and open interest >= 500 on `t`; otherwise **skip the cycle** and count the skip. If skips exceed 15% of cycles for a variant in the dev window, that variant is INCONCLUSIVE-SAMPLE (it is not rescued by loosening the filter).
Fills: each leg at that day's `OPEN` with adverse slippage `s = max(1% of premium, Rs 1.0)` (2s on loss cuts), full cost schedule from `docs/costs-tax-options-india.md` by date.
Valuation between events at `SETTLE_PR` only for legs with volume > 0 that day; if a leg has no settle-quality quote on a day, carry the last valid mark (stated, counted, reported).

## 3. Sizing and margin (frozen)

Account per variant: Rs 25,00,000, one variant per account (no pooling). Risk per trade **1.0% of equity** = Rs 25,000 against the structure's **maximum loss per lot** (A, B: `width x lot - credit x lot`, for B the wider side, plus modelled round-trip costs; C: premium paid x lot).
`lots = floor(25,000 / max_loss_per_lot)`; if 0 the cycle is skipped and counted (this is expected for BankNifty in recent years: that is a real feasibility result, not a bug). Maximum 10 lots.
**Margin proxy (SPAN is not reproducible from bhavcopy):** hedged spread and condor margin = maximum loss of the structure (plus the premium received is not used as margin credit); long straddle = premium paid. Deployed margin must not exceed 60% of equity (it cannot at 1% risk, checked anyway). Idle cash earns 6% a year (sensitivity 4% to 8%). Margin shortfall, MTM funding and broker margin changes are out of scope.
Lot size per date: from a table built from NSE circulars, committed with a SHA-256 before the first scored run (addendum). If lot-size history cannot be sourced for a date range, that range is excluded and the dev window shortened; if the shortened window yields < 100 trades the result is INCONCLUSIVE-SAMPLE.
Kill switch **off** in the assessment run (protocol section 7 rule 1), ON as a reported operating run.

## 4. Costs, slippage and tax (frozen)

As `docs/costs-tax-options-india.md`: Rs 20 per order, STT by date table (0.017% / 0.05% / 0.0625% / 0.10% / 0.15% on sell premium), exchange charge 0.05% before 2024 and 0.03553% from 2024-01-01 (dev numbers conservative), SEBI Rs 10 per crore, stamp 0.003% buy, GST 18%; slippage per section 2; stress at 1x, 2x, 3x slippage and 2x brokerage.
Tax: business income at 31.2% flat on positive annual net P&L after charges and STT, losses netted within the year, carried forward 8 years, tax paid 31 March; sensitivities 20%, 39% and no carry-forward. Headline results are net of tax.

## 5. Data, windows, folds, holdout (frozen)

| | |
| --- | --- |
| Development (scored) | cycles whose entry day is in **2012-01-01 to 2021-12-31** (about 120 monthly cycles per variant). Index closes for the NSEI series before 2012 come from `data/index/NSEI.csv`; BankNifty from `ind_close_all` files. |
| Folds | protocol v3 section 2 (test 6 months, step 6, first test 2015-01-01, 14 folds); parameters are fixed so the folds measure stability, not fit |
| **Single-touch holdout** | cycles with entry day **2022-01-01 to 2026-10-31**; its option and index files are **not downloaded and not opened** until dev passes every gate in section 8 for a variant, then it is run once and burned (protocol rule 5) |
| Dev data manifest | SHA-256 of every file used, committed in an addendum **before** the first scored run |

No tool may read bars dated inside the holdout during development. The harness must fail closed on any holdout-dated file (as `tq council-eval` does).

## 6. Trial count K and DSR deflation

Axes: `variant` {V1..V6} (6) x `param` {base, offset-20%, offset+20%, width-20%, width+20%} (5) = **K = 30**. For C the four non-base params are take-profit 40%, take-profit 60%, loss cut 40%, loss cut 60% (offset and width do not exist). Base params are 3.0% / 2.0%; neighbours are 2.4%, 3.6%, 1.6%, 2.4%.
The base cell of each variant is the only one eligible for GO; the neighbour cells are the protocol section 5 parameter-neighbourhood check and **are counted** in the ledger and in N. **DSR is deflated with N = 30** (Bailey and Lopez de Prado), also reported at N = 6 and at N = all ledger rows (informational). Sharpe variance via the honesty layer.
Sensitivities (slippage 2x/3x, brokerage 2x, tax variants, cash rate, T+2 shift) are variants of the same cells, not new trials.

## 7. Metrics

Unit: daily net returns of the account (equity marked daily, flat days included), after costs, slippage, STT and tax. Sharpe annualised on 252; equity rf = 6% for the like-for-like line, rf = 0 for gate 4e. HAC t per protocol v3 (Newey-West automatic bandwidth).
Declared `absolute-return` in the ledger (invested share of equity is low; market beta is not meaningful for a defined-risk monthly option book), so the **4c-abs** t gate applies. For C (long vol) the same.

## 8. Gates (frozen; every one must hold, none lowered)

| # | Gate | Value |
| --- | --- | --- |
| O1 Sample | closed trades taken in dev | **>= 100** (protocol section 3 asks >= 200 in dev and >= 100 in stitched OOS; the 200 and the OOS clause are not attainable with monthly cycles, see section 9, and are reported, not waived); skip rate <= 15%; >= 24 distinct months |
| O2 Deflation | DSR of daily net returns **>= 0.95** at N = 30; also on monthly returns, the more conservative counts | |
| O3 Overfitting | PBO by CSCV over the 6 base cells (16 time blocks) **<= 0.20** | |
| O4 Net edge | dev HAC t of daily net returns (after costs, slippage, tax) **>= 3.0** (4c-abs); annualised Sharpe rf=0 **>= 0.8** (4e); net CAGR > 0 | |
| O5 Benchmark | portfolio Sharpe at rf = 6% AND Calmar exceed Nifty buy-and-hold (same dates, TRI proxy stated), else buy the index (4f) | |
| O6 Robustness | >= 70% of the 4 neighbour cells net-profitable and median neighbour Sharpe >= 70% of the base; 2x slippage and 2x brokerage still net-positive with positive t; >= 60% of the 14 folds net-positive; dropping the 5 best trades leaves net P&L > 0 | |
| O7 Tail | worst single trade loss <= 1.5x the budgeted 1% risk; 95th-percentile month-block Monte Carlo max drawdown <= 0.8 x the kill-switch limit (5% placeholder from protocol 7; `UNSET-BY-USER`, so a fail at 5% is reported and the owner decides) | |
| O8 Integrity | harness passes `tq bias lookahead` (prefix invariance: signals use only data through `t`), byte-identical re-run, no holdout-dated file read in dev | |

**Outcomes per variant:** `GO-DEV` (O1 to O8), `INCONCLUSIVE-SAMPLE` (O1 fails), `KILL` otherwise. **options-v1 overall:** GO-DEV if >= 1 variant GO-DEV; KILL if none and >= 4 variants KILL; else INCONCLUSIVE.
**Holdout (GO-DEV variants only, base cell, once):** >= 30 trades; net return > 0; Sharpe >= 50% of dev; HAC t >= 1.65; beats Nifty buy-and-hold on Sharpe and Calmar (4f).
**After holdout:** forward paper trading **>= 60 sessions AND >= 20 closed trades** (protocol section 8) with results inside the Monte Carlo 5-95% band and net expectancy positive. "Profitable" is claimed only if the unamended gates pass (STATUS rule 4); otherwise report "nothing passed".

## 9. Known limits stated up front

1. **Trade-count arithmetic.** Monthly cycles give about 120 trades per variant in 10 years and about 57 in the holdout. The 100-trade gate is attainable on dev only if skips are rare; the protocol's >= 100 trades in stitched OOS (2015 to 2021 is 84 cycles) **cannot be met by a monthly structure**. This is not lowered: it means a monthly book can at best reach GO-DEV by O1 on the full dev window and cannot satisfy section 3's OOS clause. I flag it to the owner rather than moving it; a weekly design (weeklies exist for Nifty from 2019-02, BankNifty from 2016-10) would need its own pre-registration and its own K.
2. **Paper gate.** 60 sessions is about 3 monthly cycles; >= 20 trades needs about 20 months of forward paper trading.
3. No bid/ask: slippage `s` is a guess; the result depends on it (stress runs mandatory).
4. EOD only: gap and intraday wing breaches are visible only at the next settle; loss cuts execute a day late, understating tail loss. The maximum loss of a defined-risk spread bounds it, but not the gap against the stop.
5. BankNifty option liquidity in 2012 to 2015 may fail the OI >= 500 and volume filters, producing skips.
6. Costs schedule history before 2024 is partly secondary-sourced and brokerage is understated pre-2015.
7. Selecting among the 6 variants on dev is a researcher degree of freedom, covered by N = 30 deflation.
8. Tax model is a flat 31.2%; the actual position depends on the owner's income and tax regime.

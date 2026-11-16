# Result options-v1, development window (2012-2021): KILL, nothing passed

Run 2026-11-07 by `tq options-eval --all` at code `c1f29b2`, once, after `docs/prereg-options-v1-addendum.md` (committed before the run). Preregistration: `docs/prereg-options-v1.md` (tag `prereg-options-v1`), thresholds unamended, K = 30, sweep closed in `research/ledger-options-v1.jsonl` (N = 30, anchor mirrored in `hardening/ledger-anchor.txt`). Holdout (2022-01-01 onwards) not downloaded, not opened, not run.

## Honest summary

* **Overall verdict: KILL** (rule: no GO-DEV, and at least 4 variants KILL). V1 to V4 are KILL; V5 and V6 are INCONCLUSIVE-SAMPLE.
* **No variant passed more than 3 of 8 gates (V1: O1, O7, O8) and none passed any of the edge or deflation gates (O2 to O6).** Base-cell net P&L before tax over ten years on a Rs 25,00,000 account: V1 -Rs 3,984, V2 -Rs 6,383, V3 -Rs 276,127, V4 -Rs 50,740, V5 -Rs 39,847, V6 +Rs 58,193. The best HAC t is V6's 0.23 (gate 3.0); the other five are negative (V3 -2.16). The best Sharpe at rf = 0 is 0.09 (gate 0.8). Daily DSR at N = 30 is between 0.000 and 0.035 for all six (gate 0.95); PBO over the six base cells is 0.849 (gate 0.20).
* **Trade-count shortfall, reported and not waived.** The 100-trade gate needs almost every one of the ~118 monthly cycles. V1 (118), V2 (110), V3 (118) and V4 (103) reached it; V5 took 75 trades (43 of 118 cycles skipped because one lot of an ATM straddle already exceeds the Rs 25,000 risk budget) and V6 took 55 (63 skipped, 53%), so both fail the sample gate and the skip-rate gate (<= 15%) and are INCONCLUSIVE-SAMPLE. The protocol's >= 200 development trades and >= 100 stitched out-of-sample trades are not attainable with monthly cycles (prereg section 9) and were not used to relax anything. Nothing was lowered.
* **Nothing here is evidence of profit.** The expected outcome stated in the prereg (KILL or INCONCLUSIVE) occurred. The two long-straddle variants that look slightly positive (V6 +0.10% a year after tax, t 0.23) are statistically indistinguishable from zero, lose money at 2x slippage (V6 -Rs 21,676) and fail O1.

## Reading the numbers (limits)

* **Slippage is an assumption.** No bid/ask exists in the data. Short-premium variants V1 and V3 are roughly break-even or worse at 1x slippage and lose clearly at 2x and 3x (V1: -Rs 72,726 and -Rs 179,594), so the already weak result depends on the assumed Rs 1 or 1% per leg.
* **Shift test.** Protocol rule 0.2: delaying entry by one extra session must not improve results beyond noise. V1 improves from -Rs 3,984 to +Rs 41,842 with the T+2 shift (t 0.56). That is within noise for a t-statistic near zero and does not make V1 positive, but it is flagged: V1's sign is not stable to the fill day.
* **Lot sizes are unverified** (data-implied, addendum item 15): 100% of trades use an unverified lot. A wrong lot changes the lot count and costs, not the sign of the per-unit result.
* **Not computed or not applicable:** the 4% and 8% cash-rate sensitivities; the protocol's >= 200 / OOS >= 100 trade counts (unattainable); holdout and forward paper (only after GO-DEV). O7 (worst trade, Monte Carlo drawdown) is reported for completeness and passes for V1; passing it says nothing about edge.
* **Operating run (kill switch at 5% drawdown)** halts V3 after 79 trades and V4 after 34; it is a sensitivity, not a gate.
* Index closes before 2012-03-01 exist only for Nifty (fallback `NSEI.csv`); BankNifty cycles then skip with "no index close". BankNifty far-wing strikes with open interest below 500 explain the V4 skips (7 long-call strikes).

## Pre-run corrections disclosed (all before the scored run, none after any result)

* The first draft that defined each month's monthly expiry from all files was flagged by `tq bias lookahead` (February 2014 expiry shift) and was replaced by point-in-time signal days (addendum item 7). Two further parser fixes (dd/mm/yyyy index dates, two-digit-year file) came from loading the real files. These were found by structure and parsing checks that compute no P&L.
* I once passed `--holdout-ack` to the download script together with a non-existent sessions path (addendum item 2); it failed before any access. The flag was not used otherwise.

## Generated result

**Nothing in this document is evidence of profit.** It is a development-window backtest on end-of-day data with assumed slippage and a flat tax model. A strategy is called profitable only if it passes every unamended gate of the preregistration, the single-touch holdout and forward paper trading; otherwise the honest statement is that nothing passed.

Code sha `c1f29b2`; data manifest sha256 `75893215eb6454cbcea73843cac2fce32f89a74da8b72d9a0790b6b693ed61c9`; lot table sha256 `ee748e818b18b0f2d639b3c5abb238474187c438f15039248921280fcd1e8784`.
Sweep `1792a507b6cff0850017d5682d6b2789e9cbf62881f08ae4fc6e81909a483d8c` closed with Merkle root `8877d0cfb13658dcd85358f90a76a7b5bca716c1232724942fb6d7151029ae09`; ledger family trials N = 30 (cross-trial Sharpe variance 0.000310).

## Verdicts

**options-v1 overall: KILL**

| Variant | Underlying | Structure | Verdict | Trades | Cycles | Skips | Net P&L pre-tax (Rs) | CAGR after tax | Sharpe rf=0 | HAC t |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| V1 | NIFTY | BullPutSpread | **KILL** | 118 | 118 | 0 | -3984 | -0.02% | -0.02 | -0.10 |
| V2 | BANKNIFTY | BullPutSpread | **KILL** | 110 | 118 | 8 | -6383 | -0.04% | -0.06 | -0.17 |
| V3 | NIFTY | IronCondor | **KILL** | 118 | 118 | 0 | -276127 | -1.20% | -0.67 | -2.16 |
| V4 | BANKNIFTY | IronCondor | **KILL** | 103 | 118 | 15 | -50740 | -0.26% | -0.10 | -0.37 |
| V5 | NIFTY | LongStraddle | **INCONCLUSIVE-SAMPLE** | 75 | 118 | 43 | -39847 | -0.19% | -0.15 | -0.47 |
| V6 | BANKNIFTY | LongStraddle | **INCONCLUSIVE-SAMPLE** | 55 | 118 | 63 | 58193 | 0.10% | 0.09 | 0.23 |

PBO over the 6 base cells (16 blocks): 0.849.

## V1 (NIFTY BullPutSpread): KILL

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | pass | trades 118 (>= 100), skip rate 0.0% = 0/118 (<= 15%), entry months 101 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.017 and monthly N=30 0.014 (both >= 0.95); informational: daily N=6 0.088, daily N=all ledger rows 0.017 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t -0.098 (HAC -0.098, monthly -0.121, lag 8; >= 3.0), Sharpe rf=0 -0.020 (>= 0.8), CAGR -0.02% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) 0.213 vs NIFTYBEES 0.543; Calmar 6.000 vs 0.391 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 0/4 (>= 70%), median neighbour Sharpe -0.210 vs 0.7 x base -0.014; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 8/14 (>= 60%); net P&L without the 5 best trades Rs -31083 (> 0) |
| O7 Tail | pass | worst trade Rs -10993 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 3.18% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: none. Lots per trade: max 6, mean 1.86. Trades on an unverified lot size: 118/118. Carried marks: 0; unfillable exit legs: 0. Worst trade Rs -10993; max drawdown after tax 3.13%.

## V2 (BANKNIFTY BullPutSpread): KILL

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | pass | trades 110 (>= 100), skip rate 6.8% = 8/118 (<= 15%), entry months 96 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.012 and monthly N=30 0.012 (both >= 0.95); informational: daily N=6 0.069, daily N=all ledger rows 0.012 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t -0.168 (HAC -0.168, monthly -0.173, lag 8; >= 3.0), Sharpe rf=0 -0.059 (>= 0.8), CAGR -0.04% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) 0.172 vs NIFTYBEES 0.507; Calmar 9.509 vs 0.373 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 2/4 (>= 70%), median neighbour Sharpe 0.003 vs 0.7 x base -0.042; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 7/14 (>= 60%); net P&L without the 5 best trades Rs -57409 (> 0) |
| O7 Tail | FAIL | worst trade Rs -18518 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 5.32% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: 1 x no index close on the signal day, 1 x no listed strike with open interest >= 500 for a long put leg, 4 x risk budget below one lot, 2 x selected strikes do not form a valid structure. Lots per trade: max 10, mean 2.06. Trades on an unverified lot size: 110/110. Carried marks: 2; unfillable exit legs: 1. Worst trade Rs -18518; max drawdown after tax 2.29%.

## V3 (NIFTY IronCondor): KILL

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | pass | trades 118 (>= 100), skip rate 0.0% = 0/118 (<= 15%), entry months 101 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.000 and monthly N=30 0.000 (both >= 0.95); informational: daily N=6 0.000, daily N=all ledger rows 0.000 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t -2.164 (HAC -2.164, monthly -2.038, lag 8; >= 3.0), Sharpe rf=0 -0.669 (>= 0.8), CAGR -1.20% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) -0.577 vs NIFTYBEES 0.543; Calmar 0.973 vs 0.391 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 0/4 (>= 70%), median neighbour Sharpe -0.764 vs 0.7 x base -0.469; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 6/14 (>= 60%); net P&L without the 5 best trades Rs -345420 (> 0) |
| O7 Tail | FAIL | worst trade Rs -103692 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 20.47% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: none. Lots per trade: max 9, mean 2.29. Trades on an unverified lot size: 118/118. Carried marks: 2; unfillable exit legs: 0. Worst trade Rs -103692; max drawdown after tax 11.29%.

## V4 (BANKNIFTY IronCondor): KILL

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | pass | trades 103 (>= 100), skip rate 12.7% = 15/118 (<= 15%), entry months 91 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.008 and monthly N=30 0.007 (both >= 0.95); informational: daily N=6 0.054, daily N=all ledger rows 0.008 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t -0.374 (HAC -0.374, monthly -0.387, lag 8; >= 3.0), Sharpe rf=0 -0.096 (>= 0.8), CAGR -0.26% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) -0.033 vs NIFTYBEES 0.507; Calmar 1.599 vs 0.373 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 0/4 (>= 70%), median neighbour Sharpe -0.281 vs 0.7 x base -0.067; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 5/14 (>= 60%); net P&L without the 5 best trades Rs -265422 (> 0) |
| O7 Tail | FAIL | worst trade Rs -87173 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 14.58% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: 1 x no index close on the signal day, 7 x no listed strike with open interest >= 500 for a long call leg, 1 x no listed strike with open interest >= 500 for a long put leg, 4 x risk budget below one lot, 2 x selected strikes do not form a valid structure. Lots per trade: max 10, mean 3.15. Trades on an unverified lot size: 103/103. Carried marks: 41; unfillable exit legs: 5. Worst trade Rs -87173; max drawdown after tax 11.17%.

## V5 (NIFTY LongStraddle): INCONCLUSIVE-SAMPLE

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | FAIL | trades 75 (>= 100), skip rate 36.4% = 43/118 (<= 15%), entry months 64 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.006 and monthly N=30 0.007 (both >= 0.95); informational: daily N=6 0.041, daily N=all ledger rows 0.006 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t -0.466 (HAC -0.466, monthly -0.434, lag 8; >= 3.0), Sharpe rf=0 -0.149 (>= 0.8), CAGR -0.19% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) -0.020 vs NIFTYBEES 0.543; Calmar 9.411 vs 0.391 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 0/4 (>= 70%), median neighbour Sharpe -0.155 vs 0.7 x base -0.104; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 2/14 (>= 60%); net P&L without the 5 best trades Rs -126854 (> 0) |
| O7 Tail | FAIL | worst trade Rs -17015 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 9.96% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: 43 x risk budget below one lot. Lots per trade: max 4, mean 1.63. Trades on an unverified lot size: 75/75. Carried marks: 0; unfillable exit legs: 0. Worst trade Rs -17015; max drawdown after tax 6.09%.

## V6 (BANKNIFTY LongStraddle): INCONCLUSIVE-SAMPLE

| Gate | Result | Detail |
| --- | --- | --- |
| O1 Sample | FAIL | trades 55 (>= 100), skip rate 53.4% = 63/118 (<= 15%), entry months 48 (>= 24) |
| O2 Deflation | FAIL | DSR daily N=30 0.035 and monthly N=30 0.034 (both >= 0.95); informational: daily N=6 0.150, daily N=all ledger rows 0.035 (ledger rows 30) |
| O3 Overfitting | FAIL | PBO 0.849 over the 6 base cells, 16 blocks (<= 0.20) |
| O4 Net edge | FAIL | gate t 0.233 (HAC 0.233, monthly 0.260, lag 8; >= 3.0), Sharpe rf=0 0.087 (>= 0.8), CAGR 0.10% (> 0) |
| O5 Benchmark | FAIL | Sharpe(rf 6%) 0.214 vs NIFTYBEES 0.507; Calmar 4.718 vs 0.373 (strategy must exceed both) |
| O6 Robustness | FAIL | neighbours profitable 2/4 (>= 70%), median neighbour Sharpe 0.068 vs 0.7 x base 0.061; 2x slippage net-positive with t>0: false; 2x brokerage: false; folds positive 3/14 (>= 60%); net P&L without the 5 best trades Rs -92962 (> 0) |
| O7 Tail | FAIL | worst trade Rs -19603 (floor -37500 = 1.5 x budget), month-block MC p95 max drawdown 7.71% (<= 4.0% = 0.8 x 5% placeholder) |
| O8 Integrity | pass | tq bias lookahead clean Some(true), byte-identical re-run Some(true), no holdout-dated file read Some(true) |

Skips: 1 x no index close on the signal day, 62 x risk budget below one lot. Lots per trade: max 2, mean 1.11. Trades on an unverified lot size: 55/55. Carried marks: 0; unfillable exit legs: 0. Worst trade Rs -19603; max drawdown after tax 6.17%.

## All 30 cells

| Variant | Param | Trades | Skips | Net P&L pre-tax (Rs) | CAGR after tax | Sharpe rf=0 |
| --- | --- | --- | --- | --- | --- | --- |
| V1 | base | 118 | 0 | -3984 | -0.02% | -0.02 |
| V1 | offset-20 | 117 | 1 | -19132 | -0.08% | -0.19 |
| V1 | offset+20 | 117 | 1 | -36767 | -0.15% | -0.44 |
| V1 | width-20 | 118 | 0 | -41747 | -0.17% | -0.23 |
| V1 | width+20 | 116 | 2 | -11922 | -0.05% | -0.07 |
| V2 | base | 110 | 8 | -6383 | -0.04% | -0.06 |
| V2 | offset-20 | 112 | 6 | 43016 | 0.11% | 0.13 |
| V2 | offset+20 | 106 | 12 | -1172 | -0.03% | -0.04 |
| V2 | width-20 | 112 | 6 | 19227 | 0.03% | 0.05 |
| V2 | width+20 | 107 | 11 | -56210 | -0.24% | -0.32 |
| V3 | base | 118 | 0 | -276127 | -1.20% | -0.67 |
| V3 | offset-20 | 118 | 0 | -240776 | -1.04% | -0.78 |
| V3 | offset+20 | 117 | 1 | -168243 | -0.71% | -0.84 |
| V3 | width-20 | 117 | 1 | -322650 | -1.41% | -0.75 |
| V3 | width+20 | 118 | 0 | -176616 | -0.75% | -0.66 |
| V4 | base | 103 | 15 | -50740 | -0.26% | -0.10 |
| V4 | offset-20 | 110 | 8 | -162015 | -0.69% | -0.17 |
| V4 | offset+20 | 97 | 21 | -119108 | -0.54% | -0.34 |
| V4 | width-20 | 106 | 12 | -134016 | -0.63% | -0.23 |
| V4 | width+20 | 97 | 21 | -160091 | -0.74% | -0.36 |
| V5 | base | 75 | 43 | -39847 | -0.19% | -0.15 |
| V5 | offset-20 | 74 | 44 | -71837 | -0.32% | -0.26 |
| V5 | offset+20 | 74 | 44 | -59767 | -0.25% | -0.19 |
| V5 | width-20 | 75 | 43 | -27588 | -0.14% | -0.12 |
| V5 | width+20 | 75 | 43 | -28348 | -0.17% | -0.12 |
| V6 | base | 55 | 63 | 58193 | 0.10% | 0.09 |
| V6 | offset-20 | 54 | 64 | 39 | -0.15% | -0.12 |
| V6 | offset+20 | 56 | 62 | 82604 | 0.20% | 0.16 |
| V6 | width-20 | 56 | 62 | 28474 | -0.03% | -0.02 |
| V6 | width+20 | 56 | 62 | 90136 | 0.22% | 0.17 |

## Sensitivities (base cells; variants of the same trials, not new ones)

| Variant | Run | Trades | Net P&L pre-tax (Rs) | CAGR after tax | HAC t |
| --- | --- | --- | --- | --- | --- |
| V1 | base (1x slippage, 1x brokerage, 31.2% tax) | 118 | -3984 | -0.02% | -0.10 |
| V1 | 2x slippage | 118 | -72726 | -0.30% | -2.13 |
| V1 | 2x brokerage | 118 | -15951 | -0.07% | -0.44 |
| V1 | 3x slippage | 118 | -179594 | -0.76% | -4.70 |
| V1 | tax 20% | 118 | -3984 | -0.02% | -0.10 |
| V1 | tax 39% | 118 | -3984 | -0.02% | -0.10 |
| V1 | tax 31.2%, no loss carry-forward | 118 | -3984 | -0.09% | -0.57 |
| V1 | T+2 shift (entry one session later) | 117 | 41842 | 0.12% | 0.56 |
| V1 | operating run: kill switch at 5% drawdown | 118 | -3984 | -0.02% | -0.10 |
| V2 | base (1x slippage, 1x brokerage, 31.2% tax) | 110 | -6383 | -0.04% | -0.17 |
| V2 | 2x slippage | 110 | -54837 | -0.23% | -0.92 |
| V2 | 2x brokerage | 110 | -15405 | -0.08% | -0.30 |
| V2 | 3x slippage | 107 | -123201 | -0.52% | -2.01 |
| V2 | tax 20% | 110 | -6383 | -0.04% | -0.14 |
| V2 | tax 39% | 110 | -6383 | -0.05% | -0.19 |
| V2 | tax 31.2%, no loss carry-forward | 110 | -6383 | -0.12% | -0.47 |
| V2 | T+2 shift (entry one session later) | 109 | -38701 | -0.17% | -0.53 |
| V2 | operating run: kill switch at 5% drawdown | 110 | -6383 | -0.04% | -0.17 |
| V3 | base (1x slippage, 1x brokerage, 31.2% tax) | 118 | -276127 | -1.20% | -2.16 |
| V3 | 2x slippage | 118 | -325162 | -1.42% | -3.82 |
| V3 | 2x brokerage | 118 | -309989 | -1.35% | -2.45 |
| V3 | 3x slippage | 118 | -410734 | -1.83% | -5.04 |
| V3 | tax 20% | 118 | -276127 | -1.20% | -2.16 |
| V3 | tax 39% | 118 | -276127 | -1.20% | -2.16 |
| V3 | tax 31.2%, no loss carry-forward | 118 | -276127 | -1.21% | -2.18 |
| V3 | T+2 shift (entry one session later) | 117 | -89491 | -0.38% | -0.95 |
| V3 | operating run: kill switch at 5% drawdown | 79 | -122733 | -0.52% | -2.04 |
| V4 | base (1x slippage, 1x brokerage, 31.2% tax) | 103 | -50740 | -0.26% | -0.37 |
| V4 | 2x slippage | 102 | -183053 | -0.82% | -1.28 |
| V4 | 2x brokerage | 103 | -48591 | -0.25% | -0.37 |
| V4 | 3x slippage | 100 | -308156 | -1.35% | -2.01 |
| V4 | tax 20% | 103 | -50740 | -0.25% | -0.34 |
| V4 | tax 39% | 103 | -50740 | -0.28% | -0.39 |
| V4 | tax 31.2%, no loss carry-forward | 103 | -50740 | -0.38% | -0.55 |
| V4 | T+2 shift (entry one session later) | 105 | -300484 | -1.34% | -1.22 |
| V4 | operating run: kill switch at 5% drawdown | 34 | -75048 | -0.37% | -0.84 |
| V5 | base (1x slippage, 1x brokerage, 31.2% tax) | 75 | -39847 | -0.19% | -0.47 |
| V5 | 2x slippage | 74 | -64330 | -0.27% | -0.68 |
| V5 | 2x brokerage | 75 | -47014 | -0.21% | -0.52 |
| V5 | 3x slippage | 73 | -69943 | -0.29% | -0.75 |
| V5 | tax 20% | 75 | -39847 | -0.18% | -0.44 |
| V5 | tax 39% | 75 | -39847 | -0.20% | -0.48 |
| V5 | tax 31.2%, no loss carry-forward | 75 | -39847 | -0.25% | -0.61 |
| V5 | T+2 shift (entry one session later) | 74 | -48988 | -0.26% | -0.68 |
| V5 | operating run: kill switch at 5% drawdown | 62 | -86128 | -0.39% | -1.09 |
| V6 | base (1x slippage, 1x brokerage, 31.2% tax) | 55 | 58193 | 0.10% | 0.23 |
| V6 | 2x slippage | 51 | -21676 | -0.19% | -0.49 |
| V6 | 2x brokerage | 54 | 17302 | -0.06% | -0.13 |
| V6 | 3x slippage | 49 | -22295 | -0.17% | -0.44 |
| V6 | tax 20% | 55 | 58193 | 0.15% | 0.34 |
| V6 | tax 39% | 55 | 58193 | 0.07% | 0.16 |
| V6 | tax 31.2%, no loss carry-forward | 55 | 58193 | 0.00% | 0.02 |
| V6 | T+2 shift (entry one session later) | 52 | 17381 | -0.01% | -0.01 |
| V6 | operating run: kill switch at 5% drawdown | 55 | 58193 | 0.10% | 0.23 |

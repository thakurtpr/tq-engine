# Result: council-v1, development evaluation

Pre-registration `docs/prereg-council-v1.md` (tag `prereg-council-v1`) plus `docs/prereg-council-v1-addendum.md`. **Development windows only** (equity 2015-2019, crypto 2023); the 2020-2021 equity holdout and the 2024 crypto fence were not read. Paper and backtest only. Nothing here is evidence of profit.

Code git sha `febc2eee593978c2b2582cc9d8ec6b2905cf487e`; data manifest sha256 `3c26f77461acdf033db4de4bb951a3a48933fdf7f56c096dcfefe064dd63bd6c`.

**council-v1 overall: INCONCLUSIVE.**

| Base | Verdict | Judged council | Entries | Sharpe (council / unfiltered / index) | Failed gates |
| --- | --- | --- | ---: | --- | --- |
| donchian | INCONCLUSIVE-SAMPLE | minimal | 0 | 0.000 / -1.363 / 0.234 | G1, G2, G4, G7, G8 |
| trend-pullback | INCONCLUSIVE-SAMPLE | minimal | 0 | 0.000 / -2.218 / 0.234 | G1, G2, G4, G7 |
| momentum-rotation | INCONCLUSIVE-SAMPLE | full (no voter retained) | 5 | -2.237 / -1.128 / 0.234 | G1, G2, G3, G4, G7, G8 |
| crypto-burst | INCONCLUSIVE-SAMPLE | full (no voter retained) | 0 | 0.000 / -30.921 / 1.756 | G1, G2, G4, G7 |

### Did council filtering improve on the unfiltered strategies?

- donchian: council (minimal) Sharpe 0.000 vs unfiltered -1.363, Calmar 0.000 vs -0.117, 0 entries vs unfiltered book: higher on both on point estimates.
- trend-pullback: council (minimal) Sharpe 0.000 vs unfiltered -2.218, Calmar 0.000 vs -0.202, 0 entries vs unfiltered book: higher on both on point estimates.
- momentum-rotation: council (full (no voter retained)) Sharpe -2.237 vs unfiltered -1.128, Calmar 0.216 vs 0.137, 5 entries vs unfiltered book: not higher on both on point estimates.
- crypto-burst: council (full (no voter retained)) Sharpe 0.000 vs unfiltered -30.921, Calmar 0.000 vs -1.000, 0 entries vs unfiltered book: higher on both on point estimates.

Ledger: council-v1 sweep `e99022faac258671258391f559fcaa854d58d5d4ce748c128c723b7266d578db` closed with Merkle root `17df945af6f58aaa541e732fe40f42e3ae20843f6d0d979af79758dd8d1e4a63`; ledger family N = 48; cross-trial Sharpe variance 0.132085.

## Honest statement: did council filtering improve on the unfiltered strategies?

**No, not in any way that counts.** The frozen rules give INCONCLUSIVE-SAMPLE for all four bases (G1 fails: fewer than 100 independent trades) and **INCONCLUSIVE** for council-v1 overall. No base is GO-DEV and nothing passed, so there is no claim of profit and no holdout run.

What the numbers show:

- **The council mostly turned strategies that lose after costs and tax into a book that does nothing.** The unfiltered books lost money in the development window (Donchian Sharpe -1.36 at the 6% cash rate, CAGR -2.1%, 115 entries; trend-pullback -2.22, CAGR -5.3%, 672 entries; momentum rotation -1.13, CAGR +1.2%, 22 entries; crypto burst -30.9, CAGR -72%, 2,048 entries). The council under test took **0 entries** for Donchian, trend-pullback and crypto burst and **5** for momentum rotation. The Time Machine vetoed 98.3% (Donchian), 100% (trend-pullback), 87.5% plus 12.5% abstentions (crypto) of candidates because the net-of-cost expectancy of each base's own shadow history is below the frozen 0.50% / 0.60% hurdle, and the Exit Planner's 2:1 net R:R gate vetoed 80% (Donchian) and 100% (crypto) on its own. That is the prereg's null ("the council only reduces trade count ... any Sharpe gain is the effect of holding cash") and it is what the data say.
- **Point-estimate "wins" (G3, G5, G8 pass for Donchian, trend-pullback and crypto) are the cash effect**, a book that never trades against a strategy that loses. They are not evidence of selection skill; the random-veto control (G6) exists to catch exactly this and was not run because G1 failed first (it only matters once there are 100 trades to compare).
- **No selection skill is visible.** Calibration (G7): Spearman of confidence-bin vs mean outcome is -0.54 (Donchian), -0.62 (trend-pullback), +0.14 (momentum rotation, not significant) and -0.06 (crypto), i.e. higher council confidence did not predict better outcomes; the PASS-minus-FAIL bootstrap CI lower bound is negative (Donchian -0.072, trend-pullback -0.014) or straddles zero. The Time Machine confidence is close to a constant 0 on these histories, so most bins are ties broken by date: treat the sign as noise, the point is that there is no monotone effect. For momentum rotation the council (full, no voter retained) took 5 entries against 22 and was slightly better at rf 0 (Sharpe 0.374 vs 0.305, max drawdown 3.9% vs 8.7%, CAGR 0.84% vs 1.20%) but worse at the cash rate and not better in 7 of 10 folds; its paired Sharpe-difference CI is [-1.85, -0.48].
- **Ablation**: for Donchian and trend-pullback only the Time Machine is "retained" (removing it adds back the losing trades: Sharpe +2.2, drawdown +9 pp for Donchian); that is the same cash effect, not a finding that the Time Machine predicts anything. No other voter is retained anywhere; for momentum rotation and crypto no voter is retained at all (the minimal council would be Scanner + Risk). Exit Planner and Time Machine votes are strongly redundant (phi 0.66 on momentum; Jaccard of joint vetoes 0.80 on Donchian); Entry Timer, Budget and Market/News are nearly uncorrelated with the others (the Budget voter never vetoes: it only shrinks).
- **Caps of the sample, not of the idea**: with the frozen gates a council that vetoes nearly everything cannot reach 100 trades, so the data cannot tell "no edge" from "council too strict". The prereg anticipated this (section 11) and says not to lower thresholds; they were not lowered.

## Integrity evidence (G10)

- `tq bias lookahead --strategy council-voters --data data` (prefix invariance of every vote, multiplier and Time Machine confidence, through the production adapters, on real bars of RELIANCE, TCS, HDFCBANK, INFY, ICICIBANK): all five **clean**; `tq bias lookahead --data data` over every registered strategy: all clean.
- Determinism: the full council of each base, run twice on the smoke window with the same seed, wrote a byte-identical decision log, identical returns and identical seal root (donchian, trend-pullback, momentum-rotation, crypto-burst: true). Re-run on the final code before the scored run.
- Seals: every one of the 48 decision logs was re-read, its hash chain verified and its final SEAL equal to the recorded root (G10 `seals verified true`; roots, tips and file hashes in `docs/result-council-v1-dev-cells.csv`). Logs themselves are git-ignored (`reports/council-v1-dev/run1/logs`).
- Ledger: the committed council-v1 OPEN (N = 48, hash `e99022fa...`) was CLOSED with the 48 cell series (Merkle root `17df945a...`, ledger tip `6a638476...`, mirrored in `hardening/ledger-anchor.txt`). No sweep was opened; ledger family N stays 48. The prereg `N_b` used for G2 is 162 / 110 / 74 / 51.

## EntryFilter hook leaves existing outputs byte-identical

`tq-backtest` gained an opt-in `EntryFilter` on `RiskWiring` (default `None`). Proof (`scripts/byteid_entry_filter.sh`): a `tq` release binary built from the commit before the hook (`c66e092`) and one built after were run on seven report variants (Donchian, trend-pullback and momentum rotation, each with and without `--hardened-risk`/`--calendar nse --settlement collateral:80`, plus Donchian per-symbol) over 2015-2019 on PIT NIFTY 50; the reports compare equal with `cmp` after normalising only the manifest path (sha256 prefixes of the normalised reports: donchian `d6484d85192c4614`, donchian-hw `7e99d3d42b3b6306`, tp `eee1a72cd159a85c`, tp-hw `ecb5f96e9ef92798`, mom `4193ffc94bad19c2`, mom-hw `fc2f1121b193ed0b`, donchian-ps `ec3a494f3fcc69d2`); the data manifest CSV is byte-identical (sha256 prefix `9633d8eba71f`). A unit test also shows a wired run with no filter equals the plain run, and a Scanner + Risk council (no ablatable voter) reproduces the unfiltered book exactly in this study (crypto burst: 2,048 entries, Sharpe -30.921 in both).

## Limits and deviations found (read before trusting any number)

- **Idle cash is not credited in the returns series.** The "Sharpe at rf" column subtracts the 6% cash rate from every day, so a mostly-cash book is penalised (momentum rotation council: -2.24 vs unfiltered -1.13) and a book that never trades has Sharpe 0 only because a zero-variance series is defined as 0. The protocol's like-for-like rule credits idle cash at the cash rate; this was not implemented. The "Sharpe rf 0" column is unaffected. No verdict depends on it (G1 fails first for every base) but the filtered-vs-unfiltered Sharpe-at-rf comparisons for low-exposure books should not be read as like-for-like.
- **Pooled equity calibration was not computed** (the prereg asks for it as a reported item): per-candidate records are not persisted by the study, and a re-run would count as new trials. Per-base calibration (which gates) is reported above; every per-base G7 fails.
- G9 is reported only; the 4c-rel alpha regression and invested-share routing are not computed. The random-veto control (G6) and the DSR for the three zero-trade books ("undefined: degenerate returns") are not available for the reasons above.
- Candidates are the engine's signals, so a vetoed breakout re-signals on later bars (addendum F); the candidate and decision-log counts (3,118 Donchian, 3,613 trend-pullback, 571 momentum, 3,743 crypto) are therefore larger than the number of distinct setups, and calibration counterfactuals for re-signals use the running shadow episode.
- Crypto: the Market voter needs 385 daily closes, so it abstains until about 2023-01-21 (255 candidates), and Time Machine buckets are unknown for all of 2022 (addendum B7); India VIX, Nifty 500 breadth, OI and earnings data are unavailable as stated in the prereg; the base signals have no demonstrated edge, so power to detect a council effect is low.
- Costs and tax follow the addendum (book-level tax post hoc on the engine's net equity curve; 1% TDS not modelled; crypto venue access for an Indian resident unverified). Paper and backtest only.

## Reproduce

`tq council-eval --all --window dev --root <repo with data/> --out-dir <fresh dir> --integrity-file <bias_clean/deterministic lines> --result-md <path> --code-sha <sha>` (a fresh `--out-dir` is required; decision logs are never appended to). A second run closes nothing new but opens a NEW sweep, which counts as 48 more trials. Smoke window (`--window smoke`) and single cells (`--base B --variant V`) are plumbing only and write nothing to the ledger. The holdout (`--window holdout`) is refused without `--holdout-ack`, which was not given.

---

# Generated tables

## donchian: INCONCLUSIVE-SAMPLE

Council under test: **minimal**. Scored window 2015-01-01 to 2019-12-31; first invested day = first bar on or after 2015-01-01.

| Book (net of costs, slippage, tax) | Entries | Sharpe at rf | Sharpe rf 0 | CAGR | Max DD | Calmar | Total return |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| unfiltered | 115 | -1.363 | -0.336 | -2.11% | 18.11% | -0.117 | -9.83% |
| full council | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| minimal (judged) | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| NIFTYBEES buy and hold | 0 | 0.234 | 0.693 | 8.54% | 21.60% | 0.395 | 49.09% |

### Gates

| Gate | Result | Value |
| --- | --- | --- |
| G1 Sample | FAIL | 0 independent trades (>= 100); 0 entry months / 0 entry weeks / 0 symbols; abstain share 1.7% of 3118 candidates (<= 95%) |
| G2 Deflation | FAIL | DSR undefined (degenerate returns) |
| G3 Beats unfiltered | PASS | Sharpe 0.000 vs -1.363, Calmar 0.000 vs -0.117; paired Sharpe-difference CI +1.363 [+0.381, +2.349] |
| G4 Beats index | FAIL | Sharpe 0.000 vs 0.234, Calmar 0.000 vs 0.395 (index net of costs and tax) |
| G5 Drawdown | PASS | max DD 0.00% vs index 21.60% and unfiltered 18.11% |
| G6 Not cash-hoarding | not computed | not computed: the random-veto control is only run when G1 passes |
| G7 Calibration | FAIL | 2066 signals (53 without confidence, 999 without outcome); Spearman -0.539; inversions 7; PASS-FAIL -0.042 [-0.072, -0.005]; council PASS-VETO (0 vs 2066) n/a |
| G8 Folds | FAIL | filtered beats unfiltered in 6 of 10 folds (need 6); total advantage +9.32 pp, largest single fold +6.47 pp |
| G9 Protocol (reported, blocks any profit claim) | FAIL | HAC t n/a (4c-abs needs >= 3); Sharpe 0.000 at rf 0 (4e needs >= 0.8); 4c-rel alpha regression not computed |
| G10 Integrity | PASS | lookahead clean Some(true); deterministic re-run Some(true); seals verified Some(true) |

**Verdict donchian: INCONCLUSIVE-SAMPLE.**

### Ablation

| Voter | dSharpe (full - loo) | dMaxDD pp (loo - full) | folds Sharpe worse | folds DD worse | retained |
| --- | ---: | ---: | ---: | ---: | --- |
| entry-timer | +0.000 | +0.00 | 0/10 | 0/10 | no |
| budget | +0.000 | +0.00 | 0/10 | 0/10 | no |
| exit-planner | +0.000 | +0.00 | 0/10 | 0/10 | no |
| market-news | +0.000 | +0.00 | 0/10 | 0/10 | no |
| time-machine | +2.235 | +9.10 | 9/10 | 10/10 | yes |

Minimal council: **time-machine** (redundancy drops: none); no voter retained: false.

Solo councils versus the Scanner + Risk book (reported, not gated):

| Voter | dSharpe | DD improvement pp |
| --- | ---: | ---: |
| entry-timer | -0.031 | +4.73 |
| budget | -0.274 | +4.23 |
| exit-planner | -0.622 | +4.66 |
| market-news | -0.841 | +4.66 |
| time-machine | +1.363 | +18.11 |

Pairwise vote correlation on the full council (phi of PASS/VETO over jointly voted candidates; Jaccard of joint vetoes):

| Pair | phi | joint-veto Jaccard | n both |
| --- | ---: | ---: | ---: |
| entry-timer x budget | n/a | 0.000 | 3118 |
| entry-timer x exit-planner | 0.023 | 0.379 | 3118 |
| entry-timer x market-news | 0.078 | 0.084 | 3118 |
| entry-timer x time-machine | n/a | 0.411 | 3065 |
| budget x exit-planner | n/a | 0.000 | 3118 |
| budget x market-news | n/a | 0.000 | 3118 |
| budget x time-machine | n/a | 0.000 | 3065 |
| exit-planner x market-news | 0.088 | 0.077 | 3118 |
| exit-planner x time-machine | n/a | 0.800 | 3065 |
| market-news x time-machine | n/a | 0.051 | 3065 |

### Trial table (12 variants)

| Variant | Entries | Candidates | PASS | SHRINK | VETO | ABSTAIN | Sharpe | Max DD | Seal root |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| full | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `88936c27a9a35b7e` |
| loo-entry-timer | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `907e60b213ef3a2e` |
| loo-budget | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `858cae4ee1ab9b9a` |
| loo-exit-planner | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `f77ea742a3514a63` |
| loo-market-news | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `58de5a05bc0716c1` |
| loo-time-machine | 57 | 2958 | 57 | 75 | 2826 | 0 | -2.235 | 9.10% | `a12fb45b0a2d81b3` |
| solo-entry-timer | 96 | 2808 | 194 | 0 | 2614 | 0 | -1.394 | 13.38% | `fca48530cb31122d` |
| solo-budget | 99 | 2810 | 112 | 84 | 2614 | 0 | -1.637 | 13.88% | `acebe3596d8bdd87` |
| solo-exit-planner | 65 | 2922 | 217 | 0 | 2705 | 0 | -1.985 | 13.46% | `d44513a5cf1e1592` |
| solo-market-news | 116 | 2776 | 87 | 202 | 2487 | 0 | -2.204 | 13.46% | `b60424196bbdf415` |
| solo-time-machine | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `d7adc190c0a1f3c3` |
| minimal | 0 | 3118 | 0 | 0 | 3065 | 53 | 0.000 | 0.00% | `d7adc190c0a1f3c3` |

### Full council: votes and abstention

PASS 0 / SHRINK 0 / VETO 3065 / ABSTAIN 53 of 3118 candidates

| Voter | PASS | VETO | ABSTAIN | NOT_AVAILABLE | Abstain rate | Veto rate | Sole vetoes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Scanner | 3118 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| MarketNews | 2910 | 208 | 0 | 0 | 0.0% | 6.7% | 4 |
| EntryTimer | 1841 | 1277 | 0 | 0 | 0.0% | 41.0% | 0 |
| ExitPlanner | 621 | 2497 | 0 | 0 | 0.0% | 80.1% | 0 |
| TimeMachine | 0 | 3065 | 53 | 0 | 1.7% | 98.3% | 375 |
| Risk | 3118 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| Budget | 3118 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| EarningsAnalyst | 0 | 0 | 0 | 3118 | 0.0% | 0.0% | 0 |

Reasons: ExitPlanner Veto RrBelowMin: 2497; TimeMachine Veto NetEdgeBelowHurdle: 1857; EntryTimer Veto EntryNotExtendedFail: 1263; TimeMachine Veto LcbNotAboveZero: 1208; MarketNews Veto VolP95: 163; TimeMachine Abstain ColdHistory: 53; MarketNews Veto RegimeFlags2Plus: 45; EntryTimer Veto EntryVolumeLow: 14

### Fold table (council under test vs unfiltered, compounded net return per fold)

| Fold | Council | Unfiltered |
| --- | ---: | ---: |
| 1 | 0.00% | -4.20% |
| 2 | 0.00% | -6.47% |
| 3 | 0.00% | 0.53% |
| 4 | 0.00% | -3.63% |
| 5 | 0.00% | 2.51% |
| 6 | 0.00% | 9.57% |
| 7 | 0.00% | -3.92% |
| 8 | 0.00% | -1.31% |
| 9 | 0.00% | 1.66% |
| 10 | 0.00% | -4.08% |
## trend-pullback: INCONCLUSIVE-SAMPLE

Council under test: **minimal**. Scored window 2015-01-01 to 2019-12-31; first invested day = first bar on or after 2015-01-01.

| Book (net of costs, slippage, tax) | Entries | Sharpe at rf | Sharpe rf 0 | CAGR | Max DD | Calmar | Total return |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| unfiltered | 672 | -2.218 | -1.039 | -5.28% | 26.07% | -0.202 | -23.08% |
| full council | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| minimal (judged) | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| NIFTYBEES buy and hold | 0 | 0.234 | 0.693 | 8.54% | 21.60% | 0.395 | 49.09% |

### Gates

| Gate | Result | Value |
| --- | --- | --- |
| G1 Sample | FAIL | 0 independent trades (>= 100); 0 entry months / 0 entry weeks / 0 symbols; abstain share 0.0% of 3613 candidates (<= 95%) |
| G2 Deflation | FAIL | DSR undefined (degenerate returns) |
| G3 Beats unfiltered | PASS | Sharpe 0.000 vs -2.218, Calmar 0.000 vs -0.202; paired Sharpe-difference CI +2.218 [+1.448, +2.989] |
| G4 Beats index | FAIL | Sharpe 0.000 vs 0.234, Calmar 0.000 vs 0.395 (index net of costs and tax) |
| G5 Drawdown | PASS | max DD 0.00% vs index 21.60% and unfiltered 26.07% |
| G6 Not cash-hoarding | not computed | not computed: the random-veto control is only run when G1 passes |
| G7 Calibration | FAIL | 2541 signals (0 without confidence, 1072 without outcome); Spearman -0.624; inversions 6; PASS-FAIL -0.008 [-0.014, -0.001]; council PASS-VETO (0 vs 2541) n/a |
| G8 Folds | PASS | filtered beats unfiltered in 8 of 10 folds (need 6); total advantage +25.62 pp, largest single fold +6.14 pp |
| G9 Protocol (reported, blocks any profit claim) | FAIL | HAC t n/a (4c-abs needs >= 3); Sharpe 0.000 at rf 0 (4e needs >= 0.8); 4c-rel alpha regression not computed |
| G10 Integrity | PASS | lookahead clean Some(true); deterministic re-run Some(true); seals verified Some(true) |

**Verdict trend-pullback: INCONCLUSIVE-SAMPLE.**

### Ablation

| Voter | dSharpe (full - loo) | dMaxDD pp (loo - full) | folds Sharpe worse | folds DD worse | retained |
| --- | ---: | ---: | ---: | ---: | --- |
| entry-timer | +0.000 | +0.00 | 0/10 | 0/10 | no |
| budget | +0.000 | +0.00 | 0/10 | 0/10 | no |
| exit-planner | +0.000 | +0.00 | 0/10 | 0/10 | no |
| market-news | +0.000 | +0.00 | 0/10 | 0/10 | no |
| time-machine | +6.558 | +1.60 | 8/10 | 8/10 | yes |

Minimal council: **time-machine** (redundancy drops: none); no voter retained: false.

Solo councils versus the Scanner + Risk book (reported, not gated):

| Voter | dSharpe | DD improvement pp |
| --- | ---: | ---: |
| entry-timer | +0.223 | +5.75 |
| budget | -0.135 | +12.73 |
| exit-planner | -3.445 | +24.40 |
| market-news | -1.023 | +13.88 |
| time-machine | +2.218 | +26.07 |

Pairwise vote correlation on the full council (phi of PASS/VETO over jointly voted candidates; Jaccard of joint vetoes):

| Pair | phi | joint-veto Jaccard | n both |
| --- | ---: | ---: | ---: |
| entry-timer x budget | n/a | 0.000 | 3613 |
| entry-timer x exit-planner | 0.046 | 0.142 | 3613 |
| entry-timer x market-news | 0.020 | 0.088 | 3613 |
| entry-timer x time-machine | n/a | 0.138 | 3613 |
| budget x exit-planner | n/a | 0.000 | 3613 |
| budget x market-news | n/a | 0.000 | 3613 |
| budget x time-machine | n/a | 0.000 | 3613 |
| exit-planner x market-news | -0.004 | 0.150 | 3613 |
| exit-planner x time-machine | n/a | 0.944 | 3613 |
| market-news x time-machine | n/a | 0.152 | 3613 |

### Trial table (12 variants)

| Variant | Entries | Candidates | PASS | SHRINK | VETO | ABSTAIN | Sharpe | Max DD | Seal root |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| full | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `011c743ac2ca4f50` |
| loo-entry-timer | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `b3662089455a8d01` |
| loo-budget | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `73f3f61db5c64c8b` |
| loo-exit-planner | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `8f63d71f259e52fd` |
| loo-market-news | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `67800a7bd56e1006` |
| loo-time-machine | 43 | 3571 | 68 | 40 | 3463 | 0 | -6.558 | 1.60% | `79a6d59db364a8be` |
| solo-entry-timer | 680 | 3051 | 1548 | 0 | 1503 | 0 | -1.994 | 20.32% | `adc31690996dfb9a` |
| solo-budget | 422 | 3260 | 312 | 296 | 2652 | 0 | -2.353 | 13.34% | `9798d77850e42924` |
| solo-exit-planner | 58 | 3564 | 155 | 0 | 3409 | 0 | -5.663 | 1.67% | `a39ccd61cb7da008` |
| solo-market-news | 407 | 3264 | 234 | 1293 | 1737 | 0 | -3.241 | 12.19% | `37b6407d5a1bf5bf` |
| solo-time-machine | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `0bc3c4e1fdc27855` |
| minimal | 0 | 3613 | 0 | 0 | 3613 | 0 | 0.000 | 0.00% | `0bc3c4e1fdc27855` |

### Full council: votes and abstention

PASS 0 / SHRINK 0 / VETO 3613 / ABSTAIN 0 of 3613 candidates

| Voter | PASS | VETO | ABSTAIN | NOT_AVAILABLE | Abstain rate | Veto rate | Sole vetoes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Scanner | 3613 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| MarketNews | 3065 | 548 | 0 | 0 | 0.0% | 15.2% | 0 |
| EntryTimer | 3113 | 500 | 0 | 0 | 0.0% | 13.8% | 0 |
| ExitPlanner | 203 | 3410 | 0 | 0 | 0.0% | 94.4% | 0 |
| TimeMachine | 0 | 3613 | 0 | 0 | 0.0% | 100.0% | 162 |
| Risk | 3613 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| Budget | 3613 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| EarningsAnalyst | 0 | 0 | 0 | 3613 | 0.0% | 0.0% | 0 |

Reasons: TimeMachine Veto NetEdgeBelowHurdle: 3613; ExitPlanner Veto RrBelowMin: 3410; EntryTimer Veto EntryNotExtendedFail: 399; MarketNews Veto VolP95: 305; MarketNews Veto RegimeFlags2Plus: 243; EntryTimer Veto EntryVolumeLow: 101

### Fold table (council under test vs unfiltered, compounded net return per fold)

| Fold | Council | Unfiltered |
| --- | ---: | ---: |
| 1 | 0.00% | 0.70% |
| 2 | 0.00% | -3.75% |
| 3 | 0.00% | -0.44% |
| 4 | 0.00% | -1.05% |
| 5 | 0.00% | -2.71% |
| 6 | 0.00% | 0.52% |
| 7 | 0.00% | -5.10% |
| 8 | 0.00% | -6.14% |
| 9 | 0.00% | -2.44% |
| 10 | 0.00% | -5.21% |
## momentum-rotation: INCONCLUSIVE-SAMPLE

Council under test: **full (no voter retained)**. Scored window 2015-01-01 to 2019-12-31; first invested day = first bar on or after 2015-01-01.

| Book (net of costs, slippage, tax) | Entries | Sharpe at rf | Sharpe rf 0 | CAGR | Max DD | Calmar | Total return |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| unfiltered | 22 | -1.128 | 0.305 | 1.20% | 8.71% | 0.137 | 5.92% |
| full council | 5 | -2.237 | 0.374 | 0.84% | 3.87% | 0.216 | 4.11% |
| full (no voter retained) (judged) | 5 | -2.237 | 0.374 | 0.84% | 3.87% | 0.216 | 4.11% |
| NIFTYBEES buy and hold | 0 | 0.234 | 0.693 | 8.54% | 21.60% | 0.395 | 49.09% |

### Gates

| Gate | Result | Value |
| --- | --- | --- |
| G1 Sample | FAIL | 5 independent trades (>= 100); 5 entry months / 5 entry weeks / 4 symbols; abstain share 25.9% of 571 candidates (<= 95%) |
| G2 Deflation | FAIL | DSR 0.0000 at N_b = 74 (>= 0.95); at ledger N = 48: 0.0000 |
| G3 Beats unfiltered | FAIL | Sharpe -2.237 vs -1.128, Calmar 0.216 vs 0.137; paired Sharpe-difference CI -1.109 [-1.850, -0.477] |
| G4 Beats index | FAIL | Sharpe -2.237 vs 0.234, Calmar 0.216 vs 0.395 (index net of costs and tax) |
| G5 Drawdown | PASS | max DD 3.87% vs index 21.60% and unfiltered 8.71% |
| G6 Not cash-hoarding | not computed | not computed: the random-veto control is only run when G1 passes |
| G7 Calibration | FAIL | 402 signals (148 without confidence, 21 without outcome); Spearman 0.139; inversions 5; PASS-FAIL +0.005 [-0.032, +0.040]; council PASS-VETO (6 vs 396) +0.037 [-0.192, +0.268] |
| G8 Folds | FAIL | filtered beats unfiltered in 3 of 10 folds (need 6); total advantage -1.83 pp, largest single fold +1.52 pp |
| G9 Protocol (reported, blocks any profit claim) | FAIL | HAC t 0.89 (4c-abs needs >= 3); Sharpe 0.374 at rf 0 (4e needs >= 0.8); 4c-rel alpha regression not computed |
| G10 Integrity | PASS | lookahead clean Some(true); deterministic re-run Some(true); seals verified Some(true) |

**Verdict momentum-rotation: INCONCLUSIVE-SAMPLE.**

### Ablation

| Voter | dSharpe (full - loo) | dMaxDD pp (loo - full) | folds Sharpe worse | folds DD worse | retained |
| --- | ---: | ---: | ---: | ---: | --- |
| entry-timer | -0.624 | +0.00 | 2/10 | 4/10 | no |
| budget | -0.799 | +1.97 | 1/10 | 6/10 | no |
| exit-planner | -0.251 | +0.00 | 1/10 | 2/10 | no |
| market-news | +0.000 | +0.00 | 0/10 | 0/10 | no |
| time-machine | -0.727 | +0.14 | 2/10 | 7/10 | no |

Minimal council: **none** (redundancy drops: none); no voter retained: true.

Solo councils versus the Scanner + Risk book (reported, not gated):

| Voter | dSharpe | DD improvement pp |
| --- | ---: | ---: |
| entry-timer | -0.031 | +0.07 |
| budget | -0.305 | +1.09 |
| exit-planner | -0.425 | +3.87 |
| market-news | -0.072 | -0.08 |
| time-machine | -0.281 | +5.06 |

Pairwise vote correlation on the full council (phi of PASS/VETO over jointly voted candidates; Jaccard of joint vetoes):

| Pair | phi | joint-veto Jaccard | n both |
| --- | ---: | ---: | ---: |
| entry-timer x budget | -0.021 | 0.000 | 571 |
| entry-timer x exit-planner | 0.034 | 0.064 | 571 |
| entry-timer x market-news | -0.004 | 0.045 | 571 |
| entry-timer x time-machine | 0.040 | 0.070 | 423 |
| budget x exit-planner | -0.129 | 0.000 | 571 |
| budget x market-news | -0.040 | 0.000 | 571 |
| budget x time-machine | -0.101 | 0.000 | 423 |
| exit-planner x market-news | 0.129 | 0.205 | 571 |
| exit-planner x time-machine | 0.663 | 0.733 | 423 |
| market-news x time-machine | 0.216 | 0.092 | 423 |

### Trial table (12 variants)

| Variant | Entries | Candidates | PASS | SHRINK | VETO | ABSTAIN | Sharpe | Max DD | Seal root |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| full | 5 | 571 | 3 | 3 | 417 | 148 | -2.237 | 3.87% | `9ede6eb3487c723c` |
| loo-entry-timer | 6 | 559 | 3 | 3 | 405 | 148 | -1.613 | 3.87% | `f8a98f1e74429cc1` |
| loo-budget | 8 | 555 | 9 | 0 | 399 | 147 | -1.438 | 5.84% | `2cdcc496c1f11691` |
| loo-exit-planner | 6 | 566 | 3 | 3 | 412 | 148 | -1.986 | 3.87% | `5d66d521e1bc0892` |
| loo-market-news | 5 | 571 | 3 | 3 | 417 | 148 | -2.237 | 3.87% | `068cc7853dd5fecb` |
| loo-time-machine | 8 | 551 | 2 | 6 | 543 | 0 | -1.510 | 4.01% | `620a0f82c506f3b1` |
| solo-entry-timer | 13 | 540 | 60 | 0 | 480 | 0 | -1.160 | 8.64% | `7fb7b03791a7c387` |
| solo-budget | 12 | 545 | 7 | 8 | 530 | 0 | -1.433 | 7.63% | `a190a5918b819af5` |
| solo-exit-planner | 8 | 558 | 10 | 0 | 548 | 0 | -1.553 | 4.84% | `a7ce95051a14d493` |
| solo-market-news | 14 | 542 | 23 | 37 | 482 | 0 | -1.200 | 8.79% | `6870e3365288df55` |
| solo-time-machine | 9 | 564 | 30 | 0 | 378 | 156 | -1.409 | 3.66% | `f22c89e6c071f4fb` |
| minimal | 22 | 535 | 106 | 0 | 429 | 0 | -1.128 | 8.71% | `a2047f17ca084ae6` |

### Full council: votes and abstention

PASS 3 / SHRINK 3 / VETO 417 / ABSTAIN 148 of 571 candidates

| Voter | PASS | VETO | ABSTAIN | NOT_AVAILABLE | Abstain rate | Veto rate | Sole vetoes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Scanner | 571 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| MarketNews | 467 | 104 | 0 | 0 | 0.0% | 18.2% | 7 |
| EntryTimer | 537 | 34 | 0 | 0 | 0.0% | 6.0% | 2 |
| ExitPlanner | 170 | 401 | 0 | 0 | 0.0% | 70.2% | 56 |
| TimeMachine | 205 | 218 | 148 | 0 | 25.9% | 38.2% | 0 |
| Risk | 195 | 376 | 0 | 0 | 0.0% | 65.8% | 121 |
| Budget | 567 | 4 | 0 | 0 | 0.0% | 0.7% | 4 |
| EarningsAnalyst | 0 | 0 | 0 | 571 | 0.0% | 0.0% | 0 |

Reasons: ExitPlanner Veto RrBelowMin: 401; Risk Veto RiskGateRejected: 376; TimeMachine Veto WinrateLbBelowBreakeven: 169; TimeMachine Abstain ColdHistory: 148; MarketNews Veto VolP95: 67; TimeMachine Veto LcbNotAboveZero: 39; MarketNews Veto RegimeFlags2Plus: 37; EntryTimer Veto EntryNotExtendedFail: 25; TimeMachine Veto NetEdgeBelowHurdle: 10; EntryTimer Veto EntryVolumeLow: 9; Budget Veto FreqCapDay: 4

### Fold table (council under test vs unfiltered, compounded net return per fold)

| Fold | Council | Unfiltered |
| --- | ---: | ---: |
| 1 | -1.53% | -1.27% |
| 2 | -0.40% | -1.92% |
| 3 | 2.52% | 2.64% |
| 4 | 1.85% | 1.13% |
| 5 | 0.62% | 4.00% |
| 6 | 1.05% | 2.24% |
| 7 | 0.00% | -0.89% |
| 8 | 0.00% | 0.00% |
| 9 | 0.00% | 0.00% |
| 10 | 0.00% | 0.00% |
## crypto-burst: INCONCLUSIVE-SAMPLE

Council under test: **full (no voter retained)**. Scored window 2023-01-01 to 2023-12-31; first invested day = first bar on or after 2023-01-01.

| Book (net of costs, slippage, tax) | Entries | Sharpe at rf | Sharpe rf 0 | CAGR | Max DD | Calmar | Total return |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| unfiltered | 2048 | -30.921 | -30.921 | -72.11% | 72.11% | -1.000 | -72.11% |
| full council | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| full (no voter retained) (judged) | 0 | 0.000 | 0.000 | 0.00% | 0.00% | 0.000 | 0.00% |
| BTC spot buy and hold | 0 | 1.756 | 1.756 | 106.71% | 22.51% | 4.741 | 106.71% |

### Gates

| Gate | Result | Value |
| --- | --- | --- |
| G1 Sample | FAIL | 0 independent trades (>= 100); 0 entry months / 0 entry weeks / 0 symbols; abstain share 12.5% of 3743 candidates (<= 95%) |
| G2 Deflation | FAIL | DSR undefined (degenerate returns) |
| G3 Beats unfiltered | PASS | Sharpe 0.000 vs -30.921, Calmar 0.000 vs -1.000; paired Sharpe-difference CI +30.921 [+29.388, +33.060] |
| G4 Beats index | FAIL | Sharpe 0.000 vs 1.756, Calmar 0.000 vs 4.741 (index net of costs and tax) |
| G5 Drawdown | PASS | max DD 0.00% vs index 22.51% and unfiltered 72.11% |
| G6 Not cash-hoarding | not computed | not computed: the random-veto control is only run when G1 passes |
| G7 Calibration | FAIL | 3274 signals (469 without confidence, 0 without outcome); Spearman -0.055; inversions 5; PASS-FAIL -0.000 [-0.001, +0.001]; council PASS-VETO (0 vs 3274) n/a |
| G8 Folds | PASS | filtered beats unfiltered in 4 of 4 folds (need 3); total advantage +109.28 pp, largest single fold +28.76 pp |
| G9 Protocol (reported, blocks any profit claim) | FAIL | HAC t n/a (4c-abs needs >= 3); Sharpe 0.000 at rf 0 (4e needs >= 0.8); 4c-rel alpha regression not computed |
| G10 Integrity | PASS | lookahead clean Some(true); deterministic re-run Some(true); seals verified Some(true) |

**Verdict crypto-burst: INCONCLUSIVE-SAMPLE.**

### Ablation

| Voter | dSharpe (full - loo) | dMaxDD pp (loo - full) | folds Sharpe worse | folds DD worse | retained |
| --- | ---: | ---: | ---: | ---: | --- |
| entry-timer | +0.000 | +0.00 | 0/4 | 0/4 | no |
| budget | +0.000 | +0.00 | 0/4 | 0/4 | no |
| exit-planner | +0.000 | +0.00 | 0/4 | 0/4 | no |
| market-news | +0.000 | +0.00 | 0/4 | 0/4 | no |
| time-machine | +0.000 | +0.00 | 0/4 | 0/4 | no |

Minimal council: **none** (redundancy drops: none); no voter retained: true.

Solo councils versus the Scanner + Risk book (reported, not gated):

| Voter | dSharpe | DD improvement pp |
| --- | ---: | ---: |
| entry-timer | +18.664 | +56.86 |
| budget | +16.548 | +58.16 |
| exit-planner | +30.921 | +72.11 |
| market-news | +24.680 | +59.58 |
| time-machine | +30.921 | +72.11 |

Pairwise vote correlation on the full council (phi of PASS/VETO over jointly voted candidates; Jaccard of joint vetoes):

| Pair | phi | joint-veto Jaccard | n both |
| --- | ---: | ---: | ---: |
| entry-timer x budget | n/a | 0.000 | 3743 |
| entry-timer x exit-planner | n/a | 0.901 | 3743 |
| entry-timer x market-news | n/a | 0.000 | 3488 |
| entry-timer x time-machine | n/a | 0.916 | 3274 |
| budget x exit-planner | n/a | 0.000 | 3743 |
| budget x market-news | n/a | n/a | 3488 |
| budget x time-machine | n/a | 0.000 | 3274 |
| exit-planner x market-news | n/a | 0.000 | 3488 |
| exit-planner x time-machine | n/a | 1.000 | 3274 |
| market-news x time-machine | n/a | 0.000 | 3274 |

### Trial table (12 variants)

| Variant | Entries | Candidates | PASS | SHRINK | VETO | ABSTAIN | Sharpe | Max DD | Seal root |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| full | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `6720b770a1d90d15` |
| loo-entry-timer | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `27cc367e86521648` |
| loo-budget | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `cab4872b23eca4cc` |
| loo-exit-planner | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `3b421f3121277d28` |
| loo-market-news | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `6abd681a03358e5c` |
| loo-time-machine | 0 | 3743 | 0 | 0 | 3488 | 255 | 0.000 | 0.00% | `3a509aff497be4dc` |
| solo-entry-timer | 291 | 3541 | 291 | 0 | 3250 | 0 | -12.258 | 15.26% | `625d87d66c2a07d3` |
| solo-budget | 314 | 3454 | 150 | 164 | 3140 | 0 | -14.373 | 13.96% | `2bcf7aaf738a5146` |
| solo-exit-planner | 0 | 3743 | 0 | 0 | 3743 | 0 | 0.000 | 0.00% | `6fcff2c144f7d17a` |
| solo-market-news | 297 | 3459 | 163 | 134 | 2907 | 255 | -6.241 | 12.53% | `63b27d9362a46902` |
| solo-time-machine | 0 | 3743 | 0 | 0 | 3274 | 469 | 0.000 | 0.00% | `a1ee462a86fbcf6d` |
| minimal | 2048 | 2048 | 2048 | 0 | 0 | 0 | -30.921 | 72.11% | `6560456ebe075a94` |

### Full council: votes and abstention

PASS 0 / SHRINK 0 / VETO 3274 / ABSTAIN 469 of 3743 candidates

| Voter | PASS | VETO | ABSTAIN | NOT_AVAILABLE | Abstain rate | Veto rate | Sole vetoes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Scanner | 3743 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| MarketNews | 3488 | 0 | 255 | 0 | 6.8% | 0.0% | 0 |
| EntryTimer | 372 | 3371 | 0 | 0 | 0.0% | 90.1% | 0 |
| ExitPlanner | 0 | 3743 | 0 | 0 | 0.0% | 100.0% | 96 |
| TimeMachine | 0 | 3274 | 469 | 0 | 12.5% | 87.5% | 0 |
| Risk | 3743 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| Budget | 3743 | 0 | 0 | 0 | 0.0% | 0.0% | 0 |
| EarningsAnalyst | 0 | 0 | 0 | 3743 | 0.0% | 0.0% | 0 |

Reasons: ExitPlanner Veto RrBelowMin: 3743; EntryTimer Veto EntryNotExtendedFail: 3371; TimeMachine Veto NetEdgeBelowHurdle: 3274; MarketNews Abstain InputMissing: 255; TimeMachine Abstain InputMissing: 255; TimeMachine Abstain ColdHistory: 214

### Fold table (council under test vs unfiltered, compounded net return per fold)

| Fold | Council | Unfiltered |
| --- | ---: | ---: |
| 1 | 0.00% | -27.90% |
| 2 | 0.00% | -28.76% |
| 3 | 0.00% | -27.22% |
| 4 | 0.00% | -25.40% |

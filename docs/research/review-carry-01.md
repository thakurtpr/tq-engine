# Task 17: independent review of carry-01 (funding carry, edge-hunt candidate A)

Reviewer: thakur-op2, 2026-11-02. Scope: `engine/reports/carry-01.md`, `engine/research/carry_backtest.py` (commit 86e0d5c), data in
`engine/data/{funding,spot-1h,perp-1h,mark-1h}`. Read-only on `engine/`. Scripts: `docs/review-carry-01-scripts/` (`carry_replica.py`,
`carry_checks.py`, JSON summaries). The replica is pure Python without pandas, written from the spec and the report's description of its
deviations, not copied from the code. It reads the same data files, so it does not test the data itself (I did not re-verify the archive checksums).

## Verdict

**The numbers reproduce exactly and the kill is sound. A fails kill criteria 1 and 3, and the recent funding is lower than even the report
suggests. Two statements in the engine report need fixing: M = 1.0N is not "effectively 1x", and tax case (b) is not the worst case.
Deviations 1 and 2 are acceptable; deviation 3 is acceptable as coded but incomplete.**

1. **Reproduction: exact.** Full-window return, HAC t, max drawdown, in-/out-of-sample returns, rebalance counts (37 / 78 BTC, 62 / 141 ETH),
   no liquidations, and every yearly row of the pre-tax, funding, spot-gain, tax (b) and TDS (c) columns match (section 1).
2. **Funding is signed correctly** (section 2): the first BTC payment is negative (-0.0124%) and the short pays; flipping the sign gives -6.33% / -7.63%, the mirror image of the base run.
3. **The decomposition holds** (section 2): pre-tax return minus funding minus (spot + perp price PnL) is -0.05 to -0.44 percentage points a year, i.e. costs and compounding.
4. **Recent funding is below the hurdle on any denominator.** Last-12-month funding is 3.47% (BTC) and 2.51% (ETH) a year on notional, so even a position with no
   margin capital could not reach 8% (section 3). The "OOS 3.7% on capital" in the report is about 7% on notional, which looks close to 8%: the recent run-rate is not.
5. **E is not worth a backtest** (section 5): a hold-to-expiry basis would have to be 12-16% on notional to clear 8% on capital; recent funding is 2.5-3.5%.

## 1. Reproduction (M = 1.0N unless noted)

| | Engine | Replica |
| --- | --- | --- |
| BTC full / HAC t / max DD | +6.40% / 10.23 / -1.18% | +6.40% / 10.23 / -1.18% |
| BTC in-sample / out-of-sample / ratio | +8.40% / +3.66% / 0.44 | +8.39% / +3.66% / 0.44 |
| ETH full / HAC t / max DD | +7.41% / 9.27 / -0.94% | +7.41% / 9.27 / -0.94% |
| ETH in-sample / OOS / ratio | +10.22% / +3.57% / 0.35 | +10.22% / +3.57% / 0.35 |
| M = 0.5N full: BTC / ETH | +8.16% / +9.42% | +8.16% / +9.42% |
| Rebalances M=1.0 / 0.5 | BTC 37 / 78, ETH 62 / 141 | identical |
| Liquidations | none | none |

The 0.01-point difference in BTC in-sample (8.40 vs 8.39) is a window-boundary rounding difference. The year tables for all four runs agree to the second decimal.

## 2. Funding sign and decomposition

- **Sign:** funding is `q * mark_open * rate`, added to the short's wallet, so a positive rate pays the short (correct for Binance perps: longs pay shorts when positive).
  The first BTC row, 2020-01-01 00:00, is -0.00012359, so the short pays. Sign-flip stress: -6.33% (BTC) and -7.63% (ETH) against +6.40% and +7.41%.
- **Decomposition** (M = 1.0, pre-tax minus funding minus spot and perp price PnL): BTC residuals -0.33, -0.16, -0.06, -0.05, -0.16, -0.06, -0.13 points for 2020 to 2026;
  ETH -0.43, -0.44, -0.21, -0.05, -0.07, -0.19, -0.12. These are the entry and exit costs, rebalance costs and compounding, and are small.
- **Minor defect: funding payments dropped.** The code applies funding only at hours that exist in all of the spot, perp and mark series.
  36 of 7,488 BTC payments (sum of rates +0.116% of notional over 6.8 years) and 18 of 7,488 ETH payments (+0.154%) fall in hours missing from at least one series and are silently lost.
  Impact is about 0.02% a year: immaterial, but fix it by accruing funding independently of the kline index. There are 22 (BTC) and 19 (ETH) hour gaps in the common index;
  the last is two days, 2026-10-28 and 29, after which data resumes through 2026-10-31 23:00.
- **Not a decomposition error but worth stating:** the yearly "funding" column is a fraction of capital, not notional. In 2021 BTC raw funding sums to 30.6% of notional, which would be
  15.3% of capital at a constant 50% notional share, but the report shows 18.49%. The difference is notional growing with price between rebalances (section 4).

## 3. Funding decay, from raw data

Annual funding yield on constant notional (sum of the 8-hour rates), % per year:

| | 2020 | 2021 | 2022 | 2023 | 2024 | 2025 | 2026 to 31 Oct |
| --- | --- | --- | --- | --- | --- | --- | --- |
| BTCUSDT | 17.2 | 30.6 | 4.2 | 7.9 | 12.0 | 5.1 | 2.6 |
| ETHUSDT | 27.5 | 37.5 | 0.8 | 8.3 | 13.0 | 4.9 | 1.7 |

(The 2026 column is the sum of rates over the 10 months, not annualised.)

- Last 12 months: **3.47% (BTC) and 2.51% (ETH)** on notional, funding positive in **77.3% and 73.6%** of payments. The report's 77% / 74% are correct.
- This is a stronger form of the engine's finding: the hurdle fails even if the capital denominator were notional only, because the premium has decayed from 12-13% in 2024 to a few percent now.
- Caution on the in-sample 8.4% / 10.2% figures: they are dominated by 2020 and 2021 (17-37% of notional), a regime the 2025 to 2026 data does not resemble.

## 4. Deviations 1 to 3

1. **Liquidation on MARK-price highs: acceptable, and confirmed necessary.** In the data, the ETH perp's last-price high at 2020-03-13 02:00 is 323.0 against a mark-price high of 119.47
   (+170%). A last-price rule liquidates the position there for both M = 1.0N and 0.5N; on mark price there is no liquidation. Binance liquidates on mark price, so the engine's reading of
   "1h bar high" is the right one. BTC's largest last-vs-mark excess is +3.1% (2023-10-23 22:00) and does not liquidate either way. Residual risk the backtest cannot show: mark-price
   manipulation and auto-deleveraging events; none occurred in this window.
2. **Margin band instead of the |delta| > 2% trigger: acceptable, but it changes the leverage the report claims.** With equal quantities the delta trigger can never fire, so
   some margin rule is needed, and "no rebalance" confirms it: it liquidates BTC on 2020-11-05 and ETH on 2020-02-12. However the band [0.5m, 1.5m] lets the perp's margin ratio fall to
   half the initial margin before resizing, i.e. up to 2x leverage on the short leg (and 4x for M = 0.5N), so M = 1.0N is only "effectively 1x" at entry. That is also why the return on capital
   exceeds the funding yield on a constant half-notional. Sensitivity of the band (BTC M = 1.0, full / out-of-sample / rebalances): [0.25, 1.75] +6.97% / +3.93% / 16;
   [0.5, 1.5] +6.40% / +3.66% / 37; [0.7, 1.3] +5.96% / +3.55% / 94; [0.85, 1.15] +5.66% / +3.33% / 350. ETH (full / OOS): +8.24 / +3.81, +7.41 / +3.57, +6.94 / +3.38, +6.46 / +3.12.
   **No band gets the out-of-sample figure near 8%, so the kill is robust to the choice.** The band is a new un-pre-registered parameter; report it as such and state the leverage it permits.
3. **Tax case (b), spot gains taxed annually: acceptable as coded, but not the worst case, and the gap is in my spec.** The spec's (b) taxed positive funding and spot gains
   with no set-off of perp losses. It omitted the mirror image: in a falling year the short perp gains and the spot loss is not deductible. Adding the perp's positive annual price PnL
   at 30% (case b2 in `carry_replica.py`) turns 2022 from +1.48% to **-12.41%** for BTC (ETH +0.12% to -11.43%), and 2026 to date for ETH from +0.48% to -2.03%. Years negative under (b): BTC 2020, 2023, 2024;
   ETH 2020, 2021, 2023, 2024, 2025. Under (b2): BTC adds 2022; ETH adds 2022 and 2026. Taxing realised rather than annual spot gains defers but does not remove the drag, because the perp losses
   stay non-deductible. Kill criterion 3 therefore fires under every variant I tried; whether 115BBH applies at all is still the open question for a CA.

## 5. Is E (dated cash-and-carry) worth running?

My judgement: **no full backtest; at most a cheap read of the current basis.**

- **Arithmetic.** A locked annualised basis b on notional gives about b / 2 on capital at M = 1.0N (b / 1.5 at 0.5N). To reach 8% on capital the basis must be about
  16% (M = 1.0N) or 12% (M = 0.5N), before roll costs. Realised perpetual funding over the last 12 months was 3.5% and 2.5%.
- **No reason to expect more.** A dated future's fair basis is the expected funding over its life plus a term premium; the 2024 to 2026 decay in funding implies the quarterly basis has
  compressed too. I have no delivery-contract price data and no source for current quarterly basis levels; treat that as **unverified**.
- **Costs.** E rolls four times a year on the futures leg; at 5 bps taker plus 1 bp slippage that is about 24 bps a year, around a tenth of the current funding yield.
- **Window.** The archive starts 2021-02, so E's in-sample years are the same high-funding 2021 to 2023 period that already overstates the regime (`docs/binance-futures-facts-16.md`).
- **What I would do instead, if anyone wants to check:** download a handful of daily-kline files for the two nearest quarterly contracts of BTC (a few hundred KB), compute the annualised
  basis against spot over 2025 and 2026, and stop if it is under about 12%. That needs the user's explicit OK per file, so I have not done it.

## 6. Other observations

- **HAC t of 9 to 10 is not evidence that the hurdle is cleared.** Funding carry has low variance, so the drift is highly significant against zero; kill criterion 2 (t < 2) is too weak to matter here.
  The criteria that bind are 1 and 3.
- **Capital denominator.** The report's returns are on N + M capital (spot plus margin). Whether spot assets can serve as collateral for the futures leg (portfolio or unified margin) would
  change the denominator and the liquidation risk: **unverified**, not tested.
- **Costs** (7.5 / 5 / 1 bps) match the spec and the cited pages; the perp rate is the same cited example, not a confirmed schedule (`docs/binance-futures-facts-16.md`).
- **The tax cases and TDS case are not advice** and depend on classification questions that remain unresolved (`docs/edge-hunt-01.md`).

## 7. Recommendations

1. Close A for an Indian-resident retail account: it fails 1 and 3 and the current run-rate is 2.5 to 3.5% on notional.
2. Fix the engine report: state the real leverage band, show both (b) and (b2) instead of calling (b) the worst case, and fix the funding-hour drop.
3. Do not build a full E backtest; do the optional cheap basis check only with the user's OK.
4. Record the A result in the trials ledger (candidates tested so far: 3 intraday strategies and A).
5. Remaining research candidates (B, listings, index changes) are unproven; the honest expectation stays "single-digit percent at best".

## Reproducing

```
cd docs/review-carry-01-scripts && python3 carry_replica.py && python3 carry_checks.py
```
Each runs in about a second. They write `carry_replica_summary.json` and `carry_checks_summary.json` beside the scripts and read `engine/data` only.

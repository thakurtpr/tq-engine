# Re-run with a capital-gains tax stage: momentum-rotation vs NIFTYBEES, 2026-11-07

New dated file. `reports/backtest-04.md` (2026-10-30) is history and was not edited: it stays pre-tax as published. This file answers the gap named in `docs/cost-model-audit-2026-11.md` item 1 and section 6 step 7 ("add a tax stage to backtest-04 as a new dated report").

Window and configuration are exactly backtest-04's headline rows: 2012-01-01 to 2021-12-31 development window (the holdout was not touched), Rs 10 lakh shared account, point-in-time NIFTY 50 universe for MomentumRotation (top 10), 100% NIFTYBEES for BuyAndHold and IndexTrendFilter, drawdown kill switch off (`--max-drawdown-pct 100`), adopted slippage model at x1, x2 and x3. Nothing was tuned, no threshold or gate was changed, and no result file was edited. Reproduce with `scripts/rerun_tax_stage.sh <tq> <out-dir>` (branch `hardening/rates`).

## Verdict

**The tax stage widens the gap against momentum rotation; the verdict ("neither beats NIFTYBEES") is unchanged and cannot flip.** Pre-tax, momentum trails buy-and-hold by 0.29 CAGR points. After tax it trails by 0.60 points. The Sharpe gap goes from 0.81 vs 0.96 to 0.73 vs 0.90.

The size of the widening (0.31 points at x1 slippage) is much smaller than the audit's hand estimate of "about 1 to 1.5 points" (audit item 1 estimated momentum losing 1.7 to 2.2 CAGR points and buy-and-hold about 0.8). The run measures 1.19 points lost by momentum and 0.88 by buy-and-hold. The audit estimate assumed about 80% short-term gains and no netting; I did not decompose why the run differs (no trade-level breakdown by holding period was produced), so treat the audit number as superseded by this run, not explained by it.

## What was run

`tq backtest ... --tax-stage-out <file>` (new flag; the standard report and summary CSV are unchanged without it). The stage takes the run's own equity curve and trades and applies the listed-equity capital-gains model:

- STCG (held under 365 days) and LTCG by exit date from the date-keyed rates table, 4% cess, no surcharge. For 2012-2021 that resolves to 15% STCG (15.6% with cess), LTCG exempt before FY2018-19 and 10% (10.4%) above Rs 1 lakh from FY2018-19. The 2024-07-23 change (20% / 12.5% / Rs 1.25 lakh) is in the table but no trade in this window reaches it.
- Gains and losses net within each financial year, per bucket (short, long). No loss carry-forward, no short-term loss against long-term gain.
- Tax is paid on each 31 March out of the equity curve (the unfinished last year at the last bar).
- **Positions still open at the end are taxed as if sold at the last mark** (entry costs only, no exit cost). Without this the buy-and-hold row would show no tax at all, because the rebalancing engine reports only closed trades. This needed a new report field, `unrealised_at_end`, that no metric or report text reads.
- Pre-tax figures come from the same `returns::stats` the standard report uses, so they reconcile with backtest-04: 14.02%, 14.31% and 7.14% CAGR match its published table.

## Results (after tax, same window)

| Run | CAGR pre-tax | CAGR after tax | Tax cost (CAGR pts) | Sharpe pre / after | Max DD pre / after | Tax paid (Rs) | of which on open positions at end (Rs) |
|---|---:|---:|---:|---:|---:|---:|---:|
| MomentumRotation, slip x1 | 14.02% | 12.83% | 1.19 | 0.81 / 0.73 | 36.01% / 37.99% | 370,101 | 97,135 |
| MomentumRotation, slip x2 | 13.70% | 12.52% | 1.18 | 0.80 / 0.71 | 36.21% / 38.17% | 358,812 | 94,842 |
| MomentumRotation, slip x3 | 13.37% | 12.20% | 1.17 | 0.78 / 0.70 | 36.23% / 38.17% | 346,261 | 91,484 |
| BuyAndHold NIFTYBEES, slip x1 | 14.31% | 13.43% | 0.88 | 0.96 / 0.90 | 36.27% / 36.27% | 281,584 | 281,584 |
| BuyAndHold NIFTYBEES, slip x2 | 14.30% | 13.42% | 0.88 | 0.96 / 0.90 | 36.28% / 36.28% | 281,222 | 281,222 |
| BuyAndHold NIFTYBEES, slip x3 | 14.29% | 13.41% | 0.88 | 0.96 / 0.90 | 36.28% / 36.28% | 280,875 | 280,875 |
| IndexTrendFilter, slip x1 | 7.14% | 6.74% | 0.40 | 0.71 / 0.67 | 21.17% / 21.67% | 73,760 | 66,161 |

Momentum minus buy-and-hold, CAGR points: x1 -0.29 pre-tax, -0.60 after tax; x2 -0.60 and -0.90; x3 -0.92 and -1.21. Final equity at x1: momentum Rs 33.43 lakh vs Rs 35.26 lakh for buy-and-hold after tax (Rs 37.14 lakh vs Rs 38.08 lakh before).

Every run printed `unverified rates: 5 of 5` (momentum, trend filter) or `unverified rates: 4 of 4` (buy-and-hold, which only touches the long-term rows): every rate the stage used is from secondary sources, so none is marked verified.

## Reading it honestly

- **Same conclusion, a little stronger.** Momentum was already below the index before tax, and the tax hits it harder (more short-term churn: 476 closed trades, 295% one-way turnover a year). No cell is anywhere near flipping.
- **Buy-and-hold's tax is an upper bound.** The model has no Section 112A grandfathering: NIFTYBEES held from 2012 would have its cost basis stepped up to the 2018-01-31 value, which would cut the Rs 2.8 lakh terminal tax substantially (the audit's hand estimate for this effect is item 8, "unverified, from knowledge"). Fixing it makes buy-and-hold look better after tax, so the gap against momentum widens further, not narrows.
- **Momentum's tax may be slightly overstated** by the missing loss carry-forward and the missing short-term-loss-against-long-term-gain set-off (audit item 10), and slightly understated by ignoring surcharge (capped at 15% on these gains; not modelled).
- **The after-tax curve is not re-simulated.** Tax is subtracted from the pre-tax equity curve on each 31 March; the cash is not removed from the account the strategy trades with. This slightly flatters every after-tax number (money taxed away would have compounded). It affects both books and is small relative to the 0.3 point widening, but it was not measured.
- **Stage inputs are net P&L.** Capital gains use the trade's net P&L (after brokerage, STT, stamp, DP and slippage). STT is not deductible against capital gains (audit item 9, about 3 bps per winning round trip), so tax is slightly understated. Open positions use entry costs only.
- **Stamp duty before 2020-07-01** is still charged at the uniform 2020 rate (audit item 11): a small, conservative overcharge on the buy leg that this run did not change.
- Pre-tax numbers do not depend on any of the above. Drawdowns rise after tax because tax payments are lumpy 31 March withdrawals.

## What this does not claim

Nothing here is a profitability claim. Both strategies were already below the pre-registered bar; the tax stage only removes the possibility that the omission of tax was hiding a different answer. The rates are unverified (secondary sources: `docs/tax-india-equity.md` s.2, confirmed from an Income Tax Department FAQ quoted by press, statute not read). No holdout data was used.

# Pre-registration o2-c17: weekly put back-ratio spread (Nifty)

Type: A

Master: `docs/prereg-options-v2.md` (frozen at tag `prereg-options-v2`). Ledger family `options-v2`, catalogue family `o2f-c17`, 3 cells. Low priority: run last. Paper and backtest only.

## Hypothesis and mechanism
Index put skew makes near-the-money puts expensive relative to farther out-of-the-money puts. A put back-ratio sells one nearer put and buys two puts `width_points` further out: it collects (or nearly zero-costs) the rich near strike and owns convexity in a crash, while the loss zone sits between the two strikes. The question: net of costs, slippage and 31.2% business-income tax, does the structure earn a Sharpe above the Nifty ETF and the 50/50 ETF+BTC blend. Null: no net edge. Convexity honesty: beyond the long strikes the position is long convexity, but in the zone between strikes and near the short strike it is short gamma, and the master prereg lists ratios under net short convexity. It is attested short convex (true): the stricter reading, which makes G9 (stress exposure) apply.

## Structure and rule (fixed, family `c17_back_ratio_jade_lizard`, variant `back_ratio`, right `put`)
* Short 1 weekly Nifty put at strike = spot x (1 - offset_sigma x sigma); long 2 puts `width_points` = 200 further out. sigma = trailing 20-return realised volatility of closes on or before the decision day, scaled by sqrt(calendar days to expiry / 365) (A2.8). Defined risk: maximum loss at the long strike, so no stress loss cap (`stress_loss_cap_frac` must be exactly 0).
* Expiry: the weekly contract with at least `min_dte` = 3 calendar days remaining. Entry: decision on the Friday close, fill at the next open (`weekday:fri`); exit one session before expiry (default). Premium-based exits are not set.
* Hold H = 1 period (one weekly cycle).
* The jade-lizard variant (naked short put, stress label) is NOT registered here and consumes no cell.

## Axis and levels
Axis `offset_sigma`: 0.5, 1, 1.5 (ascending in string order, A2.10). Base = 1, the middle: the short strike one cycle-sigma out; 0.5 sells nearer to the money, 1.5 farther. The width is the registry's worked-example value (200 points), fixed from the strike-grid rule (four 50-point strikes), not from a return. Neighbours are robustness cells (G6) only.

## Fixed parameters
`variant` back_ratio; `right` put; `width_points` 200; `min_dte` 3; `stress_loss_cap_frac` 0.0; `entry` weekday:fri; cost multiplier 1 (2x stress derived by the adapter); tax business income 31.2% with loss carry-forward (default); account Rs 1 crore. Disclosed limitation: a fixed width in points is a different fraction of spot across 2019-2026.

## Underlying, expiry, windows
NIFTY, weekly (the only weekly survivor of the 2024-11 SEBI change). Windows `d1` (2021-11-01..2025-04-30), `p2` (weekly Nifty contracts from 2019-02 until 2021-10-31: type A blind replication on development data, spends alpha_2), `s3` (sealed 2025-05-01..2026-10-31, one batch touch).

## Sizing
Fixed notional per A1.5: 3% weekly risk budget (defined maximum loss), 6% single-lot cap, minimum one lot, skips (signal-day liquidity) <= 15% of scheduled cycles.

## What is NOT tuned
Right, width, `min_dte`, ratio (1 : 2), entry day, exit day, sigma definition. No change after a result.

## Evidence disclosure
No result for this candidate has been seen beyond what the master prereg discloses. The master gates (G1-G11, Appendix A type A hurdles, alpha_1 = 0.139, alpha_2 = 0.10, alpha_3 = 0.10) apply UNCHANGED; none is restated or added here.

## Possible verdicts
PROMOTED TO FORWARD PAPER (never "passed" or "profitable"), KILL, INCONCLUSIVE-SAMPLE, INCONCLUSIVE-TAIL, INCONCLUSIVE-SIZE, INCONCLUSIVE-QUALITY, INCOMPLETE, as the master gates define them. A near-zero-premium structure can be too quiet (G11 below 6%: INCONCLUSIVE-SIZE).

## What would make this candidate not run (never a failure)
Spec refused by the loader; the index close series missing; skips above 15% of cycles; fewer than 24 months of Phase-2 periods (INCOMPLETE under A2.11); carried marks above 10% of leg-days; study pins unset.

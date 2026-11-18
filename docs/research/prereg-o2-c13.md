# Pre-registration o2-c13: expiry-day open-to-settle iron butterfly (Nifty weekly)

Type: B

Master: `docs/prereg-options-v2.md` (frozen at tag `prereg-options-v2`). Ledger family `options-v2`, catalogue family `o2f-c13`, 3 cells. Paper and backtest only.

## Hypothesis and mechanism
On expiry day, time value decays to zero within hours and near-the-money options are priced for a wide intraday move; if realised open-to-settlement movement of the index is on average smaller than the move implied by the open prices, selling an at-the-money straddle with bought wings earns the difference, with the wings capping the loss. The question: net of costs, slippage (cheap, illiquid expiry-day wings are the cost risk) and 31.2% business-income tax, does the Sharpe exceed the Nifty ETF and the 50/50 ETF+BTC blend. Null: no net edge. Net short convexity (butterfly): a large expiry-day move beyond the wings is the loss. Classified type B in the master table: a cousin of V3 (condor/butterfly machinery), and by availability, because only about 21 months of expiry-day marks exist in Phase 2 (below the 24-month type-A requirement). Only that lineage note anchors the design; no options-v1 result document was opened.

## Structure and rule (fixed, family `c13_expiry_day_iron_butterfly`)
* On the close of the session before an expiry (the eve), decide; fill at the expiry-day open: sell the ATM call and put, buy the call and put at ATM +/- wing, wing = round(spot x sigma x wing_sigma) points, with sigma the 1-day horizon volatility (trailing 20-return realised vol of closes on or before the decision day, scaled to one calendar day).
* Hold to expiry: settled at the final settlement price (intrinsic from the index close or underlying settlement, never an option row's SETTLE_PR, A2.12). `entry` every:1 (the policy gates itself on the calendar), `hold_to_expiry` true (A2.9). Exchange-holiday eve cycles are lost, never guessed. No profit target or stop.
* Weekly Nifty expiries only; the expiry weekday follows the contract rules in force (Thursday, Tuesday from 2025-09-02).
* Hold H = 1 period (one expiry day inside a weekly period).

## Axis and levels
Axis `wing_sigma` (1-day sigma units): 0.5, 0.75, 1 (ascending in string order, A2.10), as in the catalogue row. Base = 0.75, the middle: a wing wide enough to be outside the typical open-to-settle range yet cheap enough to hold a defined, tradable loss; 0.5 is a tight wing (higher cost share, smaller loss); 1 is a wide wing with more credit at risk. Neighbours are robustness cells (G6) only.

## Fixed parameters
`entry` every:1; `hold_to_expiry` true; cost multiplier 1 (2x stress derived by the adapter); tax business income 31.2% with loss carry-forward (default); account Rs 1 crore.

## Underlying, expiry, windows
NIFTY, weekly. Windows `d1` (2021-11-01..2025-04-30) and `s3` (sealed 2025-05-01..2026-10-31, one batch touch). No `p2`: expiry-day option marks are reliable only from 2020-02-06 (settle semantics, `docs/dq-scan-options-v2.md` E01/E02), about 21 months of Phase 2, below 24, and the earlier settle is zero. Phase 2 is reported as "not run: availability" and is never read as a failure; type B spends no Phase-2 alpha.

## Sizing
Fixed notional per A1.5: 3% weekly risk budget, 6% single-lot cap, minimum one lot, skips (signal-day liquidity) <= 15% of scheduled cycles. The structure is defined risk (iron butterfly), so no stress loss cap is needed.

## What is NOT tuned
Wing offsets beyond the 3 levels, the body (ATM), the entry time (open), the exit (settlement), the sigma definition, any filter or day-of-week selection. No change after a result.

## Evidence disclosure
No result for this candidate has been seen beyond what the master prereg discloses. The master gates (type B hurdles of Appendix A, alpha_1 = 0.0139, alpha_3 = 0.10, G1-G11) apply UNCHANGED; none is restated or added here. Short convexity is attested true, so G9 applies.

## Possible verdicts
PROMOTED TO FORWARD PAPER (never "passed" or "profitable"), KILL, INCONCLUSIVE-SAMPLE, INCONCLUSIVE-TAIL, INCONCLUSIVE-SIZE, INCONCLUSIVE-QUALITY, INCOMPLETE (including an expiry-booking evidence failure), as the master gates define them.

## What would make this candidate not run (never a failure)
Spec refused by the loader; the index close series (needed for settlement) missing; expiry-booking evidence unavailable; skips above 15% of cycles (illiquid expiry-day wings); carried marks above 10% of leg-days; study pins unset.

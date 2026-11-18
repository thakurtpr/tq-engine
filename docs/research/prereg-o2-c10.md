# Pre-registration o2-c10: monthly cash-secured put (short OTM Nifty put)

Type: B

Master: `docs/prereg-options-v2.md` (frozen at tag `prereg-options-v2`). Ledger family `options-v2`, catalogue family `o2f-c10`, 3 cells. Paper and backtest only.

## Hypothesis and mechanism
Put-write (CBOE PUT index lineage): index puts are rich because investors pay for crash protection, so selling an OTM put and holding the cash that secures it earns a premium that on average exceeds the expected payoff. The question: net of option costs, slippage and 31.2% business-income tax, does the Sharpe exceed the Nifty ETF and the 50/50 ETF+BTC blend. Null: no net edge. The structure is net short convexity: bounded gain (the premium), loss growing as the index falls. The master table classifies it type B (short-put leg of the options-v1 V1 lineage, doubt rule): Phase 2 is a consistency check that spends no alpha, and the error is spent in D1 (alpha_1 = 0.0139) and S3 only. The only anchor used is that lineage note; no options-v1 result document was opened.

## Structure and rule (fixed, family `c10_monthly_cash_secured_put`)
* Short one monthly Nifty put at strike = spot x (1 - offset_sigma x sigma); sigma = trailing 20-return realised volatility of index closes dated on or before the decision day, scaled by sqrt(calendar days to expiry / 365) (A2.8). No cash credit is booked on the securing capital (A1.3).
* Expiry: the monthly contract with at least `min_dte` = 10 calendar days remaining (the engine's worked-example value; stated rule: hold about a calendar month).
* Entry: `when_flat`, decision on the close, fill at the next open. Exit one session before expiry (`exit_before_expiry_days` 1). No profit target or stop.
* Hold H = 1 period (one calendar-monthly cycle).

## Axis and levels
Axis `offset_sigma`: 0.5, 1, 1.5 (ascending in string order, A2.10). Base = 1, the middle: the put one cycle-sigma out of the money, the usual neutral put-write strike; 0.5 collects more premium for more frequent losses, 1.5 is a deep OTM put with little premium. Neighbours are robustness cells (G6) only.

## Fixed parameters
`min_dte` 10; `entry` when_flat; `exit_before_expiry_days` 1; cost multiplier 1 (the 2x stress is derived by the adapter); tax business income 31.2% with loss carry-forward (default); account Rs 1 crore.

## Underlying, expiry, windows
NIFTY, monthly. Windows `d1` (2021-11-01..2025-04-30), `p2` (2012-01-01..2021-10-31, reported consistency check, no alpha), `s3` (sealed 2025-05-01..2026-10-31, one batch touch). Index only; stock puts are not registered (assignment is NOT-RUN per the catalogue).

## Sizing
Fixed notional per A1.5: 5% monthly risk budget, 6% single-lot cap, minimum one lot, skips <= 15% of scheduled cycles. The net short put runs through the SPAN-like stress path, which refuses a zero loss cap, so the stress loss cap is the monthly risk share, 5% (`SPAN_LOSS_CAP_MONTHLY`, A2.7): an entry whose one-lot stress loss exceeds it is skipped, not resized. G10 (6% floor) and G11 (6-14% volatility band) apply on the fixed Rs 1 crore; a book too quiet is INCONCLUSIVE-SIZE.

## What is NOT tuned
Offset beyond the 3 levels, `min_dte`, entry cadence, exit day, sigma definition, any filter. No change after a result.

## Evidence disclosure
No result for this candidate has been seen beyond what the master prereg discloses. The master gates (type B hurdles of Appendix A, alpha_1 = 0.0139, alpha_3 = 0.10, G1-G11) apply UNCHANGED; none is restated or added here. Short convexity is attested true, so G9 applies.

## Possible verdicts
PROMOTED TO FORWARD PAPER (never "passed" or "profitable"), KILL, INCONCLUSIVE-SAMPLE, INCONCLUSIVE-TAIL, INCONCLUSIVE-SIZE, INCONCLUSIVE-QUALITY, INCOMPLETE, as the master gates define them. With about 42 monthly cycles in D1 the N_eff >= 30 gate may be hard to reach (INCONCLUSIVE-SAMPLE); that is a statement about the data, not a failure.

## What would make this candidate not run
Spec refused by the loader; the index close series missing from the run inputs; skips above 15% of cycles; carried marks above 10% of leg-days; study pins unset.

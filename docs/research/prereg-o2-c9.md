# Pre-registration o2-c9: monthly covered call (long Nifty future + short OTM call)

Type: A

Master: `docs/prereg-options-v2.md` (frozen at tag `prereg-options-v2`). Ledger family `options-v2`, catalogue family `o2f-c9`, 3 cells. Paper and backtest only.

## Hypothesis and mechanism
Index options carry a variance risk premium: implied volatility on average exceeds what is later realised, so a short call is sold above its expected payoff. A long index future supplies the underlying exposure (so the call is covered) and earns the equity risk premium and cost of carry. Buy-write literature (Whaley BXM) reports lower volatility than the index with similar returns. The question: does the call premium, net of futures and option costs, slippage and 31.2% business-income tax, lift the Sharpe above the Nifty ETF and the 50/50 ETF+BTC blend. Null: no net edge. The structure is net short convexity (capped upside, full downside): it is exposed to a sharp fall and gains nothing from a rally above the strike.

## Structure and rule (fixed, family `c9_monthly_covered_call`)
* Long one Nifty near-month future (lot count from the master sizing) and short one monthly Nifty call at strike = spot x (1 + offset_sigma x sigma), where sigma = trailing 20-return realised volatility of index closes dated on or before the decision day, scaled by sqrt(calendar days to expiry / 365) (A2.8).
* Expiry: the monthly contract with at least `min_dte` = 10 calendar days remaining. 10 is the engine's worked-example value, chosen from the stated rule (hold about one calendar month, avoid the last fortnight), not from any return.
* Entry: `when_flat` (one cycle at a time, decision on the close, fill at the next open). Exit: one session before expiry (`exit_before_expiry_days` 1), no premium-based exit. `profit_target` and `stop_loss` are refused for C9 (A2.9: the position premium base includes futures notional).
* Hold H = 1 period (one calendar-monthly cycle).

## Axis and levels (one numeric axis, three levels, base in the middle)
Axis `offset_sigma`: 0.5, 1, 1.5 (ascending in canonical string order, A2.10). Base = 1: the call at one cycle-sigma is the neutral buy-write strike, between the near-the-money buy-write (0.5, more premium, more capped upside) and the far OTM overwrite (1.5, little premium). The base is the middle level by rule; no return was consulted. Neighbours are robustness cells only (G6).

## Fixed parameters
`min_dte` 10; `stock_lot_policy` "all"; `entry` when_flat; `exit_before_expiry_days` 1; cost multiplier 1 (the G7 2x stress is derived by the adapter); tax business income 31.2% with loss carry-forward (default); account Rs 1 crore.

## Underlying, expiry, windows
NIFTY, monthly. Windows `d1` (2021-11-01..2025-04-30), `p2` (2012-01-01..2021-10-31: type A, a blind replication on development data that spends alpha_2), `s3` (sealed 2025-05-01..2026-10-31, one batch touch). Lots: the printed lot, else the date-keyed table (A2.12).

## Sizing
Fixed notional per A1.5: 5% monthly risk budget per structure, 6% single-lot cap, minimum one lot, skips <= 15% of scheduled cycles. Because the futures leg is margined on the SPAN-like stress path, which refuses a zero loss cap, the stress loss cap is the monthly risk share, 5% (`SPAN_LOSS_CAP_MONTHLY`, A2.7): an entry whose one-lot stress loss exceeds it is skipped, never resized.

## Stock lot-uncertainty (sensitivity)
The catalogue row names index and stocks. Only the index version is registered (one underlying per spec); stock covered calls are UNSELECTED and consume no cell. For the index, `stock_lot_policy` is fixed to "all". A `verified_only` lot sensitivity row is not a cell: it would be a dated, counted re-run from the spare reserve, reported beside this result and never a hypothesis.

## What is NOT tuned
Offset beyond the 3 registered levels, `min_dte`, entry cadence, exit day, sigma definition, lot policy, stock set. No parameter changes after a result.

## Evidence disclosure
No result for this candidate has been seen beyond what the master prereg discloses. The master prereg's gates (G1-G11, Appendix A type A hurdles, alpha_1 = 0.139, alpha_2 = 0.10, alpha_3 = 0.10) apply UNCHANGED; none is restated or added here. Short convexity is attested true, so G9 applies.

## Possible verdicts
PROMOTED TO FORWARD PAPER (never "passed" or "profitable"), KILL, INCONCLUSIVE-SAMPLE, INCONCLUSIVE-TAIL, INCONCLUSIVE-SIZE, INCONCLUSIVE-QUALITY, INCOMPLETE, as the master gates define them. A volatility-band miss (G11) is INCONCLUSIVE-SIZE, not a kill.

## What would make this candidate not run (never a failure)
Spec refused by the loader; index futures rows or the index close series missing from the run inputs; skips above 15% (the zero-volume share of index futures rows is high in 2021-22 per `docs/dq-scan-options-v2.md`; the near month is the traded leg); carried marks above 10% of leg-days; study pins unset.

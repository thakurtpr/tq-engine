# Pre-registration o2-c11: trend-filter overlay on the monthly covered call

Type: A

Master: `docs/prereg-options-v2.md` (frozen at tag `prereg-options-v2`). Ledger family `options-v2`, catalogue family `o2f-c11`, 3 cells. Paper and backtest only.

## Hypothesis and mechanism
Large index drawdowns cluster in persistent downtrends, and a covered call is long-delta with capped upside: it loses most when the index trends down. A trend filter (hold the structure only while the index close is above its simple moving average, stand aside otherwise) removes the buy-write from most of those regimes. The question: net of costs, slippage and 31.2% business-income tax, does the filtered buy-write earn a Sharpe above the Nifty ETF and the 50/50 ETF+BTC blend. Null: no net edge. The overlay is a wrapper: it adds no option leg of its own, so it is net short convex whenever the inner covered call is on.

## Structure and rule (family `c11_trend_overlay`)
* Decision on the close: SMA = mean of the last `lookback` index closes dated on or before the decision day. Close above SMA: trade `inner_up`. Close below or equal: no position (`inner_down` is null, stand aside). Fewer than `lookback` closes available: no decision (counted as a skip).
* `inner_up` = `c9_monthly_covered_call` with offset_sigma 1.0, min_dte 10, stock_lot_policy "all" (the C9 base; fixed, not an axis). Its structure is a long Nifty near-month future plus a short monthly call at spot x (1 + 1.0 x sigma), sigma as in A2.8. The inner structure is the C9 buy-write (not a v1 structure); the catalogue's directional long call/put and debit-spread legs have no registered policy family.
* Entry `when_flat`; exit one session before expiry (`exit_before_expiry_days` 1). `profit_target` and `stop_loss` are refused (the overlay inherits the C9 ban, A2.9).
* Hold H = 1 period (one calendar-monthly cycle).

## Axis and levels
Axis `lookback` (sessions): 150, 200, 250. Base = 200, the middle and the standard long trend filter. The catalogue wrote 50 / 100 / 200, but those levels sort as text 100, 200, 50 and are refused at spec load (A2.10); 150 / 200 / 250 is the smallest change that keeps 200 as the base and is strictly ascending as text. Chosen by this rule before any result. Neighbours are robustness cells (G6) only.

## Fixed parameters
`inner_up` as above; `inner_down` null; `entry` when_flat; `exit_before_expiry_days` 1; cost multiplier 1 (2x stress derived by the adapter); tax business income 31.2% with loss carry-forward (default); account Rs 1 crore.

## Underlying, expiry, windows
NIFTY, monthly. Windows `d1` (2021-11-01..2025-04-30), `p2` (2012-01-01..2021-10-31, type A blind replication, spends alpha_2), `s3` (sealed 2025-05-01..2026-10-31, one batch touch). Index closes start 2012-02-21, so the first 150-250 sessions of Phase 2 have no filter and no trade (skips counted).

## Sizing
Fixed notional per A1.5 (5% monthly risk budget, 6% single-lot cap, minimum one lot, skips <= 15% of scheduled cycles). The sizing shape is the inner covered call's: the futures leg runs on the SPAN-like stress path with the monthly 5% stress loss cap (A2.7); an entry whose one-lot stress loss exceeds it is skipped.

## What is NOT tuned
The inner offset and `min_dte`, the filter type (SMA only), the down-leg (none), entry cadence, exit day. No change after a result.

## Evidence disclosure
No result for this candidate has been seen beyond what the master prereg discloses. The master gates (G1-G11, Appendix A type A hurdles, alpha_1 = 0.139, alpha_2 = 0.10, alpha_3 = 0.10) apply UNCHANGED; none is restated or added here. Short convexity is attested true (G9 applies). Disclosure: the inner leg is the C9 base, so C11 and C9 are positively dependent; both count as separate hypotheses in m = 36.

## Possible verdicts
PROMOTED TO FORWARD PAPER (never "passed" or "profitable"), KILL, INCONCLUSIVE-SAMPLE, INCONCLUSIVE-TAIL, INCONCLUSIVE-SIZE, INCONCLUSIVE-QUALITY, INCOMPLETE, as the master gates define them. A filter that keeps the book out of falling markets can leave G5's adverse regime unevaluable (INCONCLUSIVE-TAIL) or the book too quiet (INCONCLUSIVE-SIZE).

## What would make this candidate not run
Spec refused by the loader (including an inner family or parameter the registry refuses); index futures rows or index closes missing from the run inputs; skips above 15% of scheduled cycles; carried marks above 10% of leg-days; study pins unset.

# Closure note 01: VWAP reversion, momentum burst, NSE ORB

Date: 2026-11-02. Author: thakur-op2. Status: **closed** (requested by thakur-op1, decision 1 of the edge-hunt round 1 message).

## Decision

The three pre-registered intraday strategies in `engine/crates/live/src/strategies/` (VWAP reversion, momentum burst on
1m Binance BTCUSDT/ETHUSDT; opening-range breakout on NSE 5m) are closed. No further tuning, no new variants, no
paper or live run. The code stays in the repository as a reference and as the regression baseline for the intraday runner.

## Evidence (all reproducible from the scripts named)

| Finding | Source |
| --- | --- |
| The engine's intraday-01 numbers reproduce exactly in an independent replica | `docs/review-intraday-01.md` |
| Gross edge per trade: BTC VWAP +0.30 bps [+0.02, +0.58], ETH VWAP +0.34 [+0.02, +0.65], BTC momentum -0.83 [-1.72, +0.05], ETH momentum +0.75 [-0.36, +1.86], ORB -6.9 [-21, +8] | `docs/fee-breakeven-15.md` |
| Round-trip costs of any retail venue (7.5 bps per side with BNB on Binance spot; about 4 bps all-in on NSE equity before slippage) are 10 to 50 times the best gross edge | `docs/fee-breakeven-15.md` section 4 |
| Even a maker rebate with taker stops (68-75% of trades are stops) leaves -0.3 to -1.9 bps per trade | `docs/fee-breakeven-15.md` section 2 |
| ORB's 60-session sample cannot identify an edge; 14 of 153 trades were overnight carries from missing Yahoo bars | `docs/review-intraday-01.md` section 3 |

## What this does and does not show

- It shows these three fixed, long-only rules have no edge a retail account could collect. It does not show scalping or
  intraday trading in general is impossible: shorts, maker-side market making with queue modelling, order-book signals and
  other coins were not tested.
- The one finding with a CI above zero (VWAP, 2023) is 0.3 to 0.6 bps, below any realistic cost, and absent in 2022.
- The compounding-account design in intraday-01 truncated the VWAP samples (accounts were wiped out); future gross-edge
  research should use constant notional (as in task 15).

## Residual items (not blocking closure)

1. The `SEE_SMOKE_SECTION` placeholder in `engine/reports/intraday-01.md` and the "zero edge" wording are for the engine owners to fix.
2. The live paper journal stopped on 2026-10-30 14:12 UTC; nothing is running and nothing should be restarted for these strategies.
3. Reopen only if new evidence appears, such as real NSE intraday history showing an ORB interval clearly above 8 bps, or a
   cost structure below 0.5 bps all-in, which no venue we found offers.

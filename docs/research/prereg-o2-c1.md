# Pre-registration o2-c1: weekly bull put spread, Nifty

Type: B
Family: `c1_weekly_bull_put_spread`; catalogue F5; master prereg classification C1 (v1 V1 lineage). Tag to be created by the owner process after validation, never by this agent.

## Hypothesis and mechanism
Index put options are on average priced above the downside move that is later realised (a variance risk premium, with a demand for crash protection). Selling an out-of-the-money put and buying a further-out put harvests that premium with a bounded loss. The test is whether, after Indian brokerage, STT, exchange charges, slippage and 31.2% business-income tax, any of it is left on weekly Nifty cycles. The null is no net edge; a premium seller with zero true edge looks profitable in calm samples, which is why the gates exist.

## Structure and strike/expiry rule
Short one put at a strike `offset_sigma` x sigma_h below spot, long one put 200 points lower (same expiry). sigma_h = trailing 20-return realised volatility (annualised x sqrt(252)) scaled by sqrt(calendar days to expiry / 365) (A2.8), computed from closes on or before the decision day. Underlying NIFTY, weekly expiry, nearest weekly expiry with at least 3 calendar days to expiry (`min_dte` 3). Strikes snap to the listed grid by the engine's selection rule; fills use the conservative fill rule of the catalogue (section 2.2).

## Entry and exit
Entry on the engine's default weekly entry rule on the decision day (no engine key set: `entry`, `hold_to_expiry`, `profit_target`, `stop_loss`, `max_hold_sessions`, `exit_before_expiry_days` are all left at the registry defaults, i.e. the position is held to expiry; no take-profit, no stop). Expiry-day marks come from the underlying final settlement, never SETTLE_PR (A1.7). Hold H = 1 period (one weekly cycle).

## Axis and levels (chosen before any result)
Axis `offset_sigma`, levels 0.75 / 1 / 1.25 (strictly ascending in canonical string order, A2.10). Base = 1: a short strike about one sigma out is the common ~16-delta practice and is the multiple justified by v1's roughly 1-sigma monthly offset (the lineage note of the master prereg, the only anchor used; no options-v1 result was opened). 0.75 is a closer, richer-premium, higher-hit-risk neighbour; 1.25 a further, thinner-premium neighbour; equal 0.25 steps around the base.

## Fixed parameters
`width_points` 200, `min_dte` 3, underlying NIFTY, expiry weekly. Windows: d1 and s3 (type B: Phase 2 spends no alpha and is not a registered window; weekly Nifty only, so the S3 rule is met). Expected skip rate: only the signal-day liquidity filter can skip; stated from the manifest at the first run, not guessed here.

## Sizing, cost, tax, account
Fixed notional per master prereg A1.5: every cycle trades the same number of whole lots, lots = max(1, floor(3.0% x Rs 1 crore / max loss per lot at the start of the window)); weekly budget 3.0%; if one lot alone risks more than 6.0% of capital the verdict is INCONCLUSIVE-SIZE. Defined-risk margin = maximum loss. Account Rs 1 crore fixed capital, cost multiplier 1 (the 2x stress run is the harness's G7, not a cell), tax = default business-income path (31.2%, loss carry-forward) per section 4 of the master prereg. Returns are net of costs and tax divided by the fixed initial capital. No premium or implied-volatility input sizes any cycle.

## Cells and what is NOT tuned
Cells = 3 (one axis x 3 levels), counted against the 60-cell cap; the base (middle) cell alone is the hypothesis, the two neighbours are robustness cells only (G6). Not tuned and not looked at: every fixed parameter above, the exit rules, the entry weekday, the windows, the axis levels. No argmax, no re-pick of the base after any result.

## Disclosure and gates
No result has been seen for this candidate beyond what the master prereg discloses (options-v1 monthly structures were looked at, which is why the type is as signed). The master prereg gates (G1..G10, hurdles, stage alphas) apply UNCHANGED to this candidate; none is restated or added here. Wording rule: nothing is called "passed" or "profitable"; no expected performance is claimed.

## Possible verdicts
The master prereg's: KILL, INCONCLUSIVE (TAIL, SIZE, SAMPLE, QUALITY, INCOMPLETE) or PROMOTED TO FORWARD PAPER (a promotion is never evidence of profitability). A failed S3 after an earlier pass is read under the stage's own contract regime (section 2).

## What would make this candidate not run
A refused spec at load (any cell failing configure or build); missing NIFTY index close or India VIX series in the run inputs (fail closed, A2.12); a dq-scan failure of G8 on NIFTY data; the cap of 60 cells or the sealed-touch cap exhausted; a dated addendum declaring the family NOT-RUN. Then it is recorded as "not run: reason", never as a failure.

# Pre-registration o2-c4: conditional defined-risk VRP, weekly, Nifty

Type: B
Family: `c4_vrp_conditional`; catalogue F15; master prereg classification C4 (cousin of v1 V1/V3: the same structures with an entry filter).

## Hypothesis and mechanism
The variance risk premium is not constant: when implied volatility is rich relative to recent realised volatility, the premium sold is larger relative to the movement that follows, so selling only on rich days should earn more per cycle than selling every week. The hypothesis is that this filter, applied to the weekly iron condor, leaves a net edge after Indian costs, slippage and 31.2% business-income tax. The filter removes cycles, so exposure to quiet weeks is lower; it cannot remove the short-convexity tail. Null: no net edge.

## Structure, filter and strike/expiry rule
Structure `iron_condor` (as C2): short put and call each `offset_sigma` x sigma_h from spot, long wings 200 points beyond. sigma_h as in A2.8 (20 returns, closes on or before the decision day, x sqrt(calendar days / 365)). Entry filter signal = ATM implied vol minus trailing realised vol (A2.8), computed on the decision day from the chain; trade only if signal >= the MEDIAN of the signal over the prior `history_sessions` = 252 sessions strictly before the decision day (`threshold` median); with fewer than `min_history` = 60 valid prior values the cycle is skipped (no trade). NIFTY, weekly, nearest weekly expiry with `min_dte` 3. Conservative fills.

## Entry and exit
Engine defaults for every engine key (none set): weekly decision, held to expiry, no take-profit, no stop. Filtered-out cycles enter the HAC series as zero P&L with the skip count reported; above 15% skips the verdict is INCONCLUSIVE-SAMPLE (A1.5). A median filter skips about half the cycles by construction, so this candidate is expected to be INCONCLUSIVE-SAMPLE on skip share unless the harness counts filter skips differently from liquidity skips; the registration states this risk plainly and changes no threshold for it. Hold H = 1 period.

## Axis and levels (chosen before any result)
Ambiguity resolved: the catalogue lists the filter threshold (zero / median / p75) as the C4 axis, but the master prereg row gives no axis, and the loader requires levels strictly ascending in canonical string order (A2.10), under which the three thresholds sort as median, p75, zero, putting p75 in the middle and breaking the zero-median-p75 ordering. The ONE axis is therefore `offset_sigma`, levels 0.75 / 1 / 1.25 with base 1 (same rationale as C2: about one sigma per side; 0.25 steps), and the threshold is fixed at median, the middle of the catalogue's three. The threshold levels are NOT explored: this is a deliberate narrowing, decided before any result.

## Fixed parameters
`structure` iron_condor, `width_points` 200, `min_dte` 3, `threshold` median, `history_sessions` 252, `min_history` 60 (inside the policy ranges 60..504 and 20..252; one trading year of history, min about a quarter), NIFTY, weekly. Windows d1 and s3 (type B, no Phase 2 window).

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

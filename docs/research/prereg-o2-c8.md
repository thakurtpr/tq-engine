# Pre-registration o2-c8: weekly broken-wing butterfly, Nifty

Type: A
Family: `c8_weekly_broken_wing_butterfly`; catalogue F12; master prereg classification C8 (no v1 counterpart).

## Hypothesis and mechanism
Index skew makes out-of-the-money puts richer than their symmetric counterparts. A put broken-wing butterfly sells two puts at a body strike below spot and buys a near put above and a far put below it, the far wing placed further than the near wing, so the structure is built for little or no net debit, collects the skew at the body and gives up protection on the far side only beyond the far wing. The hypothesis is that the skew premium collected exceeds the weekly realised downside beyond the body, net of Indian costs, slippage and 31.2% business-income tax. It is net short convexity around the body. Null: no net edge.

## Structure and strike/expiry rule
`right` put. Body: short 2 puts at a strike `body_offset_sigma` = 1 x sigma_h below spot. Near wing: long 1 put 200 points above the body (`near_width_points`). Far wing: long 1 put `far_width_ratio` x 200 points below the body. sigma_h = trailing 20-return realised vol (x sqrt(252)) scaled by sqrt(calendar days to expiry / 365), closes on or before the decision day (A2.8). NIFTY, weekly, nearest weekly expiry with `min_dte` 3. Conservative fills. The base parameters follow a stated rule (volatility-unit offset, written before any Phase-2 return exists), which is what the type-A definition requires.

## Entry and exit
Engine defaults for every engine key (none set): weekly entry, held to expiry, no take-profit, no stop. Expiry marks from the underlying final settlement. Hold H = 1 period.

## Axis and levels (chosen before any result)
Axis `far_width_ratio`, levels 1 / 1.5 / 2 (ascending in string order, A2.10), the catalogue's far-wing 1.0 / 1.5 / 2.0 x near width. Base = 1.5: the middle of the skew-harvest convention's range. 1 is a symmetric (unbroken) butterfly, the control end; 2 the widest broken wing, the largest maximum loss per lot below the far wing and the most credit retained.

## Fixed parameters
`right` put, `body_offset_sigma` 1, `near_width_points` 200, `min_dte` 3, NIFTY, weekly. Windows: p2, d1 and s3. Type A: Phase 2 (2012-01-01..2021-10-31, weekly Nifty contracts from 2019-02, about 33 months, above the 24-month minimum) is a pre-registered blind replication spending alpha_2 and is read only through the harness; if the Phase-2 periods number fewer than 24 months the master prereg makes it INCOMPLETE. Expected skip rate stated from the manifest at the first run.

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

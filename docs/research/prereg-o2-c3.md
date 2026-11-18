# Pre-registration o2-c3: weekly iron butterfly, Nifty

Type: B
Family: `c3_weekly_iron_butterfly`; catalogue F11; master prereg classification C3 (cousin of v1 V3: condor/butterfly machinery).

## Hypothesis and mechanism
Selling the at-the-money straddle collects the most premium for the bounded-risk structures; the long wings cap the loss. The hypothesis is the variance risk premium is concentrated near the money and that the premium collected exceeds the realised weekly absolute move, net of costs, slippage and 31.2% business-income tax. Net short convexity: the maximum gain is at the body, the loss is bounded by the wing width. Null: no net edge.

## Structure and strike/expiry rule
Short ATM call and put at the same strike (the strike nearest spot), long put and long call each `wing_sigma` x sigma_h from spot (wing width = spot x sigma_h x wing_sigma, rounded to whole points). sigma_h = trailing 20-return realised vol (x sqrt(252)) scaled by sqrt(calendar days to expiry / 365), closes on or before the decision day (A2.8). NIFTY, weekly, nearest weekly expiry with `min_dte` 3. Conservative fills.

## Entry and exit
Engine defaults for every engine key (none set): weekly entry, held to expiry, no take-profit, no stop. Expiry marks from the underlying final settlement. Hold H = 1 period.

## Axis and levels (chosen before any result)
Axis `wing_sigma`, levels 0.75 / 1 / 1.25 (ascending in string order, A2.10), the catalogue axis. Base = 1: wings at one sigma is the stated convention (catalogue: "ATM short, wings at 1 sigma"). 0.75 gives narrower wings (smaller max loss, cheaper to hold, lower net credit); 1.25 wider wings (more credit given up to protection less, larger max loss). Because the wing width scales with sigma_h, the number of lots at the start of the window is set by the sizing rule, not by this axis.

## Fixed parameters
`min_dte` 3, NIFTY, weekly. Windows d1 and s3 (type B, no Phase 2 window). Expected skip rate stated from the manifest at the first run.

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

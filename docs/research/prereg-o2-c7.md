# Candidate pre-registration o2-c7: weekly NIFTY ATM calendar spread (options-v2, C7)

Type: A

Registered under the frozen master prereg `docs/prereg-options-v2.md` (tag `prereg-options-v2`). Spec: `research/o2run/specs/o2-c7.json`; attest: `research/o2run/specs/o2-c7.attest.json`. Registration only: nothing has been run.

## Hypothesis and mechanism
A calendar sells the near-weekly option and buys a later expiry at the same strike. The near leg decays fastest (theta), the far leg keeps vega, so the position earns if the underlying stays near the strike and the term structure does not invert against it. The candidate tests whether the near-dated premium is rich relative to the far leg after costs. It loses to a large move and to a drop in far-leg implied volatility; the premium sign net of four-leg costs and 31.2% tax is uncertain, and the master prereg classifies calendars as net short convexity (section 3.3).

## Structure and rules
* Sell the near weekly NIFTY call (ATM, `offset_sigma` 0) and buy the far-tenor call at the same strike; `right` call. Far tenor is the axis.
* Entry: `weekday:fri` decision at the close, fill at the next open; near expiry has at least `min_dte` = 3 calendar days to run.
* Exit: engine default time exit at the next open when the near expiry is within 1 calendar day (the far leg is closed with it; no hold to settlement, no premium exits). A position across a stage cut is closed at the cut mark for the earlier stage.
* Hold H = 1 weekly period (attest `hold` 1). Attest `short_convex` TRUE: the master prereg section 3.3 lists calendars as net short convexity, so G9 applies (this is stricter than reading the calendar as non-short-convex and was chosen under the doubt rule).
* Margin: SPAN-like path with the calendar benefit modelled per the catalogue (3.4); the expiry-day benefit removal of 2025-02-10 is in the regulatory timeline.

## Axis (3 levels, categorical `far_tenor`)
`monthly` / `next_weekly` / `second_weekly`, written in strictly ascending canonical string order (A2.10), so the middle (base) is `next_weekly`. The catalogue's economic order (next weekly, second weekly, monthly) would put second_weekly in the middle, but it is not ascending as text and the loader would refuse it; the base is therefore fixed by the ordering rule. `next_weekly` is the shortest, most liquid far leg and the usual weekly-calendar practice. The neighbours are the longer, higher-vega tenors (second_weekly and monthly); this is a loader-forced departure, disclosed, not a choice made from returns.

## Fixed parameters
near_class weekly; right call; offset_sigma 0; min_dte 3; entry weekday:fri; underlying NIFTY (weeklies are NIFTY only, master 6.1; weeklies from 2019-02, Tuesday expiry from 2025-09-02 via the stage manifest); expiry weekly; windows p2, d1, s3; cost_multiplier 1; account Rs 1 crore; tax default.

## Sizing and skips
Fixed notional (A1.5): lots = max(1, floor(3.0% x capital / max loss per lot at window start)), single lot above 6.0% is INCONCLUSIVE-SIZE. Only skip: signal-day liquidity filter on both legs (far-leg liquidity may bind); above 15% INCONCLUSIVE-SAMPLE. Expected skip rate not stated (no data look); measured and reported.

## Cells and what is not tuned
Cells = 3; base cell only is the hypothesis; neighbours are G6 robustness cells. Not tuned: right, strike (ATM), min_dte, entry weekday, exit rule, underlying, windows.

## Evidence status
No result has been seen for this candidate beyond what the master prereg discloses. Type A: base written from a stated rule before any Phase-2 return; Phase 2 is a pre-registered blind replication on development data (spends alpha_2 = 0.10) and needs at least 24 months of weekly NIFTY Phase-2 periods (about 33 from 2019-02) or is INCOMPLETE. The master gates (alpha_1 = 0.139, alpha_2 = 0.10, alpha_3 = 0.10, Appendix A) apply UNCHANGED; none is restated or added.

## Possible verdicts
PROMOTED TO FORWARD PAPER; INCONCLUSIVE-SAMPLE, -TAIL (G9 or adverse regime not evaluable), -SIZE, -QUALITY; DIAGNOSTIC; KILL. A failed gate is a KILL and outranks INCONCLUSIVE. Never "passed" or "profitable".

## What would make this candidate not run
Spec refusal at load; far-leg expiry absent (no listed far weekly/monthly); data scan not clean or expiry bookings INCOMPLETE; missing index close; unsizable lot; fewer than 24 Phase-2 months (INCOMPLETE). Never a failure on its own.

## Data scan
`data_scan_clean` true for NIFTY on `docs/dq-scan-options-v2.md` (documented semantic properties and 2 missing sessions handled by A2.12 and the loader; caveats: 12 index-close holes, 18 swapped index dates keyed on file date).

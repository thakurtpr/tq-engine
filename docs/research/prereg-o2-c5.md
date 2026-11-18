# Candidate pre-registration o2-c5: monthly NIFTY iron condor (options-v2, C5)

Type: B

Registered under the frozen master prereg `docs/prereg-options-v2.md` (tag `prereg-options-v2`). Spec: `research/o2run/specs/o2-c5.json`; attest: `research/o2run/specs/o2-c5.attest.json`. Registration only: nothing has been run.

## Hypothesis and mechanism
Index options on average price implied volatility above what is later realised (the variance risk premium; Bondarenko, Coval-Shumway, Israelov-Nielsen). A defined-risk condor sold well outside a trailing-volatility band collects that premium while capping the loss. The edge, if any, must survive: Rs 1 crore fixed-capital sizing, slippage max(1% premium, Rs 1) on four legs, brokerage/STT/charges from the date-keyed rates table, and 31.2% business-income tax with only carry-forward loss relief. Mean premium after those costs is small and the left tail is fat, so the expected net sign is uncertain and a Sharpe near 1 is undetectable in this data (master prereg Appendix C).

## Structure and rules
* Iron condor on NIFTY, monthly expiry: short put and short call at `offset_sigma` x sigma from spot, long wings `width_points` = 300 further out (defined risk, margin = maximum loss).
* sigma = trailing realised vol (20 returns of closes dated on or before the decision day) x sqrt(calendar days to expiry / 365) (A2.8).
* Entry: `when_flat`, decided at a close and filled at the next open, taking the first monthly expiry with at least `min_dte` = 20 calendar days to run. One condor at a time.
* Exit: engine default time exit, next open when the expiry is within 1 calendar day (no hold to settlement, no premium exits). A position open across a stage cut is closed at the cut mark for the earlier stage (master 2).
* Hold H = 1 monthly period (attest `hold` 1). Net short convexity: attest `short_convex` true (G9 applies).

## Axis (3 levels, one numeric axis `offset_sigma`)
0.75 / 1 / 1.25 sigma, strictly ascending in canonical string order (A2.10). Base = 1 sigma, the usual 16-delta convention and the order of magnitude of options-v1's 3% monthly offset (the lineage note in the master table is the only anchor used; no v1 result was opened). 0.75 is a closer, richer-premium, riskier neighbour; 1.25 a farther, thinner one. The middle is the base because the rule is written from the stated convention, not from any return.

## Fixed parameters
structure iron_condor; width_points 300; min_dte 20; entry when_flat; underlying NIFTY (monthly BankNifty is not pooled: one underlying, one parameter vector); expiry monthly; windows p2, d1, s3; cost_multiplier 1 (the 2x stress cell is derived by the harness, G7); account Rs 1 crore (10,000,000); tax default 31.2% business-income path (same-year offset and 20%/39% remain sensitivities only).

## Sizing and skips
Fixed notional (A1.5): lots = max(1, floor(5.0% x capital / max loss per lot at window start)); a single lot risking more than 6.0% of capital is INCONCLUSIVE-SIZE. Premium and IV are never sizing inputs. The only skip is the signal-day liquidity filter; above 15% skipped cycles the verdict is INCONCLUSIVE-SAMPLE. The expected skip rate is NOT stated: it would need a look at the manifest or data, which this registration did not take; the measured rate is reported with the result.

## Cells and what is not tuned
Cells = 3 (`o2-c5` in the caps file); only the base cell is the hypothesis (`Pick::Base`, no argmax); the neighbours are robustness cells (G6). Not tuned: width, min_dte, entry rule, exit rule, structure choice (condor, not the bull put spread), underlying, windows. The 300-point width and 20-day entry were fixed here before any return was computed.

## Evidence status
No result has been seen for this candidate beyond what the master prereg discloses (section 8 and the options-v1 lineage, K=30, type B by the doubt rule). Phase 2 is a replication/consistency check that spends no alpha for type B. The master prereg's gates (section 3, Appendix A hurdles, alpha_1 = 0.0139, alpha_3 = 0.10) apply UNCHANGED; none is restated or added here. The master power caveat applies: monthly type B power at Sharpe 2 is about 33% (LOW) and 25% (HIGH).

## Possible verdicts
PROMOTED TO FORWARD PAPER (only positive outcome; never "passed" or "profitable"); INCONCLUSIVE-SAMPLE, -TAIL, -SIZE, -QUALITY; DIAGNOSTIC (a stage has no contracts); KILL. A failed gate is a KILL and outranks any INCONCLUSIVE flag.

## What would make this candidate not run
A spec refusal at load; the data scan not clean or INCOMPLETE expiry bookings (settle-price rule, A1.7); a missing index close series for NIFTY; a lot that cannot be sized (INCONCLUSIVE-SIZE); no monthly contracts in a stage (DIAGNOSTIC). Never a failure on its own; NOT-RUN consumes no cell.

## Data scan
`data_scan_clean` true on `docs/dq-scan-options-v2.md`: no row defect in NIFTY outside the documented semantic properties (E01/E02/E03, R04 settle rows on which marks use close, 2 missing F&O sessions skipped) handled by A2.12 and the loader; caveats: 12 NIFTY index-close holes and 18 swapped index dates (keyed on file date). Declared, not hidden.

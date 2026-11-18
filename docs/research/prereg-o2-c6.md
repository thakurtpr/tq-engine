# Candidate pre-registration o2-c6: monthly NIFTY long straddle with take-profit (options-v2, C6)

Type: B

Registered under the frozen master prereg `docs/prereg-options-v2.md` (tag `prereg-options-v2`). Spec: `research/o2run/specs/o2-c6.json`; attest: `research/o2run/specs/o2-c6.attest.json`. Registration only: nothing has been run.

## Hypothesis and mechanism, and the expected sign
A long ATM straddle pays if the realised move over the holding period exceeds what the premium implied, and is long convexity (gains in crashes and rallies, bounded loss). **The expected sign of the premium is NEGATIVE:** index implied volatility is on average above realised, so a buyer pays the variance risk premium and also pays two sets of slippage and charges. For the structure to earn net of costs and 31.2% tax, realised volatility would have to exceed implied by more than the roughly round-trip cost on enough cycles, or the take-profit would have to harvest convex bursts that the premium under-prices, in the window tested. This is registered as a long-volatility counterweight in the catalogue and as a lineage cousin of options-v1's straddles; it is expected to fail and a KILL is a normal outcome.

## Structure and rules
* Long NIFTY straddle: buy the ATM call and put, same strike, monthly expiry (`structure` straddle, `strangle_offset_sigma` 0, `expiry_class` monthly).
* Entry: `when_flat`, decided at a close, filled at the next open, first monthly expiry with at least `min_dte` = 20 calendar days to run. One straddle at a time.
* Exits: take-profit when open P&L >= `profit_target` x entry net premium (the axis); otherwise the engine default time exit at the next open when expiry is within 1 calendar day. No stop loss (loss bounded by premium).
* Hold H = 1 monthly period (attest `hold` 1). Not short convexity: attest `short_convex` false (G9 does not apply by the master definition).

## Axis (3 levels, one numeric axis `profit_target`)
0.4 / 0.5 / 0.6 of premium paid, strictly ascending in canonical string order (A2.10). Base 0.5 = take half the premium, the catalogue's base (v1 lineage anchor only). 0.4 banks sooner, 0.6 lets winners run. The middle is the base because 50% is the conventional round number, fixed before any return was computed.

## Fixed parameters
underlying NIFTY (monthly BankNifty not pooled); expiry monthly; windows p2, d1, s3; cost_multiplier 1 (2x stress derived by the harness); account Rs 1 crore; tax default business-income path; min_dte 20; entry when_flat. A weekly variant is NOT registered (catalogue row says weekly+monthly; this candidate is the monthly one, matching the lineage).

## Sizing and skips
Fixed notional (A1.5): lots = max(1, floor(5.0% x capital / max loss per lot at window start)), max loss = premium; one lot above 6.0% of capital is INCONCLUSIVE-SIZE. Only skip: signal-day liquidity filter, above 15% INCONCLUSIVE-SAMPLE. A long straddle's volatility may fall under 6% (INCONCLUSIVE-SIZE, not a kill). Expected skip rate not stated (no look at data taken); the measured rate is reported.

## Cells and what is not tuned
Cells = 3; base cell only is the hypothesis, neighbours are G6 robustness cells. Not tuned: min_dte, entry rule, straddle vs strangle (straddle fixed), underlying, windows, no stop loss.

## Evidence status
No result has been seen for this candidate beyond what the master prereg discloses (options-v1 lineage, type B by the doubt rule; no v1 result document was opened). Phase 2 is a replication check that spends no alpha for type B. The master gates (alpha_1 = 0.0139, alpha_3 = 0.10, Appendix A) apply UNCHANGED; none is restated or added. Monthly type B power at Sharpe 2 is about 33% / 25%; a Sharpe near 1 is undetectable.

## Possible verdicts
PROMOTED TO FORWARD PAPER; INCONCLUSIVE-SAMPLE, -SIZE, -QUALITY (and -TAIL only if the master rule triggers); DIAGNOSTIC; KILL. A failed gate is a KILL and outranks INCONCLUSIVE. Never "passed" or "profitable".

## What would make this candidate not run
Spec refusal at load; data scan not clean or expiry bookings INCOMPLETE; missing NIFTY index close; a lot that cannot be sized; no monthly contracts in a stage (DIAGNOSTIC). Never a failure on its own.

## Data scan
`data_scan_clean` true for NIFTY on `docs/dq-scan-options-v2.md`: the NIFTY findings are the documented semantic properties (E01/E02/E03, R04) and 2 missing F&O sessions, handled by A2.12 and the loader; caveats: 12 NIFTY index-close holes, 18 swapped index dates (file-date keyed).

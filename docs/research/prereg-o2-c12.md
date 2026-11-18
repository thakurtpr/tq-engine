# Candidate pre-registration o2-c12: daily delta-hedged NIFTY straddle (options-v2, C12)

Type: A

Registered under the frozen master prereg `docs/prereg-options-v2.md` (tag `prereg-options-v2`). Spec: `research/o2run/specs/o2-c12.json`; attest: `research/o2run/specs/o2-c12.attest.json`. Registration only: nothing has been run.

## Hypothesis and mechanism, and the expected sign
Buying an ATM straddle and hedging its delta daily with index futures isolates the volatility risk premium: P&L is roughly gamma x (realised variance - implied variance) less theta, with direction removed. **The expected sign is NEGATIVE:** implied volatility is on average above realised, so the buyer pays the premium, plus option costs, daily futures costs and 31.2% tax. For a net edge realised volatility would have to exceed implied by more than all of those, in the window tested. It is a long-gamma, not short-convex, book; a KILL is a normal outcome.

## Structure and rules
* Long NIFTY ATM straddle, weekly-class expiry, `tenor` fixed at `two_week` (weekly class, `min_dte` 8; A2.8); the engine rebalances a NIFTY futures hedge daily (`hedge_frequency` daily).
* Entry: `when_flat`, decision at a close, fill at the next open; one position at a time, flat to flat.
* Exit: time exit only, the next open when the nearest option expiry is within `exit_before_expiry_days` calendar days (the axis). Premium-based exits (`profit_target`, `stop_loss`) are FORBIDDEN for C12 (A2.9) and are absent.
* Hold H = 2 weekly periods (entry at least 8 days from expiry, exit 1-3 days before it: one to two weeks). Attest `short_convex` false, so G9 does not apply by the master definition.

## Axis (3 levels, numeric `exit_before_expiry_days`)
1 / 2 / 3 calendar days, ascending as text. Base 2. **Loader-forced departure:** the catalogue axis is tenor (weekly / two_week / monthly), but the spec loader refuses a spec whose cells span two expiry classes (monthly vs weekly) and only two tenors share the weekly class, so a 3-level tenor axis cannot load. The tenor is therefore fixed at its middle value `two_week` and the axis is the time-exit rule, which is allowed under A2.9. The exit axis was fixed from the structure (gamma and pin risk concentrate in the last days) and before any return; 1 is the engine default, 3 the most conservative, 2 the base because it avoids both the last-day pin and a truncated hold. The weekly and monthly tenors are not run (no cell).

## Fixed parameters
tenor two_week; hedge_frequency daily; entry when_flat; underlying NIFTY; expiry weekly (weeklies from 2019-02, Tuesday expiry from 2025-09-02 via the stage manifest); windows p2, d1, s3; cost_multiplier 1; account Rs 1 crore; tax default.

## Sizing, hedge margin and utilisation
Fixed notional (A1.5): lots = max(1, floor(3.0% x capital / max loss per lot at window start)), max loss = premium; one lot above 6.0% is INCONCLUSIVE-SIZE. **Disclosed assumption (A2.8): the margin of the hedge futures is NOT modelled.** Margin-utilisation sensitivity: the result must be read together with the statement that a daily-hedged futures leg carries index-futures margin (roughly a further 10-15% of its notional in 2026, indicative and unverified) that is not charged here; if that margin consumed the 60% margin cap or required capital above Rs 1 crore, the verdict would be read as an upper bound on net return per rupee of capital. The sensitivity is reported, never gating. Only skip: signal-day liquidity filter; above 15% INCONCLUSIVE-SAMPLE. Expected skip rate not stated (no data look).

## Cells and what is not tuned
Cells = 3; base cell only is the hypothesis; neighbours are G6 cells. Not tuned: tenor, hedge frequency, entry rule, strike (ATM), underlying, windows.

## Evidence status
No result has been seen for this candidate beyond what the master prereg discloses. Type A: Phase 2 is a pre-registered blind replication on development data (alpha_2 = 0.10), needing at least 24 months of weekly-NIFTY Phase-2 periods (about 33 from 2019-02) or INCOMPLETE. The master gates (alpha_1 = 0.139, alpha_2 = 0.10, alpha_3 = 0.10, Appendix A) apply UNCHANGED; none is restated or added.

## Possible verdicts
PROMOTED TO FORWARD PAPER; INCONCLUSIVE-SAMPLE, -SIZE, -QUALITY (-TAIL if the adverse regime is not evaluable); DIAGNOSTIC; KILL. A failed gate is a KILL and outranks INCONCLUSIVE. Never "passed" or "profitable".

## What would make this candidate not run
Spec refusal at load; no NIFTY futures rows for the hedge; data scan not clean or expiry bookings INCOMPLETE; missing index close; unsizable lot; fewer than 24 Phase-2 months. Never a failure on its own.

## Data scan
`data_scan_clean` true for NIFTY on `docs/dq-scan-options-v2.md` (documented semantic properties and 2 missing sessions handled by A2.12 and the loader; caveats: 12 index-close holes, 18 swapped index dates keyed on file date). NIFTY futures rows are in the same scan (index futures zero-volume 0-47% by year; marks are settle-based).

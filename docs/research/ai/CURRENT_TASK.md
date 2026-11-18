# Current task
Status: active
ID: T-002
Goal: Find whether any Indian NSE options strategy (index and liquid stock options, free EOD data) shows a detectable net-of-cost-and-tax edge over buy-and-hold Nifty ETF and the 50/50 ETF+BTC blend, or state plainly what the search could and could not detect (default stays Nifty ETF + BTC).

## Acceptance criteria
- [ ] Synthetic end-to-end dry run: no gate structurally unpassable; fail closed on bad data
- [ ] Evaluator calibrated BEFORE any strategy: null and planted-edge controls on the options calendar, power at Sharpe 1/2/3 reported; gates and K written down before testing
- [ ] Master prereg (options-v2) + every candidate pre-registered with running trial counter (options-v1 K=30 lineage included), deflated Sharpe on true N + SPA/White
- [ ] Data: free NSE bhavcopy (most recent 5 years then 2012-2021), lot-size history, bad-print scan, sources/biases recorded
- [ ] Each strategy family: runnable or 'not run: reason' (data/engine), never a failure
- [ ] Final report: table of every strategy x underlying x window, calibration power table, not-run list with the paid data that would unlock them, plain verdict

## In scope
- All main options strategy types on Nifty 50, Bank Nifty, FinNifty, Midcap Nifty, liquid stock options; weekly and monthly expiries; costs/tax from the date-keyed rates table; margin per structure

## Out of scope (non-goals)
- Anything live; engine-code pushes; paying or signing up for data; lowering a gate after seeing results; naked selling as a default

## Allowed paths
- Unrestricted (scope.allow not created). Hard rules in hardening/STATUS.md apply.

## Plan
- [ ] 1. Wave 0 (parallel): data pull + bad-print scan; capability audit + strategy catalogue + margin model; feasibility-first protocol + dry run + calibration
- [ ] 2. Freeze protocol (skeptic review), then strategy agents per family (separate worktrees, one ledger writer)
- [ ] 3. Phase 1 (5 years) then Phase 2 (2012-2021); sealed final slice only for development-GO candidates
- [ ] 4. Final report

## Constraints and references
- Related decisions: D-006. Prior: docs/research-v2-final-report.md, prereg-options-v1.md, result-options-v1-dev.md.

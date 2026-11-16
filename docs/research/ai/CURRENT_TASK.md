# Current task
Status: active
ID: T-001
Goal: Find a strategy x instrument that passes every pre-registered gate after costs and tax on the longest free history, or conclude nothing passes (default stays ETF + BTC).

## Acceptance criteria
- [ ] Master protocol (gates, splits, trial ledger, thresholds) committed BEFORE any strategy test
- [ ] Data inventory: every source, date range, known biases (survivorship, splits, gaps), unverified flags
- [ ] Each hypothesis + grid pre-registered and counted in the ledger before it runs
- [ ] Report table: every strategy x instrument (trials, IS/OOS, gate pass/fail) + data coverage
- [ ] Verdict: "candidate X on Y passed all gates, evidence" or "nothing passed, default stays ETF + BTC"
- [ ] cargo test --workspace and clippy -D warnings green on hardening/integration

## In scope
- Instruments: Nifty ETF, Nifty/BankNifty, liquid Indian large caps, BTC/ETH and other liquid crypto pairs, other liquid pairs with free long history
- Re-test killed strategies on longest data; add trend following, mean reversion, cross-sectional momentum, vol targeting, pairs/stat-arb
- Parallel agents in worktrees under wt/, merged into hardening/integration

## Out of scope (non-goals)
- Anything live, any real broker/orders/credentials; pushing engine code; lowering a gate; touching a holdout more than once per candidate

## Allowed paths
- Unrestricted for now: docs/ai/scope.allow intentionally not created (it would block worktrees and memory). Hard rules in hardening/STATUS.md apply.

## Plan
- [ ] 1. Wave 0: data inventory + master protocol (parallel)
- [ ] 2. Review protocol; freeze thresholds
- [ ] 3. Wave 1: per-strategy/instrument agents (pre-register, then dev run)
- [ ] 4. Holdout only for dev-GO candidates; final report

## Constraints and references
- Related decisions: D-002 (supersedes D-001). Frozen gates: docs/validation-protocol.md v3.

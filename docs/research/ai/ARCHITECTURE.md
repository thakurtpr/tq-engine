# Architecture
Updated: 2026-11-16

## Overview
Rust workspace `engine/` (tq-engine): tq-core (types, Clock), tq-risk (RiskGate mints ApprovedOrder, persisted KillSwitch), tq-costs (India costs + date-keyed rates and TaxModel), tq-strategy/strategies/indicators, tq-data, tq-broker (paper only, order FSM, journal), tq-backtest, tq-cli, tq-feeds, tq-live, plus bias, calendar, corpactions, pipeline, council, analogs, councileval, options crates. Work lives in worktrees under wt/ merged into hardening/integration.

## Key directories
- engine/: the Rust workspace (git repo; main is never touched).
- wt/: git worktrees, one per branch; wt/integration is the integration branch.
- hardening/: STATUS.md (hard rules + log), MORNING.md (owner summary), ledger anchor.
- docs/: research docs, preregistrations, results, tax and cost notes.
- paper/: benchmark.py and journal.csv (Nifty ETF + BTC paper benchmark).
- shared-target/: shared CARGO_TARGET_DIR. data-raw/: large raw data, outside git.

## Data flow
Bars -> Strategy signals -> Insight claim -> Portfolio -> RiskGate (only mint of ApprovedOrder) -> paper broker -> journal. LLM/council output can only veto or shrink, never place orders.

## External dependencies and environments
- Public price feeds only (Yahoo chart, Binance public). No broker, no credentials.

## Invariants (must never break)
- Brokers accept only ApprovedOrder from RiskGate; no real orders.
- No look-ahead: orders fill at the next bar's open.
- Trial ledger counts every sweep; thresholds frozen after preregistration.

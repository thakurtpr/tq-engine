# State
Updated: 2026-11-16 20:31 (by setup)

## Right now
research-v2 SEARCH EXHAUSTED: all 14 runnable candidates KILLED on the development window; nothing passed; default stays Nifty ETF + BTC. Final report: docs/research-v2-final-report.md (wt/integration 09b7371). Awaiting owner decisions (D-005 etc.).

## Done (latest 10)
- Council-v1, options-v1 (dev) evaluated: nothing passed any gate; all merged into hardening/integration (1,199 tests, clippy clean).
- Shared date-keyed rates table and tax model merged; after-tax momentum re-run written.
- Paper benchmark (paper/journal.csv) recording; last run 2026-11-16: NIFTYBEES +5.30%, BTCUSDT -14.78% on placeholder notionals.

## In progress
- Waiting on owner for the real Nifty/BTC split numbers and for the current task and allowed paths.

## Next (ordered)
1. Harness (r2gov + r2stats) built and independently reviewed; then wave 1 strategy agents (prereg-r2-<id> per candidate, K opened before run).
2. Run the benchmark weekly or on request; report on the real split once supplied.

## Blockers and open questions
- Owner's real split arrived as [X% Nifty / Y% BTC] with numbers unfilled.

## Failed approaches (do not retry)
- Free end-of-day data produced no edge: intraday, momentum rotation, basis carry, council, defined-risk options all killed or inconclusive. Do not tune or revisit.

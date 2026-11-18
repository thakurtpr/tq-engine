# Decision log

Append-only. Newest at the bottom. Status is one of: Proposed, Accepted, Superseded by D-xxx.

| ID | Date | Title | Status |
|---|---|---|---|
| D-000 | 2026-11-16 | Adopt docs/ai context system | Accepted |
| D-001 | 2026-11-16 | Stop new strategy research; plan is Nifty ETF + BTC | Superseded by D-002 |
| D-002 | 2026-11-16 | Reopen research: full search on maximum history | Accepted |
| D-003 | 2026-11-16 | Seen data is development only; trade pooling; promotion wording | Accepted |
| D-004 | 2026-11-17 | Crypto Sharpe annualisation stays sqrt(252); ADANIENT excluded; sealed-key backup | Accepted |
| D-005 | 2026-11-17 | Frozen gates cannot distinguish 'no edge' from 'edge below Sharpe 3-4'; 24% drawdown limit below the ETF's own | Proposed (owner decision) |
| D-006 | 2026-11-17 | New task: full Indian OPTIONS strategy search (options-v2) with a feasibility-first protocol | Accepted (owner) |

## D-000: Adopt docs/ai context system
- Date: 2026-11-16
- Status: Accepted
- Context: Claude Code sessions lose context across sessions and compaction.
- Decision: Keep scope in CURRENT_TASK.md, progress in STATE.md, rationale here; enforce with hooks.
- Consequences: Files must stay short and be updated at checkpoints.
- Alternatives rejected: relying on chat history alone.

## D-001: Stop new strategy research; plan is Nifty ETF + BTC
- Date: 2026-11-16
- Status: Superseded by D-002
- Context: Every pre-registered candidate (intraday, momentum rotation, basis carry, council, defined-risk options) was killed or inconclusive on free data; earnings drift is untestable without paid data.
- Decision: Owner chose option 3: no new research, hold Nifty ETF + BTC, benchmark weekly or on request, no maintenance cron. Do not tune or revisit killed strategies.
- Consequences: Research reopens only if the owner says they are buying point-in-time earnings data for Indian mid/small caps.
- Alternatives rejected: paid PIT earnings data (not bought); weekly-expiry or intraday options (need paid data and a new prereg).

## D-002: Reopen research: full search on maximum history
- Date: 2026-11-16
- Status: Accepted
- Context: Owner wants a full search for profitable strategy/pair combinations on the maximum available history (supersedes D-001). Owner message conflicts on pushing ("do not push to any remote" vs "push finished steps via docs/checkpoint.sh per memory rule").
- Decision: Reopen. Goal: find which strategy on which instrument/pair is profitable after costs and tax, or conclude nothing passes and the default stays ETF + BTC. Paper/backtest only. Every hypothesis and grid is pre-registered before testing; one running trial counter with DSR on the true total N; train / validation / single-touch holdout; walk-forward; thresholds frozen and published before any test; failed candidates logged as killed. The frozen v3 gates (docs/validation-protocol.md) remain the bar for any profit claim; the owner's extra gates are added, never substituted. Push policy resolution: no engine code is ever pushed; only research docs go via docs/checkpoint.sh (the owner's standing memory rule), no secrets, no large files.
- Consequences: Larger multiple-testing burden (N grows); expect INCONCLUSIVE/KILL again unless a real edge exists; usage-limit resumes continue without asking.
- Alternatives rejected: keep D-001 (owner reversed it); loosen gates (non-negotiable).

## D-003: Seen data is development only; pooled trade counts; promotion wording
- Date: 2026-11-16
- Status: Accepted
- Context: Options 2012-2021, crypto 2024+ and anything the paper benchmark touched have already been seen; trade-count gates (200 dev / 100 OOS) are infeasible for slow single-instrument strategies.
- Decision: (1) Already-seen data is NOT holdout; it is development data only. Where too little clean holdout remains for an asset, its true holdout is forward paper trading from the tag date. A candidate passing all development gates is PROMOTED TO FORWARD PAPER, never "PASSED". (2) No threshold is lowered. Trades may be pooled across instruments within ONE pre-registered strategy family (same rule, same parameters, no per-instrument tuning) and the pooled trades counted (with an overlap/correlation haircut defined in the protocol). A single-instrument slow strategy short of the count is INCONCLUSIVE with the shortfall stated: not killed, not passed. (3) Trial count counts CELLS, not ledger rows; the basis-carry lineage uses a documented UPPER bound (undercounting is the dangerous direction). (4) The ledger marker hash is refreshed after review edits and before the tag.
- Consequences: Cleaner honesty about contamination; "passed" requires forward paper evidence; N is larger so DSR is harder.
- Alternatives rejected: lowering trade-count gates; treating seen windows as holdout.

## D-004: Crypto annualisation sqrt(252) kept; ADANIENT excluded; sealed-key backup
- Date: 2026-11-17
- Status: Accepted (owner decision)
- Context: r2fix finding F-L10: Sharpe annualises with sqrt(252) for every class; addendum 1 section 4 says 365 for crypto. ADANIENT adjclose is negative on 47 development rows; the sealed store is AES-256-GCM with a key file outside the repo.
- Decision: (1) Keep the STRICTER sqrt(252) for crypto in every gate; any move to 365 would loosen a gate and needs a new owner decision recorded here (never silently). (2) Exclude ADANIENT from the data step (--exclude ADANIENT); it was not a NIFTY 50 member before 2022 so IN-EQ development is unaffected. (3) Back up the sealed-store key to a second location outside the repo, never pushed anywhere; STATUS.md records the key's LOCATION only, never the key.
- Consequences: Crypto candidates face a slightly higher bar (lower annualised Sharpe); documented; key loss would make the sealed rows unrecoverable, hence the backup.
- Alternatives rejected: sqrt(365) (looser); dropping the key backup.

## D-005: Gate feasibility findings (owner decision needed; nothing changed)
- Date: 2026-11-17
- Status: Proposed (owner decision). No gate, threshold, cap or boundary has been changed by anyone.
- Context: Evaluator calibration (null and planted-edge controls on the real development calendar; seeds 20261200..; 200-500 seeds per cell; 400 bootstrap draws; plateau assumption 0.9 per grid step): (a) at the real trial count (N_true about 540-600) a large-cap 9-cell candidate reaches a development GO only at planted net Sharpe 3-4, and then in about 3% of seeds (power at Sharpe 1 / 2 / 3: 0% / 0% / 3%); 27-cell families and all crypto candidates: 0% / 0% / 0% even at Sharpe 6. Zero-edge controls: 0 false passes in 18,000 evaluations. Mechanism: the neighbouring-parameter gates (OG5, V7) require a wide profitable plateau, and the deflated Sharpe counts that plateau as variance. (b) The 24% drawdown limit (V9 p95 MDD, OG3) is below the Nifty ETF's own worst drawdown in the development window (34.3%), so the default ETF plan itself fails these gates. (c) r2-xsmom-lc (closest large-cap candidate) is leveraged Nifty beta 1.16 with alpha -1.4%/yr: no alpha.
- Decision: none yet. Any change to the gates, the deflation set or the drawdown limit is an owner decision recorded here; the harness and the engineers do not change it. Until then the search runs as frozen and a verdict of 'nothing passed' must be read as 'no edge of Sharpe above about 3-4 on this data', NOT as 'no edge exists'.
- Consequences: continuing costs trial cells (each opened sweep adds to the true trial count for that strategy lineage, which would also count in any redesigned protocol); pausing or redesigning forgoes information from the remaining candidates.
- Alternatives (owner to choose): (1) continue as frozen (default, owner's standing instruction: keep going until exhausted); (2) pause and redesign a feasible protocol on a fresh family: fewer cells per candidate, plateau excluded from the deflation set or deflation on candidates not cells, drawdown limit relative to the benchmark, forward paper as primary evidence; (3) hold the 7 crypto candidates (they cannot pass per calibration) and keep them as a forward-paper watch list.

## D-006: Options-v2: all main Indian options strategy types, feasibility-first protocol
- Date: 2026-11-17
- Status: Accepted (owner directive; supersedes the research-v2 closing state, which stays published)
- Context: research-v2 ended with all 14 runnable candidates killed and a calibration showing the frozen gates were near-infeasible (power 0/0/3% at planted Sharpe 1/2/3 for 9-cell candidates). Owner asks for a search of ALL main options strategy types on Indian NSE index and stock options (paper/backtest only), most recent 5 years first (about 2021-11 to 2026-10), then 2012-2021 as an out-of-time check; free data only (NSE F&O end-of-day bhavcopy, index closes, India VIX; no intraday option prices or historical IV: derive IV from EOD where possible, mark data-hungry strategies 'not run: data unavailable' and say what paid data would unlock them).
- Decision: (1) BEFORE any strategy runs: dry-run the whole pipeline on synthetic data (no gate structurally unpassable: ledger completeness, deflation sets, benchmark inputs, tax series) and calibrate the evaluator with null (zero edge, many seeds) and planted-edge (net Sharpe 0.5-3) controls on the real calendar; report power at Sharpe 1/2/3; design the gates and the number of variants so a modest real edge CAN be detected; write thresholds and K down BEFORE testing; never lower a gate after seeing results. The owner's directive authorises a NEW pre-registered protocol for options-v2 (own gates, own ledger family, true total N including the options-v1 K=30 lineage); research-v2's frozen v3 gates stay untouched for research-v2. (2) Windows: Phase 1 = most recent 5 years: train, validation and a SEALED final slice touched at most once per candidate and only after it passes every development gate; Phase 2 = 2012-2021 as out-of-time check, DEVELOPMENT data only (already seen by options-v1). NOTE: the Phase-1 window is the options-v1 holdout window (2022-01 to 2026-10, never opened by the killed options-v1 candidates apart from five burned sessions 2025-10-27..31): it is consumed as development data by this decision; the TRUE clean holdout is forward paper from today. (3) Pre-register every hypothesis and grid with a running trial counter counting every variant including options-v1; deflated Sharpe on the true total N plus White's reality check or SPA. (4) Private copy of the tq binary for every sweep. (5) Scan every series for bad prints/gaps/non-positive values; declare repairs in a dated addendum. (6) A candidate passing every development gate is PROMOTED TO FORWARD PAPER, never 'passed'. (7) Push policy unchanged: no engine code; research docs only via docs/checkpoint.sh (owner standing rule). (8) Naked selling only as a labelled stress test with a strict margin model and a hard loss cap, never a default. (9) Report only on: a promotion, exhaustion, or a blocker needing the owner.
- Consequences: large engineering (weekly expiries, calendar/diagonal, ratio, butterflies, hedging) and data pull (about 5 years of bhavcopy); expect INCONCLUSIVE/KILL again unless a real edge exists, but the protocol must be able to see a modest one.
- Alternatives rejected: reusing the frozen research-v2 gates unchanged (infeasible); testing only monthly EOD structures again (already killed).

<!-- Template for new entries (use /decide):
## D-NNN: <title>
- Date:
- Status: Proposed | Accepted
- Context: <forces, constraints>
- Decision: <what we chose>
- Consequences: <good and bad>
- Alternatives rejected: <option: why not>
-->

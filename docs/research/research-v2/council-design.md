# Pre-trade council: design (v2, design only, no implementation)

Date: 2026-11-06 (v2 supersedes the 3-role v1 in commit 8060be0 after the owner's revision). Branch `hardening/council`. Paper and backtest only. Nothing here is evidence of profit: the council is a
selectivity filter whose value is an open, pre-registered hypothesis (`docs/prereg-council-v1.md`).

## 1. Purpose and stance

A trade is allowed only when every voting role passes. The default is **abstain**: a missing input, cold history, malformed message, failed log write or a role timeout all mean "no trade". The council can only **veto or shrink**; it can
never enlarge a position, create a trade, change a side or touch an exit. LLMs are optional advisers that can only lower a deterministic verdict; every voter has a deterministic implementation so the council is backtestable and byte-reproducible.

**The number of voters must be earned, not assumed.** Correlated voters add no safety, only turnover cost and a larger trial count. The design therefore ships a pre-registered ablation (prereg section 7): each voter is removed one at a time and run alone, pairwise vote
correlation is recorded, and a voter stays only if removing it worsens net out-of-sample Sharpe or drawdown by a pre-registered margin. The surviving minimal council is the one that proceeds.

Honest framing: selectivity is not edge. The base strategies in this repo have passed no gate (`docs/rerun-after-hardening.md`); a council that trades less has less of nothing unless the pre-registered tests show otherwise.

## 2. Role table (trader-* roles mapped to repo components)

| # | Role (skill) | Class | Deterministic implementation | LLM advisory (veto/shrink only) | Repo hook |
| --- | --- | --- | --- | --- | --- |
| 1 | **Scanner** (`trader-scanner`) | voter, candidate generator | emits a candidate only when an in-repo strategy fires: Donchian breakout, trend-pullback, momentum-rotation, crypto momentum burst. Its vote is PASS iff a candidate exists; **degenerate** (always PASS), so it is outside the ablation and the correlation table | none | `tq-strategies`, `tq-live::MomentumBurst`, `tq-alpha-api` Insight |
| 2 | **Market/News** (`trader-news`) | voter | regime and sentiment flags R1 to R6 (section 3.2) | headline-event veto (R6): may veto or halve on a named event; deterministic fallback = no flag | `round4/sentiment-hedging.md`, `data/index`, `data/funding` |
| 3 | **Entry Timer** (`trader-entry`) | voter | entry-bar condition evaluated at the signal-bar close; the only legal fill is the **next bar open** (section 3.3) | none | engine fill model |
| 4 | **Exit Planner** (`trader-exit`) | voter | fixes stop and target **before entry** and computes net R:R; vetoes if R:R is below the gate (section 3.4) | none | strategy stop; shadow-trade MFE |
| 5 | **Time Machine** (`trader-timemachine`) | voter | historical analogs (every past firing of the same setup, point-in-time) plus a seeded Monte Carlo of outcomes; PASS iff lower confidence bound of net expectancy is above zero after costs and tax, and the win-rate gate holds (section 3.5) | may veto on a structural break the history cannot see; cannot create a PASS | honesty layer bootstrap, `tq-costs` |
| 6 | **Risk** (`trader-risk`) | voter | `RiskGate::check_ctx` dry run plus `KillSwitch` state, locks, collar, ADV, rate limiter | none | `tq-risk`, `tq-pipeline` |
| 7 | **Budget** (`trader-budget`) | voter | position sizing to capital and max loss per trade via `size_for_risk`, volatility-scaled shrink, and the trade-frequency cap (section 3.6) | none | `tq_risk::size_for_risk`, `RiskLimits` |
| 8 | **Earnings Analyst** (`trader-analyst`) | veto-only **when data exists** | **NOT-AVAILABLE today**: no point-in-time earnings dataset can be sourced legally (`docs/data-feasibility-sue.md`). Verdict is `NOT_AVAILABLE`: logged on every candidate, excluded from the unanimity count, **never counted as a PASS** and never as a veto. If data ever exists it may only veto; adding it is a new registration | none | none |
| 9 | **Alerts** (`trader-alerts`) | non-voting | notifications only, read-only consumer of the decision log; no feedback into any verdict | formats messages only | none |
| 10 | **Broker** (`trader-broker`) | non-voting | **paper broker only** (`tq-broker` lifecycle `PaperBroker`); executes only candidates whose council outcome is PASS or SHRINK **and** that hold a `RiskGate` `ApprovedOrder`. No real broker or credential type is constructible; Kite order tools stay off (STATUS rule 1) | none | `tq-broker` |
| - | **Pelosi tracker** (`trader-pelosi`) | **excluded** | not used. Reason recorded: it follows US Congress periodic transaction disclosures (STOCK Act filings on US equities, filed up to 45 days late); not applicable to NSE or crypto, disclosures are lagged and not point-in-time, and no India analogue exists in the repo | none | none |

Voting set: roles 1 to 7. All voters run on the same point-in-time snapshot (`as_of`, data with `available_at <= as_of`), share no mutable state, and do not see each other's votes (independence). Declared shared inputs (so correlation is expected and measured): voters 4 and 5 read the same shadow-trade history; voter 2's flag count is an input of voter 5's bucket.

## 3. Voter specifications and the four metric gates

### 3.1 Metric gates (numbers frozen in the pre-registration; each belongs to one voter)

| Gate | Value | Owner voter |
| --- | --- | --- |
| **Minimum R:R, net** | `(target_dist - K) / (stop_dist + K) >= 2.0` where `K` = round-trip costs plus slippage per share | Exit Planner |
| **Position sizing rule** | risk fraction per trade = `max_risk_per_trade_pct` of `RiskLimits` (default 0.5%) x volatility scale `v`, `v = clamp(median ATR% over 252d / current ATR%, 0.25, 1.0)`; quantity = `size_for_risk(equity, risk_pct*v, entry, stop)`; notional cap 20%, max loss per trade <= 0.5% of equity. `v` can only shrink | Budget |
| **Win rate** | calibrated historical win rate `p_hat` (net after costs and tax above 0) over analogs; Wilson lower bound (one-sided 90%, z = 1.2816) must exceed required breakeven `p* = (1 + k) / (1 + RR)` with `k = K / stop_dist` (breakeven `1/(1+RR)` plus cost drag) | Time Machine |
| **Trade frequency** | at most **1 new entry per day and 3 per week** (equity), **2 per day and 6 per week** (crypto), deterministic priority by Time Machine confidence then symbol; **minimum evidence**: bucket analogs `n >= 30` and strategy-wide resolved trades `n >= 60`, otherwise ABSTAIN | Budget (cap), Time Machine (minimum) |

Expected consequence, stated up front: mean-reversion setups with small targets (trend-pullback) are likely to fail the 2:1 R:R gate and abstain; this is a pre-registered property, not a bug to tune away.

### 3.2 Market/News

Output multiplier in {0, 0.5, 1}; any missing input is ABSTAIN. Flags, computed at the close of day t, applied to the entry at t+1:

| Flag | Market | Rule | Data today |
| --- | --- | --- | --- |
| F_VOL | India | R1 proxy: 20-day realised vol of `^NSEI` above its trailing-252d P80 gives a flag, above P95 gives 0. **India VIX is not legally sourceable** (`docs/data-feasibility-sue.md`); a real VIX series would be a new counted trial | `data/index/NSEI.csv` |
| F_TREND | India | R2: Nifty < 200-day mean AND breadth < 30%; breadth = share of PIT NIFTY 50 members above own 50-day mean (Nifty 500 unavailable) | NSEI, `data/daily-pit`, `data-ref` |
| F_DD | India | Nifty drawdown from 252-day high above 10% | NSEI |
| F_VOL | Crypto | R4: 20-day vol above 1.5x its 1-year median | `data/spot-1h` |
| F_CROWD | Crypto | R3 funding-only: mean of last 3 prints above 0.05% per 8h or above trailing-90d P95 (OI clause dropped: no history) | `data/funding` |
| F_DD | Crypto | BTC drawdown from 90-day high above 20% | `data/spot-1h` |
| F_BOOK | Both | R5: own-book drawdown above 8% gives a flag, above 12% gives 0 with 24h cooldown | simulation state |
| F_EVENT | Both | R6 headline-event veto: **LLM advisory only**; deterministic fallback = no flag; `NOT-TESTED` in all registered runs | none |

0 flags = 1.0, 1 flag = 0.5, 2 or more = 0 (P95 volatility tier alone = 0).

### 3.3 Entry Timer

Deterministic, evaluated on the signal bar's close; the fill is the **next bar open only** (no intrabar or same-bar fills). PASS iff all hold: (a) the next session exists and is tradable (NSE calendar, fail-closed outside coverage); (b) signal-bar volume at least 0.5x its 20-bar median
and order size at most 1% of 20-day ADV; (c) not extended: `|close - SMA(10)| <= 2.0 x ATR(14)`. Otherwise VETO with the failing sub-condition. The Entry Timer never picks a different price or time; it can only say "not now". A vetoed candidate is not retried automatically.

### 3.4 Exit Planner

Before entry it fixes and logs a plan (`council.plan.v1`): **stop** = the strategy's own stop (Donchian and trend-pullback: 2 ATR; for momentum-rotation, which has no stop, a *planning* stop of 2 ATR(20) used only for R:R and sizing, never as an exit order, so the base strategy's behaviour is unchanged);
**target** = the 50th percentile of the maximum favourable excursion of past shadow trades of the same strategy (resolved, point-in-time, embargoed); then net R:R as in 3.1. The plan is immutable after entry (hash recorded); the council never moves a stop or target. PASS iff net R:R >= 2.0.

### 3.5 Time Machine

1. **Analogs:** every past firing of the same setup (same strategy; bucket by market flag count in {0, 1, 2+}) whose `exit_ts <= as_of - embargo`; shadow outcomes are recorded for every signal, taken or vetoed, so selectivity cannot starve the history. Window 1,260 trading days (equity) or 365 days (crypto).
2. **Net outcomes:** gross minus `tq-costs` schedule, slippage (`docs/slippage-model.md`) and an expected tax haircut `tau * max(net, 0)` (no loss credit).
3. **Seeded Monte Carlo:** 1,000 calendar-block bootstrap draws (month blocks equity, week blocks crypto) of the shrunk bucket mean (prior weight `k0 = 20` to the strategy mean), seed = first 8 bytes of sha256(as_of_ms || bucket); `lcb90` = 10th percentile; `confidence` = fraction of draws with mean above 0.
4. **PASS iff** `n >= 30`, strategy `n >= 60`, `mu >= H` (hurdle: 0.50% equity, 0.60% crypto), `lcb90 > 0`, AND the win-rate gate (3.1) holds. Cold history is ABSTAIN.

### 3.6 Risk and Budget

**Risk:** dry run of the unchanged chain `Strategy/Insight -> Portfolio -> RiskChain -> Execution -> RiskGate::check_ctx` on a cloned limiter, plus kill-switch state; PASS iff the unfiltered pipeline would have approved. The council can never approve what the gate rejects, and the real order still passes the real gate afterwards.
**Budget:** computes the quantity per 3.1 and VETOes if quantity is zero, max loss per trade exceeds the cap, or the frequency cap is hit; otherwise returns multiplier `v` (shrink only).

## 4. Schemas (versioned JSON, `deny_unknown_fields`)

New crate `tq-council` (implementation phase). Schema types have **no dependency on order, broker or gate types** (pattern of `tq-alpha-api`), so nothing a voter or an LLM emits can carry a side, quantity or price. Every top-level type starts with `schema: "council.<name>.v<N>"`; unknown schema or version is a parse error;
`#[serde(deny_unknown_fields)]` on every struct and tagged enum. Decimals are strings, timestamps UTC epoch ms, sizes are weights.

| Schema | Fields (all required) |
| --- | --- |
| `council.candidate.v1` | `candidate_id` (sha256 of the rest), `as_of_ms`, `symbol`, `market`, `strategy_id`, `strategy_cfg_hash`, `signal_ts_ms`, `proposed_weight`, `reference_price` |
| `council.plan.v1` | `candidate_id`, `stop`, `target`, `k_cost_per_share`, `rr_net`, `plan_hash` |
| `council.vote.v1` | `candidate_id`, `voter` (closed enum of the 8 roles above), `vote` (`PASS`, `VETO`, `ABSTAIN`, `NOT_AVAILABLE`), `multiplier` in {0, 0.5, 1} (or a decimal in [0.25, 1] for Budget), `reasons[]` (closed `ReasonCode` enum), `metrics` (named decimals, e.g. `rr_net`, `wilson_lb`, `p_star`, `lcb90`, `n_bucket`), `input_hash` |
| `council.llm_advice.v1` | `candidate_id`, `voter` (`market_news` or `time_machine`), `veto`, `shrink` in {`1`,`0.5`}, `reason` (max 200 chars), `model_id`, `prompt_hash`, `response_hash`. No other fields; an extra `order`, `qty`, `price`, `size` key is a parse error, handled as `llm_malformed` (recorded, counted as an LLM veto) |
| `council.verdict.v1` | `candidate_id`, `outcome` (`PASS`, `SHRINK`, `VETO`, `ABSTAIN`), `final_multiplier`, `votes[]` (all `council.vote.v1`), `llm[]`, `deterministic_only` |
| `council.logrecord.v1` | section 7 |

`ReasonCode` is a closed enum (`cold_history`, `lcb_not_above_zero`, `net_edge_below_hurdle`, `rr_below_min`, `winrate_lb_below_breakeven`, `entry_not_extended_fail`, `regime_flags_2plus`, `freq_cap_day`, `freq_cap_week`, `risk_gate_rejected`, `kill_switch_halted`, `input_missing`,
`not_available_data`, `llm_veto`, `llm_malformed`, `log_write_failed`, ...), so abstention and veto causes are exactly countable.

## 5. Unanimity rule

Evaluated in this order:

1. If any voter's vote is `ABSTAIN`: outcome `ABSTAIN`, multiplier 0.
2. Else if any voter `VETO`es: outcome `VETO`, multiplier 0.
3. Else (all voters PASS; `NOT_AVAILABLE` roles are excluded from the count, recorded, and never counted as PASS): `final_multiplier = min(m_market, m_budget, m_llm...)`; 1 gives `PASS`, anything below gives `SHRINK`.
4. `approved_weight = proposed_weight x final_multiplier`, never above the Risk/Budget ceiling.

No averaging or majority. LLM advice enters only as a `min` term after the deterministic votes and cannot upgrade a `VETO` or `ABSTAIN`. A voter missing at its deadline is `ABSTAIN`.

## 6. Plug-in to tq-pipeline and RiskGate (veto or shrink only)

* **Type-level shrink:** the council output is a `Shrink(Decimal)` newtype constructible only in `[0, 1]`.
* **Pipeline stage:** `CouncilRisk` implements `tq_pipeline::RiskModel` and is the **last** model in the `RiskChain`, whose `apply` clamps every model's output through `shrink_only` (same sign, magnitude no larger, no new symbols). It holds a read-only `VerdictBook` produced by `Council::evaluate(as_of, candidates)` before
  `Pipeline::run_insights`; a target without a verdict is shrunk to zero (default abstain).
* **RiskGate unchanged and last:** orders from council-shrunk targets still pass `RiskGate::check_ctx`, kill switch, locks, collar and rate limiter. The Risk voter is a dry run of that same gate.
* **Exits are never gated.** The council evaluates entries only; v1 does not shrink open positions.
* **Backtest engines:** `RiskWiring` (tq-backtest) gets an optional entry-filter hook before `check_ctx`, returning `Allow(multiplier)` or `Deny(reasons)`. Absent hook is bit-identical to today (existing byte-identity checks). The Exit Planner's planning stop for rebalancers feeds sizing only.
* **Live and paper:** same `Council::evaluate`, injected `Clock`, no wall-clock inside the council; a council error or timeout is `ABSTAIN`. The Broker role is the existing paper `PaperBroker`.
* **LLM boundary:** adviser behind a trait returning `council.llm_advice.v1`; not called in any registered backtest; recorded advice is replayable from the log byte-identically.

## 7. Append-only decision log (auditing selectivity)

`tq-council::DecisionLog`: JSONL, one `council.logrecord.v1` per line, **hash-chained** (`hash = sha256(index || prev_hash || canonical_json)`), monotone `seq`, exclusive lock, append plus `fsync`, with the trial ledger's anchor discipline (sibling anchor plus git-committable mirror, fail-closed load; see
`crates/backtest/src/honesty/store.rs`). Record kinds:

* `EVAL`: the full `council.verdict.v1` with every `council.vote.v1` (including `ABSTAIN`, `VETO`, `NOT_AVAILABLE` with reasons) and `council.plan.v1`, written for **every** candidate **before** the pipeline continues (write-ahead). If the append fails: `ABSTAIN(log_write_failed)` and a kill-switch advisory.
* `OUTCOME`: when the real or shadow trade resolves: `candidate_id`, `exit_ts`, gross, net-of-cost, net-of-tax return, MFE, `taken`. Vetoed candidates get counterfactual outcomes.
* `LLM`, `CONFIG` (frozen parameter set and hash), `SEAL` (periodic Merkle root).

Audit queries without new code: abstain rate and reason histogram; veto rate by voter; **per-voter marginal vetoes (candidates vetoed by this voter only)**; **pairwise vote correlation matrix**; PASS vs VETO realised return (calibration); selectivity by month and by day/week cap hits; LLM advice that changed an outcome; any `EVAL` with no `OUTCOME`
after its horizon. Large logs are git-ignored; `SEAL` roots and the anchor mirror are committed with every registered run.

## 8. Failure modes and guards

| Failure | Guard |
| --- | --- |
| Many correlated voters, little added safety, more turnover cost | ablation (leave-one-out and solo), pairwise correlation, retention margin; minimal council reported |
| Council trades little, reports a high Sharpe on a mostly-cash book | like-for-like from first invested day with cash credited at 6%, random-veto control, trade and months-active floors |
| Look-ahead | point-in-time snapshots; `tq-bias` prefix-invariance check before any result; shadow outcomes visible only after `exit_ts + embargo`; fills at next open only |
| Calibration set starved by the filter | shadow outcomes recorded for every signal |
| LLM injection or smuggled order fields | closed schema, `deny_unknown_fields`, veto/shrink-only, LLM absent from registered backtests |
| NOT_AVAILABLE read as approval | explicit vote value, excluded from count, never a PASS, reported |
| Parameter creep | all parameters frozen; changes are new counted trials, tighten only |
| Log tampering or loss | hash chain, anchors, write-ahead; log failure is an ABSTAIN |

## 9. Out of scope for v1

No derivative hedges, no shrink of open positions, no live or real broker, no learned voter weights, no LLM in a registered backtest, no Earnings Analyst vote, no Pelosi signal.

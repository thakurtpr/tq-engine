# Edge hunt 02: pre-registered test spec for passive market making (candidate B)

Author: thakur-op2, 2026-11-02. Answers task 18 from thakur-op1. **Pre-registration:** written before any recorded data was analysed.
I have read only the recorder's source (`engine/crates/l2rec/src/lib.rs`) and the directory listing of `engine/data/l2/um`; I have opened no data file.
Every threshold is a **[convention]** for the user to confirm, as in `docs/validation-protocol.md`. Record the SHA-256 of this file in `trials.csv` when the simulator
is built; any later change to this document is a protocol change and must be logged as one. Docs only; limits unchanged (no real money, credentials, order tools, leverage).

## 0. Verdict first: the arithmetic already says this will probably fail

- **Spread capture is tiny.** BTCUSDT's USD-M tick is 0.10 USDT **[unverified: confirm from public `exchangeInfo`]**. At a price near 77,500 that is 0.13 bps, so quoting at the touch earns at
  most about 0.065 bps of half-spread per fill, against a regular-tier maker fee of 2.0 bps (1.8 with BNB) per fill. Break-even therefore needs a mean **markout** (price drift in our favour after a fill) of
  about +2 bps per fill: the entire edge must come from forecasting the next seconds of price, not from the spread.
- **Fills are adverse by construction.** A resting bid fills when sellers hit it, which is exactly when the price tends to fall; the sample the simulator sees is conditional on that.
  Task 15 already showed that the winners are the trades a passive order misses.
- **Prior.** I expect kill criterion K2 below to fire at the regular fee tier. The test is still worth running because it is cheap (data already recording) and the VIP/rebate rows decide
  whether any realistic account tier could work, but nobody should plan around B working. The spot LP rebates (-0.4 to -0.8 bps) I verified apply to **spot and margin only**
  (`docs/fee-breakeven-15.md`); a USD-M equivalent is **unverified**.
- **What B can still show.** Whether short-horizon order-book imbalance carries more than about 2 bps of drift conditional on being filled, which is a genuine, narrow research question.

## 1. Data, sample and splits

- Source: `engine/data/l2/um/<YYYY-MM-DDTHH>.jsonl.gz` (BTCUSDT, ETHUSDT; depth@100ms, aggTrade, bookTicker, REST snapshot every 10 minutes, connect/disconnect events).
  Line forms: `{"rx":ms,"m":...}`, `{"rx":ms,"snap":SYMBOL,"m":...}`, `{"rx":ms,"event":"connect"|"disconnect","info":...}`. Ordering: the combined stream is one websocket, so **file line order is the
  canonical arrival order**; `rx` (local ms) is used for latency measurement and does not order lines.
- **Truncated files.** Hour files of a crashed process may be unfinished gzip; count them, read what decodes, and treat the undecodable tail as a gap. Hour files with a `.N` suffix are restarts
  (one already exists for 2026-11-02T18).
- **Dev slice:** the first 24 hours of recording are used only to debug the rebuild and the fill model and are **excluded** from every reported statistic.
- **Holdout:** the last 25% of days (rounded up, at least 4 days) are reserved for one pre-chosen cell (section 7) and unlocked once.
- **Sample:** see section 8. Test sample = all verified time after the dev slice and before the holdout.
- **Recorder notes for thakur-op1** (not blocking): (a) Binance's current docs show the stream URL as `wss://fstream.binance.com/public/stream?...`; the recorder uses the older `/stream` path and works today, but a
  deprecation is possible **[unverified]**. (b) Add a periodic public `GET /fapi/v1/time` call and log it as an `event` so local clock offset is measurable; without it, assume an NTP-synced clock.

## 2. Gate 1: book rebuild and data integrity (before any strategy statistic)

Binance's rule for USD-M diff depth (read from the official page): after a snapshot with `lastUpdateId`, drop events with `u < lastUpdateId`; the first processed event must satisfy `U <= lastUpdateId` and
`u >= lastUpdateId`; every later event must have `pu == previous u`, otherwise resynchronise from a new snapshot; quantities are absolute, 0 removes the level, and removing an absent level is normal.
([Binance docs](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/How-to-manage-a-local-order-book-correctly))

Checks, per symbol per hour and overall, reported as a table:
1. **Continuity gap rate:** events with `pu != previous u`, per million events and per hour; seconds with no depth event over 1.0 s; and each connect/disconnect event. A **verified segment** is a maximal span that starts at a
   snapshot satisfying the sync condition and has no break. Report the fraction of wall-clock time in verified segments.
2. **Snapshot consistency:** at each periodic snapshot (lastUpdateId L), compare the rebuilt book after the event covering L with the REST snapshot: top-20 bid and ask levels, price and quantity, ignoring
   levels touched by that covering event. Report the share of snapshots that match exactly at levels 1, 5 and 20.
3. **bookTicker agreement:** share of bookTicker messages whose best bid and ask equal the rebuilt book at the matching `u`/arrival position.
4. **Feed latency:** distribution of `rx - E` per minute (median, p99). Used for latency modelling in section 6; windows with p99 over 500 ms are excluded **[convention]**.

**Gate 1 pass [convention]:** at least 95% of wall-clock time in verified segments; at most 1 continuity break per symbol per 6 hours; at least 99.5% of snapshots match at level 20.
If Gate 1 fails, the verdict is "recorder cannot support the test" (not "no edge") and the simulator runs only on verified segments, shortening the sample.

## 3. Fill model for passive quotes (queue position)

- A quote is a limit order for a fixed size at a price on the book. One bid and one ask at a time per symbol; quotes join the **back** of the queue; our size is small enough to ignore market impact.
- **Placement time.** A decision made on the event at local arrival `rx` takes effect on the exchange at `T_place = E_event + d_feed + L_place`, where `d_feed = rx - E` for that event (the one-way feed delay plus
  clock offset) and `L_place` is the latency in section 6. Queue-ahead `Q0` = displayed quantity at that price in the rebuilt book at `T_place` (ordering by exchange event time `E`/`T`; ties by line order).
- **Fill events** for a bid at price p (mirror for an ask):
  1. **Trade-through:** an aggTrade with taker side sell at a price strictly below p (the book through p is gone): filled in full at p.
  2. **Queue exhaustion:** cumulative aggTrade sell volume at exactly p since `T_place`, minus `Q0` (adjusted per the queue-advance rule below), is positive: filled for that excess, up to our size (partial fills allowed).
  A quote that is cancelled but whose cancel arrives after `T_cancel = E_event + d_feed + L_cancel` can still fill before `T_cancel` (the cancel race is part of the adverse selection).
- **Queue-advance rule (two variants, both pre-registered):**
  - **Conservative (headline, lower bound on fills):** the queue ahead shrinks only by executed volume at p. Cancellations ahead of us are assumed not to advance our position.
  - **Optimistic (upper bound):** when displayed quantity at p falls for reasons other than trades, the queue ahead is scaled by new_qty / old_qty (cancellations are uniform across the queue).
- **Interpretation rule:** a failure under the **optimistic** model kills B. A pass only under the optimistic model is "simulation-only, inconclusive", not a pass.
- **No improving the touch** (no quoting inside the spread): the family below quotes only at the best bid and best ask.
- Quote size (fixed): 0.002 BTC and 0.05 ETH, about 150 USD each, above Binance's minimum notional **[unverified: confirm minimum quantity and notional from `exchangeInfo`]**. One unit q = one quote size.

## 4. Quoting rule family (fixed, minimal)

State: inventory `inv` in units of q, best bid/ask from the rebuilt book, imbalance `I = (sum bid qty - sum ask qty) / (sum bid qty + sum ask qty)` over the top K = 5 levels.
Re-quote only when the touch changes, at most once per 100 ms **[convention, an order-rate courtesy]**.

| Cell | Rule |
| --- | --- |
| S0 | Quote bid at best bid and ask at best ask, always, both sides. Hard inventory limit \|inv\| = 4q: on breach, flatten to 2q with a taker order. |
| S1 | S0, plus inventory skew: if inv >= +2q cancel the bid (quote only the ask); if inv <= -2q cancel the ask (quote only the bid). |
| S2a | S1, plus cancel the bid when I < -0.3 and cancel the ask when I > +0.3. |
| S2b | S1, plus the same with threshold 0.5. |

That is **4 cells x 2 symbols = 8 tests**, counted as **4 trials** in the ledger (the symbols are one hypothesis family) with a Bonferroni-adjusted interval over 8 tests (section 7). No parameter is tuned on the data:
q, K = 5, the 2q/4q limits and the thresholds are fixed here. Quoting continues around the clock (no time-of-day filter). The inventory is marked to the rebuilt mid.

## 5. Costs (reported as rows, headline row first)

| Row | Maker per fill | Taker (flatten) | Source |
| --- | --- | --- | --- |
| **Headline: regular tier** | 2.0 bps | 5.0 bps | Binance futures FAQ example rates (cited, not a confirmed schedule; `docs/binance-futures-facts-16.md`) |
| Regular + BNB | 1.8 bps | 4.5 bps | same FAQ, 10% BNB discount |
| Hypothetical VIP-like | 1.0 bps | 3.0 bps | assumption, **unverified** |
| Zero fee | 0.0 bps | 3.0 bps | assumption |
| Hypothetical rebate | -0.5 bps | 3.0 bps | assumption, **unverified**; verified spot LP rebates are -0.4 to -0.8 bps but spot-only |

Funding payments on open inventory and exchange API rate limits are excluded (inventory is at most 4q, about 600 USD). Fees are charged per fill on filled notional. Indian taxes and TDS are out of scope
for the test and would only subtract (unresolved, `docs/edge-hunt-01.md`).

## 6. Latency (stated range 50 to 200 ms)

`L_place = L_cancel = L` with **L = 100 ms as the headline**, and 50, 200 and 400 ms as sensitivity rows (400 ms is plausible for a retail connection from India; the range itself is **unverified**).
Effective times are computed in the exchange clock as in section 3, using the measured `d_feed` per event. Add a clock-error sensitivity of +/-100 ms if the clock offset is not measured. A strategy that
needs L under 50 ms to work is dropped as "latency-dependent", since this setup cannot deliver it.

## 7. Metrics, statistics, kill and pass criteria

**Primary metric:** net P&L of the simulated account, in bps of **filled notional** (fees included, inventory flatten at taker cost, inventory marked to mid at block end), per symbol and cell.
**Inference:** block bootstrap over 1-hour blocks, 10,000 resamples; report 95% intervals, and for any pass/kill decision use the **Bonferroni-adjusted 99.4% interval** (8 tests).

**Diagnostics (mechanism):**
- **Markout** at +1 s, +10 s and +60 s after each fill: `side x (mid(t_fill + h) - fill_price) / fill_price x 10^4` bps, `side` = +1 for a buy fill, -1 for a sell fill; mid from the rebuilt book. Report mean and median
  per cell, split by fill type (trade-through vs queue exhaustion), by side, and by imbalance decile at the moment of fill.
- Fills per hour, mean queue wait, share of quotes cancelled before filling, inventory distribution, share of flatten-by-taker events.
- Net per fill = markout_10s minus the maker fee (the fee is the break-even markout).

**Kill criteria [convention]; B is dropped if any one holds:**
1. **K1 (integrity):** Gate 1 fails and the verified sample cannot reach the minimum in section 8 after another 14 days of recording. Verdict: "unbuildable", not "no edge".
2. **K2 (no edge even under optimism):** with the optimistic queue model, L = 50 ms and the headline fee row, the **upper** bound of the 99.4% interval for net bps per filled notional is at or below 0 in **all** four cells on **both** symbols.
3. **K3 (mechanism):** the mean 10-second markout in every cell on both symbols is below the headline maker fee (2.0 bps) after the adjusted interval.
4. **K4 (latency fragility):** a cell that passes at L = 50 ms but fails at L = 200 ms is dropped.
5. **K5 (cost fragility):** a cell that is positive only in the hypothetical VIP, zero-fee or rebate rows is **not** a pass; it is recorded as "needs an account tier we cannot verify" and parked.

**Pass (the only outcome that advances to a paper-quote phase) requires all of:** conservative queue model, L = 100 ms and 200 ms, headline fee row, the lower bound of the 99.4% interval above 0 net bps
in at least one cell on **each** symbol; the same cell positive in the holdout (single look, 95% interval lower bound above 0); Gate 1 passed; and at least the minimum sample in section 8. After a pass the next gate
is a paper-quoting period against live books (no real orders) and a human decision; this document grants nothing about real money.

## 8. Minimum sample: is 6 days enough?

- **6 days cannot validate this; it can only kill it.** The statistics are not the bottleneck: if a symbol sees a few hundred fills per hour (my assumption, unmeasured), the standard error of an hourly mean is small, and a
  clearly negative result will show within days. The bottleneck is **regime coverage**: markout and adverse selection depend on volatility and flow, and 6 days is one weekly cycle with one volatility regime.
  The cap of 15 GB (about 6 days at 1.7 MB a minute) fixes the recorder to that length unless the cap is lifted or files are moved off the disk.
- **Pre-registered rule:** one **futility look at day 6** (after Gate 1 and the dev slice) is allowed for K2 only (it can kill but never approve). Approval requires **at least 14 days of verified data**
  (two weekends and any macro-news days that occur), and **28 days (about 70 GB compressed at the current rate; check free disk space)** for a decision I would want to rely on. If those days contain no volatile period
  (the 99th-percentile 1-minute absolute return stays within its usual band), extend until one occurs, because the strategy's loss is concentrated there.
- **Holdout:** the final 25% of days are not touched until one cell is named.
- **Precision requirement:** report the realised half-width of the 95% interval; a "pass" or "kill" with a half-width over 0.5 bps of net per filled notional is "inconclusive".
- **Data volume warning:** if disk is limited, drop to one symbol (BTCUSDT) rather than shortening the period; coverage in time matters more than a second symbol.

## 9. Trials ledger and what to record

- Add **4 trials** (S0, S1, S2a, S2b) for candidate B to `trials.csv` before the simulator runs. The latency, queue-model and fee rows are sensitivities, not trials.
- Record the SHA-256 of this document, the recorder commit, the start and end of the sample, the dev slice and holdout dates, and Gate 1 results, before any strategy statistic is computed.
- Decisions follow only the pre-registered rules; any additional analysis is exploratory and labelled so.

## 9a. Addendum 2026-11-02 (before any strategy statistic): decisions agreed with thakur-op1

- Tick/step/min notional verified by thakur-op1 from `exchangeInfo`: BTCUSDT tick 0.10, step 0.001, min notional 50 USDT; ETHUSDT tick 0.01, step 0.001, min notional 20 USDT. The fixed q (0.002 BTC, 0.05 ETH) clears both.
- Recorder commit f8fc324 logs `{"rx","tx","servertime":{"serverTime"}}` every 10 minutes (first probe: RTT 194 ms, offset about +60 ms). The clock offset is therefore measured; the +/-100 ms clock sensitivity row applies only to windows without a probe.
- Recording restarted with a 35 GB cap, about 14 days. Restart artefacts (truncated gzip tail in `2026-11-02T18.jsonl.gz.1`, current file `.gz.2`) are handled by Gate 1 as specified.
- **Dev slice:** the first 24 hours from the recorder's start at about 2026-11-02 18:2x UTC (exact start from the first `connect` event in the earliest file); excluded from all statistics.
- **Holdout boundary rule (fixed now, timestamp computed later):** when recording stops, the holdout is the final `max(4 days, ceil(25% of verified days))`. If recording stops at the planned 14 days (about 2026-11-16 18:2x UTC) the holdout starts at about 2026-11-12 18:2x UTC. The sample-closure timestamp must be written down, and the boundary computed from it, **before** any strategy statistic is opened; extending the sample moves the boundary and is allowed only before that point. Once a cell is named from the test period, the holdout is read once.
- **Day-6 futility look (K2 only):** uses test-period data only (day 2 through day 6), never the holdout.

## 9b. Addendum 2026-11-02 (before any strategy statistic): recorder fix and ordering rule

- Review of Gate 1 (`docs/review-gate1-19.md`) found zero aggTrade messages in the first recording (Binance routing split). The recorder was fixed in commit 95aed2c (depth and bookTicker on `/public/stream`, aggTrade on `/market/stream`, separate threads, one hourly file in arrival order, a 30-second silent-stream guard).
  The old files moved to `engine/data/l2/um-legacy-notrades/` and are **excluded** from the test sample and from the dev slice.
- **New dev slice:** the first 24 hours from the first `connect` event of the new recorder (pid 63675, about 2026-11-02 18:42 UTC), closing about 2026-11-03 18:42 UTC. Sample start for all later rules is the end of that slice. Section 9a is otherwise unchanged.
- **Supersedes section 1:** line order is **no longer** a canonical order across streams. Rules for the simulator:
  1. **Book rebuild** (Gate 1) uses depth lines in file order, which is valid within the depth connection (update ids chain by `pu`); aggTrade lines never affect it.
  2. **Exchange effects** (fills, queue consumption, cancel races) are ordered by exchange time: `E` for depth and bookTicker, `T` for aggTrade (trade time). Ties are broken depth first, then bookTicker, then aggTrade, then by line order.
  3. **Information available to the strategy** at decision time is only what had **arrived**: a decision made at local time `rx_d` may use only messages with `rx <= rx_d`. A trade with an early `T` that arrives after `rx_d` is unknown to the strategy at `rx_d` but still acts on its resting quote at `T`, which is how a real fill race behaves.
  4. Because the two sockets have separate delays, report `rx - E` (depth) and `rx - T` (aggTrade) separately; if their medians differ by more than 50 ms, add that difference as an extra sensitivity row for the latency analysis **[convention]**.
- **Gate 1 changes agreed** (commit 5dd540f): the full snapshot is kept; every untouched level is checked with a floor of 20 compared levels per side (below that it is "uncheckable", not a failure); PASS needs L20 and all-untouched at or above 99.5% **[convention]**. Latency uses the minimum-RTT probe within 20 minutes and prints the RTT. The bookTicker quantity check will be added with the simulator.

## 10. Open items for the user

1. Confirm the thresholds above (all [convention]) and the headline fee row.
2. Whether the machine can hold 70 GB of recordings, or whether to run BTCUSDT only.
3. The recorder is a long-running process that talks to public streams only (no keys); the earlier "no live runner" limit was written for trading runners. Thakur-op1 says the user approved it directly; it is the user's decision to keep.

# Task 19: independent review of Gate 1 (`tq-mm-gate1`, commit 5d1a15f)

Reviewer: thakur-op2, 2026-11-02. Scope: `engine/crates/mm` (`gate1.rs`, `sync.rs`) against `docs/edge-hunt-02-mm-spec.md` section 2 and Binance's USD-M diff-depth rule.
Replica: `docs/review-gate1-19-scripts/gate1_replica.py` (pure Python, written from the spec and Binance's rule; I read the engine source only after the replica was running, to explain differences).
It has a built-in self-test (`--selftest`: stale events, first-event cover, continuity break, deferred and late snapshots, an injected quantity error). Read-only on `engine/`.
**This is an interim review on the first ~21 minutes of recording (dev slice only). The 24-hour run waits for the corrected dev slice (see finding 1).**

## Verdict

**Gate 1's sync logic is right and the replica agrees with it on every count that matters. Two findings need action: the recorder receives no aggTrade messages at all, and the checker's snapshot test produces a false failure on busy books (ETHUSDT).**

1. **The recorded files contain zero aggTrade messages** (1,067,248 lines counted in the first three files; the connect URL requests `btcusdt@aggTrade` and `ethusdt@aggTrade`). The fill model needs trades. Probable cause: Binance's 2026 USD-M WebSocket routing split
   (/public for depth and bookTicker, /market for aggTrade, markPrice, kline; announcement 2026-03-06 with legacy retirement 2026-04-23, from third-party library notes, not read on Binance's own page).
   The recorder's legacy `/stream` path still serves the public streams but not the market ones. Sent to thakur-op1 as urgent; the current dev slice has no usable trade data and has to restart.
2. **Snapshot check false failure (bug in `sync.rs`).** On ETHUSDT the checker reports L1 / L5 / L20 = 100% / 0.0% / 0.0%. The book is correct (my full check below found 0 mismatches). Cause: for a snapshot whose id is ahead of the book (the deferred path),
   the checker keeps only the first 40 snapshot levels (`CHECK_LEVELS[2] * 2`, `sync.rs:155-157`) *before* removing the levels the covering event touched. ETH's covering event spans about 28,600 update ids and touches 183 bid and 170 ask levels; inside the snapshot's top 40,
   only 3 bid and 8 ask levels were untouched. `visible(..., 20)` then compares 3 to 8 snapshot levels with 20 book levels and fails. BTC (36 and 37 untouched) passes. Fix: keep all 1,000 snapshot levels, as the late-snapshot path already does.
3. **Everything else reproduces:** event counts, stale counts, breaks, silences, crossed books, bookTicker agreement, latency and clock offset (section 1).
4. **Hardening the snapshot test (recommendation):** the top-n test has weak power on active books (the covering event touches most of the top of book). Add a *full* check: every level of the 1,000-level snapshot that the covering and later events did not touch must equal the book exactly. That test is non-vacuous; see section 3.

## 1. Reproduction on the first ~21 minutes (3 files, 3 connects, 1 stop)

| Quantity | Engine (`tq-mm-gate1`, 0.33 h window) | Replica (21 min) |
| --- | --- | --- |
| Depth events BTC / ETH | 10,829 / 10,807 | 10,829 / 10,807 (same window as the engine's run) |
| Stale events BTC / ETH | 8 / 12 | 8 / 12 |
| Breaks, silences over 1 s, crossed books | 0, 0, 0 on both | 0, 0, 0 on both |
| Verified time share | 94.12% / 94.11% | 94.2% / 94.2% (segment span vs sum of gaps: 0.1 point) |
| bookTicker at the same update id | 695 / 799 checks, 100% price match | 694 / 798 checks, 0 price and 0 price+quantity mismatches |
| Clock probes | 2, offset median 106 ms (62..106), RTT median 346 ms | same probes, offset 106 ms |
| Latency `rx - E`, raw median / corrected | 51 / 116 ms; worst-minute p99 472 / 442 ms | 51 / 116 ms; 472 / 442 ms; 0 minutes over 500 ms |
| Truncated gzip tails | 2 files | 2 unfinished files, 0 undecodable tails |

(The 1-event difference in bookTicker checks is a window edge: the engine's run ends at its own last line.)

## 2. Rulings on choices (a) to (e)

- **(a) Sync rule: accepted.** Matches Binance's documented rule (drop `u < lastUpdateId`; first event must satisfy `U <= L <= u`; then `pu == previous u`); stale counts agree exactly (8 and 12).
- **(b) Late snapshots ignoring the covering event and all later applied events: accepted, with conditions.** It is slightly more lenient than my spec, but I verified the stricter state-as-of-the-covering-event reconstruction in the replica (undo logs of later events) and found the two agree here.
  Conditions: (i) report deferred and late checks separately, which the replica does; (ii) fix the truncation bug in finding 2; (iii) report the number of levels actually compared, because a check with few compared levels is vacuous.
  Your `visible(side, skip, n)` (first n *untouched* levels) is a stronger test than my union-minus-excluded set and I would keep it once the truncation is fixed; add the full untouched-level check as the primary integrity metric.
- **(c) bookTicker only at the same update id: accepted and confirmed.** bookTicker is real time and depth is batched at 100 ms, so a ticker can only be compared after the depth event whose final id equals its id; a ticker always arrives first, so the deferred check you use is required
  (my first version looked only backwards and checked 178 BTC and 0 ETH instead of ~700 and ~800: found by cross-checking against your counts). Compare quantity as well as price: the replica finds 0 quantity mismatches.
- **(d) Reset on connect, disconnect or stop; verified time between consecutive applied events while synced: accepted.** The two verified-time definitions differ by 0.1 point.
- **(e) Latency reporting: accepted, with a caution.** Raw and corrected figures reproduce exactly. The offset is estimated as the server time minus the midpoint of the request, which carries an uncertainty of about +/- RTT / 2; with a median round trip of 346 ms that is about +/- 170 ms,
  and the offset moved from 62 to 106 ms between two probes. The corrected median of 116 ms is therefore not a precise one-way delay. Use the probe with the smallest round trip in a surrounding window (minimum-RTT filter), report the round trip next to every corrected figure,
  and keep the +/-100 ms clock sensitivity row from the spec for windows without a low-RTT probe.

## 3. The snapshot check is weak on active books: evidence

First periodic snapshot (deferred path), covering event's reach:

| | BTCUSDT bids / asks | ETHUSDT bids / asks |
| --- | --- | --- |
| Levels in snapshot | 1,000 / 1,000 | 1,000 / 1,000 |
| Levels touched by the covering event | 46 / 35 | 183 / 170 |
| Of which inside the snapshot | 23 / 24 | 156 / 151 |
| Untouched levels inside the snapshot's top 40 | 36 / 37 | 3 / 8 |
| **Full check: untouched levels compared / mismatched** | **977 / 0, 976 / 0** | **844 / 0, 849 / 0** |
| Top-n check, replica (excluding touched levels) | L1, L5, L20: pass | L1, L5, L20: pass |
| Top-n check, engine | pass | L1 pass; L5, L20 **false fail** |

On ETH nearly every top-of-book level is touched within a 100 ms window, so any rule that ignores touched levels compares few of the levels the strategy cares about. This does not make the check useless (the full check validates 1,693 ETH levels exactly) but it means:
a Gate 1 pass on L5 and L20 says little by itself for ETH; the full untouched-level result should carry the weight, and Gate 1 should report the count of levels compared per side with a floor (suggest at least 20 untouched levels in the compared range or the snapshot is "uncheckable").

## 4. What is not yet tested (interim review)

- Only **one periodic snapshot** has arrived in the window, on the deferred path. The behind (late) path, continuity breaks and the first-event failure path have **not occurred in real data**; they are covered only by the replica's synthetic self-test and the engine's unit tests.
  The 24-hour run is needed, with the corrected recorder.
- The restart artefacts are handled: two files end in an unfinished gzip stream and decode up to the cut with no undecodable tails.
- Verified share is 94.1% in this window only because of startup and two restarts (spec threshold 95%); it says nothing about steady state.
- I did not examine ties on equal `rx`; line order is used (the stream is one websocket).

## 5. Recommendations

1. **Fix the recorder** (finding 1): connect aggTrade on `/market/stream` (and depth/bookTicker on `/public/stream` per Binance's current documentation), add a startup assertion that every subscribed stream produces a message within about 30 s, and restart the dev slice.
2. **Fix `sync.rs`** (finding 2): keep the full snapshot in `Pending`; add the full untouched-level check and a compared-level count; keep the first-n-untouched rule.
3. **Gate 1 thresholds** stay as pre-registered; judge them on the full corrected 24 hours, not on this window.
4. **Latency:** use a minimum-RTT probe filter and print the round trip with each corrected figure.
5. After the corrected dev slice closes, rerun `gate1_replica.py` and `tq-mm-gate1` over the full 24 hours and compare (same command shapes: `python3 gate1_replica.py --hours 24` and `./target/release/tq-mm-gate1 data/l2/um --hours 24`).

## Reproducing

```
cd docs/review-gate1-19-scripts && python3 gate1_replica.py --selftest && python3 gate1_replica.py [--skip-hours 0 --hours 24]
```
Reads `engine/data/l2/um` only (about 5 s per 20 minutes of data; expect 10 to 25 minutes for 24 hours). Writes `gate1_replica_summary.json` beside the script. The engine-side output is from `./target/release/tq-mm-gate1 data/l2/um`.

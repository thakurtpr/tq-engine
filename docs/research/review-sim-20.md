# Task 20: review of the fill simulator (`tq-mm-sim`, commit 414b6ed)

Reviewer: thakur-op2, 2026-11-02. Scope: `engine/crates/mm/src/{sim,drive,stats,record}.rs` and `bin/sim.rs` against `docs/edge-hunt-02-mm-spec.md` sections 3-7 and addenda 9a/9b. Read-only on `engine/`; dev slice only.
**Method:** (1) I read the code against the spec; (2) I built synthetic recordings with answers computed by hand **before** running the engine (`docs/review-sim-20-scripts/synth.py`, five scenarios, written to a scratch folder), and ran `tq-mm-sim` on them;
(3) I ran the simulator on the live dev slice for anomalies. **What I have not done:** an independent re-simulation of the real dev slice, because the dev slice (24 h from 2026-11-02 18:42 UTC) is still open. That replica comparison is deferred until `trading-56` tells me the slice has closed.

## Verdict

**The fill rules, queue arithmetic, optimistic denominator, markout sign, fee arithmetic and taker flatten all behave exactly as specified: 5 of 5 hand-computed scenarios reproduce to the second decimal. One real bug (the kill checks fire vacuously on missing data) and several modelling points need action before the 24 h run.**

1. **Verified correct** (section 1): queue exhaustion, trade-through on both sides, latency gating, optimistic denominator, markout sign for buys and sells, fee and taker arithmetic, skew cutoff, 4q flatten to 2q.
2. **Bug: K2 and K3 "fire" when there is no data.** On a recording with zero fills (and on any cell with no maker notional) the report prints "K2 ... **fires**" and "K3 ... **fires**" because `n/a` is treated as "dead" (`is_none_or(...)` in `stats.rs:147,150`).
   A stalled trade stream, like the one the first recording had, would therefore look like a kill. Treat `n/a` as "not evaluable" and print that.
3. **Offset uncertainty exceeds the latency being modelled.** The offset estimate is good to about +/- RTT / 2 (median RTT 346 ms, so +/-170 ms), which is larger than the 100 ms headline latency. Add the offset error as a sensitivity row (section 3).
4. **The 20 s reorder window** is adequate for the first hour but should be treated as a gate, not a statistic (section 4).
5. **Decision timestamp** (rx plus the mapped offset) is the right causal choice; keep it, and add a staleness-guard variant as a sensitivity row (section 4).

## 1. Hand-computed scenarios (BTCUSDT, q = 0.002, no clock probes so offset = 0; regular fee 2.0 / 5.0 bps)

| Scenario | What it tests | My hand answer | Engine output |
| --- | --- | --- | --- |
| S1 | Queue exhaustion: displayed 5.0 ahead, sell 4.000 then 1.003 at our price; markouts after the mid moves to 99.1, 98.1, 101.0 | L = 50, 100, 200: 1 fill (0.002 at 100.0); markouts -90.00 / -190.00 / +100.00; net +98.00 bps. L = 400: not yet live at the trades, 0 fills | identical, all four cells, both queue models |
| S2 | Optimistic denominator: displayed size falls 5 to 1 with no trade, then a 1.001 sell | Conservative: ahead 5 - 1.001 = 3.999, 0 fills. Optimistic: ahead 5 x 1/5 = 1.0, then 1.0 - 1.001: fill 0.001; net +98.00 | conservative 0 fills; optimistic L = 50/100/200 1 fill, net +98.00; L = 400 0 fills |
| S3 | Trade-through on both sides and markout sign for a sell | 2 fills, 100% trade-through; markouts +4.95 / +4.90 / +5.04 (buy -90, sell +99.90 at 1 s); net +3.00 bps | identical |
| S4 | No trades | 0 fills everywhere | 0 fills everywhere (but see finding 2) |
| S5 | Falling market, four bid trade-throughs to 4q, taker flatten to 2q; skew cells stop bidding at 2q | S0: 4 fills, 1 flatten, maker notional 0.788, taker 0.388, net **-154.21** bps. Skew cells: 2 fills, 0 flattens, net **-248.23** bps | identical (-154.21 and -248.23; flatten counts 1 and 0) |

So the engine's fee model, per-block mark to mid, 1-second markout sign (buys negative when the price falls, sells positive), partial fill sizing (0.001 of 0.002), latency gating and skew rule are right in these cases. The five scenarios are small; they cannot show an error that needs many interacting events.

## 2. The vacuous kill (bug)

`report()` evaluates K2 with `m.net[0].as_ref().is_none_or(|n| n.2[1] <= 0.0)` and K3 with `is_none_or(...)`. When a cell has no fills, `net` is `None` and the check counts as "dead". On scenario S4 (no trades) both lines print **fires**.
This is exactly the failure that would have occurred with the first recording (no aggTrade messages): the report would have declared the strategy dead. Fix: collect cells that cannot be evaluated, print "not evaluable: n cells" and make K2/K3 `fires` only if every cell is evaluable and dead. The "NOT A VERDICT" line for fewer than 24 hourly blocks already covers the sample length; it does not cover missing data.

## 3. Modelling points (read from the code; not all testable on the dev slice)

1. **Clock offset error versus latency.** Orders take effect at `rx + offset + L` with `offset = server - local` from the minimum-RTT probe within 20 minutes. The estimate's error is about +/- RTT / 2. With a 346 ms median RTT that is +/-170 ms, more than the 100 ms headline latency and as large as the 200 ms row.
   Fill timing is therefore uncertain by more than the parameter being studied. Add rows with the offset shifted by -RTT/2 and +RTT/2 (a sensitivity, not a trial), and print the probe RTT used.
2. **Cancel-replace loses queue position even when it should not.** In `decide`, `side.target()` returns the pending price if a pending action exists. If the touch moves away and back within L, `want` equals the live price but differs from the pending price, so a new pending is set; at activation `live.take()` cancels the order and places a new one at the same price at the **back** of the queue.
   Real exchanges would keep the original order if the replace had been cancelled in time, but the sim also cannot know. The effect is small and pessimistic; document it.
3. **Re-quote rate limit drops decisions.** A decision within 100 ms of the previous one returns without queuing the change (`last_decision`). Depth events normally arrive 100 ms apart, so the next event repairs it, but in late bursts (2 to 14 s, see section 4) many events arrive together and only the first changes quotes. Pessimistic, small.
4. **Q0 is the displayed size at the last depth event.** Trades at our price between that event and our order's activation are not subtracted, so Q0 is slightly high (pessimistic). The optimistic denominator `pre_qty - traded` does subtract such pre-activation trades, which biases the optimistic model slightly the other way; both are tiny but the conservative / optimistic gap is not perfectly clean.
5. **Markout timing.** The 1/10/60 s markout uses the first depth event at or after `t + h` (`resolve_markouts`); the spec said the last state at or before. The difference is at most one batch (about 100 ms, or seconds in a late burst) and unbiased in direction.
6. **Normalisation.** Net bps is divided by **maker** notional only. The spec said "filled notional"; if taker flatten notional were included the figures shrink by `maker / (maker + taker)`. Report both, or state the denominator in every table.
7. **Bootstrap.** Ratio of sums over hourly blocks with SplitMix64, 10,000 resamples, percentile intervals at 2.5/97.5 and 0.3125/99.6875 (Bonferroni over 8 tests): arithmetic and units are right (the point estimates in S1, S3 and S5 are exact). With fewer than about 100 blocks the 99.4% percentile interval is just the range of resampled means of a few blocks,
   so it is not informative until the sample has hundreds of hours; the 24-hour "not a verdict" guard is the minimum, and the day-6 look (144 blocks) is still thin. Report the number of blocks next to every interval.
8. **Everything else checked by reading:** trade side (`m = true` means the taker sold, so bids are hit), aggTrade time `T` for ordering, the bookTicker not used, post-only rejection, hour-end mark, imbalance rule signs (cancel bid when I < -c, ask when I > c), quote sizes (0.002 BTC, 0.05 ETH).

## 4. Questions asked: the 20 s window and the decision timestamp

- **Reorder window.** Keep the idea, change the status. Make order inversions a **gate**: any inversion at the chosen window means the sample is not trusted until the window is raised and the run repeated. The cost of a long window is only latency to output and memory (tens of thousands of events), so raise it to 60 s now and keep the inversion count (and worst inversion) in every report.
  On the live dev slice the counters show 0 inversions with 20 s (32,927 events released), but bursts of 2 to 14 s were seen earlier; a longer slice will contain longer bursts.
- **Late bursts are an input, not noise.** If this link delivers bursts of 2 to 14 s, the sim quotes blindly on stale data and is run over, which a real strategy with a staleness guard would avoid. Report the share of time each hour in which `rx - E` exceeded 1 s, and add a **sensitivity row with a staleness guard** (pull quotes when the latest depth event is more than 500 ms old, mapped to the exchange clock).
  Do not add it as a trial. If the bursts come from the recorder (thread scheduling, buffering) and not the network, that is a recorder bug to fix first.
- **Decision timestamp.** `rx + offset` of the depth event is right: it uses only what had arrived. Keep it. Orders take effect at that time plus L, as in the spec; the spec's `E + d_feed + L` is the same thing with `d_feed = rx - E`, except that the implementation additionally converts local to exchange time with the measured offset, which is more careful. Keep that, and add the offset-error rows (section 3, point 1).

## 5. Live dev slice anomalies (about 25 minutes, 1 hourly block, so no verdict)

- Counters: 32,927 events released, 0 order violations, 39 trades while unsynced (start-up), 3 book resets, 1 trade-socket gap, 1 truncated tail: consistent with the earlier restarts.
- BTC S0 conservative L = 100: 212 fills, 73% trade-through, 43 flattens, mean markouts -0.81 / -0.96 / -0.94 bps, net -4.96 bps. ETH: 355 fills, 88% trade-through, 48 flattens, markouts -0.93 / -1.06 / -0.82, net -4.24 bps.
  These are from a single hour block and are only sanity checks. A fill mix of 73-88% trade-through, and many taker flattens in an hour, is what the spec predicted for S0 (fills when the price moves through the quote). The pattern across hours is the real test.

## 6. Recommendations before the 24 h run

1. Fix the vacuous K2/K3 (section 2).
2. Add the offset-error sensitivity rows (+/- RTT/2) and print the RTT used.
3. Make reorder inversions a gate and raise the window to 60 s; add the hourly share of delayed feed time and a staleness-guard row.
4. Print the number of blocks beside each interval and the maker-only denominator in the table header (or both denominators).
5. When the dev slice closes: I will run my own re-implementation of S0, conservative, L = 100 on the full slice and compare fills, flattens and markouts with `tq-mm-sim`; agree the exact tolerance first (counts exact, bps to 0.01).

## Reproducing

```
python3 docs/review-sim-20-scripts/synth.py <scratch dir>        # writes the five scenario folders and prints my hand answers
./target/release/tq-mm-sim <scratch dir>/<scenario>              # run from engine/
```

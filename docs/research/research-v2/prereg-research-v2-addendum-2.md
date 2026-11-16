# Addendum 2 to `prereg-research-v2` (implementation decisions taken before any result was seen)

Dated 2026-11-17. This is a record of **decisions taken by the orchestrator on 2026-11-17 BEFORE ANY RESULT WAS SEEN**: no sweep has been opened in the real ledger, no scored run has been made, no real-data backtest exists, and the data step has not been run on `data-raw` (`docs/r2run-notes.md`). The tag `prereg-research-v2` (commit cb1da79, master sha256 `0f36d2a5f7f8acafd9a19e747c1e9553374e7f10953cbd84b7708de755f1733c`) is untouched and is never moved; addendum 1 (commit 3b45fb0) is likewise unchanged.

**Scope.** Like addendum 1, this addendum changes **no gate, threshold, cap, split boundary or catalogue entry**. Items below are engine and data limitations, harness workarounds, and wave-1 interpretations that the wave-1 preregs (`docs/prereg-r2-*.md`, specs in `research/r2run/specs/`) already state in their own section 10; this file records them centrally and **indexes** them (D11). Where an item differs from the tagged text, the tagged text governs and the difference is in section 2. This file was written after merging `hardening/integration` (wave-1 preregs, specs and `docs/r2run-catalogue-status.md`).

## 1. Decisions

### D1. Crypto universe: the "top-12 by 90-day dollar volume" rule (master s.6)
* **r2-xsmom-cr** cannot apply it: the rebalancing interface sees closes only (no volume). **Approved implementation limitation:** rank **all pool pairs listed >= 365 days (at most 20)**; there is no volume ranking.
* **r2-trend-cr-{sma,don,tsmom} and r2-mr-cr-{rsi2,zscore,ibs}** compute a **monthly top-12 inside the rule**, using the trailing 90 daily bars of `close x volume` from bars strictly before the month start; pairs younger than 365 days are ineligible.
* The **survivor-bias flag of addendum 1 s.3 stands for all crypto books** (the pool is the 20 archived pairs, survivors by construction; results cannot alone justify a promotion).

### D2. r2-xsmom-cr notional cap and the k = 3 cells
The per-position notional cap stays at the master's assumed **20%**. With `k = 3`, equal weights of one third exceed the cap, so the three `k = 3` cells (of 27) hold **zero exposure: degenerate cells**. They remain in the grid because **K is frozen at 27**, they **count in N** (27 cells opened), they are **reported as degenerate**, and they can only count as non-profitable neighbours or cells. The **base cell (90, 7, 5) is unaffected.** No cap is changed.

### D3. Crypto slippage: one flat value per run (engine limit), set per family
The engine accepts a single slippage value per run, so master s.7.2's schedule (5 bps BTC/ETH, 10 bps other top-12, 20 bps beyond the top-12) cannot be applied per pair. Spot taker stays 10 bps per side everywhere. Compared with the master:
* **r2-xsmom-cr: 20 bps flat** (prereg commit b565ada, spec e290685 on `hardening/r2-xsmom`, merged into integration as d6c94b4; orchestrator ruling 2026-11-17, before any result). Its universe is the whole pool of up to 20 pairs (D1), not a true top-12, so 20 bps is **at or above the master's schedule for every tier**: stricter for BTC/ETH (20 vs 5) and for 10 bps names (20 vs 10), equal for the 20 bps tier. **No looser case exists for xsmom-cr.**
* **r2-trend-cr-{sma,don,tsmom} and r2-mr-cr-{rsi2,zscore,ibs}: 10 bps flat** on their in-rule monthly top-12 universe (D1). Stricter for BTC/ETH (10 vs 5), equal for the other top-12 names (10 vs 10). **Looser only for a name that is in the master's 20 bps tier, i.e. outside the top-12, at the moment of a fill.** Entries cannot be in that tier: both families require the pair to be in the current month's top-12 selection, which is the master's own selection rule (highest trailing 90-bar `close x volume` among pairs listed >= 365 days), so every entry is inside the top-12 by construction. **Exits can be:** a pair that leaves the top-12 at a monthly reselection keeps its open position to the normal exit (mr-cr prereg A8; trend-cr exits when the rule turns off, regardless of membership), so the exit leg of such a position is charged 10 bps instead of the master's 20 bps. This can occur only in the months when the pool has more than 12 eligible pairs (not before 2020-04-29, when fewer than 10 pairs are eligible, and not before 2020-07-05 for a full 12), and only for pairs ranked 13 to 20, which are the pool's lower-volume names; it is one leg of a trade, 10 bps. It is **covered by the mandatory 2x-cost rerun**: 2x of the flat 10 bps is 20 bps on every fill, which equals the master's highest tier, and V7 requires the 2x rerun to stay net-positive with a positive t. No gate changes.
* Every result is reported with the 2x-cost rerun the prereg requires (V7: 2x slippage and 2x brokerage).

### D4. Long-only engine: what is and is not run
The engine is long-only, one account, one calendar, daily bars. Therefore **trend and mean-reversion books test the long side only; xsmom is a long-only top-bucket rotation.** `docs/r2run-catalogue-status.md` (generated by `tq r2run catalogue --write`) fixes the status of every family; a NOT-RUN family is **not KILL, not INCONCLUSIVE-SAMPLE, not DEV-GO**, opens no sweep and **consumes no trial cell**:

| Family | Status | Reason |
| --- | --- | --- |
| r2-trend-lc, r2-xsmom-lc, r2-mr-lc, r2-trend-cr, r2-xsmom-cr, r2-mr-cr (15 candidates, 162 cells) | RUNNABLE | long-only daily rules |
| r2-pairs-lc, r2-pairs-cr | NOT RUN: engine limitation | no short leg, no futures margin, no market-neutral accounting (pairs-cr also lacks perp klines, addendum 1 s.6) |
| r2-diag-carry | NOT RUN: engine limitation | short perp leg, margin and basis accounting absent |
| r2-diag-options-weekly | NOT RUN: engine limitation | multi-leg short-premium structures; no typed bhavcopy loader |
| r2-intraday-nse, r2-diag-intraday-cr | NOT RUN: data unavailable | no 5m NSE bars and no hourly/1m crypto bars in the development store (addendum 1 s.6) |
| r2-diag-volt-blend, r2-diag-trend-idx, r2-diag-council | NOT RUN: other | two calendars/currencies with a volatility overlay; one candidate spanning two classes with an unwired BTC-LONG proxy; council evaluator outside the sealed store |

The frozen catalogue is **not edited**; the unspent cells stay unspent. N follows master s.4.2 (`N_new_final` counts opened cells only), so skipped families lower N_new_final accordingly.

### D5. Harness workarounds recorded in the specs
* **Crypto `data_start` 2017-08-17 is explicit in the specs** because `splits::parse_master` returns the crypto train start as 2021-12-31 (the end of T); the true start is 2017-08-17 and the true crypto fold count is **32** (addendum 1 s.1.1).
* The **IN-EQ development manifest lists `MEMBERS` as a pseudo-series**; the specs exclude it from the price universe.
* **`DevReader` rejects zero-open rows** that addendum 1 s.5 says to keep and count; `r2run` parses leniently (keeps and counts such rows) **after** the content-level maximum-date check, so the date guard is not bypassed.

### D6. Closed-trade definition and V2 counts
Master s.2.5 defines a closed trade as a **flat-to-flat round trip in one instrument**. The rebalancing engine also records partial trims and top-ups as trades, which inflates the raw count. V2 therefore uses **the smaller of the engine's recorded count and the flat-to-flat count** (the stricter), before the `N_eff` haircut. The code fix is in progress in `wt/r2fix2` and **will be merged before any sweep**.

### D7. PIT members renamed or merged: alias table
Members whose price series carries the successor's ticker (TATAMOTORS to TMPV, SESAGOA and SESASTERLITE to VEDL, INFRATEL, IBULHSGFIN, MCDOWELL-N and others, addendum 1 s.5) need an **alias table**. **Unmatched members stay in the universe as UNFILLABLE** (addendum 1 s.3), never substituted or dropped, and **sweeps refuse to open while any member is unmapped and undeclared**. The table is being built in `wt/r2fix2`; **this addendum does not write alias rows**, and the table will be referenced here by path once it is merged (the path does not yet exist in this branch).

### D8. Series excluded by the data step
* **ADANIENT**: negative `adjclose` on 47 development rows (owner decision D-004): excluded (`--exclude ADANIENT`). D-004 records that it was not a NIFTY 50 member before 2022, so IN-EQ development is unaffected (membership dates were not re-checked here); if it had been a development member, its member-days would count as unfillable in the coverage figure (addendum 1 s.3).
* **JIOFIN and TMCV**: first row dated after 2021-12-31, so they have **no development rows** (they exist only in the sealed region).

### D9. Shift test unavailable for the rebalance engine
Month-end decisions cannot be delayed by one bar from outside the rebalancing engine, so **the shift test (V10) cannot be run for xsmom candidates** (r2-xsmom-lc, r2-xsmom-cr). Those candidates are evaluated **INCOMPLETE on that input**: INCOMPLETE is **not GO-DEV, not KILL for this reason alone, and not promotable**; the missing input blocks promotion until the shift test exists. No gate is waived.

### D10. Crypto eligible-symbol count
The count used for crypto books is **20** (the archived pool, addendum 1 s.1.1), not 12 as the master says in places; "12" survives only as the size of the monthly top-12 selection inside the trend and mean-reversion books (D1). The V2 diversification clause is judged on the 20-pair pool as addendum 1 fixed.

### D11. Index of the per-candidate assumptions
Each candidate prereg lists its own assumptions in its **section 10**, which this addendum only indexes: `docs/prereg-r2-trend-lc-{sma,don,tsmom}.md` and `prereg-r2-trend-cr-{sma,don,tsmom}.md` (section 10, "Author's assumptions and ambiguities"); `prereg-r2-mr-lc-{rsi2,zscore,ibs}.md` and `prereg-r2-mr-cr-{rsi2,zscore,ibs}.md` (section 10, assumptions and ambiguities resolved); `prereg-r2-xsmom-lc.md` and `prereg-r2-xsmom-cr.md` (section 10, assumptions and master ambiguities). The harness-level assumptions, limitations and findings are in `docs/r2run-notes.md`. Later candidate-level decisions must be recorded in their own preregs or in a further dated addendum, not by editing these.

## 2. Conflicts and tensions with the tagged master text (the tag is NOT edited)

1. **Top-12 rule (D1) conflicts with master s.6** ("top-12 by trailing 90-day dollar volume"). xsmom-cr applies no volume rule; the trend and mean-reversion books apply it as written. A deviation due to the engine interface, approved 2026-11-17 and recorded here. Also, master s.2.5 defines "eligible" as >= 252 days of history while the crypto books use >= 365 days listing age (master s.6): the stricter 365 governs.
2. **Flat slippage (D3) differs from master s.7.2's per-tier schedule, in one direction only for one case.** xsmom-cr (20 bps flat) is at or above every master tier: no looser case. trend-cr and mr-cr (10 bps flat) are stricter for BTC/ETH, equal for other top-12 names, and looser only for the exit leg of a position held in a pair that has left the in-rule top-12 (master tier 20 bps); that case is real but limited (D3), and covered by the 2x-cost rerun. Not a gate change.
3. **NOT RUN for engine limitations (D4).** Master s.3.4(c) provides "dropped and not counted" only for an unusable data source and says nothing about engine limits. Master s.4.7 allows Phase 1 to close when every listed candidate "has run or is declared not-run", and s.4.2 counts opened cells only, so the treatment is consistent with the master; but declaring GO-capable families (pairs, 54 cells) not-run for an engine limit rests on s.4.7, not s.3.4(c). Flagged for the reviewer. It lowers N_new_final (less deflation) versus a full run of the catalogue.
4. **Degenerate xsmom-cr cells (D2) versus OG5.** The three k = 3 cells are zero-exposure non-profitable ring-1 neighbours or grid cells of the base cell on the k axis, so OG5's 80% and 60% thresholds need the remaining cells to carry it. This can only make OG5 harder; not a gate change.
5. **`INCOMPLETE` (D9) is not an outcome of the master** (GO-DEV, INCONCLUSIVE-SAMPLE, KILL). It is stricter than GO-DEV (cannot be promoted) and is not a relaxation, but the label is new.
6. **Crypto Sharpe annualisation.** Owner decision D-004 keeps **sqrt(252) for every class including crypto** (the stricter). Addendum 1 s.4 said crypto daily Sharpe annualises on 365; **D-004 supersedes that sentence for all gates** (a move to 365 would loosen a gate). The tagged master states no crypto factor, so the tag is not contradicted.
7. **Harness `parse_master` (D5)** misreads the crypto train start; the workaround follows addendum 1 (start 2017-08-17), so there is no conflict with the master, but the harness's built-in window parser disagrees with the tagged text until fixed; the fold count used is 32 (addendum 1 s.1.1 corrects the master's 33).
8. **Lenient parsing of zero-open rows (D5)** follows addendum 1 s.5 ("rows are kept and counted"); the stricter `DevReader` behaviour is bypassed only after the content-level date check, so master s.10 item 5 (maximum-date check on every loaded bar) still holds.
9. **Alias table (D7):** refusing to open a sweep on an unmapped, undeclared member is stricter than master s.7.5 (which says only "unfillable"); no conflict.
10. **ADANIENT exclusion (D8):** a data exclusion not in the master; consistent with addendum 1 s.3 provided any development member-days count as unfillable coverage.
11. **Crypto count (D10):** the master's "12" (s.6) versus 20 (addendum 1): the 20-pair pool is the pre-registered universe; no gate changes.
12. **No conflict found** with any gate, threshold, cap, boundary or candidate id of the tagged master.

## 3. Still open
The alias-table path and the merged code fixes (`wt/r2fix2`) are not yet in this branch; the tagged ids, K = 27 for xsmom-cr and the caps are unchanged; the unverified items of addendum 1 s.8 remain unverified. Nothing in this addendum authorises a scored run: the harness review and the data step precede it (`docs/r2run-notes.md`).

# Independent review of intraday-01

Reviewer: research-review session, 2026-11-02. Scope: `engine/reports/intraday-01.md` (+ `intraday-01/*.json`),
`crates/live` (strategies, runner, stats), the Binance 1m klines (2022-2023), the NSE 5m Yahoo files, and the
live journal `data/journal/live-BTCUSDT-vwap.jsonl`. I did not edit anything under `engine/`. The only file I
downloaded is one public Binance daily kline file (checksum verified) into my scratchpad for the parity check.
Scripts: `docs/review-intraday-01-scripts/`.

## Verdict

**The conclusion stands: none of the three strategies has an edge that could pay costs, and the numbers
reproduce exactly. Four statements in the report are stronger than the evidence, and the live paper runner
is not running.**

1. **Exact reproduction.** My independent Python replica reproduces the compounding research accounts to the
   trade: BTC VWAP 16,681 trades, fees 6,747, final equity 0.82, 215,288 signals, 198,607 rejected; ETH VWAP
   20,694 trades; both momentum runs (4,035 and 4,017 trades, fees 5,822 and 6,091); and ORB per stock
   (29 / 38 / 38 / 22 / 26 trades, net P&L to the rupee). No look-ahead (section 2).
2. **The VWAP sample in the report is truncated.** The compounding research account was wiped out (BTC traded to
   2022-12-30, ETH to 2023-03-11), so the pooled VWAP figure holds only 3,097 trades from 2023 out of about
   38,700. Over the full window, with constant notional, the gross edge is **+0.30 bps (BTC) and +0.34 bps
   (ETH)**, t about +2, and **+0.56 / +0.58 bps in 2023**: small but positive, not "zero". It is still 35-65x
   too small to pay 20 bps of fees. Verdict unchanged (section 1).
3. **ORB on NSE is not informative at the level claimed.** Pooled gross is -6.9 +/- 6.6 bps (+/- 7.5 clustered by
   day); the smallest detectable edge is about 13-15 bps. 14 of 153 trades were carried overnight because Yahoo
   omits the 15:15-15:25 bars. "ORB loses before any cost" is not established (section 3).
4. **The TDS figure is right as arithmetic but misleading** (it is measured on a shrinking account); the true
   burden for an account that does not blow up is roughly 3-20x larger (section 4).
5. **Live paper trading is not running.** The journal stops at 2026-10-30 14:12 UTC (22 minutes of live data,
   1 closed trade). A fill-by-fill parity check on those 3 events matches the replay exactly (section 5).

## 1. Crypto scalps: reproduction and the truncated sample

Replica: `intraday.py` (no shared code with Rust): VWAP reversion and momentum burst as pre-registered; signal at
the bar close, fill at the next open with 5 bps adverse slippage, resting stop at `min(open, stop)`, a feed gap
cancels the pending entry, 0.5% risk / 20% notional sizing on compounding equity, 0.10% taker fee.

| Compounding research account (report vs replica) | Trades | Fees | Final equity / signals / rejected |
| --- | --- | --- | --- |
| BTC VWAP | 16,681 / **16,681** | 6,747.33 / 6,747 | 0.82 / **0.82**; 215,288 / **215,288**; 198,607 / **198,607** |
| ETH VWAP | 20,694 / **20,694** | 6,595 / 6,595 | |
| BTC momentum | 4,035 / **4,035** | 5,822 / 5,822 | |
| ETH momentum | 4,017 / **4,017** | 6,091 / 6,091 | |

**Full-window, constant-notional sample** (no compounding, so every signal over 2022-2023 is measured; gross =
raw open-to-open move, bps of entry notional):

| Run | Trades | Mean gross bps (SE; t) | Day-clustered SE (t) | 2022 / 2023 gross bps (n) | Stop exits |
| --- | --- | --- | --- | --- | --- |
| BTC VWAP | 36,130 | **+0.30** (0.14; +2.1) | 0.14 (+2.1) | -0.01 (16,863) / **+0.56** (19,267) | 68% |
| ETH VWAP | 36,999 | **+0.34** (0.17; +1.9) | 0.16 (+2.1) | +0.07 (17,589) / **+0.58** (19,410) | 69% |
| BTC momentum | 4,035 | -0.83 (0.44; -1.9) | 0.45 (-1.8) | -2.33 (1,986) / +0.61 (2,049) | 75% |
| ETH momentum | 4,017 | +0.75 (0.56; +1.3) | 0.56 (+1.3) | -0.33 (2,072) / +1.90 (1,945) | 74% |

- Momentum matches the report's pooled year split exactly (2022 -1.31, 2023 +1.24 bps) because those accounts
  were not depleted. VWAP does not: the report's pooled 2023 row has **n = 3,097**; the full 2023 sample is
  about 38,700 trades. The report's separate fresh-equity 2023 rerun (+0.54 / +0.58) is the right figure and agrees
  with mine.
- "Gross edge zero, measured precisely" should read **"gross edge +0.3 to +0.6 bps for VWAP and about 0 to +1
  for momentum, 35-65x below the 20 bps fee alone"**. Break-even cost is under 1 bp; the verdict is unaffected.
- **Design flaw, not an error:** a compounding account with 20% sizing and 30 bps friction loses about 6 bps of
  equity per trade and dies after a few thousand trades, which truncates the sample and makes the per-day HAC t
  meaningless (the report says so). Gross-edge research should use a **constant notional**; I recommend that for
  intraday-02.
- **Friction is overstated for Binance.** The runner charges 5 bps slippage per side (10 bps round trip) because it
  is a venue-agnostic convention; `docs/intraday-costs-and-access.md` assumed about 1 bp per side for BTCUSDT and
  ETHUSDT. The report's "about 30 bps per round trip" is 22 bps by that document. Irrelevant to the verdict
  (gross is about 0) but the net figures read about 8 bps worse than a realistic estimate.
- **Scope.** Three fixed, long-only rules on two coins prove nothing about scalping in general. Not tested: shorts,
  maker orders and queue position, lower-fee pairs and tiers (any current zero-fee pair promotions are unverified
  and would still face spread and Indian tax), order-book strategies, other coins.

## 2. Look-ahead and implementation checks

- Every strategy decides on a **closed** bar and fills at the **next** bar's open. VWAP includes the signal bar in
  its VWAP and 20-close window (inclusive, as documented); momentum's baselines exclude the current bar (code and
  tests). Session VWAP resets at 00:00 UTC.
- Timestamps are bar **open** times throughout (Binance and the NSE CSVs). ORB's range uses the bars opening
  09:15, 09:20 and 09:25, which close by 09:30, and signals start from the bar opening 09:30: no leakage.
  The forced flatten triggers at the first bar opening at or after 15:15.
- Data integrity: 1,051,120 bars for 2022-2023 per coin, one 80-minute gap (2023-03-24), no duplicates, no invalid
  OHLC; the loader converts the 16-digit microsecond timestamps in the 2025 files (tested), so the holdout guard
  works on milliseconds.
- Entries pass the gate at signal and again at the fill price; a fill at or below the stop is rejected; stops use
  `min(open, stop)`. All consistent with the swing engine.

## 3. NSE ORB: is the sample informative? (your question)

Replica `orb.py` reproduces the report exactly (29 / 38 / 38 / 22 / 26 trades; gross +1.0 / -5.3 / -8.3 / -4.1 /
-18.2 bps; net -8,617 / -15,995 / -18,179 / -9,039 / -17,472).

| | n | Mean gross bps | 95% CI |
| --- | --- | --- | --- |
| Pooled, as reported | 153 | -6.9 (sd 82; SE 6.6) | [-19.8, +6.1] (day-clustered SE 7.5: [-21.5, +7.8]) |
| True intraday trades only | 139 | **-11.0** (SE 6.8) | [-24.3, +2.4] |
| Carried overnight | 14 | **+34.0** (SE 23.7) | |

- **Resolving power:** the smallest edge this sample can detect at t = 2 is about **13-15 bps**, which is the size of
  the all-in cost at the adopted slippage (16 bps). It cannot see a 5-8 bps edge; the pooled 95% interval reaches
  +6 to +8 bps, which is above the cheapest NSE cost rows (4.0 bps at Rs 10 lakh, no slippage; 7.0 with 3 bps).
- **Sample:** 60 sessions (the report says about 58) in one regime, five highly correlated large caps on the same
  days: roughly 57 independent days, not 153 trades. Stops are about 30% of exits.
- **Data defects that the report does not mention:**
  - Yahoo's 5m files **omit the 15:20 bar in all 60 sessions, the 15:25 bar in 47, and the 15:15 bar in 6**. When the
    15:15 bar is missing the runner flattens at the next available bar, which can be the next morning:
    **14 of 153 trades (9%) are overnight carries** (+34 bps on average, SE 24), so the pooled figure mixes in gap
    risk and is **less negative than the true intraday result (-11.0)**.
  - The **09:15 bar has volume 0 in 58-59 of 60 sessions** (the 09:20 bar's median volume is 220,000), so the first
    bar's range (median 48-110 bps) is built from the opening print, not traded volume. ORB does not use volume,
    but the "opening range" is not what a live 5m feed would show.
- **What the report can and cannot say.** "ORB has no demonstrated gross edge and the point estimate is negative"
  is supported. "**ORB loses before any cost**" and "gross is negative, not marginally short of costs" are not:
  the pooled interval includes zero and small positive edges. The intraday-only interval [-24, +2.4] does exclude
  an edge of 4 bps or more, which is a better supported statement. Either way 58 sessions cannot settle ORB on NSE;
  months of recorded data (or one month of Kite Connect history) are needed.

## 4. After-tax view and the TDS figure (your question)

- **Arithmetic: correct.** BTC VWAP: sell turnover 3,372,037 USDT x 1% = 33,720 = 337% of the 10,000 starting
  capital, over 730 days = **168.6% a year** (all four rows follow the same formula). The after-tax per-trade gap
  (-30.0 to -32.4 bps) follows from taxing winning trades' (sale - cost) at 30% with no loss relief: 975.93 of
  tax on 16,681 trades. Cess and surcharge are excluded, as stated.
- **Misleading as a measure.** It divides by *starting* capital while the account shrank to nothing, so trade
  sizes (20% of equity) collapsed. For an account whose equity does not collapse, each round trip withholds 0.2%
  of equity (20% x 1%). At the unconstrained trade rates, VWAP BTC (about 18,000 trades a year) would withhold
  roughly **36x equity a year (about 3,600%)** and momentum BTC (about 2,000 a year) about **4x (about 400%)**,
  against the report's 169% and 145%. "1% of every sale exceeds the whole account each year" is true and
  understated.
- **The "hit rate above 59%" line is misapplied.** That threshold came from my break-even note, which assumes a
  symmetric +/-G payoff. These strategies have asymmetric payoffs (many small stop losses), so 8-16% net win rates
  are not directly comparable to it. The conclusion survives because net expectancy is about -30 bps.
- TDS on an offshore exchange, the 2025-Act numbering and Binance's current India status remain unverified
  (`intraday-costs-and-access.md`).

## 5. Live paper trading and replay-vs-live parity

- `data/journal/live-BTCUSDT-vwap.jsonl` starts 2026-10-30 13:50:57 UTC (1,500 bars backfilled) and ends
  **14:12 UTC the same day**; there is no `tq-live` process and no write for three days. So the statement that
  paper trading "is running" is not true now (machine sleep or a killed process; I did not investigate). It holds
  3 fills and 1 closed trade.
- **Parity spot check (published klines, SHA-256 verified):** replaying the same bars with signals before 13:50
  discarded gives *identical* events: entry at 13:51 (raw open 76,833.06, 0.02601 BTC, fee 1.9994), stop exit in
  the same minute at 76,679.3587 (-20.0 bps gross), second entry at 13:52 (raw 76,645.35, 0.02605 BTC). The
  replay then shows that second position exiting at 14:22 for **+41 bps**, which the live run missed by stopping.
  Three events are not a statistical parity test; a run of several days is needed.
- The live smoke test section of the report is **unfinished**: it still contains the placeholder
  `SEE_SMOKE_SECTION`.

## 6. Other observations

- The **ORB results table pairs a 60-day sample with 2022-2023 crypto in one verdict**; keep them separate when
  quoting.
- Default-limits runs (10% kill switch, 2% daily loss) end within weeks: fine, and consistent with protocol section 7.
- "Pre-registered, no other variant run" is plausible from the code and the single run per configuration; add the
  three ideas to the trials ledger (N rises by 3).
- The trade-level gross definition (raw open-to-open, unweighted mean of per-trade bps) is sound; the break-even
  claim "mean gross bps equals break-even cost" is right for a round-trip cost expressed in bps of notional.

## 7. Recommendations

1. Fix the report: replace "zero" with the measured +0.3 to +0.6 bps for VWAP, quote full-window figures from a
   constant-notional run, rewrite the ORB sentence, finish the live section, state that live trading stopped.
2. For intraday-02 use constant-notional research mode and report gross bps with day-clustered SE and 95% CI.
3. Obtain real NSE intraday history (Kite Connect for a month, or record live) and drop sessions with missing
   15:15 bars or treat them explicitly; do not carry positions overnight in an intraday test.
4. Use venue-realistic slippage (about 1 bp per side for BTC/ETH, a spread-based figure for NSE) alongside the
   conservative 5 bps.
5. Restart the live runner and, after several days, repeat the replay-vs-live parity check on every fill.

## Reproducing

`docs/review-intraday-01-scripts/` (read-only on the engine's data; run with `python3 -I`): `intraday.py`
(1m crypto replica, account and unconstrained modes, live-window replay), `intraday_run.py` (full-window
constant-notional statistics), `intraday_acct.py` (compounding-account comparison with the report), `orb.py`
(NSE ORB replica and power). The parity check used `BTCUSDT-1m-2026-10-30.csv` from
`data.binance.vision` (SHA-256 `faa0a6de98c491140035b41eb1bb0821d45ace33d34c17ec0348e70c87b6eeb4`), not stored in the repo.

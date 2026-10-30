# intraday-01: pre-registered scalping strategies, net of costs

Date: 2026-10-30. Engine: `tq-live` (commit 5f5a479 + a76b58d). **Paper only; no account, key or order
endpoint was used.** Not investment or tax advice.

## Verdict

**None of the three strategies has an edge, before or after costs.** Gross of all friction, the average
trade earns about **0 bps** (VWAP +0.1, momentum 0.0, ORB -6.9 bps); the friction is about 30 bps per
round trip on Binance (20 fees + 10 slippage) and about 16 bps on NSE. So every strategy loses roughly
its cost on every trade, with very high statistical confidence on crypto. **Break-even cost per trade is
about 0 bps** for the crypto strategies and negative for ORB, below even the cheapest venue in
`docs/intraday-costs-and-access.md` (NSE about 7 bps at Rs 10 lakh with 3 bps slippage; Binance 15 bps
with BNB). Nothing here justifies paper-trading them further except as plumbing tests; **the 2024-2025
holdout stays sealed** (no point spending it).

## Pre-registration (fixed before any run; code in `crates/live/src/strategies/`)

| | Bars | Entry (long only) | Exit | Stop |
| --- | --- | --- | --- | --- |
| (a) VWAP reversion | 1m | close < session VWAP - 2 x stdev(20 closes) | close >= VWAP, or 30 bars held | entry - 1.5 x ATR(14) |
| (b) Opening-range breakout | NSE 5m | close > high of 09:15-09:30 bars, max 1 entry/day | forced flat 15:15 IST | range low |
| (c) Momentum burst | 1m | 5-bar return > 3 x stdev of previous 100, and volume > 2 x previous 100-bar mean | 10 bars held | entry - 1 x ATR(14) |

Session VWAP resets at 00:00 UTC (crypto) / IST day (NSE). Parameters were not tuned; no other variant
was run.

## Method

- **Data.** Binance spot 1m klines from data.binance.vision, SHA-256 verified against the published
  `.CHECKSUM` (96/96 months, `scripts/fetch_binance_klines.py`); replay window **2022-01-01 to
  2023-12-31** (1,051,120 bars per coin). 2024-2025 is downloaded but `tq-live` refuses any crypto bar
  >= 2024-01-01 (tested). NSE: Yahoo 5m bars, **the only free history is the last 60 days**
  (2026-08-04 to 2026-10-29, about 58 sessions, 4,372-4,385 bars per stock) for RELIANCE, HDFCBANK,
  ICICIBANK, INFY, TCS. Yahoo's post-15:30 print is dropped by the session filter.
- **Execution (`tq-live --replay`).** Signal on the bar close, fill at the **next bar's open** with
  **5 bps adverse slippage per side**; stop exits at `min(open, stop)` when the low touches it (as
  `tq-backtest`); entries pass the risk gate at signal and at fill; sizing = 0.5% equity at risk to the stop,
  capped at 20% notional (default `RiskLimits`). Crypto fees: Binance 0.10% taker each side
  (`CryptoSpotFees`). NSE: Zerodha intraday schedule (`tq-costs`). BTC/ETH traded in 0.00001 lots.
- **Two limit settings.** *Research* (`--research`): kill switch and daily-loss stop disabled, to measure
  the strategy over the whole window. *Default*: the user's limits (2% daily loss, 10% drawdown kill
  switch). All per-trade statistics below are from research runs.
- **Statistics.** bps per trade on raw entry notional. Gross = raw open-to-open move (no fees, no
  slippage). HAC t = Newey-West t of daily account returns (`tq_backtest::tstats`, protocol lag; crypto
  730 UTC days incl. flat days, NSE 60 days). **Break-even round-trip cost = mean gross bps per trade.**

## Results (research runs; summaries in `reports/intraday-01/*.json`)

| Run | Trades | Win | Gross (USDT/Rs) | Fees | Slippage | Net | PF (net) | Gross bps/trade = break-even | Net bps/trade | HAC t (daily) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| (a) VWAP BTCUSDT | 16,681 | 13% | 122 | 6,747 | 3,374 | -9,999 | 0.15 | -0.0 | -30.0 | -9.7 |
| (a) VWAP ETHUSDT | 20,694 | 16% | -108 | 6,595 | 3,297 | -9,999 | 0.22 | +0.1 | -29.9 | -11.5 |
| (c) Momentum BTCUSDT | 4,035 | 8% | -435 | 5,822 | 2,911 | -9,169 | 0.11 | -0.8 | -30.8 | -36.1 |
| (c) Momentum ETHUSDT | 4,017 | 11% | 89 | 6,091 | 3,045 | -9,047 | 0.17 | +0.7 | -29.3 | -34.8 |
| (b) ORB RELIANCE | 29 | 34% | 581 | 3,420 | 5,779 | -8,617 | 0.65 | +1.0 | -14.9 | -1.0 |
| (b) ORB HDFCBANK | 38 | 34% | -4,006 | 4,464 | 7,524 | -15,995 | 0.50 | -5.3 | -21.2 | -2.0 |
| (b) ORB ICICIBANK | 38 | 37% | -6,267 | 4,443 | 7,469 | -18,179 | 0.42 | -8.3 | -24.2 | -2.2 |
| (b) ORB INFY | 22 | 27% | -2,153 | 2,567 | 4,319 | -9,039 | 0.59 | -4.1 | -20.0 | -1.0 |
| (b) ORB TCS | 26 | 38% | -9,459 | 3,003 | 5,010 | -17,472 | 0.37 | -18.2 | -34.3 | -2.2 |

Crypto starts at 10,000 USDT, NSE at Rs 10,00,000. Pooled per-trade gross (naive t, trades not independent):

| Strategy | Trades | Mean gross bps | t (gross) | Mean net bps | 2022 gross | 2023 gross |
| --- | --- | --- | --- | --- | --- | --- |
| (a) VWAP, BTC+ETH | 37,375 | +0.08 | 0.4 | -29.9 | +0.04 (n 34,278) | +0.56 (n 3,097; fresh-equity 2023 rerun: +0.54 BTC, +0.58 ETH) |
| (c) Momentum, BTC+ETH | 8,052 | -0.04 | -0.1 | -30.0 | -1.31 (t -2.2) | +1.24 (t 3.0) |
| (b) ORB, 5 NSE stocks | 153 | -6.86 | -1.0 | -22.8 | (60 days only) | |

Reading:
- **The gross edge is zero, measured precisely.** 37k VWAP trades put the mean gross within about
  +/-0.4 bps of zero; the cost is 30 bps. The momentum sign flips between years (-1.3 then +1.2 bps), which
  is noise or regime, and at ~1 bp it is 25x too small to pay Binance fees in any case.
- **Stops dominate.** 65-75% of crypto exits are stops: a 1-1.5 ATR stop on a 1m bar sits inside ordinary
  noise, and each stop pays the full round trip.
- **Research accounts go to about -100%**: about 0.2 x 30 bps lost per trade, compounded over thousands of
  trades. Once VWAP equity was nearly gone, one lot's notional exceeded 20% of equity and later entries were
  rejected ("position notional"); that truncated the 2023 VWAP sample, so a fresh-equity 2023 rerun is
  shown, and it agrees. The HAC t values are strongly negative, partly because returns are measured on a
  shrinking account; they are only quoted to show the direction.
- **ORB:** 153 trades over 60 days in one regime cannot establish anything (the t-stat has no power at
  this n); what can be said is that gross is negative, not marginally short of costs. Proper NSE history
  needs a paid source (Kite Connect, Rs 500/month) or months of recording.

### Default limits (what the paper account would actually do)

| Run | Trades | Return | Kill switch engaged |
| --- | --- | --- | --- |
| VWAP BTC / ETH | 175 / 183 | -10.0% / -10.1% | 2022-01-07 / 2022-01-06 |
| Momentum BTC / ETH | 153 / 197 | -9.9% / -9.9% | 2022-01-29 / 2022-02-07 |
| ORB (each NSE stock) | same as research | -0.9% to -1.8% | not reached |

The 2% daily-loss stop rejected thousands of further entries (e.g. 394k VWAP signals on BTC), and the 10%
drawdown kill switch ended every crypto strategy within its first six weeks. The risk gate works as designed.

## Break-even versus venue costs (`docs/intraday-costs-and-access.md`)

| Venue (round trip) | Cost bps | VWAP gross | Momentum gross | ORB gross |
| --- | --- | --- | --- | --- |
| Binance, 0.10% each side | 20 (+ slippage) | +0.1 | 0.0 | n/a |
| Binance with BNB discount | 15 (+ slippage) | +0.1 | 0.0 | n/a |
| NSE Zerodha, Rs 50k / 2L / 10L, no slippage | 10.7 / 5.9 / 4.0 | | | -6.9 |
| NSE with 3 bps slippage | 13.7 / 8.9 / 7.0 | | | -6.9 |
| NSE with adopted 5 bps/side | 20.7 / 15.9 / 14.0 | | | -6.9 |

No strategy clears even the cheapest row. ORB loses before any cost.

## Indian-resident after-tax view (crypto): simplified illustration, NOT tax advice

Assumptions: 30% tax (s.115BBH) on each winning trade's (sale - cost of acquisition); fees are not
deductible; losses are never offset (the conservative per-trade reading); no 4% cess or surcharge (both
would make it worse). Plus the 1% TDS (s.194S; s.393(1) of the 2025 Act from 2026-04-01) withheld from
every sale: a credit against tax, not a cost, but cash out of the trading account until refunded.

| Run | Pre-tax net bps/trade | After-tax net bps/trade | TDS withheld per year, % of starting capital |
| --- | --- | --- | --- |
| VWAP BTCUSDT | -30.0 | -32.4 | 169% |
| VWAP ETHUSDT | -29.9 | -33.1 | 165% |
| Momentum BTCUSDT | -30.8 | -32.5 | 145% |
| Momentum ETHUSDT | -29.3 | -31.6 | 152% |

Losing strategies still pay tax on their winners: about 2-3 bps a trade worse. The TDS column is the
binding problem for any scalper: at this turnover (20% of equity per trade, thousands of trades), 1%
of every sale exceeds the whole account each year. With `--tds`, the runner keeps that cash as a
receivable, and entries are refused once cash runs out. Per the analyst's note, the hit rate needed
after tax is above about 59% at any trade size; these strategies win 8-16% of trades.

## Live paper smoke test (Binance public websocket, BTCUSDT)

Command: `caffeinate -dims ./target/release/tq-live --live binance:BTCUSDT --strategy vwap --minutes 10`
(default risk limits, 10,000 USDT, 0.10% fee, 5 bps slippage). Run 2026-10-30 13:05:15-13:16:00 UTC.
Journal: `data/journal/smoke-live-BTCUSDT-vwap-10min.jsonl` (gitignored); summary
`reports/intraday-01/live-smoke-vwap-10min.json`.

- Warm-up: 1,500 closed 1m bars over public REST, all tagged backfill and stale, so their 544 entry
  signals were journaled as `suppressed: stale bar` and never traded.
- Live: **11 consecutive closed klines from the websocket (13:05-13:15), no gap, no duplicate**;
  2 entry signals, 2 next-open fills, 1 stop exit; one position still open at the deadline.
  Net -7.98 USDT on the closed trade (gross -1.99, fees 3.99, slippage 2.00), equity 9,989.48.

Excerpt (decimals shortened):

```
{"type":"status","ts_iso":"2026-10-30T13:05:16Z","msg":"warm-up: 1500 closed bars backfilled over REST"}
{"type":"status","ts_iso":"2026-10-30T13:05:17Z","msg":"websocket connected: wss://stream.binance.com:9443/ws/btcusdt@kline_1m"}
{"type":"signal","ts_iso":"2026-10-30T13:05:00Z","kind":"enter","entry":"77366.01","stop":"77288.89","action":"queued for next open"}
{"type":"fill","ts_iso":"2026-10-30T13:06:00Z","side":"buy","reason":"entry","qty":"0.02583","raw_price":"77366.01","price":"77404.69","fee":"1.9994"}
{"type":"mark","ts_iso":"2026-10-30T13:06:00Z","close":"77300.03","equity":"9995.30","units":"2583"}
{"type":"fill","ts_iso":"2026-10-30T13:10:00Z","side":"sell","reason":"stop","qty":"0.02583","raw_price":"77288.89","price":"77250.25","fee":"1.9954"}
{"type":"trade","ts_iso":"2026-10-30T13:10:00Z","reason":"stop","gross":"-1.99","fees":"3.99","slippage":"2.00","net":"-7.98"}
{"type":"fill","ts_iso":"2026-10-30T13:11:00Z","side":"buy","reason":"entry","qty":"0.02584","raw_price":"77282.63","price":"77321.27"}
{"type":"mark","ts_iso":"2026-10-30T13:15:00Z","close":"77300.36","equity":"9989.48","units":"2584"}
{"type":"end","ts_iso":"2026-10-30T13:16:00Z"}
```

**Reconnect evidence (unplanned).** An earlier attempt ran while the Mac kept going to system sleep
(`pmset -g log`), from 08:31 to 11:33 UTC (`data/journal/smoke-live-BTCUSDT-momentum-sleep.jsonl`). On each
wake the dead socket was detected (read timeout, os error 35; once a reset, os error 54), the feed
reconnected within 1-11 s **6 times**, and backfilled the missed bars over REST (62, 60 and 44 bars); three
backfills failed right after wake (network not yet up) and were made up at the next reconnect. Result:
**1,669 contiguous 1m bars, zero gaps, zero duplicates**, and nothing traded on a backfilled bar (all 11
signals suppressed as stale). It also exposed a bug, now fixed (a76b58d): `--minutes` used `Instant`,
which stops during macOS sleep, so a 10-minute run lasted 3 hours. **Long runs need the machine awake:
use `caffeinate -dims` on AC power, or a server.** Known cosmetic issue: stop prices are not rounded to
the exchange tick (0.01 USDT), so journal stops carry long decimals.

## Reproduce

```sh
python3 scripts/fetch_binance_klines.py && .venv/bin/python scripts/fetch_nse_intraday.py
cargo build --release -p tq-live
./target/release/tq-live --replay data/intraday/BTCUSDT --strategy vwap --from 2022-01-01 --to 2024-01-01 --research --summary out.json
./target/release/tq-live --replay data/intraday-nse/5m/TCS.csv --session nse --interval-ms 300000 --cash 1000000 --strategy orb --research
```

Limitations: one symbol per process; account state is not restored from the journal on restart; Yahoo NSE
bars are unofficial and unadjusted; 5 bps slippage is a convention, not a measured fill; the after-tax
view is a simplified reading of the law.

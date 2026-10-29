# Backtest 01: baseline, current NIFTY 50, development window, net of costs

Date: 2026-10-30. Author: research engineer. Status: **baseline. No parameter was tuned on this run.**
Both strategies use their `Default` configs from `tq-strategies`. Every number below is in-sample
(development window 2012-01-01 to 2021-12-31); no holdout data (2022 onward) was downloaded or loaded.

## Verdict

**Both strategies FAIL the validation protocol as it stands. Neither is eligible for paper trading.**

- **TrendPullback: no edge after costs.** 3,154 trades, mean net trade +0.06%, t = 0.93, profit
  factor 1.03. Costs consume 86% of gross P&L. FAIL on every edge gate.
- **DonchianBreakout: good trade statistics on a biased sample.** 882 trades, mean net trade +3.71%,
  t = 3.73, profit factor 1.94, +0.57R. But the median per-symbol Sharpe is 0.16, one symbol
  (ADANIENT) makes 22% of net P&L, the 5% kill switch tripped in 21 of 49 accounts, and the
  universe is today's winners (see the caveat). It is in-sample, not walk-forward. Trend following
  is the style survivorship flatters most, so treat these numbers as an upper bound until backtest-02.

## Commands

```sh
python3.11 -m venv .venv && .venv/bin/pip install yfinance          # yfinance 1.7.0
.venv/bin/python scripts/fetch_daily.py --tickers-file universe/nifty50-2026-10-30.txt \
    --start 2011-01-01 --end 2021-12-31
cargo build --release -p tq-cli
./target/release/tq quality --data-dir data/daily --symbols @universe/nifty50-2026-10-30.txt
./target/release/tq backtest --strategy trend-pullback --data-dir data/daily \
    --symbols @universe/nifty50-2026-10-30.txt --start 2012-01-01 --end 2021-12-31 \
    --cash 1000000 --risk-pct 0.5 --out reports/backtest-01/trend-pullback.md
./target/release/tq backtest --strategy donchian --data-dir data/daily \
    --symbols @universe/nifty50-2026-10-30.txt --start 2012-01-01 --end 2021-12-31 \
    --cash 1000000 --risk-pct 0.5 --out reports/backtest-01/donchian.md
# Supplementary measurement only (kill switch off, see below): same commands plus
#   --max-drawdown-pct 100 --out reports/backtest-01/<strategy>-no-killswitch.md
```

Full per-symbol tables: [trend-pullback](backtest-01/trend-pullback.md), [donchian](backtest-01/donchian.md),
[trend-pullback, no kill switch](backtest-01/trend-pullback-no-killswitch.md),
[donchian, no kill switch](backtest-01/donchian-no-killswitch.md).

## Data

- **Source:** Yahoo Finance via `yfinance` 1.7.0 (`scripts/fetch_daily.py`, `auto_adjust=True`, so
  prices are split, bonus and dividend adjusted by Yahoo), fetched **2026-10-30**. Unofficial data
  (see `scripts/README.md`); not cross-checked against NSE bhavcopy.
- **Range fetched:** 2011-01-01 to 2021-12-31. 2011 bars only warm up indicators (`RunConfig::trade_from`).
- **Universe:** the **current** NIFTY 50 list from NSE Indices,
  <https://www.niftyindices.com/IndexConstituent/ind_nifty50list.csv>, fetched 2026-10-30 and saved as
  `universe/nifty50-2026-10-30.csv` (with ISINs) and `universe/nifty50-2026-10-30.txt` (tickers).
  49 of 50 symbols had data. JIOFIN has none (listed 2023). Six have short histories: ETERNAL
  (from 2021-07-23, 111 bars, too short to warm up the 200-day SMA), MAXHEALTH (2020-08-21), BSE
  (2017-02-03), HDFCLIFE (2017-11-17), SBILIFE (2017-10-03) and INDIGO (2015-11-10).

> **SURVIVORSHIP BIAS CAVEAT.** This universe is the list of companies in the NIFTY 50 *today*
> (2026). Applied to 2012-2021, it contains only stocks that went on to succeed, and leaves out every
> name that was dropped, merged or delisted along the way. It also includes stocks such as ADANIENT,
> TRENT and BEL that were not index members for most of the period and earned their place by
> rallying. **These results are therefore biased upward**, most of all for a trend-following
> strategy that buys stocks that went up. Read them as an upper bound, not an estimate. backtest-02
> repeats the run on point-in-time membership.

## Engine assumptions

- Signals on the close; fills at the **next bar's open**; stop exits at `min(open, stop)`.
- **Costs:** Zerodha NSE delivery schedule (`tq-costs`), both sides, plus **5 bps slippage per side**
  (new `RunConfig::slippage_bps`, protocol rule 0.3).
- **Sizing:** every entry is resized at the fill price to risk **0.5% of current equity** to the
  stop (`tq_risk::size_for_risk`), then cut down to the gate's 20% notional cap rather than rejected.
  0.5% is a CEO placeholder; the user has not chosen it.
- **Risk gate:** default placeholder limits (1% max risk per trade, 20% notional, 2% daily loss,
  **5% drawdown kill switch**, which latches for the rest of the run).
- Each symbol is an independent Rs 10 lakh account. There is no portfolio-level run yet, so
  "Sharpe" below is per symbol, on an account that is at most 20% invested.

## Results

| Metric | TrendPullback | Donchian | TrendPullback, no kill switch* | Donchian, no kill switch* |
| --- | ---: | ---: | ---: | ---: |
| Symbols run / with trades / kill switch hit | 49 / 48 / 2 | 49 / 49 / **21** | 49 / 48 / 0 | 49 / 49 / 0 |
| Closed trades | 3,154 | 882 | 3,222 | 1,053 |
| Win rate | 60.4% | 33.9% | 60.4% | 33.0% |
| Mean net trade | **+0.06%** | **+3.71%** | +0.06% | +3.23% |
| Mean R (net) | +0.01R | +0.57R | +0.01R | +0.49R |
| Profit factor (net) | 1.03 | 1.94 | 1.03 | 1.80 |
| t-stat, net trade returns | **0.93** | **3.73** | 0.90 | 3.80 |
| Gross / costs / net P&L, Rs lakh (sum of 49 accounts) | 8.17 / 7.02 / 1.15 | 28.45 / 2.08 / 26.37 | 8.20 / 7.16 / 1.03 | 29.45 / 2.45 / 27.00 |
| Costs as % of gross P&L | **86.0%** | 7.3% | 87.4% | 8.3% |
| Median per-symbol Sharpe (annualised) | 0.08 | 0.16 | 0.08 | 0.26 |
| Median per-symbol max drawdown | 2.37% | 4.63% | 2.37% | 4.63% |
| Largest symbol share of net P&L | JSWSTEEL 49.9% | ADANIENT 22.2% | JSWSTEEL 55.3% | ADANIENT 21.4% |
| Net P&L without best 5 trades, Rs lakh | +0.24 | +16.78 | +0.13 | +17.41 |

\* The kill-switch-off columns are a **measurement**, not a candidate configuration. Under the
placeholder 5% drawdown kill switch, Donchian stops trading for good in 21 of 49 accounts, which
cuts the trade sample short after losing streaks. Turning the switch off shows the trade statistics
are not an artefact of that truncation: they barely move.

Reading the numbers:

- **TrendPullback** trades often (about 6.6 trades per symbol per year) and wins 60% of the time, but
  its average trade is tiny, so delivery costs and slippage take 86% of the gross P&L. Half of the
  small net P&L that remains comes from one symbol (JSWSTEEL).
- **Donchian** wins a third of the time with large winners; costs are minor (7%). The t-stat clears
  3.0 on pooled trades, but trade returns across symbols are not independent (big trends are
  market-wide), so the effective sample is smaller than 882. The Sharpe of an account that is mostly
  in cash is low by construction, so a portfolio-level run is needed before the Sharpe gate means much.
- ADANIENT alone contributes Rs 5.86 lakh to Donchian (+58.6% on its account). A run ending
  2019-12-31 gives Rs 1.18 lakh, so about 80% came in 2020-21, the Adani rally. That is exactly the
  stock a today's-constituents list selects.

## Gate-by-gate (docs/validation-protocol.md)

"In-sample" means measured on the whole development window, not on the stitched walk-forward
out-of-sample curve the protocol requires. An in-sample PASS is necessary, not sufficient.

| # | Gate | Threshold | TrendPullback | Donchian |
| --- | --- | --- | --- | --- |
| 0.1 | Point-in-time, survivorship-safe universe; adjusted prices | required | **FAIL** (today's constituents) | **FAIL** (today's constituents) |
| 0.2a | Fill at next bar's open | required | PASS | PASS |
| 0.2b | Shift test (one extra bar of delay must not improve results) | required | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 0.3 | Full costs + 5 bps slippage per side | required | PASS | PASS |
| 0.4 | Trials ledger | required | PASS (`research/trials.csv`, created now, N = 2) | PASS |
| 0.5 | Holdout touched once | required | PASS (not touched; CLI guard enforces) | PASS |
| 2 | Walk-forward, 14 folds, purge/embargo | required | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 3a | Closed trades, development window | >= 200 | PASS (3,154) | PASS (882) |
| 3b | Closed trades, stitched walk-forward OOS | >= 100 | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 3c | Distinct symbols | >= 10 | PASS (48 with trades) | PASS (49) |
| 3d | No symbol > 15% of net P&L | <= 15% | **FAIL** (JSWSTEEL 49.9%) | **FAIL** (ADANIENT 22.2%) |
| 3e | Net P&L > 0 without best 5 trades | > 0 | PASS, barely (+Rs 0.24 lakh) | PASS (+Rs 16.78 lakh) |
| 4a | Mean net return per trade | >= 0.5% | **FAIL** (0.06%) | PASS in-sample (3.71%) |
| 4b | Expectancy | >= +0.20R | **FAIL** (+0.01R) | PASS in-sample (+0.57R) |
| 4c | t-stat of mean net trade return | >= 3.0 | **FAIL** (0.93) | PASS in-sample (3.73; trades not independent) |
| 4d | Profit factor (net) | >= 1.3 | **FAIL** (1.03) | PASS in-sample (1.94) |
| 4e | Annualised Sharpe, daily equity, net | >= 0.8 OOS | **FAIL** in-sample (median per-symbol 0.08) | **FAIL** in-sample (median per-symbol 0.16); portfolio Sharpe NOT-YET-MEASURED |
| 4f | Beats NIFTY 50 TRI buy-and-hold (Sharpe and Calmar) | required | NOT-YET-MEASURED (no TRI data yet) | NOT-YET-MEASURED |
| 4g | Holdout consistency | required | NOT-YET-MEASURED (by design) | NOT-YET-MEASURED (by design) |
| 5a | Parameter neighbourhood +/-20% | >= 70% profitable | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 5b | Cost stress (2x fees and slippage) keeps expectancy > 0 | > 0 | NOT-YET-MEASURED (very likely FAIL: costs are already 86% of gross) | NOT-YET-MEASURED |
| 5c | Fold stability | >= 60% folds positive | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 5d | Random-entry baseline | > 95th percentile | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 6 | Deflated Sharpe; PBO | DSR >= 0.95 | NOT-YET-MEASURED | NOT-YET-MEASURED |
| 7 | Monte Carlo: 95th pct MDD <= 0.8 x kill switch | <= 4% | NOT-YET-MEASURED (2/49 accounts hit 5%) | NOT-YET-MEASURED, likely FAIL (21/49 accounts hit 5%) |

## Data quality anomalies (`tq quality`, all 49 files)

| Anomaly | Symbols | Assessment |
| --- | --- | --- |
| **Corrupt bar 2015-06-03**: open 294.19 after a 91.95 close, low 54.57, close 56.32 (-38.7%) | ADANIENT | Bad print plus a demerger (from memory: Adani Transmission/Power) that Yahoo did not adjust. Inflates ATR for about 20 bars. It does not change ADANIENT's Donchian trades (runs ending 2015-06-02 and 2015-12-31 give identical trades) but it does sit inside TrendPullback's indicators. Not patched. |
| Phantom zero-volume flat bars on **2014-04-24** and **2014-10-15** (NSE closed for elections) | all 48 files covering 2014 | Yahoo filler rows (O=H=L=C=previous close, volume 0). Each adds a fake flat day to the indicators. Minor. |
| Other zero-volume flat bars | ADANIPORTS 2012-01-16, 2015-08-12; BAJAJFINSV 2011-08-18; JSWSTEEL 2011-08-18, 2014-02-19, 2015-08-12, 2018-11-07, 2021-11-04; MARUTI 2015-08-12; SHRIRAMFIN 2015-08-12, 2015-09-15, 2015-10-16, 2015-11-30 | 2018-11-07 and 2021-11-04 are Diwali Muhurat sessions (real but tiny). The rest look like missing data filled flat. Unverified. |
| 6-day gap 2014-10-01 to 2014-10-07 | all files | NSE holiday cluster. Benign. |
| >20% close-to-close moves, plausibly real | 2020-03-23 COVID crash: AXISBANK -27.9%, BAJFINANCE -23.2%, BAJAJFINSV -25.9%, SHRIRAMFIN -22.2%; INFY 2013-04-12 -21.3% (results); SBIN 2017-10-25 +27.7% (PSU bank recapitalisation); TMPV 2021-10-13 +20.4% (EV-unit investment news) | Consistent with events I know of (from memory, not cross-checked against NSE). |
| >20% moves, unverified | ADANIENT 2011-07-29 -21.4%, 2014-04-10 +22.9%, 2017-04-26 -20.9%, 2018-09-06 +20.9%, 2019-05-20 +27.4%, 2020-08-25 +23.7%; BAJAJFINSV 2011-03-03 +20.0% | Check against NSE corporate actions and bhavcopy before trusting. |
| Ticker history spliced across corporate events | TMPV (history is pre-demerger **Tata Motors** as a whole); SHRIRAMFIN (history is **Shriram Transport Finance**) | Yahoo maps the old series to the new ticker. Whether the TMPV history is adjusted for the 2025 demerger is unverified. |
| No data or short history | JIOFIN (none); ETERNAL, MAXHEALTH, BSE, HDFCLIFE, SBILIFE, INDIGO | Listing dates, not errors. |

## What this does and does not tell us

- It does show that TrendPullback with default parameters has no edge after Indian delivery costs on
  large caps. Tuning is unlikely to rescue a 0.06% mean trade without fitting noise.
- It does not show that Donchian works. The trade statistics are good, but the sample is survivor-biased,
  in-sample and concentrated, and drawdowns trip the placeholder kill switch in 43% of accounts.

## Next

1. backtest-02: the same run on point-in-time NIFTY 50 membership (entries only while a member).
2. A portfolio-level run (one account, many symbols) so Sharpe, drawdown and the kill switch are measured
   on what we would actually trade, against NIFTY 50 TRI buy-and-hold.
3. Walk-forward folds, shift test, cost stress and Monte Carlo (protocol sections 2, 5 and 7).

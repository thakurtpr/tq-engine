# Backtest 02: point-in-time NIFTY 50, development window, net of costs

Date: 2026-10-30. Author: research engineer. Same strategies, default parameters, costs, slippage,
sizing and risk gate as [backtest-01](backtest-01.md). The only intended change is the **universe**:
every stock that was in the NIFTY 50 at any time in 2012-2021, and **entries only while the stock was a
member** (the snapshot in effect on the fill date). Positions opened while a member may still exit
after removal. No parameter was tuned. Holdout not touched.

## Verdict

**Removing most of the survivorship bias removes most of Donchian's edge. Both strategies FAIL.**

- **Donchian:** mean net trade 3.71% -> **1.05%**, t-stat 3.73 -> **2.41** (below 3.0), profit factor
  1.94 -> **1.32**, net P&L Rs 26.4 lakh -> **Rs 9.8 lakh**. Still positive, but it no longer clears the
  t-stat, Sharpe or concentration gates, and only just clears the R and PF gates.
- **TrendPullback:** from barely positive to **slightly negative** net (mean net trade 0.02%,
  t = 0.33, PF 1.00). Costs are 101% of gross P&L. No edge.

## Commands

```sh
# Reference data (copied unchanged from ../docs/data; built from Wikipedia + 4 NSE press releases):
#   data-ref/nifty50_members_by_date.csv, nifty50_changes.csv, nifty50_symbol_notes.csv
# Price files for every symbol ever in the index (95), plus current tickers for renamed companies:
tail -n +2 data-ref/nifty50_members_by_date.csv | cut -d, -f2 | sort -u > all95.txt   # scratch file
.venv/bin/python scripts/fetch_daily.py --tickers-file all95.txt --start 2011-01-01 --end 2021-12-31 --out data/daily-pit
.venv/bin/python scripts/fetch_daily.py INDUSTOWER SAMMAANCAP UNITDSPR LTM --start 2011-01-01 --end 2021-12-31 --out data/daily-pit
for s in trend-pullback donchian; do
  ./target/release/tq backtest --strategy $s --data-dir data/daily-pit \
      --universe-file data-ref/nifty50_members_by_date.csv --aliases data-ref/nifty50_price_aliases.csv \
      --start 2012-01-01 --end 2021-12-31 --cash 1000000 --risk-pct 0.5 --out reports/backtest-02/$s.md
done
# Control: backtest-01's universe on this run's data (separates universe from data noise):
#   --symbols @universe/nifty50-2026-10-30.txt --out reports/backtest-02/$s-current50-same-data.md
# Measurement only: add --max-drawdown-pct 100 --out reports/backtest-02/$s-no-killswitch.md
```

Per-symbol tables: [donchian](backtest-02/donchian.md), [trend-pullback](backtest-02/trend-pullback.md);
the no-kill-switch variants and current-50 controls are alongside them in `reports/backtest-02/`.

## Universe and data coverage

- **Membership:** 31 snapshots, 2012-01-01 to 2026-09-30 (`data-ref/nifty50_members_by_date.csv`).
  A snapshot holds from its date until the next. 81 symbols were members at some point in 2012-2021.
  Source: Wikipedia's replacement table, walked back from today's list. **Only 4 change dates are
  verified against NSE press releases**; the rest come from a secondary source (`../docs/data/README.md`).
- **Prices:** Yahoo Finance via yfinance 1.7.0, fetched 2026-10-30, same script and adjustment as backtest-01.
- **Renamed companies** are mapped to today's ticker (`data-ref/nifty50_price_aliases.csv`):
  SESAGOA and SESASTERLITE -> VEDL, TATAMOTORS -> TMPV, INFRATEL -> INDUSTOWER, IBULHSGFIN -> SAMMAANCAP,
  MCDOWELL-N -> UNITDSPR. A renamed company is one price series and one position, entered only while
  any of its symbols was a member.

**Member-days with price data: 117,614 of 123,250 = 95.43%** (50 members x 2,465 trading days).
The other 4.57% of member-days are the survivorship bias still left in this test.

### Symbols I could not fetch

| Symbol | Company | Member-days missing (2012-2021) | Why |
| --- | --- | ---: | --- |
| HDFC | HDFC Ltd | 2,465 (the whole window) | Merged into HDFC Bank 2023; Yahoo returns no rows |
| CAIRN | Cairn India | 1,044 | Merged into Vedanta; delisted |
| IDFC | IDFC Ltd | 836 | Merged into IDFC First Bank; delisted |
| JPASSOCIAT | Jaiprakash Associates | 553 | No rows from Yahoo (listing status not checked) |
| RANBAXY | Ranbaxy Laboratories | 553 | Merged into Sun Pharma 2015; delisted |
| STER | Sterlite Industries | 185 | Merged into Sesa Goa 2013; delisted |
| JIOFIN, LTIM | Jio Financial, LTIMindtree | 0 | Members only after 2021, outside the window; not needed |

Six more symbols failed under their historical names but are covered through the alias map
(SESAGOA, SESASTERLITE, TATAMOTORS, INFRATEL, IBULHSGFIN, MCDOWELL-N). INDUSTOWER and SAMMAANCAP start
at their listings (2012-12-28 and 2013-07-23); neither was a member before then.

What the gap means: five of the six missing names (Cairn, IDFC, Jaiprakash, Ranbaxy, Sterlite) left the
index after weak performance or in mergers, so leaving them out probably still flatters results a
little. HDFC Ltd is the opposite: a large, steady member for the whole decade. The net direction of
the remaining bias is uncertain, but it is small next to backtest-01's.

## Side by side

| Metric | 01 TrendPullback (current 50) | **02 TrendPullback (PIT)** | 01 Donchian (current 50) | **02 Donchian (PIT)** |
| --- | ---: | ---: | ---: | ---: |
| Price series run | 49 | 75 | 49 | 75 |
| Symbols with trades / kill switch hit | 48 / 2 | 73 / 1 | 49 / 21 | 75 / 19 |
| Closed trades | 3,154 | 3,153 | 882 | 940 |
| Win rate | 60.4% | 60.7% | 33.9% | 31.0% |
| Mean net trade | 0.06% | **0.02%** | 3.71% | **1.05%** |
| Mean R (net) | +0.01R | **0.00R** | +0.57R | **+0.21R** |
| Profit factor (net) | 1.03 | **1.00** | 1.94 | **1.32** |
| t-stat, net trade returns | 0.93 | **0.33** | 3.73 | **2.41** |
| Gross / costs / net P&L, Rs lakh | 8.17 / 7.02 / 1.15 | 7.16 / 7.24 / **-0.08** | 28.45 / 2.08 / 26.37 | 11.98 / 2.17 / **9.81** |
| Costs as % of gross P&L | 86.0% | **101.1%** | 7.3% | **18.1%** |
| Median per-symbol Sharpe | 0.08 | -0.03 | 0.16 | 0.05 |
| Median per-symbol max DD | 2.37% | 1.70% | 4.63% | 3.70% |
| Largest symbol share of net P&L | JSWSTEEL 49.9% | n/a (net < 0) | ADANIENT 22.2% | MARUTI 15.5% |
| Net P&L without best 5 trades, Rs lakh | +0.24 | -0.91 | +16.78 | +6.07 |

Kill switch off (measurement only): 02 Donchian 1,085 trades, mean 0.98%, t 2.43, PF 1.29; 02
TrendPullback is unchanged (its one kill-switch hit came after its last trade). The conclusions do
not depend on the placeholder kill switch.

Per-symbol Sharpe in 02 includes long non-member stretches spent in cash (each series runs the whole
window), so it is diluted further. It is not a portfolio Sharpe.

**Controls.**
- *Data noise:* backtest-01's universe re-run on this run's download gives the same trade counts and
  P&L within Rs 200. Yahoo returns slightly different adjusted prices on re-fetch (2.0% of rows differ,
  by at most 0.13% of the close), but that is immaterial here. The 01 -> 02 change is the universe.
- *Where the drop comes from:* in 02, the 40 current constituents (traded only while members) make
  Rs 9.28 lakh for Donchian, and the 35 former members make Rs 0.54 lakh over 284 trades. So most of
  the drop from Rs 26.37 lakh comes from the membership rule: Donchian can no longer buy stocks such as
  ADANIENT, TRENT and BEL during the rallies that later earned them an index place. TrendPullback:
  current members +Rs 0.43 lakh, former members -Rs 0.51 lakh.

## Gate-by-gate (docs/validation-protocol.md), backtest-02

In-sample means the whole development window, not walk-forward OOS. Gates not listed are unchanged
from backtest-01 (all NOT-YET-MEASURED: shift test, walk-forward, robustness, DSR/PBO, Monte Carlo, benchmark).

| # | Gate | Threshold | TrendPullback | Donchian |
| --- | --- | --- | --- | --- |
| 0.1 | Point-in-time, survivorship-safe universe | required | **FAIL, much improved** (95.43% of member-days covered; membership mostly from a secondary source) | same |
| 0.2a / 0.3 / 0.5 | Next-open fills; costs + 5 bps; holdout untouched | required | PASS | PASS |
| 0.4 | Trials ledger | required | PASS (`research/trials.csv`, 4 rows, 2 strategies) | PASS |
| 3a | Closed trades, development window | >= 200 | PASS (3,153) | PASS (940) |
| 3c | Distinct symbols | >= 10 | PASS (73) | PASS (75) |
| 3d | No symbol > 15% of net P&L | <= 15% | n/a (net P&L < 0) | **FAIL, marginal** (MARUTI 15.45%) |
| 3e | Net P&L > 0 without best 5 trades | > 0 | **FAIL** (-Rs 0.91 lakh) | PASS (+Rs 6.07 lakh) |
| 4a | Mean net trade | >= 0.5% | **FAIL** (0.02%) | PASS in-sample (1.05%) |
| 4b | Expectancy | >= +0.20R | **FAIL** (0.00R) | PASS in-sample, marginal (+0.21R) |
| 4c | t-stat | >= 3.0 | **FAIL** (0.33) | **FAIL** (2.41) |
| 4d | Profit factor | >= 1.3 | **FAIL** (1.00) | PASS in-sample, marginal (1.32) |
| 4e | Sharpe | >= 0.8 | **FAIL** (median per-symbol -0.03) | **FAIL** (median per-symbol 0.05; portfolio NOT-YET-MEASURED) |

## Data anomalies new in this run (`tq quality` on the 36 symbols not in backtest-01)

- **Re-fetch instability:** two Yahoo downloads 15 minutes apart differ on 2,379 of 121,821 overlapping
  rows (2.0%), by at most 0.13% of the close (BEL 2014-02-12). Results are unaffected (see Controls),
  but backtests are reproducible only if the exact data files are archived. Suggest a checksummed
  snapshot of `data/` per report.
- **Zero-volume flat rows** (on top of the 2014-04-24 and 2014-10-15 election-holiday phantoms seen in
  every file): BRITANNIA, DLF and RELINFRA on 2011-08-18, 2014-02-19, 2015-08-12, 2018-11-07 and
  2021-11-04 (the last two are Diwali Muhurat sessions); DLF and RELINFRA also on 2019-12-02; UNITDSPR on
  2014-05-23 and 2015-08-12.
- **Large moves consistent with known events** (from memory, not verified): YESBANK's 2020 rescue
  (-56% on 2020-03-06, +58% on 2020-03-17, and a run of 20-45% days); PNB +46% and BANKBARODA +31% on
  2017-10-25 (PSU bank recapitalisation); DLF -28.5% on 2014-10-14 (SEBI ban); INDUSINDBK in March 2020
  (-24%, then +45% on 2020-03-26); distressed IDEA, RCOM, RPOWER and RELINFRA (many 20-56% days in
  2017-2020); SAMMAANCAP 2019-2020; ZEEL +40% on 2021-09-14.
- **Unverified large moves:** UNITDSPR +34.7% on 2012-11-12 (possibly the Diageo deal), JINDALSTEL +25.7% on
  2015-02-19, DIVISLAB -22.2% on 2016-12-23, BANKBARODA +22.5% on 2016-02-15, HINDPETRO -23.9% on
  2018-10-05, BHEL on 2019-10-18 and 2020-05-13, AUROPHARMA on 2020-02-19, UPL on 2014-04-28, VEDL -20.4% on 2020-10-12.
- **Series identity unverified:** VEDL's history before the 2013 Sesa Sterlite merger (is it Sesa Goa's
  prices?); TMPV (pre-demerger Tata Motors; whether it is adjusted for the 2025 demerger is unknown).

## What this changes

- backtest-01's Donchian result was mostly survivorship bias: about 63% of its net P&L disappears once
  entries are restricted to actual index members. What is left (t = 2.41, PF 1.32) is a weak, unproven
  in-sample effect, and the unfetchable names (HDFC Ltd above all) could move it either way.
- TrendPullback should be dropped or rethought rather than tuned: its gross edge is the size of the costs.
- Next (unchanged): a portfolio-level run against NIFTY 50 TRI, then walk-forward, shift test, cost
  stress and Monte Carlo, all on this point-in-time universe. Also archive the exact data files behind each report.

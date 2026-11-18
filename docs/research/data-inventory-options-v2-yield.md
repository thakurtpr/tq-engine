# options-v2 data inventory: NIFTY 50 dividend yield (input to benchmark B1)

Status: file built and scanned, NOTHING repaired. No return statistic is computed or reported here.
Why it exists: B1 (buy-and-hold Nifty ETF) is price return plus a point-in-time dividend accrual of yield/252/100 per
session (`crates/r2stats/src/o2bench.rs::b1_total_return`, `yield_on`); a missing or older-than-4-day yield fails closed.

## 1. Source (free, no signup, no login, no captcha)
* Primary: NSE Indices, "Historical PE/PB/Dividend Yield", NIFTY 50. The reports page `https://www.niftyindices.com/reports/historical-data`
  loads with a plain GET; its own published script (`liveindexsa.niftyindices.com/assets/js/IISLComponet.js`, function
  `Historicaldivyield`) POSTs the form to `https://www.niftyindices.com/BackPage/getpepbHistoricaldataDBtoString` with
  `{"cinfo": "{'name':'NIFTY 50','startDate':'dd-Mon-yyyy','endDate':'dd-Mon-yyyy','indexName':'NIFTY 50'}"}` and returns JSON rows
  `{Index Name, pe, pb, divYield, DATE}`. The page refuses ranges over 1 year, so 360-day windows were used (18 requests).
* Conduct: honest User-Agent (`tq-engine-research/1.0 ...`), 1.1 s between requests, no cookies, no spoofing; no block was met
  (18 of 18 HTTP 200, no retry needed). Script: `scripts/o2_yield_fetch.py`.
* Manifest (every URL, window, fetch time UTC 2026-11-17 20:45:08 to 20:45:33, bytes, row count, sha256 of each raw JSON):
  `docs/data/options-v2-yield-manifest.tsv`. Raw JSON (outside git): `data-raw/options-v2/benchmarks/yield/raw/` (18 files).
* Output (outside git): `data-raw/options-v2/benchmarks/yield/nifty50_div_yield.csv`, columns `date,yield_pct,source`, ISO dates,
  ascending, one row per published date, `yield_pct` exactly as published (percent, 2 decimals, e.g. 1.40 = 1.40%).
  sha256 `0d269ba2ad425a027f5bf0ee64ea98b3e4a61fedaa82895cbd4d2093ebf97217`.

## 2. Coverage
| | |
|---|---|
| Rows total | 4,187 (2010-01-04 to 2026-11-17; the request started 2010-01-01 to span the NIFTYBEES start) |
| Rows in 2012-01-01..2026-10-31 | 3,676 |
| Rows after 2026-10-31 | 12 |
| NIFTYBEES sessions 2012-01-01..2026-10-31 (3,656, unrepaired store) with no row on or before the session within 4 days | **0** |

## 3. Scan (`scripts/o2_yield_scan.py`; nothing repaired)
* Non-positive values: 0. Values above 10: 0. Range over the whole file 0.90 to 2.00. Duplicate or non-monotonic dates: 0.
* Runs of identical values over 10 rows from 2012: 0 (longest run in the file is 8 rows, 2010-07-12..2010-07-21, value 0.91).
* Gaps over 4 calendar days between consecutive published dates (2012+), all market-holiday gaps with no ETF session inside; the
  B1 rule is unaffected (0 sessions lack a yield within 4 days): 2012-04-04 to 04-09 (5 d), 2014-10-01 to 10-07 (6 d),
  2015-04-01 to 04-06 (5), 2016-03-23 to 03-28 (5), 2016-04-13 to 04-18 (5), 2018-03-28 to 04-02 (5), 2022-04-13 to 04-18 (5).
* Weekend-dated rows (special sessions, expected; they match NSE special/Muhurat/Budget sessions): 2010-02-06, 2012-01-07, 2012-03-03,
  2012-04-28, 2012-09-08, 2013-05-11, 2013-11-03, 2014-03-22, 2015-02-28, 2016-10-30, 2019-10-27, 2020-02-01, 2020-11-14,
  2023-11-12, 2024-01-20, 2024-03-02, 2024-05-18, 2025-02-01, 2026-02-01, 2026-11-08. The ETF series has no bar on some of these; harmless to `yield_on` (a lookup, not a calendar).
* Precision: 2 decimals (0.01 percentage point resolution). Not repaired.

## 4. Point-in-time rule
The row's `date` is the date the figure describes (the session whose close it is computed from). The build uses the latest row on or before the
session, at most 4 days old (`yield_on`); with this file a session always finds its own date or the preceding session.

## 5. WARNING: look-ahead / revision risk (verified, not hypothetical)
What was verified: the NSE `ind_close_all` daily files already held (`compact/index_close_all.csv.gz`, NIFTY 50 `div_yield`, keyed on
`file_date`, i.e. the figure printed in the file published that day) were compared with this file on all 3,613 common dates.
* 3,446 dates (2012-02-21 to 2026-02-03, plus 2026-10-09 onward) agree EXACTLY (yield, and also P/E and P/B on the 2026 spot checks). So for
  those dates the niftyindices date is a same-day as-of value with no lag; there is no evidence of a publication lag.
* **167 dates (2026-02-04 to 2026-10-08) DISAGREE**: this file is higher by 0.05 to about 0.29 percentage points (this file 1.25 to 1.62, daily file
  1.15 to 1.52 over 2026-02-03..2026-10-09). P/E and P/B are identical on those dates; only the yield differs. Hence the niftyindices history is a
  **restated / current-vintage series, not a frozen as-published one**: NSE has revised the dividend yield for 2026-02-04..2026-10-08 after the
  daily files were printed (or the daily files carried a defective yield; which is right is not knowable from the data). Observation only: in
  the daily files the yield stays near 1.2 while the index falls about 12% across the window, which looks stale, and this file's yield rises as
  the index falls; that is an observation, not a verdict.
* Earlier history (2010 to 2012-02-20, and the special weekend sessions before 2023) has no second source, so revision there cannot be tested; the
  exact agreement over 14 years suggests revisions are rare but they happen.
* Size of the issue: at most 0.3 pp of annual yield for 167 sessions (accrual is yield/252/100 per session). Small, but it is a revision, and B1 should say which vintage it used.

## 6. Companion series (separate file, nothing merged)
`data-raw/options-v2/benchmarks/yield/nifty50_div_yield_dailyfile.csv` (sha256 `c42cee09b1f084c79d35eafb40511876a87c1febe9f33279ab76b3c6065f80af`,
3,613 rows, 2012-02-21 to 2026-11-17; `scripts/o2_yield_dailyfile.py`): the as-printed daily-file value, i.e. the strictly point-in-time series.
It cannot be the sole B1 input: 34 ETF sessions (2012-01-02 onward to 2012-02-20) have no value within 4 days, and 10 holiday gaps over 4 days exist
(the file lacks the weekend special sessions). Decision needed from the owner/protocol (no rule is applied here):
(a) primary = niftyindices file, with the daily-file series as a disclosed sensitivity for the 167 sessions; or
(b) primary = daily-file where it exists plus niftyindices for 2012-01-02..2012-02-20 (a dated rule to be written in the prereg addendum before the freeze).
Either choice must be listed as a dated repair/rule; this inventory does not choose.

## 7. Open items
* Choose (a) or (b) above and record it in the prereg addendum before the freeze.
* The yield is the index's trailing dividend yield, an approximation to the ETF's distributions (tracking, TER, timing); B1's accrual stays an ASSUMPTION, as flagged in `o2bench.rs`.
* A constant-yield series is NOT acceptable as the primary B1 and was not built; it may only serve as a labelled sensitivity.
* Paid fallback if ever needed: NSE Indices "Nifty 50 Total Returns Index (TRI)" history, which makes the accrual unnecessary.

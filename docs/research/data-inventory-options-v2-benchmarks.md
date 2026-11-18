# Data inventory: options-v2 benchmark series (NIFTYBEES, Nifty 50, BTCUSDT, USDINR)

Built 2026-11-17 on branch `hardening/o2data`. Paper/backtest only. Descriptive only: no returns, Sharpe or performance
statistic was computed. The research-v2 SEALED store and its key were not read or touched; only the plain-CSV DEV side
(`/Users/thakur/Workspace/Trading/data-sealed/dev/`: `IN-IDX/NIFTYBEES.NS.csv`, `IN-IDX/NSEI.csv`, `CRYPTO/BTCUSDT.csv`,
`CRYPTO/USDINR.csv`) was read, read-only.

Store (outside git): `/Users/thakur/Workspace/Trading/data-raw/options-v2/benchmarks/`
- `raw/` Yahoo JSON (3 files) and Binance zips (127 files, each verified against its published `.CHECKSUM`; 0 mismatches),
- `fresh/` compact CSV of the fresh pull, `unrepaired/` stitched series, `repaired/` declared-rule copy, `repairs/` repair lists,
  `overlap_<NAME>.csv` (dev vs fresh differences; all empty),
- manifest committed: `docs/data/options-v2-benchmarks-manifest.csv` (series, variant, path, sha256, rows, first_date, last_date
  for every file), repair list `docs/data/options-v2-benchmarks-repairs.csv`. Series CSVs themselves are not committed (Yahoo terms
  are understood to forbid redistribution; unverified).

Schema (both variants): `date,open,high,low,close,adjclose,volume,source` with `source` in dev / fresh / dev-fill; the repaired
copy adds `repaired` (0/1) and `close_printed`. For dev rows `adjclose` = `close` (the research-v2 dev store is Yahoo
auto-adjusted, close already adjusted); for fresh Yahoo rows `adjclose` is Yahoo's own.

## 1. Sources and conduct

| Series | Fresh source | Fresh range | Dev side used |
|---|---|---|---|
| NIFTYBEES (Nifty 50 ETF) | Yahoo chart API `https://query1.finance.yahoo.com/v8/finance/chart/NIFTYBEES.NS?interval=1d&events=div%7Csplit` | 2021-01-01 to 2026-11-17 (1,451 rows; 1 null row dropped) | 2010-01-04 to 2020-12-31 (2,712 rows) |
| Nifty 50 index (`^NSEI`) | same API | 2021-01-01 to 2026-11-17 (1,451 rows) | 2012-01-03 to 2020-12-31 (2,201 rows) |
| USDINR | same API, `USDINR=X` | 2021-01-01 to 2026-11-17 (1,528 rows; 5 null rows dropped) | 2012-01-02 to 2020-12-31 (2,345 rows) |
| BTCUSDT spot | `https://data.binance.vision/data/spot/{monthly,daily}/klines/BTCUSDT/1d/BTCUSDT-1d-<YYYY-MM[-DD]>.zip` | 2017-08-17 to 2026-11-16 (3,379 rows; monthly zips to 2026-10, daily zips 2026-11-01..16; the 2026-11-17 daily zip is 404, not yet published) | none needed (cross-check only) |
| Nifty 50 close, NSE | `ind_close_all` files already held (`docs/data/options-v2-index-close-core.csv`, keyed on `file_date`) | 2012-02-21 to 2026-11-17 | cross-check |

Conduct: honest User-Agent `tq-engine-research/1.0 (...)`, no credentials, one request at a time at least 1.1 s apart, stop on
403/429/503. No block occurred (200 or 404 only). Yahoo is an unofficial endpoint with no SLA (terms not re-read: unverified).
Binance: monthly files for 2017-08 to 2026-10 and daily files for the open month; microsecond timestamps from 2025 handled.

## 2. Stitch rule and overlap verification

Rule: dev rows strictly before the stitch date, fresh rows from the stitch date. NIFTYBEES stitch 2021-01-01 (starts 2010-01-04),
Nifty 50 stitch 2021-01-01 (starts 2012-01-03), USDINR stitch 2021-01-01 (starts 2012-01-02), BTCUSDT: fresh for all dates (no
stitch; dev checked on the whole history). Dev rows beyond each stitch date (NIFTYBEES and Nifty 50 to 2021-12-31, USDINR to
2026-10-30, BTC to 2026-10-31) were compared with the fresh rows for the same dates on open, high, low, close, tolerance 1e-4 relative.

| Series | Overlap dates | Range | Field differences > 1e-4 | Max relative difference | Dates in only one source |
|---|---|---|---|---|---|
| NIFTYBEES | 248 | 2021-01-01 to 2021-12-31 | 0 | 3.6e-9 | 0 |
| Nifty 50 (`^NSEI`) | 248 | 2021-01-01 to 2021-12-31 | 0 | 2.8e-11 | 0 |
| USDINR | 1,516 | 2021-01-01 to 2026-10-30 | 0 | 6.9e-9 | 0 |
| BTCUSDT | 3,363 | 2017-08-17 to 2026-10-31 | 0 | 0 | 0 (fresh extends 16 days past dev) |

Differences are float rounding only. **Caveat:** the dev store and the fresh pull share an upstream (Yahoo; Binance for BTC), so
agreement shows the research-v2 data step is reproducible and the stitch is seamless, not that the upstream is right. The only
independent check is Nifty 50 against NSE `ind_close_all`: 3,607 common dates (2012-02-21 to 2026-11-17), 0 closes differ by
more than 0.01%. Yahoo has 45 dates NSE lacks (33 in Jan-Feb 2012 before the NSE files start, 12 where the NSE index file is
absent at source, e.g. 2014-03-19, 2014-12-15) and NSE has 6 weekend special sessions Yahoo lacks (2023-11-12, 2024-01-20,
2024-03-02, 2024-05-18, 2026-02-01, 2026-11-08). The 2026-11-17 Nifty 50 close agrees with NSE's file, so that bar is final; the
same-day NIFTYBEES and USDINR bars are not independently checked and may be provisional. NIFTYBEES and USDINR before 2021 rest
on the dev store alone.

## 3. Total-return convention (benchmark B1 caveat)

Same convention as the research-v2 data step: Yahoo split-adjusted close, "adjusted for returns" = Yahoo `adjclose` (dev store:
`auto_adjust=True` close). **For NIFTYBEES Yahoo lists 0 dividend and 0 split events in the fresh pull and `adjclose` equals
`close` on every row (0 of 4,163 differ), so the "total return" series is in effect a PRICE return.** Whether the ETF paid
distributions that Yahoo omits is **unverified**. The Nifty 50 series is a price index (no TRI); the NSE files carry a
dividend-yield column (`div_yield` in `index_close_all`, about 1-1.5% in rows seen; accuracy unverified) that a builder could use
for an explicit approximation, but nothing here applies it. A B1 built from these series understates total return by roughly the
dividend yield: the same bias as research-v2, and it makes B1 easier to beat.

## 4. Scan results (`scripts/o2_dq_scan.py --bench-dir`; `docs/dq-scan-options-v2-benchmarks-unrepaired.md` and `-repaired.md`)

| Series (unrepaired) | Rows | First | Last | Non-positive | Close outside [low, high] | Reverting spikes >= 20% | Missing calendar sessions | Non-session bars |
|---|---|---|---|---|---|---|---|---|
| NIFTYBEES | 4,163 | 2010-01-04 | 2026-11-17 | 0 | 0 | 1 (2019-12-19) | 1 (2025-10-24) | 22 (2012+) |
| Nifty 50 | 3,652 | 2012-01-03 | 2026-11-17 | 0 | 0 | 0 | 0 | 6 (Muhurat sessions) |
| BTCUSDT | 3,379 | 2017-08-17 | 2026-11-16 | 0 | 0 | 0 | 0 (24x7) | n/a |
| USDINR | 3,873 | 2012-01-02 | 2026-11-17 | 0 | 160 strict (5 beyond 0.5%) | 0 | see below | see below |

- NIFTYBEES: 169 zero-volume rows (flat open = high = low = close fills, all before 2015-03). The Yahoo bar for **2025-10-24 is
  missing** while Nifty 50 has it. 22 bars sit on calendar non-sessions: holiday fills 2012-01-02, 2012-05-21, 2012-08-28,
  2013-01-01, 2014-01-01, 2014-02-17, 2015-01-01, 2015-02-28, 2015-04-15, 2016-01-01, 2016-08-12, 2018-01-01, 2019-01-01; special
  sessions 2017-10-19, 2018-11-07, 2019-10-27, 2020-11-14, 2021-11-04, 2022-10-24, 2024-11-01, 2025-10-21; and 2026-01-15
  (calendar says closed, Nifty 50 has no bar: a stray ETF bar, unverified). No declared rule repairs these; builders must not
  treat them as sessions.
- USDINR: volume 0 on every row (not recorded). **Date labelling quirk:** from 2012-03-25 Yahoo stamps the London-midnight daily
  bar at 23:00 UTC of the previous day in British Summer Time, so the date label is one day early in summer: Mondays appear as
  Sundays (456 Sunday-labelled rows), Friday labels are rare (319) and 465 weekday dates have no bar. The dev store has the
  identical labelling (overlap matches date for date), so the stitched series is continuous but its dates are not clean calendar
  dates. No declared rule corrects it; not repaired. Winter bars (UTC 00:00) are correctly dated.

## 5. Repairs (declared rules reused as code; nothing invented)

Tool: `crates/r2stats/examples/o2_bench_repair.rs` (new example binary; calls `tq_r2stats::aux_clean::validate_bars`,
`bench_clean::repair_bad_prints` (addendum 4) and `aux_clean::repair_usdinr` (addendum 6) unchanged). NIFTYBEES, Nifty 50 and
BTCUSDT: `validate_bars` then `repair_bad_prints`; USDINR: `repair_usdinr`. A repaired bar gets close = carried value and open =
high = low = close; `close_printed` keeps the printed value; `adjclose` follows. Full list: `options-v2-benchmarks-repairs.csv`.

| Series | Date | Printed close | Carried close | Rule |
|---|---|---|---|---|
| NIFTYBEES | 2019-12-19 | 13.02 | 129.254 | addendum 4 bad print |
| NIFTYBEES | 2019-12-20 | 13.034 | 129.254 | addendum 4 bad print |
| USDINR | 2012-01-27 | 52.493 | 50.0 | addendum 6 F1 |
| USDINR | 2012-01-31 | 52.493 | 49.388 | F1 |
| USDINR | 2012-04-03 | 50.25 | 50.868 | F1 |
| USDINR | 2013-09-03 | 68.61 | 66.51 | F2 |
| USDINR | 2018-10-18 | 74.16 | 73.92 | F1 |
| USDINR | 2020-04-01 | 77.57 | 75.325 | F2 |
| USDINR | 2020-04-02 | 77.14 | 75.325 | F2 |
| USDINR | 2020-04-05 | 76.23 | 75.325 | F2 |
| USDINR | 2023-11-02 | 85.194 | 83.277 | F1 |
| USDINR | 2026-04-12 | 94.509 | 92.470 | F2 |
| USDINR | 2026-04-13 | 94.947 | 92.470 | F2 |
| USDINR | 2026-08-25 | 93.546 | 95.723 | F2 |

These are exactly the addendum-4 and addendum-6 lists restricted to 2012 onward (the 2009-2011 USDINR repairs lie before the
stitch start); the fresh pull (2021-01 to 2026-11) added no new repair. Nifty 50 and BTCUSDT: 0 repairs. **Fail closed:** after
repair the scan finds 0 reverting spikes in every series and the tool exits 0; a synthetic 4-bar 60% reverting spike is refused
(exit 3, "unrepaired reverting price spike"), confirming the refusal path. The unrepaired copies stay alongside the repaired ones.

## 6. Notes for the builders (B1 Nifty ETF total return, B3 50/50 ETF + BTC in INR, quarterly rebalance)

1. B1 total return is a price return in effect (section 3); the ETF has a small tracking gap to the index and a unit price about 1/100 of the index.
2. NIFTYBEES has no bar on 2025-10-24 (a session day): the builder needs a carry rule (none declared here) or must refuse.
3. USDINR date labels are one day early in British summer months (section 4); the INR conversion of BTC on the NSE calendar must
   state which USDINR bar it uses. Not repaired.
4. BTC trades every day, the ETF only on NSE sessions: B3 must align on NSE sessions and state how weekend BTC moves enter the
   rebalance. BTCUSDT starts 2017-08-17, so B3 cannot exist earlier. The Binance 2026-11-17 candle is not yet published (last BTC
   bar 2026-11-16) while Yahoo series end 2026-11-17: cut the common window at 2026-11-16 or re-pull.
5. Stitch continuity at 2021-01-01 is verified only against the same upstream (section 2).
6. Yahoo FX (`USDINR=X`) is an indicative quote, not an RBI reference rate; unverified.
7. No gate, threshold or benchmark window was chosen here; no sealed data was used.

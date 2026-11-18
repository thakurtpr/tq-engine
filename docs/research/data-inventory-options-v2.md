# Data inventory: NSE derivatives, options-v2 (DATA task)

Built 2026-11-17 on branch `hardening/o2data`. Paper/backtest research only. **Descriptive only**: row counts, ranges, gaps and
bias notes; no returns, Sharpe or any performance statistic was computed on any series, and no window is treated as sealed here
(the protocol defines the sealed final slice). Companion files: `docs/dq-scan-options-v2.md` (bad prints and gaps),
`docs/regulatory-timeline-options-v2.md` (SEBI/NSE rules), `docs/data/lot-sizes-options-v2.csv`, `docs/data/options-v2-manifest.csv`.

## 1. What was built

One compact store **outside git**: `/Users/thakur/Workspace/Trading/data-raw/options-v2/compact/` (585 MB).

- `fo/<UNDERLYING>/<UNDERLYING>_<YEAR>.csv.gz`: one gzip-compressed CSV per underlying per year (gzip chosen for disk; the
  gzip is deterministic, mtime 0, so hashes are reproducible). 45 underlyings, 44,721,495 rows, 2012-01-03 to 2026-11-17
  (3,656 F&O sessions: 3,068 old format, 588 new format). Phase 1 (sessions from 2021-11-01): 1,252 sessions, 18,191,992
  kept rows. Phase 2 (to 2021-10-31): 2,404 sessions, 26,529,503 kept rows. Source rows read 144,757,935; 100,036,440 were
  dropped (other underlyings and instruments).
- `index_close_all.csv.gz` (all 239 index names, 298,810 rows), `options-v2-index-close-core.csv`, `options-v2-india-vix.csv`
  (copied to `docs/data/`, 1.1 MB and 0.18 MB), `sessions.csv` (per source file: format, SHA-256, rows total/kept).
- Committed in `docs/data/`: `options-v2-manifest.csv` (series, path, sha256, rows, first_date, last_date, bytes for every compact
  file), `options-v2-sessions-manifest.csv`, `options-v2-raw-manifest.sha256` (SHA-256 of every 2022-2026 raw file, raw index
  files, 3 circular PDFs), `options-v2-stock-universe.csv` (full ranking), `options-v2-calendar-nse.csv`,
  `options-v2-expiry-structure.csv`, `options-v2-zero-volume-by-year.csv`, `options-v2-dq-defects.csv`,
  `lot-sizes-options-v2.csv`, `options-v2-index-close-core.csv`, `options-v2-india-vix.csv`.
- Scripts: `scripts/o2_fetch.py`, `scripts/o2_convert.py` (rank, convert, crosscheck, index), `scripts/o2_dq_scan.py`,
  `scripts/o2_lots.py`, `scripts/o2_manifest.py`, `crates/calendar/examples/o2_dump_sessions.rs`.

### Compact schema (identical for both source formats)

`date, instrument, symbol, expiry, strike, option_type, open, high, low, close, settle, volume, open_interest, change_oi,
underlying_price, lot_size, value_lakh`

`instrument` is OPTIDX/OPTSTK/FUTIDX/FUTSTK; dates are ISO; `strike` and `option_type` are empty for futures; numerics are copied
verbatim except trailing zeros ('104.00' becomes '104'). `volume` is in **contracts** in both formats (verified: on 2024-07-05
both files exist and all 13,184 kept contracts agree on every field, 0 disagreements). `open_interest` and `change_oi` are in
**quantity units** (contracts x lot) in both formats. `value_lakh` is traded value in Rs lakh (old `VAL_INLAKH`; new
`TtlTrfVal`/1e5); it is an addition to the requested schema. **`underlying_price` and `lot_size` exist only for the new format
(sessions from 2024-07-05)** and are empty before. Other new-format fields (LastPric, PrvsClsgPric, trade counts, ISIN) are not
kept. 2024-07-05 is taken from the new file; the old file served only for the cross-check.

### Underlyings kept

Index: NIFTY, BANKNIFTY, FINNIFTY, MIDCPNIFTY, NIFTYNXT50 (all present; first sessions NIFTY/BANKNIFTY 2012-01-03, FINNIFTY
2021-01-11, MIDCPNIFTY 2022-01-24, NIFTYNXT50 2024-04-24), options and futures. Other index symbols in the files (sector
indices, foreign indices, FUTIVX India VIX futures) were dropped.

**Stock rule (deterministic, performance-free):** for every OPTSTK symbol sum the exchange field `VAL_INLAKH` (traded value, Rs
lakh) over every session from 2012-01-01 to 2021-10-31 (the Phase-2 window, 2,404 sessions); rank descending, ties broken
alphabetically by symbol; keep ranks 1-40. No price, return or P&L enters the rule (`o2_convert.py rank`; output
`docs/data/options-v2-stock-universe.csv`, 360 ranked symbols; rank 40 = CIPLA, rank 41 = PNB). Selected, in rank order:
RELIANCE, SBIN, TATASTEEL, INFY, ICICIBANK, TCS, TATAMOTORS, MARUTI, BAJFINANCE, AXISBANK, HDFCBANK, BHARTIARTL, ITC, LT, HDFC,
KOTAKBANK, YESBANK, SUNPHARMA, DLF, INDUSINDBK, HINDALCO, VEDL, DRREDDY, HINDUNILVR, ADANIENT, WIPRO, TITAN, ADANIPORTS,
HCLTECH, ASIANPAINT, ASHOKLEY, BPCL, AUROPHARMA, HEROMOTOCO, JSWSTEEL, TECHM, M&M (directory `M_AND_M`), LUPIN, ZEEL, CIPLA.
All option and futures contracts of each are kept.

## 2. Sources, URLs, terms, ranges

| Source | URL pattern | Range used | Rows read |
|---|---|---|---|
| NSE F&O bhavcopy, old format | `https://nsearchives.nseindia.com/content/historical/DERIVATIVES/<YYYY>/<MON>/fo<DD><MON><YYYY>bhav.csv.zip` | 2012-01-03 to 2024-07-05 (3,068 sessions; 2012-01 to 2021-12 from the earlier options-v1 download, 2022-01 onward fetched now) | 122,966,554 |
| NSE F&O bhavcopy, UDiFF new format | `https://nsearchives.nseindia.com/content/fo/BhavCopy_NSE_FO_0_0_0_<YYYYMMDD>_F_0000.csv.zip` | 2024-07-05 to 2026-11-17 (588 sessions) | 21,791,381 |
| NSE index closes and India VIX | `https://archives.nseindia.com/content/indices/ind_close_all_<DDMMYYYY>.csv` | 2012-02-21 to 2026-11-17 (VIX row from 2014-05-14) | 298,810 |
| NSE circulars (lot sizes, expiry day) | `https://archives.nseindia.com/content/circulars/FAOP64625.pdf`, `FAOP68747.pdf`, `FAOP70616.pdf` | read | 3 PDFs |
| tq-calendar | `crates/calendar` (holiday table 2012-2026) | check only | n/a |

Fetch conduct: honest User-Agent (`tq-engine-research/1.0 (personal paper-trading research; sequential, 1 request per second)`),
no cookies, no login, one request at a time, at least 1.1 s apart, stop-on-block logic (403/429/503 or 3 network errors).
**No block occurred**: every response was 200 or 404 (holidays/weekends). Weekend days were probed too (special sessions
found, see section 4).
**Terms of service: not read.** NSE data is publicly downloadable; NSE's website terms typically restrict redistribution and
commercial reuse (unverified). The compact store and raw files are local only; only small derived series (index close/VIX,
1.3 MB) and manifests are in git. The standing checkpoint rule must not publish them to a public repo unless the owner decides so.

### Format change 2024-07-05 and normalisation

Old: `INSTRUMENT, SYMBOL, EXPIRY_DT (dd-Mon-yyyy), STRIKE_PR, OPTION_TYP (CE/PE/XX), OPEN, HIGH, LOW, CLOSE, SETTLE_PR, CONTRACTS,
VAL_INLAKH, OPEN_INT, CHG_IN_OI, TIMESTAMP`. New (UDiFF): `TradDt, FinInstrmTp (IDO/IDF/STO/STF), TckrSymb, XpryDt (ISO),
StrkPric, OptnTp, OpnPric, HghPric, LwPric, ClsPric, SttlmPric, UndrlygPric, OpnIntrst, ChngInOpnIntrst, TtlTradgVol,
TtlTrfVal (Rs), NewBrdLotQty, ...`. Mapping: IDO to OPTIDX, IDF to FUTIDX, STO to OPTSTK, STF to FUTSTK; CONTRACTS = TtlTradgVol;
VAL_INLAKH = TtlTrfVal/1e5; option type `XX` becomes empty. A first draft wrongly divided `TtlTradgVol` by the lot; the 2024-07-05
two-format cross-check caught it (volume is already in contracts). Source quirk normalised, not repaired: the 2012-05-14 old
file writes some expiries with a 2-digit year (`31-May-12`), read as 2012.

## 3. Coverage table (per-file detail in `options-v2-manifest.csv`)

| Series | First | Last | Rows | Main bias |
|---|---|---|---|---|
| NIFTY (options + futures) | 2012-01-03 | 2026-11-17 | 7,433,609 | 66-87% zero-volume option rows; weeklies only from 2019-02 |
| BANKNIFTY | 2012-01-03 | 2026-11-17 | 3,760,573 | weeklies 2016-06 to 2024-11-13 only |
| FINNIFTY | 2021-01-11 | 2026-11-17 | 1,514,659 | weeklies 2021-01 to 2024-11-19 only |
| MIDCPNIFTY | 2022-01-24 | 2026-11-17 | 1,320,099 | weeklies 2022-02 to 2024-11-18 only |
| NIFTYNXT50 | 2024-04-24 | 2026-11-17 | 650,220 | short history, monthly only |
| 40 stock underlyings (total) | 2012-01-03 (some later) | 2026-11-17 (some earlier) | 30,042,335 | selection by Phase-2 liquidity; survivorship |
| Index closes, 239 indices | 2012-02-21 | 2026-11-17 | 298,810 | none for Jan-Feb 2012; month/day swapped dates in 3 files |
| India VIX | 2014-05-14 | 2026-11-17 | 3,073 | nothing before 2014-05-14; 2 rows close-only |
| Core index closes (6 names) | 2012-02-21 | 2026-11-17 | 15,737 | Nifty Midcap Select sporadic before 2021-09 |

Stock underlyings that end or have gaps: HDFC ends 2023-07-12 (merger into HDFCBANK; unverified), TATAMOTORS ends 2025-10-23
(demerger or symbol change; unverified), ZEEL ends 2024-06-27 (reason unknown), YESBANK has no rows 2020-05-29 to 2024-11-28
(1,113 sessions), BAJFINANCE starts 2015-05-29, VEDL 2015-05-07, ADANIPORTS 2012-01-17. Index closes for 2012-01-03 to
2012-02-20 (35 F&O sessions) and India VIX before 2014-05-14 do not exist in the NSE ind_close_all files (HTTP 404 / absent row).

## 4. Known biases and caveats

1. **Survivorship / selection of stock underlyings.** The 40 names are the most liquid by Phase-2 option value. They are fixed by
   a rule that never looks at Phase-1 data, so there is no look-ahead into Phase 1, but (a) names that became liquid only after
   2021 are absent; (b) names merged or removed from F&O inside 2012-2021 are present only if they ranked high; (c) kept names
   have post-2021 corporate events (HDFC merger, TATAMOTORS change, ZEEL exit). Index underlyings have no constituent survivorship.
2. **Contract-month availability.** Stock options list 3 monthly expiries; index options list near monthlies, quarterlies and
   long-dated contracts (2024+: expiries out to 2029), plus weeklies where allowed. Strike grids differ by date and underlying.
3. **Zero-volume rows** (share of rows with volume = 0; all kept underlyings; per-underlying in
   `options-v2-zero-volume-by-year.csv`):

   | year | index fut | index opt | stock fut | stock opt |
   |---|---|---|---|---|
   | 2012 | 0.0% | 85.7% | 16.0% | 89.0% |
   | 2013 | 0.0% | 85.3% | 14.3% | 86.3% |
   | 2014 | 0.0% | 85.8% | 13.5% | 84.7% |
   | 2015 | 0.0% | 86.8% | 11.0% | 83.4% |
   | 2016 | 0.0% | 87.4% | 13.6% | 83.3% |
   | 2017 | 0.0% | 87.2% | 14.5% | 82.3% |
   | 2018 | 0.0% | 86.2% | 10.5% | 79.2% |
   | 2019 | 0.0% | 86.6% | 6.8% | 76.1% |
   | 2020 | 0.0% | 83.2% | 3.8% | 75.9% |
   | 2021 | 36.2% | 79.1% | 0.9% | 78.3% |
   | 2022 | 46.6% | 77.5% | 0.3% | 76.3% |
   | 2023 | 23.3% | 65.8% | 0.8% | 71.2% |
   | 2024 | 5.3% | 66.8% | 0.5% | 57.4% |
   | 2025 | 10.0% | 63.5% | 0.1% | 51.4% |
   | 2026 | 15.5% | 65.5% | 0.2% | 51.8% |

   Index-futures zero-volume rows from 2021 are far-dated FINNIFTY/MIDCPNIFTY/NIFTYNXT50 futures. A zero-volume row carries a
   stale carried-forward close and an exchange-computed settle: **not a price anyone traded at**. Fills must require volume > 0.
4. **Settle versus close semantics (observed in the data, not from NSE documentation).** For options: (a) through 2020-01-30
   the old files carry `settle = 0` on the expiry day for every option row (136,105 traded rows, code E01); (b) from 2020-02-06
   the expiry-day option `settle` equals the **underlying's final settlement price** for every strike (530,012 rows, code E02),
   not an option premium, so expiry payoff must be recomputed from the underlying settlement; (c) separately, 86,757 traded rows
   with settle 0 occur on non-expiry days (code R04), concentrated 2019-2026. Never use `settle` as a mark on or near expiry day.
5. **Missing sessions vs tq-calendar** (calendar dumped by `o2_dump_sessions.rs` into `options-v2-calendar-nse.csv`; it fails closed as designed: `is_trading_day` returns `OutOfCoverage` for 2011-12-30 and 2027-01-04, coverage 2012-01-01 to 2026-12-31, which spans the whole store). 2013-10-09 and 2021-03-30 are calendar trading days with no F&O file. 2013-10-09 has no
   index file either while adjacent days exist: probably a calendar error or a real closure, unresolved. 2021-03-30 has an index
   file with data (Holi was 2021-03-29) but the F&O bhavcopy URL returned 404: a source gap, unrepaired. The index-close file is
   absent at source for 2014-03-19, 2014-12-15, 2015-02-02, 2015-03-12/13, 2015-05-19, 2015-07-08, 2015-09-04, 2015-10-16,
   2015-12-01 and 2016-06-20 although F&O sessions exist, so Nifty and VIX series have those holes.
6. **Special sessions.** F&O files exist on 12 days the calendar calls non-trading: 2017-10-19, 2018-11-07, 2021-11-04, 2022-10-24,
   2023-11-12, 2024-11-01, 2025-10-21 and 2026-11-08 (Diwali Muhurat sessions, unverified) and 2024-01-20, 2024-03-02, 2024-05-18,
   2026-02-01 (weekend sessions, possibly contingency or budget-day sessions; unverified). Each has rows and is kept; whether
   prices on such days are tradeable is **unverified**.
7. **Expiry-day moves (Tuesday/Thursday).** Data: monthly index and stock expiries were Thursdays until 2025-08-28 (last Thursday
   expiry in the 2024-26 listings); the first Tuesday expiries are 2025-09-02 (NIFTY weekly) and 2025-09-30 (all monthlies).
   NSE/FAOP/68747 (Circular 111/2025, 2025-06-25, read) confirms: NIFTY weekly Thursday to Tuesday, all monthlies last Thursday
   to last Tuesday, existing contracts expiring on or before 2025-08-31 unchanged. 2026 shows Monday expiries when the Tuesday is a
   holiday. Expiry labels that fall on a calendar holiday (E03; e.g. 2014-02-27, 2023-06-29) mean the last trading day is the
   previous session.
8. **Weekly expiry availability (from traded option expiries; `options-v2-expiry-structure.csv`):**

   | Underlying | First weekly expiry (first traded) | Last weekly expiry traded | Notes |
   |---|---|---|---|
   | NIFTY | 2019-02-14 (2019-02-11) | still listed | Thursday until 2025-08-28, Tuesday from 2025-09-02 |
   | BANKNIFTY | 2016-06-02 (2016-05-27) | 2024-11-13 | Thursday, mostly Wednesday from 2023 |
   | FINNIFTY | 2021-01-14 (2021-01-11) | 2024-11-19 | Tuesday from 2022 |
   | MIDCPNIFTY | 2022-02-01 (2022-01-28) | 2024-11-18 | mostly Monday |
   | NIFTYNXT50 | none (monthly/quarterly) | n/a | listed 2024-04-24 |
   | stock options | none | n/a | monthly only (3 near months) |

   The end of BANKNIFTY/FINNIFTY/MIDCPNIFTY weeklies in Nov 2024 matches the SEBI one-index rule (effective 2024-11-20); FINNIFTY
   session row counts fall around 2024-11-21 to 28. Before 2019 NIFTY had only monthlies; BANKNIFTY weeklies start 2016-06 (an
   earlier note in `data-feasibility-options.md` said 2016-10 from memory: superseded).
9. **Lot sizes** change by date: `lot-sizes-options-v2.csv` (index and stock underlyings; one row per level change).
   Before 2024-07-05 lots are implied from value/volume/close (+-2%; stock lots rounded to 5 or 25) and are NOT
   circular-verified. Index lots from 2025-01 are confirmed by NSE/FAOP/64625 (Circular 128/2024, 2024-10-18) and NSE/FAOP/70616
   (Circular 176/2025, 2025-10-03), both read. A lot belongs to the contract: for up to a month around a revision, contracts
   carry different lots; the `lot_size` column (new format) is per row. `effective_from` is the first session at which the most
   liquid future carries the new lot and can lag the circular's effective date by up to one monthly expiry.
10. **Index series quirks.** Names changed (S&P CNX Nifty, then CNX Nifty from 2013-02-08, then Nifty 50 from 2015-11-09; CNX Bank
    to Nifty Bank; CNX Finance to Nifty Financial Services; CNX Nifty Junior to Nifty Next 50); mapped in `o2_convert.py` `CORE`;
    continuity across renames is unverified. `index_date` is month/day SWAPPED inside the files dated 2023-04-06, 2023-04-10 and
    2023-04-11 (they say 04-06-2023 etc.); 58 files in 2014-2015 use `/` separators (normalised). The store keeps both `date`
    (as stated) and `file_date`; downstream must key on `file_date`. India VIX has open/high/low `-` on 2021-02-12 and 2021-03-30.
11. **Revisions.** Files were not re-downloaded to test for later revisions (unverified). The 2012-2021 files match the options-v1
    manifest hashes (2,447 of 2,447 CSVs).
12. **Corporate actions.** Splits, bonuses and mergers appear as adjusted strikes and lots without a flag; visible examples
    are put prices above strike for BAJFINANCE in 2016-09 (R13) and implausible strikes for ASHOKLEY in 2014-05 (R10).

## 5. Disk and deletions

Free space before: about 17-18 GB; low point during conversion about 10 GB; after deleting raw: 18 GB. Deleted after conversion
and recording in the SHA-256 manifests: `data-raw/options-v1/fo/*` (4,896 files, 7.1 GB; the existing
`docs/prereg-options-v1-data-manifest.sha256` is kept as the record; 2,447 of 2,447 CSV hashes matched) and
`data-raw/options-v2/raw/fo/*` (1,213 files, 1.1 GB). Kept outside git: raw `ind` files (about 30 MB), download logs, the three
circular PDFs (`data-raw/options-v2/circ`), the compact store. To recover raw bytes re-run `scripts/fetch_options_dev.py` or
`scripts/o2_fetch.py` and verify against the manifests.

## 6. Not obtained, and what paid data would unlock (vendors named from memory; unpriced, unverified; none contacted)

| Missing | Why it matters | Possible source | Strategies unlocked |
|---|---|---|---|
| Intraday option prices (1-min/tick, bid/ask, depth) | EOD only: no fills, spreads, intraday stop-outs, timed entries | TrueData, Global Datafeeds (GDFL), NSE data products; broker historical APIs need an account (excluded by rule) | intraday entries, expiry-day scalping, realistic stop and spread models, gamma scalping |
| Historical implied-vol surface / Greeks as published data | IV must be derived from EOD closes with timing noise | analytics vendors (Sensibull/Opstra-type products, ivolatility-style archives) | skew, term-structure, IV-percentile, vol-surface relative value at vendor quality |
| Bid/ask and quote history | slippage must be assumed | NSE tick-by-tick feed products | market-making, execution studies, tight-spread structures |
| Exchange margin (SPAN + exposure) by date | not in bhavcopy | NSE SPAN archives (historical availability unverified) | exact margin-aware sizing of short and defined-risk structures |
| India VIX before 2014-05-14; index closes Jan-Feb 2012 | absent from ind_close_all | NSE historical-VIX report (cookie-primed API, deliberately not used) | longer vol-regime filters |
| Corporate-action-adjusted option series; F&O constituent history | not flagged in files | vendor adjustment tables | clean stock-option panels across splits and mergers |
| Participant-wise OI/volume | not in these files | NSE participant reports (public, not fetched here) | positioning filters |

## 7. Everything unverified (summary)

NSE terms of service wording; whether source files are revised after publication; why 2013-10-09 and 2021-03-30 are missing;
tradeability on special and weekend sessions; all pre-2025 index lots and every stock lot (derived, not circular-checked); the
circular for the June 2025 lot revision seen in the data (BANKNIFTY 30 to 35, MIDCPNIFTY 120 to 140); the circular for the April
2024 NIFTY 50 lot of 25; continuity of renamed index series; reasons HDFC/TATAMOTORS/ZEEL end; settle semantics (observed in the
data only); every secondary-source row of the regulatory timeline.

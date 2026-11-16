# Data inventory, research-v1 (2026-11)

Built 2026-11-16 on branch `hardening/rdata` (worktree `wt/rdata`). Paper and backtest only. **Ranges, row counts, gaps and bias notes only: no return,
Sharpe or any performance statistic was computed on any series** (the only value-level check is Yahoo-vs-NSE close agreement, section 1, which is a
data-integrity check on identical quantities, not a performance statistic). Daily bars only (no 1m, no ticks).

Raw files live **outside git** under `/Users/thakur/Workspace/Trading/data-raw/research-v1/` (compact daily CSVs in `compact/`). Downloaded raw source
files (Yahoo JSON, Binance zips) were **deleted after conversion**; their SHA-256 is recorded in the manifests below, together with the SHA-256 of
each compact output. Committed here: manifests, gap report, scripts, this doc. Disk: 82 MB used by research-v1; free space stayed >= 24 GB throughout
(floor required: 8 GB).

## Files committed with this doc

| File | Content |
| --- | --- |
| `docs/data/research-v1-manifest.csv` | `series,path,sha256,rows,first_date,last_date,kind`: 371 compact series (research-v1) plus the engine's pre-existing daily files (`kind=engine-existing`). Funding files: dates are UTC dates of the first/last funding print. |
| `docs/data/research-v1-gap-report.csv` | gaps: equities/indices/FX > 7 calendar days, crypto klines > 1 day, funding > 8.5 h between prints. |
| `docs/data/research-v1-yahoo-source-manifest.csv` | per Yahoo series: compact SHA-256, **raw JSON SHA-256 (raw deleted)**, timezone, currency, null rows dropped, duplicate dates, bad-OHLC rows, zero-volume rows, rows where adjclose != close, gaps. |
| `docs/data/research-v1-binance-qc.csv` | per Binance series: compact SHA-256, rows, range, number of source zips, checksum mismatches (0 everywhere), missing days. |
| `docs/data/research-v1-binance-rawfiles.csv` | every source zip (key, **zip SHA-256, deleted after use**, Binance `.CHECKSUM` status = ok, rows): 4,318 files. |
| `scripts/rdata_manifest.py` | regenerates the manifest and gap report from the compact files. |
| options-v1 raw files | not re-hashed: `prereg-options-v1-data-manifest.sha256` (branch `hardening/options`) already covers all 2,447 `fo` and 2,404 `ind` files. |

## Source, licence and rate-limit notes (flagged unverified where I did not read the legal text)

| Source | Access used | Licence / ToS note | Status |
| --- | --- | --- | --- |
| Yahoo Finance chart API (`https://query1.finance.yahoo.com/v8/finance/chart/<SYM>?interval=1d&events=div|split`) | plain HTTPS GET, honest `tq-research/1.0` user agent, 1 to 2 s between calls, ~170 calls | Unofficial endpoint, no SLA; Yahoo terms are understood to allow personal use and forbid redistribution. Data stays local, not committed. **Terms text not re-read: unverified.** | pulled |
| NSE `https://archives.nseindia.com/content/indices/ind_close_all_DDMMYYYY.csv` and F&O bhavcopy (`nsearchives.nseindia.com/content/historical/DERIVATIVES/...`), fetched in options-v1 | static archive files; this task made 5 probe GETs (honest UA, 1 s apart: one 200, four 404) | NSE terms restrict systematic/automated collection and redistribution **(not re-read; flagged in `data-feasibility-options.md` and `data-feasibility-sue.md`)**. Archive host has worked for static files. Not committed. No browser spoofing, cookie priming, or retry after a block. | existing + 1 probe |
| `https://data.binance.vision` (public S3 bucket listing + CDN) | listing XML, zips + `.CHECKSUM` over HTTPS, 0.15 s pause per file, up to 4 parallel workers | Published as public market data; **licence text not read: unverified**. All 4,318 zips matched their `.CHECKSUM`. | pulled |
| engine `data/` (earlier tasks) | local | Yahoo via yfinance (`scripts/fetch_daily.py`, `auto_adjust=True`) | existing |

## 1. Coverage table (main series)

Row counts are after dropping rows with null OHLC (Yahoo returns a few null rows per series) and duplicate dates (none found).

| Series | Source / file | First | Last | Rows | Freq | Adjusted? | Main bias or caveat |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **NIFTYBEES** (Nifty 50 ETF) | Yahoo `NIFTYBEES.NS`, `yahoo/NIFTYBEES.NS.csv` | 2009-01-02 | 2026-11-06 | 4,399 | daily | close = adjclose (Yahoo lists no split and no dividend events; whether the ETF paid distributions: **unverified**) | 175 zero-volume days, 1 zero-open row (2009-07-07), 10 null rows dropped. Unit is ~1/100 of the index. Engine copy `etf/NIFTYBEES.csv` is 2010-01-04..2021-12-31, 2,958 rows; **its "adjusted" file lacks 2019-12-19 and 2019-12-20 that the `-raw` file has** (2,960 rows). |
| JUNIORBEES (Next 50 ETF) | Yahoo | 2009-01-02 | 2026-11-06 | 4,399 | daily | no events | 247 open = 0 rows (2009-01-02..2010-01-18), 353 zero-volume days. |
| BANKBEES (Nifty Bank ETF) | Yahoo | 2009-01-02 | 2026-11-06 | 4,399 | daily | no events | same 247 zero-open rows, 352 zero-volume days. |
| PSUBNKBEES | Yahoo | 2009-01-02 | 2026-11-06 | 4,399 | daily | no events | same zero-open rows. |
| ITBEES | Yahoo | 2020-06-30 | 2026-11-06 | 1,575 | daily | no events | short history. |
| LICNETFN50 (Nifty 50 ETF) | Yahoo | 2015-11-20 | 2026-11-06 | 2,707 | daily | no events | liquidity not measured here. |
| HDFCNIFTY (Nifty ETF) | Yahoo | 2023-06-08 | 2026-11-06 | 826 | daily | | short. |
| BANKETF (Nifty Bank ETF) | Yahoo | 2023-07-26 | 2026-11-06 | 808 | daily | | short. |
| HDFCSENSEX | Yahoo | 2018-05-08 | 2026-11-06 | 2,078 | daily | | Sensex ETF. |
| SETFNIF50, SETFNIFBK | Yahoo | 2026-07-17 | 2026-11-04 | 76 each | daily | | **effectively no history**; not usable. |
| NIFTYIETF, MID150BEES | Yahoo | 2026-11-02 | 2026-11-06 | 5 each | daily | | **no history**; not usable. |
| **Nifty 50 index (price)** | Yahoo `^NSEI` | 2007-09-17 | 2026-11-06 | 4,696 | daily | price index, no dividends (**TRI not obtained**) | 1,333 zero-volume rows (index volume 0 for long stretches). Close equals the NSE file close on every overlapping date (2,404 dates, 2012-02-21..2021-12-31; 0 differ by > 0.01 point). Yahoo has 12 extra dates in that window that the NSE archive lacks (**cause unverified**). Earlier than 2007-09-17: **not obtained**. |
| Nifty 50 index (NSE files) | `nse-ind/NIFTY50.csv` from options-v1 `ind20*.csv` | 2012-02-21 | 2021-12-31 | 2,404 | daily | price index; also volume, turnover, P/E, P/B, div yield | name changes in the files: "S&P CNX Nifty" to 2013-02-07, "CNX Nifty" 2013-02-08..2015-11-06, "Nifty 50" from 2015-11-09 (merged into one series). Date format changes (`dd-mm-yyyy`, and `dd/mm/yyyy` on 174 rows; parsed). Stops 2021-12-31 because the 2022+ files are the options-v1 holdout and were **not downloaded and not opened**. |
| **Nifty Bank index** | Yahoo `^NSEBANK` | 2007-09-17 | 2026-11-06 | 4,711 | daily | price index | **Yahoo adjclose differs from close on 3,763 rows: use `close` for indices.** 23 Yahoo dates are absent from the NSE file over 2012-02-21..2021-12-31. |
| Nifty Bank (NSE files) | `nse-ind/NIFTYBANK.csv` | 2012-02-21 | 2021-12-31 | 2,404 | daily | price | "CNX Bank" to 2015-11-06, "Nifty Bank" after; one series. |
| **India VIX** | Yahoo `^INDIAVIX` | 2008-03-03 | 2026-11-06 | 4,577 | daily | n/a | 34 null rows dropped; volume always 0. Against the NSE file: 11 NSE dates missing from Yahoo, 16 Yahoo dates missing from NSE, **5 dates where close differs by > 0.01 (2018-11-07, 2020-04-28, 2020-06-17, 2021-02-12, 2021-03-30)**; cause unverified. |
| India VIX (NSE files) | `nse-ind/INDIAVIX.csv` | 2014-05-14 | 2021-12-31 | 1,864 | daily | n/a | **starts 2014-05-14, not 2014-10-01** as `data-feasibility-options.md` said; no VIX row in the earlier files. NSE's VIX history page (cookie-primed JSON API) not used. |
| Other NSE indices (Yahoo) | `^CNXIT`, `^NSMIDCP` (Next 50) from 2007-09-17; `^CRSLDX` (Nifty 500) 2005-09-26; `^CNX100` 2005-11-30; `^BSESN` (Sensex) 1997-07-01; sectors `^CNXAUTO`, `^CNXENERGY`, `^CNXFMCG`, `^CNXMETAL`, `^CNXPHARMA`, `^CNXPSUBANK` from 2011 | to 2026-11-06 | 3,757 to 7,231 | daily | price | `^CNXFIN` returned an empty body (**could not pull**). |
| **GOLDBEES** | Yahoo | 2009-01-02 | 2026-11-06 | 4,398 | daily | no events | 247 zero-open rows (2009-01-02..2010-01-18), 350 zero-volume days. |
| **LIQUIDBEES** | Yahoo | 2009-01-02 | 2026-11-06 | 4,399 | daily | **adjclose differs from close on 4,127 rows** (dividend adjustment; cash-like unit that distributes) | 2,830 zero-volume days (thin). How Yahoo treats the distributions: **unverified**. |
| Other gold/silver/liquid ETFs | Yahoo `SILVERBEES` 2022-02-04 (1,174 rows), `GOLDETF` 2023-09-01 (782), `GOLDIETF` 2023-12-12 (713), `LIQUIDETF` 2018-03-13 (2,137), `LIQUIDCASE` 2024-01-17 (692) | to 2026-11-06 | | daily | | short histories. |
| **USDINR** | Yahoo `USDINR=X` | 2003-12-01 | 2026-11-06 | 5,948 | daily | n/a | OTC quote with London-tz dates, 241 rows with inconsistent OHLC (a known Yahoo FX artefact), gap 2008-07-30 to 2008-08-25, zero volume. Not an RBI reference rate. |
| JPYINR, EURUSD | Yahoo `JPYINR=X`, `EURUSD=X` | 2003-12-01 | 2026-11-06 | 5,950; 5,951 | daily | | same FX caveats; EURUSD gaps 2008-07-30..08-07 and 2008-08-07..08-25. |
| Global cheap extras | Yahoo: `^GSPC` 1970-01-02 (14,335 rows), `^IXIC` 1971-02-05 (14,057), `^VIX` 1990-01-02 (9,283), `SPY` 1993-01-29 (8,502), `GLD` 2004-11-18 (5,527), `TLT` 2002-07-30 (6,109), `EEM` 2003-04-14 (5,931), `GC=F` 2000-08-29 (6,572), `CL=F` 2000-08-22 (6,580), `DX-Y.NYB` 1971-01-04 (14,181), `BTC-USD` 2014-09-17 (4,434), `ETH-USD` 2017-11-09 (3,285) | to 2026-11-06 | | daily | ETFs carry adjclose (dividend adjusted) | futures series are Yahoo continuous front-month with an **undocumented roll rule (unverified)**; US closes are New York 16:00, not IST. |
| **BTCUSDT spot** | Binance vision `spot/monthly/klines/BTCUSDT/1d` + `daily` for 2026-11 | 2017-08-17 | 2026-11-15 | 3,378 | daily (00:00-24:00 UTC) | raw (no adjustments exist) | 0 missing days, 0 checksum mismatches. Quote is USDT, not USD (depeg risk not modelled). Columns include `quote_volume`, `trades`. |
| **ETHUSDT spot** | same | 2017-08-17 | 2026-11-15 | 3,378 | daily | raw | same. |
| **BTCUSDT USD-M perp daily klines** | `futures/um/monthly/klines/BTCUSDT/1d` | 2020-01-01 | 2026-11-15 | 2,511 | daily | raw | perp trade price, not index/mark. Archive starts 2020-01-01 (contract listed 2019-09; pre-2020 not in archive). |
| ETHUSDT USD-M perp daily klines | same | 2020-01-01 | 2026-11-15 | 2,511 | daily | raw | same. |
| **BTCUSDT USD-M funding rate** | `futures/um/monthly/fundingRate/BTCUSDT` | 2020-01-01 | 2026-10-31 | 7,488 | 8-hourly | n/a | no gaps; **the latest month is not in the monthly archive and no daily funding files exist, so it stops 2026-10-31.** |
| **ETHUSDT USD-M funding rate** | same | 2020-01-01 | 2026-10-31 | 7,488 | 8-hourly | n/a | same. |
| BTCUSD_PERP / ETHUSD_PERP (COIN-M) funding | `futures/cm/monthly/fundingRate` | 2022-07-01 | 2026-06-30 | 4,269 each | 8h to 2022-12, then month-end gaps | n/a | archive spans only 2022-07 to 2026-06 (**why: unverified**); 47 gaps of 24 to 32 h at month ends in the gap report (missing end-of-month prints). Timestamps carry ms jitter. |
| BTCUSDT and ETHUSDT dated (delivery) futures, daily klines | `futures/um/monthly/klines/<SYM>_<YYMMDD>/1d` | BTC 2021-02-03, ETH 2021-02-04 | 2026-11-15 | 24 contracts per symbol (210326 to 261225, plus partial 270326), 51 to 208 rows each | daily | raw | 2020 contracts are not in the archive (4 probes per symbol returned nothing). **Some archives continue past the nominal last-Friday expiry as flat zero-volume rows** (230929 runs to 2023-11-14, 46 flat rows): drop rows after expiry. Funding for delivery contracts: not in the archive (**existence unverified**). Roll rule belongs in the master protocol. |

### Other liquid crypto pairs: spot daily klines + USD-M funding (first date = first date in the Binance archive)

| Pair | Spot first | Spot last | Spot rows | Funding first | Funding rows | Note |
| --- | --- | --- | --- | --- | --- | --- |
| BNBUSDT | 2017-11-06 | 2026-11-15 | 3,297 | 2020-02-10 | 7,367 | |
| LTCUSDT | 2017-12-13 | 2026-11-15 | 3,260 | 2020-01-09 | 7,463 | |
| ADAUSDT | 2018-04-17 | 2026-11-15 | 3,135 | 2020-01-19 | 7,432 | |
| XRPUSDT | 2018-05-04 | 2026-11-15 | 3,118 | 2020-01-06 | 7,472 | |
| XLMUSDT | 2018-05-31 | 2026-11-15 | 3,091 | 2020-01-19 | 7,432 | |
| TRXUSDT | 2018-06-11 | 2026-11-15 | 3,080 | 2020-01-15 | 7,445 | |
| ETCUSDT | 2018-06-12 | 2026-11-15 | 3,079 | 2020-01-16 | 7,442 | |
| LINKUSDT | 2019-01-16 | 2026-11-15 | 2,861 | 2020-01-17 | 7,439 | |
| ATOMUSDT | 2019-04-29 | 2026-11-15 | 2,758 | 2020-02-06 | 7,378 | |
| DOGEUSDT | 2019-07-05 | 2026-11-15 | 2,691 | 2020-07-10 | 6,914 | |
| BCHUSDT | 2019-11-28 | 2026-11-15 | 2,545 | 2020-01-01 | 7,488 | |
| SOLUSDT | 2020-08-11 | 2026-11-15 | 2,288 | 2020-09-13 | 6,793 | **funding interval is 2 h on 99 prints and 4 h on 2 prints** (interval column kept); all other pairs 8 h |
| DOTUSDT | 2020-08-18 | 2026-11-15 | 2,281 | 2020-08-20 | 6,791 | |
| UNIUSDT | 2020-09-17 | 2026-11-15 | 2,251 | 2020-09-17 | 6,707 | |
| AVAXUSDT | 2020-09-22 | 2026-11-15 | 2,246 | 2020-09-22 | 6,691 | |
| NEARUSDT | 2020-10-14 | 2026-11-15 | 2,224 | 2020-10-14 | 6,625 | |
| AAVEUSDT | 2020-10-15 | 2026-11-15 | 2,223 | 2020-10-16 | 6,620 | |
| FILUSDT | 2020-10-15 | 2026-11-15 | 2,223 | 2020-10-16 | 6,620 | |

All pairs: 0 missing days in spot klines, 0 funding gaps > 8.5 h (SOL's interval changes are shorter, not gaps), 0 checksum mismatches. "First date" is
the first archive date; the exchange's announced listing date was **not independently verified**. Survivorship: the pairs were chosen from today's liquid
set, so they are **survivors by construction** (collapsed or delisted coins are not included). Quote currency is USDT for all. Spot archive timestamps
switched from milliseconds to microseconds in 2025-01; the compact files normalise to UTC calendar dates.

## 2. Indian large caps and point-in-time membership

**Engine data already held** (`engine/data/`, from earlier tasks via yfinance `auto_adjust=True`, so split and dividend adjusted): `daily/` 49 files,
`daily-pit/` 85 files (union of members incl. dropped names), all **2011-01-03 to 2021-12-31** (2,711 rows for full-history names), plus `index/NSEI.csv`
(2011-01-03..2021-12-31, 2,691 rows) and `etf/NIFTYBEES*.csv`. Everything stops 2021-12-31. Hashes: `research-v1-manifest.csv`, `kind=engine-existing`.

**New in research-v1:** 88 NSE equities from Yahoo, `compact/yahoo-eq/<SYM>.csv` (+ `<SYM>.events.csv` with splits and dividends), full available history
(many from 1996-01-01, to 2026-11-06). `close` is split-adjusted, `adjclose` dividend-adjusted (differs from `close` for 87 of 88). Events: 156 splits, 2,653
dividends. Per-symbol rows and ranges are in the manifest (`yahoo-eq/`). Includes IDFCFIRSTB, LTM (LTIMindtree, from 2016-07-21), TMCV (245 rows from
2025-11-12) and TMPV (from 1991-01-02, so it carries pre-demerger Tata Motors history; the demerger adjustment is **unverified**).

**Point-in-time NIFTY 50 membership** is `docs/data/nifty50_members_by_date.csv`: 31 snapshots, 2012-01-01 to 2026-09-30, 95 distinct symbols, from the
Wikipedia replacement table walked back from today's list. **4 of 45 change rows were verified against NSE press-release PDFs; 41 are Wikipedia-only
(unverified).** Membership is keyed by the symbol as of the date; `data-ref/nifty50_price_aliases.csv` maps renamed symbols to the current price ticker
(SESAGOA and SESASTERLITE to VEDL, TATAMOTORS to TMPV, INFRATEL to INDUSTOWER, IBULHSGFIN to SAMMAANCAP, MCDOWELL-N to UNITDSPR; I used ZOMATO to ETERNAL and
LTIM to LTM as well). Caveats: the temporary 51st member DUMMYTATAM/TMCV (2025-10-14..2025-11-17) is not in the table; weights, free float and intra-period
corporate actions are not covered; no ISINs.

**Survivorship status.** The membership table is survivorship-safe (it contains dropped names). **Price coverage is not**: Yahoo has no history for 6 former
members, **CAIRN, HDFC (HDFC Ltd), IDFC, JPASSOCIAT, RANBAXY, STER**. By calendar days of membership that is about **3.3% of all member-days** (HDFC Ltd
alone 4,211 of 8,912 missing days), concentrated in 2012-2014 (roughly 10 to 11% of member-days in the 2012 and 2013 snapshots) and about 2% per year until HDFC
Ltd leaves on 2023-07-13. These names are missing exactly because they merged or delisted, so a universe backtest on available prices silently drops them:
survivorship bias of known size. Whether Yahoo's pre-2013 VEDL series is Sesa Goa's history is **unverified**; split-factor spot checks of long Yahoo
histories were **not done**. NSE cash bhavcopy (the route in `docs/hdfc-ltd-price-sources.md`, blocked earlier) was not tried.

## 3. NSE F&O bhavcopy already downloaded for options-v1 (dev window only)

`/Users/thakur/Workspace/Trading/data-raw/options-v1/` (7.1 GB; the zips are kept beside the extracted CSVs). Inventory taken from **file names and download
logs only; no file dated 2022 or later exists there, and none was downloaded or opened for this task.**

| Set | Files | First | Last | By year |
| --- | --- | --- | --- | --- |
| `fo/foYYYYMMDD.csv` (+ `.zip`) | 2,447 (+2,447 zips) | 2012-01-03 | 2021-12-31 | 2012:242, 2013:246, 2014:240, 2015:244, 2016:244, 2017:248, 2018:245, 2019:241, 2020:250, 2021:247 |
| `ind/indYYYYMMDD.csv` (all-index closes + India VIX) | 2,404 | 2012-02-21 | 2021-12-31 | 2012:209, 2013:246, 2014:238, 2015:236, 2016:243, 2017:248, 2018:245, 2019:241, 2020:250, 2021:248 |

Download logs: `fo` 2 absent days (404, holidays); `ind` 45 absent days (33 before the archive starts on 2012-02-21). Old-format layout (no `UndrlygPric`, no lot
size; see `data-feasibility-options.md` for the dead-row and `SETTLE_PR` problems and the lack of bid/ask). Under the options-v1 prereg the holdout is
2022-01-01 to 2026-10-31, single touch, so **no F&O or `ind` file after 2021-12-31 may be fetched or read for development**. File hashes:
`hardening/options` branch, `docs/prereg-options-v1-data-manifest.sha256`. Index levels past 2021 in this inventory come from Yahoo (index levels only, no
options data).

## 4. Conventions that matter when joining series

- **Dates**: Yahoo NSE series are dated by the IST calendar day (NSE close 15:30 IST). Yahoo FX is dated in `Europe/London`, US series in New York. Binance bars
  are UTC calendar days (05:30 to 05:30 IST), so BTC/ETH daily closes do not align with NSE closes; the alignment rule belongs in the master protocol.
- **Adjustments**: Yahoo `close` is split-adjusted only; `adjclose` includes dividends (equities, ETFs, US series). Index `adjclose` is unreliable (Nifty Bank
  differs on 3,763 rows): use `close`. The engine's pre-existing daily files are dividend-adjusted already.
- **Symbol changes**: only via the alias table; no ISIN mapping.
- **Zero and bad rows**: zero-volume days are kept (counts in the Yahoo source manifest); rows whose low/high do not bracket open/close are kept and counted
  (`bad_ohlc`): NIFTYBEES 1, sector ETFs 247 (the early zero-open prints), USDINR 241, HDFCBANK 28, TMPV 29.
- **Series ends**: Yahoo series stop 2026-11-06, Binance daily 2026-11-15 (the last row may be a just-completed UTC day), Binance funding 2026-10-31.

## 5. Could not pull (and why)

| Item | Reason |
| --- | --- |
| Price history for CAIRN, HDFC Ltd, IDFC (pre-IDFC First), JPASSOCIAT, RANBAXY, STER | Yahoo returns 404 (delisted or merged). No other free source passed the ToS rule: NSE cash bhavcopy and BSE would be systematic scraping, earlier probes were blocked and not retried. |
| Nifty / Nifty Bank / VIX before 2007-09-17 (Yahoo) and NSE `ind` files before 2012-02-21 | not offered (the 2012-02-15 and earlier probes returned 404); niftyindices.com historical download is cookie-gated and **not attempted**. |
| NSE `ind` and F&O bhavcopy from 2022-01-01 | deliberately not fetched (options-v1 holdout); post-2021 index levels come from Yahoo. |
| Nifty TRI (total return index) | no free source found or tried. |
| ETFs `ICICINIFTY`, `KOTAKNIFTY`, `UTINIFTETF`, `KOTAKBKETF`, `ICICIBANKN` (Yahoo) | 404; **my ticker guesses may be wrong (unverified)**. SETFNIF50, SETFNIFBK, NIFTYIETF, MID150BEES returned only 5 to 76 rows. |
| `^CNXFIN` (Nifty Financial Services) | empty response. |
| Binance dated futures before 2021-02; delivery-contract funding; COIN-M funding before 2022-07 and after 2026-06 | not in the archive. |
| Binance funding for 2026-11 | monthly archive not yet published; no daily funding files exist. |
| Delisted/collapsed crypto pairs (LUNA, FTT, etc.) | not attempted; the pair list is survivor-selected. |
| Intraday or tick data for anything | out of scope (daily bars only). |

## 6. Unverified items (summary)

1. Licence/ToS text of Yahoo, NSE archive files and data.binance.vision was not re-read (earlier docs say NSE/BSE feeds bar automation).
2. Binance exchange listing dates versus archive first dates; why the COIN-M funding archive spans only 2022-07 to 2026-06 and misses month-end prints.
3. Yahoo adjustment correctness: splits, demergers (TMPV/TMCV), merger renames (VEDL pre-2013, HDFC Bank), ETF dividends (NIFTYBEES shows none).
4. Yahoo's 12 extra Nifty dates, 23 extra Bank dates and the 5 VIX close mismatches versus NSE.
5. Point-in-time membership: 41 of 45 change rows unverified against NSE press releases; MCDOWELL-N symbol and rename dates approximate.
6. Whether post-expiry rows in delivery-contract archives are stale placeholders only (46 flat rows seen in the one file inspected; others not inspected).
7. Yahoo continuous-futures roll rule (`GC=F`, `CL=F`) and FX OHLC artefacts.
8. Existence/format of the 2022+ options holdout files (never touched).

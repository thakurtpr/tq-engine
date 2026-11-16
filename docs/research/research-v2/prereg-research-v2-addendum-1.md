# Addendum 1 to `prereg-research-v2` (data inventory facts, truncations, sealing plan)

Dated 2026-11-16, written and committed after the tag `prereg-research-v2` (annotated tag object deac03c, tagged commit cb1da79, tag time 2026-11-16 21:03:36 +05:30, master sha256 `0f36d2a5f7f8acafd9a19e747c1e9553374e7f10953cbd84b7708de755f1733c`, verified identical to `git show prereg-research-v2:docs/prereg-research-v2.md`). **The tag is untouched and is never moved.** No strategy has been run; no backtest was run for this addendum.

**What this addendum may and may not do** (master s.3.4 and s.0 rule 3). It (a) replaces provisional start dates with actual ones, truncation only; (b) records data-driven facts and data-handling rules that were not specified; (c) **changes no gate, threshold, split boundary (T/V/H dates), cap or the frozen candidate list.** Where a fact makes a frozen candidate unrunnable, the master's own rule applies (s.3.4(c)): the candidate is dropped and not counted; this addendum lists such cases but does not add or alter a candidate. Where this text and the tagged master differ, the tagged text governs and the difference is listed in section 7.

**Sources.** `docs/data-inventory-2026-11.md` and `docs/data/research-v1-{manifest,gap-report,yahoo-source-manifest,binance-qc,binance-rawfiles}.csv`, merged on `hardening/integration` (commit 544658a, built on `hardening/rdata`). Only ranges, row counts, gaps and bias notes were read; **the data agent computed no return, Sharpe or other performance statistic** (its one value-level check compared Yahoo `^NSEI` closes with the NSE archive files on 2,404 overlapping dates, an identity check on the same quantity). Daily bars only. Raw Yahoo JSON and Binance zips were deleted after conversion; their SHA-256 are in the manifests.

## 1. Actual start dates (replacing "expected" dates in master s.3.1)

Compact daily files are in `/Users/thakur/Workspace/Trading/data-raw/research-v1/compact/{yahoo,yahoo-eq,nse-ind,binance}` (outside git).

| Series | Source | First | Last | Rows | Note |
| --- | --- | --- | --- | ---: | --- |
| NIFTYBEES | Yahoo `NIFTYBEES.NS` | **2009-01-02** | 2026-11-06 | 4,399 | 1 zero-open row (2009-07-07); 175 zero-volume days; close = adjclose (no distribution events listed) |
| BANKBEES, JUNIORBEES, PSUBNKBEES, GOLDBEES | Yahoo | 2009-01-02 | 2026-11-06 | 4,398-4,399 | **247 zero-open rows 2009-01-02..2010-01-18**: first usable open **2010-01-19** |
| Nifty 50 price index | Yahoo `^NSEI` | **2007-09-17** | 2026-11-06 | 4,696 | price index, no TRI; label/regime/calendar use only (master 7.1) |
| Nifty Bank price index | Yahoo `^NSEBANK` | 2007-09-17 | 2026-11-06 | 4,711 | use `close`, never `adjclose` |
| Nifty 50 / Bank (NSE index files) | `nse-ind/NIFTY50.csv`, `NIFTYBANK.csv` | **2012-02-21** | 2021-12-31 | 2,404 each | stops at the options-v1 holdout boundary by design |
| India VIX | Yahoo `^INDIAVIX` | **2008-03-03** | 2026-11-06 | 4,577 | 5 close mismatches vs NSE file |
| India VIX | NSE files `nse-ind/INDIAVIX.csv` | **2014-05-14** (not 2014-10-01) | 2021-12-31 | 1,864 | corrects `data-feasibility-options.md` |
| BTCUSDT, ETHUSDT spot daily | Binance vision | **2017-08-17** | 2026-11-15 | 3,378 each | UTC days; 0 missing days; 0 checksum mismatches |
| BTCUSDT, ETHUSDT USD-M perp daily klines | Binance vision | **2020-01-01** | 2026-11-15 | 2,511 each | trade price; contract listed 2019-09 but archive starts 2020-01-01 |
| BTCUSDT, ETHUSDT USD-M funding (8 h) | Binance vision | **2020-01-01** | **2026-10-31** | 7,488 each | November 2026 not published |
| BTC/ETH dated (delivery) futures daily | Binance vision | BTC 2021-02-03, ETH 2021-02-04 | 2026-11-15 | 24 contracts each | rows after expiry are flat zero-volume placeholders: drop; 2020 contracts absent |
| 18 other USDT pairs, spot daily | Binance vision | BNB 2017-11-06, LTC 2017-12-13, ADA 2018-04-17, XRP 2018-05-04, XLM 2018-05-31, TRX 2018-06-11, ETC 2018-06-12, LINK 2019-01-16, ATOM 2019-04-29, DOGE 2019-07-05, BCH 2019-11-28, SOL 2020-08-11, DOT 2020-08-18, UNI 2020-09-17, AVAX 2020-09-22, NEAR 2020-10-14, AAVE 2020-10-15, FIL 2020-10-15 | 2026-11-15 | 2,223-3,297 | first date = archive first date (listing date unverified) |
| 18 other pairs, USD-M funding | Binance vision | between 2020-01-01 and 2020-10-16 (SOL funding interval 2 h/4 h on 101 prints) | 2026-10-31 | 6,620-7,488 | **no perp price klines were pulled for the 18 pairs** |
| 88 large-cap equities | Yahoo `yahoo-eq/<SYM>.csv` (+ events) | per symbol (many 1996-2002) | 2026-11-06 | see manifest | `close` split-adjusted, `adjclose` dividend-adjusted |
| engine daily files (PIT union 85 files, current 49) | `engine/data/daily*`, yfinance `auto_adjust=True` | 2011-01-03 | 2021-12-31 | 2,711 | already dividend-adjusted; end at the dev boundary |
| PIT NIFTY 50 membership | `docs/data/nifty50_members_by_date.csv` | **2012-01-01** | **2026-09-30** | 31 snapshots, 95 symbols | 4 of 45 change rows verified against NSE; 41 Wikipedia-only |
| NSE F&O bhavcopy (options-v1) | `data-raw/options-v1/fo`, `ind` | 2012-01-03 (`ind` 2012-02-21) | 2021-12-31 | 2,447 `fo`, 2,404 `ind` | no file dated 2022+ exists |
| USDINR | Yahoo `USDINR=X` | 2003-12-01 | 2026-11-06 | 5,948 | 241 inconsistent-OHLC rows; gap 2008-07-30..2008-08-25; London-tz dates |
| BTC-USD, ETH-USD | Yahoo | 2014-09-17; 2017-11-09 | 2026-11-06 | 4,434; 3,285 | see 1.2 |

### 1.1 Consequences of the actual dates (truncations only; boundaries unchanged)
* **IN-IDX tradables are the ETFs, not the price index** (master 7.1 forbids a price-only index as P&L). Effective start: NIFTYBEES 2009-01-02; BANKBEES 2010-01-19 (zero-open rows). Walk-forward first test: NIFTYBEES 2012-07-01 (11 folds in T, 8 in V, 19 total); BANKBEES 2013-07-01 (9 + 8 = 17). The master's "first test 2011-01-01, 22 folds" assumed the price index as the series and no longer applies to any tradable book. `^NSEI`/`^NSEBANK` from 2007-09-17 remain available for labels and warm-up only. The OG4 regime benchmark NIFTYBEES (close) has its expanding-median start 252 trading days after 2009-01-02.
* **IN-EQ:** T = 2012-01-01..2017-12-31 unchanged; first test 2015-01-01, 14 folds (6 + 8); prices before 2012-01-01 (engine 2011-01-03 and Yahoo 1996-2011) are **warm-up only** (master rule 0.8), never scored.
* **IN-OPT:** unchanged (2012-01-03..2021-12-31 development; first test 2015-01-01). Spot before 2012-02-21 from Yahoo `^NSEI` as in options-v1. Weekly-expiry cycles need re-reading the retained raw `fo` files (options-v1 retained monthly expiries only in its loader output; the zips are kept); weekly availability (BankNifty 2016-10, Nifty 2019-02) is from `data-feasibility-options.md` and **unverified**.
* **CRYPTO:** first test 2018-10-01 for BTC/ETH spot books (start 2017-08-17 + 12 months, next quarter start). **Fold arithmetic correction:** master s.3.2 says 33 folds (13 in T, 20 in V); counting complete quarters from 2018Q4 to 2026Q3 gives **13 in T, 19 in V, 32 total** (the quarter starting 2026-10-01 is incomplete: bars end 2026-10-31 in development, so it is not a test fold). Perp and funding books (R2 carry diagnostic) start 2020-01-01: first test 2021-01-01, 4 folds in T + 19 in V = 23. Alt pairs enter only when listed >= 365 days (master 6).
* **Crypto top-12 pool.** The universe is the **20 archived pairs** (BTC, ETH + 18), not all of Binance. Pairs with >= 365 days of listing: only BTC and ETH until 2018-11-06; **>= 10 pairs only from 2020-04-29** (the tenth is ATOM) and >= 12 from 2020-07-05 (DOGE). Before those dates the book holds all eligible pairs (fewer than 12). Interpretation fixed here without changing the master: the V2 diversification clause (master 2.5) is judged on the 20-pair pool at registration (>= 10 eligible symbols, so the clause applies and is evaluated on symbols that actually traded); the early folds with < 10 live members are reported, not excluded. If a reviewer reads "eligible" per fold the clause would fail early folds: **flagged ambiguity**.
* **IN-EQ large caps:** 88 Yahoo equities (+ engine files) cover 89 of the 95 PIT symbols after aliases; 6 missing (section 3).

### 1.2 The BTC-USD splice question (master s.3.1 CRYPTO row, 2.3 B2, 7.2)
Binance BTCUSDT/ETHUSDT spot start 2017-08-17, so the proxy extension to 2014-09-17 (Yahoo BTC-USD, chained by returns, costs doubled) is now relevant to exactly two places: the **B2 benchmark and the diagnostic cell set r2-diag-trend-idx (BTC-LONG)** for any window starting before 2017-08-17, and nothing else. **Decision (truncation): every GO-capable crypto book, every pooled book and every D-OOS gate uses Binance data from 2017-08-17 only** (D-OOS for crypto begins 2018-10-01 in any case, so the 2014-2017 proxy never enters a gated window except as train/warm-up of the diagnostic BTC-LONG cells). The extension stays as a **clearly labelled lower-quality proxy** (Yahoo composite price, doubled costs, no volume reliability), not a substitute for the Binance series. Whether Yahoo BTC-USD pre-2017 prices are tradable-quality is **unverified**.

## 2. Data-sealing plan the harness (r2gov) must implement before ANY strategy agent reads data

**Problem.** The compact files already contain bars beyond each class's development end: Yahoo series run to 2026-11-06, Binance daily to 2026-11-15 (funding to 2026-10-31), the PIT membership file to 2026-09-30. They sit in a plain directory readable by any agent. **No strategy agent may open `data-raw/research-v1/compact` or `data-raw/options-v1`.** At ingestion the harness splits every series at the class development end into a **development store** (the only path the loaders read) and a **sealed store** (a directory outside the repository and outside `data-raw`, e.g. `/Users/thakur/Workspace/Trading/data-sealed/`, permissions 000 with a hash-only manifest in git), then computes new development manifests from the development store only (row counts and last dates in the existing committed manifest already reveal how many bars follow the development end; the existing manifest is history and is not re-issued).

| Class / consumer | Development end (inclusive) | Sealed store receives |
| --- | --- | --- |
| IN-EQ (equities, engine files, events, aliases, PIT membership) | **2021-12-31** | all bars and events dated 2022-01-01 or later; membership snapshots and change rows dated after 2021-12-31 (the dev store carries the membership as of 2021-12-31) |
| IN-IDX (NIFTYBEES, BANKBEES, indices, India VIX, USDINR for India candidates) | **2021-12-31** | bars dated 2022-01-01 or later (to 2026-11-06) |
| IN-OPT (NSE `fo`/`ind` files; spot and VIX) | **2021-12-31** | nothing downloaded post-2021 (options-v1 holdout), plus Yahoo index/VIX levels dated 2022-01-01 or later |
| CRYPTO (spot, perp, funding, dated futures, 18 pairs, USDINR for crypto candidates, BTC-USD) | **2026-10-31** (no clean historical holdout: master 3.1; forward paper from the tag date 2026-11-16) | Binance bars dated 2026-11-01..2026-11-15 and Yahoo bars dated 2026-11-01..2026-11-06 (seen, non-gating; reserved as forward-paper comparison) |
| Any class, bars dated after 2026-10-31 | n/a | sealed (outside every H and every development window) |

Rules for the harness (added to master s.10 as implementation detail; no gate changes):
1. **Content-level maximum-date check** on every loaded bar against the class development end in the tagged prereg (not file names; equity files are one CSV per symbol).
2. **B1/B3 inside the sealed process.** Crypto candidates need B1 (NIFTYBEES) and B3 over 2022-2026-10 for OG2 in V-OOS, which lies inside the IN class holdout. The benchmark engine must run inside the harness and release only benchmark metrics for crypto windows, never the raw NIFTYBEES series (residual leakage of Indian buy-and-hold performance for 2022-2026 is accepted and disclosed; candidate designs are frozen at the tag, so it cannot steer them).
3. **Adjustment level leak (minor).** Yahoo `close` and `adjclose` levels embed splits and dividends that occur after the development end (a constant scale on earlier prices; daily returns inside the window are unaffected). The harness stores development prices with split/dividend events dated after the development end excluded where it can recompute them; where it cannot, the leak is disclosed. Event files (`*.events.csv`) after the development end are sealed.
4. **No quality statistics, row counts or last-bar dates beyond the development end** in development logs; quality checks on sealed data are deferred to the class holdout stage.
5. The sealed split is performed once, by the harness, and the development store manifest is committed before any candidate is registered; strategy agents get only the development store path.

## 3. Survivorship disclosure and the pre-registered handling

**Equities.** Yahoo has no price history for six former NIFTY 50 members: **CAIRN, HDFC (HDFC Ltd), IDFC, JPASSOCIAT, RANBAXY, STER**. They are about **3.3% of all member-days** overall and about **10-11% of member-days in the 2012 and 2013 snapshots**, about 2% a year thereafter until HDFC Ltd leaves on 2023-07-13. They are missing because they merged or delisted, so any universe backtest on available prices silently drops them.
**Handling (fixed now, no post-hoc substitution):**
1. A member with no usable price on a date **stays in the universe as unfillable**: never replaced by another stock, never given a proxy price, never silently dropped (master 7.5). Signals may not use it; positions cannot be opened in it.
2. Every IN-EQ result reports `coverage = member-days with a usable price / member-days total` **over the full development window AND over D-OOS AND per calendar year**. Master 7.5's 95% rule (below 95% = diagnostic only) is applied to **both** the full development window and D-OOS (the stricter reading of "book"). On the inventory figures overall coverage is about 96.7% (D-OOS from 2015-01-01 is higher), so the rule does not bite on counts; the 2012-2013 snapshots at about 89-90% are reported.
3. Every IN-EQ result carries the label **"long-only; upward bias from missing delisted names"**. Direction (unverified): CAIRN, JPASSOCIAT, IDFC and STER ended as large losers or distressed, whose absence flatters long-only trend, momentum and buy-the-dip books; RANBAXY and HDFC Ltd were absorbed in acquisitions and their effect is neutral to adverse. Net direction: **upward**.
4. Report-only sensitivity (never a gate): the same result with the 2012-2013 window excluded. No gate threshold is changed.
5. The membership table itself is survivorship-safe (it contains dropped names) but 41 of 45 change rows are unverified against NSE press releases; `MCDOWELL-N` and rename dates are approximate; SESAGOA to VEDL and TATAMOTORS to TMPV aliases are unverified (section 5).

**Crypto.** The 18 other pairs were chosen from today's liquid set: **survivors by construction** (collapsed or delisted coins such as LUNA and FTT are absent). Direction: **upward bias** for cross-sectional long-only books (trend, momentum, mean reversion, pairs). **Every crypto cross-sectional result carries the flag "survivor-selected universe; upward-biased" and cannot alone justify a promotion**: the historical gates are necessary, not sufficient, and a crypto candidate's promotion to **forward paper** (master 5.3) is on the unamended gates plus this flag, with the evidential weight resting on forward paper (V11). BTC and ETH alone are not survivor-selected in this sense (top two at every date in the archive), which is stated for the reader and not used as a gate.

## 4. Timing and alignment rules

* Binance daily bars are **UTC calendar days** (a bar dated D covers 00:00-24:00 UTC D, i.e. 05:30 IST D to 05:30 IST D+1). NSE closes at 15:30 IST = 10:00 UTC. Yahoo NSE series carry the IST calendar day, Yahoo FX the London date, US series the New York date.
* **Weekly Friday alignment (master 2.2) is confirmed consistent:** NSE Friday close (10:00 UTC Friday); crypto Friday 10:00 UTC where an intraday bar exists, else the **bar dated Friday**, which closes at 00:00 UTC Saturday, i.e. **14.0 h after the NSE close** (within the master's stated "<= 15.5 h" mismatch, reported with every cross-calendar result). Weekly returns are measured from the previous Friday cut.
* **Any book or benchmark holding both asset classes** (B3 is the only one in the frozen catalogue): decisions and rebalances use the **NSE trading day R**; the crypto leg trades at the open of the bar dated R (00:00 UTC R = 05:30 IST R, before the NSE open at 09:15 IST), the NSE leg at the NSE open of R. No signal in the frozen catalogue crosses asset classes; a cross-asset signal would have to use only crypto bars whose close precedes the NSE decision time (the bar dated D-1, closing 00:00 UTC D). Daily Sharpe of crypto books annualises on 365; cross-calendar comparisons are weekly (master 2.2).
* INR conversion: crypto return in INR = (1 + r_USDT)(1 + r_USDINR) - 1 with `USDINR=X` matched by date label (London-tz OTC quote; imperfect; filtered per master 7.2).

## 5. Known Yahoo and data issues the strategy agents must handle (and what is excluded or flagged)

| Issue | Handling |
| --- | --- |
| NSE indices: Yahoo `adjclose` differs from `close` (`^NSEBANK` 3,763 rows) | use `close`; price indices are never P&L (master 7.1) |
| Zero-open ETF rows (247 rows 2009-01-02..2010-01-18 for BANKBEES, JUNIORBEES, PSUBNKBEES, GOLDBEES; NIFTYBEES 1 row 2009-07-07) | a bar with open = 0 cannot fill at next-bar open: **no entry or exit may use such an open; the first usable bar is 2010-01-19** (BANKBEES) and the 2009-07-07 NIFTYBEES row is treated as no-fill (fill moves to the next valid bar) |
| Inconsistent OHLC (low/high not bracketing open/close): NIFTYBEES 1, sector ETFs 247, USDINR 241, HDFCBANK 28, TMPV 29 | rows are kept and counted; their open/high/low are untrusted: close is used, and a fill that needs such a bar's open moves to the next valid-OHLC bar; stops on those bars use the close |
| NIFTYBEES shows no dividend events; whether it distributes is **unverified** | B1 uses `close = adjclose`; if it did distribute, B1 total return is understated and OG2/V5 are easier than intended (bias disclosed, direction: favours candidates) |
| LIQUIDBEES: adjclose differs from close on 4,127 rows; 2,830 zero-volume days; distribution adjustment unverified | **excluded** (not in the frozen catalogue; not used as a cash proxy) |
| GOLDBEES and all gold/silver ETFs | **excluded** from the counted catalogue (non-equity-oriented tax treatment unverified, master 7.1); data exist but unused |
| TATAMOTORS membership via TMPV alias; TMCV from 2025-11-12; TMPV carries pre-demerger history (demerger adjustment unverified) | **flagged**: IN-EQ results report a sensitivity excluding TATAMOTORS/TMPV; the TMCV series post-dates the development end and is sealed |
| SESAGOA/SESASTERLITE to VEDL, MCDOWELL-N to UNITDSPR, INFRATEL, IBULHSGFIN aliases; ZOMATO to ETERNAL, LTIM to LTM | aliases unverified; pre-rename history unverified (VEDL before 2013 may not be Sesa Goa's); flagged, with the same sensitivity (exclude the aliased name) |
| Yahoo extra dates vs NSE archive (12 `^NSEI`, 23 `^NSEBANK`, 5 VIX close mismatches: 2018-11-07, 2020-04-28, 2020-06-17, 2021-02-12, 2021-03-30) | cause unverified; **calendar of record = NSE files for 2012-02-21..2021-12-31**, Yahoo elsewhere; VIX is not used as a signal in the frozen catalogue |
| USDINR: 241 bad-OHLC rows, 2008-07-30..2008-08-25 gap, London-tz dates, OTC quote (not an RBI rate) | used only for INR conversion on or after 2017-08-17 (crypto candidates); the 2008 gap lies before any crypto data; prints with a daily move above 3% are filtered (master 7.2) |
| Dated futures: archives continue after expiry with flat zero-volume rows; 2020 contracts absent; roll rule unspecified | drop rows after the nominal expiry; the roll rule is the diagnostic carry candidate's prereg to state; no frozen GO-capable candidate uses dated futures |
| Yahoo continuous futures (`GC=F`, `CL=F`), US ETFs, `^GSPC`, `SPY`, `GLD`, `TLT`, `EEM` | present in the inventory but **the GLOBAL class was dropped from the catalogue**: not used |
| COIN-M funding (2022-07..2026-06, month-end gaps) | not used by any frozen candidate |
| SOL funding interval 2 h / 4 h on 101 prints | the carry diagnostic and any funding-using book must use the printed interval column |

## 6. Series that could not be obtained

CAIRN, HDFC Ltd, IDFC (pre-IDFC First), JPASSOCIAT, RANBAXY and STER price histories (Yahoo 404; no free source passing the ToS rule); Nifty/Nifty Bank/VIX before 2007-09-17 (Yahoo) and NSE `ind` files before 2012-02-21; NSE `ind` and F&O files from 2022-01-01 (deliberately not fetched); Nifty total-return index (no free source); ETFs ICICINIFTY, KOTAKNIFTY, UTINIFTETF, KOTAKBKETF, ICICIBANKN (404, ticker guesses may be wrong) and effectively empty SETFNIF50, SETFNIFBK, NIFTYIETF, MID150BEES; `^CNXFIN` (empty); Binance dated futures before 2021-02, delivery-contract funding, COIN-M funding before 2022-07 and after 2026-06, **funding for 2026-11**; **perp daily klines for the 18 other pairs**; delisted/collapsed crypto pairs; **any intraday or tick data for NSE equities (no 5m bars)**; intraday crypto bars are not in the research-v1 inventory (the earlier engine files `engine/data/intraday/BTCUSDT|ETHUSDT` 1m 2022-2023 and `perp-1h` exist but were not inventoried and their ranges are unverified here).

**Consequences for frozen candidates (master s.3.4(c): dropped and not counted; the catalogue itself is not edited):**
* `r2-intraday-nse-{orb,vwap}` (18 cells): the master made them conditional on >= 3 years of free 5m bars; **none were obtained: NOT RUN, NOT COUNTED.**
* `r2-diag-intraday-cr` (9 cells): runs only if the harness inventories hourly/1m bars (ranges only, development window) and they fall in development windows; otherwise NOT RUN, NOT COUNTED. The existing 1m files lie in 2022-2023, which is seen development data.
* `r2-pairs-cr` (27 cells): **perp price klines for the 18 pairs were not pulled.** The book may take spot prices for signals and P&L with the perp cost and the actual funding prints for the short leg (basis between spot and perp ignored: **unverified, optimistic**); if the pre-registering agent judges that invalid, the candidate is dropped and not counted. Any such use must be stated in its prereg.
* `r2-diag-carry`: usable on BTC/ETH perp daily klines and funding from 2020-01-01 (funding ends 2026-10-31); SOL-type interval changes handled via the interval column.
* `r2-diag-options-weekly`: needs weekly rows from the retained raw `fo` files (section 1.1); if weekly rows are absent in the old-format files for a cycle, the cycle is skipped and counted (as options-v1).
* The N accounting of master s.4 is unchanged: a candidate not run opens no sweep and adds no cell (the caps and the reserve stand).

## 7. Conflicts with, and clarifications of, the tagged text (the tag is NOT edited)

1. **IN-IDX start and fold counts:** master "expected 2007-09-17; first test 2011-01-01; 22 folds" holds only for the price index; tradables (ETFs) start 2009-01-02 / 2010-01-19 with 19 and 17 folds (1.1). Truncation.
2. **CRYPTO fold arithmetic:** master "33 folds, 20 in V" should read 32 folds, 19 in V (1.1). A counting correction; no boundary moved.
3. **Perp book dates:** perps and funding start 2020-01-01 (first test 2021-01-01), not 2019-09 / 2019-11 as the master's expected dates say. Truncation.
4. **India VIX NSE start** is 2014-05-14 (not 2014-10); VIX is not used by any frozen candidate.
5. **BTC-USD proxy:** retained only for B2 and the diagnostic BTC-LONG cells; all other crypto gates use Binance from 2017-08-17 (1.2). A narrowing of use, not a change of any cell.
6. **Crypto top-12 universe** is the 20-archive pool with fewer than 12 (and fewer than 10, before 2020-04-29) eligible members early (1.1): master s.6 and s.2.5 are unchanged; interpretation stated.
7. **Coverage rule (master 7.5)** is applied to both the full development window and D-OOS (stricter reading) (3).
8. **B1/B3 for crypto windows** requires NIFTYBEES 2022-2026 inside the sealed process: master 2.2 (all three benchmarks for every class) conflicts with 3.1 (IN holdout clean) unless the benchmark engine releases metrics only; stated in 2, rule 2. Residual leak of Indian buy-and-hold performance is disclosed.
9. **Frozen candidates not run** for lack of data (`r2-intraday-nse-*`, possibly `r2-diag-intraday-cr`, possibly `r2-pairs-cr`): handled by master s.3.4(c); the frozen list is not edited and no cell is added.
10. **Development-end dates to seal at:** IN-EQ, IN-IDX, IN-OPT **2021-12-31**; CRYPTO **2026-10-31** (no historical holdout; forward paper from 2026-11-16); everything dated after 2026-10-31 sealed for all classes.

## 8. Still unverified (carried from the inventory and this addendum)
Yahoo/NSE/Binance licence text; Binance listing dates vs archive first dates; Yahoo split/demerger/dividend adjustments (TMPV/TMCV, VEDL, ETF distributions); PIT membership (41 of 45 change rows); weekly-option availability dates; whether Yahoo pre-2017 BTC-USD is tradable-quality; direction of net survivorship bias (stated, not measured); interpretation of the V2 diversification clause per fold for the crypto pool; the effect of the adjustment-level leak; the 12-day, 23-day and VIX discrepancies between Yahoo and NSE files.

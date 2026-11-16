# Data feasibility: NSE index options (Nifty, BankNifty) for backtesting

Researched 2026-11-06. Paper and backtest only. Nothing here is a result. Raw files stay outside git (scratchpad); only
a hash manifest and a 24-row excerpt are committed (`docs/data/options-sample-manifest.csv`, `docs/data/options-sample-excerpt.csv`).

## Verdict

| Need | Free and legal source | Status |
| --- | --- | --- |
| End-of-day option prices, OI, volume per strike and expiry, point-in-time | NSE F&O bhavcopy (public archive files) | **EXISTS, fetched and verified** |
| Daily index close (Nifty 50, Nifty Bank), daily India VIX | NSE `ind_close_all_DDMMYYYY.csv` archive files | EXISTS (verified; India VIX row present from at least 2014-10-01) |
| Intraday (1 min or tick) option prices | none free | **UNAVAILABLE** |
| Bid/ask, depth, quote history | none free | **UNAVAILABLE** |
| Historical implied volatility and Greeks | none published; must be derived | **UNAVAILABLE as data** (derivable from EOD prices, see limits) |
| Underlying price inside the option file | only from 2024-07 format (field `UndrlygPric`) | partial; take it from the index file before |

So a defined-risk, **end-of-day, rule-based** options test is feasible with free data. Anything that needs intraday fills,
quotes or a vendor IV surface is not testable without paid data (section 5).

## 1. Source and how it was fetched

Plain HTTPS GET (browser User-Agent, no login, no cookies) from the NSE public archive host:

- Old format (`CM`-style `fo` bhavcopy), 2000-06 to 2024-07-05:
  `https://nsearchives.nseindia.com/content/historical/DERIVATIVES/<YYYY>/<MON>/fo<DD><MON><YYYY>bhav.csv.zip`
  (`archives.nseindia.com` serves the same files). One zip holding one CSV.
- New UDiFF format, from 2024-07-05 (both formats exist for 2024-07-05; old stops, 2024-07-08 old URL is 404):
  `https://nsearchives.nseindia.com/content/fo/BhavCopy_NSE_FO_0_0_0_<YYYYMMDD>_F_0000.csv.zip`
- Index closes and India VIX: `https://archives.nseindia.com/content/indices/ind_close_all_<DDMMYYYY>.csv`
  (one row per index incl. `Nifty 50`, `Nifty Bank`, `India VIX`).

Probes (HTTP status of a GET): old format 2000-06-12, 2001-06-12, 2002-01-02, 2005-01-03, 2008-01-02, 2010-01-04, 2016-10-03, 2024-07-05 all 200/206;
holidays and weekends return 404 (2019-03-04 Mahashivratri returned an HTML 404 body saved as `.zip`: validate the zip before use). `HEAD` requests return 503; use `GET`.
First Nifty option session in the archive: **2001-06-04** (30 Nifty option rows). Latest session available when probed: 2026-11-05 (2026-11-06 was 404, not yet published).
Index close files: 200 for 2012-06, 2012-09, 2012-12, 2013-07, 2014-10 ... 2025-09 (the 2012-01-03 date was 404, so the set has gaps or starts mid-2012; India VIX row not present in 2012-2013 files probed, present from 2014-10-01).
**Not done:** the NSE "historical India VIX" report page uses a cookie-primed JSON API; automated cookie priming was blocked in this session and is not used. The VIX series via `ind_close_all` replaces it from 2014.
Full history size estimate: about 6,000 sessions at 0.3 to 1.1 MB zipped = 3 to 4 GB. Download politely (sequential, 0.5 s pause), outside git.

## 2. Sample actually downloaded

Nine sessions (hashes in the manifest): 2019-03-01, -05, -06, -07 (old format; 2019-03-04 holiday) and 2025-10-27 to 2025-10-31 (new format). Plus format probes of 2001-06-04, 2005-01-03, 2012-01-02, 2016-10-03, 2023-01-02, 2024-07-05, 2024-08-01, 2026-01-05 (row counts and field names only).

| Item | Old format (2019-03 sample) | New format (2025-10 sample) |
| --- | --- | --- |
| Rows, all F&O, per day | 36,472 / 36,526 / 36,726 / 36,908 | 37,407 / 37,711 / 33,624 / 31,281 / 32,095 |
| NIFTY option rows per day | 3,150 to 3,192 | 1,566 to 1,630 |
| BANKNIFTY option rows per day | 802 to 850 | 921 to 1,005 |
| Date range of sample | 2019-03-01 to 2019-03-07 | 2025-10-27 to 2025-10-31 |
| Nifty expiries listed on one day | 21 (weeklies and monthlies) | 18 |
| Strike range (Nifty) | 4,600 to 14,100 | 12,000 to 31,000 |

Fields, old format: `INSTRUMENT` (`OPTIDX`), `SYMBOL`, `EXPIRY_DT`, `STRIKE_PR`, `OPTION_TYP` (CE/PE), `OPEN`, `HIGH`, `LOW`, `CLOSE`, `SETTLE_PR`, `CONTRACTS` (volume), `VAL_INLAKH`, `OPEN_INT`, `CHG_IN_OI`, `TIMESTAMP`.
Fields, new format: `TradDt`, `FinInstrmTp` (`IDO` index option), `TckrSymb`, `XpryDt`, `StrkPric`, `OptnTp`, `OpnPric`, `HghPric`, `LwPric`, `ClsPric`, `LastPric`, `PrvsClsgPric`, **`UndrlygPric`**, `SttlmPric`, `OpnIntrst`, `ChngInOpnIntrst`, `TtlTradgVol`, `TtlTrfVal`, `TtlNbOfTxsExctd`, `NewBrdLotQty` (lot size), `FinInstrmNm`.
Index-option symbols seen: NIFTY, BANKNIFTY, FINNIFTY, MIDCPNIFTY, NIFTYNXT50 (2024), plus older sector and foreign index options (NIFTYIT, FTSE100, S&P500 in 2016).

## 3. Quality findings from the sample (these drive the pre-registration)

1. **Dead rows.** Old-format 2019 sample: 90.6% of NIFTY and 74.9% of BANKNIFTY option rows have zero volume; 78.7% and 72.4% zero open interest. 2025 sample: 34.2% and 50.7% zero volume. Far strikes list a `CLOSE` carried from earlier days and a `SETTLE_PR` that the exchange computes: `CLOSE != SETTLE_PR` on 91% of 2019 NIFTY rows. **`SETTLE_PR` of an untraded contract is not a price anyone could trade at.** Fills must require volume > 0 and a non-zero `OPEN`.
2. **No bid/ask.** Only OHLC, last/close, settle. The spread must be assumed (see `docs/costs-tax-options-india.md`); results are sensitive to it and that is stated up front.
3. **No underlying in old files.** Spot comes from the index close file (15:30 close); option `CLOSE`/`SETTLE_PR` are last-trade or last-half-hour VWAP, so implied vol derived from them carries timing noise. Put-call-parity forward from the same chain is an alternative.
4. **No lot size in old files.** Lot-size history (from memory, unverified: Nifty 50, 25, 75, 65 at various dates; BankNifty similar) must come from NSE circulars and be committed as a hashed file before any run. New format has `NewBrdLotQty` only from 2024-07.
5. **Expiry structure changed.** Nifty weekly expiries from 2019-02; BankNifty weeklies existed from 2016-10 and the 2025-10 sample lists only monthly and longer BankNifty expiries (weeklies withdrawn; date from memory, about 2024-11); weekday of expiry moved (Thursday, later Tuesday for Nifty from 2025-09 per the 2025-10 sample expiry 2025-10-28 and Nifty weeklies dated Tuesdays). Contract month and weekly must be inferred from the expiry dates in each file, never from a calendar rule.
6. **EOD only.** One observation a day. Entry/exit is limited to the daily open (first trade) or settle price. Intraday risk (gaps through a wing, stop slippage) is not observable; stops can only be evaluated on the daily settle.
7. **Point in time.** Each file is that day's snapshot, published after close, not re-downloaded at a later date to test for revisions (unverified; re-fetch a few dates at run time and compare hashes). An index has no constituent survivorship issue for index options.
8. **Open of untraded or illiquid strikes is 0 or stale.** `OPEN = 0` appears with `CONTRACTS = 0`.

## 4. Do not claim from this data

No intraday edge, no IV-percentile or skew strategies on vendor-quality IV, no "sell at 9:20" structures, no stop-loss intraday accuracy. India VIX before 2014 was not verified from the free files.

## 5. Paid or gated alternatives (names from memory; not verified, no prices fetched)

Vendors that sell NSE tick or 1-minute historical options data include TrueData, Global Datafeeds (GDFL) and NSE's own data products; broker historical APIs (Zerodha Kite, Dhan, Upstox) need an account and credentials (excluded by hard rule 1) and typically do not return expired contracts. If intraday or IV-based work is wanted, a quote and budget are the owner's decision.

## 6. What the free data does and does not allow

Allowed: monthly (or weekly from 2019) defined-risk structures priced at the daily open/settle, 2012 to 2026, Nifty and BankNifty, with a liquidity filter and an assumed spread.
Not allowed: anything intraday; IV-surface strategies at vendor quality; evaluation of fills better than "next-day open plus assumed slippage".

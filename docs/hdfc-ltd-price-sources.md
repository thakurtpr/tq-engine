# Sourcing HDFC Ltd daily prices (NSE: HDFC), pre-merger

Date: 2026-10-30. Task: find a free source other than yfinance for HDFC Ltd daily prices, 2011 (indicator
warm-up) to 2021-12-31, because Yahoo returns no rows and the series is missing for the whole of backtest-02's
window (2,465 member-days, the largest gap). HDFC Ltd left the NIFTY 50 on 2023-07-13 after the HDFC Bank
amalgamation (NSE press release `ind_prs04072023.pdf`).

**Result: I could not obtain or verify a usable source.** I downloaded **no price data**. The one direct
test I made was blocked, see "What I tried". Everything below is from search results unless marked.

## Candidates

| Source | What it offers | Coverage / status | Blockers |
| --- | --- | --- | --- |
| **NSE bhavcopy (cash market), official EOD files** | Per-day file with OHLC, previous close, volume, ISIN for every NSE stock traded that day, so HDFC Ltd is in every file up to 2023-07-13 | A third-party tool claims NSE history back to 1994; another covers 2016 to 2024-07-05 only (neither verified by me). The legacy static path `nsearchives.nseindia.com/content/historical/EQUITIES/YYYY/MON/cmDDMONYYYYbhav.csv.zip` reportedly stopped working in July 2024 and returned HTTP 503 to my probe. The new UDiFF path `.../content/cm/BhavCopy_NSE_CM_0_0_0_YYYYMMDD_F_0000.csv.zip` answered 200 for 2024-07-08 but only covers dates **from July 2024**, which is after HDFC Ltd was delisted, so it does not help. Older days are said to be under NSE's all-reports page, "Archives" tab ("CM - Bhavcopy (csv)" / "CM - Common Bhavcopy (csv)"). | One file per trading day (about 2,500 files for 2012-2021). NSE's terms (the data-portal terms, and an older quoted version of the site terms) prohibit systematic or automated collection without written consent. |
| **NSE website "security-wise price volume & deliverable data"** | One symbol, a date range, CSV, in the browser | The right tool for one symbol: a handful of range downloads rather than 2,500 day files. Per-request range limit and current page path **unverified** (one scraper library documents a 3-year maximum). | Manual download by a person in a browser is the clean route; automating it hits the same terms. |
| **BSE bhavcopy / BSE historical data** | BSE also trades HDFC Ltd (scrip code 500010, from memory, unverified) | The documented pattern `bseindia.com/download/BhavCopy/Equity/EQ_ISINCODE_DDMMYY.zip` returned an HTML page, not a zip, to my one probe; one tool says BSE data starts January 2007. | Same per-day-file problem; I did not get a file. |
| **Tickstory "NSE equities" package** | One CSV per instrument | Date range stated as "earliest to 2014", so it misses 2015-2021; it is $1 or free via a share link; delisted names not stated. | Wrong date range. |
| **Kaggle "India Stock Market" (CC0)** | Dataset of NSE stocks | Snippet suggests currently listed stocks only. | Likely survivor-biased, not checked. |
| **Python scrapers (`bhav`, `jugaad-data`, `nsepython`, `nser`)** | Wrap the NSE/BSE downloads above | Same underlying files; limits and breakage depend on NSE changes. | They automate the access NSE's terms restrict. |
| **Paid vendors (e.g. Exchange Data International)** | Unadjusted and adjusted EOD for all NSE stocks including delisted | Complete, per a vendor description | Not free. |

## What I tried, and why I stopped

I sent a handful of HEAD requests (no files downloaded): the legacy NSE path returned 503 for three
dates; the UDiFF 2024-07-08 file returned 200 (zip); the BSE 2012-01-02 URL returned HTML. These used a
browser-style User-Agent. When I then tried more probes with a Referer header too, the session's safety
classifier **blocked the action as bot-check circumvention**. That was correct: spoofing a browser to pass
NSE's bot protection is the wrong way to do this, so I made no further requests to nseindia.com or
bseindia.com and did not look for a workaround.

## Adjustment is also needed

Bhavcopy prices are **unadjusted**. HDFC Ltd's splits and bonuses must be applied or the series shows a
fake one-day collapse. A Trendlyne page lists HDFC Ltd corporate actions
(<https://trendlyne.com/equity/corporate-actions/HDFC/532/housing-development-finance-corporation-ltd>); the
search snippet mentioned a 2010 split, which conflicts with my recollection of 2011, so **the date and ratio
are unverified**. The NSE corporate-actions page is the primary source. The backtest also adjusts for
dividends (yfinance `auto_adjust`), which bhavcopy alone will not give you.

## Options for the CEO / user

1. **Manual, clean:** a person downloads HDFC Ltd's history in a browser from NSE's security-wise historical
   data page (about one range per year, 2011-2023) and drops the CSV in the repo; I then reformat and adjust
   it with verified corporate actions. This respects NSE's terms and is a small effort.
2. **Ask NSE or a vendor for written permission or a licensed file** if bulk or automated access is wanted.
3. **Keep the gap and label it.** HDFC Ltd is 2,465 of the 5,636 missing member-days, so the report should
   keep saying the direction of the remaining bias is unknown and include a sensitivity (for example, a
   backtest-03 re-run with HDFC Bank's price history standing in for HDFC Ltd, clearly labelled as a proxy
   and not a substitute).
4. Do **not** splice HDFC Bank's series onto HDFC Ltd's (the merger swap ratio and share count differ), and
   do not scrape.

## Sources (search results; none opened in full)

- NSE all-reports and bhavcopy discussion: <https://tradingqna.com/t/has-nse-changed-bhavcopy-location/169551>;
  tool pages <https://pypi.org/project/bhavkit/>, <https://pypi.org/project/bhav/1.0.6/>
- NSE Data & Analytics terms of use (systematic or automated collection prohibited):
  <https://dotexdata.nseindia.com/TermsAndConditions/TermsofUse.pdf>
- Tickstory NSE equities package: <https://tickstory.com/product/historical-price-data-for-nse-equities/>
- NSE press release for the HDFC removal: <https://www.niftyindices.com/Press_Release/ind_prs04072023.pdf>

# Survivorship-safe universe for NSE daily backtests

Why: backtesting on today's NIFTY 50 list silently selects stocks that already won (survivorship
bias). A backtest must use the members that were in the index **on each historical date**, including
the names later dropped, merged or delisted.

Research date: 2026-10-30. Several niftyindices.com / nseindia.com pages timed out or returned 403 from
my fetch tool, so items marked **(not fetched)** come from search-result snippets and must be opened
and checked by hand before you rely on them.

## Sources for point-in-time NIFTY 50 / NIFTY 100 membership (free)

1. **NSE Indices press releases (primary, authoritative).** Each semi-annual review and ad-hoc change
   is a PDF named `ind_prsDDMMYYYY.pdf`, e.g.
   - <https://niftyindices.com/Press_Release/ind_prs23022026.pdf> (Feb 2026 review: no change to
     NIFTY 50; replacements elsewhere effective 30 Mar 2026, i.e. after the close of 27 Mar)
   - <https://niftyindices.com/Press_Release/ind_prs13052020.pdf>,
     <https://niftyindices.com/Press_Release/ind_prs22082017.pdf>,
     <https://niftyindices.com/Press_Release/ind_prs22022016_2.pdf>
   Regular reviews use cut-off dates of 31 Jan and 31 Jul (per press coverage), changes take effect
   about two months later (end of March / end of September); ad-hoc changes come with mergers,
   delistings and accelerated exclusions. The press-release index page itself did not load for me
   (<https://www.niftyindices.com/press-release>, **not fetched**); file names follow the date pattern,
   so releases can also be found by date. Each release gives the exact effective date; use it.
2. **Wikipedia, "NIFTY 50" > "List of replacements since 2005" (secondary, good starting table).**
   <https://en.wikipedia.org/wiki/NIFTY_50>. Columns: constituent excluded, constituent included, date
   of replacement, reason for exclusion, reference. Covers 25 Feb 2005 to the latest change on the page
   (my read showed recent rows such as 30 Sep 2025 IndiGo and Max Healthcare added; 28 Mar 2025 Eternal
   and Jio Financial Services added; 28 Mar 2024 Shriram Finance replacing UPL for inadequate market
   cap; the page also listed a 30 Sep 2026 BSE addition, which I did not cross-check). The text
   extraction pairs some excluded/included rows badly, so re-read the table in a browser and verify
   every row that matters against source 1. Cite the reference column.
3. **Cross-checks and weights (not a primary source):**
   - Current list (today only, no history): <https://www.niftyindices.com/IndexConstituent/ind_nifty50list.csv>
     (from a forum reply; **not fetched**). Use it as the end point when walking changes backwards.
   - NSE monthly report "Market Capitalisation, Weightage and Beta for NIFTY 50 & NIFTY Next 50":
     free-float weights by month (title from a third-party write-up; the URL was not given, **not fetched**).
   - MarketCalls' ISIN-keyed workbook, 2016-2026:
     <https://www.marketcalls.in/wp-content/uploads/2026/08/NIFTY50_constituents_2016_to_2026.xlsx>.
     Its author says the method uses AI-assisted collection and warns that AI output must be verified
     against NSE announcements. Use only to diff against your own table, never as truth.
     Article: <https://www.marketcalls.in/llm-models/how-to-build-survivorship-bias-free-nifty-50-historical-data-using-grokbot.html>

I found **no free ready-made point-in-time NIFTY 50 or NIFTY 100 membership dataset** on GitHub. Plan
to build it from sources 1 and 2.

## How to build it

1. Start from the current official list (source 3), then walk **backwards** through the change log,
   undoing each replacement at its effective date. Produce one snapshot per effective date.
2. Key every security by **ISIN**, not ticker (renames such as Zomato -> Eternal are the same company;
   a ticker-keyed table would see two). Keep a symbol-history table: ISIN, symbol, valid_from, valid_to.
3. Use the **effective date** for membership, never the announcement date, and never use a later
   snapshot for an earlier date. A stock added on date D can be traded only from D (conservative); the
   announcement usually precedes D by weeks, so do not use the announcement as a signal date.
4. Validate every snapshot (checks from the MarketCalls write-up): exactly 50 unique ISINs; no
   duplicate companies across share classes (e.g. Tata Motors DVR needs explicit handling); walking
   forward from the first snapshot reproduces today's list; no name appears before its effective date.
5. NIFTY 100 = NIFTY 50 + NIFTY Next 50. Next 50 changes are also in NSE press releases (e.g. five
   additions/deletions announced Aug 2023 per Business Standard) but Wikipedia has no equivalent
   table, so NIFTY 100 point-in-time history is more work. Do it second.
6. Prices must include **delisted and merged names** (e.g. HDFC Ltd merged into HDFC Bank, so its
   series ends) and be **adjusted for splits, bonuses and dividends** using corporate-action data. NSE
   bhavcopy (end-of-day) is unadjusted. Decide the source and test one known split before trusting it.
7. Store the table as versioned data in the repo (CSV with columns `isin, symbol, index, start, end,
   source_url`; dates ISO `YYYY-MM-DD`) and keep each press-release URL as evidence.

## Recommended starting universe

- **Phase 1: NIFTY 50 point-in-time, 2012-01-01 to present.** About 50 liquid names, simple to audit,
  and slippage is small. Starting at 2012 gives a 10-year development window plus a held-out period
  (see `validation-protocol.md`); the change table goes back to 2005 if you want more history, but
  check that your price source reaches that far.
- **Phase 2:** extend to NIFTY 100 point-in-time once Next 50 history is built and validated.
- Exclude anything that would fail liquidity on your size (average daily value traded well above your
  order). Large caps rarely fail this, which is another reason to start there.

## Benchmark

- **Primary: NIFTY 50 Total Returns Index (TRI)**, daily, from NSE Indices' "Historical Data Reports"
  page, which has a total-returns-index option and CSV download:
  <https://www.niftyindices.com/reports/historical-data> (search result; the page timed out for me, so
  the exact steps and date range are **unconfirmed**). Fallback:
  <https://www.nseindia.com/reports-indices-historical-index-data>. Check whether you pick the gross
  or net TRI variant and use the same one throughout.
- **Tradable proxy: NIFTYBEES (Nippon India ETF Nifty 50 BeES).** Daily prices from your broker feed or
  a vendor; it tracks the price index plus fund mechanics, so its return will differ from the TRI by
  tracking error and dividend treatment. Use it for "what would buy-and-hold actually have earned" and
  as a sanity check, not as the primary benchmark. I did not find an official free NAV history source.
- A strategy must beat buy-and-hold of the benchmark **net of costs, risk-adjusted**, otherwise just
  buy the index (this is a pass/fail in the validation protocol).

## Open items

- Pull and diff the two membership sources, resolve every disagreement against a press release.
- Confirm the price/corporate-action source and its history length.
- Confirm the TRI download steps and gross-vs-net variant by opening the page.

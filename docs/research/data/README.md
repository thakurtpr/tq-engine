# Point-in-time NIFTY 50 membership (2012-01-01 to 2026-10-30)

Built 2026-10-30 from the Wikipedia article "NIFTY 50" ("List of replacements since 2005" plus the
current constituents table), walking back from today's 50 members. Secondary source: **verify against
NSE Indices press releases before relying on any row** (see `../universe.md`).

| File | Columns | Content |
| --- | --- | --- |
| `nifty50_changes.csv` | `effective_date,added_symbol,removed_symbol,source_url` | 45 replacement pairs, 2012-04-27 to 2026-09-30. `source_url` is the reference the Wikipedia row cites (mostly news articles, a few NSE files); the 2026-09-30 row had none, so I attached a news report of the NSE Indices announcement. |
| `nifty50_members_by_date.csv` | `date,symbol` | The 50 members on each change date (31 snapshots, 1,550 rows), starting with a 2012-01-01 snapshot. A snapshot holds from its date until the next one. |
| `nifty50_symbol_notes.csv` | `symbol,company,note,certainty` | Renames, mergers and caveats. |
| `build_nifty50_membership.py` | n/a | Reproduces all three files from the Wikipedia HTML. |

Symbols are NSE trading symbols **as of the snapshot date** (e.g. `HDFC` until 2023-07-13, `TATAMOTORS`
until 2025-10-24 then `TMPV`). Key analysis by company/ISIN, not symbol: renames and mergers
(`HDFC` -> `HDFCBANK`, `STER`/`SESAGOA`/`SESASTERLITE`/`VEDL`, `RANBAXY` -> `SUNPHARMA`, `INFRATEL`) are in the
notes file. No ISINs are included (I could not source them reliably).

## Validation (all run by the build script)

- Every one of the 31 snapshots has exactly 50 members and 50 unique symbols.
- Walking back and replaying forward give identical snapshots on every change date, and the forward
  replay from 2012-01-01 reproduces today's list.
- Every current member's "date added" in the Wikipedia constituents table either appears as an
  inclusion in the change table (if on/after 2012-01-01) or the member is in the 2012-01-01 snapshot.
- 0 unreconciled rows in the change table; the script aborts if any company name has no symbol mapping.
- Spot check: the 2012-01-01 snapshot (SESAGOA, STER, RCOM, RPOWER, DLF, JPASSOCIAT, SAIL, SIEMENS,
  RELINFRA, RANBAXY, CAIRN, PNB, IDFC, JINDALSTEL ... ) matches my memory of the index then, but I did
  not check it against an NSE document.

## Spot-check against NSE Indices press releases (2026-10-30)

Four change dates (five rows) of `nifty50_changes.csv` checked against the NSE Indices press-release
PDFs. **How I checked:** first via search-result snippets (my fetch tool timed out on niftyindices.com),
then, after `thakur-bc` pointed out the site answers `curl -A "Mozilla/5.0"`, by downloading all four
PDFs and reading them with `pdftotext`. Each PDF names the excluded and included companies with their NSE
symbols and states the effective date, matching the CSV. `thakur-bc` also opened `ind_prs04072023.pdf`
independently. Only these four dates are verified against the primary source; every other row is still
Wikipedia-only.

| CSV row (effective date) | NSE release | What the release text says | Match |
| --- | --- | --- | --- |
| 2025-09-30: IndiGo, Max Healthcare in; Hero MotoCorp, IndusInd Bank out | <https://niftyindices.com/Press_Release/ind_prs22082025.pdf> (22 Aug 2025) | Hero and IndusInd excluded (earlier exclusion from Nifty 100), InterGlobe Aviation and Max Healthcare included, effective 30 Sep 2025 (close of 29 Sep) | Yes |
| 2024-03-28: Shriram Finance in; UPL out | <https://www.niftyindices.com/Press_Release/ind_prs28022024.pdf> (28 Feb 2024) | UPL excluded, Shriram Finance included, effective 28 Mar 2024 (close of 27 Mar) | Yes |
| 2023-07-13: LTIMindtree in; HDFC out | <https://www.niftyindices.com/Press_Release/ind_prs04072023.pdf> (4 Jul 2023) | HDFC excluded (HDFC/HDFC Bank amalgamation), LTIMindtree included, effective 13 Jul 2023 (close of 12 Jul) | Yes |
| 2022-09-30: Adani Enterprises in; Shree Cement out | <https://niftyindices.com/Press_Release/ind_prs01092022.pdf> (1 Sep 2022) | Shree Cement excluded, Adani Enterprises included, effective 30 Sep 2022 (close of 29 Sep) | Yes |

Result: **4 of 4 rows match** on the stocks and the effective date. Two observations:
- The releases give an inclusion list and an exclusion list; the pairing in the CSV (e.g. Hero -> IndiGo)
  comes from Wikipedia's layout and is arbitrary when a date has two swaps. Membership sets are right;
  do not read meaning into which added symbol sits beside which removed symbol.
- For these five rows `source_url` in `nifty50_changes.csv` now holds the NSE PDF (the build script's
  override table does the same, so a rebuild keeps it). The other 40 rows still cite the Wikipedia reference.

## Known gaps and caveats (not reconciled)

1. **Temporary 51st member, Oct-Nov 2025:** NSE added the demerged Tata Motors commercial-vehicle entity
   (dummy symbol `DUMMYTATAM`, later `TMCV`) to Nifty 50 from 2025-10-14 and excluded it effective
   2025-11-17 (NSE Indices press release 2025-11-13, via search summary). The Wikipedia table has no row
   for it, so it is **not** in the CSVs; during that month the real index had 51 constituents.
2. **Approximate rename dates** for Sesa Goa -> Sesa Sterlite (about 2013-08-17) -> Vedanta (about
   2015-04-01), from my own knowledge. The snapshot symbols are not sensitive to the exact day, but the
   notes file marks them approximate.
3. `MCDOWELL-N` for United Spirits (2014) is an assumption about the symbol at the time.
4. The 2026-09-30 BSE-for-Wipro swap is confirmed by several news reports of the NSE Indices
   announcement (effective after the close on 2026-09-29); I did not read the NSE release itself.
5. Changes are listed at their effective date; an announcement typically comes weeks earlier. Do not use
   announcement dates as signal dates. Also not covered: weights, free-float, intra-period corporate
   actions (splits, bonuses), delisted-stock price histories, and anything before 2012-01-01.

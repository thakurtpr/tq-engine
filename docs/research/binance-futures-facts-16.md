# Task 16: Binance USD-M delivery futures and perp fees

Author: thakur-op2, 2026-11-02. Docs only. Every fact below was read from the page named; unconfirmed items are in section 4.
I fetched web pages and public S3 directory listings (XML key names only); I downloaded no data file.

## 1. USD-M perpetual fee, regular tier

- The Binance futures FAQ gives **0.02% maker / 0.05% taker** for a regular user, but labels them "hypothetical fee rates" in worked examples and points
  to a fee table for current rates: https://www.binance.com/en/support/faq/detail/360033544231
- The live fee table page (https://www.binance.com/en/fee/futureFee) returned no table ("No records found") when fetched without a login. So 2 / 5 bps per side
  is **a cited example, not a confirmed current schedule**; verify on the account fee page before relying on it.
- **BNB discount on USD-M futures: 10%**, and only if enough BNB sits in the USD-M futures wallet; otherwise USDT is charged without discount (same FAQ). This is
  a smaller discount than the 25% on spot (`docs/fee-breakeven-15.md`). With it: 1.8 bps maker / 4.5 bps taker.
- For the edge-hunt spec use 5 bps per side taker (conservative, matches the cited page), 4.5 with BNB.

## 2. Dated USD-M delivery futures exist

- Binance calls them USDⓈ-Margined **delivery** futures (formerly "Quarterly"): linear contracts quoted, settled and collateralised in USDⓈ, expiring on the **last
  Friday of the quarter at 08:00 UTC**; the next contract is usually launched one week before the settlement date. Page published 2021-02-03, updated 2026-01-30, and it
  warns it "may be outdated": https://www.binance.com/en/support/faq/detail/3ae441db4ae740e19af3fe9228eb6619
- The 1231-expiry BTC and ETH contracts were listed on 2021-09-22 03:30 UTC (the 0924 contracts expired 2021-09-24 08:00 UTC; no new positions in the last 10 minutes before delivery):
  https://www.binance.com/en/support/announcement/8784896df8f048968c2a64a6c620c57f
- The spec page names two symbols, BTCUSDT and ETHUSDT quarterly. Whether other coins have delivery contracts: **unverified**.
- Symbol suffix is YYMMDD (for example `BTCUSDT_210326`, `BTCUSDT_261225`).
- The spec page lists no funding rate for delivery contracts. Their price converges to spot at expiry through settlement, which is what makes the basis lockable. Whether funding applies to
  delivery contracts is **not stated on the page**.
- A December 2025 announcement of 0626 contracts was reported by a search summary only (not read from the primary page).

## 3. Archive depth (data.binance.vision, public S3 listing)

Listing queried: `https://s3-ap-northeast-1.amazonaws.com/data.binance.vision?delimiter=/&prefix=data/futures/um/monthly/klines/BTCUSDT_`

- **25 dated BTCUSDT contract folders**, complete listing (not truncated): first `BTCUSDT_210326`, then `210625`, `210924`, ..., `260925`, `261225`, last `270326`. That is every
  quarter from March 2021 expiry to March 2027 expiry, i.e. continuous quarterly coverage since early 2021. Same 25 for **ETHUSDT** (`ETHUSDT_210326` ... `ETHUSDT_270326`).
- Intervals available per contract (checked for BTCUSDT_210326): 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1mo.
- The first contract's 1h files are `BTCUSDT_210326-1h-2021-02.zip` (27,521 bytes) and `...-2021-03.zip` (27,350 bytes), each with a `.CHECKSUM` file (96 bytes). So the earliest
  dated-futures data begins in **February 2021**, not 2020; the in-sample window for E must start there.
- Each contract only has a few months of data (listing to expiry), so the basis series is a stitched sequence of about 25 contracts, one roll per quarter.
- Not checked here: whether delivery contracts have `fundingRate`, `markPriceKlines` or `premiumIndexKlines` folders in the archive; whether files for the already-listed 270326 contract are complete.

## 4. Open items

1. Confirm current USD-M regular maker/taker from the account fee page or the API (`/fapi/v1/commissionRate` needs a key, so only the user or a keyed session should check).
2. Confirm whether delivery contracts exist for coins other than BTC and ETH, and their fee tier (page says only "USDⓈ-M").
3. Confirm Indian-resident access to Binance futures at all; derivative products are restricted in several jurisdictions **[unverified, not searched here]**.
4. Tax treatment is unchanged from `docs/edge-hunt-01.md`: unresolved.

## 5. Implications for the carry test

- E can be tested from 2021-02, a 4.7-year window with about 20 contract rolls, shorter than A's 2020-01 start. Keep A's split (in-sample through 2023, out-of-sample 2024 on),
  but E's in-sample portion is only 2021 to 2023.
- Round-trip cost for E: one entry and one expiry settlement; whether delivery settlement carries a fee is **unverified** (not stated on the page).

# Intraday costs and access: NSE (Zerodha), Binance spot, Kite Connect

Date: 2026-10-30. Purpose: what a scalper must earn to break even on each venue, and what the user must
decide before live NSE paper trading or any real crypto trading. **This is arithmetic on published rules,
not tax, legal or investment advice.** Verification level is stated for every source. Rates and rules
change; re-check before acting.

## Summary: minimum edge to break even (before income tax unless stated)

| Venue | All-in cost per round trip | Minimum average gross edge per trade |
| --- | --- | --- |
| NSE intraday via Zerodha, order Rs 50,000 | 10.7 bps | 10.7 bps; **13.7** with 3 bps spread/slippage; **20.7** with the adopted 5 bps/side |
| NSE intraday, order Rs 2,00,000 | 5.9 bps | 5.9; **8.9**; **15.9** |
| NSE intraday, order Rs 10,00,000 | 4.0 bps | 4.0; **7.0**; **14.0** |
| Binance spot, regular fees | 20 bps fees + about 2 bps slippage = **22 bps** | 22 bps (17 with the 25% BNB discount) |
| Binance spot **after Indian VDA tax** | same costs, but 30% tax on every winning trade with no loss relief | needs a hit rate above about 59% at *any* size of win, and a far larger gross move (section d) |

Plain reading: NSE scalping needs roughly 7-21 bps of gross edge per trade depending on size and
slippage; Binance spot scalping needs 17-22 bps before tax, and Indian tax rules make small-edge,
high-frequency crypto trading uneconomic.

## (a) NSE intraday all-in round-trip cost

Rates (Zerodha, <https://zerodha.com/charges>, fetched 2026-10-30; no effective date printed; same rates as
`costs-india.md`): brokerage 0.03% or Rs 20 per executed order, whichever is lower; STT 0.025% on the sell
side; NSE transaction charge 0.00307% of turnover; SEBI Rs 10 per crore; stamp duty 0.003% on the buy side;
GST 18% on brokerage + SEBI + transaction charge. STT rounded to the whole rupee per order (half-up);
everything else unrounded, as in `tq-costs`. Each size is a buy and a sell of the same value at an unchanged
price.

| Order value per leg | Brokerage | STT | Exchange | SEBI | Stamp | GST | **Total (Rs)** | **bps of one leg** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Rs 50,000 | 30.00 | 13 | 3.07 | 0.10 | 1.50 | 5.97 | **53.64** | **10.73** |
| Rs 2,00,000 | 40.00 | 50 | 12.28 | 0.40 | 6.00 | 9.48 | **118.16** | **5.91** |
| Rs 10,00,000 | 40.00 | 250 | 61.40 | 2.00 | 30.00 | 18.61 | **402.01** | **4.02** |

Why bps fall with size: the Rs 20-per-order brokerage cap binds above about Rs 66,700 per order. STT
(2.5 bps of the sell) is the largest percentage cost for larger orders. Caveats: **brokerage is per
executed order**, so a scalper whose orders fill in several pieces, or who places many modify/re-entry orders,
pays more; margin/leverage costs and other brokers' schedules are not included (intraday has no DP charge);
I did not research how NSE intraday gains are taxed (not asked).

## (b) Binance spot: fees, legal access, Indian tax

**Fees.** Regular spot rate 0.10% maker and 0.10% taker; paying fees in BNB gives about a 25% discount
(0.075%); VIP tiers lower the rate, but sources disagree on the VIP numbers. I could not read Binance's
official fee schedule (search results only, 2026 secondary pages), so check
<https://www.binance.com/en/fee/schedule> before relying on anything but the 0.10% base. A round trip at the
base rate costs 20 bps (15 bps with BNB). The spread on BTCUSDT/ETHUSDT is tiny; I assume about 1 bp of
slippage per side for small orders [convention], so about 2 bps per round trip.

**Is Binance legally usable by Indian residents today?** What I could verify (secondary reports only):
- The Binance app was pulled from Indian app stores in **January 2024**, along with other offshore exchanges.
- FIU-IND imposed a penalty of **Rs 18.82 crore on Binance by order dated 2024-06-19** under section 13 of the
  PMLA for non-compliance with its obligations. **Binance registered with FIU-IND as a reporting entity in
  mid-August 2024** and its website and app became available to Indian users again (reports by Inc42,
  Cointelegraph, DL News and Finance Magnates, August 2024). The FIU head said at the time that compliance
  proceedings were not complete because the penalty amount was still to be decided.
- **I found no reliable report on its 2025-2026 status** (outcome of those proceedings, current INR deposit or
  P2P availability). One 2026-dated page calls Binance "fully registered", but it is an unattributed SEO-style
  page; do not rely on it. The authoritative sources are FIU-IND's registered-entities list and Binance's India
  announcements, which I could not open.
- FIU registration is an anti-money-laundering registration, not a licence to trade or a statement that an
  activity is "legal" in a wider sense. I did not research other restrictions (RBI, banking access). **Get
  advice before depositing real money.**
- The planned **paper runner uses only Binance's public market-data websocket with no account or keys**, so
  none of this applies to it. It applies the day real orders are considered.

**Indian tax on crypto (virtual digital assets).** Per the Income Tax Department material as quoted in search
results (its own pages returned 403 to my fetch tool, so these are secondary quotations):
- **30% flat tax on income from transfer of a VDA (section 115BBH)**; the only deduction is the **cost of
  acquisition**; no deduction for fees, expenses or any other cost (clause (a) of sub-section (2)).
  Plus 4% health and education cess (from general knowledge, not in the sources I read) and surcharge for
  high incomes.
- **No set-off of a VDA loss against any other income, and no carry-forward** (clause (b)). Commentary
  disagrees on whether a loss on one VDA can offset a gain on *another*; the commonly stated reading is that it
  cannot. For a scalper this means every winning trade is taxed and every losing trade is simply lost.
- **1% TDS on the consideration (section 194S)** on transfers above Rs 50,000 a year for "specified persons"
  and above Rs 10,000 a year for others; in force since 2022-07-01. **Who deducts on an offshore exchange
  is unclear to me** (an Indian-run exchange deducts; I could not confirm what happens on Binance).
  TDS is a credit against tax, but it is withheld from every sale: for example, Rs 1,00,000 recycled through
  1% withholding on each sale leaves about Rs 90,400 usable after 10 round trips and Rs 60,500 after 50
  (illustration; it ignores credits or refunds, which arrive only after filing).
- **New numbering:** the Income-tax Act, 2025 takes effect 2026-04-01. Secondary sources map the 30% rule to
  clause 194 of the new Act and TDS to section 393(1), but the sources conflict; check the Act and the
  Income-tax Rules, 2026.

## (c) Kite Connect (live NSE ticks and minute history)

Source: Zerodha support, "What are the charges for Kite APIs"
(<https://support.zerodha.com/category/trading-and-markets/kite-api/articles/what-are-the-charges-for-kite-apis>,
fetched 2026-10-30) and the Kite developer docs (<https://kite.trade/docs/connect/v3/>, fetched 2026-10-30).

| | Personal (free) | **Connect, Rs 500 per app per month** |
| --- | --- | --- |
| Orders, GTT, alerts, margins, portfolio | yes | yes |
| **Live market data (websocket)** | **no** | **yes** |
| **Historical candles** | **no** | **yes** |

- Price: Rs 500 per month (Rs 6,000 a year); the page does not say whether GST is added.
- **Developer credits (from October 2026):** paid Connect users who generate **Rs 2,000 or more of brokerage in a
  calendar month** receive Rs 500 of credit toward the next renewal. **Paper trading generates no brokerage,
  so it earns no credit.** Startups building mass-retail products get it free (email Zerodha).
- Websocket: up to 3 connections per API key, 3,000 instruments per connection; modes `ltp` (price only),
  `quote` (no depth), `full` (5 bid and 5 ask levels); packets carry last-trade and exchange timestamps. The
  docs do not state the update frequency or say whether any data is delayed.
- Historical data: `minute`, 3, 5, 10, 15, 30, 60 minute and `day` candles, "several years" of depth, open
  interest optional, no tokens for expired contracts; the fetched page states **no per-request day limit and no
  rate limit**, so test both before building on them.
- Access: needs a Zerodha account and your own login; **never type credentials or TOTP into chat**. One
  secondary source says a whitelisted static IP is required for API *order placement* from 2026-04-01; it should
  not matter for data only (unverified).
- **Decision aid:** a month of Connect (Rs 500) is enough to record live 1-minute bars and depth and test the
  data limits; there is no reason to commit to a year before a strategy shows an edge larger than the
  break-even bar in the summary on public Binance data or recorded NSE data.

## (d) Minimum edge to break even

Notation: `c` = all-in round-trip cost in bps (fees + slippage); a trade wins `+G` or loses `-G` bps of order
value before costs, with hit rate `p`; costs are paid on every trade, winners and losers.

**Untaxed break-even:** `G = c / (2p - 1)`; no edge exists at p <= 50%.

**With Indian VDA tax (30% on each winning trade, no loss relief):**
`p * t * (G - c) = (1 - p) * (G + c)` with `t = 0.70` (or 0.688 with cess), so
`G = c (p t + 1 - p) / (p t - (1 - p))`, which only has a solution when `p > 1 / (1 + t)`: **about 58.8%
(59.2% with cess), whatever the move size.**

Break-even gross move `G` (bps) for a symmetric scalp, Binance spot at `c` = 22 bps:

| Hit rate p | Untaxed | 30% tax, no loss relief | 30% + 4% cess |
| --- | --- | --- | --- |
| 52% | 550 | impossible | impossible |
| 55% | 220 | impossible | impossible |
| 60% | 110 | 902 | 1,397 |
| 65% | 73 | 169 | 180 |
| 70% | 55 | 92 | 95 |
| 80% | 37 | 46 | 47 |
| 90% | 28 | 30 | 31 |

For NSE at Rs 2,00,000 per order with `c` = 8.9 bps (costs plus 3 bps of slippage), untaxed: p = 55% needs
G = 89 bps, p = 60% needs 45 bps, p = 70% needs 22 bps. NSE intraday income is taxed under a different regime
(not researched), so no tax overlay is given here.

**Practical bar for the pre-registered scalping strategies:** a strategy must show, net of the venue's real
costs on out-of-sample data, an average edge above the summary table's figure *and* a hit rate and move size
consistent with the tables above. An average gross edge of a few bps per trade cannot survive either venue,
and cannot survive Indian crypto tax at all.

## Sources and verification level

- Zerodha charges, <https://zerodha.com/charges> (fetched 2026-10-30): read in full.
- Kite Connect pricing and credits, Zerodha support page above (fetched 2026-10-30): read in full. Forum post
  about the October 2026 waiver, <https://tradingqna.com/t/free-kite-connect-subscription-for-active-users/198623>
  (fetched; consistent with the support page). Kite docs for historical data and websocket (fetched).
- Binance FIU-IND registration and penalty: search summaries of reports from 2024-06 and 2024-08 (Inc42,
  Cointelegraph, DL News, Finance Magnates, Business Standard); not opened in full; 2025-2026 status not found.
- Binance fees: search summaries only (2026 secondary pages); official fee page not read.
- VDA tax: search summaries of the Income Tax Department's section 115BBH text, its VDA explainer, its section
  194S tutorial and Taxmann/Taxguru commentary; official pages blocked (403), so unverified against the Act.
- New Act numbering: conflicting secondary sources; check the Income-tax Act, 2025.
- All bps figures are my arithmetic from the rates above; slippage values are [convention] and match
  `slippage-model.md` (5 bps/side default; 3 bps round trip as a liquid-name spread assumption).

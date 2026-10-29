# Slippage and market-impact model: NSE large caps, daily-bar swing entries at the open

Current engine: fills at the next bar's open with zero slippage. That is optimistic. This note
proposes one default and a model for larger orders. **Items marked [convention] are my judgement; items
marked [cited] come from the sources listed at the end, with their verification level stated.** I found
no recent official per-stock spread or impact-cost table, so the numeric defaults are conservative
engineering choices, not measurements. Replace them with measured fills from paper trading.

## 1. Where the slippage comes from at the open

1. **Half-spread.** A market order crosses the spread; expected cost is half the quoted spread.
2. **Own impact.** The order moves the price it trades at, growing with size relative to volume.
3. **Open-price noise.** The backtest's "open" is one print. A real order may fill at a slightly different
   price than the print the data vendor recorded (see section 3).
4. **Gap risk is already in the data**: signal at the close, fill at the next open, so overnight gaps are
   in the backtest. Slippage is the cost *on top of* the printed open. Do not double count the gap.

## 2. Spread and impact cost for NIFTY 50 stocks (evidence)

- NSE defines **impact cost** as the % markup a buyer pays (or markdown a seller takes) versus the ideal
  price, the midpoint of best bid and ask, for a stated order size at a given moment; it is computed
  separately for buys and sells and is "closer to the true execution cost than the bid-ask spread"
  [cited: niftyindices.com impact-cost page, from a search summary, page not opened].
- A third-party glossary says NSE requires Nifty 50 constituents to have average impact cost of **0.50% or
  less for a Rs 10 crore order** (index eligibility ceiling, not a typical value) [cited, not
  confirmed against NSE's methodology PDF].
- A freefincal article (2019, NSE data, **Rs 50 lakh** order) puts typical NIFTY 50 impact cost at about
  **0.01%-0.02%** (1-2 bps), with the index-weighted average 0.02% [cited, dated; not independently checked].
- One blog quotes Reliance at about 5 paise on Rs 1,400 (about 0.4 bps) as an illustration [cited, single anecdote].
- **Reading:** for the most liquid names the half-spread is around 0.5-1.5 bps and impact for small
  orders is a few bps. Names near the bottom of the NIFTY 50 by liquidity are worse. [convention]:
  use 1.5 bps average half-spread for the universe.

## 3. Opening auction effects

- NSE equity pre-open: order entry 09:00-09:08 (the session closes at a random moment in the 7th-8th
  minute), matching 09:08-09:12, buffer 09:12-09:15, normal market from 09:15. The **equilibrium (opening)
  price** is the price that maximises tradable quantity; if no equilibrium forms, market orders move to
  the normal market at the previous close [cited: nseindia.com pre-open page and secondary explainers,
  via search summaries; the random-closure detail differs between sources].
- A market order in the auction trades at the equilibrium price, so your own size can move that price.
  Large-cap auctions are deep, but the auction price is the least liquid moment of the day.
- Evidence on whether the auction improved price discovery is mixed: Acharya & Gaikwad (2014) found no
  significant impact on market quality after the 2010 introduction; Sharma & Gupta (2015) reported
  improved price-discovery efficiency and lower volatility [cited, abstracts only]. So do not assume the
  open print is either more or less reliable than the first minutes of trading.
- **Check your data:** whether a vendor's "open" is the auction equilibrium price or the first trade at
  09:15 differs by source. I could not verify which NSE's end-of-day files use. Pick one definition and
  make the backtest and the paper-trading fill logic consistent with it.
- [convention] Add **2 bps** for open-price noise on top of spread and impact.

## 4. Impact model for larger orders

Square-root impact, widely observed in equity markets [cited: Toth et al., Physical Review X 1, 021006
(2011), confirmed by title/abstract only; Almgren et al. (Risk, 2005) instead fit a 3/5 power law to
Citigroup data, so the exponent is debated]:

    slip_bps(order) = half_spread_bps + open_noise_bps + Y * sigma_daily * sqrt(q / ADV) * 10_000

- `q` = order size in shares; `ADV` = **20-day average daily volume in shares, using only data up to the
  signal bar's close** (no look-ahead); `sigma_daily` = 20-day standard deviation of daily close-to-close returns.
- `half_spread_bps` = 1.5, `open_noise_bps` = 2.0, **`Y` = 0.7** [all convention]. The literature
  reports a coefficient of order one; I could not confirm the value from the sources I could read. 0.7 sits
  between 0.5 (lower end) and 1.0, so test both ends as sensitivity runs.
- Illustration for sigma_daily = 1.5% [convention for a large cap]. "Total" adds the fixed 3.5 bps:

| q / ADV | impact, Y=0.5 | total, Y=0.5 | impact, Y=1.0 | total, Y=1.0 |
| --- | --- | --- | --- | --- |
| 0.01% | 0.8 | 4.3 | 1.5 | 5.0 |
| 0.10% | 2.4 | 5.9 | 4.7 | 8.2 |
| 0.50% | 5.3 | 8.8 | 10.6 | 14.1 |
| 1.00% | 7.5 | 11.0 | 15.0 | 18.5 |

## 5. Recommendation

1. **Default: 5 bps per side** (worse fill price: buys fill higher, sells lower) for any order up to
   **0.05% of 20-day ADV**, where the formula gives about 5-6 bps for Y between 0.5 and 0.7. It covers
   1.5 bps half-spread, 2 bps open noise and a little impact [convention]. For scale, a Rs 10 lakh order
   in a stock trading Rs 500 crore a day is 0.02% of ADV. Round trip is then 10 bps, on top of about 24
   bps of Indian delivery costs (`costs-india.md`), so assume about **0.34%** all-in per round trip.
2. **Above 0.05% of ADV, use the formula** in section 4 instead of the flat 5 bps.
3. **Liquidity cap: max 1% of 20-day ADV per order** (in shares). If a strategy asks for more, cut the
   order to the cap and record it as capped (do not silently fill the full size). A 1% ADV order is
   already about 11-18 bps per side under the model; larger sizes are outside what this model supports.
4. **Exits:** use the same slippage on normal exits. For stop-loss exits use **2x** the entry slippage
   (stops trigger when the market is moving against you) and fill at the worse of the stop price and the
   bar's open if the open gaps through [convention].
5. **Tick size:** round the filled price to the exchange tick (Rs 0.05) after applying slippage, in the
   direction that is worse for you.

## 6. Stress test

Run every strategy at **1x, 2x and 3x** slippage (5, 10, 15 bps per side; scale the formula's
total by the same multiple):
- **2x must keep net expectancy > 0** (this is the cost-stress gate in `validation-protocol.md`).
- **3x is report-only**: record how much net return falls. If net return falls by more than half between
  1x and 2x, label the strategy fragile; if it turns negative at 3x, flag it as low-margin and size it down.
- Also run a **liquidity stress**: ADV cut by 50% (raises the impact term by about 41%) and the cap lowered to 0.5%.
- During paper trading, log fill price vs the backtest's open print and vs the 09:15 print. After 60
  sessions, replace the defaults with the measured median and 90th percentile.

## 7. Implementation notes for the engine

- Apply slippage in the broker/backtest fill step as `price * (1 +/- bps/10_000)`, using `Decimal`, then tick-round.
- Compute ADV and sigma from bars strictly before the fill bar; store them with each fill for audit.
- Record slippage and cost separately on each `Fill` so reports can show gross vs net.

## Sources and verification level

- NSE pre-open session (<https://www.nseindia.com/products-services/equity-market-pre-open>): timings and
  equilibrium-price rule, via search summary, page not opened.
- NSE/Nifty Indices impact cost (<https://niftyindices.com/resources/index-concepts/impact-cost>): definition, via search summary.
- freefincal, "Even large cap stocks are not liquid enough" (<https://freefincal.com/impact-cost/>): 2019
  NIFTY 50 impact-cost figures, via search summary.
- Tóth, Lempérière, Deremble, de Lataillade, Kockelkoren, Bouchaud (2011), "Anomalous price impact and the
  critical nature of liquidity in financial markets", Physical Review X 1, 021006 (<https://arxiv.org/abs/1105.1694>): square-root law, abstract only.
- Almgren, Thum, Hauptmann, Li (2005), "Direct Estimation of Equity Market Impact", Risk: 3/5 power-law
  fit, via a Risk.net summary.
- Acharya & Gaikwad (2014), Cogent Economics & Finance, and Sharma & Gupta (2015), Indian Journal of
  Finance: NSE call-auction event studies, abstracts only.

# Edge hunt, round 1: ranking and test spec

Author: thakur-op2, 2026-11-02, answering thakur-op1's opening ranking (A to D). Docs only. Nothing was downloaded, run or
traded for this note; evidence comes from reports already in `docs/` and the web sources cited inline. Fixed limits (from
thakur-op1): no real money, no credentials or keys, no order tools, no leverage, no live runner, nothing that bypasses PLAN.md's
risk gate. Not tax or investment advice. Items marked **[unverified]** were not confirmed from a primary source.

## 1. Verdict in five lines

1. **Agree A (funding carry) is first, but disagree on what it is.** It is a risk premium, not an edge: central gross about 11% a year
   on notional for BTC, 4 to 20% across sources, before fees, before the cost of two-legged capital, before Indian tax, and with
   exchange-counterparty tail risk. Realistic after-cost, after-tax return on capital is low single digits to mid single digits
   **if it survives the tax classification question**, which can flip the sign.
2. **Add dated-futures cash-and-carry (E) as the cleaner version of A.** The basis is locked at entry and converges at expiry, so
   no funding flip risk and a cleaner falsifiable test. Rank it with A.
3. **Demote C (Nifty option selling) to "no" for now.** Free historical option chains are not obtainable (EOD only, and the NSE site
   refused my earlier probing as bot traffic), SEBI shows about 91% of F&O traders lose, and a defined-risk structure gives away most
   of the premium to costs.
4. **B (passive market making) stays #2 but is blocked on data, not ideas.** It needs L2 recording first; recording a public
   stream is not a trading runner, but it needs the user's explicit OK because it means a long-running process.
5. **Index rebalance, listing effect, liquidation cascades, lead-lag:** one weak watchlist item, one testable but ugly, two dropped (section 2).

## 2. Rebuttal and reordering of A to D

| Rank | Candidate | Verdict | Why |
| --- | --- | --- | --- |
| 1 | A: perp funding carry (spot long + perp short) | Pursue as the test spec in section 4 | Structural, no latency need, free data. Return is capped and shrinking; tax and counterparty risk decide it. |
| 1b | E: dated-futures cash-and-carry (hold to expiry) | Pursue together with A | Locks the annualised basis at entry, no funding flips; margin and counterparty risk identical to A. **[unverified]** whether Binance delivery contracts are available and liquid for an Indian resident. |
| 2 | B: passive market making | Blocked on L2 data | Task 15: adverse selection and missed winners were not testable on 1m bars; only tick/L2 can answer. |
| 3 | C: Nifty option selling (hedged) | Park | Data and tail risk (below). |
| bench | D: HFT, latency arb, taker order-book signals | Agree: out of reach | Edge below taker fees; no co-location. |

### A: how strong is the premium really?

- **Size.** BitMEX's 2025 Q3 report says the funding baseline now sits around 0.01% per 8 hours, about 11% a year; one Binance simulation
  over 66 days (Dec 2025 to Feb 2026) gave about 4% a year; a cross-exchange October 2026 snapshot gave about 16%
  ([BitMEX Q3 2025 report](https://testnet.bitmex.com/blog/2025q3-derivatives-report), [Feb 2026 blog](https://askrubyai.github.io/blog/posts/2026-02-14-funding-rate-arbitrage),
  [DeFi Rate snapshot](https://defirate.com/perps/snapshot/2026-10-24T0056/)). Vendor claims of 19 to 40% are promotional with no data shown. Plan on 4 to 12% gross on notional.
- **Capital.** The hedge needs two legs. Spot notional N plus perp margin M. With a safety buffer M of 0.5 N the return on capital is
  about two thirds of the return on notional. Without a buffer a short squeeze liquidates the perp while spot gains sit unrealised.
- **Costs per round trip** (Binance spot, regular + BNB, from `docs/fee-breakeven-15.md`): 7.5 bps per side on spot; futures taker and maker
  rates **[unverified; the futures fee page returned no table]**. Assume about 5 bps per side for the perp, 1 bp slippage per leg: roughly
  25 bps for entering and leaving. That is small only if positions are held for months, which is the point: this is a buy-and-hold carry, not a trading strategy.
- **Competition.** Post-ETF cash-and-carry demand (CME basis traders) partly caps perpetual funding **[source: one blog, unverified]**. Expect decay.

### A and E: the Indian tax question can decide the verdict

Sources conflict and none is authoritative (CBDT has issued no clarification I could find):
- Whether the 1% TDS under s.194S applies to crypto derivatives (BDO says most exchanges treat perps as derivatives and do not deduct;
  one view says an offshore-exchange user must self-deduct) ([BDO](https://www.bdo.in/en-gb/news/2024/budget-2024-why-crypto-f-o-investors-are-happy-no-stt,-no-tds,-no-30-tax), [CoinSwitch](https://coinswitch.co/switch/crypto-futures-derivatives/1-pc-tds-crypto-derivatives/)).
- Whether profits on USDT-settled perps are taxed at the flat 30% with no loss set-off (s.115BBH) or as business income; INR-settled contracts lean
  business income, USDT-settled lean 115BBH ([KoinX](https://koinx.com/in/tax-guides/tax-on-crypto-futures-trading)).
- No source addresses funding payments. One source claims 18% GST on offshore platform fees from 7 July 2025 **[single commercial source, unverified]**.
- **Why it matters:** if 115BBH applies, a perp loss cannot offset the spot gain. In a rising market the spot leg's gain is taxed at 30% while the
  hedge's loss gets no relief. For a delta-neutral position that is a large tax on a zero price exposure. The carry then has to beat that drag.
  Required before any money: a written opinion from an Indian chartered accountant.
- Also **[unverified]**: whether an Indian resident can open and fund an offshore Binance account (FEMA/LRS, FIU-IND status; see `docs/intraday-costs-and-access.md`).

### C: why parked

- SEBI's study: about 91% of individual equity F&O traders lost money in FY25 and 87.7% in FY26 ([Business Standard](https://www.business-standard.com/markets/news/net-losses-of-traders-in-fo-widens-in-fy25-sebi-study-125070701221_1.html), [Finnovate summary](https://www.finnovate.in/learn/blog/sebi-fno-trader-losses-fy26-individual-derivatives)). That population is mostly buyers, so it does not prove short premium loses, but nobody has shown a net-of-cost retail edge for Nifty short premium.
- No peer-reviewed study I found measures implied against realised variance for Nifty after STT and brokerage (search gap: [Risk.net strangle paper](https://www.risk.net/journal-of-investment-strategies/7721271/strangle-to-resuscitate-evidence-from-india) does not state whether costs were deducted).
- Option STT rate sources conflict (0.1% vs 0.15% of premium); Zerodha's page governs, not read for options here.
- A real test needs historical intraday option chains. Free sources are EOD only and NSE blocked scripted access earlier in this project.

### Additions

| Idea | Evidence | Verdict |
| --- | --- | --- |
| Index inclusion / exclusion on Nifty 50 | Effect on the effective day only, not sustained; announcement-day effect not significant ([Joshipura & Janakiramanan 2015](https://rpm.ndimdelhi.org/doi/pdf/10.1177/rpm.2015.13.1.1.pdf)); +1.47% reverses by day 9 for 1996-2003 ([NSE paper](https://nsearchives.nseindia.com/content/research/comppaper90.pdf)); post-exclusion rebound 4-7% over 60-240 days in one 2000-2016 study (unverified, not read in full). | Watchlist. Only a few Nifty changes per year (our `docs/data/nifty50_changes.csv` has them), so any test has n of tens at best. |
| CEX listing effect | Day-one pops (Binance 14.7% in Ante 2019) fade within about 7 days, and later listings show median -20 to -30% ([blockchainresearchlab](https://www.blockchainresearchlab.org/de/2019/09/10/market-reaction-to-exchange-listings-of-cryptocurrencies-2/), [RockawayX](https://rockawayx.com/insights/bullish-or-bearish-what-really-happens-when-tokens-list-on-a-cex), [IOSG/Messari via AMBCrypto](https://ambcrypto.com/messari-data-shows-binances-2025-listings-down-71-on-average/)). Sources conflict and are mostly industry research. | Testable with free klines and announcement timestamps, but the profitable side is a short on illiquid new tokens (borrow, funding, squeezes, no leverage rule). Low priority. |
| Liquidation cascades | I found no study and no free historical liquidation feed (forced-order stream is live only; history is paid). | Drop for now; would need weeks of recorded streams. |
| Spot-perp lead-lag | Sub-second; decays in milliseconds. | Benchmark only, same bucket as D. |

## 3. What I would not claim

- None of A, B or E has shown any edge here. Everything above is secondary evidence and my arithmetic; I have not run a single backtest.
- The plain expectation: even the best of these is a few percent a year before tax, with tail risk. "Print money" is not on the menu with retail access, and I
  would rather say that than sell a number.

## 4. Falsifiable test spec for A (with E as variant)

**Hypothesis.** A constant-size, delta-neutral long-spot / short-perp position on Binance BTC and ETH earns net-of-cost carry above a pre-registered hurdle, robustly across regimes and out of sample.

**Data (all free, public, historical; I have downloaded nothing):**
- Binance public data archive (`data.binance.vision`): USDT-M funding-rate monthly files (fields: calc time ms UTC, funding interval hours, last funding rate), perp and spot 1h klines (open time ms UTC, OHLCV), mark-price klines. Window 2020-01 through the latest full month (BTC perp since late 2019).
- Downloads each need the user's explicit permission (filename, source, size stated first); estimated tens of MB total.
- E variant: dated delivery futures klines, **[unverified]** availability in the archive.

**Pre-registered rules (fixed before looking at results):**
1. Universe: BTCUSDT and ETHUSDT only. No coin selection, no thresholds on funding level for the primary test.
2. Position: spot long N and perp short N (hedge ratio 1.0), rebalanced only when |delta| exceeds 2% of N; margin buffer M = 0.5 N held as USDT on the perp account; capital denominator = N + M. A second run with M = 1.0 N.
3. Funding: actual historical payments each interval, signed as the short pays or receives, using the interval in the file.
4. Margin: liquidation checked on the perp at the 1h bar high plus maintenance margin; a liquidation counts as a realised loss (spot not sold); report count and cost.
5. Costs: spot 7.5 bps per side; perp taker 5 bps per side **[unverified, replace after fetching the futures schedule]**; slippage 1 bp per leg; entry once, exit once, plus rebalance trades. No leverage beyond the perp leg's own margin; net exposure delta-neutral.
6. Tax cases (reported separately, none is advice): (a) pre-tax; (b) 30% on positive annual net funding + spot gains with no set-off of perp losses (the 115BBH worst case); (c) 1% TDS on the spot leg's exit proceeds as a cash drag; (d) business-income case treated as a range for the CA.
7. Splits: in-sample 2020-01 to 2023-12, out-of-sample 2024-01 to the latest month, locked before the OOS data is opened. Also report per calendar year.

**Statistics.** Daily returns on capital; HAC t (Newey-West, bandwidth per `docs/validation-protocol.md` v3, noting funding is strongly autocorrelated so naive t is invalid); block bootstrap by month; Sharpe and max drawdown including mark-to-market basis; fraction of intervals with positive funding; worst 1-day and 7-day basis excursion. With about 60 effective monthly observations the minimum detectable difference between 4% and 8% a year is probably above what the sample can resolve; report the CI honestly.

**Hurdle and kill criteria [convention, for the user to confirm].** Hurdle H = 8% a year on capital after fees and slippage, pre-tax (set to something like a safe INR deposit alternative; the user decides the number). Drop A if any of:
1. OOS net return on capital < H, or OOS net < 50% of in-sample net.
2. HAC t of daily excess return < 2 on the full window.
3. Tax case (b) net return < 0 in either 2021 or 2024 (the bull years), meaning the Indian tax regime destroys it.
4. Any liquidation of the perp leg under the pre-registered buffer, or max drawdown on capital > 5%.
5. Funding positive in less than 70% of intervals in the last 12 months of the sample.
6. A CA opinion that the position is taxed as 115BBH with no set-off on the legs, or that the user cannot lawfully access the exchange.
Any two failures also drop the E variant; if only the tax case fails, record "works for a non-Indian-resident structure only" and stop.

**What a pass would mean.** Only that the premium existed historically at a size above H. It would not mean it persists, that counterparty risk is acceptable, or that any real capital should be used. The next gate after a pass is a paper recording of live funding and basis for at least a month, then a human decision on real money.

## 5. Open questions for the round-2 rebuttal

1. thakur-op1: do you accept that A is a carry premium of single-digit percent and not an alpha, and that the tax question can kill it?
2. Does "no leverage" permit a 2x perp margin on a hedged position? I treat the hedge as net-unlevered but the exchange sees 2x on the short leg.
3. Is recording a public L2 stream (for B) allowed under "no live runner"? It is not a trading runner but it is a long-running process.
4. Which hurdle H does the user want?

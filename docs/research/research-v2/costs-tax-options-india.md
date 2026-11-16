# Costs, slippage and tax for NSE index options (India), for the options-v1 pre-registration

Researched 2026-11-06. Rates are regulator/exchange-set and change; keep them in config keyed by date, not in code. Items marked [recollection] were not
fetched this session and must be re-verified against the cited primary source in a dated addendum committed before any scored run.

## 1. Rates

Current schedule (Zerodha charges page, fetched 2026-11-06, <https://zerodha.com/charges>; the page prints no effective date):

| Charge | Options | Charged on |
| --- | --- | --- |
| Brokerage | flat Rs 20 per executed order | each order, each leg is an order |
| STT, premium | **0.15%** | **sell side only**, on premium turnover |
| STT, exercised option | **0.15% of intrinsic value** | buyer of an ITM option held to expiry (cash settlement) |
| Exchange transaction, NSE | 0.03553% | premium turnover, buy and sell |
| SEBI fee | Rs 10 per crore | turnover |
| Stamp duty | 0.003% (Rs 300 per crore) | buy side premium |
| GST | 18% | brokerage + SEBI fee + exchange charge (not STT, not stamp) |
| Futures STT (for reference) | 0.05% sell side | n/a here |

**STT history on option premium (sell side)**, needed because the development window is 2012 to 2021 and the holdout 2022 to 2026:

| Effective | Rate | Source |
| --- | --- | --- |
| 2013-06-01 (and before, back to 2008) | 0.017% | tradingqna STT history table, <https://tradingqna.com/t/how-has-stt-changed-historically-what-has-been-its-impact-on-traders-exchanges-and-brokerages/125582> [secondary] |
| 2016-06-01 | 0.05% | same [secondary] |
| 2023-04-01 | 0.0625% | Finance Act 2023; upstox note [secondary] |
| 2024-10-01 | 0.10% | Finance (No.2) Act 2024; <https://upstox.com/news/market-news/financial-regulations/stt-tax-on-futures-and-options-set-to-increase-from-october-1-here-s-how-it-will-affect-f-and-o-traders/article-118871> |
| 2026-04-01 | 0.15% (futures 0.02% to 0.05%) | Union Budget 2026-27 coverage, e.g. <https://www.businesstoday.in/markets/stocks/story/budget-2026-fm-sitharaman-raises-stt-on-fo-trades-signals-tighter-market-oversight-513932-2026-02-01>; confirmed by Zerodha's page above |

Exercise STT before 2026 was 0.125% of intrinsic value; sources differ on the prior rate (verify). **Exit before expiry in every registered strategy** so exercise STT is not paid (rule in the prereg); if a position is nonetheless exercised, charge the exercise STT on intrinsic value.
Exchange transaction charge history is not verified: apply **0.05% of premium turnover** for any date before 2024-01-01 (at or above every figure I recall, so conservative) and 0.03553% after [recollection].
Brokerage is held at Rs 20 per order for all years (full-service brokers charged more before about 2015; this understates early costs and is stated as a known bias; the sensitivity run uses 2x brokerage).

## 2. Slippage assumption (no bid/ask data exists)

Daily bhavcopy has no quotes (`docs/data-feasibility-options.md`). Assumption, a **[convention]**, frozen in the prereg:

- Every leg, every side, fill at the day's `OPEN` price (first trade) moved against us by `s = max(1.0% of that premium, Rs 1.0)` per unit. Consistent with the repo's slippage model rule of 2x on stop exits (`docs/slippage-model.md`): exits triggered by a stop or loss cut use `2s`.
- Contracts with `CONTRACTS = 0` or `OPEN = 0` on the fill day are not fillable (trade skipped, counted).
- Stress at 1x, 2x, 3x `s` as the protocol requires (2x must remain net profitable with a positive t).
Basis: NSE Nifty options tick is Rs 0.05 and liquid ATM spreads are a few ticks, but far OTM and BankNifty wings are wider; Rs 1 or 1% is a stated guess, not an observation.

## 3. Worked example (current schedule)

Nifty bull put spread, lot 65: sell 60, buy 25 (credit 35); profit-take: buy back at 25, sell the long leg at 10. Gross P&L = ((60-25) + (10-25)) x 65 = Rs 1,300.
Premium turnover 7,800; sell-side 4,550. Brokerage 4 orders x 20 = 80.00; STT 0.15% x 4,550 = 6.83; exchange 0.03553% x 7,800 = 2.77; SEBI 0.01; GST 18% x (80 + 0.01 + 2.77) = 14.90; stamp 0.003% x buy 3,250 = 0.10. **Total Rs 104.61 (8.0% of gross), net Rs 1,195.39** before slippage. Brokerage dominates; an iron condor has 8 orders, so Rs 160 + GST = Rs 188.8 of fixed cost per round trip regardless of size. Slippage of Rs 1 per leg-side on 65 units x 4 fills is Rs 260 on top.
Consequence: with Rs 20 flat brokerage, structures with small maximum profit per lot (narrow Nifty spreads) lose a large share of gross to fixed cost; this is tested, not assumed away.

## 4. Tax

- Index options are non-speculative **business income** (not capital gains) under the Income-tax Act; profits and losses are netted per financial year; F&O losses set off against other business income and carry forward 8 years (ICAI guidance and Budget-2026 coverage; <https://www.patronaccounting.com/blog/stt-hike-2026-securities-transaction-tax-fo-traders>; verify against the Income-tax Act 2025, in force from 2026-04-01). STT is an allowable business deduction (the old s.36(1)(xv) logic) when income is declared as business income, so STT is deducted before tax.
- Taxed at the individual's slab. **Model for the registered runs [convention]:** flat **31.2%** (30% top slab + 4% cess; surcharge ignored, a lower bound for high incomes) on positive annual net business income, losses netted within the financial year and carried forward 8 years, tax paid out of the account on 31 March. Sensitivity: 20% and 39%; worst case: no loss carry-forward.
- Compliance cost not modelled: tax audit applies when turnover (sum of absolute trade profits plus, for options, premium sold-bought logic) exceeds the stated thresholds (Rs 2 crore, or Rs 10 crore at 95% digital receipts, per the Patron summary); a chartered accountant fee of the order of Rs 10,000 to 30,000 a year would apply [recollection, indicative]. Flagged, not in the net numbers.
- Equity/crypto tax rates in `docs/prereg-council-v1.md` do not apply here.

## 5. What is not modelled

Margin interest and the opportunity cost of margin beyond the cash-rate credit (see prereg section on margin), physical settlement (not applicable, index options are cash settled), early-assignment risk (European style, none), freeze-quantity splits, exchange penalties.

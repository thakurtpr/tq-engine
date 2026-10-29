# Zerodha equity charges (India): reference + test fixtures

Source of all rates: <https://zerodha.com/charges> (fetched 2026-10-30). The page prints no effective
date ("as of" = fetch date). Rates are regulator/exchange-set and change; keep them in config, not code.
Not verified against an independent second source: re-check before any real-money use.

## Rates

| Charge | Delivery | Intraday | Side | Charged on |
| --- | --- | --- | --- | --- |
| Brokerage | Rs 0 | 0.03% or Rs 20 per executed order, **whichever is lower** | each order (buy and sell) | order turnover (qty x price) |
| STT | 0.1% | 0.025% | delivery: buy AND sell; intraday: **sell only** | turnover of that side |
| Exchange transaction charge, NSE | 0.00307% | 0.00307% | buy and sell | total turnover (buy + sell) |
| Exchange transaction charge, BSE | 0.00375% | 0.00375% | buy and sell | total turnover |
| SEBI turnover fee | Rs 10 per crore (0.0001%) | same | buy and sell | total turnover |
| Stamp duty | 0.015% (Rs 1,500/crore) | 0.003% (Rs 300/crore) | **buy only** | buy turnover |
| GST | 18% | 18% | n/a | brokerage + SEBI fee + exchange transaction charge (NOT on STT or stamp duty) |
| DP charge | Rs 15.34 per scrip per sell day | none | sell, delivery only | flat, irrespective of quantity |

DP charge breakdown: Rs 3.50 CDSL + Rs 9.50 Zerodha + Rs 2.34 GST = Rs 15.34. It is debited when
shares are sold from the demat holding, once per scrip, not per order.

Also listed on the same page, not modelled above: IPFT (NSE) charge, Rs 0.01 per crore + GST (the page
text is terse; treat as negligible, unconfirmed); AMC (Rs 300/yr + GST on non-BSDA demat, first year free for
residents), pledge Rs 30 + GST, off-market transfer Rs 25, delayed-payment interest 18% a year,
extra Rs 20/order for debit-balance accounts, payment gateway Rs 9 + GST (not for UPI).

## Calculation rules the fixtures use

- turnover_buy = qty x buy_price, turnover_sell = qty x sell_price, total = both.
- Compute everything in exact decimals; round **only the final per-charge display amounts and the total
  to 2 decimals**. Total = unrounded sum, then rounded. In examples 1-3 this equals the sum of the
  rounded lines; example 4 has a half-paisa case (see below).
- Rounding mode for the half-paisa case: half-up (2.525 -> 2.53) is the common convention;
  Python's `Decimal.quantize` default is half-even and gives 2.52. Zerodha's own calculator may round
  STT to whole rupees; **unverified**, so the fixtures keep STT unrounded. Decide one rule and test it.
- Intraday brokerage cap is applied **per order**: buy and sell are separate orders.

## Worked examples (buy and sell 100 shares, Rs 1000 -> Rs 1010; gross P&L Rs 1,000.00)

Buy turnover Rs 100,000; sell turnover Rs 101,000; total Rs 201,000.

### 1. NSE, delivery

| Charge | Exact | Rs |
| --- | --- | --- |
| Brokerage | 0 | 0.00 |
| STT | 0.1% x 201,000 | 201.00 |
| Exchange txn (NSE) | 0.00307% x 201,000 = 6.1707 | 6.17 |
| SEBI | 201,000 x 1e-6 = 0.201 | 0.20 |
| GST | 18% x (0 + 0.201 + 6.1707) = 1.146906 | 1.15 |
| Stamp duty | 0.015% x 100,000 | 15.00 |
| DP | flat | 15.34 |
| **Total** | 238.858606 | **238.86** |
| **Net P&L** | 1000 - 238.858606 | **761.14** |

### 2. NSE, intraday

| Charge | Exact | Rs |
| --- | --- | --- |
| Brokerage | min(30.00, 20) + min(30.30, 20) | 40.00 |
| STT | 0.025% x 101,000 (sell only) | 25.25 |
| Exchange txn (NSE) | 6.1707 | 6.17 |
| SEBI | 0.201 | 0.20 |
| GST | 18% x (40 + 0.201 + 6.1707) = 8.346906 | 8.35 |
| Stamp duty | 0.003% x 100,000 | 3.00 |
| DP | none | 0.00 |
| **Total** | 82.968606 | **82.97** |
| **Net P&L** | 1000 - 82.968606 | **917.03** |

### 3. BSE, delivery

| Charge | Exact | Rs |
| --- | --- | --- |
| Brokerage | 0 | 0.00 |
| STT | 201.00 | 201.00 |
| Exchange txn (BSE) | 0.00375% x 201,000 = 7.5375 | 7.54 |
| SEBI | 0.201 | 0.20 |
| GST | 18% x (0.201 + 7.5375) = 1.39293 | 1.39 |
| Stamp duty | 15.00 | 15.00 |
| DP | flat | 15.34 |
| **Total** | 240.47143 | **240.47** |
| **Net P&L** | 1000 - 240.47143 | **759.53** |

### 4. Bonus: NSE intraday, 10 shares (brokerage below the Rs 20 cap)

Buy 10,000; sell 10,100; total 20,100; gross P&L Rs 100.00.

| Charge | Exact | Rs |
| --- | --- | --- |
| Brokerage | 0.03% x 10,000 + 0.03% x 10,100 = 3.00 + 3.03 | 6.03 |
| STT | 0.025% x 10,100 = 2.525 (half-paisa: 2.52 half-even, 2.53 half-up) | 2.52 / 2.53 |
| Exchange txn (NSE) | 0.617070 | 0.62 |
| SEBI | 0.0201 | 0.02 |
| GST | 18% x (6.03 + 0.0201 + 0.61707) = 1.2000906 | 1.20 |
| Stamp duty | 0.003% x 10,000 | 0.30 |
| **Total** | 10.6922606 | **10.69** |
| **Net P&L** | 100 - 10.6922606 | **89.31** |

Total and Net use the unrounded sum, so they do not depend on the half-paisa STT rounding.

## Takeaway for the backtester

Round-trip cost on a Rs 1,000 stock is about 0.24% for delivery and 0.08% for intraday at 100 shares, so a
swing strategy needs an edge well above ~0.3% per round trip before slippage. DP is a flat Rs 15.34
per sell, which makes tiny delivery positions disproportionately expensive. Slippage and market
impact are NOT included here.

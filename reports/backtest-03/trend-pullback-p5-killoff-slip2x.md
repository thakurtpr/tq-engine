# Backtest: TrendPullback (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x2, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap, Rs 0.05 ticks); costs: Zerodha NSE delivery
- Risk gate: max risk 1%, notional cap 20%, max open positions 5, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | -41.40% | 275.28% |
| CAGR | -5.21% | 14.15% |
| Sharpe (daily, annualised, rf 0) | -1.00 | 0.88 |
| t, month-clustered daily returns | -2.86 (G = 120) | 2.77 |
| Max drawdown | 43.97% | 38.44% |
| Calmar (CAGR / max DD) | -0.12 | 0.37 |
| Exposure (mean invested / equity) | 22.66% | 100% |
| Open positions: mean / max | 2.45 / 5 | - |
| Final equity (Rs) | 585965.33 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: month-clustered t = -4.54 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 1541 / 71 |
| Win rate % | 56.26 |
| Mean net trade % | -0.36 |
| Mean R (net) | -0.07 |
| Profit factor (net) | 0.76 |
| t-stat of net trade returns, naive (assumes independent trades) | -3.92 |
| t-stat, clustered by entry month (review tstat.py, CR1) | -2.99 (G = 120 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 380.64 |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | 147960.45 / 281210.30 / 281989.54 / -415239.39 |
| Net P&L without the best 5 trades (Rs) | -460712.14 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 7542 |
| Ignored: already held or pending | 0 |
| Rejected: risk at fill: fill price is at or through the stop (gap); entry skipped | 1 |
| Rejected: risk: max open positions reached | 4750 |
| Skipped: not an index member on the fill date | 1248 |
| Skipped: sized to zero shares | 0 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 3084 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | -3.25% | 27.70% |
| 2013 | -8.13% | 6.76% |
| 2014 | -0.64% | 31.39% |
| 2015 | -4.14% | -4.06% |
| 2016 | -4.97% | 3.01% |
| 2017 | 1.07% | 28.65% |
| 2018 | -14.14% | 3.15% |
| 2019 | -8.50% | 12.02% |
| 2020 | -5.30% | 14.90% |
| 2021 | -3.14% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 1541 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 71 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs -460712 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | -0.36 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | -0.07 | **FAIL** |
| 4c | t-stat, month-clustered daily portfolio returns | >= 3.0 | -2.86 | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0.76 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | -1.00 | **FAIL** |
| 4f | Beats NIFTY 50 price index (Sharpe and Calmar) | both higher; necessary only | Sharpe -1.00 vs 0.88; Calmar -0.12 vs 0.37 (price index) | **FAIL** |

Not assessable from one run: cost stress (5b, needs the 2x run), walk-forward OOS (2, 4), robustness (5), DSR/PBO (6), Monte Carlo (7). A PASS here is in-sample only.

## Point-in-time universe coverage

Entries only while an index member; exits always allowed.

Member-days with price data: **117614 of 123250 (95.43%)**.

| Member symbol | Price file | Member-days missing | Member-days |
| --- | --- | ---: | ---: |
| CAIRN | CAIRN | 1044 | 1044 |
| HDFC | HDFC | 2465 | 2465 |
| IDFC | IDFC | 836 | 836 |
| JPASSOCIAT | JPASSOCIAT | 553 | 553 |
| RANBAXY | RANBAXY | 553 | 553 |
| STER | STER | 185 | 185 |

## Not run (no usable data)

- CAIRN: no data for NSE:CAIRN
- HDFC: no data for NSE:HDFC
- IDFC: no data for NSE:IDFC
- JPASSOCIAT: no data for NSE:JPASSOCIAT
- RANBAXY: no data for NSE:RANBAXY
- STER: no data for NSE:STER

# Backtest: Donchian (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x1, flat; costs: Zerodha NSE delivery
- Risk gate: max risk 1%, notional cap 20%, max open positions 5, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | 14.13% | 275.28% |
| CAGR | 1.33% | 14.15% |
| Sharpe (daily, annualised, rf 0) | 0.21 | 0.88 |
| t, month-clustered daily returns | 0.63 (G = 120) | 2.77 |
| Max drawdown | 21.44% | 38.44% |
| Calmar (CAGR / max DD) | 0.06 | 0.37 |
| Exposure (mean invested / equity) | 43.24% | 100% |
| Open positions: mean / max | 4.15 / 5 | - |
| Final equity (Rs) | 1141349.06 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: month-clustered t = -2.86 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 288 / 58 |
| Win rate % | 30.90 |
| Mean net trade % | 0.46 |
| Mean R (net) | 0.06 |
| Profit factor (net) | 1.06 |
| t-stat of net trade returns, naive (assumes independent trades) | 0.64 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 0.54 (G = 103 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 63.44 |
| Largest symbol share of net P&L | NSE:HCLTECH 102.08% |
| Gross P&L before slippage / slippage / costs / net (Rs) | 157542.16 / 29623.59 / 70325.08 / 57593.50 |
| Net P&L without the best 5 trades (Rs) | -193560.75 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 6707 |
| Ignored: already held or pending | 0 |
| Rejected: risk: max open positions reached | 6102 |
| Skipped: not an index member on the fill date | 313 |
| Skipped: sized to zero shares | 0 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 580 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | -0.70% | 27.70% |
| 2013 | 9.02% | 6.76% |
| 2014 | 3.38% | 31.39% |
| 2015 | -7.65% | -4.06% |
| 2016 | -4.59% | 3.01% |
| 2017 | 8.88% | 28.65% |
| 2018 | -2.46% | 3.15% |
| 2019 | -4.57% | 12.02% |
| 2020 | 3.25% | 14.90% |
| 2021 | 10.61% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 288 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 58 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | NSE:HCLTECH 102.08% | **FAIL** |
| 3e | Net P&L without best 5 trades | > 0 | Rs -193561 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | 0.46 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | 0.06 | **FAIL** |
| 4c | t-stat, month-clustered daily portfolio returns | >= 3.0 | 0.63 | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 1.06 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | 0.21 | **FAIL** |
| 4f | Beats NIFTY 50 price index (Sharpe and Calmar) | both higher; necessary only | Sharpe 0.21 vs 0.88; Calmar 0.06 vs 0.37 (price index) | **FAIL** |

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

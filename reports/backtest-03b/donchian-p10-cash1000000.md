# Backtest: Donchian (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x1, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03b/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | 0.86% | 275.28% |
| CAGR | 0.09% | 14.15% |
| Sharpe (daily, annualised, rf 0) | 0.07 | 0.88 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 218) | 0.18 | - |
| t, monthly returns (4c cross-check) | 0.21 | - |
| t, per-trade, entry-month clustered (4c cross-check) | 0.16 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **0.18** | - |
| t, month-clustered daily returns (backtest-03 figure) | 0.20 (G = 120) | 2.77 |
| Max drawdown | 45.46% | 38.44% |
| Calmar (CAGR / max DD) | 0.00 | 0.37 |
| Exposure (mean invested / equity) | 71.23% | 100% |
| Open positions: mean / max | 7.18 / 10 | - |
| Final equity (Rs) | 1008581.69 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: HAC t = -5.28 (L = 218); month-clustered t = -3.13 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 524 / 68 |
| Win rate % | 29.20 |
| Mean net trade % | 0.11 |
| Mean R (net) | 0.00 |
| Profit factor (net) | 0.97 |
| t-stat of net trade returns, naive (assumes independent trades) | 0.19 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 0.16 (G = 115 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 150.89 |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | 121764.58 / 64088.11 / 119636.60 / -61960.13 |
| Net P&L without the best 5 trades (Rs) | -384693.40 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 5957 |
| Ignored: already held or pending | 0 |
| Rejected: insufficient cash | 363 |
| Rejected: risk: daily loss limit reached | 29 |
| Rejected: risk: max open positions reached | 4085 |
| Skipped: not an index member on the fill date | 952 |
| Skipped: sized to zero shares | 0 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 1052 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Turnover, one-way, per year (% of mean equity) | 499.28% |
| Costs paid over the run (% of starting capital) | 12.01% |
| Slippage paid over the run (% of starting capital) | 6.43% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 14 / 1052 |
| Entries cut to the 1% ADV cap | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | 0.99% | 27.70% |
| 2013 | 9.29% | 6.76% |
| 2014 | 9.13% | 31.39% |
| 2015 | -12.45% | -4.06% |
| 2016 | -8.15% | 3.01% |
| 2017 | 6.65% | 28.65% |
| 2018 | -9.78% | 3.15% |
| 2019 | -14.95% | 12.02% |
| 2020 | 16.35% | 14.90% |
| 2021 | 9.37% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 524 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 68 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs -384693 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | 0.11 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | 0.00 | **FAIL** |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | 0.18 (HAC 0.18, L = 218; monthly 0.21; per-trade 0.16) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0.97 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | 0.07 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher; necessary only (price index) | Sharpe 0.07 vs 0.88; Calmar 0.00 vs 0.37 (NSEI) | **FAIL** |

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

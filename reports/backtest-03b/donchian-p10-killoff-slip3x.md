# Backtest: Donchian (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x3, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03b/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | -14.95% | 275.28% |
| CAGR | -1.61% | 14.15% |
| Sharpe (daily, annualised, rf 0) | -0.08 | 0.88 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 218) | -0.23 | - |
| t, monthly returns (4c cross-check) | -0.23 | - |
| t, per-trade, entry-month clustered (4c cross-check) | -0.36 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **-0.23** | - |
| t, month-clustered daily returns (backtest-03 figure) | -0.24 (G = 120) | 2.77 |
| Max drawdown | 50.24% | 38.44% |
| Calmar (CAGR / max DD) | -0.03 | 0.37 |
| Exposure (mean invested / equity) | 70.80% | 100% |
| Open positions: mean / max | 7.23 / 10 | - |
| Final equity (Rs) | 850472.68 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: HAC t = -5.88 (L = 218); month-clustered t = -3.49 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 535 / 69 |
| Win rate % | 28.04 |
| Mean net trade % | -0.23 |
| Mean R (net) | -0.06 |
| Profit factor (net) | 0.88 |
| t-stat of net trade returns, naive (assumes independent trades) | -0.44 |
| t-stat, clustered by entry month (review tstat.py, CR1) | -0.36 (G = 116 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 359.97 |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | 80220.40 / 177570.56 / 111197.27 / -208547.42 |
| Net P&L without the best 5 trades (Rs) | -503092.31 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 5950 |
| Ignored: already held or pending | 0 |
| Rejected: insufficient cash | 319 |
| Rejected: risk: daily loss limit reached | 28 |
| Rejected: risk: max open positions reached | 4151 |
| Skipped: not an index member on the fill date | 913 |
| Skipped: sized to zero shares | 0 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 1074 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Turnover, one-way, per year (% of mean equity) | 501.96% |
| Costs paid over the run (% of starting capital) | 11.16% |
| Slippage paid over the run (% of starting capital) | 17.80% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 12 / 1074 |
| Entries cut to the 1% ADV cap | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | 1.08% | 27.70% |
| 2013 | 5.93% | 6.76% |
| 2014 | 6.10% | 31.39% |
| 2015 | -14.01% | -4.06% |
| 2016 | -9.48% | 3.01% |
| 2017 | 3.89% | 28.65% |
| 2018 | -11.12% | 3.15% |
| 2019 | -15.63% | 12.02% |
| 2020 | 14.90% | 14.90% |
| 2021 | 7.46% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 535 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 69 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs -503092 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | -0.23 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | -0.06 | **FAIL** |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | -0.23 (HAC -0.23, L = 218; monthly -0.23; per-trade -0.36) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0.88 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | -0.08 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher; necessary only (price index) | Sharpe -0.08 vs 0.88; Calmar -0.03 vs 0.37 (NSEI) | **FAIL** |

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

# Backtest: TrendPullback (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x3, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap, Rs 0.05 ticks); costs: Zerodha NSE delivery
- Risk gate: max risk 1%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | -64.74% | 275.28% |
| CAGR | -9.90% | 14.15% |
| Sharpe (daily, annualised, rf 0) | -1.33 | 0.88 |
| t, month-clustered daily returns | -3.97 (G = 120) | 2.77 |
| Max drawdown | 65.69% | 38.44% |
| Calmar (CAGR / max DD) | -0.15 | 0.37 |
| Exposure (mean invested / equity) | 33.70% | 100% |
| Open positions: mean / max | 3.69 / 10 | - |
| Final equity (Rs) | 352613.17 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: month-clustered t = -5.98 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 2361 / 72 |
| Win rate % | 54.76 |
| Mean net trade % | -0.43 |
| Mean R (net) | -0.09 |
| Profit factor (net) | 0.70 |
| t-stat of net trade returns, naive (assumes independent trades) | -5.81 |
| t-stat, clustered by entry month (review tstat.py, CR1) | -4.15 (G = 120 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 407.92 |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | 211261.45 / 507566.80 / 354208.77 / -650514.12 |
| Net P&L without the best 5 trades (Rs) | -687894.33 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 6866 |
| Ignored: already held or pending | 0 |
| Rejected: insufficient cash | 22 |
| Rejected: risk at fill: fill price is at or through the stop (gap); entry skipped | 4 |
| Rejected: risk: daily loss limit reached | 92 |
| Rejected: risk: max open positions reached | 2292 |
| Skipped: not an index member on the fill date | 2087 |
| Skipped: sized to zero shares | 3 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 4727 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | -9.18% | 27.70% |
| 2013 | -15.09% | 6.76% |
| 2014 | -1.67% | 31.39% |
| 2015 | -7.44% | -4.06% |
| 2016 | -8.88% | 3.01% |
| 2017 | -1.87% | 28.65% |
| 2018 | -19.16% | 3.15% |
| 2019 | -11.16% | 12.02% |
| 2020 | -11.31% | 14.90% |
| 2021 | -11.80% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 2361 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 72 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs -687894 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | -0.43 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | -0.09 | **FAIL** |
| 4c | t-stat, month-clustered daily portfolio returns | >= 3.0 | -3.97 | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0.70 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | -1.33 | **FAIL** |
| 4f | Beats NIFTY 50 price index (Sharpe and Calmar) | both higher; necessary only | Sharpe -1.33 vs 0.88; Calmar -0.15 vs 0.37 (price index) | **FAIL** |

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

# Backtest: TrendPullback (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x2, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03b/manifests/manifest-405ea8789715.csv` (78 files), SHA-256 `405ea878971504648d3c01eafdee8269d9140ac71774cdc22d00e1385dd5f7bf`

## Portfolio result (one shared account)

| Metric | Strategy | NSEI buy-and-hold |
| --- | ---: | ---: |
| Total return | -52.33% | 275.28% |
| CAGR | -7.14% | 14.15% |
| Sharpe (daily, annualised, rf 0) | -0.93 | 0.88 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 10) | -2.91 | - |
| t, monthly returns (4c cross-check) | -2.84 | - |
| t, per-trade, entry-month clustered (4c cross-check) | -2.84 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **-2.91** | - |
| t, month-clustered daily returns (backtest-03 figure) | -2.82 (G = 120) | 2.77 |
| Max drawdown | 53.71% | 38.44% |
| Calmar (CAGR / max DD) | -0.13 | 0.37 |
| Exposure (mean invested / equity) | 34.08% | 100% |
| Open positions: mean / max | 3.69 / 10 | - |
| Final equity (Rs) | 476650.94 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: HAC t = -5.05 (L = 10); month-clustered t = -5.23 (G = 120).

The benchmark is the NIFTY 50 **price** index (^NSEI): it excludes dividends, so it understates the total-return index by roughly 1.2-1.5% a year. Beating it is necessary, not sufficient.

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 2362 / 72 |
| Win rate % | 56.99 |
| Mean net trade % | -0.29 |
| Mean R (net) | -0.06 |
| Profit factor (net) | 0.78 |
| t-stat of net trade returns, naive (assumes independent trades) | -3.94 |
| t-stat, clustered by entry month (review tstat.py, CR1) | -2.84 (G = 120 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 315.13 |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | 245327.08 / 367340.67 / 405757.52 / -527771.11 |
| Net P&L without the best 5 trades (Rs) | -570566.78 |

## Entry funnel

| Step | Count |
| --- | ---: |
| Entry signals | 6866 |
| Ignored: already held or pending | 0 |
| Rejected: insufficient cash | 25 |
| Rejected: risk at fill: fill price is at or through the stop (gap); entry skipped | 4 |
| Rejected: risk: daily loss limit reached | 92 |
| Rejected: risk: max open positions reached | 2291 |
| Skipped: not an index member on the fill date | 2087 |
| Skipped: sized to zero shares | 0 |
| Cut to the 1% ADV cap (still filled unless zero) | 0 |
| Filled orders (entries + exits) | 4729 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Turnover, one-way, per year (% of mean equity) | 2269.77% |
| Costs paid over the run (% of starting capital) | 40.60% |
| Slippage paid over the run (% of starting capital) | 36.76% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 59 / 4729 |
| Entries cut to the 1% ADV cap | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | -6.46% | 27.70% |
| 2013 | -12.12% | 6.76% |
| 2014 | 2.00% | 31.39% |
| 2015 | -4.94% | -4.06% |
| 2016 | -5.95% | 3.01% |
| 2017 | 1.45% | 28.65% |
| 2018 | -17.02% | 3.15% |
| 2019 | -8.49% | 12.02% |
| 2020 | -9.55% | 14.90% |
| 2021 | -8.75% | 24.12% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 2362 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 72 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs -570567 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | -0.29 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | -0.06 | **FAIL** |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | -2.91 (HAC -2.91, L = 10; monthly -2.84; per-trade -2.84) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0.78 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | -0.93 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher; necessary only (price index) | Sharpe -0.93 vs 0.88; Calmar -0.13 vs 0.37 (NSEI) | **FAIL** |

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

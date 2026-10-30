# Backtest: MomentumRotation (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; sized by month-end target weights (no stops; --risk-pct unused), 0.5% cash buffer, trims under 0.5% of equity skipped
- Slippage: 5 bps base per side x1, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 10%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-04/manifests/manifest-924622021a25.csv` (78 files), SHA-256 `924622021a2591f2e56827f5ac6189ecac6ab6eeef61333a677b9fd12270289b`

## Portfolio result (one shared account)

| Metric | Strategy | NIFTYBEES buy-and-hold |
| --- | ---: | ---: |
| Total return | 38.20% | 299.47% |
| CAGR | 3.29% | 14.86% |
| Sharpe (daily, annualised, rf 0) | 0.56 | 0.98 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 490) | 1.37 | - |
| t, monthly returns (4c cross-check) | 1.87 | - |
| t, per-trade, entry-month clustered (4c cross-check) | 2.49 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **1.37** | - |
| t, month-clustered daily returns (backtest-03 figure) | 1.90 (G = 120) | 2.87 |
| Max drawdown | 15.36% | 36.34% |
| Calmar (CAGR / max DD) | 0.21 | 0.41 |
| Exposure (mean invested / equity) | 19.18% | 100% |
| Open positions: mean / max | 1.93 / 10 | - |
| Final equity (Rs) | 1381955.91 | - |
| Kill switch engaged | 2013-08-07 | - |

Strategy minus benchmark daily return: HAC t = -2.93 (L = 490); month-clustered t = -2.23 (G = 120).

The benchmark is frictionless buy-and-hold of NIFTYBEES on its dividend-adjusted closes (for NIFTYBEES: close to the NIFTY 50 TRI less the fund's expense ratio and tracking error).

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 96 / 30 |
| Win rate % | 73.96 |
| Mean net trade % | 17.83 |
| Mean R (net) | n/a |
| Profit factor (net) | 2.42 |
| t-stat of net trade returns, naive (assumes independent trades) | 4.88 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 2.49 (G = 19 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 6.36 |
| Largest symbol share of net P&L | NSE:HCLTECH 41.24% |
| Gross P&L before slippage / slippage / costs / net (Rs) | 407909.98 / 7668.73 / 18285.38 / 381955.87 |
| Net P&L without the best 5 trades (Rs) | 109691.97 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Rebalance dates traded | 120 |
| Turnover, one-way, per year (% of mean equity) | 56.26% |
| Costs paid over the run (% of starting capital) | 1.83% |
| Slippage paid over the run (% of starting capital) | 0.77% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 8 / 177 |
| Entries cut to the 1% ADV cap | 0 |
| Buys without an ADV estimate (zero or missing volume: base slippage, no cap) | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | 24.14% | 27.40% |
| 2013 | 6.71% | 6.45% |
| 2014 | 4.32% | 31.58% |
| 2015 | 0.00% | -4.27% |
| 2016 | 0.00% | 3.97% |
| 2017 | 0.00% | 29.90% |
| 2018 | 0.00% | 4.82% |
| 2019 | 0.00% | 13.61% |
| 2020 | 0.00% | 15.42% |
| 2021 | 0.00% | 25.97% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 96 | **FAIL** |
| 3c | Distinct symbols traded | >= 10 | 30 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | NSE:HCLTECH 41.24% | **FAIL** |
| 3e | Net P&L without best 5 trades | > 0 | Rs 109692 | PASS (in-sample) |
| 4a | Mean net trade % | >= 0.5% | 17.83 | PASS (in-sample) |
| 4b | Expectancy (mean R) | >= +0.20R | n/a | NOT-ASSESSABLE |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | 1.37 (HAC 1.37, L = 490; monthly 1.87; per-trade 2.49) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 2.42 | PASS (in-sample) |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | 0.56 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher | Sharpe 0.56 vs 0.98; Calmar 0.21 vs 0.41 (NIFTYBEES) | **FAIL** |

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

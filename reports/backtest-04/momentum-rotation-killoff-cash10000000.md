# Backtest: MomentumRotation (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 10000000; sized by month-end target weights (no stops; --risk-pct unused), 0.5% cash buffer, trims under 0.5% of equity skipped
- Slippage: 5 bps base per side x1, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 20%, max open positions 10, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-04/manifests/manifest-924622021a25.csv` (78 files), SHA-256 `924622021a2591f2e56827f5ac6189ecac6ab6eeef61333a677b9fd12270289b`

## Portfolio result (one shared account)

| Metric | Strategy | NIFTYBEES buy-and-hold |
| --- | ---: | ---: |
| Total return | 268.39% | 299.47% |
| CAGR | 13.93% | 14.86% |
| Sharpe (daily, annualised, rf 0) | 0.81 | 0.98 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 616) | 4.10 | - |
| t, monthly returns (4c cross-check) | 2.79 | - |
| t, per-trade, entry-month clustered (4c cross-check) | 5.43 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **2.79** | - |
| t, month-clustered daily returns (backtest-03 figure) | 2.87 (G = 120) | 2.87 |
| Max drawdown | 36.14% | 36.34% |
| Calmar (CAGR / max DD) | 0.39 | 0.41 |
| Exposure (mean invested / equity) | 98.30% | 100% |
| Open positions: mean / max | 9.88 / 10 | - |
| Final equity (Rs) | 36838976.87 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: HAC t = -0.21 (L = 616); month-clustered t = -0.10 (G = 120).

The benchmark is frictionless buy-and-hold of NIFTYBEES on its dividend-adjusted closes (for NIFTYBEES: close to the NIFTY 50 TRI less the fund's expense ratio and tracking error).

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 476 / 63 |
| Win rate % | 69.96 |
| Mean net trade % | 13.07 |
| Mean R (net) | n/a |
| Profit factor (net) | 1.89 |
| t-stat of net trade returns, naive (assumes independent trades) | 10.86 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 5.43 (G = 116 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 9.48 |
| Largest symbol share of net P&L | NSE:HCLTECH 12.02% |
| Gross P&L before slippage / slippage / costs / net (Rs) | 22827476.99 / 858478.83 / 1306477.18 / 20662520.99 |
| Net P&L without the best 5 trades (Rs) | 15369939.33 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Rebalance dates traded | 120 |
| Turnover, one-way, per year (% of mean equity) | 295.60% |
| Costs paid over the run (% of starting capital) | 13.42% |
| Slippage paid over the run (% of starting capital) | 8.79% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 484 / 956 |
| Entries cut to the 1% ADV cap | 0 |
| Buys without an ADV estimate (zero or missing volume: base slippage, no cap) | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | 23.97% | 27.40% |
| 2013 | 12.92% | 6.45% |
| 2014 | 27.90% | 31.58% |
| 2015 | -4.68% | -4.27% |
| 2016 | 3.23% | 3.97% |
| 2017 | 43.53% | 29.90% |
| 2018 | -3.31% | 4.82% |
| 2019 | 6.54% | 13.61% |
| 2020 | 2.11% | 15.42% |
| 2021 | 38.49% | 25.97% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 476 | PASS (in-sample) |
| 3c | Distinct symbols traded | >= 10 | 63 | PASS (in-sample) |
| 3d | Largest symbol share of net P&L | <= 15% | NSE:HCLTECH 12.02% | PASS (in-sample) |
| 3e | Net P&L without best 5 trades | > 0 | Rs 15369939 | PASS (in-sample) |
| 4a | Mean net trade % | >= 0.5% | 13.07 | PASS (in-sample) |
| 4b | Expectancy (mean R) | >= +0.20R | n/a | NOT-ASSESSABLE |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | 2.79 (HAC 4.10, L = 616; monthly 2.79; per-trade 5.43) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 1.89 | PASS (in-sample) |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | 0.81 | PASS (in-sample) |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher | Sharpe 0.81 vs 0.98; Calmar 0.39 vs 0.41 (NIFTYBEES) | **FAIL** |

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

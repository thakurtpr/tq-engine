# Backtest: IndexTrendFilter (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; sized by month-end target weights (no stops; --risk-pct unused), 0.5% cash buffer, trims under 0.5% of equity skipped
- Slippage: 5 bps base per side x1, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 100%, max open positions 5, daily loss 2%, drawdown kill switch 10%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-04/manifests/manifest-4010b78e052f.csv` (1 files), SHA-256 `4010b78e052f590f425e33f48a0647f78acc8a6ef610c73c4b938bd5c7664d63`

## Portfolio result (one shared account)

| Metric | Strategy | NIFTYBEES buy-and-hold |
| --- | ---: | ---: |
| Total return | -9.00% | 299.47% |
| CAGR | -0.94% | 14.86% |
| Sharpe (daily, annualised, rf 0) | -0.41 | 0.98 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 62) | -1.18 | - |
| t, monthly returns (4c cross-check) | -1.64 | - |
| t, per-trade, entry-month clustered (4c cross-check) | n/a | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **-1.18** | - |
| t, month-clustered daily returns (backtest-03 figure) | -1.63 (G = 120) | 2.87 |
| Max drawdown | 10.51% | 36.34% |
| Calmar (CAGR / max DD) | -0.09 | 0.41 |
| Exposure (mean invested / equity) | 2.22% | 100% |
| Open positions: mean / max | 0.03 / 1 | - |
| Final equity (Rs) | 909971.19 | - |
| Kill switch engaged | 2012-05-14 | - |

Strategy minus benchmark daily return: HAC t = -3.06 (L = 62); month-clustered t = -3.07 (G = 120).

The benchmark is frictionless buy-and-hold of NIFTYBEES on its dividend-adjusted closes (for NIFTYBEES: close to the NIFTY 50 TRI less the fund's expense ratio and tracking error).

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 1 / 1 |
| Win rate % | 0 |
| Mean net trade % | -9.05 |
| Mean R (net) | n/a |
| Profit factor (net) | 0 |
| t-stat of net trade returns, naive (assumes independent trades) | n/a |
| t-stat, clustered by entry month (review tstat.py, CR1) | n/a |
| Friction (costs + slippage) as % of pre-friction gross P&L | n/a |
| Largest symbol share of net P&L | n/a |
| Gross P&L before slippage / slippage / costs / net (Rs) | -85489.41 / 2401.75 / 2137.65 / -90028.81 |
| Net P&L without the best 5 trades (Rs) | 0 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Rebalance dates traded | 120 |
| Turnover, one-way, per year (% of mean equity) | 10.42% |
| Costs paid over the run (% of starting capital) | 0.21% |
| Slippage paid over the run (% of starting capital) | 0.24% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 3 / 3 |
| Entries cut to the 1% ADV cap | 57 |
| Buys without an ADV estimate (zero or missing volume: base slippage, no cap) | 7 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | -9.00% | 27.40% |
| 2013 | 0.00% | 6.45% |
| 2014 | 0.00% | 31.58% |
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
| 3a | Closed trades | >= 200 | 1 | **FAIL** |
| 3c | Distinct symbols traded | >= 10 | 1 | **FAIL** |
| 3d | Largest symbol share of net P&L | <= 15% | n/a (net P&L <= 0) | NOT-ASSESSABLE |
| 3e | Net P&L without best 5 trades | > 0 | Rs 0 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | -9.05 | **FAIL** |
| 4b | Expectancy (mean R) | >= +0.20R | n/a | NOT-ASSESSABLE |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | -1.18 (HAC -1.18, L = 62; monthly -1.64; per-trade n/a) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 0 | **FAIL** |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | -0.41 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher | Sharpe -0.41 vs 0.98; Calmar -0.09 vs 0.41 (NIFTYBEES) | **FAIL** |

Not assessable from one run: cost stress (5b, needs the 2x run), walk-forward OOS (2, 4), robustness (5), DSR/PBO (6), Monte Carlo (7). A PASS here is in-sample only.


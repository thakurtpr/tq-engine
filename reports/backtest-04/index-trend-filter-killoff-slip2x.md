# Backtest: IndexTrendFilter (portfolio: ONE shared Rs account across all symbols)

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; sized by month-end target weights (no stops; --risk-pct unused), 0.5% cash buffer, trims under 0.5% of equity skipped
- Slippage: 5 bps base per side x2, adopted model (docs/slippage-model.md: sqrt impact > 0.05% ADV, 2x stop exits, 1% ADV entry cap); no tick rounding on adjusted prices; costs: Zerodha NSE delivery
- Risk gate: max risk 0.5%, notional cap 100%, max open positions 5, daily loss 2%, drawdown kill switch 100%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-04/manifests/manifest-4010b78e052f.csv` (1 files), SHA-256 `4010b78e052f590f425e33f48a0647f78acc8a6ef610c73c4b938bd5c7664d63`

## Portfolio result (one shared account)

| Metric | Strategy | NIFTYBEES buy-and-hold |
| --- | ---: | ---: |
| Total return | 95.22% | 299.47% |
| CAGR | 6.92% | 14.86% |
| Sharpe (daily, annualised, rf 0) | 0.69 | 0.98 |
| t, HAC Newey-West daily returns (gate 4c primary, L = 363) | 1.68 | - |
| t, monthly returns (4c cross-check) | 2.10 | - |
| t, per-trade, entry-month clustered (4c cross-check) | 0.57 | - |
| **Gate 4c t** (HAC, or lowest if a cross-check is > 0.5 below) | **0.57** | - |
| t, month-clustered daily returns (backtest-03 figure) | 2.12 (G = 120) | 2.87 |
| Max drawdown | 21.72% | 36.34% |
| Calmar (CAGR / max DD) | 0.32 | 0.41 |
| Exposure (mean invested / equity) | 69.57% | 100% |
| Open positions: mean / max | 0.76 / 1 | - |
| Final equity (Rs) | 1952183.48 | - |
| Kill switch engaged | never | - |

Strategy minus benchmark daily return: HAC t = -3.18 (L = 363); month-clustered t = -1.88 (G = 120).

The benchmark is frictionless buy-and-hold of NIFTYBEES on its dividend-adjusted closes (for NIFTYBEES: close to the NIFTY 50 TRI less the fund's expense ratio and tracking error).

## Closed trades (one account, so rupee sums are real)

| Metric | Value |
| --- | --- |
| Closed trades / symbols traded | 10 / 1 |
| Win rate % | 30.00 |
| Mean net trade % | 2.20 |
| Mean R (net) | n/a |
| Profit factor (net) | 1.73 |
| t-stat of net trade returns, naive (assumes independent trades) | 0.57 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 0.57 (G = 10 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 24.28 |
| Largest symbol share of net P&L | NSE:NIFTYBEES 100% |
| Gross P&L before slippage / slippage / costs / net (Rs) | 306495.30 / 49793.42 / 24622.44 / 232079.44 |
| Net P&L without the best 5 trades (Rs) | -278505.46 |

## Turnover, friction and the liquidity model

| Metric | Value |
| --- | --- |
| Rebalance dates traded | 120 |
| Turnover, one-way, per year (% of mean equity) | 91.03% |
| Costs paid over the run (% of starting capital) | 2.61% |
| Slippage paid over the run (% of starting capital) | 5.17% |
| Fills priced by the impact term (> 0.05% of 20-day ADV) / all fills | 38 / 38 |
| Entries cut to the 1% ADV cap | 19 |
| Buys without an ADV estimate (zero or missing volume: base slippage, no cap) | 0 |

## Calendar-year returns

| Year | Strategy | Benchmark |
| --- | ---: | ---: |
| 2012 | 1.91% | 27.40% |
| 2013 | -2.82% | 6.45% |
| 2014 | 31.93% | 31.58% |
| 2015 | -4.42% | -4.27% |
| 2016 | -2.75% | 3.97% |
| 2017 | 18.31% | 29.90% |
| 2018 | -6.38% | 4.82% |
| 2019 | -2.79% | 13.61% |
| 2020 | 18.57% | 15.42% |
| 2021 | 25.89% | 25.97% |

## Gates (docs/validation-protocol.md), in-sample, this run only

| # | Gate | Threshold | Value | Verdict |
| --- | --- | --- | --- | --- |
| 3a | Closed trades | >= 200 | 10 | **FAIL** |
| 3c | Distinct symbols traded | >= 10 | 1 | **FAIL** |
| 3d | Largest symbol share of net P&L | <= 15% | NSE:NIFTYBEES 100% | **FAIL** |
| 3e | Net P&L without best 5 trades | > 0 | Rs -278505 | **FAIL** |
| 4a | Mean net trade % | >= 0.5% | 2.20 | PASS (in-sample) |
| 4b | Expectancy (mean R) | >= +0.20R | n/a | NOT-ASSESSABLE |
| 4c | t, HAC daily portfolio returns (lowest-of-three rule) | >= 3.0 | 0.57 (HAC 1.68, L = 363; monthly 2.10; per-trade 0.57) | **FAIL** |
| 4d | Profit factor (net) | >= 1.3 | 1.73 | PASS (in-sample) |
| 4e | Sharpe, daily portfolio returns | >= 0.8 | 0.69 | **FAIL** |
| 4f | Beats buy-and-hold of the benchmark (Sharpe and Calmar) | both higher | Sharpe 0.69 vs 0.98; Calmar 0.32 vs 0.41 (NIFTYBEES) | **FAIL** |

Not assessable from one run: cost stress (5b, needs the 2x run), walk-forward OOS (2, 4), robustness (5), DSR/PBO (6), Monte Carlo (7). A PASS here is in-sample only.


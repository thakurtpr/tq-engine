# Backtest: Donchian (per-symbol: DIAGNOSTIC, NOT A PERFORMANCE CLAIM (each symbol has its own fresh account))

- Window: 2012-01-01 to 2021-12-31 (earlier bars warm up indicators only)
- Starting cash: Rs 1000000; risk per trade 0.5% of equity
- Slippage: 5 bps base per side x1, flat; costs: Zerodha NSE delivery
- Risk gate: max risk 1%, notional cap 20%, max open positions 5, daily loss 2%, drawdown kill switch 5%
- Entry priority when slots or cash run out: Alphabetical
- Strategy parameters: defaults (not tuned)
- Data manifest: `reports/backtest-03/manifests/manifest-69e6b5be26ec.csv` (77 files), SHA-256 `69e6b5be26eca26e9b225d7eb4d82040d11aa31f4f95e4665252f13cafc32a76`

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

## Pooled closed trades (diagnostic)

Independent accounts summed or averaged here are not capital anyone has; rupee figures are shown only as a share of the 75 accounts' combined starting capital.

| Metric | Value |
| --- | --- |
| Symbols run / with trades / kill switch hit | 75 / 75 / 19 |
| Closed trades / symbols traded | 940 / 75 |
| Win rate % | 30.96 |
| Mean net trade % | 1.05 |
| Mean R (net) | 0.21 |
| Profit factor (net) | 1.32 |
| t-stat of net trade returns, naive (assumes independent trades) | 2.41 |
| t-stat, clustered by entry month (review tstat.py, CR1) | 1.35 (G = 116 months) |
| Friction (costs + slippage) as % of pre-friction gross P&L | 23.92 |
| Largest symbol share of net P&L | NSE:MARUTI 15.45% |
| Net P&L, % of combined starting capital (whole window) | 1.31% |
| Net P&L without the best 5 trades, % of combined capital | 0.81% |
| Median per-symbol Sharpe (cash-diluted; NOT the protocol Sharpe) | 0.05 |
| Median per-symbol max drawdown % | 3.70 |

## Gates

All edge gates (3a-3e, 4a-4f) are NOT-ASSESSABLE from independent per-symbol accounts: each account has its own cash and position cap, so the pooled trades could never all be taken in one account. Use a `--mode portfolio` run for gate verdicts.

## Per symbol

| Symbol | Bars | Trades | Win % | PF | Net P&L (Rs) | Return % | Max DD % | Sharpe | Kill switch | Rejected | Size 0 | Non-member | ADV-capped |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| ACC | 2465 | 10 | 50.00 | 2.25 | 28826.82 | 2.88 | 2.85 | 0.24 | - | 0 | 0 | 37 | 0 |
| ADANIPORTS | 2465 | 12 | 25.00 | 1.73 | 31974.44 | 3.20 | 5.06 | 0.20 | 2021-10-22 | 0 | 0 | 26 | 0 |
| AMBUJACEM | 2465 | 16 | 25.00 | 0.52 | -31117.94 | -3.11 | 4.02 | -0.26 | - | 0 | 0 | 38 | 0 |
| ASIANPAINT | 2465 | 26 | 38.46 | 2.17 | 84283.15 | 8.45 | 3.87 | 0.44 | - | 0 | 0 | 6 | 0 |
| AUROPHARMA | 2465 | 4 | 25.00 | 0.06 | -14634.16 | -1.46 | 1.56 | -0.35 | - | 0 | 0 | 119 | 0 |
| AXISBANK | 2465 | 23 | 26.09 | 1.14 | 11599.57 | 1.16 | 5.15 | 0.08 | 2020-09-01 | 19 | 0 | 0 | 0 |
| BAJAJ-AUTO | 2465 | 23 | 34.78 | 1.30 | 19947.52 | 1.99 | 4.44 | 0.16 | - | 0 | 0 | 0 | 0 |
| BAJAJFINSV | 2465 | 10 | 50.00 | 5.02 | 80920.49 | 8.09 | 2.30 | 0.56 | - | 0 | 0 | 73 | 0 |
| BAJFINANCE | 2465 | 13 | 46.15 | 2.20 | 35451.23 | 3.55 | 3.81 | 0.30 | - | 0 | 0 | 105 | 0 |
| BANKBARODA | 2465 | 11 | 36.36 | 0.43 | -19888.44 | -1.99 | 4.77 | -0.15 | - | 0 | 0 | 26 | 0 |
| BHARTIARTL | 2465 | 11 | 18.18 | 0.25 | -34778.15 | -3.48 | 5.25 | -0.49 | 2016-04-01 | 43 | 0 | 0 | 0 |
| BHEL | 2465 | 10 | 20.00 | 0.49 | -14741.32 | -1.47 | 2.96 | -0.13 | - | 0 | 0 | 23 | 0 |
| BOSCHLTD | 2465 | 7 | 14.29 | 0.11 | -22912.89 | -2.29 | 2.94 | -0.45 | - | 0 | 0 | 55 | 0 |
| BPCL | 2465 | 16 | 31.25 | 1.74 | 37078.20 | 3.71 | 5.01 | 0.25 | 2017-08-09 | 25 | 0 | 0 | 0 |
| BRITANNIA | 2465 | 4 | 75.00 | 5.13 | 22122.56 | 2.21 | 1.07 | 0.32 | - | 0 | 0 | 103 | 0 |
| CIPLA | 2465 | 18 | 33.33 | 1.27 | 16872.78 | 1.69 | 4.60 | 0.13 | - | 0 | 0 | 0 | 0 |
| COALINDIA | 2465 | 18 | 22.22 | 0.59 | -30025.34 | -3.00 | 5.39 | -0.24 | 2019-05-07 | 20 | 0 | 0 | 0 |
| DIVISLAB | 2465 | 3 | 66.67 | 3.90 | 14992.45 | 1.50 | 1.36 | 0.29 | - | 0 | 0 | 112 | 0 |
| DLF | 2465 | 8 | 0 | 0 | -35948.43 | -3.59 | 4.14 | -0.45 | - | 0 | 0 | 72 | 0 |
| DRREDDY | 2465 | 16 | 25.00 | 0.25 | -45785.77 | -4.58 | 5.16 | -0.33 | 2017-07-24 | 36 | 0 | 0 | 0 |
| EICHERMOT | 2465 | 12 | 50.00 | 1.57 | 14388.29 | 1.44 | 2.07 | 0.14 | - | 0 | 0 | 64 | 0 |
| GAIL | 2465 | 17 | 23.53 | 1.18 | 9871.71 | 0.99 | 3.93 | 0.07 | - | 0 | 0 | 3 | 0 |
| GRASIM | 2465 | 18 | 33.33 | 2.59 | 86102.05 | 8.61 | 3.11 | 0.52 | - | 0 | 0 | 12 | 0 |
| HCLTECH | 2465 | 14 | 35.71 | 2.97 | 92123.00 | 9.21 | 5.35 | 0.55 | 2019-02-12 | 42 | 0 | 0 | 0 |
| HDFCBANK | 2465 | 20 | 45.00 | 3.92 | 137623.31 | 13.76 | 2.35 | 0.69 | - | 0 | 0 | 0 | 0 |
| HDFCLIFE | 1018 | 3 | 33.33 | 1.16 | 1281.14 | 0.13 | 1.85 | 0.04 | - | 0 | 0 | 23 | 0 |
| HEROMOTOCO | 2465 | 22 | 27.27 | 0.96 | -3654.56 | -0.37 | 4.29 | -0.02 | - | 0 | 0 | 0 | 0 |
| HINDALCO | 2465 | 25 | 24.00 | 1.01 | 1003.43 | 0.10 | 3.99 | 0.01 | - | 0 | 0 | 0 | 0 |
| HINDPETRO | 2465 | 1 | 0 | 0 | -5184.38 | -0.52 | 1.14 | -0.16 | - | 0 | 0 | 110 | 0 |
| HINDUNILVR | 2465 | 26 | 38.46 | 1.71 | 58652.79 | 5.87 | 4.66 | 0.32 | - | 0 | 0 | 0 | 0 |
| ICICIBANK | 2465 | 20 | 25.00 | 0.38 | -44297.00 | -4.43 | 5.03 | -0.44 | 2017-11-30 | 59 | 0 | 0 | 0 |
| IDEA | 2465 | 3 | 0 | 0 | -11976.41 | -1.20 | 1.60 | -0.26 | - | 0 | 0 | 47 | 0 |
| INDUSINDBK | 2465 | 18 | 50.00 | 2.51 | 73257.40 | 7.33 | 3.70 | 0.44 | - | 0 | 0 | 14 | 0 |
| INDUSTOWER | 2222 | 6 | 16.67 | 0.37 | -13480.76 | -1.35 | 2.48 | -0.18 | - | 0 | 0 | 28 | 0 |
| INFY | 2465 | 14 | 28.57 | 1.12 | 5936.54 | 0.59 | 5.01 | 0.05 | 2017-12-01 | 79 | 0 | 0 | 0 |
| IOC | 2465 | 9 | 44.44 | 1.03 | 803.65 | 0.08 | 3.27 | 0.01 | - | 0 | 0 | 56 | 0 |
| ITC | 2465 | 18 | 22.22 | 0.55 | -31474.99 | -3.15 | 5.31 | -0.22 | 2018-01-29 | 31 | 0 | 0 | 0 |
| JINDALSTEL | 2465 | 7 | 0 | 0 | -31187.29 | -3.12 | 3.83 | -0.59 | - | 0 | 0 | 88 | 0 |
| JSWSTEEL | 2465 | 7 | 42.86 | 4.09 | 51951.08 | 5.20 | 2.22 | 0.49 | - | 0 | 0 | 95 | 0 |
| KOTAKBANK | 2465 | 27 | 37.04 | 1.22 | 20364.70 | 2.04 | 3.31 | 0.14 | - | 0 | 0 | 0 | 0 |
| LT | 2465 | 23 | 26.09 | 1.25 | 20405.93 | 2.04 | 5.21 | 0.14 | 2020-08-31 | 24 | 0 | 0 | 0 |
| LUPIN | 2465 | 11 | 27.27 | 2.26 | 44958.66 | 4.50 | 4.17 | 0.37 | - | 0 | 0 | 23 | 0 |
| M&M | 2465 | 16 | 18.75 | 1.39 | 21807.25 | 2.18 | 5.14 | 0.15 | 2018-04-18 | 23 | 0 | 0 | 0 |
| MARUTI | 2465 | 22 | 40.91 | 3.01 | 151582.82 | 15.16 | 5.43 | 0.83 | 2021-10-13 | 2 | 0 | 0 | 0 |
| NESTLEIND | 2465 | 6 | 50.00 | 1.64 | 7467.38 | 0.75 | 1.58 | 0.10 | - | 0 | 0 | 89 | 0 |
| NMDC | 2465 | 2 | 100 | n/a | 13208.85 | 1.32 | 1.86 | 0.22 | - | 0 | 0 | 91 | 0 |
| NTPC | 2465 | 11 | 9.09 | 0.02 | -47693.37 | -4.77 | 5.32 | -0.55 | 2016-08-10 | 35 | 0 | 0 | 0 |
| ONGC | 2465 | 17 | 41.18 | 2.05 | 53720.92 | 5.37 | 3.26 | 0.30 | - | 0 | 0 | 0 | 0 |
| PNB | 2465 | 9 | 11.11 | 0.74 | -10325.55 | -1.03 | 3.56 | -0.10 | - | 0 | 0 | 39 | 0 |
| POWERGRID | 2465 | 19 | 15.79 | 0.53 | -33865.98 | -3.39 | 5.47 | -0.25 | 2019-06-27 | 17 | 0 | 0 | 0 |
| RCOM | 2465 | 1 | 0 | 0 | -3615.84 | -0.36 | 1.05 | -0.12 | - | 0 | 0 | 99 | 0 |
| RELIANCE | 2465 | 27 | 33.33 | 1.09 | 6931.80 | 0.69 | 4.90 | 0.05 | - | 0 | 0 | 0 | 0 |
| RELINFRA | 2465 | 3 | 66.67 | 0.86 | -707.11 | -0.07 | 1.88 | -0.01 | - | 0 | 0 | 68 | 0 |
| RPOWER | 2465 | 1 | 100 | n/a | 784.57 | 0.08 | 1.60 | 0.02 | - | 0 | 0 | 79 | 0 |
| SAIL | 2465 | 1 | 0 | 0 | -5190.57 | -0.52 | 0.53 | -0.38 | - | 0 | 0 | 80 | 0 |
| SAMMAANCAP | 2080 | 8 | 12.50 | 0.02 | -35516.38 | -3.55 | 3.82 | -0.68 | - | 0 | 0 | 61 | 0 |
| SBILIFE | 1050 | 3 | 33.33 | 1.15 | 1752.93 | 0.18 | 1.72 | 0.06 | - | 0 | 0 | 25 | 0 |
| SBIN | 2465 | 19 | 21.05 | 0.92 | -5389.37 | -0.54 | 5.05 | -0.03 | 2020-02-03 | 32 | 0 | 0 | 0 |
| SHREECEM | 2465 | 4 | 50.00 | 0.78 | -1445.70 | -0.14 | 1.69 | -0.02 | - | 0 | 0 | 94 | 0 |
| SIEMENS | 2465 | 2 | 0 | 0 | -9755.75 | -0.98 | 1.48 | -0.23 | - | 0 | 0 | 90 | 0 |
| SUNPHARMA | 2465 | 17 | 17.65 | 0.86 | -9718.81 | -0.97 | 5.23 | -0.08 | 2017-12-28 | 29 | 0 | 0 | 0 |
| TATACONSUM | 2465 | 2 | 50.00 | 3.02 | 10535.38 | 1.05 | 0.76 | 0.27 | - | 0 | 0 | 96 | 0 |
| TATAPOWER | 2465 | 9 | 22.22 | 0.22 | -27062.18 | -2.71 | 3.03 | -0.35 | - | 0 | 0 | 49 | 0 |
| TATASTEEL | 2465 | 23 | 39.13 | 2.22 | 86995.55 | 8.70 | 2.93 | 0.49 | - | 0 | 0 | 0 | 0 |
| TCS | 2465 | 18 | 33.33 | 0.48 | -31275.82 | -3.13 | 5.01 | -0.27 | 2017-06-07 | 60 | 0 | 0 | 0 |
| TECHM | 2465 | 14 | 35.71 | 4.10 | 111398.50 | 12.09 | 2.44 | 0.76 | - | 0 | 0 | 39 | 0 |
| TITAN | 2465 | 11 | 27.27 | 1.31 | 13769.76 | 1.38 | 3.73 | 0.12 | - | 0 | 0 | 66 | 0 |
| TMPV | 2465 | 14 | 28.57 | 0.43 | -28404.49 | -2.84 | 5.28 | -0.28 | 2015-12-02 | 62 | 0 | 0 | 0 |
| ULTRACEMCO | 2465 | 20 | 35.00 | 1.54 | 33455.15 | 3.35 | 3.40 | 0.22 | - | 0 | 0 | 17 | 0 |
| UNITDSPR | 2465 | 1 | 0 | 0 | -5204.48 | -0.52 | 0.52 | -0.69 | - | 0 | 0 | 89 | 0 |
| UPL | 2465 | 8 | 25.00 | 2.18 | 31757.78 | 3.18 | 2.31 | 0.31 | - | 0 | 0 | 62 | 0 |
| VEDL | 2465 | 11 | 36.36 | 1.20 | 6218.57 | 0.62 | 2.49 | 0.06 | - | 0 | 0 | 51 | 0 |
| WIPRO | 2465 | 19 | 36.84 | 2.48 | 87894.97 | 8.79 | 4.36 | 0.42 | - | 0 | 0 | 9 | 0 |
| YESBANK | 2465 | 9 | 22.22 | 1.22 | 8192.70 | 0.82 | 3.65 | 0.09 | - | 0 | 0 | 61 | 0 |
| ZEEL | 2465 | 13 | 30.77 | 0.52 | -22794.50 | -2.28 | 4.32 | -0.23 | - | 0 | 0 | 34 | 0 |

## Not run

- CAIRN: no data for NSE:CAIRN
- HDFC: no data for NSE:HDFC
- IDFC: no data for NSE:IDFC
- JPASSOCIAT: no data for NSE:JPASSOCIAT
- RANBAXY: no data for NSE:RANBAXY
- STER: no data for NSE:STER

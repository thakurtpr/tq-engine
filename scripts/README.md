# scripts

## fetch_daily.py

Downloads free daily OHLCV for NSE stocks from Yahoo Finance using
[`yfinance`](https://pypi.org/project/yfinance/), and writes one CSV per ticker to
`engine/data/daily/<SYMBOL>.csv`. This is the format `tq_data::CsvDaily` reads:

```
date,open,high,low,close,volume
2024-01-01,100.25,101.60,99.15,100.50,101000
```

Dates are NSE trading days (YYYY-MM-DD). The engine stamps each one at the 15:30 IST close.
No API key or account is needed.

```sh
pip install yfinance
python3 scripts/fetch_daily.py RELIANCE INFY TCS HDFCBANK          # 2015-01-01 to today
python3 scripts/fetch_daily.py --tickers-file nifty50.txt --start 2018-01-01 --end 2024-12-31
python3 scripts/fetch_daily.py --help
```

- Give tickers without the `.NS` suffix; the script adds it. A `.NS` you include is stripped.
- Prices are adjusted for splits, bonuses and dividends (`auto_adjust=True`) and rounded to 2 dp.
- Each file is fully rewritten through a temp file, so an interrupted run never leaves a half-written CSV.
- A ticker with missing values, duplicate dates or invalid OHLC fails loudly instead of being
  patched. The other tickers continue, and the exit code is 1 if any ticker failed.
- `data/` is in `.gitignore`. Downloaded data is never committed.

After fetching, load the data with `CsvDaily` and run `tq_data::quality_report` on each series.
It checks for gaps, zero-volume days and suspicious jumps.

## Limitations and known biases

**Unofficial data.** Yahoo Finance is not an exchange feed. It has no SLA and can contain
wrong prints, missing days, stale or late corrections, and adjustment errors. The yfinance
library also breaks from time to time when Yahoo changes its site. Use this data for research
only. Before risking money, cross-check against NSE bhavcopy or a broker feed such as Kite historical.

**Split and bonus adjustment.** `auto_adjust=True` should back-adjust prices for corporate
actions, but Yahoo sometimes misses or misdates them, especially bonus issues on Indian stocks.
An unadjusted 1:1 bonus shows up as a ~50% one-day drop. Always check the `jumps` field of
`quality_report` (close-to-close moves above 20%). Check each flagged date against the NSE
corporate-actions list before trusting a backtest on that symbol.

**Survivorship bias.** If you fetch a list of tickers that exist today, every stock that was
delisted, merged or dropped from the index along the way is left out, and those are mostly
the losers. A backtest on today's constituents therefore looks better than it would have
done in real time. Treat results from this data as an upper bound. The fix is to use
point-in-time index constituents (e.g. NIFTY 500 membership as of each rebalance date) and
include delisted names. Plan for this before trusting any portfolio-level result.

## fetch_binance_klines.py (crypto 1-minute bars)

Public Binance archive at <https://data.binance.vision> (no account, no keys). Downloads monthly spot
1m kline zips with their `.CHECKSUM` files, verifies SHA-256, and extracts to
`engine/data/intraday/<SYMBOL>/<SYMBOL>-1m-YYYY-MM.csv`. Standard library only; re-runs skip months already present.

```sh
python3 scripts/fetch_binance_klines.py                       # BTCUSDT ETHUSDT, 2022-01..2025-12 (~600 MB)
python3 scripts/fetch_binance_klines.py SOLUSDT --start 2023-01 --end 2023-12
```

Rows are headerless (`open_time,open,high,low,close,volume,close_time,...`). **From 2025-01 Binance
stamps spot archives in microseconds**; `tq_data::IntradayCsv` converts to ms. 2024-2025 is the
pre-registered holdout for the intraday strategies: `tq-live --replay` refuses to read it.

## fetch_nse_intraday.py (NSE 5m and 1m bars, very short history)

```sh
.venv/bin/python scripts/fetch_nse_intraday.py                 # RELIANCE HDFCBANK ICICIBANK INFY TCS
```

Writes `engine/data/intraday-nse/<5m|1m>/<SYMBOL>.csv` (`ts_ms,open,high,low,close,volume`, bar open
in UTC ms, unadjusted). **Yahoo serves only the last 60 days of 5m bars (about 58 sessions, 4,380 bars
per stock on 2026-10-30) and the last ~30 days of 1m bars (about 20 sessions, 6,850 bars).** That is
one market regime and a few dozen trades per strategy: enough to exercise the code, nowhere near enough
to validate an edge. Proper NSE intraday history needs a paid source (e.g. Kite Connect historical,
Rs 500/month; see `docs/intraday-costs-and-access.md`), or recording live bars over months. Yahoo also
emits a post-close 15:30+ print; the runner ignores bars outside 09:15-15:30 IST.

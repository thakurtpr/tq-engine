# tq-engine

A Rust research and backtesting engine for NSE equities (and later crypto spot), built around
one rule: **every order passes a hard risk gate, and every backtest pays realistic costs.**

> Not financial advice. No system guarantees profit or zero loss. This engine runs
> backtests and paper trading only; there is no live broker.

## Crates

| Crate | Purpose |
| --- | --- |
| `tq-core` | Domain types. Money is `Decimal`, never `f64`. Injectable clock. |
| `tq-risk` | Risk gate: risk per trade, position cap, daily loss, drawdown kill switch. Brokers only accept `ApprovedOrder`. Live trading is blocked until the user confirms the limits. |
| `tq-costs` | Indian cost model: brokerage, STT, exchange, SEBI, stamp, GST, DP. Rates are config. |
| `tq-strategy` | `Strategy` trait emitting `Enter` / `Exit` signals; same code for backtest and paper. |
| `tq-strategies` | Concrete strategies (edge unproven until walk-forward tested net of costs). |
| `tq-indicators` | Streaming O(1) indicators. |
| `tq-data` | Bar sources; rejects unordered or invalid data. |
| `tq-broker` | `Paper` broker only. |
| `tq-backtest` | Event-driven backtester; orders fill at the next bar's open (no look-ahead). `run_portfolio` steps all symbols in ONE shared account (the headline mode); per-symbol runs are a diagnostic. Adopted slippage model, month-clustered t, CAGR/Calmar/exposure. |
| `tq-cli` | Command-line entry point. |

## Run

```sh
cargo test
cargo run -p tq-cli
# Portfolio backtest with the adopted slippage model and the NIFTY 50 price index as benchmark;
# writes reports/.../manifests/manifest-<sha>.csv (SHA-256 of every input CSV) and cites it:
./target/release/tq backtest --strategy donchian --data-dir data/daily-pit \
    --universe-file data-ref/nifty50_members_by_date.csv --aliases data-ref/nifty50_price_aliases.csv \
    --start 2012-01-01 --end 2021-12-31 --max-positions 5 --benchmark data/index/NSEI.csv --out report.md
```

Latest results: [reports/backtest-03.md](reports/backtest-03.md) (no demonstrated edge).

## Roadmap

1. Backtester with positions, costs, equity curve and metrics
2. NSE daily data loader and data-quality checks
3. Walk-forward and Monte Carlo validation
4. Paper trading for 60+ sessions
5. Guarded live trading only if 4 is convincing, with per-order human approval

## License

MIT

# Rust Trading Research Platform: Plan

Goal: a fast, safe, Rust-native platform for research, backtesting and (later) guarded execution.
Reality check up front: **no system can guarantee zero loss.** SEBI's FY22-FY24 study (quoted in the
Claude x Zerodha Kite guide) found 93% of individual F&O traders lost money. The design target is
*bounded* loss: hard risk limits, honest backtests, paper trading first.

## What the Kite guide (MadeToScale, 6 pp.) actually contains
1. Kite MCP: `npx mcp-remote https://mcp.kite.trade/mcp` (Claude Desktop) or
   `claude mcp add --transport http kite https://mcp.kite.trade/mcp` (Claude Code). Login happens on
   Zerodha's own page; session expires daily. Never type password/TOTP/PIN in chat.
2. Safety: turn OFF every order tool (place/modify/cancel, GTT create/modify/delete). Keep read-only
   (profile, margins, holdings, positions, quotes, OHLC, history, order history). Never "Always allow".
3. 10 Indian trading skills (github.com/ajeeshworkspace/indian-trading-skills, MIT): technical analyst,
   NSE VCP screener, stock analysis, scenario analyzer, FII/DII flows, options strategy advisor,
   backtest expert (STT, stamp, exchange, SEBI, GST), market breadth, news + SEBI circulars,
   weekly F&O planner. Note: its backtest excludes brokerage.
4. Six research prompts (screener, FII/DII, news, options, backtest, portfolio risk), each ending with
   "state sources and data dates".
5. Checklist: order tools off, no credentials in chat, verify numbers, fix position size first,
   not financial advice.

## Reference repos (cloned in `repos/`) and what to borrow
| Repo | Borrow |
| --- | --- |
| nautilus_trader | Closest to the target: Rust core, event-driven engine, same code for backtest and live. Study its architecture first. |
| freqtrade | Strategy interface, hyperopt, dry-run mode, FreqAI, backtest reports |
| ccxt | Unified exchange API: model as a Rust `Exchange` trait |
| hummingbot | Market-making loops, connectors, inventory skew |
| jesse | Clean strategy DSL, indicator set, walk-forward testing |
| FinRL | RL environments for trading (gym-style). Research only. |
| ai-hedge-fund | Multi-agent LLM analysts producing signals, not orders |
| eliza | Agent/plugin framework patterns |

## Proposed Rust architecture (Cargo workspace)
- `core`: types (`Decimal` money, never f64), instruments, orders, events, clock (injectable for replay)
- `data`: Kite historical + websocket ticks, NSE/BSE bhavcopy, Parquet store (polars / arrow), DuckDB queries
- `indicators`: SMA/EMA/ATR/RSI/VWAP, opening range, VCP, bull flag
- `strategy`: `trait Strategy { on_bar, on_tick, on_fill }`, identical in backtest and live
- `backtest`: event-driven engine, realistic fills, slippage, **full Indian cost model** (brokerage, STT,
  stamp, exchange, SEBI, GST), walk-forward and Monte Carlo (like trader-timemachine)
- `risk`: hard gate every order must pass: max loss/trade, daily loss limit, position cap, no
  naked options, kill switch. Cannot be bypassed by strategies or agents.
- `broker`: `trait Broker` with `Paper` (default) and `Kite` impls; live requires explicit human approval
- `agents`: LLM analysts via MCP (Rust `rmcp` SDK) that output research and signals only
- `cli` / `tui` (ratatui) / optional `axum` dashboard
- Crates: tokio, reqwest, serde, rust_decimal, polars, duckdb, sqlx, tracing, clap, ratatui, axum, rmcp

## Roadmap
1. **Read-only research (week 1-2):** connect Kite MCP read-only, install indian-trading-skills, review
   nautilus architecture, pick ONE market and ONE simple strategy family.
2. **Data + backtester (week 3-6):** `core`, `data`, `backtest` with the full cost model; reproduce a
   known result from freqtrade/jesse to validate the engine.
3. **Validation:** out-of-sample + walk-forward + Monte Carlo; reject anything that does not survive costs.
4. **Paper trading (month 2-3):** `Paper` broker on live data for at least 60 sessions.
5. **Guarded live (only if 4 is convincing):** tiny size, risk gate, per-order human approval, kill switch.

## Decided: both markets
- **India (Kite):** equities first, intraday/swing, read-only Kite MCP for research, `Kite` broker impl later.
  Costs: brokerage, STT, stamp, exchange, SEBI, GST. Hours 09:15-15:30 IST.
- **Crypto (ccxt-style):** 24/7 spot first (no leverage), exchange `trait` modelled on ccxt, maker/taker
  fees + funding for perps later. Testnet/paper only until validated.
- Same `Strategy`, `risk` and `backtest` crates serve both; only `data` and `broker` adapters and the
  cost models differ. Build and validate India equities end-to-end first, then add the crypto adapter.

## Still needed from the user
- Market: (decided: India + crypto; US stocks out of scope for now)
- Style: intraday, swing, or market making?
- Capital and max loss per trade/day you can truly afford to lose.

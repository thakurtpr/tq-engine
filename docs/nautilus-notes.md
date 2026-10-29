# nautilus_trader: what to learn from it

Read from `repos/nautilus_trader/docs/concepts/` (architecture, backtesting/, live, orders, message_bus).
Claims are from the docs, not from running the code.

## 1. Engine structure

Style: DDD + event-driven + pub/sub messaging + ports-and-adapters + crash-only (architecture.md).
Everything lives inside one `NautilusKernel` that owns the shared core:

| Component | Job |
| --- | --- |
| `MessageBus` | Routes data, commands and events between components: pub/sub, request/response, typed endpoints |
| `Cache` | In-memory store of instruments, accounts, orders, positions, recent bars/quotes; optional DB backing |
| `DataEngine` | Takes data from `DataClient`s, writes it to the cache, then publishes it on topics |
| `RiskEngine` | Sits between strategies and execution; pre-trade checks; emits `OrderDenied` |
| `ExecutionEngine` | Routes commands to `ExecutionClient`s, tracks order/position state, applies venue events |
| `Portfolio` | Derived state: balances, net positions, PnL, exposure, margin |
| `Trader` | Registers actors/strategies/algorithms and manages their lifecycle |

Three environment contexts share the same kernel: `Backtest` (historical data, simulated execution),
`Sandbox` (live data, simulated execution) and `Live` (live data, real or paper venue accounts).
Strategy code is the same in all three.

Threading: the core (bus, strategies, risk, cache) is **single-threaded**, LMAX-style. Network, logging
and persistence run on separate async tasks and talk to the core through channels. A single thread gives
deterministic ordering and no locks in strategy code.

Crates under `crates/` (~30): `core`, `model` (domain types), `common` (clock, bus, cache), `data`,
`execution`, `risk`, `portfolio`, `trading`, `backtest`, `live`, `system` (kernel), `indicators`,
`adapters/*`, `persistence`. Dependency flow is one-way: domain first, infrastructure last.

## 2. Message bus

- Topic strings derive from the data identity, e.g. `data.quotes.BINANCE.BTCUSDT-PERP`.
- Market data order of operations: adapter -> channel -> `DataEngine` -> **cache write first** ->
  bus publish -> strategy `on_quote` / `on_bar`. Strategies read current state from the cache.
- Commands go through typed endpoints (point-to-point); events are published (pub/sub).
- Each thread has its own bus instance; channels bridge threads. Optional external backing (Redis) is
  for live transport, not durable recovery.

## 3. Order lifecycle

Flow (architecture.md, "life of an order"): `Strategy.submit_order` -> `RiskEngine` checks (fail ->
`OrderDenied`, never reaches the venue) -> `ExecutionEngine` routes -> `ExecutionClient` -> venue.
Events come back as `OrderAccepted`, `OrderFilled`, `Canceled`, `Rejected`, `Expired`; the engine
updates the cache, then publishes to the strategy, and fills update positions and the portfolio.

Order status machine (orders/index.md): `INITIALIZED` -> `SUBMITTED` -> `ACCEPTED` ->
(`TRIGGERED`) -> `PARTIALLY_FILLED` -> `FILLED`; side states `PENDING_UPDATE`, `PENDING_CANCEL`;
terminal `DENIED`, `REJECTED`, `CANCELED`, `EXPIRED`, `FILLED`. Open, in-flight and closed are
distinct sets, so "is this order still live?" is a defined question.

Components also have their own FSM (`PRE_INITIALIZED` -> `READY` -> `STARTING` -> `RUNNING` ->
`STOPPING` -> `STOPPED`, or `FAULTING`), with `ComponentTrigger`s constraining legal transitions.

## 4. Backtest vs live parity

- Same kernel, same engines, same strategy code; only the data source, the venue (simulated vs real)
  and the clock change. The clock is injected (controlled in backtest, wall in live).
- Backtest loop per data point (backtesting/execution-flow.md): (1) the simulated exchange updates its
  book and matches **resting** orders, (2) strategy callbacks run and may submit orders, (3) venues
  settle queued commands, repeating until no cascading orders remain, then the clock advances.
  Resting orders therefore see the new market before newly submitted orders do.
- Bar timestamp rule (bar-execution.md): a bar's `ts_init` must be the **close** of its interval, so a
  bar never becomes visible before it has formed (no look-ahead). Bars carry no intrabar path, so the
  simulator builds a plausible OHLC sequence.
- Fill models (fill-models.md) add configurable slippage and probabilistic limit fills.
- Known differences (live.md): venue rules, unknown command outcomes after network failure,
  non-FIFO interleaving of live inputs, persistence not gating dispatch, externally created orders.
  Reconciliation (execution/reconciliation.md) aligns local state with venue reports on startup.
- Crash-only: unrecoverable invariant violations abort the process; an external supervisor restarts it
  and normal startup does recovery. Venue commands are never blindly retried.

## 5. Five things to copy into `engine/`

1. **Risk gate as a mandatory hop with an explicit denied event.** Orders go
   `Strategy -> RiskGate -> Broker`; a failed check yields an `OrderDenied` event the strategy sees,
   and the broker accepts only an `ApprovedOrder`. Keep the type-level gate; add the denied event and
   per-reason codes.
2. **One kernel, three contexts.** `Backtest | Paper | Live` differ only by data feed, broker adapter
   and clock; strategy, risk and portfolio code are shared. Add an integration test that runs the same
   strategy through backtest and paper and compares order/fill streams.
3. **Explicit order-state machine with legal transitions.** Model `OrderStatus` as an enum plus a
   transition function that rejects illegal moves; distinguish open / in-flight / closed. Do the same
   for component lifecycle (`Starting/Running/Stopping/Faulted`).
4. **Backtest event ordering and the no-look-ahead rule.** Per bar: match resting orders against the
   new bar first, then call the strategy, then settle newly submitted orders (cascades in the same
   timestamp). Enforce "a bar is visible only at its close"; with daily bars, signals at the close fill
   at the next open. Write these down as invariants and test them.
5. **Cache-then-publish and a single-threaded deterministic core.** Update state (cache/portfolio)
   before notifying subscribers, process events serially, keep I/O on async tasks bridged by channels.
   For live: abort on invariant violation, recover through normal startup plus reconciliation against
   broker state, never blind-retry orders.

Also worth borrowing later: configurable fill/slippage models, a startup reconciliation report,
`Portfolio` as state derived from events (event sourcing), and tearsheet-style reports.

## What not to copy

Nautilus is large (Python bindings, many adapters, options, FX rollover, order emulation). For an NSE
swing-trading engine, skip emulated orders, execution algos, order books and multi-venue routing until
a strategy has shown an edge after costs.

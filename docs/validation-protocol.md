# Validation protocol: what a strategy must pass before paper trading

Purpose: stop us from paper trading (or ever risking money on) a strategy whose backtest is luck,
overfitting or cost-blind. Every threshold is net of the Indian cost model (`costs-india.md`) plus
slippage. **Thresholds marked [convention] are my proposed defaults, not published constants; the user
decides them. Items marked [cited] come from the papers listed at the end.** Strict on purpose:
rejecting a real edge costs time, accepting a fake one costs money.

Passing this protocol does NOT mean the strategy will make money. It means we found no evidence it
won't. Paper trading and live trading are separate gates.

## 0. Rules of the game (integrity gate; any failure = stop)

1. Universe is point-in-time and survivorship-safe (`universe.md`); prices are split/bonus/dividend adjusted.
2. Signals use only data up to bar close; orders fill at the **next bar's open** (as `tq-backtest` does).
   Shift test: delaying every signal by one extra bar must not *improve* net results beyond noise.
3. Costs on every trade: full Zerodha schedule from `tq-costs`, plus slippage **5 bps per side**
   [convention] for NIFTY 50 names. Round-trip cost on a Rs 1,000 stock is about 0.24% delivery /
   0.08% intraday before slippage.
4. A **trials ledger** (CSV in repo; fields `date` ISO YYYY-MM-DD, `strategy_id`, `params`,
   `per_period_sharpe`, `result`) records every strategy idea and parameter set ever run, including
   discarded ones. N used below = number of *independent* trials in the ledger.
5. Held-out data is touched **once**. If the holdout result leads to any change, that holdout is burned:
   the changed strategy is a new trial (N+1) and its next out-of-sample is forward/paper data only.

## 1. Data split [convention]

| Window | Dates | Use |
| --- | --- | --- |
| Development (in-sample) | 2012-01-01 to 2021-12-31 (10 y) | research, parameter choice, walk-forward |
| Held-out (out-of-sample) | 2022-01-01 to 2026-09-30 (about 4.75 y) | single final test, never used for tuning |

Do not move these dates after seeing results. Do not use holdout statistics (volatility, regime) to
choose parameters.

## 2. Walk-forward (inside development window) [convention]

- Rolling windows: **train 3 years, test 6 months, step 6 months** -> first test 2015-01-01, last test
  ending 2021-12-31, **14 folds**. Re-fit parameters on each train window only; score only on its test.
- **Embargo/purge:** leave a gap of at least the maximum holding period (and not less than 10 trading
  days) between train end and test start, and drop training trades that overlap the test window
  (purging, per López de Prado, *Advances in Financial Machine Learning*, ch. 7).
- Stitch the 14 test segments into one out-of-sample equity curve and report all metrics on it.
- A strategy with fixed, no-fitted parameters still runs through the same folds so fold stability is visible.

## 3. Minimum sample [convention]

- >= **200** closed trades over the development window and >= **100** in the stitched walk-forward OOS.
- >= **30** closed trades in the holdout (fewer = the holdout cannot reject anything; extend and wait).
- >= 10 distinct symbols; no single symbol > 15% of net P&L; dropping the 5 best trades must leave net P&L > 0.
- Why: the standard error of a mean trade return falls as 1/sqrt(n); at n=100 it is still 10% of the
  trade-return standard deviation, so small samples cannot support a thin edge.

## 4. Edge net of costs (stitched walk-forward OOS, then holdout) [convention unless noted]

| Metric | Pass |
| --- | --- |
| Mean net return per trade | >= 0.5% (about 2x the delivery round-trip cost) and expectancy >= +0.20R |
| t-statistic of mean net trade return | >= **3.0** [cited: Harvey, Liu & Zhu 2016, new factors need t > 3.0 after multiple testing] |
| Profit factor (net) | >= 1.3 |
| Annualised Sharpe (daily equity returns, rf = 0, net) | >= 0.8 OOS and >= 0.5 in holdout |
| Versus benchmark (NIFTY 50 TRI, same dates, `universe.md`) | strategy Sharpe AND Calmar both exceed buy-and-hold, otherwise buy the index |
| Holdout consistency | holdout net return > 0 and holdout Sharpe >= 50% of walk-forward OOS Sharpe |

## 5. Robustness [convention]

- **Parameter neighbourhood:** vary each parameter +/-20% (and +/-1 step for integers). >= 70% of
  neighbours stay net-profitable and the median neighbour Sharpe >= 70% of the chosen one. A sharp peak = overfit.
- **Cost stress:** doubling brokerage/fees and slippage must keep net expectancy > 0.
- **Fold stability:** >= 60% of the 14 folds net-positive; worst fold max drawdown <= the kill-switch limit.
- **Random-entry baseline:** generate 1,000 random strategies with the same number of trades, holding
  periods and position sizes; the strategy's net return must exceed the 95th percentile.

## 6. Multiple-testing correction [cited]

- Record N (trials ledger) and each trial's per-period Sharpe. Compute the **Deflated Sharpe Ratio**
  (Bailey & López de Prado 2014) on the selected strategy: **DSR >= 0.95** [convention for the cut-off].
  Formulas as I recall them from the paper (**re-verify against the PDF before coding; I could not
  re-fetch the formulas this session**):
  - SR0 = sqrt(V[SR_n]) * ( (1 - g) * Phi^-1(1 - 1/N) + g * Phi^-1(1 - 1/(N*e)) ), with g = 0.5772 (Euler-Mascheroni)
  - DSR = Phi( (SR_hat - SR0) * sqrt(T - 1) / sqrt(1 - skew * SR_hat + ((kurt - 1) / 4) * SR_hat^2) )
  where SR_hat is the per-period (not annualised) Sharpe, T the number of return observations, skew and
  kurt the sample skewness and (non-excess) kurtosis, V[SR_n] the variance of the trial Sharpes.
- **Minimum backtest length:** more trials need more data. Per Bailey, Borwein, López de Prado & Zhu
  (2014), with 5 years of data testing more than about 45 independent configurations will almost surely
  find one with an in-sample annualised Sharpe of 1 and an expected out-of-sample Sharpe of 0. We have
  10 development years; still keep N small and log honestly. (The 45 figure is theoretical; check the
  paper's assumptions.)
- If the strategy comes from a parameter grid of >= 20 configurations, also compute the **Probability of
  Backtest Overfitting** by combinatorially symmetric cross-validation (Bailey, Borwein, López de Prado
  & Zhu): **PBO <= 0.20** [convention].

## 7. Monte Carlo on trades (10,000 runs) [convention]

On the stitched OOS net trade returns, at the intended position sizing:
1. **Order shuffle** (permute, no replacement) and **bootstrap** (with replacement), 10,000 each.
2. Record max drawdown (MDD) and final equity for each run.
3. Pass if: **95th-percentile MDD <= 0.8 x the kill-switch drawdown limit**; at least 95% of runs end
   profitable; probability of hitting the kill switch within 2 years <= 5%.
4. If it fails only on drawdown, scale risk per trade down (MDD scales roughly linearly with size) and
   re-run; if no sensible size passes, the strategy fails.

Note for the user: the placeholder limits (1% risk per trade, 5% drawdown kill switch) are close
together: five consecutive full-stop losses is already a 5% drawdown, so a 40-50% win-rate strategy
will trip the kill switch often. Expect this gate to force either smaller risk per trade or a wider
kill switch. That is a user decision, not mine.

## 8. After the protocol

Pass all of 0-7 -> eligible for **paper trading**: >= 60 sessions and >= 20 trades, results must sit
inside the Monte Carlo 5th-95th percentile band and net expectancy must remain positive. Anything
else goes back to research as a new trial. Live trading is a separate human decision and is outside this document.

## Sources

- Bailey, D. H. & López de Prado, M. (2014), "The Deflated Sharpe Ratio: Correcting for Selection
  Bias, Backtest Overfitting and Non-Normality", Journal of Portfolio Management 40(5), 94-107.
  <https://papers.ssrn.com/abstract=2460551>
- Bailey, D. H., Borwein, J. M., López de Prado, M. & Zhu, Q. J. (2017), "The Probability of Backtest
  Overfitting", Journal of Computational Finance. <https://papers.ssrn.com/abstract=2326253>
- Bailey, Borwein, López de Prado & Zhu (2014), "Pseudo-Mathematics and Financial Charlatanism: The
  Effects of Backtest Overfitting on Out-of-Sample Performance", Notices of the AMS.
  <https://carmamaths.org/jon/backtest.pdf>
- Harvey, C. R., Liu, Y. & Zhu, H. (2016), "... and the Cross-Section of Expected Returns", Review of
  Financial Studies (t-statistic hurdle above 3.0 under multiple testing).
- López de Prado, M. (2018), *Advances in Financial Machine Learning*, Wiley (purged cross-validation
  and embargo; cited from knowledge, not fetched here).
- Walk-forward practice: Pardo, *The Evaluation and Optimization of Trading Strategies* (2008), cited
  from knowledge, not fetched here; the 3y/6m window sizes are my [convention].

I confirmed the paper titles, venues and headline claims through search results and abstracts, but did
not read the full PDFs. Verify formulas and exact claims against the papers before encoding them.

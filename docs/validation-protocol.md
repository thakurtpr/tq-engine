# Validation protocol: what a strategy must pass before paper trading

## Changelog

- **2026-10-30, v2** (amended after `review-backtest-01.md`, which reproduced backtest-01/02 exactly and
  showed the pooled statistics overstate what a real account could earn):
  1. **Headline metrics come from one shared-capital portfolio account.** Per-symbol (independent
     account) runs are diagnostic only and decide nothing (new rules 0.6 and 0.7).
  2. **Gate 4c changed:** the pooled per-trade t-stat (which assumed independent trades) is replaced by a
     t-stat on **portfolio daily net returns with Newey-West (HAC) standard errors**, cross-checked by a
     month-clustered / monthly-return t. Per-trade statistics remain as diagnostics only.
  3. **Section 3:** trade counts are trades the portfolio actually took, not signals.
  4. **Section 5:** cost stress now follows `slippage-model.md` (1x/2x/3x).
  5. **Section 6:** the trials ledger's `per_period_sharpe` must come from portfolio daily returns.
  6. **Section 7:** Monte Carlo uses month-block resampling of portfolio returns; new **kill-switch
     interaction** note (a shared account at 0.5% risk and a 5% kill switch latched after about 17 trades).
  7. Reporting rules (new 0.7): return on capital instead of summed rupees; slippage counted as friction;
     data manifest with hashes.
- 2026-10-30, v1: initial protocol.

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
3. Costs on every trade: full Zerodha schedule from `tq-costs`, plus slippage per `slippage-model.md`
   (adopted 2026-10-30: **5 bps per side** up to 0.05% of 20-day ADV, the impact formula above that,
   **2x on stop exits**, orders capped at 1% of 20-day ADV). Round-trip cost on a Rs 1,000 stock is about
   0.24% delivery / 0.08% intraday before slippage.
4. A **trials ledger** (CSV in repo; fields `date` ISO YYYY-MM-DD, `strategy_id`, `params`,
   `per_period_sharpe`, `result`) records every strategy idea and parameter set ever run, including
   discarded ones. N used below = number of *independent* trials in the ledger.
5. Held-out data is touched **once**. If the holdout result leads to any change, that holdout is burned:
   the changed strategy is a new trial (N+1) and its next out-of-sample is forward/paper data only.
6. **(v2) Headline metrics come from one shared-capital portfolio account.** One cash balance, the risk
   gate's position limit (`max_open_positions`), the stated risk-per-trade and notional cap applied to
   portfolio equity, and a documented, deterministic **priority rule** for choosing among simultaneous
   signals. No trade may be taken that a real account of that size could not hold. Because the priority rule
   is arbitrary, also report a **random-priority sensitivity** (>= 20 seeds, median and 5th-95th
   percentile of CAGR, Sharpe, max drawdown and the daily-return t) and judge pass/fail on the **median**
   and require the 5th percentile not to be catastrophic (net CAGR > 0 at the 5th percentile).
   Runs that give every symbol its own independent account are **diagnostic only**: label them as such,
   and never use them for a pass/fail gate, Rs totals or a Sharpe.
7. **(v2) Reporting rules.** Report return on capital (CAGR, max drawdown %), not rupee totals summed across
   accounts. Count slippage as friction alongside costs (gross P&L must be stated before slippage, or the
   report must say it is after slippage). `per_period_sharpe` in the trials ledger comes from portfolio daily
   returns. Archive the exact data files behind each report with a manifest of SHA-256 hashes.

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

- Counts below are **trades the shared-capital portfolio actually took** (signals rejected for lack of a
  free position slot or cash are reported separately and do not count). In backtest-02 only about 300
  of 1,085 signalled trades fit in a 5-position account.
- >= **200** closed trades over the development window and >= **100** in the stitched walk-forward OOS.
- >= **30** closed trades in the holdout (fewer = the holdout cannot reject anything; extend and wait).
- >= 10 distinct symbols; no single symbol > 15% of net P&L; dropping the 5 best trades must leave net P&L > 0.
- Why: the standard error of a mean trade return falls as 1/sqrt(n); at n=100 it is still 10% of the
  trade-return standard deviation, so small samples cannot support a thin edge.

## 4. Edge net of costs (stitched walk-forward OOS, then holdout) [convention unless noted]

**(v2) All rows are measured on the shared-capital portfolio** (rule 0.6), net of costs and slippage.

| Gate | Metric | Pass |
| --- | --- | --- |
| 4c | **t-statistic of portfolio daily net returns**, Newey-West (HAC) standard error (see below) | >= **3.0** on the development window and walk-forward OOS [cited: Harvey, Liu & Zhu 2016, new factors need t > 3.0 after multiple testing]. Holdout: **>= 1.65** (one-sided 5%, a single pre-registered test) [convention] |
| 4c-x | Cross-check: t-stat of the **monthly** net portfolio returns, and the month-clustered t of per-trade returns | report both; if either is more than 0.5 below the HAC t, the gate uses the **lowest** of the three |
| 4e | Annualised Sharpe (daily portfolio returns, rf = 0, net) | >= 0.8 OOS and >= 0.5 in holdout |
| 4f | Versus benchmark (NIFTY 50 TRI, same dates, `universe.md`) | portfolio Sharpe AND Calmar both exceed buy-and-hold, otherwise buy the index |
| 4g | Holdout consistency | holdout net return > 0 and holdout Sharpe >= 50% of walk-forward OOS Sharpe |
| 4h | Portfolio max drawdown and CAGR | reported; max drawdown is judged in section 7 against the kill switch |
| 4a, 4b, 4d | Per-trade **diagnostics** on trades the portfolio took: mean net trade >= 0.5% (about 2x the delivery round-trip cost), expectancy >= +0.20R, profit factor >= 1.3 | sanity floors only; **passing them never substitutes for 4c/4e/4f**. Also report a month-block bootstrap 95% interval for the mean trade; its lower bound should be > 0 |

**How the 4c t-stat is computed (v2).** Take the portfolio's daily net return series `r_t` (equity
change per day, after costs and slippage; flat days included, not dropped). `t = mean(r) / SE`, with
`SE` the Newey-West / Bartlett-kernel HAC standard error of the mean, bandwidth
`L = max( floor(4 * (T/100)^(2/9)), longest holding period in trading days )`, capped at `T/4` (T = number
of daily observations). The Newey-West rule is the one commonly implemented as the automatic default;
using the holding period as a floor accounts for overlapping trades. For an i.i.d.-like return series,
`t` is about `annualised Sharpe x sqrt(years)`, so t >= 3.0 over the 10-year development window needs an
annualised Sharpe of roughly 0.95, which is why the holdout (about 4.75 years) uses the single-test
threshold 1.65 instead (about a 0.76 Sharpe over 4.75 years, so this t, not the 0.5 Sharpe floor, is the
binding holdout test). Why the change: the old pooled per-trade t treated overlapping, market-wide trades as
independent (backtest-02 Donchian: 2.41 pooled, 1.35 month-clustered, 0.84 year-clustered).

## 5. Robustness [convention]

- **Parameter neighbourhood:** vary each parameter +/-20% (and +/-1 step for integers). >= 70% of
  neighbours stay net-profitable and the median neighbour Sharpe >= 70% of the chosen one. A sharp peak = overfit.
- **Cost stress (v2, per `slippage-model.md`):** run at 1x, 2x and 3x slippage (and 2x brokerage/fees at
  2x). **2x must keep the portfolio net-profitable with a positive daily-return t**; 3x is report-only (label
  the strategy fragile if net return falls by more than half from 1x to 2x).
- **Fold stability:** >= 60% of the 14 folds net-positive; worst fold max drawdown <= the kill-switch limit.
- **Random-entry baseline:** generate 1,000 random strategies with the same number of trades, holding
  periods and position sizes; the strategy's net return must exceed the 95th percentile.

## 6. Multiple-testing correction [cited]

- Record N (trials ledger) and each trial's per-period Sharpe, **taken from the shared-capital portfolio's
  daily returns** (v2; the earlier ledger rows hold a median per-symbol Sharpe diluted by idle cash, which
  is not a valid input). Because daily returns are autocorrelated when positions overlap, also compute
  the DSR on monthly returns and use the more conservative result. Compute the **Deflated Sharpe Ratio**
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

## 7. Monte Carlo (10,000 runs) [convention]

On the stitched OOS **portfolio** returns, at the intended position sizing:
1. **(v2) Month-block bootstrap** of the portfolio's daily net returns (resample whole calendar months
   with replacement, 10,000 runs). A plain shuffle or bootstrap of individual trades treats overlapping
   trades as independent and understates drawdown; keep the trade-level **order shuffle** and
   **bootstrap** (10,000 each) only as a secondary comparison.
2. Record max drawdown (MDD) and final equity for each run.
3. Pass if: **95th-percentile MDD <= 0.8 x the kill-switch drawdown limit**; at least 95% of runs end
   profitable; probability of hitting the kill switch within 2 years <= 5%.
4. If it fails only on drawdown, scale risk per trade down (MDD scales roughly linearly with size) and
   re-run; if no sensible size passes, the strategy fails.

Note for the user: the placeholder limits (1% risk per trade, 5% drawdown kill switch) are close
together: five consecutive full-stop losses is already a 5% drawdown, so a 40-50% win-rate strategy
will trip the kill switch often. Expect this gate to force either smaller risk per trade or a wider
kill switch. That is a user decision, not mine.

### Kill-switch interaction (added 2026-10-30, v2)

Evidence from `review-backtest-01.md`: with 0.5% risk per trade and the 5% drawdown kill switch on, the
shared-capital Donchian account on the point-in-time universe **latched after about 17 trades and ended
-3.1%**, and with the switch off its max drawdown was **21-41%** (5 or 10 positions). A backtest kill
switch latches for the rest of the run; in live use a human would review and reset it, so one number
cannot describe both. Rules:
1. **Edge assessment runs with the kill switch OFF** (a measurement, never a candidate configuration), so
   a single early drawdown does not truncate the sample. Gates 4c, 4e, 4f use this run.
2. **Also report the kill-switch-ON operating run:** date and trade number when it latches, the share of
   the window traded, and the final return. A strategy that latches in the first third of the window is
   described as such.
3. **The drawdown gate above uses the kill-switch-off run.** A strategy whose 95th-percentile MDD exceeds
   0.8 x the kill-switch limit **fails section 7 regardless of its edge**, until the user changes risk per
   trade, position limits or the kill-switch level. Raising a limit to rescue a strategy after seeing its
   drawdown is not allowed; set the limits first (they are the user's, `UNSET-BY-USER` until confirmed).
4. MDD scales roughly linearly with risk per trade and grows with the position limit (point-in-time
   Donchian: 21-25% at 5 positions, 31-41% at 10), so those two settings are tested together.
5. Per-symbol accounts understate this: each is mostly in cash, so their median max drawdown is only
   3.7-4.6% even though 19-21 of them still latched. That is one more reason they are diagnostic only.

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
- Newey, W. K. & West, K. D. (1987), "A Simple, Positive Semi-Definite, Heteroskedasticity and
  Autocorrelation Consistent Covariance Matrix", Econometrica 55(3), 703-708; and (1994), "Automatic Lag
  Selection in Covariance Matrix Estimation", Review of Economic Studies 61(4), 631-653 (the
  `floor(4 (T/100)^(2/9))` bandwidth rule, as commonly implemented). Cited from knowledge, not fetched here.
- Walk-forward practice: Pardo, *The Evaluation and Optimization of Trading Strategies* (2008), cited
  from knowledge, not fetched here; the 3y/6m window sizes are my [convention].

I confirmed the paper titles, venues and headline claims through search results and abstracts, but did
not read the full PDFs. Verify formulas and exact claims against the papers before encoding them.

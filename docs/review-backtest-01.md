# Independent review of backtest-01 and backtest-02

Reviewer: research-review session, 2026-10-30. Scope: `engine/reports/backtest-01.md`, `backtest-02.md`
and `reports/backtest-0{1,2}/*.md`, `research/trials.csv`, against `docs/validation-protocol.md`. I did
not edit anything under `engine/`. Scripts and exact commands: `docs/review-backtest-01-scripts/`.

## Verdict

**The FAIL verdicts are right, and for Donchian the true picture is worse than backtest-02 says.**

1. **The engine arithmetic is correct.** I rebuilt the Donchian strategy, sizing, costs, stops and kill
   switch from scratch in Python (from the Rust source's stated rules) and ran it on the raw CSVs. It
   reproduces **every per-symbol trade count and net P&L** in both reports, to the paisa (section 1).
2. **No look-ahead or effective-date error in the membership gating.** Entries are allowed on the fill
   date from the snapshot's effective date and blocked from the removal date (section 2).
3. **The reported Donchian statistics do not describe a tradeable strategy.** The "75 independent Rs 10
   lakh accounts" design implies about 16 open positions on an average day (up to 45) and total notional
   up to about 4x one account's capital. The same trades in **one Rs 10 lakh shared account** (max 5 or
   10 positions) give a point-in-time CAGR of about **1-2.5%, Sharpe about 0.2, max drawdown 21-41%** and
   a per-trade t-stat below 1.5 (section 3).
4. **The t-stat is computed correctly but its independence assumption fails.** Trades overlap heavily and
   are market-wide. Clustering by entry month cuts the point-in-time t from 2.41 to **1.35** (0.84 by
   year); a block bootstrap interval for the mean trade includes zero (section 4).
5. **backtest-02's wording ("weak, unproven in-sample effect"; several "PASS in-sample") overstates.** The
   defensible statement is **no demonstrated edge** (section 5).

Neither strategy is eligible for paper trading. I would not spend more effort tuning either until a
portfolio-level run exists.

## 1. Does the engine do what the reports say?

Method: independent replica (`replica.py`), no shared code with Rust: next-open fills, 5 bps slippage per
side, `min(open, stop)` stop exits, risk-sized entries (0.5% of previous-close equity, 20% notional cap,
gap-through-stop rejected), Zerodha delivery costs with STT rounded half-up per order and the trade total
rounded once, 5% drawdown kill switch latching, membership windows built by me from
`nifty50_members_by_date.csv` plus the alias map.

| Run | Reported | My replica |
| --- | --- | --- |
| backtest-01 Donchian, kill switch on | 882 trades, net Rs 26,36,915.55 | 882 trades, Rs 26,36,915.56 |
| backtest-01 Donchian, kill switch off | 1,053 trades, Rs 26,99,776.71 | 1,053 trades, Rs 26,99,776.71 |
| backtest-02 Donchian, kill switch on | 940 trades, Rs 9,81,214.04 | 940 trades, Rs 9,81,214.07 |
| backtest-02 Donchian, kill switch off | 1,085 trades, Rs 10,26,004.15 | 1,085 trades, Rs 10,26,004.15 |

All 49 (backtest-01) and 75 (backtest-02) per-symbol rows match, including the count of entries blocked as
non-member. Differences of a few paise are the per-symbol tables' 2-decimal rounding.

**Three trades audited by hand** from the raw rows with a separate, deliberately plain script (`audit.py`),
tied to single-trade symbols so the report's symbol P&L *is* the trade:

| Symbol | Signal bar (close > prior 55-bar high) | Entry fill | Exit | Net, hand vs report |
| --- | --- | --- | --- | --- |
| RPOWER | 2012-02-09: 104.87 > 102.57; ATR(20) 4.1295; stop 96.6111 | 2012-02-10 open 105.12 -> 105.17256; 584 sh (risk 5,000 / 8.5615) | 2012-04-24 open (signal exit after 2012-04-23 close 106.83 < 20-bar low 110.11) at 106.776585 | **+784.57 = +784.57** |
| RCOM | 2012-01-24: 94.14 > 92.60; ATR 4.1353; stop 85.8693 | 2012-01-25 open 94.98 -> 95.02749; 545 sh | 2012-02-28 open 88.62 after 2012-02-27 close 87.28 < 88.52 | **-3,615.84 = -3,615.84** |
| HINDPETRO | 2019-03-11: 123.73 > 119.95; ATR 3.9630; stop 115.8040 | 2019-03-12 open 124.90 -> 124.96245; 545 sh | **Stop:** 2019-04-23 low 115.56 <= stop; open 117.43, so exit at the stop 115.804 less 5 bps | **-5,184.38 = -5,184.38** |

(HINDPETRO also shows the removal rule working: the stop fired after its 2019-03-29 removal, and exits
are never blocked.) Hand workings, including the cost lines, are printed by `audit.py`.

I verified these report claims too: ADANIENT net Rs 5.86 lakh, Rs 1.18 lakh if the run ends 2019-12-31
(**80%** of its P&L after 2019); the corrupt ADANIENT bar 2015-06-03 (O 294.19, L 54.57, C 56.32);
current constituents +Rs 9.28 lakh over 656 trades vs former members +Rs 0.54 lakh over 284; MARUTI at
15.45% of net P&L.

Limits of this check: I replicated from the Rust *source*, so a misreading of the rules shared by both
would not show. I did not run `tq`, did not replicate TrendPullback (I read its signal code and found no
look-ahead), and did not independently verify the Yahoo prices against NSE.

## 2. Look-ahead and survivorship

**Membership timing (your question 1): correct.** Bars carry `ist_close_ts(date)`; entry windows are
`[ist_close_ts(from), ist_close_ts(to))` over the snapshot's effective dates and are tested against the
**fill** bar. So a stock added on effective date D can fill at D's open, and a stock removed on D cannot
enter at D's open. Signals are not gated, so a pre-membership signal that fills on the first member day is
allowed. That is legitimate (membership is announced weeks ahead). The strategy reads only data up to the
signal bar's close: the Donchian channel excludes the current bar, and the strategy's `no_look_ahead` test
exists. I found no look-ahead.

**Residual survivorship/data issues** (all mentioned in the reports except the last two):
- 4.57% of member-days have no prices (HDFC Ltd for the whole decade, plus Cairn, IDFC, Jaiprakash,
  Ranbaxy, Sterlite). The direction of the bias is unknown.
- Only 4 of the 45 change dates are checked against NSE primary releases; the rest are Wikipedia.
- "95.43% covered" counts that a price file exists, not that it is the right series. TMPV (pre-demerger Tata
  Motors) and VEDL (pre-2013 = Sesa Goa?) are unverified identities for large, long-time members.
- Re-fetch instability (2% of rows differ) means the results are reproducible only if the data files are
  archived and hashed. Agreed with the reports' suggestion; do it before backtest-03.
- **New:** "gross P&L" in the reports is **after** slippage (slippage is in the fill prices), so "costs are
  86-101% of gross" omits the 10 bps round-trip slippage. Total friction is larger than stated.
- **New:** prices are dividend-adjusted back-fills, so early-year share prices and turnover are understated;
  the flat Rs 15.34 DP charge looks slightly heavier than in reality (small, conservative).

## 3. Independent accounts vs a shared-capital portfolio (your question 2)

The pooled statistics sum or average 49-75 independent Rs 10 lakh accounts, each with its own cash and
its own 5-position limit that can never bind. That is not capital anyone has.

Implied concurrency (from the 1,053 / 1,085 no-kill-switch trades):

| Universe | Mean open positions | Max | Days with > 5 | Days with total notional > Rs 10 lakh | Peak notional |
| --- | --- | --- | --- | --- | --- |
| backtest-01 (current 50) | 16.3 | 44 | 85% | 70% | Rs 37.9 lakh |
| backtest-02 (PIT) | 15.8 | 45 | 82% | 68% | Rs 39.7 lakh |

So the pooled results assume up to ~4x leverage and ignore that the gate allows only 5 positions.

I re-ran the same strategy and rules in **one Rs 10 lakh account** (shared cash; risk 0.5% of portfolio
equity; 20% notional cap; same costs and 5 bps slippage; kill switch off; ties broken at random, 8 seeds,
plus a "strongest breakout first" ranking). Simplified simulator, indicative not definitive:

| Universe, max positions | Trades | t-stat of mean trade | CAGR | Max DD | Sharpe |
| --- | --- | --- | --- | --- | --- |
| **PIT, 5** | 290-313 | 0.42 to 1.43 (median 0.77) | -0.3% to 2.7% (median 1.0%) | 20.7-24.7% | 0.00-0.38 |
| **PIT, 10** | 497-523 | -0.13 to 1.14 | -0.05% to 3.1% (median 2.3%) | 31-41% | 0.06-0.31 |
| Current 50 (survivor), 5 | 276-298 | 1.85-2.82 | 4.2-6.2% | 18-22% | 0.55-0.76 |
| Current 50 (survivor), 10 | 485-511 | 2.37-3.01 | 7.3-10.1% | 26-30% | 0.63-0.84 |

Read-outs: (a) survivorship still costs about 4-7 points of CAGR at portfolio level; (b) the point-in-time
Donchian portfolio makes roughly nothing, with drawdowns of 21-41%, which no 5% kill switch tolerates;
(c) with the **5% kill switch on**, the shared account latches after about 17 trades and ends -3.1%, so the
placeholder kill switch and a portfolio-level run cannot coexist; (d) the benchmark was not measured; as
general knowledge (unverified) the NIFTY 50 compounded at low double digits per year over 2012-2021, so
none of these would beat buy-and-hold. Compute it from the TRI data before quoting it.

## 4. The t-statistic (your question 4)

Computed correctly: `mean / (sd / sqrt(n))` of per-trade net return % on the entry notional; I reproduce
3.73 and 2.41 exactly. The problem is independence, not arithmetic. On average each trade overlaps with
about **30 other trades** (30.2 in backtest-01, 31.0 in backtest-02), and trend following makes them
cluster in the same market-wide moves.

| | Naive t | Cluster by entry month (G=116) | by entry year (G=10) | Block-bootstrap 95% CI of mean trade (months), P(mean <= 0) |
| --- | --- | --- | --- | --- |
| backtest-01 (current 50), n=882 | 3.73 | **2.51** (effective n about 400) | 1.87 | [1.17, 6.89]%, 0.000 |
| backtest-02 (PIT), n=940 | 2.41 | **1.35** (effective n about 295) | 0.84 | **[-0.41, 2.68]%**, 0.088 |

Clustering by symbol barely moves it (2.55 for PIT), so the dependence is across time, not across names.
Other points:
- Heavy tails: backtest-01 skew 16 and kurtosis 360; backtest-02 skew 3.1, kurtosis 16. A normal-theory t
  is unreliable; a handful of trades (ADANIENT 2020-21) carries backtest-01.
- The mean is an equal-weight average of percent returns, while profit factor and net P&L are
  rupee-weighted; they answer different questions.
- Protocol gate 4c (t >= 3.0) was written assuming independent trades. Suggest amending it to a
  **cluster-robust or portfolio-daily-return t-stat**, and requiring >= 3.0 on that.

## 5. Overclaiming and presentation (specific)

1. backtest-02 gate table marks Donchian 4a, 4b, 4d **"PASS in-sample"**. These hold only if all 940 trades
   can be taken at once, which the engine's own 5-position gate forbids. Mark them *not assessable until a
   portfolio-level run*.
2. "What is left (t = 2.41, PF 1.32) is a weak, unproven in-sample effect": the evidence supports "no
   demonstrated edge" (cluster t 1.35, bootstrap CI includes zero, portfolio CAGR 1-2%).
3. **Rs lakh totals** sum 75 accounts (Rs 7.5 crore of notional capital): Rs 9.8 lakh net is +1.3% on that
   base over ten years. Report return on capital, not summed rupees.
4. **"Median per-symbol Sharpe"** is not the protocol's Sharpe (cash-diluted, per account), and it is what
   `research/trials.csv` records as `per_period_sharpe`. Those ledger values are the wrong input for a deflated
   Sharpe; recompute from portfolio daily returns.
5. "Costs as % of gross" excludes slippage (section 2). For TrendPullback the honest line is that total
   friction (costs + 10 bps slippage) exceeds the strategy's gross move.
6. backtest-02's "conclusions do not depend on the placeholder kill switch": true per account, false for any
   portfolio (section 3c).
7. Gates 0.2a/0.3/0.5 "PASS": fine against the protocol as written, but the adopted slippage model (2x on
   stop exits, 1% ADV cap) is not implemented yet; state that.

What the reports got right and should keep: next-open fills with a tested no-look-ahead rule, the control
that separates universe from data noise, the explicit survivorship caveat, the holdout CLI guard, the
no-kill-switch measurement, reporting data anomalies, and honest FAILs.

## 6. Recommendations, in order

1. **Portfolio-level engine first** (shared cash, max positions, deterministic priority rule, random-priority
   sensitivity). Make its CAGR, Sharpe, max drawdown and NIFTY 50 TRI comparison the headline; keep the
   per-symbol pooled table as a diagnostic.
2. Amend protocol 4c as above; report month-clustered t and a block-bootstrap interval for every
   pooled-trade claim.
3. Fix the presentation items in section 5; compute DSR from portfolio returns.
4. Data: obtain HDFC Ltd and the other missing member series and corporate actions from NSE; verify the
   TMPV and VEDL series; archive and hash `data/` per report.
5. Implement the adopted slippage model before backtest-03.
6. Decide, with the user, how the kill switch and risk per trade relate at portfolio level (point 3c).
7. Power: with about 100-300 independent observations in one decade, even a real 1% per trade edge would be
   hard to detect. Plan for that rather than keep adding variants (each one adds to N in the trials ledger).

## Reproducing

`docs/review-backtest-01-scripts/`: `replica.py` (engine), `pit.py` (membership windows, backtest-02
comparison), `compare1.py` (backtest-01 comparison), `audit.py` (3 hand audits), `conc.py` (concurrency),
`portfolio.py` + `sens.py` (shared-capital runs), `tstat.py` (clustered t, bootstrap). They read the
engine's `data/daily*`, `data-ref/` and `reports/` read-only; run with `python3 -I`.

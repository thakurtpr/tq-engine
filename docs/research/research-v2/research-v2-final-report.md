# Research-v2 final report (development window): nothing passed

Dated 2026-11-17. Paper and backtest only. Holdouts untouched. Nothing pushed.

## Verdict
**No candidate passed. All 14 runnable candidates are killed on the development window; none reached a development GO, so none is promoted to forward paper.** The default stays a Nifty ETF plus BTC. A development GO, had one existed, would be evidence to watch on forward paper, not proof of profit.

**Read this verdict correctly:** "nothing passed" does NOT mean "no edge exists". The frozen gates, at the real number of trials, can only be cleared by a very large edge (see "What this search can and cannot say").

## Every candidate tested (development window; IN-EQ out-of-sample 2015-01-01 to 2021-12-31, CRYPTO 2018-10-01 to 2026-10-31)
| Candidate | Cells | Net return (OOS) | Net CAGR | Alpha t (needs 3.0) | Sharpe (needs 0.8) | Drawdown sim p95 (limit 24%) | Gates failed |
|---|---|---|---|---|---|---|---|
| Large-cap trend, SMA | 9 | +31.3% | 4.0% | -1.57 | 0.38 | 45% | 21 |
| Large-cap trend, Donchian | 9 | +13.5% | 1.8% | -2.20 | 0.22 | 44% | 21 |
| Large-cap trend, time-series momentum | 9 | +76.4% | 8.5% | -0.36 | 0.66 | 44% | 20 |
| Large-cap cross-sectional momentum (closest) | 27 | +89.6% | 9.6% | -0.32 | 0.55 | 57% | 19 |
| Large-cap mean reversion, RSI(2) | 9 | -39.1% | -6.8% | -2.46 | -0.95 | 58% | 27 |
| Large-cap mean reversion, z-score | 9 | -9.5% | -1.4% | -0.65 | -0.26 | 35% | 24 |
| Large-cap mean reversion, IBS | 9 | -67.5% | -14.8% | -4.81 | -1.86 | 79% | 28 |
| Crypto trend, SMA | 9 | +411% | 22.6% | 0.56 | 0.54 | 89% | 14 |
| Crypto trend, Donchian | 9 | +105% | 9.4% | -0.61 | 0.41 | 69% | 11 |
| Crypto trend, time-series momentum | 9 | +450% | 23.8% | 0.62 | 0.54 | 89% | 14 |
| Crypto cross-sectional momentum | 27 | not valid | not valid | not valid | not valid | not valid | 2 evaluable |
| Crypto mean reversion, RSI(2) | 9 | -29.8% | -4.3% | -3.13 | -0.66 | 43% | 18 |
| Crypto mean reversion, z-score | 9 | -15.1% | -2.0% | -2.44 | -0.56 | 25% | 18 |
| Crypto mean reversion, IBS | 9 | -65.7% | -12.5% | -4.85 | -1.16 | 77% | 18 |

Returns are the pre-registered base cell, net of costs and tax. Trials used: 173 of 450 (162 main budget, 11 reserve); effective trials for deflation N_true = 537.

What the numbers say:
- **Trend following (long-only):** earns something, but the alpha is statistically indistinguishable from zero or negative and it earns less risk-adjusted than buying the benchmark. Crypto trend is a part-time long-BTC position (beta 0.2 to 0.4) that earns less than holding BTC (weekly Sharpe 0.5 vs 0.83) and depends on a handful of trades.
- **Large-cap cross-sectional momentum:** the closest, and it is leveraged Nifty beta (1.16) with alpha of -1.4% a year, so no alpha.
- **Mean reversion (long-only oversold):** loses after costs and tax everywhere; IBS loses 66-67%.
- **Crypto cross-sectional momentum:** its after-tax series went negative (see "Open questions"), so most numbers are not valid. It still fails on trade count and other gates that do not read the ruined series. It is a kill with a caveat, not a clean one.

## What this search can and cannot say (calibration; no gate was changed)
Planting a known net Sharpe into real development returns, at the real trial count:

| Planted net Sharpe | 1 | 2 | 3 | 4 | 6 |
|---|---|---|---|---|---|
| Large-cap 9-cell candidate passes | 0% | 0% | 3% | 3% | 0.5% |
| 27-cell families and all crypto candidates | 0% | 0% | 0% | 0% | 0% |

Zero-edge controls: 0 false passes in 18,000 evaluations. So the gates do not let junk through, but they also cannot see an edge below Sharpe 3 to 4. Mechanism: the neighbouring-parameter gates require a wide profitable plateau, and the deflated Sharpe counts that plateau as variance. Caveats: exact numbers depend on my plateau and trade-profile assumptions; 400 bootstrap draws instead of 10,000 (agreement check not run); run before the final fixes.

Also: the 24% drawdown limit is below the Nifty ETF's own worst drawdown in the window (34.3%), so the default ETF plan itself fails that gate.

## Data coverage and known biases
- Large caps: 85 point-in-time Nifty 50 members, development 2012-2021. Six former members have no price history (about 3% of member-days overall, 10% in 2012-13); long-only results are biased upward. Seven renamed-ticker aliases are unverified.
- Crypto: 20 Binance USDT spot pairs 2017-2026, perpetual and funding 2020-2026. Survivors by construction. There is NO clean historical crypto holdout (everything was seen by earlier studies), so crypto's true holdout is forward paper from 2026-11-16.
- Dividend-adjusted price levels in the large-cap store embed later dividends (raw closes not kept): ratios are safe, absolute levels affect whole-share sizing slightly (0.03% to 2.3% drift in a stress test). Not fixed; disclosed.
- Corrupted prints found and repaired by declared rules: NIFTYBEES/BANKBEES 2019-12-19/20 (benchmark and regime labels; 64 daily regime labels changed), USDINR (17 bars incl. 2012-01-27/31), zero rows in NSE India VIX. No crypto, funding or other series had a bad print.
- Benchmarks: Nifty ETF, BTC in INR, fixed 50/50 blend. Benchmarks and trade books are still supplied by the run framework, not re-derived independently.

## Not run (not failures, no trials consumed)
Pairs on large caps and crypto, short carry, weekly options, and the diagnostic families: the long-only engine cannot run short legs or market-neutral books. Intraday NSE and intraday crypto: no 5-minute or 1-minute data. The earlier studies (council, defined-risk options, basis carry, intraday, momentum rotation) were killed or inconclusive before this search.

## Harness defects found and fixed this search (each by a dated addendum written after the kills, applied identically to all 14 candidates; none changes a gate)
1. IG2 prefix-invariance required every symbol clean (impossible for short-history symbols): now zero look-ahead and at least 90% clean (addendum 3). The whole-engine look-ahead test found NO look-ahead in all 14 base cells at six cut dates.
2. Benchmark bad prints repaired (addendum 4); regime inputs, USDINR and VIX repaired, loading now fails closed (addendum 6).
3. Neighbour ordering sorted numbers as text: fixed; centre cell only.
4. Ledger cell matching (-2.0 vs -2): fixed.
5. V10 "ledger complete" demanded no unclosed sweeps: declared orphans (addendum 5).
6. The deflation set demanded closed sweeps for not-run families: excluded (addendum 7).
7. Non-physical series guard (negative after-tax equity) and the rule that a guard cannot turn a real failure into INCOMPLETE.
Process errors disclosed: one sweep opened with a stale binary (orphaned, 9 reserve cells), one with the cell-matching bug (resumed at no cost).

## Open questions for the owner (nothing changed without you)
1. **D-005 (Proposed):** continue as frozen, or pause and redesign a feasible protocol (fewer cells per candidate, deflation that does not penalise a plateau, a drawdown limit relative to the benchmark, forward paper as the main evidence).
2. **After-tax series when tax exceeds the account:** floor at -100% and stop, or treat tax as paid out of pocket. Needed before crypto cross-sectional momentum can be evaluated cleanly.
3. The 30% kill-switch limit is my assumption; Sharpe uses the stricter sqrt(252) for crypto (D-004).
4. The sealed-store key backup is on the same disk; copy it to external media.
5. Real Nifty/BTC split (the paper benchmark still uses placeholder notionals).

## State
Local commits only; tests 1,913 passing, clippy clean; no engine code pushed; holdouts untouched.

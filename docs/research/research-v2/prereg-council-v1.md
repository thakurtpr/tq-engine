# Pre-registration council-v1: does a multi-voter pre-trade council, with the number of voters earned by ablation, improve base strategies net of costs and tax?

Registered: 2026-11-06, **before any council implementation exists and before any filtered result, vote, decile or Sharpe has been computed.** Git tag `prereg-council-v1` on the commit that adds this file. Design: `docs/council-design.md` (v2, commit 41985fb).
Frozen under `docs/validation-protocol.md` v3: after a result is seen, changes may only tighten, need a dated written reason, and make the changed test a new trial. Paper and backtest only. Nothing here is evidence of profit, and passing is necessary, not sufficient, for any claim (section 10).

Ledger family: **`council-v1`**, **K = 48** trials (section 6), registered through the honesty ledger API (`Ledger::open_sweep`) in the same commit: `research/ledger-council-v1.jsonl`, anchor mirror `hardening/ledger-anchor.txt`.

## 1. Hypotheses

**H1.** For a base strategy B, the council-filtered book has, out of sample, net of the full Indian cost model, slippage and tax: (1) higher Sharpe AND Calmar than the **unfiltered** book of the same strategy; (2) higher Sharpe AND Calmar than **index buy-and-hold** (NIFTYBEES for equity; BTCUSDT spot buy-and-hold for crypto) from the same first invested day with idle cash credited; (3) lower maximum drawdown than the index.
**H2 (calibration).** Realised net return per signal rises with council confidence (monotone), and PASS signals earn significantly more than FAIL signals.
**H3 (parsimony).** Each voter that stays in the council earns its place: removing it worsens net out-of-sample Sharpe or drawdown by at least the pre-registered margin. Voters that do not are dropped, and the surviving minimal council is what is tested for GO.
**Null.** The council only reduces trade count; filtered and unfiltered per-trade expectancy are equal after costs, voters' vetoes are redundant with one another, and any Sharpe gain is the effect of holding cash.

Known weakness: the base strategies have passed no gate (`docs/rerun-after-hardening.md`, backtest-04), so the council may filter noise. The tests are built to show that if so.

## 2. Council, voters and base strategies

Voters (role table in `docs/council-design.md` section 2): **1 Scanner** (candidate generator, degenerate vote), **2 Market/News** (R1 to R6 flags; LLM headline veto not used in any registered run), **3 Entry Timer**, **4 Exit Planner**, **5 Time Machine**, **6 Risk** (existing `RiskGate::check_ctx` + kill switch), **7 Budget**.
**Veto-only when data exists:** Earnings Analyst, `NOT_AVAILABLE` (data-blocked, `docs/data-feasibility-sue.md`), logged on every candidate, excluded from the unanimity count, never a PASS. **Non-voting:** Alerts (notifications), Broker (paper broker only, executes only what all voters approved). **Excluded:** Pelosi tracker (US Congress disclosures; not applicable to NSE or crypto; lagged, non-point-in-time).
All parameters are the values in the design doc and section 3 below; **no LLM is used in any run covered here**.

Base strategies (only those in the repo; repo-default parameters, frozen):

| Id | Strategy | Frozen parameters | Universe and data |
| --- | --- | --- | --- |
| `donchian` | `tq-strategies::DonchianBreakout` | entry 55, exit 20, ATR 20, stop 2 ATR | PIT NIFTY 50 (`data-ref/nifty50_members_by_date.csv`), `data/daily-pit`, entries only while a member |
| `trend-pullback` | `tq-strategies::TrendPullback` | SMA200, EMA20/50, RSI(2) < 10, ATR 14, stop 2 ATR, max hold 10 | same |
| `momentum-rotation` | `tq-strategies::MomentumRotation` | lookback 252, skip 21, top 10 equal weight, monthly | same |
| `crypto-burst` | `tq-live::MomentumBurst` | r5 > 3 sd of previous 100, volume > 2x previous-100 mean, hold 10 bars, stop 1 ATR(14) | BTCUSDT 1m, spot 1h, funding; **no bar dated 2024-01-01 or later is read in development** |

Shared account (protocol rule 0.6): Rs 10,00,000 equity / 10,000 USDT crypto, `RiskLimits` defaults (5 open positions, 0.5% risk per trade, 20% notional cap), deterministic priority rule plus random-priority sensitivity (20 seeds, median judged), next-bar-open fills,
equity with `--calendar nse --settlement collateral:80` (strict as a stress line). Risk (voter 6) is on in **both** filtered and unfiltered runs; the council hook adds voters 2 to 5, Budget's volatility shrink and frequency cap. The unfiltered book is the base strategy through the unmodified pipeline and gate.

## 3. Metric gates (frozen numbers)

| Gate | Value | Voter |
| --- | --- | --- |
| Minimum R:R, net | `(target_dist - K) / (stop_dist + K) >= 2.0`; `K` = round-trip costs + slippage per share; target = median MFE of past same-strategy shadow trades, stop = the strategy's own stop (2 ATR(20) planning stop for momentum-rotation; sizing and R:R only, not an exit) | Exit Planner |
| Position sizing | `risk_pct = 0.5% x v`, `v = clamp(median ATR% 252d / current ATR%, 0.25, 1.0)`; `qty = size_for_risk(equity, risk_pct, entry, stop)`; notional <= 20%; max loss per trade <= 0.5% of equity; shrink only | Budget |
| Win rate | Wilson lower bound (one-sided 90%, z = 1.2816) of calibrated net win rate over analogs must exceed `p* = (1 + k)/(1 + RR)`, `k = K/stop_dist` (breakeven + cost drag) | Time Machine |
| Trade frequency | at most 1 new entry per day and 3 per week (equity), 2 per day and 6 per week (crypto); deterministic priority by Time Machine confidence then symbol; minimum evidence: bucket analogs `n >= 30` and strategy-wide `n >= 60`, else ABSTAIN | Budget (cap), Time Machine (minimum) |
| Expectancy | seeded Monte Carlo (1,000 calendar-block bootstrap draws, seed = first 8 bytes of sha256(as_of_ms or bucket)) lower bound `lcb90 > 0` AND shrunk mean `mu >= H`; `H` = **0.50% equity, 0.60% crypto** (about 2x round-trip cost), net of costs, slippage and the tax haircut | Time Machine |
| Entry condition | next-bar open only; next session tradable; signal-bar volume >= 0.5x its 20-bar median and order <= 1% ADV; `abs(close - SMA(10)) <= 2.0 ATR(14)` | Entry Timer |
| Market flags | 0 flags 1.0, 1 flag 0.5, 2+ flags 0 (P95 vol alone 0). India: 20d realised vol of NSEI vs trailing-252d P80/P95 (**VIX proxy**), Nifty < 200d mean AND PIT NIFTY 50 breadth < 30%, Nifty drawdown > 10% from 252d high, own-book drawdown 8% / 12%. Crypto: 20d vol > 1.5x 1y median, funding mean of last 3 > 0.05% per 8h or trailing-90d P95, BTC drawdown > 20% from 90d high, own-book drawdown. Declared deviations: no India VIX, Nifty 500 breadth, or OI | Market/News |

Council confidence (for H2): `c = confidence_TimeMachine x m_market` (confidence = bootstrap fraction with mean > 0). Cold-history abstentions have no `c`, are excluded from the decile test and counted.

## 4. Costs, slippage and tax (frozen)

Costs: `tq-costs` Zerodha delivery schedule (`docs/costs-india.md`); crypto taker 0.10% per side (`docs/costs-crypto.md`). Slippage: `docs/slippage-model.md` (5 bps per side up to 0.05% of 20d ADV, impact above, 2x on stop exits, orders capped at 1% ADV); crypto 5 bps per side.
**Equity tax (book level, identical for filtered, unfiltered, ablations and benchmark):** gains held under 365 days at 15% + 4% cess = 15.6%; over 365 days 10% + 4% cess above Rs 1,00,000 per financial year from FY2018-19, exempt before; gains netted within a financial year, **no loss carry-forward**; paid from the account on 31 March; the index benchmark is taxed as if liquidated on the window's last day. Post-July-2024 rates (20%, 12.5%) are recollection and apply only if a window were extended past 2024-07-23.
**Crypto tax:** 30% + 4% cess = 31.2% on each realised gain, no loss offset; 1% TDS per sale as a cash flow credited back at settlement; benchmark taxed on terminal gain. Indian-resident access to the venues is legally unverified: paper only.
The tax haircut inside the voters (`tau x max(net, 0)`, tau = 15.6% / 31.2%) is only an input estimate; headline results use the book-level model.

## 5. Data, windows, folds, holdout (frozen)

Local equity data end 2021-12-31; the protocol's 2022-01-01 to 2026-09-30 holdout is **not downloaded and stays untouched** (reserved for forward evidence). Data manifest: `docs/prereg-council-v1-data-manifest.sha256` (sha256 `3c26f77461acdf033db4de4bb951a3a48933fdf7f56c096dcfefe064dd63bd6c`).

| | Equity | Crypto |
| --- | --- | --- |
| Verifier/Time Machine warm-up (shadow trades only, not scored) | 2012-01-01 to 2014-12-31 | 2022-01-01 to 2022-12-31 |
| **Development, scored** | 2015-01-01 to 2019-12-31 | 2023-01-01 to 2023-12-31 |
| Walk-forward folds | test 6 months, step 6 months, 10 folds, 3 years trailing history | test 3 months, step 3 months, 4 folds, 12 months history |
| Purge | drop shadow trades with exit within max(hold, 10 trading days; 21 for momentum rotation) of a test start | max(hold, 24h) |
| Embargo | 21 trading days after any train segment | 7 days |
| **Single-touch holdout** | **2020-01-01 to 2021-12-31** (last 20% of the local 2012 to 2021 span; includes the 2020 crash) | **2024-01-01 to last downloaded month** (the 2024 fence) |

The holdout is **NOT RUN** until the development result passes every gate in section 8 for at least one base; it is then run once, for passing bases only, and burned whatever the outcome (protocol rule 5). No tool may read bars inside a holdout window in development except the voters' own pre-holdout shadow history (never later than scoring date minus embargo).

## 6. Trial count K and DSR deflation

Axes: `base` in {donchian, trend-pullback, momentum-rotation, crypto-burst} (4) x `variant` (12) = **K = 48**. Variants: `full` (voters 1 to 7), five **leave-one-out** councils (full without Market/News, without Entry Timer, without Exit Planner, without Time Machine, without Budget), five **solo** councils (Scanner + Risk + exactly one of the same five voters),
and `minimal` (the surviving council from section 7). Scanner and Risk are on in every variant (no candidates without Scanner; Risk is the existing gate and also runs in the unfiltered baseline). The unfiltered baseline is a control, not a trial. Sensitivities (T+2 shift, strict settlement, 4% to 8% cash rate, priority seeds) are variants of the same trials.
**DSR:** `N_b = 48 + prior ledger rows of the base strategy` (`research/trials.csv` at registration): donchian 48+114 = **162**, trend-pullback 48+62 = **110**, momentum-rotation 48+26 = **74**, crypto-burst 48+3 = **51** (lower bound: the 3 intraday strategies pre-registered 2026-10-30), crypto also reported at N = 100. Sharpe variance per the honesty layer (cell `sr_var`, floored at 1/T).

## 7. Ablation plan, pairwise vote correlation, and the surviving minimal council

Run on the **development scored window**, per base, before any GO assessment:

1. **Leave-one-out and solo runs** (section 6) for the five ablatable voters (Market/News, Entry Timer, Exit Planner, Time Machine, Budget). Scanner (degenerate) and Risk (baseline) are not ablated. Metrics per run: net-of-cost, net-of-tax Sharpe (equity at the 6% cash rate) and max drawdown, trade count, abstain rate.
2. **Retention rule (frozen).** A voter **stays** iff removing it from `full` worsens the council's dev out-of-sample result by at least ONE of: **annualised Sharpe lower by >= 0.10**, or **max drawdown deeper by >= 2.0 percentage points (absolute)**; AND the worsening has the same sign in at least 6 of 10 equity folds (3 of 4 crypto folds). A voter that fails is dropped.
3. **Redundancy rule.** For each voter pair, record the vote correlation (phi coefficient of PASS=1 / VETO=0 over candidates where both voted; ABSTAIN and NOT_AVAILABLE excluded) and the joint-veto Jaccard overlap. If two retained voters have phi >= 0.80, the one whose removal costs less Sharpe is dropped (ties: the less complex deterministic rule is kept in the order Entry Timer, Budget, Exit Planner, Market/News, Time Machine).
4. **Minimal council** = Scanner + Risk + the retained voters. It is evaluated once as the `minimal` trial. **If no voter is retained, the council adds nothing over Scanner + Risk: outcome KILL for that base ("no value added").** The minimal council is what the GO gates in section 8 are judged on and what is run on the holdout; `full` is reported alongside.
5. Solo runs and the correlation matrix are reported in full whatever the outcome. The reverse control (only candidates the council vetoed) is reported as a diagnostic.

## 8. Metrics, gates and outcomes (frozen)

Unit: daily net returns of the shared-capital account, net of costs, slippage and tax (dates per section 4). Sharpe annualised on 252 (equity) or 365 (crypto) days; equity rf = 6% (sensitivity 4% to 8%), crypto rf = 0 (sensitivity 6%). Like-for-like start (protocol rule 0.8): the same first invested day for filtered, unfiltered and index, date stated.

**Per-base GO-DEV gates, judged on the `minimal` council over the development scored window (all must hold):**

| # | Gate |
| --- | --- |
| G1 Sample | at least **100 independent closed trades** taken (no two counted trades share an entry day), in at least 24 distinct calendar months (equity) or 12 distinct weeks (crypto); at least 10 distinct symbols (equity); council not abstaining on more than 95% of signals |
| G2 Deflation | deflated Sharpe of daily net returns **>= 0.95** with `N_b` (section 6) |
| G3 Beats unfiltered | Sharpe and Calmar both above the unfiltered book; paired month-block bootstrap of the daily-return difference (10,000 draws, seed 20261106): 95% CI lower bound for the Sharpe difference > 0 |
| G4 Beats index | Sharpe (at the cash rate) and Calmar both above the index, both after costs and tax |
| G5 Drawdown | max drawdown below the index and not above the unfiltered book |
| G6 Not cash-hoarding | filtered Sharpe above the 95th percentile of a random-veto control (1,000 runs, seed 20261107, dropping the same number of the unfiltered book's entries at random); filtered net CAGR > 0 and >= 50% of the unfiltered net CAGR |
| G7 Calibration | below |
| G8 Folds | filtered net return exceeds unfiltered in at least 6 of 10 equity folds (3 of 4 crypto), with no single fold supplying more than 50% of the total advantage |
| G9 Protocol | applicable protocol v3 row (4c-abs or 4c-rel by invested-share rule; threshold 3.0 as K > 6) and 4e (Sharpe >= 0.8) hold unamended; reported separately, blocks any profit claim |
| G10 Integrity | council passes `tq bias lookahead` (prefix invariance) and a byte-identical determinism re-run before the first scored run; decision-log seal verifies |

**G7 Calibration (H2).** Pool the base's out-of-sample candidate signals (taken and vetoed, counterfactual outcomes recorded) after warm-up, excluding cold-history abstentions. Rank by `c` (ties by date, symbol) into deciles (fewer than 300 signals: `floor(N/30)` bins, minimum 5). Outcome = per-signal net-of-cost, net-of-tax return of the base trade rule.
All must hold: (i) Spearman correlation of bin index and bin mean outcome >= 0.70 with at most 2 adjacent-bin inversions in 9; (ii) PASS bucket (top 3 deciles) minus FAIL bucket (bottom 3) mean outcome > 0 with month-block bootstrap 95% CI lower bound > 0 (10,000 draws, seed 20261108); (iii) the council's actual PASS set outperforms its VETO set with the same CI rule;
(iv) reported, not gated: per-bin hit-rate, reliability table. Run pooled over the three equity bases and per base; the **per-base** result gates GO-DEV.

**Outcomes per base:** `GO-DEV` (G1 to G8, G10); `INCONCLUSIVE-SAMPLE` (G1 fails: fewer than 100 trades or more than 95% abstention, which is no information); `KILL` otherwise, including "no voter retained".
**council-v1 overall:** `GO-DEV` if at least one base is GO-DEV (only those proceed); `KILL` if none is and at least two bases (one equity) are KILL; else `INCONCLUSIVE`. A KILL is final for this configuration; reopening needs a new registration and family.
**Holdout (single touch, GO-DEV bases, minimal council):** at least 30 trades; holdout net return > 0; holdout Sharpe >= 50% of development Sharpe; beats unfiltered AND index on Sharpe and Calmar; drawdown below the index; HAC t of the applicable 4c row >= 1.65.

## 9. Required reporting regardless of outcome

Candidate counts; PASS/SHRINK/VETO/ABSTAIN counts and reason histogram by month; per-voter veto and sole-veto counts; the full pairwise correlation matrix; the 11 ablation rows per base and the retention decision per voter; trades vs unfiltered; invested share; gross vs net P&L; fold table; day/week cap hits;
`DecisionLog` seal root and anchor; code git sha; data manifest hash. Negative results get the same prominence and are committed.

## 10. What counts as a claim

`GO-DEV` plus a passing holdout shows only that the minimal council improved selectivity on these data. "Profitable" is permitted only if a pre-registered strategy passes the **unamended** `docs/validation-protocol.md` v3 gates (DSR >= 0.95, PBO <= 0.20, single-touch holdout, at least 60 paper sessions and 100 trades, net of costs and tax; STATUS rule 4).
If nothing passes, the report says "nothing passed". For base strategies with no demonstrated edge, KILL or INCONCLUSIVE is the expected and a useful outcome.

## 11. Known limits stated up front

Equity data end in 2021, so the holdout is two years with one crash; Yahoo adjusted prices and the PIT membership file are survivor-limited (`docs/universe.md`); with no base edge, power to detect a council effect is low, and Sharpe differences of 0.10 are inside sampling noise on five years (the margin is a pre-registered
screen, not inference; G3 carries the inference); the 2:1 R:R gate is expected to starve trend-pullback; the crypto leg has one scored year and a base signal whose gross edge is a few bps against about 30 bps round-trip cost, so it is expected to abstain nearly always; India VIX, Nifty 500 breadth, OI and earnings data are unavailable (Market/News is a proxy, Earnings Analyst is `NOT_AVAILABLE`);
selecting the minimal council on the same development data is a researcher degree of freedom, covered only by counting all 48 trials in the DSR; post-July-2024 tax rates are recollection; the LLM headline veto is untested.

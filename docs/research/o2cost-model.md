# tq-opt2cost: costs, slippage, margin and tax for options-v2

Crate `crates/opt2cost`. Paper/backtest only. Money is `Decimal`, data is immutable, dates are
injected. Every rate is a row (value, effective_from, source, verified) in the shared
`tq-costs` `RateTable` (`crates/costs/src/rates_data_opt.rs` holds the options-v2 rows, the
options-v1 STT steps re-recorded there). `RatesFacade` (alias `Rates`) is the typed reader; a test
proves its STT and exchange charge equal the frozen `tq-options` step tables on every day of
2012-2026. Result writers print `Rates::audit(..).note()`, e.g. `unverified rates: 3 of 5`
(`rate_uses`, `margin_rate_uses`, `TaxConfig::rate_uses` list the rows a computation consults).

Verified rows are ONLY: options/futures STT from 2026-04-01 (NSE/FATAX/73524), NSE exchange
charges from 2026-03-01 (NSE/FA/73061), index lot sizes (NSE/FAOP/70616). Every other row,
including the whole STT history before 2026-04-01 (secondary sources), is UNVERIFIED.

## 1. Costs (`trade_cost`)

Per leg, unrounded (round a whole trade once with `CostBreakdown::rounded`). Trade: Rs 20 flat
brokerage per order (x `brokerage_mult`), STT on the sell side only, stamp on the buy side only,
exchange and SEBI on premium turnover, 18% GST on brokerage + exchange + SEBI. Exercise (cash
index option, buyer): STT on intrinsic turnover, exchange and SEBI on it (conservative), no
brokerage (no order), no stamp. Expiry worthless: no charge; Expiry with a price, futures
exercise/expiry and stock-option exercise (physical settlement, F-X2) are errors (fail closed).
A missing rate row is an error, never zero. Costs are computed from actual units, never a
notional percentage.

## 2. Slippage (`slippage_per_unit`) - assumption, not an observation

No quotes exist in EOD bhavcopy, so this is a stated convention on SIGNAL-DAY volume and open
interest (in contracts; `volume_in_units` converts with the lot size):

| Tier | Condition (index) | Slip per unit per side |
|---|---|---|
| A | volume >= 2,000 and OI >= 5,000 | max(Rs 0.50, 0.5% of premium) |
| B | volume >= 100 and OI >= 1,000 | max(Rs 1.00, 1.5% of premium) |
| C | below B | untradable: skip and count; if priced anyway, 3x tier B |

Stock options: thresholds / 10, then one tier worse (A prices as B, B is untradable). Tier B uses
the stricter reading of the catalogue's "or" (both conditions): a contract failing either test is
treated as untradable, the conservative side. `mult` implements the 1x/2x/3x stress;
`fill_multiplier` gives x2 for stop/loss-cut exits and x2 for expiry-day open fills (x4 when both).
Justification: the NSE tick is Rs 0.05 and liquid ATM spreads are a few ticks (about 0.05-0.2% of a
Rs 100-200 premium), so a pure percent rule overcharges ATM; far-OTM wings of Rs 5-20 trade 0.5-1.0
wide (5-10% of premium), so the rupee floor is needed. Thresholds are guesses covered by the
stress runs.

## 3. Tax (`BusinessIncomeTax`)

Rate 31.2% = 30% slab x (1 + 4% cess) from rows `tax.business_slab_pct`, `tax.business_cess_pct`
(surcharge ignored: a lower bound; sensitivities 20%/39% by replacing `rate`). Default loss
treatment `CarryForwardOnly { years: 8 }` (assessment years, business income only);
`NoCarry` is the worst case; `SameYearOffset` refunds a loss at the rate in the year it arises
(upper bound sensitivity). `liquidation_series` takes daily PRE-TAX equity (cash + open MTM, no tax
inside) and returns after-tax equity = pre-tax - tax paid on closed years - liability accrued on the
current year's cumulative net income (provisional rule: rate x positive part after the unexpired
loss pool). Tax is paid at financial-year close (31 March). The liability is never negative (no
deferred-tax asset). The first day with after-tax equity <= 0 is a terminal RUIN: the series is
truncated, `ruin` is set, and `returns()` returns `Err(Ruin)`; there is no 0.0 return (the
options-v1 `metrics.rs:32` defect). Tax paid on old gains can ruin an account whose pre-tax equity
is still positive (no refund of later losses): this is flagged, not hidden. Matches the options-v1
`tax_payments` total on a gain/loss/gain history (test). Not modelled: tax-audit threshold and CA
fee, advance-tax interest, STT deductibility (unverified).

## 4. Margin (`structure_margin`)

* Defined-risk (single expiry, no net short call or put): exact worst expiry payoff over price 0
  and every strike, premiums included; `exposure` 0. Buyers pay the premium at entry in this cash
  model, so the 2025-02-01 upfront-premium rule (row `margin.buyer_premium_upfront`) changes
  nothing here and only adds the long premium on the SPAN-like path.
* Debit calendar/diagonal whose shorts are covered by later-dated longs (same kind, favourable
  strike, enough units): margin = net debit. From 2025-02-01 (row
  `margin.calendar_benefit_on_expiry_day`, single source) the near legs margin standalone on the
  near expiry day.
* Everything else (naked or net-short legs, futures, credit calendars) needs `MarginConfig.span`,
  the SPAN-like 16-scenario stress path, used ONLY for the labelled naked-selling stress test:
  14 scenarios (price 0, +-1/3, +-2/3, +-1 PSR x volatility +-VSR) + two +-3 PSR extremes at 35%,
  Black-76 on the forward at each leg's IV, floored by the short-option minimum charge (3% of
  notional), plus ELM (2% index; 5% stock floor, the 1.5 sigma term is not implemented) and from
  2024-11-20 an extra 2% on short options expiring today (one source says 12%); risk parts are scaled by the stress multiplier 1.25 and a
  peak-margin uplift (1.10 at 100% peak margin from 2021-09-01, phased 25/50/75/100 from
  2020-12-01). A hard loss cap is mandatory: ex-ante stress loss (+-2 PSR, vol +50%) above
  `loss_cap` is an error, and `max_lots_under_cap` sizes lots = floor(cap / stress loss per lot).
  The cap is a sizing rule, never an accounting floor. All SPAN parameters (PSR floor 8%, k 3.5,
  VSR 4 vol points, ELM, minimum charge) are placeholders: UNVERIFIED approximations to be
  replaced if NSE SPAN files are obtained.

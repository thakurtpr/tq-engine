# Cost and tax model audit, 2026-11-07

Read-only audit. No code was edited, nothing committed or pushed. Code paths are in the integration worktree (`wt/integration/...`) unless marked `wt/options/...`.

**Rates caveat.** Every "correct" rate below comes from `docs/tax-india-equity.md`, `docs/tax-india-fno.md`, `docs/tax-india-crypto.md` and `wt/options/docs/costs-tax-options-india.md`. Those docs are secondary-sourced (broker pages, CA and press explainers). No primary source (Finance Act text, NSE circular, CBDT circular) was read. Treat each rate as UNVERIFIED. This audit says "the engine differs from the research docs", not "the engine is wrong in law". Bps figures are hand arithmetic on those rates, not engine runs, unless stated.

## 1. Summary

1. **The engine's equity STT, stamp, exchange, SEBI and GST rates match the research docs.** `docs/costs-india.md` and `crates/costs/src/lib.rs` agree with `docs/tax-india-equity.md` on every cash-equity row. The only open item is the NSE exchange charge (0.00307% vs a reported 0.00297%), worth under 0.05 bps.
2. **No stale F&O STT exists in the engine.** The integration worktree has no F&O cost schedule. The options worktree (`wt/options/crates/options/src/costs.rs:27-38`) already steps STT to 0.15% from 2026-04-01 and matches `tax-india-fno.md`. The risk is a reader applying the equity schedule to F&O.
3. **The equity tax rates (15.6% STCG, 10.4% LTCG, Rs 1 lakh exemption) are stale for any trade on or after 2024-07-23** (research docs: 20%, 12.5%, Rs 1.25 lakh). They are correct for every published window (2012-2021, 2015-2019), so no published equity number is wrong because of them. They are a trap for any forward or 2024+ run.
4. **The largest gap in a published verdict is that `reports/backtest-04.md` (momentum-rotation vs NIFTYBEES) is pre-tax.** It models no capital-gains tax. Momentum at 295% one-way turnover realises mostly short-term gains every year, while buy-and-hold defers to one terminal long-term gain. Tax widens the gap against momentum by roughly 1 to 1.5 CAGR points. The "does not beat NIFTYBEES" verdict gets stronger and cannot flip.
5. **Crypto:** the 1% TDS is modelled only as cash in the live runner and is ignored in `councileval`. Taxable gains in `councileval` and `analogs` are computed after fees, which `s.115BBH` does not allow. The after-tax view in `live` uses 30%, while `councileval` and `analogs` use 31.2%. GST on exchange fees is zero. None of these can flip a published crypto verdict. Every published crypto result is already negative, and the only positive one (carry) is far below its kill line.
6. **No published result changes sign or verdict.** Section 5 lists which results are closest to a flip.

## 2. Ranked mismatches

Rank is by effect on a published verdict first, then by bps on a typical trade. "Typical equity trade" is the doc fixture: 100 shares at Rs 1,000 to Rs 1,010, buy turnover Rs 100,000 (`docs/costs-india.md`, delivery round trip Rs 238.86, which is 23.9 bps of buy turnover; STT is 20.1 bps of that). "Typical crypto trade" is Binance spot taker 0.10% a side (20 bps round trip, `docs/costs-crypto.md`).

| # | Mismatch | Engine says (file:line) | Research docs say | Effect on a typical trade | Published result exposed |
|---|---|---|---|---|---|
| 1 | **No equity capital-gains tax in the rebalancer comparison.** | `reports/backtest-04.md` is pre-tax (lines 11-14, 70-74). `crates/cli/src/backtest.rs:213` and `crates/broker/src/paper.rs:24` use `zerodha_nse_delivery()` with no tax stage. | STCG 20% and LTCG 12.5% now; 15% and 10% before 2024-07-23 (`tax-india-equity.md` s.2). Tax realised on every sale vs deferred on buy-and-hold. | Hand estimate: momentum loses about 1.7 to 2.2 CAGR points to annual STCG at 15.6% (assuming about 80% short-term). Buy-and-hold loses about 0.8 points (one LTCG hit on about 3.8x). Gap widens by about 1 to 1.5 points a year. | `reports/backtest-04.md` momentum 14.02% vs NIFTYBEES 14.31%. Verdict "neither beats NIFTYBEES" strengthens. |
| 2 | **Crypto 1% TDS (s.194S) not applied in `councileval`; locked-cash effect large; offshore applicability unresolved.** | `crates/councileval/src/netreturns.rs:9-10` ignores TDS by design. Live runner models it only with `--tds` (`crates/live/src/main.rs:225`, `crates/live/src/runner/mod.rs:55`). Thresholds ignored (`crates/costs/src/crypto.rs:70-72`). | 1% of consideration, thresholds Rs 10,000 / 50,000, credit against the 30% (`tax-india-crypto.md` rows 8-13). An offshore exchange may not deduct at all (row 12). | Not a P&L cost, a capital lock of 100 bps per sell. `reports/intraday-01.md:115-126` already shows 145 to 169% of starting capital withheld per year. | `result-council-v1-dev.md` crypto-burst (2,048 entries) ignores it: not feasible on an Indian exchange. Sign unchanged (-72% CAGR already). |
| 3 | **Equity STCG/LTCG rates, exemption and surcharge stale for 2024-07-23 onward.** | `crates/councileval/src/netreturns.rs:25-27`: `STCG_RATE 0.156`, `LTCG_RATE 0.104`, `LTCG_EXEMPT 100_000`. `crates/analogs/src/market.rs:40`: `tau 0.156`. Single constants, no date key. | 20% / 12.5% / Rs 1.25 lakh from 2024-07-23 (`tax-india-equity.md` s.2, confirmed in press). | +5.2 percentage points of tax on short-term gains (15.6% to 20.8% with cess). +2.6 points on long-term (10.4% to 13.0%). On a strategy earning 14% pre-tax that is about 0.7 CAGR points. | None (windows end 2021). Blocks any 2024+ run or holdout extension. |
| 4 | **Crypto: gains taxed after fees, but s.115BBH allows only cost of acquisition.** | `crates/councileval/src/netreturns.rs:127`: `f(t.net_pnl).max(0.0) * CRYPTO_TAX`; `net_pnl` is after fees. `crates/analogs/src/outcome.rs:102-130`: tau applied to the net outcome. (`crates/live/src/stats.rs:64-68` does it correctly from fills.) | Only cost of acquisition is deductible; no fees (`tax-india-crypto.md` row 2). | Tax understated by 31.2% x 20 bps = **6.2 bps per winning round trip** on top of the existing tax. | council-v1 crypto cell, `analogs` calibration. Both already negative. |
| 5 | **Crypto tax rate inconsistent across crates (30% vs 31.2%).** | `crates/live/src/main.rs:31`: `VDA_TAX_PCT = 30`. `crates/councileval/src/netreturns.rs:30` and `crates/analogs/src/market.rs:49`: 0.312. Carry result uses 30% with cess omitted (`docs/result-0b-basis-carry.md:99`). `reports/intraday-01.md:110` omits cess. | 30% + 4% cess (+ surcharge) = 31.2% minimum (`tax-india-crypto.md` row 1). | 4% relative error in tax, about 1 to 3 bps per trade on `intraday-01`; about 0.05 points on the carry F median. | `reports/intraday-01.md` after-tax table; `result-0b` F reading. No sign change. |
| 6 | **GST on crypto exchange fees = 0.** | `crates/costs/src/crypto.rs:64`: `gst_pct: ZERO`. `docs/costs-crypto.md` lists no GST. (The carry result does include 18% GST on fees.) | 18% on platform fees (row 21, confirmed). Whether an offshore venue charges Indian GST is contradictory in the sources (row 23). | +1.8 bps a side, **+3.6 bps per round trip** (fee 20 to 23.6 bps), if charged. That is 12% more cost on `intraday-01` runs (about -30 bps per trade). | `intraday-01`, council crypto-burst, `analogs`. Already negative. Carry (0b) already includes GST. |
| 7 | **Equity intraday: no tax at all (speculative business income at slab).** | `crates/live/src/main.rs:208` runs `zerodha_nse_intraday()` with no after-tax view. `after_tax_india` only exists for crypto (`main.rs:300`). | Speculative income, slab rate, loss set-off only against speculative gains, 4-year carry-forward (`tax-india-equity.md` s.3). | Up to 31.2% on winning days. The cost schedule itself (8.3 bps round trip on the fixture) matches the docs. | `reports/intraday-01.md` equity runs (lose before any cost, so no flip). |
| 8 | **Benchmark taxed without Section 112A grandfathering.** | `crates/councileval/src/netreturns.rs:151`: whole 5-year gain taxed at 10.4% above Rs 1 lakh. | Cost basis for assets bought before 2018-01-31 is stepped up to the higher of cost and the 2018-01-31 fair value (`tax-india-equity.md` s.2, marked "unverified, from knowledge"). | Hand estimate: about 70% of the 2015-2019 NIFTYBEES gain fell before the cut-off (Nifty 8,280 to 11,027 to about 12,170, from memory). Benchmark total return is understated by about 3 to 4 points of 49%; CAGR about +0.4 points on 8.54%. | council-v1 G4 "beats index". Index already beats every book. |
| 9 | **Capital-gains tax computed on a net-of-STT, net-of-fee gain.** | `crates/councileval/src/netreturns.rs:68-78` uses `t.net_pnl` (STT, stamp, DP deducted). | STT is not deductible against capital gains; brokerage and transfer expenses are (the docs do not state this; from general knowledge, unverified). | STT is 20 bps of buy turnover. Tax understated by 15.6% x 20 = about **3 bps per winning round trip**. | council-v1 equity cells. |
| 10 | **Equity loss handling is conservative.** | `crates/councileval/src/netreturns.rs:75-78`: no carry-forward, no short-term loss against long-term gain. | Short-term loss may offset short- or long-term gains and carries forward 8 years (`tax-india-equity.md` s.3, unverified). | Overstates tax for choppy years. Direction favours strategies if fixed; small because the books lose money. | council-v1 momentum (+1.20% CAGR, 22 entries). |
| 11 | **Stamp duty applied at the 2020 uniform rate to pre-2020 windows.** | `crates/costs/src/lib.rs:59`: 0.015% for all dates. | 0.015% only from 2020-07-01. Earlier rates were state-specific and lower (my recollection is about 0.002 to 0.003%; not in any research doc). | Overcharge of about 1.2 bps on the buy leg for 2012-2019 windows. Conservative. | `backtest-04`, council-v1 equity (2015-2019). Strategies look slightly worse than reality. |
| 12 | **Exchange charge: rate unverified; history not date-keyed.** | `crates/costs/src/lib.rs:57`: 0.00307%. Zerodha page; reported 0.00297% from 2024-10-01 (`tax-india-equity.md`, flagged unverified). The earlier 0.00325% is my recollection. | `tax-india-equity.md` s.1. BSE rate is group-dependent. | Under 0.05 bps a round trip. | None. |
| 13 | **Options exchange charge cut-over date and rate.** | `wt/options/crates/options/src/costs.rs:41-47`: 0.05% before 2024-01-01, 0.03553% after. | `tax-india-fno.md` s.1: 0.03503% effective 2024-10-01 (circular NSE/FA/64232), current page shows 0.03553%, conflict unresolved. | If the real cut-over is 2024-10-01, Jan to Sep 2024 is under-charged by about 1.5 bps of premium turnover per leg (inside the holdout). 0.03553 vs 0.03503 is 0.05 bps. | `prereg-options-v1`: no result yet. |
| 14 | **F&O tax assumptions stated more firmly than the F&O research doc.** | `wt/options/docs/costs-tax-options-india.md` s.4 and `wt/options/crates/options/src/tax.rs`: STT deductible, 31.2% flat, 8-year carry-forward, no surcharge. | `tax-india-fno.md` s.3: STT deductibility and fee deductibility "unsure/unverified"; s.5 loss carry-forward requires the return filed on time. | Deductibility of STT (about 6.8 of Rs 104.6 on the worked example): tax delta about 31.2% x 6.8 = Rs 2.1 per trade. Immaterial. | None yet. |
| 15 | **Carry result assumes the BNB discount; engine default is none.** | `docs/result-0b-basis-carry.md:100`: BNB discount assumed on spot. `crates/costs/src/crypto.rs:35`: default 0.10%. | `tax-india-crypto.md` does not cover fee tiers. | 25% of the 10 bps spot fee, 2.5 bps a side on the spot leg. | `result-0b`. Far below the kill line. |
| 16 | **Crypto futures tax head unresolved; carry uses two readings.** | `docs/result-0b-basis-carry.md:25-29` readings F (net, 30%) and A (profit leg only). | VDA vs business income is contested (`tax-india-crypto.md` row 17). | Not a rate error. A reading is -32.2% a year, F is +1.37%. | `result-0b`. |

Not mismatches (checked and consistent): equity delivery STT 0.1% both sides, intraday STT 0.025% sell only, stamp 0.015% / 0.003% buy only, SEBI Rs 10 per crore, GST 18% on brokerage + exchange + SEBI + DP and not on STT or stamp (`crates/costs/src/lib.rs:51-98`, `crates/costs/src/lib.rs:97`), DP Rs 15.34, options STT history by date (`wt/options/crates/options/src/costs.rs:27-38`), option stamp 0.003%, crypto loss no set-off modelled per trade (`crates/councileval/src/netreturns.rs:127`, `crates/live/src/stats.rs:64-68`).

## 3. Where the checks you listed landed

| Item asked | Result |
|---|---|
| Stale STT rates | Cash equity: not stale. F&O: no F&O schedule in the integration engine; options worktree is current. Exercise STT (0.15% of intrinsic) is not modelled, by design (positions exit before expiry). |
| Missing GST on fees | Equity: present and correct. Crypto spot fee: absent (#6). Options: present. |
| Missing 1% crypto TDS | Live runner: optional cash model. `councileval`: ignored. Offshore applicability: unresolved (#2). |
| Crypto loss no-setoff (s.115BBH) | Modelled per trade in `councileval`, `analogs` and `live`; fee-deductibility error (#4) and 30 vs 31.2 inconsistency (#5). |
| Equity STCG / LTCG rates | Stale for 2024-07-23 onward (#3); right for all published windows. |
| Stamp duty | Right today; pre-2020 overcharge (#11). |
| Exchange charges | Equity within 0.05 bps (#12); options cut-over date (#13). |

## 4. Why none of this flips a verdict

Pre-registered gate verdicts in `docs/result-council-v1-dev.md` are INCONCLUSIVE-SAMPLE for all four bases because G1 (100 independent trades) fails. No cost or tax change moves entry counts, so no gate that is currently FAIL on sample can become PASS.

## 5. Published results and what could change

| Result | Current verdict | Effect of corrections | Could the verdict or sign change? |
|---|---|---|---|
| `reports/backtest-04.md`: MomentumRotation 14.02% vs NIFTYBEES (cost-paying) 14.31%, HAC t -0.17; IndexTrendFilter 7.14% | Neither beats the index. | Add tax (#1): momentum worse relative to buy-and-hold by about 1 to 1.5 points a year. IndexTrendFilter trades about 21 switches in 10 years, so its tax gap is small. The t of -0.17 for momentum would move clearly negative. | Verdict unchanged; strengthened. It cannot flip because pre-tax it is already below the benchmark. This is the result most exposed, since it is silent on tax. |
| `docs/result-council-v1-dev.md` momentum-rotation: council 0.84% CAGR vs unfiltered 1.20% vs NIFTYBEES 8.54% | INCONCLUSIVE-SAMPLE | Benchmark understated by about 0.4 CAGR points (#8); stamp overcharge (#11) helps the strategies by under 1 bp a trade. | No. The strategy trails by about 7 points. |
| council-v1 donchian, trend-pullback | Unfiltered lose (-2.1%, -5.3% CAGR) | Tax understated by about 3 bps per winner (#9), stamp overcharge about +1.2 bps per buy. Net near zero. | No. |
| council-v1 crypto-burst: unfiltered -72.11%, council 0 entries | INCONCLUSIVE-SAMPLE | Fee non-deductibility (#4) +6.2 bps per winner, GST (#6) +3.6 bps per round trip, TDS (#2) makes it infeasible on an Indian exchange. All push worse. | No. Already -72%. The council book has 0 trades; a Sharpe of 0 against the "BTC buy and hold 1.756" is unchanged. |
| `reports/intraday-01.md` crypto scalpers (-30 bps a trade, after-tax -32 bps) | Fail | GST (#6) -3.6 bps, cess (#5) and fee non-deductibility (#4) a few bps each. | No. |
| `docs/result-0b-basis-carry.md`: F median +1.37% (mean -0.15%), A median -32.2%, kill line 7% | KILL | 31.2% not 30% and fees non-deductible (#4, #5) lower F by about 0.1 point; the BNB discount (#15) removes up to 0.1 point. | F median could drift toward 1.2%; still far below 7%. The mean is already negative. Verdict unchanged. |
| `docs/result-0a-sue-stratum-b.md` | Not examined in depth: equity pre-tax, same schedule. | Not expected to move. | Not expected. |
| `prereg-options-v1` | No result yet. | Fix #13 and the date-key before the scored run; confirm #14 with a CA. | n/a |

## 6. Proposed minimal fix (not applied)

Goal: move every rate behind a dated, source-tagged, `verified` config entry, and make unverified rates visible in every result file. No behaviour change on the published windows.

1. **One rates file**, e.g. `config/rates-india.toml`, loaded by `tq-costs` and `councileval`. Each rate carries `value`, `effective_from`, `source`, `verified` (default `false` for everything today, because all sources are secondary):

   ```toml
   [[equity.stt_delivery_pct]]
   effective_from = "2004-10-01"
   value = "0.1"
   source = "zerodha.com/charges; tax-india-equity.md s.1"
   verified = false

   [[equity.stcg_pct]]            # base rate, cess applied separately
   effective_from = "2018-04-01"
   value = "15"
   verified = false
   [[equity.stcg_pct]]
   effective_from = "2024-07-23"
   value = "20"
   source = "tax-india-equity.md s.2 (IT Dept FAQ quoted by press)"
   verified = false

   [[equity.ltcg_pct]]   ...  10 from 2018-04-01, 12.5 from 2024-07-23
   [[equity.ltcg_exempt_inr]] ... 100000 -> 125000 on 2024-07-23
   [[equity.stamp_buy_delivery_pct]] ... 0.015 from 2020-07-01 (earlier: state rates, unknown)
   [[equity.exchange_nse_pct]] ... 0.00307 current, verified = false

   [crypto]
   vda_tax_pct = { value = "30", verified = false }
   cess_pct = { value = "4", verified = false }
   tds_194s_pct = { value = "1", verified = false }
   gst_on_fees_pct = { value = "18", verified = false, applies = "indian_venue" }  # offshore: unresolved, row 23
   fees_deductible = { value = false, verified = false }
   ```

2. **Tax model enum** shared by `councileval`, `analogs` and `live` (replaces `STCG_RATE`, `LTCG_RATE`, `CRYPTO_TAX`, `tau`, `VDA_TAX_PCT`): `none`, `equity_cg` (date-keyed rates, optional grandfathering flag, optional short-term-loss carry-forward), `vda_30` (30% + cess, tax computed on the fee-free gain `(exit_fill - entry_fill) * qty`, as `live/src/stats.rs` already does), `business_slab` (scenario only). The default for old windows reproduces today's numbers exactly (regression test: council-v1 dev cells byte-identical).
3. **GST and TDS switches in `CryptoSpotFees`:** `gst_on_fees: bool` (default `false` offshore) and `tds: Option<Tds194S>` surfaced to `councileval` as a cash-lock metric (report peak locked cash and entries refused), not as P&L.
4. **Date-keyed options exchange charge** (`wt/options/crates/options/src/costs.rs:41-47`): cut-over to 2024-10-01 and 0.03503 vs 0.03553 as two named variants; keep the conservative one for the preregistered run, record the other as a sensitivity.
5. **Result-file banner:** every `result-*.md` and `reports/*.md` gets a line "rates unverified: N of M" derived from the `verified` flags, and the run refuses "profit claim" wording if any flag used is `false`. Add a `--require-verified` flag that fails the run until the primary-source lists in `tax-india-crypto.md` s.9 and `tax-india-equity.md` s.7 are done.
6. **Tests:** crypto +100 / -100 yields tax 31.2 and after-tax -31.2 (from `tax-india-crypto.md` s.10); a 2024-08-01 equity trade uses 20% STCG; a pre-2020-07-01 trade uses the older stamp rate; benchmark with and without grandfathering.
7. **Re-run, do not edit history:** add a tax stage to `backtest-04` as a new dated report (new file). The original `reports/backtest-04.md` stays as published.

## 7. Open items before any of this is trusted

- Read the primary sources listed in `tax-india-equity.md` s.7, `tax-india-fno.md` s.7 and `tax-india-crypto.md` s.9.
- Pre-2020 stamp duty, pre-2024 NSE equity and options exchange charges, grandfathering and short-term loss treatment are my recollection, not in the research docs; they need a source before they enter config.
- Whether Binance (offshore) deducts TDS or charges Indian GST decides items #2 and #6.
- The backtest-04 tax delta in #1 and the grandfathering delta in #8 are hand estimates and should be replaced by an actual run once the tax stage exists.

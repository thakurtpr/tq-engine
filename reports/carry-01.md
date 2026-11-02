# carry-01: funding carry (edge-hunt-01 candidate A)

Spec: docs/edge-hunt-01.md section 4 (pre-registered by thakur-op2). Code: research/carry_backtest.py. Data: Binance public archive, SHA-256 verified (scripts/fetch_binance_funding.py), 2020-01 to 2026-10, BTCUSDT and ETHUSDT. Costs: spot 7.5 bps, perp 5 bps taker, 1 bp slippage per leg. Hurdle 8%/yr. Tax cases are not advice. Independent replica: docs/review-carry-01.md (thakur-op2) reproduced every number of the first version.

**Verdict: A is closed. It fails pre-registered kill criteria 1 and 3 in all four runs.** The premium is real (HAC t 9-10, max DD under 2%, no liquidation) but has decayed: out-of-sample (2024-01 onward) is 3.6-4.7%/yr against an 8% hurdle. Raw funding on notional was 12-13% in 2024, about 5% in 2025 and 1.7-2.6% in 2026 to date, so A fails the hurdle even on a notional denominator. Under the s.115BBH worst cases, 2022 and 2024 go negative. HAC t is not evidence against the hurdle: for a low-variance carry, kills 1 and 3 are what bind.

Deviations from the spec (accepted in the review):
1. Liquidation and the funding mark use MARK-price klines. On 2020-03-13 02:00 the ETH perp's last-price high was 323.0 while mark peaked at 119.47; last price would have faked a liquidation.
2. **Un-pre-registered parameter:** a margin band [0.5m, 1.5m] replaces the "|delta| > 2%" trigger, which never fires with equal-quantity legs. The band lets the short leg drift to 2x leverage (4x at M = 0.5N) before resizing, so "1x" holds only at entry. Band sensitivity (review): OOS 3.1-3.9%, never near 8%. With no rebalancing, both symbols liquidate in 2020.
3. Tax (b) taxes spot gains annually as if realised. (b2) also taxes the short perp's price gain in down years; it is the stricter worst case.
4. Funding in hours missing from a kline file is accrued at the next simulated hour (36 BTC and 18 ETH payments; about 0.02%/yr).

E (dated cash-and-carry) was not run: 8% on capital needs about 16% basis on notional, against funding of 2.5-3.5% now.

### BTCUSDT, M = 1.0N
- liquidated: no; rebalances: 37
- full +6.41%/yr (month-block 95% CI +4.54% to +8.52%), HAC t 10.24, max DD -1.18%
- in-sample +8.41%/yr, out-of-sample +3.67%/yr (OOS/IS 0.44)
- funding positive in 77% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | tax (b2) | TDS (c) |
|---|---|---|---|---|---|---|
| 2020 | +9.24% | +9.91% | +98.71% | -23.34% | -23.34% | +9.14% |
| 2021 | +18.25% | +18.43% | +37.67% | +1.42% | +1.42% | +18.19% |
| 2022 | +2.18% | +2.26% | -46.32% | +1.50% | -12.39% | +2.18% |
| 2023 | +4.34% | +4.51% | +52.84% | -12.87% | -12.87% | +4.31% |
| 2024 | +6.62% | +6.81% | +55.68% | -12.13% | -12.13% | +6.55% |
| 2025 | +2.59% | +2.64% | +1.16% | +1.45% | +1.45% | +2.58% |
| 2026 | +1.23% | +1.35% | -1.10% | +0.82% | +0.49% | +1.17% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b/b2) negative in 2021 or 2024

### BTCUSDT, M = 0.5N
- liquidated: no; rebalances: 78
- full +8.17%/yr (month-block 95% CI +5.80% to +10.84%), HAC t 10.17, max DD -1.67%
- in-sample +10.66%/yr, out-of-sample +4.75%/yr (OOS/IS 0.45)
- funding positive in 77% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | tax (b2) | TDS (c) |
|---|---|---|---|---|---|---|
| 2020 | +12.08% | +12.98% | +119.98% | -27.81% | -27.81% | +11.96% |
| 2021 | +22.92% | +23.47% | +62.16% | -2.77% | -2.77% | +22.76% |
| 2022 | +2.70% | +2.86% | -61.93% | +1.84% | -16.73% | +2.69% |
| 2023 | +5.60% | +5.80% | +71.40% | -17.56% | -17.56% | +5.55% |
| 2024 | +8.57% | +8.77% | +68.74% | -14.68% | -14.68% | +8.48% |
| 2025 | +3.37% | +3.45% | +0.20% | +2.27% | +2.27% | +3.34% |
| 2026 | +1.61% | +1.80% | -0.76% | +1.07% | +0.84% | +1.53% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b/b2) negative in 2021 or 2024

### ETHUSDT, M = 1.0N
- liquidated: no; rebalances: 62
- full +7.43%/yr (month-block 95% CI +5.00% to +10.15%), HAC t 9.28, max DD -0.94%
- in-sample +10.24%/yr, out-of-sample +3.57%/yr (OOS/IS 0.35)
- funding positive in 74% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | tax (b2) | TDS (c) |
|---|---|---|---|---|---|---|
| 2020 | +15.37% | +16.44% | +128.96% | -28.25% | -28.25% | +15.22% |
| 2021 | +22.18% | +23.16% | +129.43% | -23.60% | -23.60% | +22.01% |
| 2022 | +0.27% | +0.51% | -38.51% | +0.12% | -11.42% | +0.23% |
| 2023 | +4.40% | +4.51% | +37.77% | -8.28% | -8.28% | +4.37% |
| 2024 | +7.06% | +7.13% | +26.26% | -2.96% | -2.96% | +7.03% |
| 2025 | +2.37% | +2.53% | +12.20% | -2.05% | -2.05% | +2.32% |
| 2026 | +0.76% | +0.90% | -8.36% | +0.49% | -2.01% | +0.71% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b/b2) negative in 2021 or 2024

### ETHUSDT, M = 0.5N
- liquidated: no; rebalances: 141
- full +9.43%/yr (month-block 95% CI +6.31% to +12.86%), HAC t 9.36, max DD -1.25%
- in-sample +12.92%/yr, out-of-sample +4.68%/yr (OOS/IS 0.36)
- funding positive in 74% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | tax (b2) | TDS (c) |
|---|---|---|---|---|---|---|
| 2020 | +19.73% | +21.03% | +160.95% | -34.86% | -34.86% | +19.53% |
| 2021 | +28.47% | +29.71% | +164.04% | -29.65% | -29.65% | +28.21% |
| 2022 | -0.00% | +0.68% | -37.89% | -0.21% | -11.55% | -0.15% |
| 2023 | +5.57% | +5.85% | +54.48% | -12.53% | -12.53% | +5.50% |
| 2024 | +9.27% | +9.48% | +39.09% | -5.30% | -5.30% | +9.20% |
| 2025 | +3.19% | +3.45% | +9.04% | -0.55% | -0.55% | +3.12% |
| 2026 | +0.94% | +1.22% | -7.98% | +0.58% | -1.81% | +0.85% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b/b2) negative in 2021 or 2024


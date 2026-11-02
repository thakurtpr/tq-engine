# carry-01: funding carry (edge-hunt-01 candidate A)

Spec: docs/edge-hunt-01.md section 4 (pre-registered by thakur-op2). Code: research/carry_backtest.py. Data: Binance public archive, SHA-256 verified (scripts/fetch_binance_funding.py), 2020-01 to 2026-10, BTCUSDT and ETHUSDT. Costs: spot 7.5 bps, perp 5 bps taker, 1 bp slippage per leg. Hurdle 8%/yr. Tax cases are not advice.

**Verdict: A fails the pre-registered kill criteria 1 and 3 in all four runs.** The premium is real (HAC t 9-10, max DD under 2%, no liquidation) but has decayed: out-of-sample (2024-01 onward) is 3.6-4.7%/yr against an 8% hurdle, 2025 earned 2.4-3.4%, 2026 to date 0.7-1.6%. Under the s.115BBH worst case the bull years go negative.

### BTCUSDT, M = 1.0N
- liquidated: no; rebalances: 37
- full +6.40%/yr (month-block 95% CI +4.55% to +8.51%), HAC t 10.23, max DD -1.18%
- in-sample +8.40%/yr, out-of-sample +3.66%/yr (OOS/IS 0.44)
- funding positive in 77% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | TDS (c) |
|---|---|---|---|---|---|
| 2020 | +9.20% | +9.87% | +98.69% | -23.36% | +9.10% |
| 2021 | +18.30% | +18.49% | +37.67% | +1.45% | +18.24% |
| 2022 | +2.15% | +2.23% | -46.32% | +1.48% | +2.15% |
| 2023 | +4.33% | +4.51% | +52.84% | -12.87% | +4.30% |
| 2024 | +6.62% | +6.81% | +55.68% | -12.13% | +6.55% |
| 2025 | +2.59% | +2.64% | +1.16% | +1.45% | +2.58% |
| 2026 | +1.20% | +1.32% | -1.10% | +0.80% | +1.14% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b) negative in 2021 or 2024

### BTCUSDT, M = 0.5N
- liquidated: no; rebalances: 78
- full +8.16%/yr (month-block 95% CI +5.80% to +10.83%), HAC t 10.18, max DD -1.67%
- in-sample +10.66%/yr, out-of-sample +4.74%/yr (OOS/IS 0.44)
- funding positive in 77% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | TDS (c) |
|---|---|---|---|---|---|
| 2020 | +12.02% | +12.93% | +119.93% | -27.83% | +11.90% |
| 2021 | +23.01% | +23.56% | +62.17% | -2.71% | +22.84% |
| 2022 | +2.67% | +2.84% | -61.93% | +1.82% | +2.66% |
| 2023 | +5.59% | +5.78% | +71.39% | -17.56% | +5.54% |
| 2024 | +8.57% | +8.77% | +68.74% | -14.68% | +8.48% |
| 2025 | +3.37% | +3.45% | +0.20% | +2.27% | +3.34% |
| 2026 | +1.57% | +1.76% | -0.76% | +1.04% | +1.49% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b) negative in 2021 or 2024

### ETHUSDT, M = 1.0N
- liquidated: no; rebalances: 62
- full +7.41%/yr (month-block 95% CI +5.00% to +10.13%), HAC t 9.27, max DD -0.94%
- in-sample +10.22%/yr, out-of-sample +3.57%/yr (OOS/IS 0.35)
- funding positive in 74% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | TDS (c) |
|---|---|---|---|---|---|
| 2020 | +15.31% | +16.38% | +128.92% | -28.28% | +15.17% |
| 2021 | +22.18% | +23.16% | +129.43% | -23.60% | +22.01% |
| 2022 | +0.27% | +0.51% | -38.51% | +0.12% | +0.23% |
| 2023 | +4.39% | +4.50% | +37.77% | -8.29% | +4.37% |
| 2024 | +7.06% | +7.13% | +26.26% | -2.96% | +7.03% |
| 2025 | +2.37% | +2.53% | +12.20% | -2.05% | +2.32% |
| 2026 | +0.74% | +0.88% | -8.36% | +0.48% | +0.69% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b) negative in 2021 or 2024

### ETHUSDT, M = 0.5N
- liquidated: no; rebalances: 141
- full +9.42%/yr (month-block 95% CI +6.31% to +12.83%), HAC t 9.36, max DD -1.25%
- in-sample +12.91%/yr, out-of-sample +4.67%/yr (OOS/IS 0.36)
- funding positive in 74% of last-12-month intervals

| year | pre-tax | funding | spot gain | tax (b) | TDS (c) |
|---|---|---|---|---|---|
| 2020 | +19.72% | +20.99% | +160.66% | -34.78% | +19.52% |
| 2021 | +28.47% | +29.71% | +164.04% | -29.65% | +28.22% |
| 2022 | -0.01% | +0.67% | -37.89% | -0.21% | -0.15% |
| 2023 | +5.56% | +5.84% | +54.47% | -12.53% | +5.49% |
| 2024 | +9.27% | +9.48% | +39.09% | -5.30% | +9.20% |
| 2025 | +3.19% | +3.45% | +9.04% | -0.55% | +3.12% |
| 2026 | +0.91% | +1.19% | -7.98% | +0.55% | +0.82% |

- kill criteria hit: 1 OOS below hurdle or <50% of IS, 3 tax case (b) negative in 2021 or 2024


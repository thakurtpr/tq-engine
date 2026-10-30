# Crypto spot costs and Indian TDS (as modelled in `tq-costs`)

Date: 2026-10-30. Arithmetic on published rules, **not tax advice**. See also
`docs/intraday-costs-and-access.md` (analyst research, with verification levels).

## Exchange fees: `CryptoSpotFees`

| | Default | Source |
| --- | --- | --- |
| Taker (market orders) | 0.10% of turnover | Binance spot VIP 0, no BNB discount (secondary sources; check <https://www.binance.com/en/fee/schedule>) |
| Maker (resting limit) | 0.10% of turnover | same |

Both are config. `CryptoSpotFees::schedule(Liquidity::Taker)` returns a `CostSchedule` with only
`brokerage_pct` set (no STT, stamp, GST or DP), so the existing `Paper` broker bills crypto fills
unchanged. The paper runner uses market orders, so it pays taker on both legs: **20 bps per round trip**
(15 bps with the 25% BNB discount: `--fee-pct 0.075`). Slippage is separate (runner default 5 bps per side).

## Indian resident: 1% TDS on sells (`Tds194S`, `TdsLedger`)

- Income-tax Act 1961 **s.194S**: 1% of the consideration is withheld on transfer of a virtual digital
  asset (in force since 2022-07-01). Secondary sources (taxguru, calcguru, patronaccounting; fetched
  2026-10-30, no CBDT text read) say the **rate stays 1%** under the Income-tax Act 2025 from 2026-04-01,
  renumbered as **s.393(1), Table Sl. No. 8(vi)**. Not independently verified against the Act.
- **Modelled as a cash-flow deduction, not a cost.** TDS is an advance tax payment credited against the
  year's tax liability and refundable if it exceeds it. The runner (`--tds`) removes 1% of each sell's
  proceeds from cash and books it in `TdsLedger::receivable()`. Equity = cash + positions + receivable, so
  P&L is unchanged; what changes is **usable cash**, which is what binds for a high-turnover scalper
  (the credit returns only after the return is filed and processed).
- The yearly exemption thresholds (Rs 50,000 specified persons / Rs 10,000 others) are ignored, which
  over-withholds slightly; an active trader crosses them within days.
- Who withholds on an offshore exchange is unclear (an Indian exchange deducts). The model withholds anyway.
- **Income tax is separate:** 30% flat on VDA gains (s.115BBH), cost of acquisition the only deduction, no
  set-off of losses. Not part of `tq-costs`; `reports/intraday-01.md` shows a simplified after-tax view.

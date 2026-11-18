# SEBI / NSE regulatory timeline relevant to options backtests (options-v2)

Compiled 2026-11-17. Paper/backtest only. Confidence: **P** = primary NSE/SEBI document read in this repo's sessions (see
`docs/tax-india-fno.md` section 8); **S** = two or more secondary sources (broker bulletins, press) agree; **1S** = one secondary
source; **D** = observed in the bhavcopy data itself (see `docs/data-inventory-options-v2.md`); **U** = from memory, unverified.
Nothing below was fetched from a source that required a login. Every date a backtest depends on must be re-derived from the
data (expiry dates, lot sizes: `docs/data/lot-sizes-options-v2.csv`, `docs/data/options-v2-expiry-structure.csv`), never from this table.

| Effective date | Change | Source / circular | Conf. | Backtest consequence |
|---|---|---|---|---|
| 2016-06-02 | BANKNIFTY weekly options: first weekly expiry 2016-06-02, first traded 2016-05-27 (the earlier note of 2016-10 was wrong) | bhavcopy data | D | weekly structures only from the first listing date seen in data |
| 2019-02-14 | NIFTY weekly options: first weekly expiry 2019-02-14, first traded 2019-02-11 | bhavcopy data | D | same |
| 2021-01-11 | FINNIFTY F&O first session in data; first weekly expiry 2021-01-14 | bhavcopy data | D | first session per underlying from data |
| 2022-01-24 | MIDCPNIFTY F&O first session in data (first weekly expiry 2022-02-01); NIFTYNXT50 first session 2024-04-24 | bhavcopy data | D | same |
| 2024-04-26 | Nifty 50 lot cut 50 -> 25 (data: most liquid future carries 25 from 2024-04-26; secondary sources cite an NSE circular dated 2024-04-02; circular not read) | bhavcopy implied lot (D); fyers / Angel One bulletins | D + 1S | lot series in `lot-sizes-options-v2.csv` |
| 2024-07-05 | NSE bhavcopy switches to UDiFF format (old-format URL 404 from 2024-07-08) | `docs/data-feasibility-options.md` probes | D | schema normalised; `lot_size` and `underlying_price` only after this date |
| 2024-10-01 | SEBI circular SEBI/HO/MRD/TPD-1/P/CIR/2024/132 "Measures to strengthen equity index derivatives framework": six measures | title/date/number read at sebi.gov.in per `docs/tax-india-fno.md`; body via press | P (existence) / S (content) | below |
| 2024-11-20 | Weekly expiry limited to ONE benchmark index per exchange (NSE kept NIFTY; BANKNIFTY, FINNIFTY, MIDCPNIFTY lost weeklies, NIFTYNXT50 had none) | same SEBI circular; Business Standard, Arihant | S + D | weekly BANKNIFTY/FINNIFTY/MIDCPNIFTY structures cannot be tested after the date; verify last weekly in data |
| 2024-11-20 | Minimum index derivative contract value Rs 15 lakh (lot sizes raised: NIFTY 25->75, BANKNIFTY 15->30, FINNIFTY 25->65, MIDCPNIFTY 50->120, NIFTYNXT50 10->25 for NEW contracts; existing contracts keep old lot to expiry; quarterly/half-yearly switch 2024-12-24 (BANKNIFTY) and 2024-12-26 (NIFTY)) | **NSE/FAOP/64625, Circular 128/2024, dated 2024-10-18, https://archives.nseindia.com/content/circulars/FAOP64625.pdf, read** (lots before: NIFTY 25, BANKNIFTY 15, FINNIFTY 25, MIDCPNIFTY 50, NIFTYNXT50 10) | P | lot size is per contract expiry, not per date: use lot_size column (new format) / OI-GCD |
| 2024-11-20 | Extra Extreme Loss Margin (ELM) +2% on short options on expiry day (some sources say other rate) | secondary only | 1S | expiry-day short-option margin is higher; rate unsure |
| 2025-02-10 (originally 2025-02-01) | Upfront collection of option premium from buyers; removal of calendar-spread margin benefit on expiry day. Press: NSE notice moved both from 1 Feb to 10 Feb 2025 | Business Standard 2025-01-09 (secondary); `docs/tax-india-fno.md` says 2025-02-01 | S (10 Feb) | date discrepancy: the earlier repo doc says 1 Feb; treat 2025-02-10 as more likely and re-verify. Affects margin/calendar structures only on expiry day |
| 2025-04-01 / 2025-10-01 | Intraday monitoring of position limits (index options): broker checks from April, regime from October | SEBI/HO/MRD/TPD-1/P/CIR/2025/79 dated 2025-05-29 (per `docs/tax-india-fno.md`) | 1S | not modelled; irrelevant at paper size |
| 2025-06-26 (data) | BANKNIFTY lot 30 -> 35 and MIDCPNIFTY 120 -> 140 appear in the new-format `lot_size` (first session of the most liquid future); NSE/FAOP/70616 confirms the pre-Oct-2025 lots were 35 and 140. A secondary source mentions 2025-04-25 for quarterlies. The circular for this revision was not found | D (+P for the values) | lots are per contract; read the column |
| 2025-08-29 / 09-01 | Expiry day: SEBI circular 2025-05-26 requires one expiry weekday per exchange (Tuesday or Thursday). NSE moves ALL its F&O expiries from Thursday to Tuesday for contracts expiring on or after 2025-09-01; BSE moved to Thursday | **NSE/FAOP/68747, Circular 111/2025, dated 2025-06-25 (partially modifies circular 68685 of 2025-06-23), https://archives.nseindia.com/content/circulars/FAOP68747.pdf, read**: existing contracts expiring on/before 2025-08-31 unchanged; new contracts expiring on/after 2025-09-01 on Tuesday from 2025-06-26 EOD; long-dated NIFTY/BANKNIFTY options from Sept 2025 realigned 2025-07-31 EOD | P + D (last Thursday expiry 2025-08-28; first Tuesday expiry 2025-09-02 NIFTY weekly, 2025-09-30 monthlies) | expiry weekday is a function of the date: derive from the expiry column |
| 2025-10-28 (EOD) | Index lots re-set: NIFTY 75->65, BANKNIFTY 35->30, FINNIFTY 65->60, MIDCPNIFTY 140->120, NIFTYNXT50 25 unchanged; first weekly NIFTY expiry on new lot 2026-01-06, first monthly 2026-01-27, quarterly/half-yearly from 2025-12-30 EOD | **NSE/FAOP/70616 dated 2025-10-03 (Circular 176/2025), https://archives.nseindia.com/content/circulars/FAOP70616.pdf, read** | P | the pre-values (75/35/65/140) come from the circular itself |
| 2026-03-01 | NSE transaction charges / IPFT split changed (total unchanged: options Rs 3,553 per crore of premium, futures Rs 183 per crore) | NSE/FA/73061 (2026-02-27), read | P | costs table |
| 2026-04-01 | STT: options sold 0.10% -> 0.15% of premium; options exercised 0.125% -> 0.15% of intrinsic; futures sell 0.02% -> 0.05% | NSE/FATAX/73524 (2026-03-31), read | P | any 2022-2026 backtest must date-key STT |
| 2026-05-01 (claimed) | Alleged SEBI circular 2026/034 (position limits, margin +200-300 bps, weekly removal) | multibagg.ai only; internally inconsistent | U (do not rely) | none |
| after 2026-07 | NSE expiry weekday after mid-2026 not re-checked beyond one secondary source | search 2026-11-17 | U | read the expiry column |

## Other contract-specification facts a backtest must handle (unverified unless marked D)

- Index options are European, cash-settled. Stock F&O moved to physical settlement in phases during 2018-2019 (U); a stock-option
  backtest must close before expiry or model delivery.
- Strike intervals vary by underlying and by date: derive from the strikes present, never assume.
- Stock-option lot sizes are revised by NSE periodically: see the lot CSV (derived from the data).
- Corporate actions (splits, bonuses, mergers such as HDFC into HDFCBANK in 2023) adjust strikes/lots; the scan flags what is visible.
- Trading holidays and special sessions (Saturday budget sessions, Muhurat) are in `tq-calendar`; special sessions appear as extra bhavcopy dates.
- Exchange margin (SPAN + exposure) is not in the bhavcopy and no free historical source exists.

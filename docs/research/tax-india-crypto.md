# India: crypto / VDA tax and compliance for a resident individual

Researched 2026-11-06. NOT tax or legal advice. Confirm with a CA before relying on any row marked single-source, unsure or unverified.

## Method and honest limits

Verification status (2026-11-07): limited primary pass, see section 11. Only Budget-2026 items (TDS criminal liability, Rs 200/Rs 50,000 penalties) were read from primary text (Finance Bill 2026 Memorandum, Bill stage). s.115BBH and s.194S text and CBDT Circular 13/2022 remain unread from official sites (403/404); only third-party reproductions were read.

- Primary sources (incometaxindia.gov.in, Budget docs, RBI, FIU-IND) were NOT fetched directly in this pass. Search and fetch returned mostly CA and crypto-vendor explainers. A row is "confirmed" only where many independent secondary sources agree AND it is long-settled law. Everything else is single-source, unsure or unverified.
- The Income-tax Act, 1961 was replaced by the Income-tax Act, 2025 effective 2026-04-01. Section numbers are given for both regimes where sources gave them; new-Act numbers are unverified against the Act text.
- Sources label the 2026 changes variously as Finance Bill 2026 (clause 87) or Finance Act 2026.
- See also docs/intraday-costs-and-access.md lines 78-85 (earlier 194S note, official pages were 403 there too).

## 1. Core income-tax rules

| # | Rule | Value | Effective | Source | Confidence |
|---|------|-------|-----------|--------|------------|
| 1 | Tax on VDA transfer income, s.115BBH (1961 Act) | Flat 30% + 4% cess (+ surcharge); no slab benefit | FY 2022-23 onward (1 Apr 2022) | https://www.gadhiaassociate.com/blogs/98/your-crypto-exchange-is-now-talking-to-the-income-tax-department-what-changes-from-april-2026 ; https://taxgarden.in/blog/crypto-vda-tax-india-ay-2026-27-schedule-vda-itr-guide | confirmed (many sources agree). Verification pass: incometaxindia.gov.in 403; Indian Kanoon page (third-party) shows s.115BBH as inserted by Finance Act 2022 w.e.f. 2023-04-01 with 30% flat rate, no deduction other than cost of acquisition, no loss set-off/carry-forward. Still not read from an official site, so no upgrade |
| 2 | Deductions | Only cost of acquisition; no brokerage, gas, exchange fees, interest or other expense | same | same | confirmed |
| 3 | Loss set-off | VDA loss cannot be set off against any other income | same | gadhiaassociate; https://whale-alert.io/stories/978301d668f6a4/India-retains-30-crypto-tax-and-1-TDS-in-Budget-2026-bars-deductions-adds-reporting-penalties-and-eases-TDS-criminal-liability-to-2-years | confirmed. Whether a loss on one VDA can offset a gain on another VDA is not stated in these sources; the common CA reading is NO (each transfer taxed separately). Mark that point unsure |
| 4 | Loss carry-forward | Not allowed | same | same | confirmed |
| 5 | Changed in Budget 2026-27? | No change to 30% rate, 1% TDS or thresholds. Verification pass: Finance Bill 2026 Memorandum text mentions VDA only for the s.476 TDS prosecution relaxation and s.446/509 reporting penalty; no 115BBH/194S rate proposal found (grep). Inference from absence, not statute | Feb 2026 | gadhiaassociate; https://www.patronaccounting.com/blog/crypto-vda-taxation-income-tax-act-2025-rules; whale-alert.io | single-source (several secondary agree, no Budget document fetched) |
| 6 | Section mapping, Income-tax Act 2025 | Charge and TDS renumbered (one source: TDS at s.393(1) table row 8(vi); charge in s.194); sources disagree | 2026-04-01 | patronaccounting | unverified |
| 7 | Undisclosed VDA income found in search/audit | 60% special rate (about 78% with surcharge+cess), s.158B-type | date not given | gadhiaassociate | single-source, unverified |

## 2. TDS s.194S (1% on transfer of VDA)

| # | Rule | Value | Effective | Source | Confidence |
|---|------|-------|-----------|--------|------------|
| 8 | Rate | 1% of consideration | 2022-07-01 | gadhiaassociate; https://www.legalserviceindia.com/Legal-Articles/crypto-taxation-india-vda-tax-30-percent-tds-guide/ ; verification pass: TaxGuru reproduction of CBDT Circular 13/2022 (22 June 2022) says deduct 1% https://taxguru.in/income-tax/guidelines-section-194s-tds-payment-virtual-digital-asset.html | confirmed (third-party reproduction of circular read; official copy not read) |
| 9 | Thresholds | Rs 10,000 per FY; Rs 50,000 for specified persons | 2022-07-01 | gadhiaassociate; same TaxGuru reproduction of Circular 13/2022 (Rs 50,000 where payer is a specified person, Rs 10,000 otherwise, counted from 2022-04-01) | confirmed (third-party reproduction) |
| 10 | Who deducts | Buyer/payer; on an Indian exchange the exchange deducts. CBDT Circular 13/2022 is cited by a CA site; circular not fetched | 2022 | secondary only | single-source |
| 11 | Credit | Advance tax, creditable against the 30% liability; refund only if excess | n/a | gadhiaassociate | confirmed |
| 12 | TDS on an offshore exchange | A foreign exchange has no Indian TAN and does not deduct; payer-side obligation unclear; one source says for crypto-for-crypto P2P "both sides may have TDS obligations" | n/a | search summaries | unsure |
| 13 | TDS on derivatives | KoinX: triggered on conversion to INR / transfer out, not on open positions. Another source: 1% applies on futures payouts. Conflict | n/a | https://www.koinx.com/tax-guides/tax-on-crypto-futures-trading-india | unsure (conflict) |
| 14 | TDS criminal liability eased (s.476, new Act): imprisonment up to 2 years or fine where tax exceeds Rs 50 lakh; up to 6 months or fine for Rs 10-50 lakh; fine only otherwise. Wholly in-kind VDA consideration excluded from criminal liability | Finance Bill 2026, effective 2026-04-01 | PRIMARY (Bill stage), read: Finance Bill 2026 Memorandum (https://www.caalley.com/budget2026/memo.pdf) | **verified at Bill stage** (enacted text not read) |

## 3. Swaps, perpetual futures and derivatives

| # | Rule | Value | Source | Confidence |
|---|------|-------|--------|------------|
| 15 | Crypto-to-crypto swap | Taxable transfer. Gain = INR value of coin received minus cost of acquisition of coin given. 1% TDS may be paid in kind on an Indian exchange | gadhiaassociate; koinx/cleartax | confirmed (long-standing position) |
| 16 | Stablecoin (USDT) trades | Treated like any VDA; each USDT-quoted trade is a transfer | CA explainers | confirmed |
| 17 | Perp futures / derivatives on an offshore exchange: head and rate | No statutory ruling found. View (a): VDA transfer, s.115BBH 30%, no loss set-off. View (b): speculative/non-speculative business income at slab rates with set-off and carry-forward (4 AY for speculative). KoinX presents (b) but admits it is "reasonable but not explicitly confirmed" and that the department has issued no specific guidance | https://www.koinx.com/tax-guides/tax-on-crypto-futures-trading-india ; https://www.caindelhiindia.com/blog/crypto-futures-taxation/ | unsure (contested) |
| 18 | Conservative modelling assumption | All realised perp PnL = VDA income, 30% on gains, losses unusable. Worst case; this is a modelling choice, not a rule | derived | n/a |
| 19 | Funding payments | Not addressed by any source found | n/a | unverified |
| 20 | Residents taxed on global income | Offshore exchange gains are taxable in India | general residency rule | confirmed |

## 4. GST on exchange fees

| # | Rule | Value | Effective | Source | Confidence |
|---|------|-------|-----------|--------|------------|
| 21 | GST on platform/service fees | 18% on the fee, not on trade value or profit | one source says 2025-07-07 | https://www.koinx.com/tax-guides/tax-on-crypto-futures-trading-india ; https://busy.in/gst-rates/cryptocurrency/ ; https://cleartax.in/s/gst-on-supply-of-crypto-digital-assets | confirmed for 18% on fees; the 7 Jul 2025 date is single-source |
| 22 | GST on swap principal | Not charged; fee only. One source says CBIC classification of the underlying is unsettled | n/a | cleartax; legalsuvidha | unsure |
| 23 | Does an offshore exchange charge Indian GST? | KoinX key-takeaways say offshore platform fees; its FAQ says fees by exchanges in India. Self-contradictory | 2025 | koinx | unsure |

## 5. Reporting

| # | Rule | Value | Source | Confidence |
|---|------|-------|--------|------------|
| 24 | Schedule VDA in ITR-2 / ITR-3, AY 2026-27 (FY 2025-26) | Required: dates of acquisition and transfer, consideration, cost of acquisition, income, TDS | https://taxgarden.in/blog/crypto-vda-tax-india-ay-2026-27-schedule-vda-itr-guide ; https://www.koinx.com/in/tax-guides/file-crypto-taxes-on-schedule-vda | confirmed (exists since AY 2023-24) |
| 25 | Which ITR | ITR-2 without business income; ITR-3 if business income (relevant only if derivatives are business income) | same | single-source |
| 26 | Exchange-to-department reporting | From 2026-04-01 Indian exchanges send user-level data directly to the department (s.509 Income-tax Act 2025) | gadhiaassociate; patronaccounting | single-source (two secondary) |
| 27 | Penalties on reporting entities | Rs 200 per day for not filing the statement; Rs 50,000 for furnishing inaccurate particulars and failing to correct (amend s.446, Clause 87, effective 2026-04-01; s.509 is the reporting obligation); aimed at entities, not individuals. Rs 1 lakh figure NOT found. Verified at Bill stage in the Finance Bill 2026 Memorandum (https://www.caalley.com/budget2026/memo.pdf) | https://coinpaprika.com/news/india-cross-border-crypto-data-2027-budget-2026-fines/ ; https://signpostnews.com/rs-200-per-day-fine-india-cracks-down-on-missing-crypto-reports-starting-2026 | **verified at Bill stage (primary read)**; enacted text not read |
| 28 | Individual under-reporting penalty | General s.270A: 50%-200% of tax; one guide applies it to offshore crypto gains. New-Act section unknown | search summary | unsure |
| 29 | CARF | India to share cross-border crypto data from April 2027 | coinpaprika (above) | single-source |
| 30 | Schedule FA for foreign exchange accounts/coins | Possibly required; no source confirmed | n/a | unverified |

## 6. FEMA / LRS and offshore exchanges

| # | Rule | Value | Source | Confidence |
|---|------|-------|--------|------------|
| 31 | LRS limit | USD 250,000 per FY per resident individual | RBI LRS Master Direction; many guides | confirmed (long-standing; RBI page not fetched) |
| 32 | Crypto via LRS | Banks reportedly require a declaration that funds will not buy crypto (ICICI reported); LRS excludes margin trading and speculative forex | https://inc42.com/?p=256492 | single-source |
| 33 | TCS on LRS | 20% for "other purposes"; threshold Rs 10 lakh per FY after Budget 2025; sources conflict on 2026 changes | https://businesstoday.in/amp/personal-finance/tax/story/budget-2025-tcs-threshold-for-foreign-remittances-hiked-to-rs10-lakh-what-it-means-for-you-462983-2025-02-01 | single-source for Rs 10 lakh; 2026 unverified |
| 34 | Funding an offshore exchange | Usually via P2P or stablecoins bought on Indian exchanges; RBI has issued no crypto-specific FEMA permission. One low-quality source says FEMA penalty can reach 3x the sum involved | search summary | unsure / unverified |
| 35 | RBI banking ban | Struck down by the Supreme Court in 2020; no ban in force | from memory, not re-fetched | confirmed (memory) |

## 7. Binance access

| # | Fact | Source | Confidence |
|---|------|--------|------------|
| 36 | Dec 2023: FIU-IND show-cause notices to 9 offshore exchanges (Binance, KuCoin, Huobi, Kraken, Bitfinex, Bittrex, Bitstamp, MEXC, Gate.io) for operating without PMLA registration; MeitY asked to block URLs; blocks from Jan 2024 | https://www.businesstoday.in/latest/corporate/story/operating-illegally-indias-financial-intelligence-unit-asks-it-ministry-to-block-urls-of-binance-8-others-411198-2023-12-28 | confirmed |
| 37 | Binance registered with FIU-IND and resumed Indian operations about Aug 2024 (one source says May 2024); penalty reported about Rs 18.82 crore, unconfirmed | https://cointelegraph.com/news/binance-india-relaunch-fiu-registration ; https://www.medianama.com/2024/08/223-binance-registers-with-the-finacial-intelligence-unit-to-end-hiatus-from-the-indian-crypto-market/ | single-source, details unsure |
| 38 | Status Nov 2026: crypto is not banned and not legal tender; trading is legal, taxed and PMLA-regulated. Whether Binance's India-facing service offers perpetual futures to residents is NOT verified (Indian-facing venues generally spot only). Accessing offshore derivatives via VPN would breach exchange ToS and possibly FEMA/PMLA | none authoritative | unverified |
| 39 | Third-party "FIU registered exchanges 2026" lists | cryptowire.in, bestcryptoindia.com | unverified, low authority |

## 8. 2025-2026 changes

- Budget 2025: undisclosed-income (s.158B) provisions extended to crypto; TCS on LRS threshold raised to Rs 10 lakh. Not re-fetched: unsure.
- Budget 2026-27: 30%/1%/thresholds unchanged; reporting-entity penalties added; CARF exchange from April 2027; TDS criminal liability eased (single-source).
- Income-tax Act 2025 in force 2026-04-01, renumbered sections.
- Pre-Budget talk of cutting the 30% rate did not materialise per sources.

## 9. Primary sources still to pull

1. incometaxindia.gov.in: s.115BBH and s.194S text; CBDT Circulars 13 and 14 of 2022; Income-tax Act 2025; ITR-2/3 Schedule VDA instructions AY 2026-27.
2. indiabudget.gov.in: Finance Bill 2026 clause 87 and memorandum.
3. RBI: LRS Master Direction; any crypto statements 2025-26.
4. fiuindia.gov.in: registered VASP list and blocked entities.
5. CBIC: GST notification or clarification on VDA fees.

## 10. Implications for the paper-trading cost model

Current assumption: 1% TDS is not modelled. Recommendation:

1. On an offshore perp venue no TDS is deducted, so the real exposure is the 30% (+4% cess) tax on gross realised gains with no loss offset (worst case, row 18). That matters far more than 1% TDS, which is a cash-flow credit against the same liability, not an extra cost.
2. Add a `tax_model` config with modes:
   - `none` (current): label results as pre-tax.
   - `vda_30` (conservative default): per FY, tax = 30% x 1.04 x sum of positive per-trade (or per-closed-position) gains, with no netting of losers, fees not deductible. Report after-tax PnL, effective tax rate, and after-tax Sharpe and breakeven.
   - `business_slab`: scenario only, if a CA confirms derivatives are business income (netting and fee deduction allowed); needs a slab parameter.
3. Optional `tds_194s_pct = 1.0` for Indian-exchange spot scenarios: model as locked cash (1% of sell consideration above the Rs 10,000/yr threshold), a capital-drag line, not an expense.
4. Optional `gst_on_fees` flag (fees x 1.18), default off for offshore until row 23 is resolved.
5. Add a test: trades +100 and -100 (net 0) produce tax 31.2 under `vda_30`, so after-tax PnL is -31.2. Gate decisions should require edge to beat 30% of gross winners, not net PnL.
6. Keep all rates in config with `verified: false` metadata until section 9 is done.

## 11. Verification log (2026-11-07, primary-source pass)

Read (text-extracted): Finance Bill 2026 direct-tax Memorandum, copy at https://www.caalley.com/budget2026/memo.pdf (third-party host of the official memorandum; official indiabudget.gov.in not fetched). Verified at Bill stage: s.476 criminal-liability relaxation for VDA TDS (rows 14), Rs 200/day and Rs 50,000 reporting penalties via s.446 (row 27), effective 2026-04-01. Memo contains no 115BBH/194S rate change.

Read but third-party only: Indian Kanoon page for s.115BBH (https://indiankanoon.org/doc/4837707/) and TaxGuru reproduction of CBDT Circular 13/2022. Consistent with rows 1-4, 8-9 but not upgraded.

Failed: incometaxindia.gov.in (403 on section-115bbh pages), indiacode.nic.in (403), caalley circular PDF (404), Wayback (blocked by tool). Not verified: s.115BBH/194S official text, Income-tax Act 2025 section mapping (TDS s.393 row 8(vi) claim, s.509 appears in Memorandum as the reporting obligation, which supports it), LRS/TCS, FIU, GST. Funding-rate and perp-derivative characterisation remain unsure (no authoritative ruling exists as far as found).

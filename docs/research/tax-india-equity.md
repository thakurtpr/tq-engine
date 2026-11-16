# India tax and transaction charges: listed equity (delivery and intraday), resident individual

Research date: 2026-11-06. Reference information only, not tax advice. Verify against primary sources before coding any rate into the engine.

Confidence key: **confirmed** = official or an Income Tax Dept FAQ quoted by press, or 3+ consistent independent sources; **single-source** = one source or only broker/press sources; **unsure/unverified** = conflicting or not retrieved.

Verification status (2026-11-07): NSE cash transaction charge, equity STT (unchanged) and the surcharge cap are now verified from primary text; see section 8. Method note below is from the original pass: the NSE circular PDF (FA73061) and the NSE charges page timed out on fetch, and no incometaxindia.gov.in page was retrieved directly. Most values below therefore come from broker pages and press, so "confirmed" is only used where sources agree and the figure is long-standing statute. Re-check every row marked single-source or unverified.

Recent-change flag: **[CHANGED <2y]** = changed on or after 2024-11-06. **[CHANGED 2024]** = older than 2 years but recent.

## 1. Transaction charges (per trade)

| Item | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| STT, delivery | 0.1% on buy AND sell value | long-standing; no equity-delivery change found in Budget 2026 | https://zerodha.com/charges ; https://www.angelone.in/news/taxation/union-budget-2026-securities-transaction-tax-stt-raised-on-futures-and-options (article lists only F&O changes) | **verified unchanged (primary read)**: NSE/FATAX/73524 dated 2026-03-31 table items 1-2, 0.1% purchaser and seller, "No Change" from 2026-04-01. https://archives.nseindia.com/content/circulars/FATAX73524.pdf |
| STT, intraday | 0.025% on sell side only | long-standing | NSE/FATAX/73524 table item 3 (sale settled otherwise than by delivery): 0.025%, seller, No Change. https://archives.nseindia.com/content/circulars/FATAX73524.pdf | **verified unchanged (primary read)** |
| Stamp duty, delivery | 0.015% (Rs 1,500/crore), buy side only | 2020-07-01 (uniform national rate) | https://zerodha.com/marketintel/bulletin/259741/uniform-stamp-duty-applicable-from-july-1-2020 ; https://www.tradejini.com/blogs/one-stamp-duty-unified-stamp-duty-effective-01st-july-2020 | confirmed (multiple brokers agree; no official notification retrieved) |
| Stamp duty, intraday | 0.003% (Rs 300/crore), buy side only | 2020-07-01 | same as above | confirmed (same caveat) |
| NSE transaction charge, equity cash | Rs 306.99 per crore + Rs 0.01 per crore IPFT = Rs 307 per crore total = 0.00307%, each side. Before 2026-03-01: Rs 297 + Rs 10 IPFT = Rs 307 (total outflow unchanged; only the split moved). Earlier 0.00297% figure was the transaction-charge-only part | 2026-03-01 (NSE/FA/73061, dated 2026-02-27) | https://nsearchives.nseindia.com/content/circulars/FA73061.pdf ; https://support.zerodha.com/category/account-opening/resident-individual/ri-charges/articles/exchange-transaction-charges ; https://zerodha.com/marketintel/bulletin/391488/revision-in-transactions-charges-from-1st-october-2024 | **verified (primary read)**: https://archives.nseindia.com/content/circulars/FA73061.pdf (the nsearchives host timed out; archives mirror worked). **[CHANGED <2y]** |
| BSE transaction charge, equity | 0.00375% (Rs 3.75/lakh) for Group A/B and non-exclusive scrips only; other groups differ | date unverified | https://support.zerodha.com/category/account-opening/resident-individual/ri-charges/articles/exchange-transaction-charges | single-source; other groups unverified |
| SEBI turnover fee | Rs 10 per crore (0.0001%) | long-standing | https://zerodha.com/charges ; https://hdfcsky.com/pricing/other-trading-charges | single-source on the broker pages (two brokers agree) |
| GST | 18% on brokerage + exchange transaction charges + SEBI fee. NOT on STT or stamp duty | long-standing (18% since 2017-07) | https://zerodha.com/charges | single-source (broker); consistent with long-standing practice |
| DP charges (on sell of delivery, per scrip per day) | Broker-specific. Zerodha example: Rs 15.34 = Rs 3.5 CDSL + Rs 9.5 broker + Rs 2.34 GST. Only the CDSL component is depository-set, and it can differ for NSDL | varies | https://zerodha.com/charges | single-source; treat as a configurable per-broker constant, not a statutory rate |
| Brokerage | Broker-specific (not statutory) | n/a | n/a | n/a |

Notes for the engine:
- Exchange transaction charge, SEBI fee and stamp duty are rounded per exchange/broker conventions. Rounding rules were not verified (a TradingQnA post cites NSE circular 43115 for stamp duty rounding; text not retrieved).
- Exchange charges are revised periodically (2024-10-01 and 2026-03-01 revisions seen). Make them config, not constants.
- F&O STT was raised in Budget 2026 (futures 0.02% to 0.05%; options premium 0.10% to 0.15%; options exercise 0.125% to 0.15%; per https://www.angelone.in/news/taxation/union-budget-2026-securities-transaction-tax-stt-raised-on-futures-and-options and https://navia.co.in/blog/union-budget-2026-stt-hike/, effective 2026-04-01 per Navia only). Out of scope for equity cash, listed for context. **[CHANGED <2y]**, effective date single-source.

## 2. Capital gains (delivery, treated as investment)

| Item | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| Long-term holding period, listed equity | More than 12 months | since 2018; reaffirmed 2024-07-23 | https://www.businesstoday.in/personal-finance/tax/story/budget-2024-income-tax-dept-issues-faqs-on-new-capital-gains-tax-regime-check-details-438651-2024-07-24 (quotes Income Tax Dept FAQs: 1 year for listed securities) | confirmed |
| STCG rate, STT-paid listed equity (s.111A under the 1961 Act) | 20% (plus surcharge and 4% cess) | transfers on/after 2024-07-23 (was 15%) | same Business Today / IT Dept FAQ page | confirmed. **[CHANGED 2024]** |
| LTCG rate, STT-paid listed equity (s.112A) | 12.5%, no indexation (was 10%) | transfers on/after 2024-07-23 | same | confirmed. **[CHANGED 2024]** |
| LTCG exemption | Rs 1.25 lakh per financial year on s.112A gains (was Rs 1 lakh), aggregate across equity and equity-oriented MFs | FY 2024-25 onward | same | confirmed |
| Budget 2026 changes to these rates | None found. Multiple outlets say LTCG 12.5% unchanged for FY 2026-27, exemption unchanged | Budget 2026-02-01 | https://www.angelone.in/news/taxation/union-budget-2026-ltcg-tax-unchanged-at-12-5-stt-on-commodity-futures-hiked ; https://www.business-standard.com/budget/news/budget-2026-ltcg-tax-rates-unchanged-for-fy27-fm-sitharaman-126020100424_1.html | Partly verified: the Finance Bill 2026 Memorandum has no proposal changing s.111A/112A rates (it mentions them only in surcharge provisions), consistent with no change. Absence in a memo is inference, not statute text, so keep **single-source/inferred**; the 12.5%/20%/Rs 1.25 lakh values still rest on the IT Dept FAQ via press |
| Buyback proceeds | Treated as capital gains instead of dividend; promoters: effective liability 30% (individual promoters), 22% (promoter companies), via additional tax. Effective 2026-04-01, tax year 2026-27 on | 2026-04-01 per Finance Bill 2026 Memorandum (Bill-stage) | https://www.business-standard.com/budget/news/budget-2026-buyback-proceeds-to-be-treated-as-capital-gains-126020101050_1.html ; https://www.wrightresearch.in/blog/buyback-taxation-in-india-2026-budget-changes-old-vs-new-rules-examples-and-benefits/ | Memorandum read (https://www.caalley.com/budget2026/memo.pdf), so rates/date **verified at Bill stage**; enacted text not read. **[CHANGED <2y]** |
| Surcharge/cess on capital gains | 4% cess applies. Surcharge on 111A/112A (and dividend) income capped at 15% | AY 2026-27 / FY 2026-27 rates | Finance Bill 2026 Memorandum, rates-of-tax para 3.1 ("rate of surcharge ... shall not exceed 15%"; no change proposed): copy at https://www.caalley.com/budget2026/memo.pdf | **verified in Bill-stage Memorandum (primary text, third-party host)**; cess 4% not restated there |
| Grandfathering of pre-31-Jan-2018 cost | Applies to s.112A (fair market value rule, from knowledge, not re-verified this session) | 2018 | not retrieved | unverified |

STT condition: s.111A/112A rates apply when STT was paid on the sale (and on the buy for LTCG, with exceptions). Exact condition wording not retrieved.

## 3. Intraday: speculative business income

| Item | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| Characterisation | Equity intraday (no delivery) is speculative business income (s.43(5) of the 1961 Act), taxed at normal slab rates, not capital gains | long-standing | https://taxguru.in/?p=1056526 ; https://www.angelone.in/knowledge-center/income-tax/intraday-trading-tax-audit | single-source (secondary); statute from knowledge |
| Loss set-off | Speculative loss can be set off only against speculative profit | long-standing | https://taxguru.in/?p=1056526 ; https://medium.com/nitinjogad/tax-treatment-on-loss-from-speculative-and-non-speculative-business-bb996c40198e | single-source |
| Loss carry-forward | 4 assessment years, speculative income only; requires the return filed by the due date | long-standing | same | single-source |
| Capital loss treatment (delivery) | STCL set off against STCG or LTCG; LTCL only against LTCG; carry forward 8 years (from knowledge; not verified this session) | long-standing | not retrieved | unverified |
| Non-speculative business loss (e.g. F&O) | Carry forward 8 years against business income | long-standing | https://taxguru.in/?p=1056526 | single-source (out of scope for cash equity) |
| Delivery trades as business vs investment | Depends on intent and facts; some filers treat delivery as business income (non-speculative). Not for the engine to decide | n/a | https://www.casahuja.com/2025/08/guidance-note-correct-business-codes.html | single-source |

## 4. ITR form, audit, due dates

| Item | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| ITR form: delivery capital gains only | ITR-2 (no business income) | AY 2026-27 | https://www.casahuja.com/2025/08/guidance-note-correct-business-codes.html | single-source |
| ITR form: intraday (speculative) income | ITR-3 (business income); presumptive s.44AD not available for speculative equity | AY 2026-27 | https://taxguru.in/?p=1056526 ; https://calcguru.in/?p=1911 | single-source |
| Tax audit (s.44AB) turnover threshold | Rs 1 crore baseline, Rs 10 crore if cash receipts and payments each within 5% of total. Exchange trades settle digitally, so Rs 10 crore is the working threshold per one CA article. Intraday turnover = sum of absolute profits and losses (ICAI Guidance Note, revised 2022). Sources conflict (1 crore vs 10 crore), and the Rs 10 crore condition must be met | current | https://taxguru.in/?p=1056526 (2026-07-08, cites s.44AB and ICAI GN) | **unsure** (secondary only; check s.44AB text and ICAI guidance) |
| Audit trigger if 44AD opted out | s.44AD(4)/44AB(e): 5-year lock and audit exposure after opting out of presumptive scheme (F&O context) | current | https://taxguru.in/?p=1056526 | single-source |
| Due dates AY 2026-27 | ITR-1/ITR-2: 31 July 2026. Non-audit ITR-3/ITR-4: 31 August 2026 (reported as moved from 31 July, said to be permanent from AY 2026-27 via Finance Act 2026). Audit cases: 31 October 2026 vs 21 Nov 2026 (reports conflict). Finance Bill 2026 Memorandum (Bill stage) proposes non-audit business filers 31 Aug (from 31 July), ITR-1/2 stay 31 July, audit cases 31 Oct | AY 2026-27 | https://www.jmfinancialservices.in/blogs-and-articles/itr-3-and-itr-4-filing-deadline-extended-to-31st-august-2026 ; https://www.businesstoday.in/personal-finance/tax/story/itr-due-dates-revised-returns-and-audit-penalties-3-big-tax-changes-from-ay-2026-27-537295-2026-06-17 ; https://studycafe.in/cbdt-extends-due-dates-for-audit-itr-and-tar-filing-for-ay-2026-27-427200.html | single-source; audit-case date **unverified**. **[CHANGED <2y]** |

## 5. Structural change: new Income-tax Act, 2025

- Income-tax Act, 2025 (Act 30 of 2025) replaced the 1961 Act from **2026-04-01**. Income from 2026-27 onward is governed by the new Act, using a single "Tax Year" concept. For AY 2026-27 (income of FY 2025-26) the 1961 Act and existing ITR forms still apply. Sources: https://taxguru.in/income-tax/income-tax-act-2025-force-1st-april-2026.html ; https://www.freepressjournal.in/business/new-income-tax-act-replaces-1961-law-from-april-1-2026 . Confidence: single-source (press/CA sites); CBDT release not read. **[CHANGED <2y]**
- Section numbers cited in this doc (43(5), 73, 111A, 112A, 44AB, 44AD) are 1961-Act numbers. Their new-Act equivalents were NOT verified. Rates and slabs are reported unchanged by sources. Any engine labels should be keyed by concept, not section number, until mapped.

## 6. What changed in the last 2 years (summary)

1. 2024-07-23: STCG 15% to 20%; LTCG 10% to 12.5%; exemption Rs 1 lakh to Rs 1.25 lakh (2y3m ago, flagged for context).
2. 2024-10-01: NSE/BSE transaction-charge revision (cash 0.00297% on NSE per brokers).
3. 2026-02-01 Budget: F&O STT hike (futures, options); buyback taxation as capital gains; equity capital gains rates unchanged (per press).
4. 2026-03-01: NSE transaction-charge revision per circular NSE/FA/73061 (rate not retrieved; broker now shows 0.00307%).
5. 2026-04-01: Income-tax Act 2025 in force; F&O STT effective date per one source.
6. AY 2026-27: non-audit ITR-3/ITR-4 due date 31 August (press-reported).

## 7. Open items to verify against primary sources

- NSE circular NSE/FA/73061 rate table (CM segment) and BSE equity transaction fee schedule by group.
- Finance Act 2026 text: confirm no change to STT on equity delivery/intraday, and to s.111A/112A.
- s.44AB threshold wording for speculative turnover and the ICAI Guidance Note on turnover.
- Income-tax Act 2025 section mapping for capital gains and speculative business provisions; any new ITR form numbers for Tax Year 2026-27.
- Official notification for stamp duty (Ministry of Finance, 2020) and NSE stamp-duty rounding circular.
- Surcharge cap on equity capital gains and grandfathering rule.
- Audit-case ITR due date for AY 2026-27 (31 Oct vs 21 Nov 2026 conflict).

## 8. Verification log (2026-11-07, primary-source pass)

Primary texts read (text-extracted locally):
- NSE/FA/73061, 2026-02-27, Revision in Transaction Charges: https://archives.nseindia.com/content/circulars/FA73061.pdf. CM Rs 306.99 + Rs 0.01 IPFT = Rs 307 per crore each side, effective 2026-03-01. Verified.
- NSE/FATAX/73524, 2026-03-31, STT rates: https://archives.nseindia.com/content/circulars/FATAX73524.pdf. Delivery 0.1% (both), intraday 0.025% (sell): "No Change" from 2026-04-01. Verified. Also records Finance Act 2026 assent on 2026-03-30.
- Finance Bill 2026 direct-tax Memorandum (copy: https://www.caalley.com/budget2026/memo.pdf). Used for surcharge cap 15%, buyback, due-date proposal. Bill-stage, third-party host; official indiabudget.gov.in copy not fetched.

Not verified:
- BSE equity transaction charge by group (no BSE circular found).
- s.111A/112A rates, Rs 1.25 lakh exemption, holding period, grandfathering: statute/IT Dept FAQ not read this pass (incometaxindia.gov.in and indiacode return 403).
- s.44AB thresholds and ICAI Guidance Note turnover rule: not read. Memorandum does not restate them (it converts the failure-to-audit penalty into a graded fee of Rs 75,000/Rs 1,50,000 under proposed s.428(c), Bill stage).
- Stamp duty, SEBI turnover fee, GST 18%, DP charges: not re-checked.
- Audit-case due date (31 Oct vs 21 Nov) and enactment of the 31 Aug change: not checked against a CBDT notification.

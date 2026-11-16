# India tax and charges: equity/index F&O (NSE/BSE), resident individual

Research date: 2026-11-06. Docs only, not tax advice. Confirm with a CA before filing.

Verification status (2026-11-07): STT rates, NSE transaction charges and index lot sizes are now verified from primary NSE circulars; see section 8. Other rows remain as sourced below.

Honesty note on sourcing (original pass): official pages (incometaxindia.gov.in, sebi.gov.in circular body, NSE circular PDFs, Budget speech) could not be read in this session (403 or body not rendered). Most values below come from broker and CA explainers, cross-checked across 2+ sources where noted. "confirmed" means multiple independent secondary sources agree (and, where stated, a regulator page confirms title/date/number); it does NOT mean the statute text was read. Anything not verified is marked **unverified**.

Confidence legend: confirmed / single-source / unsure.

## 1. Transaction taxes and charges (per side as noted)

| Item | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| STT, futures (sell side, on traded price) | 0.05% (was 0.02%) | 2026-04-01 (Finance Act 2026, assent 2026-03-30) | PRIMARY, read: NSE circular NSE/FATAX/73524 dated 2026-03-31 https://archives.nseindia.com/content/circulars/FATAX73524.pdf (item 4(c): 0.05% from 2026-04-01, 0.02% up to 2026-03-31, payable by seller). Also Finance Bill 2026 Memorandum, Clause 143 (see log) | **verified (primary read)**. Enacted Act text itself not read; NSE circular quotes the Act |
| STT, options sold (sell side, on premium) | 0.15% (was 0.10%) | 2026-04-01 | same NSE/FATAX/73524 item 4(a), seller | **verified (primary read)** |
| STT, options exercised (buyer, on intrinsic/settlement value) | 0.15% (was 0.125%) | 2026-04-01 | same NSE/FATAX/73524 item 4(b), payable by purchaser. Old rate 0.125% confirmed (ICICI "0.13%" was rounding) | **verified (primary read)** |
| STT, previous step (0.0125%/0.0625% -> 0.02%/0.10%) | Budget July 2024 | 2024-10-01 | Zerodha bulletin https://zerodha.com/marketintel/bulletin/391488/revision-in-transactions-charges-from-1st-october-2024 (0.02%/0.1% seen); old 0.0125%/0.0625% and exact date from memory | single-source / unsure on date |
| STT on option BUY (not exercised) | none | long-standing | Zerodha charges page | confirmed |
| Cash-equity delivery/intraday STT | unchanged by 2026 hike | n/a | sansalegal link above | confirmed |
| Stamp duty, futures (buy side) | 0.002% (Rs 200/crore) | since 2020-07-01 (uniform stamp duty); date from memory | https://zerodha.com/charges | confirmed value, date unverified |
| Stamp duty, options (buy side, on premium) | 0.003% (Rs 300/crore) | same | https://zerodha.com/charges | confirmed value (search snippet cited 0.003% for F&O generally; Zerodha page splits 0.002/0.003) |
| SEBI turnover fee | Rs 10 per crore of turnover (0.0001%) | long-standing | https://zerodha.com/charges | single-source (broker page) |
| NSE transaction charge, equity futures | Rs 182.99 per crore + Rs 0.01 per crore NSE IPFT = Rs 183 per crore total = 0.00183%, each side. Before 2026-03-01: Rs 173 + Rs 10 IPFT = Rs 183 (so total outflow is unchanged; only the split moved) | 2026-03-01 (NSE/FA/73061, dated 2026-02-27) | PRIMARY, read: https://archives.nseindia.com/content/circulars/FA73061.pdf . Prior circular cited inside it is NSE/FA/64323 dated 2024-09-27 (not 64232) | **verified (primary read)**. Conflict with earlier notes resolved: 0.00173% was the transaction-charge-only part; 0.00183% is the total incl. IPFT |
| NSE transaction charge, equity options (on premium) | Rs 3,552.99 per crore + Rs 0.01 IPFT = Rs 3,553 per crore total = 0.03553%, each side. Before 2026-03-01: Rs 3,503 + Rs 50 IPFT = Rs 3,553 | 2026-03-01 | same FA73061 | **verified (primary read)**. 0.03503% was the transaction-charge-only part; total 0.03553% unchanged |
| BSE transaction charge, options / futures | 0.0325% of premium / 0% (Zerodha page) | current as of page | https://zerodha.com/charges | single-source |
| Clearing charges (NSCCL) | Zerodha shows none for F&O; unverified at exchange level | | | unsure |
| GST | 18% on (brokerage + SEBI fee + exchange transaction charges). Not on STT or stamp duty | long-standing | https://zerodha.com/charges | confirmed |
| Brokerage (discount broker, for scale only) | Rs 20 flat per executed order for options | | https://zerodha.com/charges | broker-specific |

Worked example for a model: Nifty futures notional ~Rs 16 lakh: STT sell side ~Rs 806 per lot at 0.05% (sansalegal cites this).

Cost-model implication: STT is on the sell leg only; for options, sell premium x 0.15%. Cost per round trip is dominated by STT and spread on low-premium options. Existing repo file docs/costs-india.md likely holds the pre-2026 rates and should be reviewed against this table (not edited here).

## 2. SEBI / exchange rules, 2024-2026

| Rule | Value | Effective | Source | Confidence |
|---|---|---|---|---|
| SEBI circular "Measures to strengthen equity index derivatives framework" SEBI/HO/MRD/TPD/P/CIR/2024/132 | Six measures | issued 2024-10-01 | https://www.sebi.gov.in/legal/circulars/oct-2024/measures-to-strengthen-equity-index-derivatives-framework-for-increased-investor-protection-and-market-stability_87208.html (title/date/number read; body not rendered) | confirmed for existence; body details via secondary sources |
| Min index derivative contract value | Rs 15 lakh at introduction; lot sized so value on review day is Rs 15-20 lakh (was Rs 5-10 lakh) | Nov 20, 2024 (some outlets say Nov 21) | https://www.business-standard.com/amp/markets/news/sebi-announces-six-key-changes-to-curb-speculation-in-derivatives-trading-124100101316_1.html | confirmed (date +-1 day) |
| Weekly expiry | One benchmark index per exchange only | 2024-11-20 | same; https://blog.arihantcapital.com/sebi-tightens-the-reins-introduces-6-new-measures-for-fo-trading/ | confirmed |
| Upfront option premium collection from buyers | Required | 2025-02-01 | https://5paisa.com/news/sebi-tightens-fo-rules-to-curb-speculation-and-protect-investors | confirmed (date single-source) |
| Calendar spread benefit removed on expiry day | Yes | 2025-02-01 | same | single-source |
| Extra Extreme Loss Margin on short options on expiry day | +2% (some sources say 2%, one says 12%) | 2024-11-20 | secondary only | rate unsure |
| Intraday position-limit monitoring (index options) | Snapshots during the day; broker checks 4x/day | from 2025-04-01 (broker), regime from 2025-10-01 | https://www.valueresearchonline.com/stories/226216/real-time-checks-index-expiry-trades/ ; SEBI/HO/MRD/TPD-1/P/CIR/2025/79 dated 2025-05-29 | single/unsure |
| Expiry day restriction | One expiry weekday per exchange, Tuesday or Thursday | SEBI circular 2025-05-26; NSE moved to Tuesday for contracts expiring on/after 2025-09-01; BSE Thursday | https://www.businesstoday.in/markets/stocks/story/nse-receives-sebi-nod-for-tuesday-expiry-bse-thursday-480736-2025-06-17 | confirmed for mid-2025; no source after July 2026 re-checked: verify now |
| Index lot sizes (monthly/weekly) | Nifty 50 (NIFTY) 65 (was 75); Nifty Bank (BANKNIFTY) 30 (was 35); FINNIFTY 60 (was 65); MIDCPNIFTY 120 (was 140); NIFTYNXT50 25 unchanged. Earlier history: 25 -> 75 (Nov 2024) from secondary sources | Circular effective 2025-10-28 EOD; first weekly NIFTY expiry on new lot 2026-01-06; first monthly 2026-01-27; quarterly/half-yearly from 2025-12-30 EOD | PRIMARY, read: NSE/FAOP/70616 dated 2025-10-03 (Circular 176/2025) https://archives.nseindia.com/content/circulars/FAOP70616.pdf | **verified as of that circular (primary read)**. Later SEBI periodic revisions (Jun/Dec cycles) after Oct 2025 were searched for and none found, but not proven absent: check NSE contract specs on the day of use |
| 2026 SEBI circular "SEBI/HO/MRD/TPD/CIR/2026/034" (position limits, margins +200-300 bps, ELM, Bank Nifty/FinNifty weekly removed, 1 May 2026) | **UNVERIFIED**. Source is a social-media-driven article (multibagg.ai) and internally inconsistent; the circular number could not be located. Do not rely on it. | claimed 2026-05-01 | https://www.multibagg.ai/market-pulse/articles/sebi-fo-limits-margin-stt-cmsgh322a046b0zl3fi1zl5ht | unsure |

Note: Bank Nifty/FinNifty weekly expiries were already removed on NSE from Nov 2024 (one-benchmark rule). The claim of a 2026 re-do in that article is suspect.

## 3. Income tax characterisation

| Rule | Value | Source | Confidence |
|---|---|---|---|
| F&O on recognised exchange is non-speculative business income (PGBP) | Explanation 2 to s.43(5) of IT Act 1961 | https://www.taxninja.in/blog/turnover-calculation-for-option-traders-for-tax-audit ; https://www.jainam.in/blog/fo-income-tax-filing-guide/ | confirmed (secondary; statute widely known) |
| New Act: Income-tax Act 2025 in force from 2026-04-01 (tax year 2026-27 on). Old Act governs years up to 2025-26 (AY 2026-27 filing is still under 1961 Act, i.e. FY 2025-26) | | taxscan / taxguru results (see 44AB row) | confirmed by several secondary sources; statute not read |
| STT deductibility against F&O business income | sansalegal says STT is not deductible where income is speculative and forms part of acquisition cost for business income; general law (s.36(1)(xv)) allows STT as a business expense for business income. Treat as deductible for F&O but **unverified**; confirm with CA | https://www.sansalegal.com/post/stt-hike-on-futures-and-options-new-rates-effective-april-1-2026 | unsure |
| Other charges (brokerage, exchange fees, GST on brokerage) deductible as business expenses | | general practice | unsure/unverified |
| Tax rate | Slab rates (not 20%/12.5% capital gains) | general | confirmed (general law) |

## 4. Turnover computation (and why it matters)

- Method used in practice, ICAI Guidance Note on Tax Audit: futures and options turnover = sum of absolute profits and absolute losses on each settled/squared-off contract (net per-trade), plus the sales value of contracts not squared off (open at year-end/settled at expiry). Premium on sale of options: 7th edition added it to turnover; the 8th edition (issued 2022-08-19) said premium already in net profit should not be added separately. Source: https://www.taxninja.in/blog/turnover-calculation-for-option-traders-for-tax-audit . Confidence: confirmed for 8th edition via 2+ blogs. A taxguru post on the 2023 revision cites a formula including premium received (para 5.10(b)), and blogs on the 10th edition (revised 2025) say the method was updated. **Current edition wording unverified. Read the ICAI PDF.** https://taxguru.in/income-tax/methodology-determining-turnover-f-o-transactions.html
- Why it matters: turnover drives (a) 44AB audit trigger, (b) 44AD eligibility, (c) reconciliation vs AIS/broker P&L. Because turnover = absolute P&L rather than notional, a high-frequency options strategy rarely reaches Rs 1 cr turnover unless it trades many round-trips; but when premium is added (old basis) it can cross much sooner. Notional/premium-based turnover (which many people wrongly use) overstates it.
- Backtest implication: report turnover per the ICAI basis (sum of |P&L| per trade, plus unsquared open positions) rather than gross notional.

## 5. Tax audit (44AB), presumptive (44AD), ITR, advance tax

| Rule | Value | Source | Confidence |
|---|---|---|---|
| 44AB(a) audit | Turnover > Rs 1 crore; raised to Rs 10 crore if cash receipts and cash payments each <= 5% of total (F&O: effectively all digital, so Rs 10 crore) | https://www.jainam.in/blog/fo-income-tax-filing-guide/ ; https://taxguru.in/?p=1056526 | confirmed (multiple blogs). incometaxindia page returned 403 |
| 44AB(e) | If eligible for 44AD but declare profit < 6% (8% if non-digital) or a loss, and total income > basic exemption limit, audit required | jainam link | confirmed |
| Loss with turnover below threshold, income below basic exemption | no audit | https://fnotax.pages.dev/articles/fo-loss-and-tax-audit-the-basic-exemption-limit-exception-2026-guide/ | single-source (low-quality blog) |
| 44AD presumptive limit | Rs 2 crore; Rs 3 crore if cash receipts <= 5% (Finance Act 2023) | jainam, taxscan | confirmed value; whether F&O trading legitimately falls under 44AD is contested; many CAs advise against. **Unsure.** Using 44AD forbids carrying forward losses in practice, rate 6% of turnover. |
| Opt-out of 44AD within 5 yrs | audit when income > exemption limit (44AD(4)/44AB(e)) | taxscan | confirmed |
| New Act: s.63 replaces 44AB; single Form 26 replaces 3CA-3CD; "tax year" terminology; thresholds said to be same but one report says the audit safety-net for non-presumptive filers is removed | https://www.taxscan.in/top-stories/section63-new-act-vs-section44ab-old-income-tax-act-low-profit-declarations-now-mandate-audit-1445774 | single-source, unsure on mechanics. Applies to tax year 2026-27 (FY 2026-27), filed in 2027 |
| ITR form | ITR-3 (PGBP schedule; business code 09028 cited by jainam). ITR-4 only if 44AD. ITR-1/2 not valid with business income | jainam | confirmed (form), code single-source |
| Due dates | ITR with audit: 31 Oct (blogs cite 31 Oct 2026 for AY 2026-27); audit report 30 Sep. Non-audit individuals: 31 Jul | fnotax / jainam (conflicting statements on July 31 vs Oct 31) | single-source, check government notifications for extension |
| Advance tax | Required if liability >= Rs 10,000; instalments 15 Jun / 15 Sep / 15 Dec / 15 Mar (15%/45%/75%/100%); interest ss.234B/234C. Presumptive filers: single instalment by 15 Mar | jainam | confirmed (general law; percentages from memory) |
| Loss carry-forward | Non-speculative business loss carries forward 8 assessment years, only if return filed by the due date (s.139(1)) | jainam; https://fnotax.pages.dev/articles/fo-loss-and-tax-audit-do-you-need-an-audit-to-carry-forward-losses-2026-guide/ | confirmed |
| Set-off | Non-speculative business loss can be set off against any income except salary in same year (house property, capital gains incl. STCG/LTCG, other sources); carry-forward losses only against business income (any business) in later years | jainam; general law | confirmed |
| Books/audit needed to carry forward loss? | Loss can be carried forward without audit if turnover below audit limits and filed on time; if audit applies (44AB(e)), must be audited | fnotax | single-source |

## 6. Other 2024-2026 changes noted
- STT hike April 2026 (above). Reports say F&O volumes still rose year-on-year afterwards, and the government said no review of STT structure: https://www.basispointinsight.com/Story/govt-says-no-review-of-stt-structure--f-o-volumes-rise-despite-budget-tax-hike_9c6d75f3c414.html (single-source).
- Exchange charge reductions Oct 2024 (flat, same slab for all participants; "true to label" SEBI circular): https://zerodha.com/marketintel/bulletin/391488/revision-in-transactions-charges-from-1st-october-2024
- Budget 2024 changes to capital gains (STCG 20%, LTCG 12.5%) do not apply to F&O (business income).

## 7. Open items to verify before trusting numbers
1. ~~NSE equity-derivative transaction charge circular~~ done (section 8).
2. ~~Nifty lot size~~ done as of NSE/FAOP/70616; NSE expiry day still open (NSE contract spec page).
3. Enacted Finance Act 2026 text (STT verified via NSE circular and Bill memo); any Income-tax Act 2025 s.63 / schedule specifics.
4. Existence/content of claimed SEBI 2026 circular 2026/034.
5. ICAI Guidance Note edition in force for AY 2026-27 turnover rule for option premium.
6. Whether STT on F&O is deductible against business income (s.36(1)(xv)).
7. Stamp duty rate/date from NSE/Gazette notification.

## 8. Verification log (2026-11-07, primary-source pass)

Primary texts actually read (text-extracted locally):
- NSE/FATAX/73524, 2026-03-31, "Changes in relation to rates of Securities Transaction Tax" (https://archives.nseindia.com/content/circulars/FATAX73524.pdf; the nsearchives.nseindia.com host timed out, the archives.nseindia.com mirror worked). Confirms F&O STT rows in section 1 and that delivery (0.1% both sides), intraday (0.025% sell) and equity-fund (0.001%) STT are unchanged.
- NSE/FA/73061, 2026-02-27 (https://archives.nseindia.com/content/circulars/FA73061.pdf). Confirms NSE transaction charges and IPFT split from 2026-03-01 (total outflow unchanged: CM Rs 307, futures Rs 183, options Rs 3,553 per crore each side).
- NSE/FAOP/70616, 2025-10-03, lot sizes (https://archives.nseindia.com/content/circulars/FAOP70616.pdf).
- Finance Bill 2026 (Gazette extraordinary, 2026-02-01) explanatory extract (https://d23z1tp9il9etb.cloudfront.net/download/pdf26/Relevant%20Extract%20of%20The%20Finance%20Bill,%202026.pdf): Clause 143 amends s.98 Finance (No.2) Act 2004; old rates 0.1%/0.125%/0.02% and new options rates 0.15%/0.15% read; the new futures rate line is cut off in the extract. Finance Act s.1(2)(a): sections 2-113 in force 2026-04-01.
- Finance Bill 2026 direct-tax Memorandum (copy hosted at https://www.caalley.com/budget2026/memo.pdf; official indiabudget.gov.in copy not fetched): STT 0.1->0.15, 0.125->0.15, 0.02->0.05, effective 2026-04-01 for transactions entered on/after that date [Clause 143].

Not verified (could not read primary text):
- BSE equity/derivative transaction charges: no BSE circular found. A press aggregate (pulse.bot citing Hindu BusinessLine, May 2026) says BSE eliminated futures fees and cut options to Rs 5 per lakh (Rs 500 per crore); BSE Oct 2024 Sensex/Bankex options Rs 3,250 per crore. Leave as single-source/unverified.
- Stamp duty (F&O), SEBI turnover fee, GST, clearing charges: not re-checked.
- ICAI Guidance Note turnover rule (current edition): not read. ICAI PDF not found via search; the ITAT order PDF (itat.gov.in) failed TLS verification. Section 4 stays unverified.
- Tax-audit thresholds (Rs 1 cr/10 cr), 44AD limits, income-tax Act 2025 s.63: not read in the Act. The Finance Bill Memorandum does not restate them. It does show (new Act) the s.446 penalty for failure to get accounts audited being replaced by a graded fee of Rs 75,000/Rs 1,50,000 under proposed s.428(c); Bill-stage text.
- Due dates: Memorandum (Bill-stage) proposes non-audit business filers move from 31 July to 31 August (s.263(1)(c) new Act; similar amendment to Explanation 2 to s.139(1) of the 1961 Act); audit cases remain 31 October. Enactment and AY 2026-27 applicability not checked against the Act or a CBDT notification.
- STT deductibility (s.36(1)(xv)), SEBI circulars 2024/132 and 2026/034, expiry-day rules after mid-2026: not re-checked.

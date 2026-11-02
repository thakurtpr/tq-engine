# Rebuilds the NIFTY 50 point-in-time CSVs from the Wikipedia "NIFTY 50" article.
# Input (fetch first, in the working directory):
#   curl -sS -A "tq-research/0.1" "https://en.wikipedia.org/w/api.php?action=parse&page=NIFTY_50&prop=text&format=json&formatversion=2" -o nifty50_html.json
# Run: python3 -I build_nifty50_membership.py   (writes next to this file; OUT path below is absolute)
# Wikipedia is a secondary source: re-verify rows against NSE press releases (see ../universe.md).
import json, re, csv, html as H, datetime as dt, os, sys
from html.parser import HTMLParser

OUT = '/Users/thakur/Workspace/Trading/docs/data'
START = dt.date(2012, 1, 1)

class T(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tables=[]; self.cur=None; self.row=None; self.cell=None; self.depth=0; self.skip=0
    def handle_starttag(self, tag, attrs):
        a=dict(attrs)
        if tag=='table':
            self.depth+=1
            if self.depth==1: self.cur={'id':a.get('id'),'rows':[]}
        elif self.depth==1:
            if tag=='tr': self.row=[]
            elif tag in('td','th'):
                self.cell={'text':'','rs':int(a.get('rowspan',1)),'cs':int(a.get('colspan',1)),'cites':[]}
            elif tag=='sup': self.skip+=1
            elif tag=='a' and self.cell is not None and self.skip>0:
                h=a.get('href','')
                if h.startswith('#cite_note-'): self.cell['cites'].append(h[1:])
    def handle_endtag(self, tag):
        if tag=='table':
            if self.depth==1 and self.cur is not None:
                self.tables.append(self.cur); self.cur=None
            self.depth-=1
        elif self.depth==1:
            if tag in('td','th') and self.cell is not None:
                self.row.append(self.cell); self.cell=None
            elif tag=='tr' and self.row is not None:
                self.cur['rows'].append(self.row); self.row=None
            elif tag=='sup': self.skip=max(0,self.skip-1)
    def handle_data(self, d):
        if self.depth==1 and self.cell is not None and self.skip==0:
            self.cell['text']+=d

def expand(rows):
    grid=[]; pend={}
    for r in rows:
        out=[]; col=0; idx=0
        while idx<len(r) or pend:
            if col in pend:
                c,left=pend[col]; out.append(c)
                if left>1: pend[col]=(c,left-1)
                else: del pend[col]
                col+=1; continue
            if idx>=len(r):
                if any(k>=col for k in pend): col+=1; out.append(None); continue
                break
            c=r[idx]; idx+=1
            c=dict(c); c['text']=re.sub(r'\s+',' ',c['text']).strip()
            for _ in range(c['cs']):
                out.append(c)
                if c['rs']>1: pend[col]=(c,c['rs']-1)
                col+=1
        grid.append(out)
    return grid

raw=json.load(open('nifty50_html.json'))['parse']['text']
p=T(); p.feed(raw)
cur_grid=expand(p.tables[1]['rows']); chg_grid=expand(p.tables[2]['rows'])

# reference id -> first external URL
refs={}
for m in re.finditer(r'<li id="(cite(?:&#95;|_)note-[^"]+)">(.*?)</li>', raw, re.S):
    hm=re.search(r'href="(https?://[^"]+)"', m.group(2))
    if hm: refs[H.unescape(m.group(1))]=H.unescape(hm.group(1))

# canonical key -> NSE symbol history [(effective_from or None, symbol)]
D=dt.date
SYM = {}
def sym(key, hist=None): SYM[key]=hist or [(None,key)]
for k in """ADANIENT ADANIPORTS APOLLOHOSP ASIANPAINT AXISBANK BAJAJ-AUTO BAJFINANCE BAJAJFINSV BEL BHARTIARTL BSE CIPLA
COALINDIA DRREDDY EICHERMOT ETERNAL GRASIM HCLTECH HDFCBANK HDFCLIFE HINDALCO HINDUNILVR ICICIBANK INDIGO INFY ITC JIOFIN
JSWSTEEL KOTAKBANK LT M&M MARUTI MAXHEALTH NESTLEIND NTPC ONGC POWERGRID RELIANCE SBILIFE SHRIRAMFIN SBIN SUNPHARMA TCS
TATACONSUM TATASTEEL TECHM TITAN TRENT ULTRACEMCO RCOM RPOWER BANKBARODA LUPIN SAIL STER SIEMENS INDUSINDBK WIPRO NMDC
RELINFRA JPASSOCIAT RANBAXY ZEEL DLF IDEA JINDALSTEL YESBANK IDFC BOSCHLTD CAIRN AUROPHARMA PNB INFRATEL EICHERMOT BHEL
IOC ACC HINDPETRO TATAPOWER UPL AMBUJACEM BRITANNIA SHREECEM DIVISLAB GAIL HDFC LTIM BPCL HEROMOTOCO""".split(): sym(k)
sym('TATAMOTORS',[(None,'TATAMOTORS'),(D(2025,10,24),'TMPV')])           # NSE circular 16 Oct 2025 (via search summary)
sym('VEDANTA',[(None,'SESAGOA'),(D(2013,8,17),'SESASTERLITE'),(D(2015,4,1),'VEDL')])  # rename dates APPROX
sym('UNITEDSPIRITS',[(None,'MCDOWELL-N')])

NAME2KEY = {
 'Indian Hotels Company':None,'Adani Enterprises':'ADANIENT','Adani Ports & SEZ':'ADANIPORTS','Apollo Hospitals':'APOLLOHOSP',
 'Asian Paints':'ASIANPAINT','Axis Bank':'AXISBANK','Bajaj Auto':'BAJAJ-AUTO','Bajaj Finance':'BAJFINANCE','Bajaj Finserv':'BAJAJFINSV',
 'Bharat Electronics':'BEL','BSE':'BSE','Eternal':'ETERNAL','Grasim Industries':'GRASIM','Jio Financial Services':'JIOFIN',
 'JSW Steel':'JSWSTEEL','Kotak Mahindra Bank':'KOTAKBANK','Max Healthcare':'MAXHEALTH','Nestlé India':'NESTLEIND','Shriram Finance':'SHRIRAMFIN',
 'SBI Life Insurance Company':'SBILIFE','HDFC Life':'HDFCLIFE','Tata Consumer Products':'TATACONSUM','Tech Mahindra':'TECHM','Titan Company':'TITAN',
 'Trent':'TRENT','UltraTech Cement':'ULTRACEMCO','IndiGo':'INDIGO','Tata Consultancy Services':'TCS','Dr. Reddy\'s Laboratories':'DRREDDY',
 'Reliance Communications':'RCOM','Reliance Power':'RPOWER','Bank of Baroda':'BANKBARODA','Lupin':'LUPIN','Steel Authority of India':'SAIL',
 'Sterlite Industries':'STER','Siemens India':'SIEMENS','IndusInd Bank':'INDUSINDBK','Wipro':'WIPRO','NMDC':'NMDC','Reliance Infrastructure':'RELINFRA',
 'JP Associates':'JPASSOCIAT','Ranbaxy Laboratories':'RANBAXY','United Spirits':'UNITEDSPIRITS','Zee Entertainment Enterprises':'ZEEL','DLF':'DLF',
 'Idea Cellular':'IDEA','Jindal Steel & Power':'JINDALSTEL','Yes Bank':'YESBANK','IDFC':'IDFC','Bosch India':'BOSCHLTD','Cairn India':'CAIRN',
 'Aurobindo Pharma':'AUROPHARMA','Punjab National Bank':'PNB','Bharti Infratel':'INFRATEL','Vedanta':'VEDANTA','Eicher Motors':'EICHERMOT',
 'BHEL':'BHEL','Indiabulls Housing Finance':'IBULHSGFIN','Indian Oil Corporation':'IOC','ACC':'ACC','Hindustan Petroleum':'HINDPETRO',
 'Tata Power':'TATAPOWER','UPL':'UPL','Ambuja Cements':'AMBUJACEM','Britannia Industries':'BRITANNIA','Shree Cement':'SHREECEM',
 "Divi's Laboratories":'DIVISLAB','GAIL':'GAIL','HDFC':'HDFC','LTIMindtree':'LTIM','Bharat Petroleum':'BPCL','Hero MotoCorp':'HEROMOTOCO',
}
sym('IBULHSGFIN')
CUR2KEY={'TMPV':'TATAMOTORS'}

def sym_at(key, d):
    s=None
    for frm,name in SYM[key]:
        if frm is None or d>=frm: s=name
    return s

def pdate(s): return dt.datetime.strptime(s,'%d %B %Y').date()

# changes in window (rows after 3 header rows)
rows=[]
unmapped=set()
for r in chg_grid[3:]:
    exc,inc,dtx=r[0]['text'],r[1]['text'],r[2]['text']
    d=pdate(dtx)
    if d<START: continue
    for nm in (exc,inc):
        if nm not in NAME2KEY: unmapped.add(nm)
    OVERRIDE={dt.date(2026,9,30):'https://www.angelone.in/news/market-updates/bse-to-enter-nifty-50-replace-wipro-from-september-30-2026',
        dt.date(2022,9,30):'https://www.niftyindices.com/Press_Release/ind_prs01092022.pdf',
        dt.date(2023,7,13):'https://www.niftyindices.com/Press_Release/ind_prs04072023.pdf',
        dt.date(2024,3,28):'https://www.niftyindices.com/Press_Release/ind_prs28022024.pdf',
        dt.date(2025,9,30):'https://www.niftyindices.com/Press_Release/ind_prs22082025.pdf'}  # NSE originals, verified
    cites=[c for c in (r[4]['cites'] if len(r)>4 and r[4] else [])]
    url=''
    for c in cites:
        if c in refs: url=refs[c]; break
    url=OVERRIDE.get(d) or url
    rows.append((d,NAME2KEY.get(exc),NAME2KEY.get(inc),url,exc,inc,r[3]['text'] if len(r)>3 and r[3] else ''))
if unmapped: print('UNMAPPED',unmapped); sys.exit(1)

current={CUR2KEY.get(r[1]['text'],r[1]['text']) for r in cur_grid[1:]}
issues=[]
assert len(current)==50, len(current)

# walk back
state=set(current)
by_date={}
for row in rows: by_date.setdefault(row[0],[]).append(row)
dates_desc=sorted(by_date,reverse=True)
snap_state={}
for d in dates_desc:
    snap_state[d]=set(state)           # membership on/after d
    A={r[2] for r in by_date[d]}; R={r[1] for r in by_date[d]}
    if not A<=state: issues.append(f'{d}: included {A-state} not in membership when walking back')
    if R&state: issues.append(f'{d}: excluded {R&state} already in membership when walking back')
    state=(state-A)|R
start_state=set(state)
if len(start_state)!=50: issues.append(f'start snapshot has {len(start_state)} members')

# forward replay + validation
snaps=[(START,start_state)]
fwd=set(start_state)
for d in sorted(by_date):
    A={r[2] for r in by_date[d]}; R={r[1] for r in by_date[d]}
    if not R<=fwd: issues.append(f'{d}: forward replay missing {R-fwd}')
    fwd=(fwd-R)|A
    if fwd!=snap_state[d]: issues.append(f'{d}: forward != backward snapshot')
    snaps.append((d,set(fwd)))
if fwd!=current: issues.append('forward replay does not reproduce current list')
RENAME_DATE=dt.date(2025,10,24)
prev=[x for x in snaps if x[0]<RENAME_DATE][-1][1]
snaps.append((RENAME_DATE,set(prev))); snaps.sort(key=lambda x:x[0])
for d,s in snaps:
    syms=[sym_at(k,d) for k in s]
    if len(set(syms))!=50 or len(s)!=50: issues.append(f'{d}: {len(set(syms))} unique symbols / {len(s)} members')

# cross-check "Date added" for current members
incl_dates={}
for d,k_ex,k_in,*_ in rows: incl_dates.setdefault(k_in,[]).append(d)
for r in cur_grid[1:]:
    k=CUR2KEY.get(r[1]['text'],r[1]['text'])
    da=re.match(r'(\d+ \w+ \d{4})',r[3]['text'])
    if not da: issues.append(f'{k}: cannot parse date added {r[3]["text"]}'); continue
    da=pdate(da.group(1))
    if da>=START:
        if da not in incl_dates.get(k,[]): issues.append(f'{k}: "date added" {da} not in change table')
    else:
        if k not in start_state: issues.append(f'{k}: added {da} (pre-window) but absent from {START} snapshot')

os.makedirs(OUT,exist_ok=True)
with open(f'{OUT}/nifty50_changes.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow(['effective_date','added_symbol','removed_symbol','source_url'])
    for d,ke,ki,url,*_ in sorted(rows,key=lambda x:(x[0],x[2])):
        w.writerow([d.isoformat(),sym_at(ki,d),sym_at(ke,d),url])
with open(f'{OUT}/nifty50_members_by_date.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow(['date','symbol'])
    for d,s in snaps:
        for x in sorted(sym_at(k,d) for k in s): w.writerow([d.isoformat(),x])
# notes file
NOTES=[
('HDFC','HDFC Ltd','merged into HDFCBANK; removed from Nifty 50 on 2023-07-13 (table reason: Merger); merger effective ~2023-07-01 (my knowledge, unverified)','table'),
('STER','Sterlite Industries','removed 2012-09-28; later merged into Sesa Goa (2013, my knowledge)','table+knowledge'),
('SESAGOA','Sesa Goa -> Sesa Sterlite -> Vedanta Ltd','same company, key VEDL; symbols SESAGOA until ~2013-08-17, SESASTERLITE until ~2015-04-01, VEDL after. Rename dates APPROXIMATE (my knowledge); snapshot dates are not sensitive to them','approx'),
('SESASTERLITE','Sesa Sterlite','see SESAGOA','approx'),
('VEDL','Vedanta Ltd','member 2010-10-01..2016-03 (as SESAGOA/SESASTERLITE/VEDL), again 2017-05-26..2020-07-31; table calls it Vedanta','table'),
('RANBAXY','Ranbaxy Laboratories','removed 2014-03-28; merged into SUNPHARMA in 2015 (my knowledge)','table+knowledge'),
('CAIRN','Cairn India','removed 2016-04-01 (table: Merger); merged into Vedanta (my knowledge)','table'),
('INFRATEL','Bharti Infratel','removed 2020-09-25 (table: Merger with Indus Towers); later symbol INDUSTOWER (my knowledge)','table+knowledge'),
('IDEA','Idea Cellular / Vodafone Idea','symbol IDEA unchanged across the 2018 rename','knowledge'),
('IDFC','IDFC Ltd','removed 2015-05-29; later merged into IDFC First Bank (IDFCFIRSTB), my knowledge','table+knowledge'),
('IBULHSGFIN','Indiabulls Housing Finance','member 2017-03-31..2019-09-27; later renamed Sammaan Capital (my knowledge)','table+knowledge'),
('MCDOWELL-N','United Spirits','NSE symbol assumed MCDOWELL-N while a member (2014-03-28..2014-09-19); symbol at that time UNVERIFIED (later UNITDSPR)','unverified'),
('LTIM','LTIMindtree','formed by L&T Infotech + Mindtree merger (LTI->LTIM, ~2022-11); Nifty member 2023-07-13..2024-09-30 as LTIM','table+knowledge'),
('SHRIRAMFIN','Shriram Finance','formed by merger of Shriram Transport Finance and Shriram City Union (2022); added 2024-03-28','table+knowledge'),
('ETERNAL','Eternal (ex Zomato)','symbol ZOMATO until 2025 rename; Nifty added 2025-03-28 as ETERNAL','table'),
('TATAMOTORS','Tata Motors -> Tata Motors Passenger Vehicles','symbol TATAMOTORS until 2025-10-24, TMPV after (NSE circular dated 2025-10-16, via search summary). Demerged CV entity (TMCV, dummy symbol DUMMYTATAM) was added to Nifty 50 from 2025-10-14 and excluded effective 2025-11-17 per NSE press release 13 Nov 2025, so Nifty 50 had 51 constituents for about a month. NOT in the CSVs.','external'),
('BSE','BSE Ltd','added 2026-09-30 in place of Wipro (NSE Indices announcement effective after close 2026-09-29; confirmed by several news reports, Wikipedia row has no reference)','news'),
('WIPRO','Wipro','removed 2013-04-01 (demerger), re-added 2013-09-27, removed 2026-09-30','table'),
('HEROMOTOCO','Hero MotoCorp','member throughout window until removed 2025-09-30; its original inclusion predates the table','table'),
]
with open(f'{OUT}/nifty50_symbol_notes.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow(['symbol','company','note','certainty'])
    for n in NOTES: w.writerow(n)
print('changes',len(rows),'snapshots',len(snaps),'issues',len(issues))
for i in issues: print(' ISSUE',i)
print('missing url rows:',[(r[0].isoformat(),r[4],r[5]) for r in rows if not r[3]])

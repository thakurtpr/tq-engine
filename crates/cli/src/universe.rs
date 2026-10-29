//! Point-in-time universe for `tq backtest --universe-file`: which price series to run, when each
//! may be entered, and how many index member-days the downloaded data actually covers.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use chrono::NaiveDate;
use tq_core::Timestamp;
use tq_data::{ist_close_ts, Membership, Window};

use crate::CliError;

/// One price series to backtest and the fill-time windows in which it may be entered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSymbol {
    pub price_symbol: String,
    pub entry_windows: Vec<(Timestamp, Timestamp)>,
}

/// A membership symbol whose member-days in the window lack price data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    pub symbol: String,
    pub price_symbol: String,
    pub missing_days: usize,
    pub member_days: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Coverage {
    pub member_days: usize,
    pub covered_days: usize,
    pub missing: Vec<Missing>,
}

pub struct Universe {
    membership: Membership,
    /// Membership symbol -> price-file symbol (renames such as SESAGOA -> VEDL).
    aliases: BTreeMap<String, String>,
}

impl Universe {
    pub fn load(members: &Path, aliases: Option<&Path>) -> Result<Self, CliError> {
        let membership = Membership::load(members).map_err(|e| CliError::Usage(e.to_string()))?;
        let aliases = match aliases {
            Some(p) => parse_aliases(&fs::read_to_string(p).map_err(|e| CliError::Usage(format!("{}: {e}", p.display())))?)?,
            None => BTreeMap::new(),
        };
        Ok(Self { membership, aliases })
    }

    #[cfg(test)]
    fn new(membership: Membership, aliases: BTreeMap<String, String>) -> Self {
        Self { membership, aliases }
    }

    pub fn price_symbol<'a>(&'a self, symbol: &'a str) -> &'a str {
        self.aliases.get(symbol).map_or(symbol, String::as_str)
    }

    /// Membership windows overlapping `[start, end]`, keyed by membership symbol; members only.
    fn windows_in(&self, start: NaiveDate, end: NaiveDate) -> BTreeMap<String, Vec<Window>> {
        let mut out = BTreeMap::new();
        for symbol in self.membership.symbols() {
            let ws: Vec<Window> = self
                .membership
                .windows(&symbol)
                .into_iter()
                .filter(|(from, to)| *from <= end && to.is_none_or(|t| t > start))
                .collect();
            if !ws.is_empty() {
                out.insert(symbol, ws);
            }
        }
        out
    }

    /// Price series that were index members at some point in `[start, end]`, with entry windows
    /// merged across every membership symbol that maps to the same series.
    pub fn plan(&self, start: NaiveDate, end: NaiveDate) -> Vec<PlannedSymbol> {
        let mut by_price: BTreeMap<String, Vec<(Timestamp, Timestamp)>> = BTreeMap::new();
        for (symbol, ws) in self.windows_in(start, end) {
            let entry = by_price.entry(self.price_symbol(&symbol).to_string()).or_default();
            entry.extend(ws.iter().map(|&(from, to)| (ist_close_ts(from), to.map_or(Timestamp::MAX, ist_close_ts))));
        }
        by_price.into_iter().map(|(price_symbol, entry_windows)| PlannedSymbol { price_symbol, entry_windows }).collect()
    }

    /// Member-days on `calendar` within `[start, end]`, and how many have a bar in the member's
    /// price series (`dates_of` returns the series' bar dates, or `None` when there is no file).
    pub fn coverage<F>(&self, start: NaiveDate, end: NaiveDate, calendar: &BTreeSet<NaiveDate>, dates_of: F) -> Coverage
    where
        F: Fn(&str) -> Option<BTreeSet<NaiveDate>>,
    {
        let mut cov = Coverage::default();
        for (symbol, ws) in self.windows_in(start, end) {
            let price_symbol = self.price_symbol(&symbol).to_string();
            let have = dates_of(&price_symbol).unwrap_or_default();
            let days: Vec<&NaiveDate> =
                calendar.range(start..=end).filter(|d| ws.iter().any(|(f, t)| *d >= f && t.is_none_or(|t| **d < t))).collect();
            let missing_days = days.iter().filter(|d| !have.contains(d)).count();
            cov.member_days += days.len();
            cov.covered_days += days.len() - missing_days;
            if missing_days > 0 {
                cov.missing.push(Missing { symbol, price_symbol, missing_days, member_days: days.len() });
            }
        }
        cov
    }
}

/// `symbol,price_symbol,note` rows; the note may contain quoted commas and is ignored.
fn parse_aliases(text: &str) -> Result<BTreeMap<String, String>, CliError> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("symbol,price_symbol,note") {
        return Err(CliError::Usage("aliases file must start with `symbol,price_symbol,note`".into()));
    }
    let mut out = BTreeMap::new();
    for (i, line) in lines.enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let mut fields = line.splitn(3, ',').map(str::trim);
        match (fields.next(), fields.next()) {
            (Some(s), Some(p)) if !s.is_empty() && !p.is_empty() => out.insert(s.to_string(), p.to_string()),
            _ => return Err(CliError::Usage(format!("aliases line {}: want symbol,price_symbol,note", i + 2))),
        };
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMBERS: &str = "date,symbol\n2012-01-01,OLDNAME\n2012-01-01,GONE\n2012-01-10,NEWNAME\n2012-01-10,ADDED\n";

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2012, 1, day).unwrap()
    }

    fn universe() -> Universe {
        let m = Membership::from_reader(MEMBERS.as_bytes(), "m.csv").unwrap();
        let aliases = parse_aliases("symbol,price_symbol,note\nOLDNAME,NEWNAME,\"renamed, same company\"\n").unwrap();
        Universe::new(m, aliases)
    }

    #[test]
    fn plan_merges_renamed_symbols_into_one_price_series() {
        let plan = universe().plan(d(1), d(31));
        let names: Vec<&str> = plan.iter().map(|p| p.price_symbol.as_str()).collect();
        assert_eq!(names, vec!["ADDED", "GONE", "NEWNAME"]);
        let renamed = &plan[2].entry_windows;
        assert_eq!(renamed, &vec![(ist_close_ts(d(10)), Timestamp::MAX), (ist_close_ts(d(1)), ist_close_ts(d(10)))]);
        assert_eq!(plan[1].entry_windows, vec![(ist_close_ts(d(1)), ist_close_ts(d(10)))]);
    }

    #[test]
    fn plan_skips_symbols_never_members_in_the_window() {
        let plan = universe().plan(d(10), d(31));
        assert!(plan.iter().all(|p| p.price_symbol != "GONE"));
    }

    #[test]
    fn coverage_counts_member_days_with_and_without_data() {
        let calendar: BTreeSet<NaiveDate> = (1..=12).map(d).collect();
        // GONE has no data; ADDED misses day 12; NEWNAME (incl. OLDNAME) is complete.
        let cov = universe().coverage(d(1), d(12), &calendar, |s| match s {
            "NEWNAME" => Some(calendar.clone()),
            "ADDED" => Some((10..=11).map(d).collect()),
            _ => None,
        });
        // Days 1-9: OLDNAME (covered) + GONE (missing); days 10-12: NEWNAME (covered) + ADDED (2 of 3).
        assert_eq!((cov.member_days, cov.covered_days), (24, 9 + 3 + 2));
        let missing: Vec<(&str, usize)> = cov.missing.iter().map(|m| (m.symbol.as_str(), m.missing_days)).collect();
        assert_eq!(missing, vec![("ADDED", 1), ("GONE", 9)]);
    }

    #[test]
    fn aliases_reject_bad_header_and_rows() {
        assert!(parse_aliases("a,b\n").is_err());
        assert!(parse_aliases("symbol,price_symbol,note\nONLYONE\n").is_err());
    }
}

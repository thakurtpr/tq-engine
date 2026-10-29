//! Point-in-time index membership from `date,symbol` snapshot rows.
//!
//! Each snapshot date lists the full index on that date and holds until the next snapshot date,
//! so a change is in effect from its effective date (inclusive). Before the first snapshot nobody
//! is a member. Symbols are kept exactly as written (as of each snapshot date).

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use chrono::NaiveDate;
use csv::{ReaderBuilder, Trim};

use crate::DataError;

const HEADER: [&str; 2] = ["date", "symbol"];
const DATE_FORMAT: &str = "%Y-%m-%d";

/// A membership window: `[from, to)`; `to` is `None` while still a member at the last snapshot.
pub type Window = (NaiveDate, Option<NaiveDate>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    snapshots: BTreeMap<NaiveDate, BTreeSet<String>>,
}

impl Membership {
    pub fn load(path: &Path) -> Result<Self, DataError> {
        let name = path.display().to_string();
        let file = File::open(path).map_err(|e| DataError::Io { file: name.clone(), msg: e.to_string() })?;
        Self::from_reader(file, &name)
    }

    /// Parses `date,symbol` rows. `file` is used only in error messages.
    pub fn from_reader<R: Read>(input: R, file: &str) -> Result<Self, DataError> {
        let parse_err = |line: u64, msg: String| DataError::Parse { file: file.to_string(), line, msg };
        let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(input);
        let header = reader.headers().map_err(|e| parse_err(1, e.to_string()))?;
        if header.iter().collect::<Vec<_>>() != HEADER {
            return Err(parse_err(1, format!("expected header `{}`", HEADER.join(","))));
        }
        let mut snapshots: BTreeMap<NaiveDate, BTreeSet<String>> = BTreeMap::new();
        for record in reader.records() {
            let record = record.map_err(|e| parse_err(e.position().map_or(0, |p| p.line()), e.to_string()))?;
            let line = record.position().map_or(0, |p| p.line());
            let (raw_date, symbol) = (&record[0], &record[1]);
            let date = NaiveDate::parse_from_str(raw_date, DATE_FORMAT)
                .map_err(|e| parse_err(line, format!("bad date `{raw_date}`: {e}")))?;
            if symbol.is_empty() {
                return Err(parse_err(line, "empty symbol".into()));
            }
            snapshots.entry(date).or_default().insert(symbol.to_string());
        }
        if snapshots.is_empty() {
            return Err(parse_err(1, "no membership rows".into()));
        }
        Ok(Self { snapshots })
    }

    /// Every symbol that appears in any snapshot.
    pub fn symbols(&self) -> BTreeSet<String> {
        self.snapshots.values().flatten().cloned().collect()
    }

    /// Members in effect on `date` (the latest snapshot on or before it).
    pub fn members_on(&self, date: NaiveDate) -> Option<&BTreeSet<String>> {
        self.snapshots.range(..=date).next_back().map(|(_, members)| members)
    }

    pub fn is_member(&self, symbol: &str, date: NaiveDate) -> bool {
        self.members_on(date).is_some_and(|m| m.contains(symbol))
    }

    /// Contiguous membership windows of `symbol`, oldest first.
    pub fn windows(&self, symbol: &str) -> Vec<Window> {
        let mut out = Vec::new();
        let mut open: Option<NaiveDate> = None;
        for (date, members) in &self.snapshots {
            match (members.contains(symbol), open) {
                (true, None) => open = Some(*date),
                (false, Some(from)) => {
                    out.push((from, Some(*date)));
                    open = None;
                }
                _ => {}
            }
        }
        out.extend(open.map(|from| (from, None)));
        out
    }
}

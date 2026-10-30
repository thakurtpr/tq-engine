//! Append-only JSONL journal: one JSON object per line, never rewritten.
//!
//! Every record has `type`, `ts` (epoch ms of the bar or event) and `ts_iso` (UTC). Money is
//! written as decimal strings so nothing is rounded through `f64`. Record types: `start`,
//! `status`, `gap`, `stale`, `session`, `signal`, `fill`, `trade`, `mark`, `cancel`, `end`.

use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use chrono::{DateTime, SecondsFormat};
use rust_decimal::Decimal;
use serde_json::{Map, Value};
use tq_core::Timestamp;

pub struct Journal {
    out: Box<dyn Write>,
    /// Flush after every record (live), so a crash loses nothing.
    flush_each: bool,
    /// Also print non-`mark` records to stderr for the operator.
    echo: bool,
}

/// A decimal as a JSON string, trailing zeros removed.
pub fn dec(x: Decimal) -> Value {
    Value::String(x.normalize().to_string())
}

fn iso(ts: Timestamp) -> String {
    DateTime::from_timestamp_millis(ts).map_or_else(|| ts.to_string(), |d| d.to_rfc3339_opts(SecondsFormat::Secs, true))
}

impl Journal {
    /// Opens `path` for appending, creating it and its parent directory if needed.
    pub fn open(path: &Path, flush_each: bool, echo: bool) -> io::Result<Self> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            fs::create_dir_all(dir)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { out: Box::new(BufWriter::new(file)), flush_each, echo })
    }

    /// A journal writing to any sink (tests, or `io::sink()` to discard).
    pub fn to_writer(out: Box<dyn Write>) -> Self {
        Self { out, flush_each: true, echo: false }
    }

    pub fn record(&mut self, kind: &str, ts: Timestamp, fields: Value) -> io::Result<()> {
        let mut obj = Map::new();
        obj.insert("type".into(), Value::String(kind.into()));
        obj.insert("ts".into(), Value::from(ts));
        obj.insert("ts_iso".into(), Value::String(iso(ts)));
        if let Value::Object(extra) = fields {
            obj.extend(extra);
        }
        let line = Value::Object(obj).to_string();
        if self.echo && kind != "mark" {
            eprintln!("{line}");
        }
        self.out.write_all(line.as_bytes())?;
        self.out.write_all(b"\n")?;
        if self.flush_each {
            self.out.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::cell::RefCell;
    use std::io::{self, Write};
    use std::rc::Rc;

    /// A cloneable in-memory sink for asserting on journal lines.
    #[derive(Clone, Default)]
    pub struct SharedBuf(pub Rc<RefCell<Vec<u8>>>);

    impl Write for SharedBuf {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl SharedBuf {
        pub fn lines(&self) -> Vec<serde_json::Value> {
            let text = String::from_utf8(self.0.borrow().clone()).unwrap();
            text.lines().map(|l| serde_json::from_str(l).unwrap()).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::SharedBuf;
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[test]
    fn writes_one_typed_object_per_line_with_decimal_strings() {
        let buf = SharedBuf::default();
        let mut j = Journal::to_writer(Box::new(buf.clone()));
        j.record("fill", 1_704_067_200_000, json!({"price": dec(dec!(42000.50))})).unwrap();
        j.record("mark", 1_704_067_260_000, json!({})).unwrap();
        let lines = buf.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["type"], "fill");
        assert_eq!(lines[0]["ts_iso"], "2024-01-01T00:00:00Z");
        assert_eq!(lines[0]["price"], "42000.5");
    }

    #[test]
    fn open_appends_instead_of_truncating() {
        let p = std::env::temp_dir().join(format!("tq-journal-{}.jsonl", std::process::id()));
        let _ = fs::remove_file(&p);
        for _ in 0..2 {
            let mut j = Journal::open(&p, true, false).unwrap();
            j.record("start", 0, json!({})).unwrap();
        }
        assert_eq!(fs::read_to_string(&p).unwrap().lines().count(), 2);
    }
}

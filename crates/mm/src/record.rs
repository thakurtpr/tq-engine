//! One parsed recorder line. Unknown or malformed lines become `Record::Other` and are counted.

use serde_json::Value;

use crate::fixed;

pub type Levels = Vec<(i64, i64)>;

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    Depth { sym: String, e: i64, first: u64, last: u64, prev: u64, bids: Levels, asks: Levels },
    Trade { sym: String, e: i64, t: i64, px: i64, qty: i64, buyer_maker: bool },
    Ticker { sym: String, update: u64, bid: i64, bid_qty: i64, ask: i64, ask_qty: i64 },
    Snapshot { sym: String, last_update: u64, bids: Levels, asks: Levels },
    ServerTime { tx: i64, server: i64 },
    /// `info` carries the URL for connect/disconnect, so a trade-socket gap is told from a book gap.
    Link { event: String, info: String },
    Other,
}

impl Record {
    /// Connect/disconnect/stop of the socket carrying depth (anything but the USD-M `/market` route).
    pub fn is_book_link(&self) -> bool {
        matches!(self, Record::Link { event, info }
            if matches!(event.as_str(), "connect" | "disconnect" | "stop") && !info.contains("/market/"))
    }

    /// Connect/disconnect of the trade-only `/market` route: trades may be missing, the book is intact.
    pub fn is_trade_link(&self) -> bool {
        matches!(self, Record::Link { event, info } if matches!(event.as_str(), "connect" | "disconnect") && info.contains("/market/"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub rx: i64,
    pub rec: Record,
}

fn levels(v: &Value) -> Option<Levels> {
    v.as_array()?
        .iter()
        .map(|l| Some((fixed(l.get(0)?.as_str()?)?, fixed(l.get(1)?.as_str()?)?)))
        .collect()
}

fn num(v: &Value, key: &str) -> Option<i64> {
    v.get(key)?.as_i64()
}

fn uid(v: &Value, key: &str) -> Option<u64> {
    v.get(key)?.as_u64()
}

fn px(v: &Value, key: &str) -> Option<i64> {
    fixed(v.get(key)?.as_str()?)
}

fn stream(d: &Value) -> Option<Record> {
    let sym = d.get("s")?.as_str()?.to_string();
    match d.get("e")?.as_str()? {
        "depthUpdate" => Some(Record::Depth {
            sym,
            e: num(d, "E")?,
            first: uid(d, "U")?,
            last: uid(d, "u")?,
            prev: uid(d, "pu")?,
            bids: levels(d.get("b")?)?,
            asks: levels(d.get("a")?)?,
        }),
        "aggTrade" => Some(Record::Trade {
            sym,
            e: num(d, "E")?,
            t: num(d, "T")?,
            px: px(d, "p")?,
            qty: px(d, "q")?,
            buyer_maker: d.get("m")?.as_bool()?,
        }),
        "bookTicker" => Some(Record::Ticker {
            sym,
            update: uid(d, "u")?,
            bid: px(d, "b")?,
            bid_qty: px(d, "B")?,
            ask: px(d, "a")?,
            ask_qty: px(d, "A")?,
        }),
        _ => None,
    }
}

pub fn parse_line(text: &str) -> Option<Line> {
    let v: Value = serde_json::from_str(text).ok()?;
    let rx = num(&v, "rx")?;
    let rec = if let Some(sym) = v.get("snap").and_then(Value::as_str) {
        let m = v.get("m")?;
        Record::Snapshot {
            sym: sym.to_string(),
            last_update: uid(m, "lastUpdateId")?,
            bids: levels(m.get("bids")?)?,
            asks: levels(m.get("asks")?)?,
        }
    } else if let Some(st) = v.get("servertime") {
        Record::ServerTime { tx: num(&v, "tx")?, server: num(st, "serverTime")? }
    } else if let Some(ev) = v.get("event").and_then(Value::as_str) {
        let info = v.get("info").and_then(Value::as_str).unwrap_or("").to_string();
        Record::Link { event: ev.to_string(), info }
    } else {
        v.get("m").and_then(|m| m.get("data")).and_then(stream).unwrap_or(Record::Other)
    };
    Some(Line { rx, rec })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_recorder_line_shape() {
        let depth = r#"{"rx":10,"m":{"stream":"btcusdt@depth@100ms","data":{"e":"depthUpdate","E":9,"T":8,"s":"BTCUSDT","U":5,"u":7,"pu":4,"b":[["100.0","1.5"]],"a":[["100.1","0"]]}}}"#;
        assert_eq!(
            parse_line(depth).unwrap(),
            Line {
                rx: 10,
                rec: Record::Depth {
                    sym: "BTCUSDT".into(),
                    e: 9,
                    first: 5,
                    last: 7,
                    prev: 4,
                    bids: vec![(10_000_000_000, 150_000_000)],
                    asks: vec![(10_010_000_000, 0)],
                }
            }
        );
        let snap = r#"{"rx":1,"snap":"ETHUSDT","m":{"lastUpdateId":42,"E":1,"T":1,"bids":[["1.0","2.0"]],"asks":[]}}"#;
        assert!(matches!(parse_line(snap).unwrap().rec, Record::Snapshot { last_update: 42, .. }));
        let st = r#"{"rx":3,"tx":1,"servertime":{"serverTime":2}}"#;
        assert_eq!(parse_line(st).unwrap().rec, Record::ServerTime { tx: 1, server: 2 });
        let trade = r#"{"rx":3,"m":{"stream":"x","data":{"e":"aggTrade","E":2,"T":1,"s":"BTCUSDT","p":"100.0","q":"0.010","m":true}}}"#;
        assert!(matches!(parse_line(trade).unwrap().rec, Record::Trade { buyer_maker: true, .. }));
    }

    #[test]
    fn truncated_or_unknown_lines_do_not_panic() {
        assert_eq!(parse_line(r#"{"rx":1,"m":{"stream":"btcusdt@depth@100ms","data":{"e":"depthUp"#), None);
        assert_eq!(parse_line(r#"{"rx":1,"m":{"result":null,"id":1}}"#).unwrap().rec, Record::Other);
    }
}

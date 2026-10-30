//! Parsing of Binance public kline payloads (websocket `<sym>@kline_<i>` and REST /api/v3/klines).

use std::str::FromStr;

use rust_decimal::Decimal;
use serde_json::Value;
use tq_core::{Bar, Timestamp};

use crate::FeedError;

/// One kline. `close_ts` is Binance's inclusive close time (open + interval - 1 ms).
#[derive(Debug, Clone, PartialEq)]
pub struct Kline {
    pub bar: Bar,
    pub close_ts: Timestamp,
    /// The websocket's `x` flag; for REST it is decided by the caller from the clock.
    pub closed: bool,
}

fn err(msg: impl Into<String>) -> FeedError {
    FeedError::Parse(msg.into())
}

fn dec(v: &Value, what: &str) -> Result<Decimal, FeedError> {
    let s = v.as_str().ok_or_else(|| err(format!("{what} is not a string")))?;
    Decimal::from_str(s).map_err(|_| err(format!("{what} is not a decimal: {s}")))
}

fn int(v: &Value, what: &str) -> Result<i64, FeedError> {
    v.as_i64().ok_or_else(|| err(format!("{what} is not an integer")))
}

fn bar(ts: Timestamp, o: &Value, h: &Value, l: &Value, c: &Value, v: &Value) -> Result<Bar, FeedError> {
    let (o, h, l, c, v) = (dec(o, "open")?, dec(h, "high")?, dec(l, "low")?, dec(c, "close")?, dec(v, "volume")?);
    Bar::new(ts, o, h, l, c, v).map_err(|e| err(e.to_string()))
}

/// Parses a websocket kline message. `Ok(None)` for messages that are not klines (e.g. the
/// subscription ack).
pub fn ws_kline(text: &str) -> Result<Option<Kline>, FeedError> {
    let v: Value = serde_json::from_str(text).map_err(|e| err(e.to_string()))?;
    let Some(k) = v.get("k") else { return Ok(None) };
    let ts = int(&k["t"], "t")?;
    let close_ts = int(&k["T"], "T")?;
    let closed = k["x"].as_bool().ok_or_else(|| err("x is not a bool"))?;
    let bar = bar(ts, &k["o"], &k["h"], &k["l"], &k["c"], &k["v"])?;
    Ok(Some(Kline { bar, close_ts, closed }))
}

/// Parses a REST klines array. Every row is marked closed iff `close_ts < now_ms`.
pub fn rest_klines(text: &str, now_ms: Timestamp) -> Result<Vec<Kline>, FeedError> {
    let v: Value = serde_json::from_str(text).map_err(|e| err(e.to_string()))?;
    let rows = v.as_array().ok_or_else(|| err(format!("expected an array, got: {}", truncate(text))))?;
    rows.iter()
        .map(|r| {
            let ts = int(&r[0], "open time")?;
            let close_ts = int(&r[6], "close time")?;
            let bar = bar(ts, &r[1], &r[2], &r[3], &r[4], &r[5])?;
            Ok(Kline { bar, close_ts, closed: close_ts < now_ms })
        })
        .collect()
}

fn truncate(s: &str) -> &str {
    s.get(..200).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const WS: &str = r#"{"e":"kline","E":1,"s":"BTCUSDT","k":{"t":60000,"T":119999,"s":"BTCUSDT","i":"1m","o":"100.5","h":"101.0","l":"100.0","c":"100.8","v":"2.5","x":true}}"#;

    #[test]
    fn parses_closed_ws_kline() {
        let k = ws_kline(WS).unwrap().unwrap();
        assert!(k.closed);
        assert_eq!(k.bar.ts, 60_000);
        assert_eq!(k.close_ts, 119_999);
        assert_eq!(k.bar.close, dec!(100.8));
    }

    #[test]
    fn open_ws_kline_is_marked_not_closed() {
        let k = ws_kline(&WS.replace("\"x\":true", "\"x\":false")).unwrap().unwrap();
        assert!(!k.closed);
    }

    #[test]
    fn non_kline_message_is_ignored() {
        assert_eq!(ws_kline(r#"{"result":null,"id":1}"#).unwrap(), None);
    }

    #[test]
    fn rest_rows_closed_only_when_close_time_passed() {
        let text = r#"[[0,"1","2","0.5","1.5","10",59999,"0",1,"0","0","0"],[60000,"1.5","2","1","1.8","3",119999,"0",1,"0","0","0"]]"#;
        let ks = rest_klines(text, 100_000).unwrap();
        assert_eq!(ks.len(), 2);
        assert!(ks[0].closed && !ks[1].closed);
    }

    #[test]
    fn rest_error_body_is_a_parse_error() {
        assert!(rest_klines(r#"{"code":-1121,"msg":"Invalid symbol."}"#, 0).is_err());
    }

    #[test]
    fn invalid_ohlc_is_rejected() {
        assert!(ws_kline(&WS.replace("\"l\":\"100.0\"", "\"l\":\"102.0\"")).is_err());
    }
}

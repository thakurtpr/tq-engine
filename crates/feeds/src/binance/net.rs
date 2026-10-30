//! Real network adapters: tungstenite websocket, ureq REST, system clock. Public endpoints only.

use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tq_core::Timestamp;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use super::{RestKlines, WallClock, WsConn, WsConnector};
use crate::FeedError;

const WS_READ_TIMEOUT: Duration = Duration::from_secs(60);
const REST_TIMEOUT: Duration = Duration::from_secs(10);
const REST_BASE: &str = "https://api.binance.com";

fn net(e: impl ToString) -> FeedError {
    FeedError::Network(e.to_string())
}

/// Websocket connector. A read that sees nothing for 60 s fails, which forces a reconnect
/// (Binance sends kline updates every ~2 s and pings every 20 s, so silence means a dead link).
#[derive(Debug, Default)]
pub struct TungsteniteWs;

impl WsConnector for TungsteniteWs {
    fn connect(&mut self, url: &str) -> Result<Box<dyn WsConn>, FeedError> {
        let (mut ws, _resp) = tungstenite::connect(url).map_err(net)?;
        let timeout = Some(WS_READ_TIMEOUT);
        match ws.get_mut() {
            MaybeTlsStream::Plain(s) => s.set_read_timeout(timeout),
            MaybeTlsStream::Rustls(s) => s.get_mut().set_read_timeout(timeout),
            _ => Ok(()),
        }
        .map_err(net)?;
        Ok(Box::new(TungConn(ws)))
    }
}

struct TungConn(WebSocket<MaybeTlsStream<TcpStream>>);

impl WsConn for TungConn {
    fn next_text(&mut self) -> Result<Option<String>, FeedError> {
        match self.0.read() {
            Ok(Message::Text(t)) => Ok(Some(t)),
            Ok(Message::Close(frame)) => Err(net(format!("closed by server: {frame:?}"))),
            Ok(_) => Ok(None),
            Err(e) => Err(net(e)),
        }
    }
}

/// Public REST klines (`/api/v3/klines`, no key). Each call is one request (weight 2).
pub struct UreqRest {
    agent: ureq::Agent,
    base: String,
}

impl Default for UreqRest {
    fn default() -> Self {
        Self { agent: ureq::AgentBuilder::new().timeout(REST_TIMEOUT).build(), base: REST_BASE.into() }
    }
}

impl RestKlines for UreqRest {
    fn klines(&mut self, symbol: &str, interval: &str, start_ms: Timestamp, limit: u32) -> Result<String, FeedError> {
        let url = format!("{}/api/v3/klines", self.base);
        let resp = self
            .agent
            .get(&url)
            .query("symbol", symbol)
            .query("interval", interval)
            .query("startTime", &start_ms.to_string())
            .query("limit", &limit.to_string())
            .call();
        match resp {
            Ok(r) => r.into_string().map_err(net),
            Err(ureq::Error::Status(code, r)) => Err(net(format!("HTTP {code}: {}", r.into_string().unwrap_or_default()))),
            Err(e) => Err(net(e)),
        }
    }
}

#[derive(Debug, Default)]
pub struct SystemClock;

impl WallClock for SystemClock {
    fn now_ms(&self) -> Timestamp {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
    }

    fn sleep_ms(&mut self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}

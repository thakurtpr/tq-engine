//! Trading sessions: which bars are tradeable, which calendar day a bar belongs to (for the
//! daily-loss limit and session VWAP), and when positions must be flattened.
//!
//! All decisions use the bar's OPEN time (Binance and the NSE CSVs both stamp bars by open).

use tq_core::Timestamp;

const DAY_MS: i64 = 86_400_000;
const MINUTE_MS: i64 = 60_000;
/// IST is UTC+05:30, no daylight saving.
const IST_OFFSET_MS: i64 = 19_800_000;
pub const NSE_OPEN_MIN: i64 = 9 * 60 + 15;
pub const NSE_CLOSE_MIN: i64 = 15 * 60 + 30;
/// Positions are flattened at the open of the first bar starting at or after 15:15 IST.
pub const NSE_FLATTEN_MIN: i64 = 15 * 60 + 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    /// 24/7; the trading day (daily-loss limit, VWAP reset) is the UTC calendar day.
    Crypto24x7,
    /// NSE cash market, 09:15-15:30 IST; forced flatten at 15:15; day = IST calendar day.
    Nse,
}

impl Session {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "crypto" => Some(Self::Crypto24x7),
            "nse" => Some(Self::Nse),
            _ => None,
        }
    }

    fn local_ms(self, ts: Timestamp) -> Timestamp {
        match self {
            Self::Crypto24x7 => ts,
            Self::Nse => ts + IST_OFFSET_MS,
        }
    }

    /// Days since the epoch in the session's time zone.
    pub fn day_key(self, ts: Timestamp) -> i64 {
        self.local_ms(ts).div_euclid(DAY_MS)
    }

    /// Minutes since local midnight.
    pub fn minute_of_day(self, ts: Timestamp) -> i64 {
        self.local_ms(ts).rem_euclid(DAY_MS) / MINUTE_MS
    }

    /// True if a bar opening at `ts` belongs to the continuous session.
    pub fn in_session(self, ts: Timestamp) -> bool {
        match self {
            Self::Crypto24x7 => true,
            Self::Nse => (NSE_OPEN_MIN..NSE_CLOSE_MIN).contains(&self.minute_of_day(ts)),
        }
    }

    /// True if an order filling at `ts` would be at or after the forced-flatten time.
    pub fn flatten_due(self, ts: Timestamp) -> bool {
        match self {
            Self::Crypto24x7 => false,
            Self::Nse => self.minute_of_day(ts) >= NSE_FLATTEN_MIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2024-01-02 (a Tuesday) at `h:m` IST, as UTC epoch ms.
    fn ist(h: i64, m: i64) -> Timestamp {
        1_704_153_600_000 + (h * 60 + m) * MINUTE_MS - IST_OFFSET_MS
    }

    #[test]
    fn nse_session_bounds_and_flatten() {
        let s = Session::Nse;
        assert!(!s.in_session(ist(9, 10)));
        assert!(s.in_session(ist(9, 15)));
        assert!(s.in_session(ist(15, 25)));
        assert!(!s.in_session(ist(15, 30)));
        assert!(!s.flatten_due(ist(15, 10)));
        assert!(s.flatten_due(ist(15, 15)));
    }

    #[test]
    fn nse_day_is_the_ist_date() {
        let s = Session::Nse;
        assert_eq!(s.day_key(ist(9, 15)), s.day_key(ist(15, 29)));
        assert_eq!(s.minute_of_day(ist(9, 15)), NSE_OPEN_MIN);
    }

    #[test]
    fn crypto_day_rolls_at_utc_midnight_and_never_flattens() {
        let s = Session::Crypto24x7;
        assert_eq!(s.day_key(DAY_MS - 1), 0);
        assert_eq!(s.day_key(DAY_MS), 1);
        assert!(s.in_session(123) && !s.flatten_due(DAY_MS - 1));
    }

    #[test]
    fn parses_names() {
        assert_eq!(Session::parse("nse"), Some(Session::Nse));
        assert_eq!(Session::parse("x"), None);
    }
}

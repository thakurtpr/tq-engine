//! Offline market-making research on `tq-l2rec` recordings (docs/edge-hunt-02-mm-spec.md).
//!
//! Prices and quantities are fixed-point `i64` with 8 decimals, parsed from Binance's decimal
//! strings, so book levels compare exactly (no float keys).

pub mod book;
pub mod gate1;
pub mod reader;
pub mod record;
pub mod sync;

/// Fixed-point scale: 1.0 == 100_000_000.
pub const SCALE: i64 = 100_000_000;
const DECIMALS: usize = 8;

/// Parse a non-negative decimal string ("77192.10", "0.000") into fixed point. `None` on anything else.
pub fn fixed(s: &str) -> Option<i64> {
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    if int.is_empty() || frac.len() > DECIMALS || !int.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
        return None;
    }
    let int: i64 = int.parse().ok()?;
    let frac_val: i64 = if frac.is_empty() { 0 } else { frac.parse().ok()? };
    let frac_scaled = frac_val * 10_i64.pow((DECIMALS - frac.len()) as u32);
    int.checked_mul(SCALE)?.checked_add(frac_scaled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_binance_decimals_exactly() {
        assert_eq!(fixed("77192.10"), Some(7_719_210_000_000));
        assert_eq!(fixed("0.000"), Some(0));
        assert_eq!(fixed("123"), Some(12_300_000_000));
        assert_eq!(fixed("0.00000001"), Some(1));
    }

    #[test]
    fn rejects_malformed_numbers() {
        for bad in ["", ".5", "1.2.3", "-1", "1e5", "0.000000001", "abc"] {
            assert_eq!(fixed(bad), None, "{bad}");
        }
    }
}

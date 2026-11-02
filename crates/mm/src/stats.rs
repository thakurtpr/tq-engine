//! Spec section 7: net P&L in bps of filled notional per fee row, 1-hour block bootstrap with 95% and
//! Bonferroni 99.4% intervals, markouts, and the K2 / K3 kill checks.

use std::collections::BTreeMap;

use crate::sim::{FillKind, Queue, Sim};

pub const BOOT: usize = 10_000;
const SEED: u64 = 0x5EED_0019;
const HOUR_MS: i64 = 3_600_000;
/// (label, maker bps, taker bps); rows marked * are unverified account tiers (spec 5).
pub const FEE_ROWS: [(&str, f64, f64); 5] =
    [("regular", 2.0, 5.0), ("regular+BNB", 1.8, 4.5), ("VIP-like*", 1.0, 3.0), ("zero*", 0.0, 3.0), ("rebate*", -0.5, 3.0)];
pub const HEADLINE_MAKER_BPS: f64 = 2.0;
/// Fewer hourly blocks than this and the kill lines are labelled "not a verdict" [convention].
pub const MIN_VERDICT_HOURS: usize = 24;
/// Bonferroni over 8 tests at 5%: two-sided 99.375%.
pub const ADJ_LO: f64 = 0.003_125;
pub const ADJ_HI: f64 = 0.996_875;

/// SplitMix64: deterministic and dependency-free, enough for resampling indices.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) % n as u64) as usize
    }
}

/// Ratio of sums over resampled hours: returns (point, [2.5%, 97.5%], [adj lo, adj hi]).
pub fn ratio_ci(pairs: &[(f64, f64)]) -> Option<(f64, [f64; 2], [f64; 2])> {
    let den: f64 = pairs.iter().map(|p| p.1).sum();
    if pairs.is_empty() || den == 0.0 {
        return None;
    }
    let point = pairs.iter().map(|p| p.0).sum::<f64>() / den;
    let mut rng = Rng(SEED);
    let mut draws: Vec<f64> = (0..BOOT)
        .filter_map(|_| {
            let (mut n, mut d) = (0.0, 0.0);
            for _ in 0..pairs.len() {
                let p = pairs[rng.below(pairs.len())];
                n += p.0;
                d += p.1;
            }
            (d != 0.0).then(|| n / d)
        })
        .collect();
    draws.sort_by(f64::total_cmp);
    let q = |x: f64| draws[((draws.len() - 1) as f64 * x).round() as usize];
    Some((point, [q(0.025), q(0.975)], [q(ADJ_LO), q(ADJ_HI)]))
}

pub struct Summary {
    pub hours: usize,
    pub fills: usize,
    pub maker_notional: f64,
    pub taker_notional: f64,
    pub through_share: f64,
    pub cancel_share: f64,
    pub markout_mean: [Option<f64>; 3],
    /// Mean 10 s markout (bps) with its hour-block intervals.
    pub markout10: Option<(f64, [f64; 2], [f64; 2])>,
    /// Per fee row: net bps of filled notional with intervals.
    pub net: Vec<Option<(f64, [f64; 2], [f64; 2])>>,
}

pub fn summarise(s: &Sim) -> Summary {
    let mn: f64 = s.blocks.iter().map(|b| b.maker_notional).sum();
    let net = FEE_ROWS
        .iter()
        .map(|&(_, fm, ft)| {
            let pairs: Vec<(f64, f64)> = s
                .blocks
                .iter()
                .map(|b| ((b.gross - fm * 1e-4 * b.maker_notional - ft * 1e-4 * b.taker_notional) * 1e4, b.maker_notional))
                .collect();
            ratio_ci(&pairs)
        })
        .collect();
    let mut by_hour: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
    for f in &s.fills {
        if let Some(m) = f.markout_bps[1] {
            let e = by_hour.entry(f.t.div_euclid(HOUR_MS)).or_default();
            e.0 += m;
            e.1 += 1.0;
        }
    }
    let markout_mean = [0, 1, 2].map(|h| {
        let v: Vec<f64> = s.fills.iter().filter_map(|f| f.markout_bps[h]).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    });
    let placed = s.quotes_placed.max(1) as f64;
    Summary {
        hours: s.blocks.len(),
        fills: s.fills.len(),
        maker_notional: mn,
        taker_notional: s.blocks.iter().map(|b| b.taker_notional).sum(),
        through_share: s.fills.iter().filter(|f| f.kind == FillKind::TradeThrough).count() as f64 / s.fills.len().max(1) as f64,
        cancel_share: s.quotes_cancelled_unfilled as f64 / placed,
        markout_mean,
        markout10: ratio_ci(&by_hour.into_values().collect::<Vec<_>>()),
        net,
    }
}

fn fmt_ci(c: &Option<(f64, [f64; 2], [f64; 2])>) -> String {
    c.as_ref().map_or("n/a".into(), |(p, ci, adj)| format!("{p:+.2} [{:+.2}, {:+.2}] adj [{:+.2}, {:+.2}]", ci[0], ci[1], adj[0], adj[1]))
}

fn fmt_opt(x: Option<f64>) -> String {
    x.map_or("n/a".into(), |v| format!("{v:+.2}"))
}

/// One row per simulation (headline fee row), then fee rows and kill checks.
pub fn report(runs: &BTreeMap<String, Vec<Sim>>) -> String {
    let mut o = vec![
        "| symbol | cell | queue | L ms | hours | fills | trade-through | quotes cancelled unfilled | flattens | markout 1 s / 10 s / 60 s (bps) | net bps, regular fee (95%, adj 99.4%) |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    let mut k2_all_dead = true;
    let mut k3_all_below = true;
    let mut fee_rows = vec![];
    for (sym, sims) in runs {
        for s in sims {
            let m = summarise(s);
            o.push(format!(
                "| {sym} | {:?} | {:?} | {} | {} | {} | {:.0}% | {:.0}% | {} | {} / {} / {} | {} |",
                s.p.cell,
                s.p.queue,
                s.p.latency_ms,
                m.hours,
                m.fills,
                m.through_share * 100.0,
                m.cancel_share * 100.0,
                s.flattens,
                fmt_opt(m.markout_mean[0]),
                fmt_opt(m.markout_mean[1]),
                fmt_opt(m.markout_mean[2]),
                fmt_ci(&m.net[0])
            ));
            if s.p.queue == Queue::Optimistic && s.p.latency_ms == 50 {
                k2_all_dead &= m.net[0].as_ref().is_none_or(|n| n.2[1] <= 0.0);
            }
            if s.p.queue == Queue::Conservative && s.p.latency_ms == 100 {
                k3_all_below &= m.markout10.as_ref().is_none_or(|x| x.2[1] < HEADLINE_MAKER_BPS);
                let rows: Vec<String> = FEE_ROWS.iter().zip(&m.net).map(|(r, n)| format!("{}: {}", r.0, fmt_ci(n))).collect();
                fee_rows.push(format!("- {sym} {:?}: {}", s.p.cell, rows.join("; ")));
            }
        }
    }
    o.push(String::new());
    o.push("Fee rows, conservative queue, L = 100 ms (net bps of filled notional; * = unverified tier):".into());
    o.push(String::new());
    o.extend(fee_rows);
    o.push(String::new());
    let min_hours = runs.values().flatten().map(|s| s.blocks.len()).min().unwrap_or(0);
    if min_hours < MIN_VERDICT_HOURS {
        o.push(format!(
            "- **NOT A VERDICT:** only {min_hours} hourly block(s); the bootstrap needs at least {MIN_VERDICT_HOURS}, \
             and kills are read only on test-period data (spec 8, addendum 9a)."
        ));
    }
    o.push(format!(
        "- K2 (optimistic queue, L = 50 ms, regular fee: adj upper bound <= 0 in every cell on both symbols): {}",
        if k2_all_dead { "**fires**" } else { "does not fire" }
    ));
    o.push(format!(
        "- K3 (conservative, L = 100 ms: adj upper bound of mean 10 s markout < 2.0 bps in every cell): {}",
        if k3_all_below { "**fires**" } else { "does not fire" }
    ));
    o.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_ci_brackets_the_point_and_widens_for_the_adjusted_level() {
        let pairs: Vec<(f64, f64)> = (0..50).map(|i| ((i % 5) as f64 - 1.0, 1.0)).collect();
        let (p, ci, adj) = ratio_ci(&pairs).unwrap();
        assert!((p - 1.0).abs() < 1e-12);
        assert!(adj[0] <= ci[0] && ci[0] <= p && p <= ci[1] && ci[1] <= adj[1]);
    }

    #[test]
    fn ratio_ci_is_none_without_notional() {
        assert!(ratio_ci(&[]).is_none());
        assert!(ratio_ci(&[(1.0, 0.0)]).is_none());
    }
}

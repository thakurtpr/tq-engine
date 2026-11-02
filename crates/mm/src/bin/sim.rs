//! `tq-mm-sim`: replay a `tq-l2rec` recording through the pre-registered quoting grid
//! (4 cells x 2 queue models x 4 latencies per symbol, spec sections 3-7) in one pass.
//!
//! The window is measured from the first line's `rx`: `--skip-hours 0 --hours 24` is the dev slice.
//! Do not point this at test or holdout data before the sample is closed (spec addendum 9a).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tq_mm::drive::Driver;
use tq_mm::reader::{for_each_line, ordered_files, ReadStats};
use tq_mm::record::parse_line;
use tq_mm::sim::{Cell, Params, Queue, Sim};
use tq_mm::stats::report;
use tq_mm::SCALE;

const HOUR_MS: i64 = 3_600_000;
/// Spec 6: 100 ms headline, 50 / 200 / 400 ms sensitivity.
const LATENCIES_MS: [i64; 4] = [50, 100, 200, 400];
/// Spec 3: fixed quote sizes, about 150 USD (both above exchangeInfo minimum notional).
const QUOTE_SIZE: [(&str, i64); 2] = [("BTCUSDT", SCALE / 500), ("ETHUSDT", SCALE / 20)];

#[derive(Parser)]
#[command(about = "Simulate passive quoting on a tq-l2rec recording (research only, no orders)")]
struct Args {
    #[arg(default_value = "data/l2/um")]
    dir: PathBuf,
    #[arg(long, default_value_t = 0.0)]
    skip_hours: f64,
    #[arg(long)]
    hours: Option<f64>,
    #[arg(long)]
    out: Option<PathBuf>,
}

fn grid() -> BTreeMap<String, Vec<Sim>> {
    QUOTE_SIZE
        .iter()
        .map(|&(sym, q)| {
            let mut sims = vec![];
            for cell in Cell::ALL {
                for queue in [Queue::Conservative, Queue::Optimistic] {
                    for latency_ms in LATENCIES_MS {
                        sims.push(Sim::new(Params { cell, queue, latency_ms, q }));
                    }
                }
            }
            (sym.to_string(), sims)
        })
        .collect()
}

fn run(args: Args) -> Result<(), String> {
    let files = ordered_files(&args.dir).map_err(|e| format!("cannot list {}: {e}", args.dir.display()))?;
    if files.is_empty() {
        return Err(format!("no .jsonl.gz files in {}", args.dir.display()));
    }
    let mut driver = Driver::new(grid());
    let (mut stats, mut unparsable) = (ReadStats::default(), 0u64);
    let mut anchor: Option<i64> = None;
    let to_ms = |h: f64| (h * HOUR_MS as f64) as i64;
    for_each_line(&files, &mut stats, |text| {
        let Some(line) = parse_line(text) else {
            unparsable += 1;
            return;
        };
        let start = *anchor.get_or_insert(line.rx) + to_ms(args.skip_hours);
        if line.rx >= start && args.hours.is_none_or(|h| line.rx < start + to_ms(h)) {
            driver.push(line);
        }
    })
    .map_err(|e| format!("read failed: {e}"))?;
    let (runs, c) = driver.finish();
    let inversions: u64 = c.order_violations.iter().sum();
    let gate = if inversions == 0 {
        String::new()
    } else {
        format!("\n**SAMPLE NOT TRUSTED:** {inversions} events left the reorder window out of exchange-time order; raise REORDER_MS and rerun.\n")
    };
    let text = format!(
        "# MM simulation: {}\n\n- files {}; lines {}; unparsable {}; truncated tails {}\n- window: skip {} h, length {}\n\
         - released {}; exchange-time order violations (depth / snapshot+link / trade) {:?}, worst {} ms; \
         trades while unsynced {}; book resets {}; trade-socket gaps {}\n{gate}\n{}\n",
        args.dir.display(),
        stats.files,
        stats.lines,
        unparsable,
        stats.truncated_files.len(),
        args.skip_hours,
        args.hours.map_or("all".to_string(), |h| format!("{h} h")),
        c.released,
        c.order_violations,
        c.worst_inversion_ms,
        c.trades_while_unsynced,
        c.book_resets,
        c.trade_gaps,
        report(&runs)
    );
    print!("{text}");
    if let Some(out) = &args.out {
        std::fs::write(out, &text).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    }
    if inversions > 0 {
        return Err("exchange-time order inversions: sample not trusted".into());
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

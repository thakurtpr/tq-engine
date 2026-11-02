//! `tq-mm-gate1`: run the Gate 1 book-integrity checks over a `tq-l2rec` directory.
//!
//! The window is measured from the first line's `rx`: `--skip-hours 0 --hours 24` is the dev slice.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tq_mm::gate1::Gate1;
use tq_mm::reader::{for_each_line, ordered_files, ReadStats};
use tq_mm::record::parse_line;

const HOUR_MS: i64 = 3_600_000;

#[derive(Parser)]
#[command(about = "Gate 1: can this tq-l2rec recording rebuild the order book?")]
struct Args {
    #[arg(default_value = "data/l2/um")]
    dir: PathBuf,
    /// Start this many hours after the first recorded line.
    #[arg(long, default_value_t = 0.0)]
    skip_hours: f64,
    /// Window length in hours (default: everything).
    #[arg(long)]
    hours: Option<f64>,
    /// Write the markdown report here as well as to stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn run(args: Args) -> Result<bool, String> {
    let files = ordered_files(&args.dir).map_err(|e| format!("cannot list {}: {e}", args.dir.display()))?;
    if files.is_empty() {
        return Err(format!("no .jsonl.gz files in {}", args.dir.display()));
    }
    let mut gate = Gate1::default();
    let mut stats = ReadStats::default();
    let mut anchor: Option<i64> = None;
    let to_ms = |h: f64| (h * HOUR_MS as f64) as i64;
    for_each_line(&files, &mut stats, |text| {
        let Some(line) = parse_line(text) else {
            gate.unparsable();
            return;
        };
        let start = *anchor.get_or_insert(line.rx) + to_ms(args.skip_hours);
        let in_window = line.rx >= start && args.hours.is_none_or(|h| line.rx < start + to_ms(h));
        if in_window {
            gate.observe(line);
        }
    })
    .map_err(|e| format!("read failed: {e}"))?;
    let mut report = format!(
        "# Gate 1: {}\n\n- files read: {}; complete lines: {}; truncated gzip tails: {} {:?}\n- window: skip {} h, length {}\n",
        args.dir.display(),
        stats.files,
        stats.lines,
        stats.truncated_files.len(),
        stats.truncated_files,
        args.skip_hours,
        args.hours.map_or("all".to_string(), |h| format!("{h} h")),
    );
    report.push_str(&gate.report());
    report.push('\n');
    print!("{report}");
    if let Some(out) = &args.out {
        std::fs::write(out, &report).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    }
    Ok(gate.verdicts().iter().all(|v| v.passes()))
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

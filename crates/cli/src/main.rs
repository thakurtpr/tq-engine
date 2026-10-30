//! `tq`: backtests and data checks. Paper/backtest only; there is no live broker.

mod args;
mod backtest;
mod benchmark;
mod gates;
mod manifest;
mod markdown;
mod report_portfolio;
mod stats;
mod summary;
mod universe;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;
use tq_core::Symbol;
use tq_data::{quality_report, BarSource, CsvDaily};
use tq_risk::RiskLimits;

use tq_data::ist_close_ts;

use crate::args::{check_holdout, parse_symbols, BacktestArgs, Cli, Command, Mode, QualityArgs};
use crate::backtest::{EntryWindows, Setup};
use crate::manifest::Manifest;
use crate::report_portfolio::Context;
use crate::universe::{Coverage, Universe};

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(String),
    #[error("cannot write {path}: {msg}")]
    Write { path: String, msg: String },
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Some(Command::Backtest(a)) => backtest(&a),
        Some(Command::Quality(a)) => quality(&a),
        None => {
            banner();
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn banner() {
    let limits = RiskLimits::default();
    println!("tq engine: paper/backtest only (try `tq --help`)");
    println!("risk limits: {limits:?}");
    match limits.ensure_live_ready() {
        Ok(()) => println!("live: limits confirmed (live broker not implemented)"),
        Err(e) => println!("live: BLOCKED ({e})"),
    }
}

fn backtest(args: &BacktestArgs) -> Result<(), CliError> {
    check_holdout(args)?;
    let setup = Setup::from_args(args)?;
    let (plan, coverage) = plan(args, &setup)?;
    let manifest = manifest::build(&inputs(args, &plan))?;
    let manifest_line = match &args.out {
        Some(out) => {
            let path = manifest::write(&manifest, out)?;
            format!("`{}` ({} files), SHA-256 `{}`", path.display(), manifest.files, manifest.sha256)
        }
        None => format!("{} files, SHA-256 `{}` (not written: no --out)", manifest.files, manifest.sha256),
    };
    let report = match args.mode {
        Mode::PerSymbol => {
            let results = setup.run_all(&plan);
            markdown::render(args, &setup, &results, &stats::pool(&results), coverage.as_ref(), &manifest_line)
        }
        Mode::Portfolio => portfolio(args, &setup, &plan, coverage.as_ref(), (&manifest, &manifest_line))?,
    };
    match &args.out {
        Some(path) => {
            fs::write(path, &report).map_err(|e| CliError::Write { path: path.display().to_string(), msg: e.to_string() })?;
            eprintln!("wrote {}", path.display());
        }
        None => print!("{report}"),
    }
    Ok(())
}

fn portfolio(args: &BacktestArgs, setup: &Setup, plan: &[(String, EntryWindows)], coverage: Option<&Coverage>, manifest: (&Manifest, &str)) -> Result<String, CliError> {
    let bench = match &args.benchmark {
        Some(p) => Some(benchmark::load(p, ist_close_ts(args.start), ist_close_ts(args.end))?),
        None => None,
    };
    let (report, skipped) = setup.run_portfolio(plan)?;
    let view = report_portfolio::view(&report, args.cash, bench.as_ref());
    if let Some(path) = &args.summary_csv {
        summary::append(path, &summary::line(args, setup, &view, bench.as_ref(), &manifest.0.sha256))?;
    }
    let ctx = Context { args, setup, coverage, bench: bench.as_ref(), skipped: &skipped, manifest: manifest.1 };
    Ok(report_portfolio::render(&ctx, &view))
}

/// Every input CSV the run reads: price files that exist, membership, aliases, benchmark.
fn inputs(args: &BacktestArgs, plan: &[(String, EntryWindows)]) -> Vec<PathBuf> {
    let prices = plan.iter().map(|(t, _)| args.data_dir.join(format!("{t}.csv"))).filter(|p| p.is_file());
    let extra = [&args.universe_file, &args.aliases, &args.benchmark].into_iter().flatten().cloned();
    prices.chain(extra).collect()
}

/// Symbols to run with their entry windows, plus member-day coverage for a point-in-time universe.
/// Tickers with their entry windows, and coverage when the universe is point-in-time.
type Plan = (Vec<(String, EntryWindows)>, Option<Coverage>);

fn plan(args: &BacktestArgs, setup: &Setup) -> Result<Plan, CliError> {
    let Some(path) = &args.universe_file else {
        let spec = args.symbols.as_deref().unwrap_or_default();
        return Ok((parse_symbols(spec)?.into_iter().map(|s| (s, None)).collect(), None));
    };
    let universe = Universe::load(path, args.aliases.as_deref())?;
    let planned = universe.plan(args.start, args.end);
    let calendar = planned.iter().filter_map(|p| setup.bar_dates(&p.price_symbol)).flatten().collect();
    let coverage = universe.coverage(args.start, args.end, &calendar, |s| setup.bar_dates(s));
    let plan = planned.into_iter().map(|p| (p.price_symbol, Some(p.entry_windows))).collect();
    Ok((plan, Some(coverage)))
}

fn quality(args: &QualityArgs) -> Result<(), CliError> {
    let source = CsvDaily::new(&args.data_dir);
    for ticker in parse_symbols(&args.symbols)? {
        let line = source
            .bars(&Symbol(format!("NSE:{ticker}")))
            .map_err(|e| e.to_string())
            .and_then(|bars| quality_report(&bars).map_err(|e| e.to_string()));
        match line {
            Ok(q) => {
                let range = format!("{:?}..{:?}", q.first, q.last);
                println!("{ticker}: bars={} range={range} gaps={:?} zero_volume={:?} jumps={:?}", q.bars, q.gaps, q.zero_volume, q.jumps);
            }
            Err(e) => println!("{ticker}: ERROR {e}"),
        }
    }
    Ok(())
}

//! `tq`: backtests and data checks. Paper/backtest only; there is no live broker.

mod args;
mod backtest;
mod markdown;
mod stats;

use std::fs;
use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;
use tq_core::Symbol;
use tq_data::{quality_report, BarSource, CsvDaily};
use tq_risk::RiskLimits;

use crate::args::{check_holdout, parse_symbols, BacktestArgs, Cli, Command, QualityArgs};
use crate::backtest::Setup;

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
    let symbols = parse_symbols(&args.symbols)?;
    let setup = Setup::from_args(args)?;
    let results = setup.run_all(&symbols);
    let pooled = stats::pool(&results);
    let report = markdown::render(args, &setup, &results, &pooled);
    match &args.out {
        Some(path) => {
            fs::write(path, &report).map_err(|e| CliError::Write { path: path.display().to_string(), msg: e.to_string() })?;
            eprintln!("wrote {} ({} symbols, {} trades)", path.display(), pooled.symbols_run, pooled.trades);
        }
        None => print!("{report}"),
    }
    Ok(())
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

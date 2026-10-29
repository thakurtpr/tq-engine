//! Command-line arguments for `tq backtest`, the symbol list and the holdout guard.

use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};
use rust_decimal::Decimal;

use crate::CliError;

/// Last day of the development window (`docs/validation-protocol.md`, section 1).
pub const DEVELOPMENT_END: NaiveDate = match NaiveDate::from_ymd_opt(2021, 12, 31) {
    Some(d) => d,
    None => panic!("invalid DEVELOPMENT_END"),
};

#[derive(Debug, Parser)]
#[command(name = "tq", about = "tq engine: backtests and paper trading only, no live broker")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Backtest one strategy on each symbol independently and write a Markdown report.
    Backtest(BacktestArgs),
    /// Print the data quality report (gaps, zero volume, >20% jumps) for each symbol's file.
    Quality(QualityArgs),
}

#[derive(Debug, Args)]
pub struct QualityArgs {
    #[arg(long)]
    pub data_dir: PathBuf,
    /// Comma-separated NSE tickers, or @path to a file with one ticker per line.
    #[arg(long)]
    pub symbols: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StrategyName {
    TrendPullback,
    Donchian,
}

#[derive(Debug, Args)]
pub struct BacktestArgs {
    #[arg(long, value_enum)]
    pub strategy: StrategyName,
    /// Directory of <SYMBOL>.csv daily files (scripts/fetch_daily.py output).
    #[arg(long)]
    pub data_dir: PathBuf,
    /// Comma-separated NSE tickers, or @path to a file with one ticker per line (# comments).
    #[arg(long, required_unless_present = "universe_file", conflicts_with = "universe_file")]
    pub symbols: Option<String>,
    /// Point-in-time membership CSV (`date,symbol` snapshots). Runs every symbol that was a member
    /// during the window and lets it be ENTERED only while a member; exits are never restricted.
    #[arg(long)]
    pub universe_file: Option<PathBuf>,
    /// `symbol,price_symbol,note` CSV mapping membership symbols to price files (renames).
    #[arg(long)]
    pub aliases: Option<PathBuf>,
    /// First trading day, YYYY-MM-DD. Earlier bars in the file only warm up indicators.
    #[arg(long)]
    pub start: NaiveDate,
    /// Last day, YYYY-MM-DD, inclusive. Later bars are never loaded.
    #[arg(long)]
    pub end: NaiveDate,
    #[arg(long, default_value = "1000000")]
    pub cash: Decimal,
    /// Percent of current equity risked per trade (placeholder default, not a user decision).
    #[arg(long, default_value = "0.5")]
    pub risk_pct: Decimal,
    /// Adverse slippage per side in basis points (validation protocol: 5 for NIFTY 50 names).
    #[arg(long, default_value = "5")]
    pub slippage_bps: Decimal,
    /// Override the risk gate's drawdown kill switch (percent). Default: the gate's placeholder.
    #[arg(long)]
    pub max_drawdown_pct: Option<Decimal>,
    /// Write the Markdown report here instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Allow --end inside the held-out window. The holdout may be used ONCE (protocol rule 0.5).
    #[arg(long = "i-am-running-the-holdout")]
    pub holdout: bool,
}

/// Refuses any run that would read held-out data unless the operator says so explicitly.
pub fn check_holdout(args: &BacktestArgs) -> Result<(), CliError> {
    if args.aliases.is_some() && args.universe_file.is_none() {
        return Err(CliError::Usage("--aliases only applies with --universe-file".into()));
    }
    if args.start > args.end {
        return Err(CliError::Usage(format!("--start {} is after --end {}", args.start, args.end)));
    }
    if args.end > DEVELOPMENT_END && !args.holdout {
        return Err(CliError::Usage(format!(
            "--end {} is inside the held-out window (after {DEVELOPMENT_END}). The holdout is touched \
             once, for the final test only; any change after seeing it burns it (validation-protocol.md, \
             rules 0.5 and 1). Pass --i-am-running-the-holdout if this is that run.",
            args.end
        )));
    }
    Ok(())
}

/// Parses `A,B,C` or `@file`. Tickers are upper-cased, de-duplicated and kept in input order.
pub fn parse_symbols(spec: &str) -> Result<Vec<String>, CliError> {
    let text = match spec.strip_prefix('@') {
        Some(path) => fs::read_to_string(path).map_err(|e| CliError::Usage(format!("cannot read {path}: {e}")))?,
        None => spec.replace(',', "\n"),
    };
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines().map(|l| l.split('#').next().unwrap_or_default().trim()) {
        let t = raw.to_ascii_uppercase();
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
    }
    if out.is_empty() {
        return Err(CliError::Usage("no symbols given".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> BacktestArgs {
        let base = ["tq", "backtest", "--strategy", "donchian", "--data-dir", "d", "--symbols", "INFY"];
        let cli = Cli::try_parse_from(base.iter().chain(extra)).unwrap();
        match cli.command {
            Some(Command::Backtest(a)) => a,
            other => panic!("not a backtest: {other:?}"),
        }
    }

    #[test]
    fn defaults_match_the_brief() {
        let a = args(&["--start", "2012-01-01", "--end", "2021-12-31"]);
        assert_eq!((a.cash, a.risk_pct, a.slippage_bps), (Decimal::from(1_000_000), Decimal::new(5, 1), Decimal::from(5)));
        assert!(check_holdout(&a).is_ok());
    }

    #[test]
    fn holdout_guard_refuses_end_after_development_window() {
        let a = args(&["--start", "2012-01-01", "--end", "2022-01-01"]);
        let err = check_holdout(&a).unwrap_err().to_string();
        assert!(err.contains("held-out") && err.contains("--i-am-running-the-holdout"), "{err}");
    }

    #[test]
    fn holdout_guard_yields_to_explicit_flag() {
        let a = args(&["--start", "2012-01-01", "--end", "2026-09-30", "--i-am-running-the-holdout"]);
        assert!(check_holdout(&a).is_ok());
    }

    #[test]
    fn rejects_start_after_end() {
        assert!(check_holdout(&args(&["--start", "2015-01-01", "--end", "2014-01-01"])).is_err());
    }

    #[test]
    fn symbols_and_universe_file_are_mutually_exclusive_but_one_is_required() {
        let base = ["tq", "backtest", "--strategy", "donchian", "--data-dir", "d", "--start", "2012-01-01", "--end", "2013-01-01"];
        let parse = |extra: &[&str]| Cli::try_parse_from(base.iter().chain(extra)).is_ok();
        assert!(!parse(&[]));
        assert!(parse(&["--universe-file", "m.csv", "--aliases", "a.csv"]));
        assert!(!parse(&["--universe-file", "m.csv", "--symbols", "INFY"]));
    }

    #[test]
    fn aliases_without_universe_file_are_refused() {
        assert!(check_holdout(&args(&["--start", "2012-01-01", "--end", "2013-01-01", "--aliases", "a.csv"])).is_err());
    }

    #[test]
    fn parses_comma_list_dedups_and_uppercases() {
        assert_eq!(parse_symbols("infy, TCS,,INFY").unwrap(), vec!["INFY", "TCS"]);
        assert!(parse_symbols(" , ").is_err());
    }

    #[test]
    fn parses_symbol_file_with_comments() {
        let path = std::env::temp_dir().join(format!("tq-symbols-{}.txt", std::process::id()));
        fs::write(&path, "# universe\nRELIANCE\nM&M  # comment\n\n").unwrap();
        let parsed = parse_symbols(&format!("@{}", path.display()));
        fs::remove_file(&path).unwrap();
        assert_eq!(parsed.unwrap(), vec!["RELIANCE", "M&M"]);
    }
}

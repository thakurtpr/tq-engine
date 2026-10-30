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
    /// Backtest one strategy (shared-account portfolio by default) and write a Markdown report.
    Backtest(Box<BacktestArgs>),
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
    /// Rebalancer: 12-1 momentum, top 10 of the point-in-time members, monthly (pre-registered).
    MomentumRotation,
    /// Rebalancer: hold the one --symbols fund above its 10-month SMA, else cash (Faber 2007).
    IndexTrendFilter,
    /// Rebalancer: hold the one --symbols fund throughout (the benchmark).
    BuyHold,
}

impl StrategyName {
    /// True for the month-end target-weight strategies (`run_rebalance`).
    pub fn is_rebalancer(self) -> bool {
        matches!(self, Self::MomentumRotation | Self::IndexTrendFilter | Self::BuyHold)
    }
}

/// How the symbols share capital.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// One shared-cash account and one risk state across all symbols (the headline).
    Portfolio,
    /// Each symbol in its own fresh account: a diagnostic, not a performance claim.
    PerSymbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PriorityArg {
    /// Ticker order: deterministic and unrelated to the signal.
    Alphabetical,
    /// Seeded daily shuffle (`--seed`), for sensitivity analysis.
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SlippageArg {
    /// docs/slippage-model.md: base bps, sqrt impact, 2x stops, 1% ADV cap, Rs 0.05 ticks.
    Adopted,
    /// Flat --slippage-bps on every fill (reproduces backtest-01/02).
    Flat,
}

/// Whether prices are on the exchange's tick grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PriceBasis {
    /// Split/dividend-adjusted (Yahoo `auto_adjust`): off the tick grid, so no tick rounding.
    Adjusted,
    /// Raw traded prices: fills are rounded to the Rs 0.05 tick against the trader.
    Unadjusted,
}

#[derive(Debug, Args)]
pub struct BacktestArgs {
    #[arg(long, value_enum, default_value = "portfolio")]
    pub mode: Mode,
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
    /// Base adverse slippage per side in basis points (validation protocol: 5 for NIFTY 50 names).
    #[arg(long, default_value = "5")]
    pub slippage_bps: Decimal,
    #[arg(long, value_enum, default_value = "adopted")]
    pub slippage_model: SlippageArg,
    /// Entry cap as a percent of 20-day ADV under the adopted model (docs/slippage-model.md: 1).
    /// A sensitivity knob only: the impact term still prices every order above 0.05% of ADV.
    #[arg(long, default_value = "1")]
    pub adv_cap_pct: Decimal,
    /// Adjusted prices skip the Rs 0.05 tick rounding (review-backtest-03, section 3).
    #[arg(long, value_enum, default_value = "adjusted")]
    pub price_basis: PriceBasis,
    /// Stress multiplier on all slippage (protocol cost stress: 1, 2, 3).
    #[arg(long, default_value = "1")]
    pub slippage_mult: Decimal,
    /// Override the risk gate's per-position notional cap (percent of equity), e.g. 100 for one
    /// diversified index fund. Default: the gate's 20%.
    #[arg(long)]
    pub max_notional_pct: Option<Decimal>,
    /// Override the risk gate's max open positions (portfolio mode: across all symbols).
    #[arg(long)]
    pub max_positions: Option<usize>,
    /// Which same-day entry signals get the free slots and cash.
    #[arg(long, value_enum, default_value = "alphabetical")]
    pub priority: PriorityArg,
    /// Seed for `--priority random`.
    #[arg(long, requires = "priority")]
    pub seed: Option<u64>,
    /// Benchmark index CSV (date,open,high,low,close,volume), e.g. data/index/NSEI.csv.
    #[arg(long)]
    pub benchmark: Option<PathBuf>,
    /// Append a one-line run summary (CSV) here.
    #[arg(long)]
    pub summary_csv: Option<PathBuf>,
    /// Free-text label for the summary row.
    #[arg(long, default_value = "")]
    pub label: String,
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
    if args.seed.is_some() != (args.priority == PriorityArg::Random) {
        return Err(CliError::Usage("--seed goes with --priority random, and random needs a --seed".into()));
    }
    if args.slippage_mult <= Decimal::ZERO {
        return Err(CliError::Usage("--slippage-mult must be positive".into()));
    }
    if args.strategy.is_rebalancer() && args.mode == Mode::PerSymbol {
        return Err(CliError::Usage("rebalancing strategies run in --mode portfolio only".into()));
    }
    if args.adv_cap_pct <= Decimal::ZERO {
        return Err(CliError::Usage("--adv-cap-pct must be positive".into()));
    }
    if args.max_notional_pct.is_some_and(|p| p <= Decimal::ZERO || p > Decimal::ONE_HUNDRED) {
        return Err(CliError::Usage("--max-notional-pct must be in (0, 100] (no leverage)".into()));
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
            Some(Command::Backtest(a)) => *a,
            other => panic!("not a backtest: {other:?}"),
        }
    }

    #[test]
    fn defaults_match_the_brief() {
        let a = args(&["--start", "2012-01-01", "--end", "2021-12-31"]);
        assert_eq!((a.cash, a.risk_pct, a.slippage_bps), (Decimal::from(1_000_000), Decimal::new(5, 1), Decimal::from(5)));
        assert_eq!((a.mode, a.slippage_model, a.priority), (Mode::Portfolio, SlippageArg::Adopted, PriorityArg::Alphabetical));
        assert_eq!((a.slippage_mult, a.max_positions), (Decimal::ONE, None));
        assert_eq!((a.price_basis, a.adv_cap_pct), (PriceBasis::Adjusted, Decimal::ONE));
        assert!(check_holdout(&a).is_ok());
    }

    #[test]
    fn random_priority_and_seed_go_together() {
        let dates = ["--start", "2012-01-01", "--end", "2013-01-01"];
        let with = |extra: &[&str]| check_holdout(&args(&[&dates[..], extra].concat())).is_ok();
        assert!(with(&["--priority", "random", "--seed", "3"]));
        assert!(!with(&["--priority", "random"]));
        assert!(!with(&["--priority", "alphabetical", "--seed", "3"]));
        assert!(!with(&["--slippage-mult", "0"]));
    }

    #[test]
    fn rebalancers_are_portfolio_only_and_unlevered() {
        let dates = ["--start", "2012-01-01", "--end", "2013-01-01"];
        let parse = |extra: &[&str]| {
            let base = ["tq", "backtest", "--data-dir", "d", "--symbols", "NIFTYBEES"];
            match Cli::try_parse_from(base.iter().chain(&dates).chain(extra)).map(|c| c.command) {
                Ok(Some(Command::Backtest(a))) => check_holdout(&a).is_ok(),
                _ => false,
            }
        };
        assert!(parse(&["--strategy", "index-trend-filter", "--max-notional-pct", "100"]));
        assert!(!parse(&["--strategy", "buy-hold", "--mode", "per-symbol"]));
        assert!(!parse(&["--strategy", "buy-hold", "--max-notional-pct", "150"]));
        assert!(StrategyName::MomentumRotation.is_rebalancer() && !StrategyName::Donchian.is_rebalancer());
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

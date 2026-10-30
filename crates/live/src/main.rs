//! `tq-live`: intraday PAPER trading. `--replay <dir|file>` replays stored bars; `--live
//! binance:SYMBOL` runs on Binance's public kline stream. Never places orders anywhere.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use clap::Parser;
use rust_decimal::Decimal;
use serde_json::json;
use tq_core::Symbol;
use tq_costs::{CostSchedule, CryptoSpotFees, Liquidity, Tds194S};
use tq_data::IntradayCsv;
use tq_feeds::binance::{BinanceLive, LiveConfig, SystemClock, TungsteniteWs, UreqRest};
use tq_feeds::ReplayStream;
use tq_live::journal::{dec, Journal};
use tq_live::runner::{run, RunOutcome, Runner, RunnerConfig};
use tq_live::session::Session;
use tq_live::stats::{summarize, PerTradeTax};
use tq_live::strategies::{MomentumBurst, OpeningRangeBreakout, VwapReversion};
use tq_live::{LiveError, HOLDOUT_START_MS};
use tq_risk::RiskLimits;
use tq_strategy::Strategy;

/// Binance BTCUSDT/ETHUSDT spot step sizes are 0.00001 / 0.0001; 0.00001 serves both.
const CRYPTO_LOT: &str = "0.00001";
const RESEARCH_LIMIT_PCT: Decimal = Decimal::ONE_HUNDRED;
const VDA_TAX_PCT: Decimal = Decimal::from_parts(30, 0, 0, false, 0);

#[derive(Parser, Debug)]
#[command(name = "tq-live", about = "Intraday paper trading: historical replay or live public feed. Paper only.")]
struct Args {
    /// Directory of intraday CSVs (or one CSV file) to replay.
    #[arg(long, conflicts_with = "live", required_unless_present = "live")]
    replay: Option<PathBuf>,
    /// Live public feed, e.g. binance:BTCUSDT.
    #[arg(long)]
    live: Option<String>,
    /// vwap | momentum | orb (orb needs --session nse).
    #[arg(long, default_value = "vwap")]
    strategy: String,
    /// crypto | nse.
    #[arg(long, default_value = "crypto")]
    session: String,
    /// Symbol name for a replay (default: the file or directory name).
    #[arg(long)]
    symbol: Option<String>,
    /// Replay bar interval in ms (60000 = 1m, 300000 = 5m).
    #[arg(long, default_value_t = 60_000)]
    interval_ms: i64,
    /// Replay start date, YYYY-MM-DD (UTC), inclusive.
    #[arg(long)]
    from: Option<String>,
    /// Replay end date, YYYY-MM-DD (UTC), exclusive. Crypto: at most 2024-01-01 (holdout).
    #[arg(long)]
    to: Option<String>,
    /// Base-asset quantity per traded unit (default 0.00001 crypto, 1 NSE).
    #[arg(long)]
    lot: Option<Decimal>,
    #[arg(long, default_value = "10000")]
    cash: Decimal,
    /// Crypto taker fee in percent (Binance VIP 0 = 0.1; with BNB discount 0.075).
    #[arg(long, default_value = "0.1")]
    fee_pct: Decimal,
    #[arg(long, default_value = "5")]
    slippage_bps: Decimal,
    /// Withhold Indian 1% TDS (s.194S) on crypto sells as a receivable.
    #[arg(long)]
    tds: bool,
    /// Research mode: disable the drawdown kill switch and the daily-loss stop, so the whole
    /// period is measured. Never use for live paper trading.
    #[arg(long, conflicts_with = "live")]
    research: bool,
    /// Journal path (JSONL, appended). Default data/journal/<mode>-<symbol>-<strategy>-<time>.jsonl.
    #[arg(long)]
    journal: Option<PathBuf>,
    /// Equity mark every N bars (default 1 live, 60 replay).
    #[arg(long)]
    mark_every: Option<u64>,
    /// Live: stop after N minutes (default: run until killed).
    #[arg(long)]
    minutes: Option<u64>,
    #[arg(long, default_value_t = 15_000)]
    max_stale_ms: i64,
    /// Also write the summary JSON here.
    #[arg(long)]
    summary: Option<PathBuf>,
}

enum Mode {
    Replay(ReplayStream),
    Live(LiveConfig),
}

fn cfg_err(msg: impl Into<String>) -> LiveError {
    LiveError::Config(msg.into())
}

fn date_ms(s: &str) -> Result<i64, LiveError> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| cfg_err(format!("date must be YYYY-MM-DD: {s}")))?;
    Ok(d.and_hms_opt(0, 0, 0).map_or(0, |t| t.and_utc().timestamp_millis()))
}

fn replay_mode(args: &Args, path: &Path, session: Session) -> Result<(Mode, Symbol), LiveError> {
    let from = args.from.as_deref().map(date_ms).transpose()?.unwrap_or(i64::MIN);
    let cap = if session == Session::Crypto24x7 { HOLDOUT_START_MS } else { i64::MAX };
    let to = args.to.as_deref().map(date_ms).transpose()?.unwrap_or(cap);
    if to > cap {
        return Err(cfg_err("--to is inside the 2024-2025 holdout; crypto replays must end by 2024-01-01"));
    }
    let loader = IntradayCsv::new(from, to);
    let bars = if path.is_file() { loader.load_file(path) } else { loader.load_dir(path) }.map_err(|e| cfg_err(e.to_string()))?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let symbol = Symbol(args.symbol.clone().unwrap_or(stem));
    eprintln!("replay: {} bars of {} from {}", bars.len(), symbol.0, path.display());
    Ok((Mode::Replay(ReplayStream::new(symbol.clone(), args.interval_ms, bars)?), symbol))
}

fn live_mode(spec: &str, session: Session) -> Result<(Mode, Symbol), LiveError> {
    let sym = spec.strip_prefix("binance:").ok_or_else(|| cfg_err("--live must look like binance:BTCUSDT"))?;
    if session != Session::Crypto24x7 {
        return Err(cfg_err("live NSE needs a paid Kite Connect feed; only binance:<SYMBOL> is supported"));
    }
    let lc = LiveConfig::binance_1m(sym)?;
    let symbol = Symbol(lc.symbol.clone());
    Ok((Mode::Live(lc), symbol))
}

fn runner_config(args: &Args, session: Session, symbol: Symbol, live: bool) -> Result<RunnerConfig, LiveError> {
    let crypto = session == Session::Crypto24x7;
    let default_lot = if crypto { CRYPTO_LOT.parse().map_err(|_| cfg_err("bad default lot"))? } else { Decimal::ONE };
    let costs = if crypto {
        CryptoSpotFees { maker_pct: args.fee_pct, taker_pct: args.fee_pct }.schedule(Liquidity::Taker)
    } else {
        CostSchedule::zerodha_nse_intraday()
    };
    let mut limits = RiskLimits::default();
    if args.research {
        limits.max_drawdown_pct = RESEARCH_LIMIT_PCT;
        limits.max_daily_loss_pct = RESEARCH_LIMIT_PCT;
    }
    Ok(RunnerConfig {
        symbol,
        lot: args.lot.unwrap_or(default_lot),
        initial_cash: args.cash,
        slippage_bps: args.slippage_bps,
        costs,
        limits,
        session,
        max_stale_ms: args.max_stale_ms,
        mark_every: args.mark_every.unwrap_or(if live { 1 } else { 60 }),
        tds: (crypto && args.tds).then(Tds194S::default),
        holdout_from: (crypto && !live).then_some(HOLDOUT_START_MS),
    })
}

fn wall_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

fn journal_path(args: &Args, symbol: &Symbol, live: bool) -> PathBuf {
    args.journal.clone().unwrap_or_else(|| {
        let mode = if live { "live" } else { "replay" };
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        PathBuf::from(format!("data/journal/{mode}-{}-{}-{secs}.jsonl", symbol.0, args.strategy))
    })
}

fn execute<S: Strategy>(strategy: S, mode: Mode, cfg: RunnerConfig, mut journal: Journal, minutes: Option<u64>) -> Result<RunOutcome, LiveError> {
    let start = json!({
        "paper_only": true, "strategy": strategy.name(), "symbol": cfg.symbol.0, "session": format!("{:?}", cfg.session),
        "lot": dec(cfg.lot), "cash": dec(cfg.initial_cash), "slippage_bps": dec(cfg.slippage_bps),
        "fee_pct": dec(cfg.costs.brokerage_pct), "tds": cfg.tds.is_some(), "max_drawdown_pct": dec(cfg.limits.max_drawdown_pct),
        "max_daily_loss_pct": dec(cfg.limits.max_daily_loss_pct),
    });
    journal.record("start", wall_ms(), start)?;
    let mut runner = Runner::new(cfg, strategy, journal)?;
    match mode {
        Mode::Replay(mut feed) => run(&mut runner, &mut feed, None)?,
        Mode::Live(lc) => {
            let deadline = minutes.map(|m| Instant::now() + Duration::from_secs(m * 60));
            let mut feed = BinanceLive::new(lc, TungsteniteWs, UreqRest::default(), SystemClock);
            run(&mut runner, &mut feed, deadline)?;
        }
    }
    runner.journal().record("end", wall_ms(), json!({}))?;
    runner.finish()
}

fn dispatch(args: &Args, mode: Mode, cfg: RunnerConfig, journal: Journal) -> Result<RunOutcome, LiveError> {
    let session = cfg.session;
    let none = |what: &str| cfg_err(format!("cannot build {what}"));
    match args.strategy.as_str() {
        "vwap" => execute(VwapReversion::new(session).ok_or_else(|| none("vwap"))?, mode, cfg, journal, args.minutes),
        "momentum" => execute(MomentumBurst::new().ok_or_else(|| none("momentum"))?, mode, cfg, journal, args.minutes),
        "orb" if session == Session::Nse => execute(OpeningRangeBreakout::new(session), mode, cfg, journal, args.minutes),
        "orb" => Err(cfg_err("orb is defined on the NSE session only (--session nse)")),
        other => Err(cfg_err(format!("unknown strategy {other}; use vwap, momentum or orb"))),
    }
}

fn main_inner(args: Args) -> Result<(), LiveError> {
    let session = Session::parse(&args.session).ok_or_else(|| cfg_err("--session must be crypto or nse"))?;
    let live = args.live.is_some();
    let (mode, symbol) = match (&args.replay, &args.live) {
        (Some(p), _) => replay_mode(&args, p, session)?,
        (None, Some(spec)) => live_mode(spec, session)?,
        (None, None) => return Err(cfg_err("give --replay <dir> or --live binance:SYMBOL")),
    };
    let path = journal_path(&args, &symbol, live);
    let journal = Journal::open(&path, live, live)?;
    eprintln!("journal: {}", path.display());
    let cfg = runner_config(&args, session, symbol, live)?;
    let crypto = session == Session::Crypto24x7;
    let outcome = dispatch(&args, mode, cfg, journal)?;
    let tax = crypto.then_some(PerTradeTax { rate_pct: VDA_TAX_PCT });
    let summary = summarize(&outcome, tax, crypto.then_some(Decimal::ONE));
    let text = serde_json::to_string_pretty(&summary).map_err(|e| cfg_err(e.to_string()))?;
    println!("{text}");
    if let Some(p) = &args.summary {
        std::fs::write(p, &text)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match main_inner(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

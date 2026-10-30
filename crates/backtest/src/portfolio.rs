//! Shared-account portfolio runs: every symbol on one date calendar, one cash balance, one
//! `AccountState`. The gate's position cap, daily loss limit and kill switch therefore bind
//! across the whole portfolio, unlike independent per-symbol accounts.

use std::collections::{BTreeMap, HashSet};

use tq_broker::Broker;
use tq_core::{Bar, Symbol, Timestamp};
use tq_risk::{AccountState, RiskGate};
use tq_strategy::Strategy;

use crate::engine::{DayBar, Engine, EntryWindows};
use crate::{BacktestError, RunConfig, RunReport};

/// One symbol's bars (oldest first) and the fill-time windows in which it may be entered
/// (`None`: any time). Exits are never restricted.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub symbol: Symbol,
    pub bars: Vec<Bar>,
    pub entry_windows: Option<Vec<(Timestamp, Timestamp)>>,
}

/// Runs one strategy instance over all `series` in one account. The calendar is the union of
/// every series' bar timestamps; a symbol without a bar on a day is simply not stepped (its
/// position keeps the last close). Within a day, symbols are visited in ticker order and
/// competing entries follow `config.priority`. `config.entry_windows` must be `None`: windows
/// belong to each [`Series`].
pub fn run_portfolio<S: Strategy, B: Broker>(
    strategy: &mut S,
    broker: &mut B,
    gate: &RiskGate,
    config: &RunConfig,
    series: &[Series],
) -> Result<RunReport, BacktestError> {
    config.validate()?;
    if config.entry_windows.is_some() {
        return Err(BacktestError::InvalidConfig("portfolio runs take entry windows per series"));
    }
    validate_series(series)?;
    let acct = AccountState::fresh(config.initial_cash).map_err(|_| BacktestError::NonPositiveCash)?;
    let windows: EntryWindows =
        series.iter().filter_map(|s| s.entry_windows.clone().map(|w| (s.symbol.clone(), w))).collect();
    let mut engine = Engine::new(strategy, broker, gate, config, acct, windows);
    for (ts, day) in calendar(series) {
        engine.step(ts, &day);
    }
    Ok(engine.finish())
}

fn validate_series(series: &[Series]) -> Result<(), BacktestError> {
    let mut seen = HashSet::new();
    for s in series {
        if !seen.insert(&s.symbol) {
            return Err(BacktestError::InvalidConfig("duplicate symbol in portfolio"));
        }
        if s.bars.windows(2).any(|w| w[0].ts >= w[1].ts) {
            return Err(BacktestError::BarsOutOfOrder);
        }
    }
    Ok(())
}

/// Each timestamp's bars, symbols in ticker order.
fn calendar(series: &[Series]) -> BTreeMap<Timestamp, Vec<DayBar<'_>>> {
    let mut sorted: Vec<&Series> = series.iter().collect();
    sorted.sort_by(|a, b| a.symbol.0.cmp(&b.symbol.0));
    let mut days: BTreeMap<Timestamp, Vec<DayBar>> = BTreeMap::new();
    for s in sorted {
        let last = s.bars.len().saturating_sub(1);
        for (i, bar) in s.bars.iter().enumerate() {
            days.entry(bar.ts).or_default().push(DayBar { symbol: &s.symbol, bar, has_next: i < last });
        }
    }
    days
}

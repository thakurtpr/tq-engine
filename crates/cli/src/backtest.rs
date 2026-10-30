//! Runs one strategy either as a shared-account portfolio or per symbol (fresh account each).

use std::collections::BTreeSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use tq_backtest::{run, run_portfolio, Priority, RunConfig, RunReport, Series, SlippageModel, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, Symbol, Timestamp};
use tq_data::{ist_close_ts, ist_date, BarSource, CsvDaily};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategies::{DonchianBreakout, TrendPullback};
use tq_strategy::Strategy;

use crate::args::{BacktestArgs, PriceBasis, PriorityArg, SlippageArg, StrategyName};
use crate::stats::SymbolResult;
use crate::CliError;

/// Fill-time windows in which entries are allowed; `None` = always.
pub type EntryWindows = Option<Vec<(Timestamp, Timestamp)>>;

/// Everything shared by the runs of one command.
pub struct Setup {
    pub gate: RiskGate,
    pub config: RunConfig,
    source: CsvDaily,
    end_ts: i64,
    strategy: StrategyName,
}

impl Setup {
    pub fn from_args(args: &BacktestArgs) -> Result<Self, CliError> {
        let defaults = RiskLimits::default();
        let limits = RiskLimits {
            max_drawdown_pct: args.max_drawdown_pct.unwrap_or(defaults.max_drawdown_pct),
            max_open_positions: args.max_positions.unwrap_or(defaults.max_open_positions),
            ..defaults
        };
        if args.risk_pct <= Decimal::ZERO || args.risk_pct > limits.max_risk_per_trade_pct {
            let max = limits.max_risk_per_trade_pct;
            return Err(CliError::Usage(format!("--risk-pct must be in (0, {max}] (the gate's per-trade limit)")));
        }
        let config = RunConfig {
            initial_cash: args.cash,
            sizing: Sizing::RiskPct(args.risk_pct),
            slippage_bps: args.slippage_bps,
            slippage_model: (args.slippage_model == SlippageArg::Adopted).then(|| slippage_model(args.price_basis)),
            slippage_mult: args.slippage_mult,
            priority: match (args.priority, args.seed) {
                (PriorityArg::Random, Some(seed)) => Priority::Random { seed },
                _ => Priority::Alphabetical,
            },
            trade_from: ist_close_ts(args.start),
            entry_windows: None,
        };
        Ok(Self {
            gate: RiskGate::new(limits),
            config,
            source: CsvDaily::new(&args.data_dir),
            end_ts: ist_close_ts(args.end),
            strategy: args.strategy,
        })
    }

    /// Each ticker in its own account; `Some(windows)` restricts when its entries may fill.
    pub fn run_all(&self, plan: &[(String, EntryWindows)]) -> Vec<SymbolResult> {
        plan.iter()
            .map(|(s, windows)| SymbolResult { symbol: s.clone(), outcome: self.run_symbol(s, windows) })
            .collect()
    }

    /// All tickers in ONE account. Tickers without data are returned as not run.
    pub fn run_portfolio(&self, plan: &[(String, EntryWindows)]) -> Result<(RunReport, Vec<(String, String)>), CliError> {
        let mut series = Vec::new();
        let mut skipped = Vec::new();
        for (ticker, windows) in plan {
            let symbol = Symbol(format!("NSE:{ticker}"));
            match self.load(&symbol) {
                Ok(bars) => series.push(Series { symbol, bars, entry_windows: windows.clone() }),
                Err(why) => skipped.push((ticker.clone(), why)),
            }
        }
        let report = match self.strategy {
            StrategyName::TrendPullback => self.portfolio_with(TrendPullback::new(Default::default()), &series),
            StrategyName::Donchian => self.portfolio_with(DonchianBreakout::new(Default::default()), &series),
        }?;
        Ok((report, skipped))
    }

    fn portfolio_with<S: Strategy, E: std::fmt::Display>(&self, strategy: Result<S, E>, series: &[Series]) -> Result<RunReport, CliError> {
        let mut strategy = strategy.map_err(|e| CliError::Usage(e.to_string()))?;
        run_portfolio(&mut strategy, &mut Paper::default(), &self.gate, &self.config, series).map_err(|e| CliError::Usage(e.to_string()))
    }

    /// IST dates of `ticker`'s bars up to `--end`; `None` if the file is missing or unreadable.
    pub fn bar_dates(&self, ticker: &str) -> Option<BTreeSet<NaiveDate>> {
        let bars = self.source.bars(&Symbol(format!("NSE:{ticker}"))).ok()?;
        Some(bars.iter().take_while(|b| b.ts <= self.end_ts).filter_map(|b| ist_date(b.ts)).collect())
    }

    fn run_symbol(&self, ticker: &str, windows: &EntryWindows) -> Result<RunReport, String> {
        let symbol = Symbol(format!("NSE:{ticker}"));
        let bars = self.load(&symbol)?;
        let config = RunConfig { entry_windows: windows.clone(), ..self.config.clone() };
        match self.strategy {
            StrategyName::TrendPullback => {
                let s = TrendPullback::new(Default::default()).map_err(|e| e.to_string())?;
                run_with(s, &self.gate, &config, &symbol, &bars)
            }
            StrategyName::Donchian => {
                let s = DonchianBreakout::new(Default::default()).map_err(|e| e.to_string())?;
                run_with(s, &self.gate, &config, &symbol, &bars)
            }
        }
    }

    /// Bars up to `--end` only: later data is never loaded into the run.
    fn load(&self, symbol: &Symbol) -> Result<Vec<Bar>, String> {
        let bars = self.source.bars(symbol).map_err(|e| e.to_string())?;
        let bars: Vec<Bar> = bars.into_iter().take_while(|b| b.ts <= self.end_ts).collect();
        if !bars.iter().any(|b| b.ts >= self.config.trade_from) {
            return Err("no bars inside the test window".into());
        }
        Ok(bars)
    }
}

/// The adopted model; the Rs 0.05 tick only applies to unadjusted (on-grid) prices.
pub fn slippage_model(basis: PriceBasis) -> SlippageModel {
    match basis {
        PriceBasis::Unadjusted => SlippageModel::default(),
        PriceBasis::Adjusted => SlippageModel { tick: Decimal::ZERO, ..SlippageModel::default() },
    }
}

fn run_with<S: Strategy>(mut strategy: S, gate: &RiskGate, config: &RunConfig, symbol: &Symbol, bars: &[Bar]) -> Result<RunReport, String> {
    let mut broker = Paper::default();
    run(&mut strategy, &mut broker, gate, config, symbol, bars).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_rounding_only_on_unadjusted_prices() {
        assert_eq!(slippage_model(PriceBasis::Adjusted).tick, Decimal::ZERO);
        assert_eq!(slippage_model(PriceBasis::Unadjusted).tick, Decimal::new(5, 2));
    }
}

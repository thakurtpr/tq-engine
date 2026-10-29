//! Runs one strategy over each symbol independently: fresh cash, broker and strategy per symbol.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use tq_backtest::{run, RunConfig, RunReport, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, Symbol, Timestamp};
use tq_data::{ist_close_ts, ist_date, BarSource, CsvDaily};
use tq_risk::{RiskGate, RiskLimits};
use tq_strategies::{DonchianBreakout, TrendPullback};
use tq_strategy::Strategy;

use crate::args::{BacktestArgs, StrategyName};
use crate::stats::SymbolResult;
use crate::CliError;

/// Everything shared by the per-symbol runs.
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

    /// Runs each ticker; `Some(windows)` restricts when its entries may fill (index membership).
    pub fn run_all(&self, plan: &[(String, EntryWindows)]) -> Vec<SymbolResult> {
        plan.iter()
            .map(|(s, windows)| SymbolResult { symbol: s.clone(), outcome: self.run_symbol(s, windows) })
            .collect()
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

/// Fill-time windows in which entries are allowed; `None` = always.
pub type EntryWindows = Option<Vec<(Timestamp, Timestamp)>>;

fn run_with<S: Strategy>(mut strategy: S, gate: &RiskGate, config: &RunConfig, symbol: &Symbol, bars: &[Bar]) -> Result<RunReport, String> {
    let mut broker = Paper::default();
    run(&mut strategy, &mut broker, gate, config, symbol, bars).map_err(|e| e.to_string())
}

//! Runs one strategy over each symbol independently: fresh cash, broker and strategy per symbol.

use rust_decimal::Decimal;
use tq_backtest::{run, RunConfig, RunReport, Sizing};
use tq_broker::Paper;
use tq_core::{Bar, Symbol};
use tq_data::{ist_close_ts, BarSource, CsvDaily};
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
        };
        Ok(Self {
            gate: RiskGate::new(limits),
            config,
            source: CsvDaily::new(&args.data_dir),
            end_ts: ist_close_ts(args.end),
            strategy: args.strategy,
        })
    }

    pub fn run_all(&self, symbols: &[String]) -> Vec<SymbolResult> {
        symbols
            .iter()
            .map(|s| SymbolResult { symbol: s.clone(), outcome: self.run_symbol(s) })
            .collect()
    }

    fn run_symbol(&self, ticker: &str) -> Result<RunReport, String> {
        let symbol = Symbol(format!("NSE:{ticker}"));
        let bars = self.load(&symbol)?;
        match self.strategy {
            StrategyName::TrendPullback => {
                let s = TrendPullback::new(Default::default()).map_err(|e| e.to_string())?;
                self.run_with(s, &symbol, &bars)
            }
            StrategyName::Donchian => {
                let s = DonchianBreakout::new(Default::default()).map_err(|e| e.to_string())?;
                self.run_with(s, &symbol, &bars)
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

    fn run_with<S: Strategy>(&self, mut strategy: S, symbol: &Symbol, bars: &[Bar]) -> Result<RunReport, String> {
        let mut broker = Paper::default();
        run(&mut strategy, &mut broker, &self.gate, &self.config, symbol, bars).map_err(|e| e.to_string())
    }
}

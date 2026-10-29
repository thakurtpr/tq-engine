use std::process::ExitCode;
use tq_risk::RiskLimits;

fn main() -> ExitCode {
    let limits = RiskLimits::default();
    println!("tq engine: paper/backtest only");
    println!("risk limits: {limits:?}");
    match limits.ensure_live_ready() {
        Ok(()) => println!("live: limits confirmed (live broker not implemented)"),
        Err(e) => println!("live: BLOCKED ({e})"),
    }
    ExitCode::SUCCESS
}

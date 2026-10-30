#!/bin/bash
# Reproduces every run behind reports/backtest-03b.md. Run from the engine root after
# `cargo build --release`. Development window only; the holdout guard stays on.
set -euo pipefail
OUT=reports/backtest-03b
U="--data-dir data/daily-pit --universe-file data-ref/nifty50_members_by_date.csv \
   --aliases data-ref/nifty50_price_aliases.csv --start 2012-01-01 --end 2021-12-31 --risk-pct 0.5 \
   --benchmark data/index/NSEI.csv"
rm -f $OUT/summary.csv $OUT/seeds.csv $OUT/scaling.csv
run() { ./target/release/tq backtest "$@" 2>/dev/null; }
# 1. Grid: adjusted prices (no tick), kill switch off (edge measurement), slippage 1x/2x/3x.
for st in donchian trend-pullback; do for mp in 5 10; do for m in 1 2 3; do
  run --strategy $st $U --max-positions $mp --max-drawdown-pct 100 --slippage-mult $m \
      --summary-csv $OUT/summary.csv --label $st-p$mp-killoff-slip${m}x --out $OUT/$st-p$mp-killoff-slip${m}x.md
done; done; done
# 2. Operating runs with the user's 10% kill switch (the new default).
for st in donchian trend-pullback; do for mp in 5 10; do
  run --strategy $st $U --max-positions $mp --summary-csv $OUT/summary.csv --label $st-p$mp-kill10-slip1x \
      --out $OUT/$st-p$mp-kill10-slip1x.md
done; done
# 3. Tick comparison (backtest-03 convention): Rs 0.05 tick on adjusted prices.
run --strategy donchian $U --max-positions 5 --max-drawdown-pct 100 --price-basis unadjusted \
    --summary-csv $OUT/summary.csv --label donchian-p5-killoff-slip1x-tick
# 4. Random-priority seeds 0-19 for EVERY configuration (rule 0.6), kill switch off, 1x.
for st in donchian trend-pullback; do for mp in 5 10; do for seed in $(seq 0 19); do
  run --strategy $st $U --max-positions $mp --max-drawdown-pct 100 --priority random --seed $seed \
      --summary-csv $OUT/seeds.csv --label $st-p$mp-killoff-seed$seed
done; done; done
# 5. Capital scaling: Rs 10 lakh, 1 crore, 10 crore (Donchian 5 and 10, kill off, 1x).
for cash in 1000000 10000000 100000000; do for mp in 5 10; do
  run --strategy donchian $U --max-positions $mp --max-drawdown-pct 100 --cash $cash \
      --summary-csv $OUT/scaling.csv --label donchian-p$mp-cash$cash --out $OUT/donchian-p$mp-cash$cash.md
done; done

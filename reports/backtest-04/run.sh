#!/bin/bash
# Reproduces every run behind reports/backtest-04.md (pre-registered rebalancers).
# Run from the engine root after `cargo build --release`. Development window only.
# Data: data/etf/NIFTYBEES.csv = scripts/fetch_daily.py NIFTYBEES --start 2010-01-01
# --end 2021-12-31 --out data/etf, minus the two Yahoo rows at 1/10 price (2019-12-19, 2019-12-20);
# the raw download is kept as data/etf/NIFTYBEES-raw.csv.
set -euo pipefail
OUT=reports/backtest-04
W="--start 2012-01-01 --end 2021-12-31 --benchmark data/etf/NIFTYBEES.csv"
PIT="--data-dir data/daily-pit --universe-file data-ref/nifty50_members_by_date.csv \
     --aliases data-ref/nifty50_price_aliases.csv --max-positions 10"
ETF="--data-dir data/etf --symbols NIFTYBEES --max-notional-pct 100"
rm -f $OUT/summary.csv $OUT/seeds.csv
run() { ./target/release/tq backtest "$@" 2>/dev/null; }
for st in momentum-rotation index-trend-filter buy-hold; do
  [ $st = momentum-rotation ] && D="$PIT" || D="$ETF"
  # Edge measurement: kill switch off, slippage 1x/2x/3x.
  for m in 1 2 3; do
    run --strategy $st $D $W --max-drawdown-pct 100 --slippage-mult $m \
        --summary-csv $OUT/summary.csv --label $st-killoff-slip${m}x --out $OUT/$st-killoff-slip${m}x.md
  done
  # Operating run with the user's 10% kill switch (default).
  run --strategy $st $D $W --summary-csv $OUT/summary.csv --label $st-kill10-slip1x --out $OUT/$st-kill10-slip1x.md
done
# Sensitivity: ETF strategies without the stock-oriented 1% ADV entry cap (impact still charged).
for st in index-trend-filter buy-hold; do
  run --strategy $st $ETF $W --max-drawdown-pct 100 --adv-cap-pct 100 \
      --summary-csv $OUT/summary.csv --label $st-killoff-slip1x-advcap100 --out $OUT/$st-killoff-slip1x-advcap100.md
done
# Rule 0.6: random buy order on rebalance days, 20 seeds (momentum is the only multi-name book).
for seed in $(seq 0 19); do
  run --strategy momentum-rotation $PIT $W --max-drawdown-pct 100 --priority random --seed $seed \
      --summary-csv $OUT/seeds.csv --label momentum-rotation-killoff-seed$seed
done
# Capital scaling for the momentum book: Rs 1 crore and Rs 10 crore.
for cash in 10000000 100000000; do
  run --strategy momentum-rotation $PIT $W --max-drawdown-pct 100 --cash $cash \
      --summary-csv $OUT/summary.csv --label momentum-rotation-killoff-cash$cash --out $OUT/momentum-rotation-killoff-cash$cash.md
done

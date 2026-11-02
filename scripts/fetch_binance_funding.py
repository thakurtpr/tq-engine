#!/usr/bin/env python3
"""Download free Binance USD-M perp funding rates plus 1h spot and perp klines for carry research.

Public archive at data.binance.vision (no account, no keys). For each symbol and month it fetches:
  futures/um/monthly/fundingRate/<SYM>/<SYM>-fundingRate-YYYY-MM.zip -> data/funding/<SYM>/
  futures/um/monthly/klines/<SYM>/1h/<SYM>-1h-YYYY-MM.zip            -> data/perp-1h/<SYM>/
  futures/um/monthly/markPriceKlines/<SYM>/1h/<SYM>-1h-YYYY-MM.zip   -> data/mark-1h/<SYM>/
  spot/monthly/klines/<SYM>/1h/<SYM>-1h-YYYY-MM.zip                  -> data/spot-1h/<SYM>/
Each zip's SHA-256 is checked against its .CHECKSUM before the CSV is extracted. Months already on
disk are skipped.

The funding CSV HAS a header: calc_time,funding_interval_hours,last_funding_rate (calc_time in epoch
ms; the rate is per interval, e.g. 0.0001 = 1 bp per 8h). Kline CSVs follow fetch_binance_klines.py.

Standard library only; shares its download and verify helpers with fetch_binance_klines.py.
Exit code 1 if any file failed (others still complete).
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

from fetch_binance_klines import (
    SYMBOL_RE,
    FetchError,
    expected_sha256,
    extract_csv,
    http_get,
    months,
    parse_month,
    write_atomic,
)

ARCHIVE = "https://data.binance.vision/data"
DEFAULT_SYMBOLS = ["BTCUSDT", "ETHUSDT"]
DEFAULT_START = "2022-01"
DEFAULT_END = "2025-12"
DATA_ROOT = Path(__file__).resolve().parent.parent / "data"

# (output subdir, archive path template, file stem template)
DATASETS = [
    ("funding", "futures/um/monthly/fundingRate/{sym}", "{sym}-fundingRate-{month}"),
    ("perp-1h", "futures/um/monthly/klines/{sym}/1h", "{sym}-1h-{month}"),
    # WHY: Binance liquidates on MARK price; last-price highs carry wicks (ETH 2020-03-13: 323 vs 119.5).
    ("mark-1h", "futures/um/monthly/markPriceKlines/{sym}/1h", "{sym}-1h-{month}"),
    ("spot-1h", "spot/monthly/klines/{sym}/1h", "{sym}-1h-{month}"),
]


def fetch_one(url_dir: str, stem: str, target: Path) -> str:
    if target.exists():
        return "skipped (exists)"
    url = f"{ARCHIVE}/{url_dir}/{stem}.zip"
    want = expected_sha256(http_get(url + ".CHECKSUM").decode("ascii", "replace"), f"{stem}.zip")
    blob = http_get(url)
    got = hashlib.sha256(blob).hexdigest()
    if got != want:
        raise FetchError(f"SHA-256 mismatch for {stem}.zip: got {got}, want {want}")
    write_atomic(target, extract_csv(blob, f"{stem}.csv"))
    return f"ok sha256={got[:12]}"


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("symbols", nargs="*", default=DEFAULT_SYMBOLS, help="Binance USD-M perp symbols")
    p.add_argument("--start", default=DEFAULT_START, help=f"first month YYYY-MM (default {DEFAULT_START})")
    p.add_argument("--end", default=DEFAULT_END, help=f"last month YYYY-MM (default {DEFAULT_END})")
    p.add_argument("--out", type=Path, default=DATA_ROOT, help=f"data root (default {DATA_ROOT})")
    args = p.parse_args(argv)
    span = months(parse_month(args.start), parse_month(args.end))
    failures = 0
    for raw in args.symbols:
        symbol = raw.strip().upper()
        if not SYMBOL_RE.match(symbol):
            raise SystemExit(f"error: invalid symbol {raw!r}")
        for subdir, dir_tpl, stem_tpl in DATASETS:
            out_dir = args.out / subdir / symbol
            out_dir.mkdir(parents=True, exist_ok=True)
            for month in span:
                stem = stem_tpl.format(sym=symbol, month=month)
                try:
                    status = fetch_one(dir_tpl.format(sym=symbol), stem, out_dir / f"{stem}.csv")
                    print(f"{subdir} {symbol} {month}: {status}", flush=True)
                except FetchError as e:
                    failures += 1
                    print(f"{subdir} {symbol} {month}: FAILED: {e}", file=sys.stderr, flush=True)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

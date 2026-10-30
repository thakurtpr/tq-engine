#!/usr/bin/env python3
"""Download the maximum FREE intraday history Yahoo Finance serves for NSE stocks (yfinance).

Yahoo limits (as enforced by its chart API, not documented by NSE):
  * 5m bars: only the last 60 calendar days (about 40 sessions).
  * 1m bars: only the last ~30 calendar days, and at most 8 days per request, so 1m is
    fetched in 7-day chunks.
That is far too short for a statistically meaningful intraday backtest; see scripts/README.md.

Writes engine/data/intraday-nse/<interval>/<SYMBOL>.csv with header
`ts_ms,open,high,low,close,volume`, where ts_ms is the bar OPEN time in UTC epoch ms (read by
`tq_data::IntradayCsv`). Prices are NOT adjusted (intraday) and rounded to 2 dp. Bars that are
not yet closed are dropped. A ticker with invalid OHLC fails loudly; others continue.
"""

from __future__ import annotations

import argparse
import datetime as dt
import math
import os
import re
import sys
import tempfile
from pathlib import Path

HEADER = "ts_ms,open,high,low,close,volume"
DEFAULT_TICKERS = ["RELIANCE", "HDFCBANK", "ICICIBANK", "INFY", "TCS"]
DEFAULT_OUT = Path(__file__).resolve().parent.parent / "data" / "intraday-nse"
TICKER_RE = re.compile(r"^[A-Z0-9&_-]{1,20}$")
INTERVAL_MINUTES = {"1m": 1, "5m": 5}
FIVE_MIN_PERIOD = "60d"
ONE_MIN_LOOKBACK_DAYS = 29
ONE_MIN_CHUNK_DAYS = 7
PRICE_DECIMALS = 2


class FetchError(Exception):
    """A ticker could not be downloaded or failed validation."""


def download(yf, ticker: str, interval: str):
    sym = f"{ticker}.NS"
    if interval == "5m":
        return yf.download(sym, period=FIVE_MIN_PERIOD, interval="5m", auto_adjust=False,
                           progress=False, multi_level_index=False)
    import pandas as pd
    end = dt.datetime.now(dt.timezone.utc)
    start = end - dt.timedelta(days=ONE_MIN_LOOKBACK_DAYS)
    parts = []
    while start < end:
        stop = min(start + dt.timedelta(days=ONE_MIN_CHUNK_DAYS), end)
        parts.append(yf.download(sym, start=start, end=stop, interval="1m", auto_adjust=False,
                                 progress=False, multi_level_index=False))
        start = stop
    frames = [p for p in parts if p is not None and not p.empty]
    if not frames:
        return None
    df = pd.concat(frames)
    return df[~df.index.duplicated(keep="first")].sort_index()


def to_rows(df, interval: str) -> list[str]:
    if df is None or df.empty:
        raise FetchError("no data returned")
    now_ms = int(dt.datetime.now(dt.timezone.utc).timestamp() * 1000)
    bar_ms = INTERVAL_MINUTES[interval] * 60_000
    rows, last_ts = [], None
    for ts, r in df.iterrows():
        ts_ms = int(ts.tz_convert("UTC").timestamp() * 1000)
        if ts_ms + bar_ms > now_ms:
            continue  # bar still forming
        vals = [float(r[c]) for c in ("Open", "High", "Low", "Close", "Volume")]
        if any(math.isnan(v) for v in vals):
            continue  # Yahoo emits empty rows for halted minutes; there was no trade
        o, h, lo, c = (round(v, PRICE_DECIMALS) for v in vals[:4])
        if not (lo > 0 and lo <= min(o, c) and max(o, c) <= h):
            raise FetchError(f"invalid OHLC at {ts}: {o} {h} {lo} {c}")
        if last_ts is not None and ts_ms <= last_ts:
            raise FetchError(f"unordered timestamp at {ts}")
        last_ts = ts_ms
        rows.append(f"{ts_ms},{o:.2f},{h:.2f},{lo:.2f},{c:.2f},{int(vals[4])}")
    return rows


def write_atomic(path: Path, rows: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=".tmp-", suffix=".csv")
    try:
        with os.fdopen(fd, "w") as f:
            f.write(HEADER + "\n" + "\n".join(rows) + "\n")
        os.replace(tmp, path)
    except BaseException:
        Path(tmp).unlink(missing_ok=True)
        raise


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("tickers", nargs="*", default=DEFAULT_TICKERS, help="NSE tickers without .NS")
    p.add_argument("--intervals", default="5m,1m", help="comma list of 5m,1m (default both)")
    p.add_argument("--out", type=Path, default=DEFAULT_OUT, help=f"output root (default {DEFAULT_OUT})")
    args = p.parse_args(argv)
    import yfinance as yf
    failures = 0
    for interval in args.intervals.split(","):
        if interval not in INTERVAL_MINUTES:
            raise SystemExit(f"error: unsupported interval {interval!r}")
        for raw in args.tickers:
            t = raw.strip().upper().removesuffix(".NS")
            if not TICKER_RE.match(t):
                raise SystemExit(f"error: invalid ticker {raw!r}")
            try:
                rows = to_rows(download(yf, t, interval), interval)
                write_atomic(args.out / interval / f"{t}.csv", rows)
                first, last = rows[0].split(",")[0], rows[-1].split(",")[0]
                span = (int(last) - int(first)) / 86_400_000
                print(f"{t} {interval}: {len(rows)} bars over {span:.1f} days", flush=True)
            except Exception as e:  # yfinance raises many types; report and continue
                failures += 1
                print(f"{t} {interval}: FAILED: {e}", file=sys.stderr, flush=True)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Download free daily OHLCV for NSE tickers from Yahoo Finance (via yfinance).

Writes engine/data/daily/<SYMBOL>.csv with the header `date,open,high,low,close,volume`,
the format read by `tq_data::CsvDaily`. Prices are split/bonus adjusted (auto_adjust=True).
No API keys or accounts are needed. The data is unofficial: see scripts/README.md.
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

HEADER = "date,open,high,low,close,volume"
NSE_SUFFIX = ".NS"
PRICE_DECIMALS = 2
DEFAULT_START = "2015-01-01"
DEFAULT_OUT = Path(__file__).resolve().parent.parent / "data" / "daily"
# NSE tickers: letters, digits and a few symbols, e.g. M&M, BAJAJ-AUTO. A leading ^ marks a
# Yahoo index symbol (e.g. ^NSEI, the NIFTY 50 price index): fetched without the .NS suffix and
# saved without the ^ (NSEI.csv).
TICKER_RE = re.compile(r"^\^?[A-Z0-9&_-]{1,20}$")
INDEX_PREFIX = "^"


class FetchError(Exception):
    """A ticker could not be downloaded or failed validation."""


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("tickers", nargs="*", help="NSE tickers without suffix, e.g. RELIANCE INFY TCS")
    p.add_argument("--tickers-file", type=Path, help="file with one ticker per line (# comments allowed)")
    p.add_argument("--start", default=DEFAULT_START, help=f"first date YYYY-MM-DD (default {DEFAULT_START})")
    p.add_argument("--end", help="last date YYYY-MM-DD, inclusive (default: today)")
    p.add_argument("--out", type=Path, default=DEFAULT_OUT, help=f"output directory (default {DEFAULT_OUT})")
    return p.parse_args(argv)


def parse_date(text: str, flag: str) -> dt.date:
    try:
        return dt.date.fromisoformat(text)
    except ValueError as e:
        raise SystemExit(f"error: {flag} must be YYYY-MM-DD, got {text!r}") from e


def normalise_ticker(raw: str) -> str:
    t = raw.strip().upper()
    if t.endswith(NSE_SUFFIX):
        t = t[: -len(NSE_SUFFIX)]
    if not TICKER_RE.match(t):
        raise SystemExit(f"error: invalid NSE ticker {raw!r}")
    return t


def collect_tickers(args: argparse.Namespace) -> list[str]:
    raw = list(args.tickers)
    if args.tickers_file:
        try:
            lines = args.tickers_file.read_text().splitlines()
        except OSError as e:
            raise SystemExit(f"error: cannot read {args.tickers_file}: {e}") from e
        raw += [ln.split("#", 1)[0] for ln in lines]
    tickers = sorted({normalise_ticker(t) for t in raw if t.strip()})
    if not tickers:
        raise SystemExit("error: no tickers given (pass them as arguments or via --tickers-file)")
    return tickers


def download(yf, ticker: str, start: dt.date, end: dt.date):
    # yfinance's `end` is exclusive, so ask for one extra day.
    try:
        yahoo = ticker if ticker.startswith(INDEX_PREFIX) else ticker + NSE_SUFFIX
        df = yf.Ticker(yahoo).history(
            start=start.isoformat(),
            end=(end + dt.timedelta(days=1)).isoformat(),
            interval="1d",
            auto_adjust=True,
            actions=False,
        )
    except Exception as e:  # yfinance raises many types for network/HTTP problems
        raise FetchError(f"download failed: {e}") from e
    if df is None or df.empty:
        raise FetchError("no rows returned (unknown ticker, delisted, or no data in range)")
    return df


def fmt_price(x: float) -> str:
    return f"{x:.{PRICE_DECIMALS}f}"


def to_rows(df) -> list[str]:
    """Converts a yfinance frame to CSV rows, failing loudly on anything CsvDaily would reject."""
    rows: list[str] = []
    seen: set[dt.date] = set()
    for ts, r in df.iterrows():
        day = ts.date()
        vals = [r["Open"], r["High"], r["Low"], r["Close"], r["Volume"]]
        if any(v is None or (isinstance(v, float) and math.isnan(v)) for v in vals):
            raise FetchError(f"{day}: missing value in {vals}")
        if day in seen:
            raise FetchError(f"{day}: duplicate date from source")
        seen.add(day)
        o, h, lo, c = (fmt_price(float(v)) for v in vals[:4])
        check_ohlc(day, float(o), float(h), float(lo), float(c), int(vals[4]))
        rows.append(f"{day.isoformat()},{o},{h},{lo},{c},{int(vals[4])}")
    return rows


def check_ohlc(day: dt.date, o: float, h: float, lo: float, c: float, vol: int) -> None:
    if lo <= 0 or h < lo or not (lo <= o <= h and lo <= c <= h) or vol < 0:
        raise FetchError(f"{day}: invalid bar o={o} h={h} l={lo} c={c} v={vol}")


def write_atomic(path: Path, rows: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=path.parent, suffix=".tmp")
    try:
        with os.fdopen(fd, "w", newline="\n") as f:
            f.write(HEADER + "\n" + "\n".join(rows) + "\n")
        os.replace(tmp, path)
    except BaseException:
        Path(tmp).unlink(missing_ok=True)
        raise


def fetch_one(yf, ticker: str, start: dt.date, end: dt.date, out: Path) -> int:
    df = download(yf, ticker, start, end)
    rows = to_rows(df.sort_index())
    write_atomic(out / f"{ticker.lstrip(INDEX_PREFIX)}.csv", rows)
    return len(rows)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    start = parse_date(args.start, "--start")
    end = parse_date(args.end, "--end") if args.end else dt.date.today()
    if start > end:
        raise SystemExit(f"error: --start {start} is after --end {end}")
    tickers = collect_tickers(args)
    try:
        import yfinance as yf
    except ImportError:
        raise SystemExit("error: yfinance is not installed; run `pip install yfinance`") from None

    failed: list[str] = []
    for t in tickers:
        try:
            n = fetch_one(yf, t, start, end, args.out)
            print(f"{t}: {n} rows -> {args.out / (t.lstrip(INDEX_PREFIX) + '.csv')}")
        except (FetchError, OSError) as e:
            print(f"{t}: FAILED: {e}", file=sys.stderr)
            failed.append(t)
    if failed:
        print(f"{len(failed)}/{len(tickers)} tickers failed: {' '.join(failed)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env python3
"""Download free Binance spot 1-minute klines from data.binance.vision (public, no account).

For each symbol and month it fetches `<SYM>-1m-YYYY-MM.zip` and its `.CHECKSUM` file, verifies
the SHA-256, and extracts the CSV to engine/data/intraday/<SYM>/<SYM>-1m-YYYY-MM.csv. The zip is
discarded after a verified extract. A month whose CSV already exists is skipped.

The CSV is headerless, one row per minute:
  open_time, open, high, low, close, volume, close_time, quote_volume, trades,
  taker_buy_base, taker_buy_quote, ignore
open_time is epoch MILLISECONDS up to 2024-12 and MICROSECONDS from 2025-01 (Binance changed the
spot archive format); `tq_data::IntradayCsv` normalises both to ms.

Standard library only. Exit code 1 if any month failed (others still complete).
"""

from __future__ import annotations

import argparse
import hashlib
import io
import os
import re
import sys
import tempfile
import time
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

BASE_URL = "https://data.binance.vision/data/spot/monthly/klines"
INTERVAL = "1m"
DEFAULT_SYMBOLS = ["BTCUSDT", "ETHUSDT"]
DEFAULT_START = "2022-01"
DEFAULT_END = "2025-12"
DEFAULT_OUT = Path(__file__).resolve().parent.parent / "data" / "intraday"
SYMBOL_RE = re.compile(r"^[A-Z0-9]{2,20}$")
MONTH_RE = re.compile(r"^(\d{4})-(\d{2})$")
RETRIES = 3
TIMEOUT_S = 60
BACKOFF_S = 2.0


class FetchError(Exception):
    """A month could not be downloaded or failed verification."""


def parse_month(text: str) -> tuple[int, int]:
    m = MONTH_RE.match(text)
    if not m or not 1 <= int(m.group(2)) <= 12:
        raise SystemExit(f"error: month must be YYYY-MM, got {text!r}")
    return int(m.group(1)), int(m.group(2))


def months(start: tuple[int, int], end: tuple[int, int]) -> list[str]:
    (y, mo), out = start, []
    while (y, mo) <= end:
        out.append(f"{y:04d}-{mo:02d}")
        y, mo = (y + 1, 1) if mo == 12 else (y, mo + 1)
    return out


def http_get(url: str) -> bytes:
    last: Exception | None = None
    for attempt in range(RETRIES):
        try:
            with urllib.request.urlopen(url, timeout=TIMEOUT_S) as resp:
                return resp.read()
        except urllib.error.HTTPError as e:
            if e.code == 404:
                raise FetchError(f"not published: {url}") from e
            last = e
        except (urllib.error.URLError, TimeoutError, OSError) as e:
            last = e
        time.sleep(BACKOFF_S * (attempt + 1))
    raise FetchError(f"failed after {RETRIES} tries: {url}: {last}")


def expected_sha256(checksum_text: str, zip_name: str) -> str:
    parts = checksum_text.split()
    if len(parts) != 2 or parts[1] != zip_name or not re.fullmatch(r"[0-9a-f]{64}", parts[0]):
        raise FetchError(f"malformed CHECKSUM for {zip_name}: {checksum_text!r}")
    return parts[0]


def extract_csv(blob: bytes, csv_name: str) -> bytes:
    with zipfile.ZipFile(io.BytesIO(blob)) as zf:
        names = zf.namelist()
        if names != [csv_name]:
            raise FetchError(f"unexpected zip members {names}, wanted [{csv_name}]")
        return zf.read(csv_name)


def write_atomic(path: Path, data: bytes) -> None:
    fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=".tmp-", suffix=".csv")
    try:
        with os.fdopen(fd, "wb") as f:
            f.write(data)
        os.replace(tmp, path)
    except BaseException:
        Path(tmp).unlink(missing_ok=True)
        raise


def fetch_month(symbol: str, month: str, out_dir: Path) -> str:
    stem = f"{symbol}-{INTERVAL}-{month}"
    target = out_dir / f"{stem}.csv"
    if target.exists():
        return "skipped (exists)"
    url = f"{BASE_URL}/{symbol}/{INTERVAL}/{stem}.zip"
    want = expected_sha256(http_get(url + ".CHECKSUM").decode("ascii", "replace"), f"{stem}.zip")
    blob = http_get(url)
    got = hashlib.sha256(blob).hexdigest()
    if got != want:
        raise FetchError(f"SHA-256 mismatch for {stem}.zip: got {got}, want {want}")
    write_atomic(target, extract_csv(blob, f"{stem}.csv"))
    return f"ok sha256={got[:12]}"


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("symbols", nargs="*", default=DEFAULT_SYMBOLS, help="Binance spot symbols")
    p.add_argument("--start", default=DEFAULT_START, help=f"first month YYYY-MM (default {DEFAULT_START})")
    p.add_argument("--end", default=DEFAULT_END, help=f"last month YYYY-MM (default {DEFAULT_END})")
    p.add_argument("--out", type=Path, default=DEFAULT_OUT, help=f"output root (default {DEFAULT_OUT})")
    args = p.parse_args(argv)
    span = months(parse_month(args.start), parse_month(args.end))
    failures = 0
    for raw in args.symbols:
        symbol = raw.strip().upper()
        if not SYMBOL_RE.match(symbol):
            raise SystemExit(f"error: invalid symbol {raw!r}")
        out_dir = args.out / symbol
        out_dir.mkdir(parents=True, exist_ok=True)
        for month in span:
            try:
                print(f"{symbol} {month}: {fetch_month(symbol, month, out_dir)}", flush=True)
            except FetchError as e:
                failures += 1
                print(f"{symbol} {month}: FAILED: {e}", file=sys.stderr, flush=True)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

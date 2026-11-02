#!/usr/bin/env python3
"""Funding carry backtest (edge-hunt-01 candidate A): long spot N, short USD-M perp N, margin M.

Implements the pre-registered spec in docs/edge-hunt-01.md section 4. No parameter here is tuned;
every number below comes from that spec or docs/binance-futures-facts-16.md.

Hourly simulation on Binance public-archive 1h klines (scripts/fetch_binance_funding.py):
- Entry at the first hour's open: buy q = N/S spot, short q perp, M USDT in the futures wallet.
- Funding: each archived payment is applied at its hour; the short RECEIVES q * mark * rate
  (mark = mark-price open of that hour; negative rate means the short pays).
- Liquidation: checked at the MARK-price high of every hour against maintenance margin.
  WHY: Binance liquidates on mark, and last-price highs carry bad wicks (ETH perp 2020-03-13 02:00
  printed 323 while mark peaked at 119.5); the spec's "1h bar high" is read as the mark bar.
  A liquidation is fatal for the run (kill criterion 4), so it is reported, not recovered from.
- Rebalance: legs always hold equal quantity, so price delta is zero; what drifts is the margin.
  WHY: the spec's "|delta| > 2% of N" trigger never fires with equal quantities, so the spec intent
  (keep the short safe) is implemented as a margin band: when perp equity / perp notional leaves
  [0.5 m, 1.5 m] (m = M/N) total equity is re-split N:M and both legs are resized at taker cost.
- Exit at the last hour's close, paying costs on both legs.

Tax cases (none is advice): (a) pre-tax; (b) s.115BBH worst case: 30% of positive annual funding
plus positive annual spot price gain, no set-off of perp losses, no deduction of fees; (c) 1% TDS
on every spot sell, refunded after a year, costed at the hurdle rate.
"""

from __future__ import annotations

import argparse
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import pandas as pd

DATA = Path(__file__).resolve().parent.parent / "data"
SPOT_FEE = 7.5e-4
PERP_FEE = 5e-4
SLIPPAGE = 1e-4
MMR = 0.005  # WHY: tier-1 MMR is 0.4% (BTC) / 0.5% (ETH); use the stricter one for both.
BAND = (0.5, 1.5)
TAX_RATE = 0.30
TDS_RATE = 0.01
HURDLE = 0.08
IS_END = "2023-12-31"
SEED = 16
BOOT = 2000
KLINE_COLS = ["open_time", "open", "high", "low", "close", "volume", "close_time",
              "quote_volume", "count", "taker_buy_volume", "taker_buy_quote_volume", "ignore"]


def load_klines(sub: str, sym: str) -> pd.DataFrame:
    frames = []
    for f in sorted((DATA / sub / sym).glob("*.csv")):
        df = pd.read_csv(f, header=None, names=KLINE_COLS, dtype=str)
        df = df[df.open_time.str.isdigit()]  # some perp months ship a header row
        frames.append(df[["open_time", "open", "high", "close"]])
    df = pd.concat(frames).astype({"open_time": "int64", "open": float, "high": float, "close": float})
    # Spot archive switched to microseconds in 2025-01.
    df["open_time"] = np.where(df.open_time > 10**14, df.open_time // 1000, df.open_time)
    df["ts"] = pd.to_datetime(df.open_time, unit="ms", utc=True)
    return df.drop_duplicates("ts").set_index("ts").sort_index()[["open", "high", "close"]]


def load_funding(sym: str) -> pd.Series:
    df = pd.concat(pd.read_csv(f) for f in sorted((DATA / "funding" / sym).glob("*.csv")))
    ts = pd.to_datetime(df.calc_time, unit="ms", utc=True).dt.floor("h")
    s = pd.Series(df.last_funding_rate.astype(float).values, index=ts)
    return s.groupby(level=0).sum()


@dataclass
class Run:
    daily: pd.Series        # daily equity, UTC day close
    funding: pd.Series      # funding cash received per hour
    spot_pnl: pd.Series     # spot price PnL per hour
    spot_sells: pd.Series   # spot sell proceeds per hour (TDS base)
    rebalances: int
    liquidated: pd.Timestamp | None


def simulate(spot: pd.DataFrame, perp: pd.DataFrame, mark: pd.DataFrame, rates: pd.Series, m: float) -> Run:
    idx = spot.index.intersection(perp.index).intersection(mark.index)
    s_open, s_close = spot.open.reindex(idx).values, spot.close.reindex(idx).values
    p_open, p_close = perp.open.reindex(idx).values, perp.close.reindex(idx).values
    m_open, m_high = mark.open.reindex(idx).values, mark.high.reindex(idx).values
    rate = rates.reindex(idx).fillna(0.0).values
    n_frac = 1.0 / (1.0 + m)
    leg_cost = lambda dq, s, p: dq * s * (SPOT_FEE + SLIPPAGE) + dq * p * (PERP_FEE + SLIPPAGE)

    equity0 = 1.0
    q = equity0 * n_frac / s_open[0]
    cash_spot = 0.0
    perp_cash = equity0 * (1 - n_frac) - leg_cost(q, s_open[0], p_open[0])
    entry_p = p_open[0]
    fund, spnl, sells, eq = (np.zeros(len(idx)) for _ in range(4))
    rebalances, liquidated = 0, None
    prev_s = s_open[0]
    for i in range(len(idx)):
        fund[i] = q * m_open[i] * rate[i]
        perp_cash += fund[i]
        if perp_cash + q * (entry_p - m_high[i]) < MMR * q * m_high[i]:
            liquidated = idx[i]
            eq[i:] = np.nan
            break
        spnl[i] = q * (s_close[i] - prev_s)
        prev_s = s_close[i]
        perp_eq = perp_cash + q * (entry_p - p_close[i])
        total = q * s_close[i] + cash_spot + perp_eq
        ratio = perp_eq / (q * p_close[i])
        if not BAND[0] * m <= ratio <= BAND[1] * m:
            new_q = total * n_frac / s_close[i]
            dq = abs(new_q - q)
            if new_q < q:
                sells[i] = dq * s_close[i]
            # Realise the perp PnL into cash and re-enter the short at the new size.
            perp_cash = total * (1 - n_frac) - leg_cost(dq, s_close[i], p_close[i])
            cash_spot, q, entry_p = 0.0, new_q, p_close[i]
            total = q * s_close[i] + perp_cash
            rebalances += 1
        eq[i] = total
    if liquidated is None:
        exit_cost = leg_cost(q, s_close[-1], p_close[-1])
        eq[-1] -= exit_cost
        sells[-1] += q * s_close[-1]
    ser = lambda a: pd.Series(a, index=idx)
    return Run(ser(eq).resample("D").last().dropna(), ser(fund), ser(spnl), ser(sells), rebalances, liquidated)


def hac_t(r: np.ndarray) -> float:
    t_obs = len(r)
    lag = int(math.floor(4 * (t_obs / 100) ** (2 / 9)))
    d = r - r.mean()
    var = d @ d / t_obs
    for k in range(1, lag + 1):
        var += 2 * (1 - k / (lag + 1)) * (d[k:] @ d[:-k]) / t_obs
    return r.mean() / math.sqrt(var / t_obs)


def ann(r: pd.Series) -> float:
    return float((1 + r).prod() ** (365 / len(r)) - 1) if len(r) else float("nan")


def boot_ci(r: pd.Series) -> tuple[float, float]:
    months = [g.values for _, g in r.groupby(r.index.to_period("M"))]
    rng = np.random.default_rng(SEED)
    draws = []
    for _ in range(BOOT):
        pick = np.concatenate([months[j] for j in rng.integers(0, len(months), len(months))])
        draws.append((1 + pick).prod() ** (365 / len(pick)) - 1)
    return float(np.percentile(draws, 2.5)), float(np.percentile(draws, 97.5))


def max_dd(eq: pd.Series) -> float:
    return float((eq / eq.cummax() - 1).min())


def yearly(run: Run) -> pd.DataFrame:
    eq = run.daily
    end = eq.groupby(eq.index.year).last()
    start = end.shift(1).fillna(1.0)
    fy, sy = (x.groupby(x.index.year).sum() for x in (run.funding, run.spot_pnl))
    tds = run.spot_sells.groupby(run.spot_sells.index.year).sum() * TDS_RATE * HURDLE
    pnl = end - start
    tax_b = TAX_RATE * (fy.clip(lower=0) + sy.clip(lower=0))
    out = pd.DataFrame({"pretax": pnl / start, "funding": fy / start, "spot_gain": sy / start,
                        "tax_b": (pnl - tax_b) / start, "tds_c": (pnl - tds) / start})
    return out.reindex(end.index)


def report(sym: str, m: float, run: Run, rates: pd.Series) -> list[str]:
    r = run.daily.pct_change().dropna()
    r_is, r_oos = r[:IS_END], r[IS_END:].iloc[1:]
    lo, hi = boot_ci(r)
    last12 = rates[rates.index > rates.index.max() - pd.Timedelta(days=365)]
    y = yearly(run)
    lines = [f"### {sym}, M = {m:.1f}N",
             f"- liquidated: {run.liquidated or 'no'}; rebalances: {run.rebalances}",
             f"- full {ann(r):+.2%}/yr (month-block 95% CI {lo:+.2%} to {hi:+.2%}), HAC t {hac_t(r.values):.2f}, "
             f"max DD {max_dd(run.daily):.2%}",
             f"- in-sample {ann(r_is):+.2%}/yr, out-of-sample {ann(r_oos):+.2%}/yr "
             f"(OOS/IS {ann(r_oos) / ann(r_is):.2f})",
             f"- funding positive in {(last12 > 0).mean():.0%} of last-12-month intervals",
             "", "| year | pre-tax | funding | spot gain | tax (b) | TDS (c) |", "|---|---|---|---|---|---|"]
    lines += [f"| {yr} | {row.pretax:+.2%} | {row.funding:+.2%} | {row.spot_gain:+.2%} | "
              f"{row.tax_b:+.2%} | {row.tds_c:+.2%} |" for yr, row in y.iterrows()]
    kills = []
    if ann(r_oos) < HURDLE or ann(r_oos) < 0.5 * ann(r_is):
        kills.append("1 OOS below hurdle or <50% of IS")
    if hac_t(r.values) < 2:
        kills.append("2 HAC t < 2")
    if any(y.tax_b.get(yr, 0) < 0 for yr in (2021, 2024)):
        kills.append("3 tax case (b) negative in 2021 or 2024")
    if run.liquidated is not None or max_dd(run.daily) < -0.05:
        kills.append("4 liquidation or DD > 5%")
    if (last12 > 0).mean() < 0.70:
        kills.append("5 funding positive < 70% in last 12 months")
    lines += ["", f"- kill criteria hit: {', '.join(kills) if kills else 'none'}", ""]
    return lines


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--symbols", nargs="+", default=["BTCUSDT", "ETHUSDT"])
    p.add_argument("--start", default="2020-01-01")
    p.add_argument("--out", type=Path, default=None, help="write markdown here (default: stdout)")
    args = p.parse_args()
    lines = []
    for sym in args.symbols:
        spot, perp, mark = (load_klines(sub, sym)[args.start:] for sub in ("spot-1h", "perp-1h", "mark-1h"))
        rates = load_funding(sym)[args.start:]
        for m in (1.0, 0.5):
            lines += report(sym, m, simulate(spot, perp, mark, rates, m), rates)
    text = "\n".join(lines)
    if args.out:
        args.out.write_text(text + "\n")
    print(text)


if __name__ == "__main__":
    main()

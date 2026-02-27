#!/usr/bin/env python3
"""
Random-effects meta-analysis (DerSimonian–Laird) from a simple effect-size CSV.

Input CSV (minimum columns):
  - outcome_id
  - study_id
  - yi   (effect size on the analysis scale, e.g., log(OR), SMD, Fisher's z)
  - sei  (standard error of yi)

Usage:
  python meta_analysis_random_effects.py effect_sizes.csv --out results.csv

Notes:
  - This is a lightweight template (no stats packages required).
  - Convert back from log-scales after pooling (e.g., OR = exp(yi)).
  - For Hartung-Knapp, meta-regression, trim-and-fill, etc., prefer R/metafor.
"""

from __future__ import annotations

import argparse
import csv
import math
from collections import defaultdict
from dataclasses import dataclass


Z_975 = 1.959963984540054  # ~1.96


@dataclass(frozen=True)
class Row:
    outcome_id: str
    study_id: str
    yi: float
    sei: float


def _safe_float(value: str) -> float:
    try:
        return float(value)
    except ValueError as exc:
        raise ValueError(f"Expected float, got {value!r}") from exc


def read_rows(path: str) -> list[Row]:
    with open(path, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        required = {"outcome_id", "study_id", "yi", "sei"}
        if not reader.fieldnames or not required.issubset(set(reader.fieldnames)):
            raise ValueError(f"CSV must contain columns: {sorted(required)}")

        rows: list[Row] = []
        for i, r in enumerate(reader, start=2):
            outcome_id = (r.get("outcome_id") or "").strip()
            study_id = (r.get("study_id") or "").strip()
            if not outcome_id or not study_id:
                raise ValueError(f"Missing outcome_id/study_id at line {i}")
            yi = _safe_float((r.get("yi") or "").strip())
            sei = _safe_float((r.get("sei") or "").strip())
            if sei <= 0:
                raise ValueError(f"sei must be > 0 at line {i}")
            rows.append(Row(outcome_id=outcome_id, study_id=study_id, yi=yi, sei=sei))
        return rows


def dersimonian_laird(ys: list[float], ses: list[float]) -> dict[str, float]:
    k = len(ys)
    if k < 2:
        raise ValueError("Random-effects pooling requires at least 2 studies.")

    vs = [se * se for se in ses]
    ws = [1.0 / v for v in vs]
    sum_w = sum(ws)
    y_fixed = sum(w * y for w, y in zip(ws, ys)) / sum_w

    q = sum(w * (y - y_fixed) ** 2 for w, y in zip(ws, ys))
    df = k - 1
    c = sum_w - (sum(w * w for w in ws) / sum_w)
    tau2 = max(0.0, (q - df) / c) if c > 0 else 0.0

    ws_re = [1.0 / (v + tau2) for v in vs]
    sum_w_re = sum(ws_re)
    y_re = sum(w * y for w, y in zip(ws_re, ys)) / sum_w_re
    se_re = math.sqrt(1.0 / sum_w_re)
    ci_low = y_re - Z_975 * se_re
    ci_high = y_re + Z_975 * se_re

    i2 = 0.0
    if q > 0:
        i2 = max(0.0, (q - df) / q) * 100.0

    return {
        "k": float(k),
        "y_fixed": y_fixed,
        "q": q,
        "df": float(df),
        "tau2": tau2,
        "i2": i2,
        "y_random": y_re,
        "se_random": se_re,
        "ci_low": ci_low,
        "ci_high": ci_high,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("csv_path", help="CSV with outcome_id, study_id, yi, sei")
    parser.add_argument("--out", default="meta_analysis_results.csv", help="Output CSV path")
    args = parser.parse_args()

    rows = read_rows(args.csv_path)
    grouped: dict[str, list[Row]] = defaultdict(list)
    for r in rows:
        grouped[r.outcome_id].append(r)

    results: list[dict[str, str]] = []
    for outcome_id, rs in sorted(grouped.items(), key=lambda x: x[0]):
        ys = [r.yi for r in rs]
        ses = [r.sei for r in rs]
        stats = dersimonian_laird(ys, ses)

        print(f"\nOutcome: {outcome_id}")
        print(f"  k={int(stats['k'])}  tau^2={stats['tau2']:.4g}  I^2={stats['i2']:.1f}%")
        print(
            f"  pooled={stats['y_random']:.4g}  95% CI [{stats['ci_low']:.4g}, {stats['ci_high']:.4g}]"
        )

        results.append(
            {
                "outcome_id": outcome_id,
                "k": str(int(stats["k"])),
                "pooled_yi": f"{stats['y_random']:.10g}",
                "pooled_se": f"{stats['se_random']:.10g}",
                "ci_low": f"{stats['ci_low']:.10g}",
                "ci_high": f"{stats['ci_high']:.10g}",
                "tau2": f"{stats['tau2']:.10g}",
                "i2_percent": f"{stats['i2']:.10g}",
                "q": f"{stats['q']:.10g}",
                "df": str(int(stats["df"])),
            }
        )

    with open(args.out, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "outcome_id",
                "k",
                "pooled_yi",
                "pooled_se",
                "ci_low",
                "ci_high",
                "tau2",
                "i2_percent",
                "q",
                "df",
            ],
        )
        writer.writeheader()
        writer.writerows(results)

    print(f"\nWrote: {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


#!/usr/bin/env python3
"""
Compare TPS benchmark results between current branch and base branch.

Usage:
    python3 scripts/bench_compare.py \
        --current results_current.json \
        --base results_base.json \
        --output report.md \
        [--strict]

If --base is missing or the file doesn't exist, only the current branch
results are reported (no regression check).

Exit codes:
    0 - OK (or strict mode disabled)
    1 - Regression detected AND --strict is enabled
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Optional, Tuple

# ---------------------------------------------------------------------------
# Thresholds  (empirical: CV ≈ 20 % across runs on same machine)
# ---------------------------------------------------------------------------
# Overall TPS metrics
TPS_REGRESSION_THRESHOLD = 0.25           # 25 % drop allowed
STABLE_TPS_REGRESSION_THRESHOLD = 0.25    # 25 % drop allowed
BLOCK_TPS_AVG_REGRESSION_THRESHOLD = 0.30 # 30 % drop allowed (noisier)
LATENCY_INCREASE_THRESHOLD = 0.30         # 30 % increase allowed

# Per-stage throughput regression thresholds
# Throughput (txn/s): higher is better → drop = regression
STAGE_THROUGHPUT_THRESHOLDS = {
    "TxPool Verify":  0.20,   # 20 % — most stable stage (CV ≈ 2 %)
    "Block Build":    0.20,   # 20 % — stable (CV ≈ 2 %)
    "VM Execute":     0.40,   # 40 % — extremely fast, noisy in relative terms
    "State Commit":   0.30,   # 30 % — moderate noise
}
# Per-stage avg_ms regression thresholds
# avg_ms (ms): lower is better → increase = regression
STAGE_AVG_MS_THRESHOLDS = {
    "TxPool Verify":  0.20,
    "Block Build":    0.20,
    "VM Execute":     0.50,   # sub-ms, very noisy
    "State Commit":   0.30,
}

# Stages to check (in display order)
PIPELINE_STAGES = ["TxPool Verify", "Block Build", "VM Execute", "State Commit"]


def load_json(path: str) -> Optional[dict]:
    """Load a benchmark_results.json file, return None on failure."""
    p = Path(path)
    if not p.exists():
        return None
    try:
        with open(p) as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError) as e:
        print(f"⚠️  Failed to load {path}: {e}", file=sys.stderr)
        return None


def pct_change(current: float, base: float) -> float:
    """Return relative change: positive means improvement, negative means regression."""
    if base == 0:
        return 0.0
    return (current - base) / base


def fmt_pct(val: float) -> str:
    sign = "+" if val >= 0 else ""
    return f"{sign}{val * 100:.1f}%"


def fmt_num(val: float) -> str:
    """Format a number: use .3f for sub-1, .1f otherwise."""
    if abs(val) < 1:
        return f"{val:.3f}"
    return f"{val:.1f}"


def compare_metric(
    name: str,
    current_val: float,
    base_val: float,
    threshold: float,
    higher_is_better: bool = True,
) -> dict:
    """Compare a single metric and return a result dict."""
    change = pct_change(current_val, base_val)
    if higher_is_better:
        regression = change < -threshold
    else:
        regression = change > threshold

    return {
        "name": name,
        "current": current_val,
        "base": base_val,
        "change": change,
        "regression": regression,
        "threshold": threshold,
    }


def build_report(current: dict, base: Optional[dict]) -> Tuple[str, bool]:
    """
    Build a markdown report comparing current vs base.
    Returns (markdown_string, has_regression).
    """
    lines = []  # type: list[str]
    has_regression = False
    cs = current.get("summary", {})

    lines.append("# TPS Benchmark Report")
    lines.append("")
    lines.append(f"**Timestamp**: {current.get('timestamp', 'N/A')}")
    lines.append(f"**Blocks**: {cs.get('block_count', 'N/A')} "
                 f"(middle: {cs.get('middle_block_count', 'N/A')})")
    lines.append(f"**Total Executed**: {cs.get('total_executed', 'N/A')}")
    lines.append("")

    # --- Current branch summary ---
    lines.append("## Current Branch Results")
    lines.append("")
    lines.append("| Metric | Value |")
    lines.append("|--------|-------|")
    lines.append(f"| TPS | {cs.get('tps', 0):.1f} |")
    lines.append(f"| Stable TPS | {cs.get('stable_tps', 0):.1f} |")
    lines.append(f"| Block TPS Avg | {cs.get('block_tps_avg', 0):.1f} |")
    lines.append(f"| Block TPS Median | {cs.get('block_tps_median', 0):.1f} |")
    lines.append(f"| Avg Latency (ms) | {cs.get('avg_latency_ms', 0):.1f} |")
    lines.append(f"| Median Latency (ms) | {cs.get('median_latency_ms', 0):.1f} |")
    lines.append(f"| Duplicate Exec % | {cs.get('duplicate_pct', 0):.2f} |")
    lines.append("")

    # --- Pipeline stages (current only) ---
    cps = current.get("pipeline_stages", {})
    if cps:
        lines.append("## Pipeline Stages (Current)")
        lines.append("")
        lines.append("| Stage | Count | Avg (ms) | Throughput (txn/s) |")
        lines.append("|-------|-------|----------|-------------------|")
        for stage in PIPELINE_STAGES:
            s = cps.get(stage, {})
            if s.get("count", 0) > 0:
                lines.append(
                    f"| {stage} | {s['count']} | {fmt_num(s['avg_ms'])} "
                    f"| {fmt_num(s['throughput'])} |"
                )
        lines.append("")

    # --- No base → stop here ---
    if base is None:
        lines.append("## Comparison")
        lines.append("")
        lines.append("> ℹ️  No base branch results available — skipping regression check.")
        lines.append("")
        return "\n".join(lines), False

    # ================================================================
    # Comparison vs base
    # ================================================================
    bs = base.get("summary", {})
    bps = base.get("pipeline_stages", {})

    # -- Overall metrics --
    lines.append("## Overall Comparison vs Base Branch")
    lines.append("")

    overall = [
        compare_metric("TPS", cs.get("tps", 0), bs.get("tps", 0),
                        TPS_REGRESSION_THRESHOLD, higher_is_better=True),
        compare_metric("Stable TPS", cs.get("stable_tps", 0), bs.get("stable_tps", 0),
                        STABLE_TPS_REGRESSION_THRESHOLD, higher_is_better=True),
        compare_metric("Block TPS Avg", cs.get("block_tps_avg", 0), bs.get("block_tps_avg", 0),
                        BLOCK_TPS_AVG_REGRESSION_THRESHOLD, higher_is_better=True),
        compare_metric("Avg Latency (ms)", cs.get("avg_latency_ms", 0), bs.get("avg_latency_ms", 0),
                        LATENCY_INCREASE_THRESHOLD, higher_is_better=False),
    ]

    lines.append("| Metric | Current | Base | Change | Status |")
    lines.append("|--------|---------|------|--------|--------|")
    for c in overall:
        status = "❌ REGRESSION" if c["regression"] else "✅ OK"
        if c["regression"]:
            has_regression = True
        lines.append(
            f"| {c['name']} | {fmt_num(c['current'])} | {fmt_num(c['base'])} "
            f"| {fmt_pct(c['change'])} | {status} |"
        )
    lines.append("")

    # -- Per-stage comparison --
    if cps and bps:
        lines.append("## Pipeline Stage Comparison vs Base Branch")
        lines.append("")
        lines.append("### Throughput (txn/s) — higher is better")
        lines.append("")
        lines.append("| Stage | Current | Base | Change | Status |")
        lines.append("|-------|---------|------|--------|--------|")
        for stage in PIPELINE_STAGES:
            cs_stage = cps.get(stage, {})
            bs_stage = bps.get(stage, {})
            ct = cs_stage.get("throughput", 0)
            bt = bs_stage.get("throughput", 0)
            if bt == 0 and ct == 0:
                continue
            threshold = STAGE_THROUGHPUT_THRESHOLDS.get(stage, 0.25)
            c = compare_metric(stage, ct, bt, threshold, higher_is_better=True)
            status = "❌ REGRESSION" if c["regression"] else "✅ OK"
            if c["regression"]:
                has_regression = True
            lines.append(
                f"| {stage} | {fmt_num(ct)} | {fmt_num(bt)} "
                f"| {fmt_pct(c['change'])} | {status} |"
            )
        lines.append("")

        lines.append("### Avg Latency per-op (ms) — lower is better")
        lines.append("")
        lines.append("| Stage | Current | Base | Change | Status |")
        lines.append("|-------|---------|------|--------|--------|")
        for stage in PIPELINE_STAGES:
            cs_stage = cps.get(stage, {})
            bs_stage = bps.get(stage, {})
            ca = cs_stage.get("avg_ms", 0)
            ba = bs_stage.get("avg_ms", 0)
            if ba == 0 and ca == 0:
                continue
            threshold = STAGE_AVG_MS_THRESHOLDS.get(stage, 0.25)
            c = compare_metric(stage, ca, ba, threshold, higher_is_better=False)
            status = "❌ REGRESSION" if c["regression"] else "✅ OK"
            if c["regression"]:
                has_regression = True
            lines.append(
                f"| {stage} | {fmt_num(ca)} | {fmt_num(ba)} "
                f"| {fmt_pct(c['change'])} | {status} |"
            )
        lines.append("")

    # -- Threshold info --
    lines.append("<details><summary>Threshold Configuration</summary>")
    lines.append("")
    lines.append("**Overall metrics:**")
    lines.append(f"- TPS drop tolerance: **{TPS_REGRESSION_THRESHOLD * 100:.0f}%**")
    lines.append(f"- Stable TPS drop tolerance: **{STABLE_TPS_REGRESSION_THRESHOLD * 100:.0f}%**")
    lines.append(f"- Block TPS Avg drop tolerance: **{BLOCK_TPS_AVG_REGRESSION_THRESHOLD * 100:.0f}%**")
    lines.append(f"- Latency increase tolerance: **{LATENCY_INCREASE_THRESHOLD * 100:.0f}%**")
    lines.append("")
    lines.append("**Per-stage throughput drop tolerance:**")
    for stage in PIPELINE_STAGES:
        t = STAGE_THROUGHPUT_THRESHOLDS.get(stage, 0)
        lines.append(f"- {stage}: **{t * 100:.0f}%**")
    lines.append("")
    lines.append("**Per-stage avg_ms increase tolerance:**")
    for stage in PIPELINE_STAGES:
        t = STAGE_AVG_MS_THRESHOLDS.get(stage, 0)
        lines.append(f"- {stage}: **{t * 100:.0f}%**")
    lines.append("")
    lines.append("</details>")
    lines.append("")

    if has_regression:
        lines.append("> ⚠️  **Regression detected.** "
                     "One or more metrics exceeded the allowed tolerance threshold.")
    else:
        lines.append("> ✅  **No regression detected.** All metrics within tolerance.")
    lines.append("")

    return "\n".join(lines), has_regression


def main():
    parser = argparse.ArgumentParser(description="Compare TPS benchmark results")
    parser.add_argument("--current", required=True, help="Path to current branch results JSON")
    parser.add_argument("--base", default="", help="Path to base branch results JSON (optional)")
    parser.add_argument("--output", default="bench_report.md", help="Output markdown report path")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Exit with code 1 if regression is detected (default: off)",
    )
    args = parser.parse_args()

    current = load_json(args.current)
    if current is None:
        print(f"❌ Cannot load current branch results from {args.current}", file=sys.stderr)
        sys.exit(1)

    base = load_json(args.base) if args.base else None

    report, has_regression = build_report(current, base)

    # Write report
    out_path = Path(args.output)
    out_path.write_text(report, encoding="utf-8")
    print(f"📄 Report written to {out_path}")

    # Also print to stdout for CI log
    print()
    print(report)

    if has_regression and args.strict:
        print("❌ Strict mode: failing CI due to regression.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

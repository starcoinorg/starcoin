#!/usr/bin/env python3
"""
Analyze sync profiling logs emitted with prefix "[sync-prof]".

Example:
  python3 scripts/analyze_sync_profile_log.py --log-file /path/to/starcoin.log
"""

from __future__ import annotations

import argparse
import math
import statistics
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List


@dataclass
class StageStat:
    count: int = 0
    error_count: int = 0
    durations_ms: List[float] = field(default_factory=list)

    def add(self, status: str | None, duration_ms: float | None) -> None:
        self.count += 1
        if status is not None and status not in {"ok", "slow"}:
            self.error_count += 1
        if duration_ms is not None:
            self.durations_ms.append(duration_ms)

    def avg(self) -> float:
        if not self.durations_ms:
            return 0.0
        return statistics.fmean(self.durations_ms)

    def p95(self) -> float:
        if not self.durations_ms:
            return 0.0
        data = sorted(self.durations_ms)
        idx = min(len(data) - 1, math.ceil(0.95 * len(data)) - 1)
        return data[idx]

    def total(self) -> float:
        return sum(self.durations_ms)

    def max(self) -> float:
        if not self.durations_ms:
            return 0.0
        return max(self.durations_ms)


def parse_kv(segment: str) -> Dict[str, str]:
    kv: Dict[str, str] = {}
    for token in segment.strip().split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        kv[key] = value
    return kv


def pick_duration_ms(kv: Dict[str, str]) -> float | None:
    # Prefer end-to-end stage time first.
    for key in ("total_ms", "elapsed_ms", "waited_ms", "remote_fetch_ms"):
        value = kv.get(key)
        if value is None:
            continue
        try:
            return float(value)
        except ValueError:
            continue
    return None


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Analyze sync profiling logs.")
    parser.add_argument("--log-file", required=True, help="Path to log file.")
    parser.add_argument(
        "--prefix",
        default="[sync-prof]",
        help='Log prefix to filter (default: "[sync-prof]").',
    )
    parser.add_argument(
        "--top",
        type=int,
        default=10,
        help="How many stages to show (sorted by total duration).",
    )
    return parser


def main() -> int:
    args = build_arg_parser().parse_args()
    log_path = Path(args.log_file)
    if not log_path.exists():
        print(f"log file not found: {log_path}")
        return 2

    stats: Dict[str, StageStat] = {}
    total_profile_lines = 0

    with log_path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            idx = line.find(args.prefix)
            if idx < 0:
                continue
            total_profile_lines += 1
            segment = line[idx + len(args.prefix) :]
            kv = parse_kv(segment)
            stage = kv.get("stage", "unknown")
            status = kv.get("status")
            duration_ms = pick_duration_ms(kv)
            stats.setdefault(stage, StageStat()).add(status, duration_ms)

    if total_profile_lines == 0:
        print("no profiling lines found")
        return 0

    ranked = sorted(
        stats.items(),
        key=lambda item: (item[1].total(), item[1].avg()),
        reverse=True,
    )

    print(f"profile_lines={total_profile_lines} stages={len(stats)}")
    print(
        "stage,count,error_count,total_ms,avg_ms,p95_ms,max_ms,duration_samples"
    )
    for stage, stat in ranked[: args.top]:
        print(
            f"{stage},{stat.count},{stat.error_count},"
            f"{stat.total():.2f},{stat.avg():.2f},{stat.p95():.2f},{stat.max():.2f},"
            f"{len(stat.durations_ms)}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

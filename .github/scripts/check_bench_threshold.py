#!/usr/bin/env python3
"""Fail CI if criterion benchmark mean exceeds configured threshold."""

from __future__ import annotations

import json
import os
import pathlib
import sys


def main() -> int:
    bench_name = os.getenv("BENCH_NAME", "parse_and_store_64kb_file")
    max_mean_ns = float(os.getenv("BENCH_MAX_MEAN_NS", "50000000"))
    root = pathlib.Path("target/criterion")

    candidates = list(root.rglob("estimates.json"))
    if not candidates:
        print("No criterion estimates found under target/criterion")
        return 1

    selected = None
    for path in candidates:
        if bench_name in str(path):
            selected = path
            break

    if selected is None:
        print(f"Could not find estimates.json for benchmark '{bench_name}'")
        return 1

    with selected.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)

    try:
        mean_ns = float(payload["mean"]["point_estimate"])
    except (KeyError, TypeError, ValueError) as exc:
        print(f"Invalid criterion estimates format in {selected}: {exc}")
        return 1

    print(f"Benchmark: {bench_name}")
    print(f"Mean (ns): {mean_ns:.0f}")
    print(f"Max  (ns): {max_mean_ns:.0f}")

    if mean_ns > max_mean_ns:
        print("Benchmark mean exceeded threshold")
        return 1

    print("Benchmark threshold check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())


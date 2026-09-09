#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
WARN_LINES = 600
FAIL_LINES = 1000

EXCLUDED_PARTS = {
    ".git",
    "target",
    "vendor",
    "vendors",
    "generated",
    "migrations",
    "fixtures",
}

# This is one database-backed acceptance harness, not a production ownership
# unit. Splitting it solely by line count would duplicate the embedded
# PostgreSQL setup and reduce the signal of the scenario suite.
EXCLUDED_FILES = {
    Path("apps/nous-wave/tests/authority_semantics.rs"),
}

def excluded(path: Path) -> bool:
    rel = path.relative_to(ROOT)
    return rel in EXCLUDED_FILES or any(part in EXCLUDED_PARTS for part in rel.parts)

def physical_lines(path: Path) -> int:
    with path.open("rb") as f:
        return sum(1 for _ in f)

def main() -> int:
    warnings: list[tuple[int, Path]] = []
    failures: list[tuple[int, Path]] = []

    for path in sorted(ROOT.rglob("*.rs")):
        if excluded(path):
            continue
        count = physical_lines(path)
        if count > FAIL_LINES:
            failures.append((count, path))
        elif count > WARN_LINES:
            warnings.append((count, path))

    for count, path in warnings:
        print(f"WARN source-shape {count:5d} lines  {path.relative_to(ROOT)}")

    for count, path in failures:
        print(f"FAIL source-shape {count:5d} lines  {path.relative_to(ROOT)}")

    if failures:
        print(
            f"\n{len(failures)} Rust source file(s) exceed the hard {FAIL_LINES}-line limit.",
            file=sys.stderr,
        )
        return 1

    print(
        f"source-shape OK "
        f"({len(warnings)} file(s) above warning threshold {WARN_LINES}; "
        f"none above hard limit {FAIL_LINES})"
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

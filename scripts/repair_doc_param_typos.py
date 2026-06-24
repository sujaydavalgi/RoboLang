#!/usr/bin/env python3
"""Repair truncated parameter names in structured // doc comments."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# First character was lost when legacy single-line comments were replaced.
REPAIRS = (
    (re.compile(r"(//\s+)rl:"), r"\1url:"),
    (re.compile(r"(//\s+)esh_url:"), r"\1mesh_url:"),
    (re.compile(r"(//\s+)_reques:"), r"\1_request:"),
    (re.compile(r"(//\s+)reques:"), r"\1request:"),
    (re.compile(r"(//\s+)_?oken:"), r"\1token:"),
    (re.compile(r"Caller-supplied rl\b"), "Caller-supplied url"),
    (re.compile(r"Caller-supplied esh url"), "Caller-supplied mesh url"),
    (re.compile(r"Caller-supplied reques\b"), "Caller-supplied request"),
    (re.compile(r"Caller-supplied oken\b"), "Caller-supplied token"),
    (re.compile(r"::parse_http_url\(rl\)"), "::parse_http_url(url)"),
    (re.compile(r"::relay_recovery_via_mesh\(esh_url"), "::relay_recovery_via_mesh(mesh_url"),
)


def repair_file(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    original = text
    fixes = 0
    for pattern, repl in REPAIRS:
        text, n = pattern.subn(repl, text)
        fixes += n
    if text != original:
        path.write_text(text, encoding="utf-8")
    return fixes


def main() -> int:
    total = 0
    changed = 0
    for ext in ("*.rs", "*.ts"):
        for path in sorted(ROOT.rglob(ext)):
            if any(p in path.parts for p in ("target", "node_modules", "dist")):
                continue
            n = repair_file(path)
            if n:
                changed += 1
                total += n
                print(f"repaired {path.relative_to(ROOT)} ({n})")
    print(f"\nDone. {total} repairs in {changed} files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

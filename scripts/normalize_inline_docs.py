#!/usr/bin/env python3
"""Remove extra blank lines between consecutive // doc comment lines."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DOC_LINE = re.compile(r"^(\s*)//")


def normalize_doc_gaps(text: str) -> tuple[str, int]:
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    fixes = 0
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.rstrip().endswith("{") and i + 2 < len(lines):
            if lines[i + 1].strip() == "" and DOC_LINE.match(lines[i + 2]):
                out.append(line)
                i += 2
                fixes += 1
                continue
        if DOC_LINE.match(line):
            block: list[str] = [line]
            i += 1
            while i < len(lines):
                nxt = lines[i]
                if nxt.strip() == "":
                    if i + 1 < len(lines) and DOC_LINE.match(lines[i + 1]):
                        fixes += 1
                        i += 1
                        continue
                    block.append(nxt)
                    i += 1
                    break
                if DOC_LINE.match(nxt):
                    block.append(nxt)
                    i += 1
                    continue
                break
            out.extend(block)
            continue
        out.append(line)
        i += 1
    return "".join(out), fixes


def main() -> int:
    changed = 0
    total_fixes = 0
    for ext in ("*.rs", "*.ts"):
        for path in sorted(ROOT.rglob(ext)):
            if any(p in path.parts for p in ("target", "node_modules", "dist")):
                continue
            original = path.read_text(encoding="utf-8")
            text, fixes = normalize_doc_gaps(original)
            if fixes and text != original:
                path.write_text(text, encoding="utf-8")
                changed += 1
                total_fixes += fixes
                print(f"normalized {path.relative_to(ROOT)} ({fixes} gaps removed)")
    print(f"\nDone. Updated {changed} files, removed {total_fixes} extra blank lines.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

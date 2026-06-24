#!/usr/bin/env python3
"""Normalize inline // doc comments: gaps and block-comment indentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DOC_LINE = re.compile(r"^(\s*)//")
EXAMPLE_USE_LINE = re.compile(r"^\s*// use (?:crate::|\w+::)")


def is_example_use_line(line: str) -> bool:
    return EXAMPLE_USE_LINE.match(line) is not None

API_MARKERS = (
    "Description:",
    "Inputs:",
    "Outputs:",
    "Parameters:",
    "Returns:",
    "Options:",
    "Example:",
    "Logic:",
    "None.",
)

TOOLING_SCRIPTS = {
    "doc_validation_lib.py",
    "validate_documentation.py",
    "migrate_legacy_inline_docs.py",
    "add_structured_api_docs.py",
    "add_inline_docs.py",
    "add_logic_block_docs.py",
    "normalize_inline_docs.py",
    "repair_doc_corruption.py",
}


def is_api_marker_line(line: str) -> bool:
    stripped = line.strip()
    if stripped in {"//", "// -"}:
        return True
    if stripped.startswith("// - `"):
        return True
    return any(stripped == f"// {m}" or stripped.startswith(f"// {m} ") for m in API_MARKERS)


def is_block_explanation(line: str) -> bool:
    stripped = line.strip()
    if not stripped.startswith("//"):
        return False
    if is_api_marker_line(line):
        return False
    if stripped.startswith("// let result =") or is_example_use_line(line):
        return False
    return True


def leading_ws(line: str) -> str:
    return line[: len(line) - len(line.lstrip(" \t"))]


def is_logic_block_comment(lines: list[str], index: int) -> bool:
    line = lines[index]
    if not is_block_explanation(line):
        return False
    j = index + 1
    while j < len(lines) and lines[j].strip() == "":
        j += 1
    if j >= len(lines):
        return False
    return not lines[j].strip().startswith("//")


def precedes_block_comment(prev_line: str) -> bool:
    stripped = prev_line.strip()
    if not stripped.startswith("//"):
        return True
    if is_block_explanation(prev_line):
        return False
    return True


def ensure_blank_before_block_comments(text: str) -> tuple[str, int]:
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    fixes = 0
    for i, line in enumerate(lines):
        if is_logic_block_comment(lines, i):
            prev_idx = len(out) - 1
            while prev_idx >= 0 and out[prev_idx].strip() == "":
                prev_idx -= 1
            if prev_idx >= 0 and precedes_block_comment(out[prev_idx]):
                blank_count = len(out) - 1 - prev_idx
                if blank_count == 0:
                    out.append("\n")
                    fixes += 1
                elif blank_count > 1:
                    del out[prev_idx + 1 :]
                    out.append("\n")
                    fixes += blank_count - 1
        out.append(line)
    return "".join(out), fixes


def align_block_comment_indent(text: str) -> tuple[str, int]:
    lines = text.splitlines(keepends=True)
    fixes = 0
    for i in range(len(lines)):
        line = lines[i]
        if not is_logic_block_comment(lines, i):
            continue
        j = i + 1
        while j < len(lines) and lines[j].strip() == "":
            j += 1
        if j >= len(lines):
            continue
        nxt = lines[j]
        if nxt.strip().startswith("//"):
            continue
        target = leading_ws(nxt)
        current = leading_ws(line)
        if current == target:
            continue
        body = line.strip()[2:].strip()
        nl = "\n" if line.endswith("\n") else ""
        lines[i] = f"{target}// {body}{nl}"
        fixes += 1
    return "".join(lines), fixes


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
        if line.strip() == "":
            peek = i + 1
            while peek < len(lines) and lines[peek].strip() == "":
                peek += 1
            if peek < len(lines) and DOC_LINE.match(lines[peek]):
                if is_logic_block_comment(lines, peek):
                    if out and out[-1].strip() == "":
                        fixes += 1
                        i += 1
                        continue
                    out.append(line)
                    i += 1
                    continue
                fixes += peek - i
                i = peek
                continue
            if out and out[-1].strip() == "":
                fixes += 1
                i += 1
                continue
            out.append(line)
            i += 1
            continue
        if DOC_LINE.match(line):
            block: list[str] = [line]
            i += 1
            while i < len(lines):
                nxt = lines[i]
                if nxt.strip() == "":
                    peek = i + 1
                    while peek < len(lines) and lines[peek].strip() == "":
                        peek += 1
                    if peek < len(lines) and DOC_LINE.match(lines[peek]):
                        if is_logic_block_comment(lines, peek):
                            block.append(nxt)
                            i += 1
                            break
                        fixes += peek - i
                        i = peek
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


def normalize_file(text: str) -> tuple[str, int, int, int]:
    text, gap_fixes = normalize_doc_gaps(text)
    text, indent_fixes = align_block_comment_indent(text)
    text, blank_fixes = ensure_blank_before_block_comments(text)
    return text, gap_fixes, indent_fixes, blank_fixes


def main() -> int:
    changed = 0
    total_gaps = 0
    total_indents = 0
    total_blanks = 0
    for ext in ("*.rs", "*.ts"):
        for path in sorted(ROOT.rglob(ext)):
            if path.name in TOOLING_SCRIPTS:
                continue
            if any(p in path.parts for p in ("target", "node_modules", "dist")):
                continue
            original = path.read_text(encoding="utf-8")
            text, gaps, indents, blanks = normalize_file(original)
            if text != original:
                path.write_text(text, encoding="utf-8")
                changed += 1
                total_gaps += gaps
                total_indents += indents
                total_blanks += blanks
                parts = []
                if gaps:
                    parts.append(f"{gaps} gaps")
                if indents:
                    parts.append(f"{indents} indents")
                if blanks:
                    parts.append(f"{blanks} blanks")
                print(f"normalized {path.relative_to(ROOT)} ({', '.join(parts)})")
    print(
        f"\nDone. Updated {changed} files, "
        f"removed {total_gaps} extra blank lines, "
        f"fixed {total_indents} comment indents, "
        f"added {total_blanks} blanks before block comments."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

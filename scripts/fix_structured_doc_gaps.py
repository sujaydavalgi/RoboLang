#!/usr/bin/env python3
"""Fix incomplete structured documentation (empty Inputs, legacy single-line comments)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from doc_validation_lib import (
    ROOT,
    TOOLING_SCRIPTS,
    assess_callable,
    extract_body_doc_block,
    find_py_functions,
    find_rust_functions,
    find_ts_arrows,
    find_ts_callables,
    should_scan,
)
from migrate_legacy_inline_docs import build_structured_block, module_hint_from_path

EMPTY_INPUTS = re.compile(
    r"(?m)^(\s*//\s*Inputs:\s*)\n(\s*//\s*)\n(\s*//\s*Outputs:)",
    re.MULTILINE,
)

LEGACY_LEAD = re.compile(
    r"(?m)^(\s*//\s+)(?!Description:|Inputs:|Outputs:|Example:)([A-Z][^\n]*\.)\s*\n(\s*//\s+(?!Description:))",
)


def fix_empty_inputs(text: str) -> tuple[str, int]:
    count = 0

    def repl(m: re.Match[str]) -> str:
        nonlocal count
        count += 1
        return f"{m.group(1)}\n{m.group(2)}     None.\n{m.group(2)}\n{m.group(3)}"

    return EMPTY_INPUTS.sub(repl, text), count


def is_legacy_lead_only(doc: str) -> bool:
    if "Description:" in doc:
        return False
    lines = [ln.strip() for ln in doc.splitlines() if ln.strip()]
    return bool(lines) and all(
        not ln.startswith(("Inputs:", "Outputs:", "Example:", "Parameters:", "Returns:"))
        for ln in lines
    )


def replace_body_lead_doc(text: str, body_start: int, language: str, new_block: str) -> str:
    pos = body_start
    end = body_start
    while pos < len(text):
        line_end = text.find("\n", pos)
        if line_end == -1:
            line_end = len(text)
        line = text[pos:line_end]
        stripped = line.strip()
        if not stripped:
            if end > body_start:
                break
            pos = line_end + 1
            continue
        if language != "python" and stripped.startswith("//"):
            if stripped.startswith("// Description:") or any(
                stripped.startswith(f"// {s}")
                for s in ("Inputs:", "Outputs:", "Example:")
            ):
                break
            end = line_end + 1
            pos = line_end + 1
            continue
        break
    return text[:body_start] + "\n" + new_block + text[end:]


def process_rust(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    text, _ = fix_empty_inputs(text)
    module_hint = module_hint_from_path(path)
    for fm in reversed(find_rust_functions(text, path)):
        if assess_callable(text, fm).documented:
            continue
        body_doc = extract_body_doc_block(text, fm.body_start, "rust")
        if is_legacy_lead_only(body_doc) or (
            not body_doc.strip() and fm.preceding_doc and not assess_callable(text, fm).documented
        ):
            block = build_structured_block(
                fm.indent + "    ", fm.name, fm.params, fm.ret, "rust", module_hint
            )
            text = replace_body_lead_doc(text, fm.body_start, "rust", block)
        elif body_doc.strip() and not assess_callable(text, fm).documented:
            # Partial structured doc — replace entire leading block
            if "Description:" in body_doc or "Inputs:" in body_doc:
                block = build_structured_block(
                    fm.indent + "    ", fm.name, fm.params, fm.ret, "rust", module_hint
                )
                text = replace_body_lead_doc(text, fm.body_start, "rust", block)
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def process_ts(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    text, _ = fix_empty_inputs(text)
    module_hint = module_hint_from_path(path)
    callables = (
        find_ts_callables(text, path, False)
        + find_ts_callables(text, path, True)
        + find_ts_arrows(text, path)
    )
    seen: set[tuple[int, str]] = set()
    for fm in reversed(callables):
        key = (fm.line, fm.name)
        if key in seen:
            continue
        seen.add(key)
        if assess_callable(text, fm).documented:
            continue
        body_doc = extract_body_doc_block(text, fm.body_start, "typescript")
        if fm.preceding_doc and "@param" in (fm.preceding_doc or ""):
            continue
        if is_legacy_lead_only(body_doc) or (
            not body_doc.strip() and fm.preceding_doc
        ):
            block = build_structured_block(
                fm.indent + "  ", fm.name, fm.params, fm.ret, "typescript", module_hint
            )
            text = replace_body_lead_doc(text, fm.body_start, "typescript", block)
        elif body_doc.strip() and not assess_callable(text, fm).documented:
            if "Description:" in body_doc or "Inputs:" in body_doc:
                block = build_structured_block(
                    fm.indent + "  ", fm.name, fm.params, fm.ret, "typescript", module_hint
                )
                text = replace_body_lead_doc(text, fm.body_start, "typescript", block)
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def process_py(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    for fm in find_py_functions(text, path):
        if assess_callable(text, fm).documented:
            continue
    return False


def main() -> int:
    changed = 0
    fixed_inputs = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in TOOLING_SCRIPTS:
            continue
        lang = should_scan(path)
        if lang in {"rust", "typescript"}:
            raw = path.read_text(encoding="utf-8")
            patched, n = fix_empty_inputs(raw)
            if n and patched != raw:
                path.write_text(patched, encoding="utf-8")
                fixed_inputs += n
                changed += 1
        if lang == "rust" and process_rust(path):
            changed += 1
            print(f"fixed rust: {path.relative_to(ROOT)}")
        elif lang == "typescript" and process_ts(path):
            changed += 1
            print(f"fixed ts: {path.relative_to(ROOT)}")
    print(f"\nDone. Updated {changed} files, fixed {fixed_inputs} empty Inputs sections.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

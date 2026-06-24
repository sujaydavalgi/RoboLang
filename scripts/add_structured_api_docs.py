#!/usr/bin/env python3
"""Add structured Description/Inputs/Outputs/Example docs to undocumented callables."""

from __future__ import annotations

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
from migrate_legacy_inline_docs import (
    build_python_docstring,
    build_structured_block,
    module_hint_from_path,
)


def has_complete_doc(text: str, fm) -> bool:
    if fm.preceding_doc and assess_callable(text, fm).documented:
        return True
    body = extract_body_doc_block(text, fm.body_start, fm.language)
    if not body.strip() and not fm.preceding_doc:
        return False
    return assess_callable(text, fm).documented


def process_rust(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    module_hint = module_hint_from_path(path)
    inserts: list[tuple[int, str]] = []
    for fm in find_rust_functions(text, path):
        if has_complete_doc(text, fm):
            continue
        body = extract_body_doc_block(text, fm.body_start, "rust")
        if body.strip() and assess_callable(text, fm).has_any_doc:
            continue
        block = build_structured_block(
            fm.indent + "    ", fm.name, fm.params, fm.ret, "rust", module_hint
        )
        inserts.append((fm.body_start, block))
    for pos, block in reversed(inserts):
        text = text[:pos] + "\n" + block + text[pos:]
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def process_ts(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    module_hint = module_hint_from_path(path)
    inserts: list[tuple[int, str]] = []
    callables = (
        find_ts_callables(text, path, False)
        + find_ts_callables(text, path, True)
        + find_ts_arrows(text, path)
    )
    seen: set[tuple[int, str]] = set()
    for fm in callables:
        key = (fm.line, fm.name)
        if key in seen:
            continue
        seen.add(key)
        if has_complete_doc(text, fm):
            continue
        body = extract_body_doc_block(text, fm.body_start, "typescript")
        if body.strip() and fm.preceding_doc and assess_callable(text, fm).has_any_doc:
            continue
        block = build_structured_block(
            fm.indent + "  ", fm.name, fm.params, fm.ret, "typescript", module_hint
        )
        inserts.append((fm.body_start, block))
    for pos, block in reversed(inserts):
        text = text[:pos] + "\n" + block + text[pos:]
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def process_py(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    module_hint = path.stem
    inserts: list[tuple[int, str]] = []
    for fm in find_py_functions(text, path):
        doc = extract_body_doc_block(text, fm.body_start, "python")
        if doc.strip():
            continue
        py_doc = build_python_docstring(
            fm.indent, fm.name, fm.params, fm.ret, module_hint
        )
        inserts.append((fm.body_start, py_doc))
    for pos, block in reversed(inserts):
        text = text[:pos] + block + text[pos:]
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def main() -> int:
    changed = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in TOOLING_SCRIPTS:
            continue
        lang = should_scan(path)
        if lang == "rust" and process_rust(path):
            changed += 1
            print(f"updated rust: {path.relative_to(ROOT)}")
        elif lang == "typescript" and process_ts(path):
            changed += 1
            print(f"updated ts: {path.relative_to(ROOT)}")
        elif lang == "python" and process_py(path):
            changed += 1
            print(f"updated python: {path.relative_to(ROOT)}")
    print(f"\nDone. Updated {changed} files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

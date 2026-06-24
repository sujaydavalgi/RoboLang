#!/usr/bin/env python3
"""Migrate legacy Parameters/Returns inline docs to Description/Inputs/Outputs format."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from doc_validation_lib import (
    ROOT,
    TOOLING_SCRIPTS,
    extract_body_doc_block,
    find_rust_functions,
    find_ts_arrows,
    find_ts_callables,
    should_scan,
)


def snake_to_words(name: str) -> str:
    return re.sub(r"_+", " ", name).strip()


def describe_name(name: str) -> str:
    if name == "new":
        return "Construct a new instance"
    if name == "default":
        return "Provide the default value for this type"
    words = snake_to_words(name)
    if words.startswith("is ") or words.startswith("has "):
        return words[0].upper() + words[1:]
    return words[0].upper() + words[1:] if words else name


def collapse_type(typ: str) -> str:
    return re.sub(r"\s+", " ", typ.replace("\n", " ")).strip()


def build_structured_block(
    indent: str,
    name: str,
    params: list[tuple[str, str]],
    ret: str | None,
    lang: str,
    module_hint: str,
) -> str:
    desc = describe_name(name)
    lines = [f"{indent}// Description:", f"{indent}//     {desc}."]
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Inputs:")
    non_self = [p for p in params if p[0] not in {"self", "cls"}]
    if non_self:
        for pname, ptype in non_self:
            hint = collapse_type("value" if ptype == "value" else ptype)
            lines.append(f"{indent}//     {pname}: {hint}")
            lines.append(f"{indent}//         Caller-supplied {snake_to_words(pname)}.")
    else:
        lines.append(f"{indent}//     None.")
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Outputs:")
    out_type = collapse_type(ret.strip() if ret else ("void" if lang != "rust" else "()"))
    if out_type in {"()", "void", "None"}:
        lines.append(f"{indent}//     None.")
    else:
        lines.append(f"{indent}//     result: {out_type}")
        lines.append(f"{indent}//         Return value from `{name}`.")
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Example:")
    call_params = ", ".join(p[0] for p in non_self)
    if lang == "rust":
        if name == "new":
            ex = f"let value = {module_hint}::new({call_params});"
        elif params and params[0][0] == "self":
            ex = f"let result = instance.{name}({call_params});"
        else:
            ex = f"let result = {module_hint}::{name}({call_params});"
    else:
        ex = f"const result = {name}({call_params});"
    lines.append(f"{indent}//     {ex}")
    return "\n".join(lines) + "\n"


def build_python_docstring(
    indent: str,
    name: str,
    params: list[tuple[str, str]],
    ret: str | None,
    module_hint: str,
) -> str:
    block = build_structured_block(indent + "    ", name, params, ret, "python", module_hint)
    py_lines = [f'{indent}    """']
    for line in block.splitlines():
        content = line.strip()
        if content.startswith("//"):
            content = content[2:].strip()
        py_lines.append(f"{indent}    {content}" if content else f"{indent}")
    py_lines.append(f'{indent}    """')
    return "\n".join(py_lines) + "\n"


def module_hint_from_path(path: Path) -> str:
    parts = list(path.parts)
    if "crates" in parts:
        idx = parts.index("crates")
        crate = parts[idx + 1].replace("-", "_")
        if path.name == "lib.rs":
            return crate
        return f"{crate}::{path.stem.replace('-', '_')}"
    return path.stem


def is_legacy_block(doc: str) -> bool:
    return (
        "Parameters:" in doc
        and "Returns:" in doc
        and "Description:" not in doc
        and "Inputs:" not in doc
    )


def replace_legacy_block(text: str, body_start: int, new_block: str) -> str:
    start = body_start
    end = start
    pos = body_start
    while pos < len(text):
        line_end = text.find("\n", pos)
        if line_end == -1:
            line_end = len(text)
        line = text[pos:line_end]
        stripped = line.strip()
        if not stripped:
            pos = line_end + 1
            continue
        if stripped.startswith("//"):
            end = line_end + 1
            pos = line_end + 1
            continue
        break
    if end <= start:
        return text
    return text[:start] + "\n" + new_block + text[end:]


def process_rust(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    module_hint = module_hint_from_path(path)
    for fm in reversed(find_rust_functions(text, path)):
        doc = extract_body_doc_block(text, fm.body_start, "rust")
        if not is_legacy_block(doc):
            continue
        block = build_structured_block(
            fm.indent + "    ", fm.name, fm.params, fm.ret, "rust", module_hint
        )
        text = replace_legacy_block(text, fm.body_start, block)
    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def process_ts(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    original = text
    module_hint = module_hint_from_path(path)
    callables = (
        find_ts_callables(text, path, False)
        + find_ts_callables(text, path, True)
        + find_ts_arrows(text, path)
    )
    for fm in reversed(callables):
        doc = extract_body_doc_block(text, fm.body_start, "typescript")
        if not is_legacy_block(doc):
            continue
        block = build_structured_block(
            fm.indent + "  ", fm.name, fm.params, fm.ret, "typescript", module_hint
        )
        text = replace_legacy_block(text, fm.body_start, block)
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
            print(f"migrated rust: {path.relative_to(ROOT)}")
        elif lang == "typescript" and process_ts(path):
            changed += 1
            print(f"migrated ts: {path.relative_to(ROOT)}")
    print(f"\nDone. Migrated {changed} files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

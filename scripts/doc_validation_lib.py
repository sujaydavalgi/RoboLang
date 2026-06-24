#!/usr/bin/env python3
"""Shared documentation scanning and validation for Spanda coding standards."""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SKIP_PATH_PARTS = {
    "target",
    "node_modules",
    ".git",
    "dist",
    "build",
    "out",
    ".next",
    "coverage",
}
SKIP_FILE_SUFFIXES = {".min.js", ".d.ts", ".map"}
TOOLING_SCRIPTS = {
    "doc_validation_lib.py",
    "validate_documentation.py",
    "migrate_legacy_inline_docs.py",
    "add_structured_api_docs.py",
    "add_inline_docs.py",
    "add_logic_block_docs.py",
    "normalize_inline_docs.py",
    "repair_doc_corruption.py",
    "fix_structured_doc_gaps.py",
    "repair_doc_param_typos.py",
}

RUST_FN_HEAD = re.compile(
    r"(?m)^(?P<indent>\s*)"
    r"(?:(?:pub\s*\([^)]*\)\s+|pub\s+|async\s+|unsafe\s+|const\s+)*)"
    r"fn\s+(?P<name>\w+)\s*"
)

TS_FN_HEAD = re.compile(
    r"(?m)^(?P<indent>\s*)"
    r"(?:(?:export\s+|async\s+|static\s+)*)"
    r"function\s+(?P<name>\w+)\s*"
)

TS_METHOD_HEAD = re.compile(
    r"(?m)^(?P<indent>\s+)"
    r"(?:(?:public|private|protected|static|async|readonly)\s+)+"
    r"(?P<name>[a-zA-Z_]\w*)\s*"
)

TS_ARROW = re.compile(
    r"(?m)^(?P<indent>\s*)"
    r"(?:(?:export\s+|async\s+)*)"
    r"(?:const|let)\s+(?P<name>\w+)\s*=\s*"
    r"(?:async\s+)?"
    r"\((?P<params>[^)]*)\)"
    r"(?:\s*:\s*(?P<ret>[^=]+))?"
    r"\s*=>\s*\{"
)

PY_DEF = re.compile(r"(?m)^(?P<indent>\s*)def\s+(?P<name>\w+)\s*\(")

SECTION_ALIASES = {
    "description": "description",
    "purpose": "description",
    "inputs": "inputs",
    "parameters": "inputs",
    "input": "inputs",
    "outputs": "outputs",
    "returns": "outputs",
    "return": "outputs",
    "example": "example",
    "examples": "example",
    "safety": "safety",
    "errors": "errors",
    "notes": "notes",
    "options": "options",
}


@dataclass
class CallableMatch:
    path: Path
    language: str
    name: str
    line: int
    is_public: bool
    params: list[tuple[str, str]]
    ret: str | None
    body_start: int
    indent: str
    preceding_doc: str | None = None


@dataclass
class DocAssessment:
    match: CallableMatch
    sections: dict[str, bool] = field(default_factory=dict)
    missing: list[str] = field(default_factory=list)
    legacy_only: bool = False

    @property
    def documented(self) -> bool:
        required = ("description", "inputs", "outputs", "example")
        return all(self.sections.get(s, False) for s in required)

    @property
    def has_any_doc(self) -> bool:
        return bool(self.sections)


def skip_string_literal(text: str, i: int) -> int:
    if i >= len(text):
        return i
    if text.startswith("r#", i):
        hash_count = 0
        j = i + 1
        while j < len(text) and text[j] == "#":
            hash_count += 1
            j += 1
        if j < len(text) and text[j] in "\"'":
            quote = text[j]
            j += 1
            while j < len(text):
                if text[j] == quote:
                    if hash_count == 0:
                        break
                    if text[j + 1 : j + 1 + hash_count] == "#" * hash_count:
                        j += hash_count
                        break
                j += 1
            return j
    if text[i] in "\"'":
        quote = text[i]
        j = i + 1
        while j < len(text):
            if text[j] == "\\":
                j += 2
                continue
            if text[j] == quote:
                return j
            j += 1
        return j
    return i


def scan_balanced(text: str, start: int, open_ch: str, close_ch: str) -> int | None:
    if start >= len(text) or text[start] != open_ch:
        return None
    depth = 0
    i = start
    while i < len(text):
        if text.startswith("//", i):
            i = text.find("\n", i)
            if i == -1:
                return None
            continue
        if text.startswith("/*", i):
            end = text.find("*/", i + 2)
            if end == -1:
                return None
            i = end + 2
            continue
        ch = text[i]
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return i
        elif ch in "\"'" or (
            text.startswith("r", i) and i + 1 < len(text) and text[i + 1] in "'\""
        ):
            i = skip_string_literal(text, i)
        i += 1
    return None


def line_number(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def parse_rust_params(params: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    if not params.strip():
        return out
    depth = 0
    chunk = ""
    for ch in params + ",":
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            part = chunk.strip()
            chunk = ""
            if not part:
                continue
            if part == "self" or part.startswith("self:"):
                if part.startswith("self"):
                    out.append(("self", "receiver"))
                continue
            if ":" in part:
                name, typ = part.split(":", 1)
                out.append((name.strip().strip("&mut ").strip("&"), typ.strip()))
            else:
                out.append((part, "value"))
            continue
        chunk += ch
    return out


def parse_ts_params(params: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    if not params.strip():
        return out
    depth = 0
    chunk = ""
    for ch in params + ",":
        if ch in "(<[{":
            depth += 1
        elif ch in ")>}]":
            depth -= 1
        if ch == "," and depth == 0:
            part = chunk.strip()
            chunk = ""
            if not part:
                continue
            if ":" in part:
                name, typ = part.split(":", 1)
                name = name.strip()
                if name.startswith("..."):
                    name = name[3:]
                out.append((name, typ.strip()))
            else:
                out.append((part, "value"))
            continue
        chunk += ch
    return out


def parse_py_params(params: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    if not params.strip():
        return out
    depth = 0
    chunk = ""
    for ch in params + ",":
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            part = chunk.strip()
            chunk = ""
            if not part or part in {"self", "cls"}:
                if part:
                    out.append((part, "receiver"))
                continue
            name = part.split(":", 1)[0].split("=", 1)[0].strip()
            typ = part.split(":", 1)[1].strip() if ":" in part else "value"
            if "=" in typ:
                typ = typ.split("=", 1)[0].strip()
            out.append((name, typ))
            continue
        chunk += ch
    return out


def find_rust_functions(text: str, path: Path) -> list[CallableMatch]:
    matches: list[CallableMatch] = []
    for m in RUST_FN_HEAD.finditer(text):
        header = text[max(0, m.start() - 200) : m.start()]
        is_public = bool(re.search(r"(?m)^\s*pub\s", header + m.group(0)[:20]))
        pos = m.end()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos < len(text) and text[pos] == "<":
            end = scan_balanced(text, pos, "<", ">")
            if end is None:
                continue
            pos = end + 1
            while pos < len(text) and text[pos].isspace():
                pos += 1
        if pos >= len(text) or text[pos] != "(":
            continue
        params_end = scan_balanced(text, pos, "(", ")")
        if params_end is None:
            continue
        params = text[pos + 1 : params_end]
        pos = params_end + 1
        while pos < len(text) and text[pos].isspace():
            pos += 1
        ret: str | None = None
        if text.startswith("->", pos):
            pos += 2
            ret_start = pos
            while pos < len(text) and text[pos] not in "{;":
                pos += 1
            ret = text[ret_start:pos].strip()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text) or text[pos] != "{":
            continue
        preceding = extract_preceding_rust_doc(text, m.start())
        matches.append(
            CallableMatch(
                path=path,
                language="rust",
                name=m.group("name"),
                line=line_number(text, m.start()),
                is_public=is_public,
                params=parse_rust_params(params),
                ret=ret,
                body_start=pos + 1,
                indent=m.group("indent"),
                preceding_doc=preceding,
            )
        )
    return matches


def extract_preceding_rust_doc(text: str, fn_start: int) -> str | None:
    lines = text[:fn_start].splitlines()
    doc_lines: list[str] = []
    i = len(lines) - 1
    while i >= 0:
        line = lines[i]
        stripped = line.strip()
        if not stripped:
            if doc_lines:
                break
            i -= 1
            continue
        if stripped.startswith("///") or stripped.startswith("//!"):
            doc_lines.insert(0, stripped.lstrip("/").strip())
            i -= 1
            continue
        if stripped.startswith("#[") or stripped.startswith("@"):
            i -= 1
            continue
        break
    return "\n".join(doc_lines) if doc_lines else None


def extract_preceding_ts_doc(text: str, fn_start: int) -> str | None:
    chunk = text[:fn_start]
    m = re.search(r"/\*\*[\s\S]*?\*/\s*$", chunk)
    if m:
        return m.group(0)
    lines = chunk.splitlines()
    doc_lines: list[str] = []
    i = len(lines) - 1
    while i >= 0:
        line = lines[i].strip()
        if not line:
            if doc_lines:
                break
            i -= 1
            continue
        if line.startswith("//"):
            doc_lines.insert(0, line[2:].strip())
            i -= 1
            continue
        break
    return "\n".join(doc_lines) if doc_lines else None


def find_ts_callables(text: str, path: Path, is_method: bool) -> list[CallableMatch]:
    head = TS_METHOD_HEAD if is_method else TS_FN_HEAD
    matches: list[CallableMatch] = []
    for m in head.finditer(text):
        name = m.group("name")
        if name in {"if", "for", "while", "switch", "catch", "constructor", "super"}:
            continue
        header = text[max(0, m.start() - 80) : m.start() + 40]
        is_public = "export" in header or "public" in header
        pos = m.end()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text) or text[pos] != "(":
            continue
        params_end = scan_balanced(text, pos, "(", ")")
        if params_end is None:
            continue
        params = text[pos + 1 : params_end]
        pos = params_end + 1
        while pos < len(text) and text[pos].isspace():
            pos += 1
        ret: str | None = None
        if text[pos:].startswith(":"):
            pos += 1
            ret_start = pos
            while pos < len(text) and text[pos] not in "{=":
                pos += 1
            ret = text[ret_start:pos].strip()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text):
            continue
        if text.startswith("=>", pos):
            pos += 2
            while pos < len(text) and text[pos].isspace():
                pos += 1
        if pos >= len(text) or text[pos] != "{":
            continue
        preceding = extract_preceding_ts_doc(text, m.start())
        matches.append(
            CallableMatch(
                path=path,
                language="typescript",
                name=name,
                line=line_number(text, m.start()),
                is_public=is_public,
                params=parse_ts_params(params),
                ret=ret,
                body_start=pos + 1,
                indent=m.group("indent"),
                preceding_doc=preceding,
            )
        )
    return matches


def find_ts_arrows(text: str, path: Path) -> list[CallableMatch]:
    matches: list[CallableMatch] = []
    for m in TS_ARROW.finditer(text):
        header = text[max(0, m.start() - 40) : m.start()]
        is_public = "export" in header
        params = m.group("params") or ""
        ret = (m.group("ret") or "").strip() or None
        preceding = extract_preceding_ts_doc(text, m.start())
        matches.append(
            CallableMatch(
                path=path,
                language="typescript",
                name=m.group("name"),
                line=line_number(text, m.start()),
                is_public=is_public,
                params=parse_ts_params(params),
                ret=ret,
                body_start=m.end(),
                indent=m.group("indent"),
                preceding_doc=preceding,
            )
        )
    return matches


def find_py_functions(text: str, path: Path) -> list[CallableMatch]:
    matches: list[CallableMatch] = []
    for m in PY_DEF.finditer(text):
        pos = m.end() - 1
        params_end = scan_balanced(text, pos, "(", ")")
        if params_end is None:
            continue
        params = text[pos + 1 : params_end]
        pos = params_end + 1
        while pos < len(text) and text[pos].isspace():
            pos += 1
        ret: str | None = None
        if text.startswith("->", pos):
            pos += 2
            ret_start = pos
            while pos < len(text) and text[pos] not in ":{":
                pos += 1
            ret = text[ret_start:pos].strip()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text) or text[pos] != ":":
            continue
        body_start = text.find("\n", pos)
        if body_start == -1:
            continue
        body_start += 1
        is_public = not m.group("name").startswith("_")
        matches.append(
            CallableMatch(
                path=path,
                language="python",
                name=m.group("name"),
                line=line_number(text, m.start()),
                is_public=is_public,
                params=parse_py_params(params),
                ret=ret,
                body_start=body_start,
                indent=m.group("indent"),
                preceding_doc=None,
            )
        )
    return matches


def extract_body_doc_block(text: str, body_start: int, language: str) -> str:
    if language == "python":
        chunk = text[body_start : body_start + 4000]
        m = re.match(r'\s*("""|\'\'\')[\s\S]*?\1', chunk)
        if m:
            return m.group(0)
        return ""
    lines: list[str] = []
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
            lines.append(stripped[2:].strip())
            pos = line_end + 1
            continue
        break
    return "\n".join(lines)


def normalize_section_name(raw: str) -> str | None:
    key = raw.strip().rstrip(":").lower()
    return SECTION_ALIASES.get(key)


def parse_sections(doc_text: str, language: str) -> dict[str, bool]:
    sections = {k: False for k in ("description", "inputs", "outputs", "example")}
    if not doc_text.strip():
        return sections

    if language in {"rust", "typescript", "spanda"} and "@param" in doc_text:
        if re.search(r"@param\s+\w+", doc_text):
            sections["inputs"] = True
        if re.search(r"@returns?\s", doc_text, re.I):
            sections["outputs"] = True
        if re.search(r"@example", doc_text, re.I):
            sections["example"] = True
        if doc_text.strip():
            sections["description"] = True
        return sections

    lines = doc_text.splitlines()
    current: str | None = None
    has_body = {k: False for k in sections}

    for line in lines:
        raw = line.strip()
        if not raw:
            continue
        m = re.match(r"^([A-Za-z][A-Za-z ]*)\s*:\s*(.*)$", raw)
        if m:
            name = normalize_section_name(m.group(1))
            rest = m.group(2).strip()
            if name in sections:
                current = name
                if rest and rest.lower() not in {"none.", "none"}:
                    has_body[name] = True
                elif name == "inputs" and rest.lower() in {"none.", "none"}:
                    has_body[name] = True
                elif name == "outputs" and rest.lower() in {"none.", "none"}:
                    has_body[name] = True
                continue
        if current and raw not in {"None.", "None"}:
            has_body[current] = True
        elif not current and raw.endswith(".") and "description" not in has_body:
            sections["description"] = True

    for key, filled in has_body.items():
        if filled:
            sections[key] = True

    if re.search(r"(?m)^\s*Inputs\s*:\s*None\.?\s*$", doc_text, re.I):
        sections["inputs"] = True
    if re.search(r"(?m)^\s*Outputs\s*:\s*None\.?\s*$", doc_text, re.I):
        sections["outputs"] = True
    if re.search(r"(?m)^\s*Parameters\s*:", doc_text):
        sections["inputs"] = True
    if re.search(r"(?m)^\s*Returns\s*:", doc_text):
        sections["outputs"] = True
    if re.search(r"(?m)^\s*Example\s*:", doc_text):
        sections["example"] = True
    if sections["inputs"] and sections["outputs"] and not sections["description"]:
        first = next((ln.strip() for ln in lines if ln.strip() and not ln.strip().endswith(":")), "")
        if first and not re.match(r"^(Parameters|Returns|Options|Example)\s*:", first, re.I):
            sections["description"] = True

    return sections


def assess_callable(text: str, cm: CallableMatch) -> DocAssessment:
    body_doc = extract_body_doc_block(text, cm.body_start, cm.language)
    outer_doc = cm.preceding_doc or ""
    combined = f"{outer_doc}\n{body_doc}".strip()
    sections = parse_sections(combined, cm.language)
    legacy_only = (
        ("Parameters:" in combined or "Returns:" in combined)
        and "Description:" not in combined
        and "Inputs:" not in combined
    )
    missing: list[str] = []
    labels = {
        "description": "Description section",
        "inputs": "Inputs section",
        "outputs": "Outputs section",
        "example": "Example section",
    }
    for key, label in labels.items():
        if not sections.get(key):
            missing.append(label)
    if not combined.strip():
        missing.insert(0, "documentation block")
    return DocAssessment(
        match=cm,
        sections=sections,
        missing=missing,
        legacy_only=legacy_only,
    )


def module_key(path: Path) -> str:
    parts = list(path.parts)
    if "crates" in parts:
        idx = parts.index("crates")
        return f"rust/{parts[idx + 1]}"
    if "src" in parts and "packages" not in parts:
        idx = parts.index("src")
        sub = parts[idx + 1] if idx + 1 < len(parts) else "root"
        if sub.endswith(".ts"):
            return "ts/root"
        return f"ts/{sub.replace('.ts', '')}"
    if "packages" in parts:
        idx = parts.index("packages")
        sub = parts[idx + 1] if idx + 1 < len(parts) else "packages"
        return f"ts/{sub}"
    if "editor" in parts:
        return "ts/vscode"
    if "scripts" in parts:
        return "python/scripts"
    if path.suffix == ".sd":
        return "spanda/examples"
    return path.parent.name


def should_scan(path: Path) -> str | None:
    if path.name in TOOLING_SCRIPTS:
        return None
    if any(part in SKIP_PATH_PARTS for part in path.parts):
        return None
    if any(path.name.endswith(s) for s in SKIP_FILE_SUFFIXES):
        return None
    if path.suffix == ".rs" and "crates" in path.parts:
        return "rust"
    if path.suffix in {".ts", ".tsx"}:
        if any(p in path.parts for p in ("src", "packages", "editor", "tests")):
            if path.name.endswith(".test.ts") or path.name.endswith(".test.tsx"):
                return None
            return "typescript"
    if path.suffix == ".py" and "scripts" in path.parts:
        return "python"
    if path.suffix == ".sd":
        return "spanda"
    return None


def scan_file(path: Path) -> list[DocAssessment]:
    lang = should_scan(path)
    if not lang:
        return []
    text = path.read_text(encoding="utf-8", errors="replace")
    callables: list[CallableMatch] = []
    if lang == "rust":
        callables = find_rust_functions(text, path)
    elif lang == "typescript":
        callables = find_ts_callables(text, path, False)
        callables += find_ts_callables(text, path, True)
        callables += find_ts_arrows(text, path)
    elif lang == "python":
        callables = find_py_functions(text, path)
    elif lang == "spanda":
        callables = find_rust_functions(text, path)
        for c in callables:
            c.language = "spanda"
    seen: set[tuple[int, str]] = set()
    unique: list[CallableMatch] = []
    for c in callables:
        key = (c.line, c.name)
        if key in seen:
            continue
        seen.add(key)
        unique.append(c)
    return [assess_callable(text, c) for c in unique]


def scan_repository(root: Path = ROOT) -> list[DocAssessment]:
    results: list[DocAssessment] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if should_scan(path):
            results.extend(scan_file(path))
    return results


def render_coverage_report(assessments: list[DocAssessment]) -> str:
    total = len(assessments)
    documented = sum(1 for a in assessments if a.documented)
    undocumented = total - documented
    pct = (documented / total * 100) if total else 100.0

    by_module: dict[str, list[DocAssessment]] = {}
    by_lang: dict[str, list[DocAssessment]] = {}
    for a in assessments:
        key = module_key(a.match.path)
        by_module.setdefault(key, []).append(a)
        by_lang.setdefault(a.match.language, []).append(a)

    lines = [
        "# Documentation Coverage Report",
        "",
        f"Generated: {date.today().isoformat()}",
        "",
        "This report is produced by `scripts/validate_documentation.py`. "
        "See [coding-standards.md](./coding-standards.md) for the required docstring format.",
        "",
        "## Summary",
        "",
        "| Metric | Count |",
        "|--------|------:|",
        f"| Total methods / functions audited | {total} |",
        f"| Fully documented (structured standard) | {documented} |",
        f"| Undocumented or incomplete | {undocumented} |",
        f"| Coverage | {pct:.1f}% |",
        "",
        "## Coverage by module",
        "",
        "| Module | Total | Documented | Coverage |",
        "|--------|------:|-----------:|---------:|",
    ]
    for mod in sorted(by_module):
        items = by_module[mod]
        doc = sum(1 for a in items if a.documented)
        mod_pct = doc / len(items) * 100 if items else 100
        lines.append(f"| `{mod}` | {len(items)} | {doc} | {mod_pct:.1f}% |")

    lines.extend(
        [
            "",
            "## Coverage by language",
            "",
            "| Language | Total | Documented | Coverage |",
            "|----------|------:|-----------:|---------:|",
        ]
    )
    for lang in sorted(by_lang):
        items = by_lang[lang]
        doc = sum(1 for a in items if a.documented)
        lang_pct = doc / len(items) * 100 if items else 100
        lines.append(f"| {lang} | {len(items)} | {doc} | {lang_pct:.1f}% |")

    public_gaps = [a for a in assessments if a.match.is_public and not a.documented]
    lines.extend(
        [
            "",
            "## Remaining gaps (public APIs, sample)",
            "",
            "Public APIs missing one or more required sections. "
            "Run `python3 scripts/validate_documentation.py --warn` for the full list.",
            "",
        ]
    )
    for a in public_gaps[:100]:
        rel = a.match.path.relative_to(ROOT)
        miss = ", ".join(a.missing)
        lines.append(f"- `{rel}:{a.match.line}` `{a.match.name}` — {miss}")
    if len(public_gaps) > 100:
        lines.append(f"- … and {len(public_gaps) - 100} more public APIs")

    lines.extend(
        [
            "",
            "## CI enforcement",
            "",
            "CI runs `python3 scripts/validate_documentation.py --warn --report` on every pull request. "
            "Warnings are emitted for public APIs that lack structured documentation; builds do not fail yet.",
            "",
            "## Regenerating",
            "",
            "```bash",
            "python3 scripts/validate_documentation.py --report",
            "python3 scripts/migrate_legacy_inline_docs.py",
            "python3 scripts/add_structured_api_docs.py",
            "python3 scripts/normalize_inline_docs.py",
            "```",
            "",
        ]
    )
    return "\n".join(lines)

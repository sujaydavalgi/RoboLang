#!/usr/bin/env python3
"""Validate structured documentation coverage across the Spanda repository."""

from __future__ import annotations

import argparse
import sys

from doc_validation_lib import ROOT, render_coverage_report, scan_repository

REPORT_PATH = ROOT / "docs" / "documentation-coverage.md"


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate Spanda documentation coverage.")
    parser.add_argument(
        "--warn",
        action="store_true",
        help="Emit warnings for public APIs missing structured documentation.",
    )
    parser.add_argument(
        "--report",
        action="store_true",
        help="Write docs/documentation-coverage.md.",
    )
    parser.add_argument(
        "--public-only",
        action="store_true",
        help="Only report public APIs when using --warn.",
    )
    args = parser.parse_args()

    assessments = scan_repository()
    total = len(assessments)
    documented = sum(1 for a in assessments if a.documented)
    pct = (documented / total * 100) if total else 100.0

    print(f"Documentation coverage: {documented}/{total} ({pct:.1f}%)")

    if args.report:
        REPORT_PATH.write_text(render_coverage_report(assessments), encoding="utf-8")
        print(f"Wrote {REPORT_PATH.relative_to(ROOT)}")

    if args.warn:
        warnings = 0
        for a in assessments:
            if args.public_only and not a.match.is_public:
                continue
            if a.documented:
                continue
            rel = a.match.path.relative_to(ROOT)
            miss = ", ".join(a.missing)
            print(f"warning: {rel}:{a.match.line} `{a.match.name}` — {miss}")
            warnings += 1
        print(f"\n{warnings} documentation warning(s)")
        return 0

    return 0


if __name__ == "__main__":
    sys.exit(main())

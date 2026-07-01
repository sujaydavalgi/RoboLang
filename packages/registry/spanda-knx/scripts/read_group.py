#!/usr/bin/env python3
"""Read one KNX group address via the Spanda Python bridge (xknx when installed)."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def main() -> None:
    if len(sys.argv) != 2:
        print("usage: read_group.py <group_address>", file=sys.stderr)
        sys.exit(2)
    address = sys.argv[1]
    bridge = repo_root() / "scripts" / "spanda_python_bridge.py"
    payload = json.dumps({"fn": "knx_read_group", "args": [address]})
    proc = subprocess.run(
        [sys.executable, str(bridge)],
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        print(proc.stderr or proc.stdout, file=sys.stderr)
        sys.exit(proc.returncode or 1)
    data = json.loads(proc.stdout)
    if not data.get("ok"):
        print(data.get("error", "bridge error"), file=sys.stderr)
        sys.exit(1)
    print(data["result"])


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Read one Home Assistant entity state via the Spanda Python bridge."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def main() -> None:
    if len(sys.argv) != 2:
        print("usage: get_state.py <entity_id>", file=sys.stderr)
        sys.exit(2)
    entity_id = sys.argv[1]
    bridge = repo_root() / "scripts" / "spanda_python_bridge.py"
    payload = json.dumps({"fn": "home_assistant_get_state", "args": [entity_id]})
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

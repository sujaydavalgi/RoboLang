# Spanda Python SDK

Official Python client for Spanda Control Center API v1.

## Install

From PyPI:

```bash
pip install spanda-sdk
pip install "spanda-sdk[stream]"   # WebSocket telemetry
```

From this monorepo:

```bash
pip install -e sdk/python
# WebSocket telemetry
pip install -e "sdk/python[stream]"
```

## Quick start

```python
from spanda import SpandaClient

client = SpandaClient.local()
report = client.readiness("rover.sd")
print(report["report"])
```

Requires Control Center running:

```bash
spanda control-center serve --program examples/robotics/rover.sd
```

## Environment

| Variable | Purpose |
|----------|---------|
| `SPANDA_CONTROL_CENTER_URL` | API base URL (default `http://127.0.0.1:8080`) |
| `SPANDA_API_KEY` | Bearer token for authenticated endpoints |

## Documentation

- [docs/sdk-python.md](../../docs/sdk-python.md)
- [docs/sdk-publishing.md](../../docs/sdk-publishing.md)
- [docs/control-center-api.md](../../docs/control-center-api.md)

## Legacy client

Enterprise ops helpers (`ControlCenterClient` for drift, OTA, SRE) remain in `packages/sdk-python`.

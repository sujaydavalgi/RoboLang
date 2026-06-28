# Python SDK (`spanda-sdk`)

Official Python client for robotics scripts, notebooks, CI/CD, and ROS2 integrations.

## Install

```bash
pip install -e sdk/python
# stream extras for WebSocket telemetry
pip install -e "sdk/python[stream]"
```

## Usage

```python
from spanda import SpandaClient

client = SpandaClient.local()
report = client.readiness("rover.sd")
score = report.get("report", {}).get("score", {})
print(score.get("total") if isinstance(score, dict) else score)
```

Alternative import:

```python
from spanda_sdk import SpandaClient
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `SPANDA_CONTROL_CENTER_URL` | Base URL (default `http://127.0.0.1:8080`) |
| `SPANDA_API_KEY` | Bearer token for authenticated endpoints |

## Event stream

```python
from spanda_sdk import TelemetryStream

def on_event(event):
    print(event.get("type"), event)

TelemetryStream().listen(on_event)  # requires [stream] extra
```

## Error handling

```python
from spanda_sdk.errors import ConnectionError, PermissionError

try:
    client.list_devices()
except PermissionError as exc:
    print("Set SPANDA_API_KEY", exc.status)
except ConnectionError:
    print("Start Control Center first")
```

## Examples

```bash
python examples/sdk/python/readiness.py
python examples/sdk/python/robot_health.py
```

## Tests

```bash
python -m pytest sdk/python
```

## Legacy client

`packages/sdk-python` provides `ControlCenterClient` with enterprise ops helpers (drift, OTA, SRE). New integrations should use `SpandaClient` from `sdk/python`.

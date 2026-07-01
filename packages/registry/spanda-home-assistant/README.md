# spanda-home-assistant

Interop bridge to Home Assistant — Spanda orchestrates verified missions; HA remains device authority.

**Import path:** `bridge.home_assistant` · **Status:** Experimental (REST + mock)

Does **not** replace Home Assistant. See [docs/smart-space-packages.md](../../docs/smart-space-packages.md) and [docs/smart-space-bms-bridge.md](../../docs/smart-space-bms-bridge.md).

## Runtime env

| Variable | Purpose |
|----------|---------|
| `SPANDA_LIVE_HOME_ASSISTANT=1` | Enable live HA reads in provider dispatch |
| `SPANDA_HOME_ASSISTANT_URL` | HA base URL, e.g. `http://127.0.0.1:8123` |
| `SPANDA_HOME_ASSISTANT_TOKEN` | Long-lived access token for REST `/api/states/{entity}` |
| `SPANDA_HOME_ASSISTANT_CMD` | Shell template (`{entity}`) — overrides registry script |
| `SPANDA_HOME_ASSISTANT_FORCE_MOCK=1` | Force mock reads (CI) |

## Package scripts

```bash
python3 packages/registry/spanda-home-assistant/scripts/get_state.py binary_sensor.leak_basement
# or
packages/registry/spanda-home-assistant/scripts/get_state.sh binary_sensor.leak_basement
```

Read order in Rust: `SPANDA_HOME_ASSISTANT_CMD` → `get_state.py` → `spanda_python_bridge.py` → mock.

# spanda-knx

KNX building automation bridge for Smart Spaces blueprints.

## Runtime

Provider dispatch (`iot.knx.read_group_address`) routes through `spanda-providers` → `iot_hub` → `iot_live` when:

- `SPANDA_LIVE_KNX=1`
- `SPANDA_KNX_CMD` shell template (`{address}`), or
- Python bridge handler `knx_read_group` (xknx when installed, else mock)

### Native reads (xknx)

```bash
pip install -r packages/registry/spanda-knx/requirements-knx.txt
export SPANDA_LIVE_KNX=1
export SPANDA_KNX_GATEWAY=192.168.1.40          # KNX/IP gateway
# optional: SPANDA_KNX_LOCAL_IP=192.168.1.50
# optional: SPANDA_KNX_VALUE_TYPE=temperature     # decode hint for read_group_value
```

### Package CLI (for `SPANDA_KNX_CMD`)

```bash
export SPANDA_LIVE_KNX=1
export SPANDA_KNX_CMD='packages/registry/spanda-knx/scripts/read_group.sh {address}'
```

Or call directly:

```bash
python3 packages/registry/spanda-knx/scripts/read_group.py 1/2/3
```

Force mock (CI / offline):

```bash
export SPANDA_KNX_FORCE_MOCK=1
```

## Smoke

```bash
spanda check packages/registry/spanda-knx/tests/smoke.sd
./scripts/smart_spaces_live_iot_smoke.sh
```

## Example

```bash
export SPANDA_LIVE_KNX=1
export SPANDA_KNX_CMD='echo live-knx:{address}'
spanda control-center smart-spaces environment --zone-id room-lobby
```

## Native integration

For production KNX/IP, point `SPANDA_KNX_CMD` at the package script after `pip install -r requirements-knx.txt`, or register a custom Python `knx_read_group` handler in your provider bootstrap.

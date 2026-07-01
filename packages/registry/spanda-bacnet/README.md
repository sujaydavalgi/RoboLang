# spanda-bacnet

BACnet building automation bridge for Smart Spaces blueprints.

## Runtime

Provider dispatch (`iot.bacnet.read_point`) routes through `spanda-providers` → `iot_hub` → `iot_live` when:

- `SPANDA_LIVE_BACNET=1`
- `SPANDA_BACNET_CMD` shell template (`{device}`, `{object_id}`), or
- Python bridge handler `bacnet_read_point` (bacpypes3 when installed, else mock)

### Native reads (bacpypes3)

```bash
pip install -r packages/registry/spanda-bacnet/requirements-bacnet.txt
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_NETWORK=192.168.1.50/24   # local BACnet/IP bind
export SPANDA_BACNET_TARGET=192.168.1.100       # remote BACnet device IP
export SPANDA_BACNET_OBJECT=analog-value,1      # default object when object_id is a property name
# optional: SPANDA_BACNET_INSTANCE=599
```

`read_point(device, object_id)` arguments:

| Argument | Example | Meaning |
|----------|---------|---------|
| `device` | `ahu-12` | Logical id (mock) or BACnet/IP host if no `SPANDA_BACNET_TARGET` |
| `object_id` | `analog-value,1` | BACnet object identifier |
| `object_id` | `present-value` | Property on `SPANDA_BACNET_OBJECT` (default `analog-value,1`) |

### Package CLI (for `SPANDA_BACNET_CMD`)

```bash
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_CMD='packages/registry/spanda-bacnet/scripts/read_point.sh {device} {object_id}'
```

Or call directly:

```bash
python3 packages/registry/spanda-bacnet/scripts/read_point.py ahu-12 analog-value,1
```

Force mock (CI / offline):

```bash
export SPANDA_BACNET_FORCE_MOCK=1
```

## Smoke

```bash
spanda check packages/registry/spanda-bacnet/tests/smoke.sd
./scripts/smart_spaces_live_iot_smoke.sh
```

## Example

```bash
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_CMD='echo live:{device}:{object_id}'
spanda control-center smart-spaces readiness --facility-id tower-demo
```

# Fleet succession

Rank fleet members for mission takeover after a courier goes offline.

```bash
spanda succession examples/showcase/fleet_succession/delivery.sd \
  --failed CourierA --scope fleet --trigger fleet_offline --json
```

Expected: ranked list of CourierB, CourierC, CourierD with capability and readiness scores.

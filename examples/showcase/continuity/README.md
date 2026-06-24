# Mission continuity

Warehouse inventory scan with checkpoint resume when a scanner goes offline.

## Commands

```bash
# Full continuity evaluation — resume from 72% checkpoint
spanda continuity examples/showcase/continuity/warehouse.sd \
  --failed ScannerAlpha --progress 72 --trigger robot_failed

# Rank fleet successors
spanda succession examples/showcase/continuity/warehouse.sd \
  --failed ScannerAlpha --scope fleet --json

# Coordinate takeover to a specific successor
spanda takeover examples/showcase/continuity/warehouse.sd \
  --failed ScannerAlpha --successor ScannerBeta --progress 72
```

## Expected outcome

- **Can continue:** yes
- **Decision:** resume from checkpoint at 72%
- **Successor:** highest-ranked healthy fleet member (ScannerBeta or ScannerGamma)

# Swarm takeover

Swarm member lost — coordinator replacement and task redistribution.

```bash
spanda takeover examples/showcase/swarm_takeover/swarm.sd \
  --failed DroneTwo --trigger swarm_lost --scope swarm --progress 30

spanda succession examples/showcase/swarm_takeover/swarm.sd \
  --failed DroneTwo --scope swarm
```

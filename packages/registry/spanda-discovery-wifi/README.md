# spanda-discovery-wifi

Optional **WiFi subnet** discovery transport for the Spanda Device Pool.

## Status

**Experimental** — package contract stub. Core subnet scanning ships in `spanda-config`; this package tags matches when installed.

## API

```bash
curl 'http://127.0.0.1:8080/v1/discovery?transport=wifi'
```

## Configuration

```bash
export SPANDA_DISCOVERY_WIFI_MATCHES="rover-ap@192.168.1.50"
export SPANDA_DISCOVERY_SUBNET=192.168.1.0/24
```

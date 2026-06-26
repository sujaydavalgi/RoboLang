# spanda-discovery-serial

Optional **serial port** discovery transport for embedded devices on USB-UART bridges.

## Status

**Experimental** — package contract stub. Host `ls /dev/tty*` probe with env override fallback.

## API

```bash
curl 'http://127.0.0.1:8080/v1/discovery?transport=serial'
```

## Configuration

```bash
export SPANDA_DISCOVERY_SERIAL_MATCHES="gps@/dev/ttyUSB0"
```

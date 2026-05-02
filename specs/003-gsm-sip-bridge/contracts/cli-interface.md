# CLI Interface Contract: gsm-sip-bridge

## Invocation

```text
gsm-sip-bridge [OPTIONS]
```

## Options

| Flag | Long | Argument | Default | Description |
|------|------|----------|---------|-------------|
| `-c` | `--config` | `PATH` | `config.ini` | Configuration file path |
| `-s` | `--serial` | `PATH` | auto-detect | Override EC20 serial port |
| `-a` | `--audio` | `DEVICE` | auto-detect | Override ALSA audio device |
| `-v` | `--verbose` | -- | off | Enable verbose PJSIP and AT logging |
| `-h` | `--help` | -- | -- | Show help and exit |
| | `--version` | -- | -- | Show version and exit |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Clean shutdown (SIGINT/SIGTERM) |
| 1 | Device discovery failure (no EC20 found) |
| 2 | Serial port open failure |
| 3 | Configuration file error |
| 4 | PJSIP initialization failure |
| 5 | SIP registration failure (timeout after retries) |
| 6 | GSM network registration failure |

## Log Format

```text
YYYY-MM-DDTHH:MM:SS.mmm LEVEL message
```

All log output goes to stdout. Levels: `INFO`, `WARN`, `ERROR`.

## Lifecycle

1. Parse CLI arguments
2. Load `config.ini` (SIP + bridge sections)
3. Discover/open EC20 serial port
4. Initialize ALSA audio parameters (verify device accessible)
5. Initialize PJSIP endpoint and register SIP account
6. Initialize GSM modem (ATE0, AT+CLIP=1, AT+QPCMV=1,2, network check)
7. Enter main event loop (wait for RING URCs)
8. On SIGINT/SIGTERM: hang up active calls, de-register SIP, close serial, exit 0

# CLI Interface Contract: audio-echo

**Branch**: `001-gsm-audio-echo` | **Date**: 2026-05-02

## Binary Name

`audio-echo`

## Usage

```text
audio-echo [OPTIONS]
```

## Options

| Flag | Long Form | Argument | Default | Description |
|------|-----------|----------|---------|-------------|
| `-s` | `--serial` | PATH | Auto-detect | Override serial port path (e.g., `/dev/ttyUSB2`) |
| `-a` | `--audio` | DEVICE | Auto-detect | Override ALSA device name (e.g., `hw:1,0`) |
| `-v` | `--verbose` | (none) | Off | Enable verbose logging (AT command trace) |
| `-h` | `--help` | (none) | N/A | Print usage and exit |
| | `--version` | (none) | N/A | Print version and exit |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Normal shutdown (SIGINT/SIGTERM received) |
| 1 | EC20 module not found (auto-detection failed) |
| 2 | Serial port open/configuration failed |
| 3 | ALSA device open/configuration failed |
| 4 | SIM not registered on network |
| 5 | Unrecoverable runtime error |

## Stdout Log Format

All log lines are written to stdout with the format:

```text
YYYY-MM-DDTHH:MM:SS.sss LEVEL message
```

**Levels**: `INFO`, `WARN`, `ERROR`

### Log Events

| Event | Level | Example Output |
|-------|-------|----------------|
| Startup | INFO | `2026-05-02T12:00:00.000 INFO audio-echo v0.1.0 starting` |
| Device detected | INFO | `2026-05-02T12:00:00.100 INFO detected EC20 at /dev/ttyUSB2, audio hw:1,0` |
| Ready | INFO | `2026-05-02T12:00:01.000 INFO ready, waiting for incoming calls` |
| Incoming call | INFO | `2026-05-02T12:00:10.000 INFO RING from +1234567890` |
| Call answered | INFO | `2026-05-02T12:00:10.500 INFO call answered, echo active` |
| Call ended | INFO | `2026-05-02T12:00:30.000 INFO call ended (remote hangup)` |
| Shutdown | INFO | `2026-05-02T12:01:00.000 INFO shutting down` |
| Device error | ERROR | `2026-05-02T12:00:05.000 ERROR ALSA read error: -32 (Broken pipe)` |
| AT trace | INFO | `2026-05-02T12:00:10.000 INFO [AT] >>> ATA` (verbose only) |

## Signal Handling

| Signal | Behavior |
|--------|----------|
| SIGINT (Ctrl+C) | Graceful shutdown: hang up active call, close devices, exit 0 |
| SIGTERM | Same as SIGINT |

## Examples

```bash
# Auto-detect everything
audio-echo

# Override serial port only
audio-echo --serial /dev/ttyUSB3

# Override both devices with verbose logging
audio-echo -s /dev/ttyUSB2 -a hw:2,0 -v

# Print help
audio-echo --help
```

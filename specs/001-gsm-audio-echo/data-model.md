# Data Model: GSM Audio Echo

**Branch**: `001-gsm-audio-echo` | **Date**: 2026-05-02

## Entities

### DeviceInfo

Represents the auto-detected or manually overridden hardware paths
for the EC20 module.

| Field | Type | Description |
|-------|------|-------------|
| serial_port | string | Path to AT command serial port (e.g., `/dev/ttyUSB2`) |
| alsa_device | string | ALSA hardware device name (e.g., `hw:1,0` or `hw:CARD=Android,DEV=0`) |
| usb_vendor_id | uint16 | USB vendor ID (0x2C7C for Quectel) |
| usb_product_id | uint16 | USB product ID (0x0125 for EC20) |

**Identity**: One DeviceInfo per runtime. Determined at startup and
immutable for the process lifetime.

### AudioConfig

Represents the ALSA PCM parameters for audio capture and playback.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| sample_rate | uint32 | 8000 or 16000 | Samples per second |
| format | enum | S16_LE | Sample format (signed 16-bit little-endian) |
| channels | uint32 | 1 | Mono only |
| period_frames | uint32 | 160 | Frames per period (20 ms at 8 kHz) |
| buffer_frames | uint32 | 640 | Total buffer size (4 periods) |

**Identity**: Derived from ALSA device capabilities at call start.
Immutable for the duration of a call.

### CallSession

Represents the lifecycle of a single voice call.

| Field | Type | Description |
|-------|------|-------------|
| state | CallState enum | Current state in the lifecycle |
| caller_id | string (optional) | Caller number from CLIP URC if available |
| start_time | timestamp | When the call was answered |
| end_time | timestamp (optional) | When the call ended |

**Identity**: At most one active CallSession at any time. Created
on RING, destroyed on hangup/error.

## State Transitions

### CallState

```text
        RING URC
           │
           v
       ┌────────┐
       │RINGING │
       └───┬────┘
           │ ATA sent
           v
       ┌────────┐
       │ANSWERED│
       └───┬────┘
           │ Audio devices opened
           v
       ┌────────┐
       │ECHOING │ ◄── audio loopback active
       └───┬────┘
           │ Remote hangup / error / signal
           v
       ┌────────┐
       │ ENDED  │
       └───┬────┘
           │ Resources released
           v
       ┌────────┐
       │  IDLE  │ ◄── waiting for next RING
       └────────┘
```

**Transitions**:

| From | To | Trigger | Actions |
|------|----|---------|---------|
| IDLE | RINGING | `RING` URC received on serial | Log incoming call |
| RINGING | ANSWERED | `ATA` sent, `OK` received | Log answer |
| ANSWERED | ECHOING | ALSA capture + playback opened | Begin audio loopback |
| ECHOING | ENDED | `NO CARRIER` URC, `AT+CLCC` shows no active call, or SIGINT | Stop audio loopback, close ALSA devices |
| ENDED | IDLE | Resources released | Log call end, reset state |
| RINGING | ENDED | Error sending ATA, timeout | Log error, reset |
| ECHOING | ENDED | ALSA read/write error | Log error, send AT+CHUP, reset |
| Any | ENDED | SIGINT/SIGTERM | Graceful shutdown: AT+CHUP if call active, close devices |

### Application Lifecycle

```text
  startup
     │
     v
  ┌──────────────┐
  │ INITIALIZING │ ── detect device, open serial, verify module
  └──────┬───────┘
         │ success
         v
  ┌──────────────┐
  │   RUNNING    │ ── IDLE state, monitoring for RING
  └──────┬───────┘
         │ SIGINT/SIGTERM
         v
  ┌──────────────┐
  │ SHUTTING_DOWN│ ── hangup if active, close all resources
  └──────┬───────┘
         │
         v
       exit(0)
```

**Error states**: If device detection fails at startup, the
application logs the error and exits with nonzero status (never
enters RUNNING).

## Relationships

```text
DeviceInfo 1──1 Application (one device per process)
AudioConfig 1──1 CallSession (config determined per call)
CallSession 0..1──1 Application (at most one active call)
```

## Validation Rules

- `serial_port` MUST be a readable/writable character device
- `alsa_device` MUST be openable by `snd_pcm_open` in both capture
  and playback modes
- `sample_rate` MUST match what the ALSA device reports as supported
  (no forced resampling)
- Only one CallSession may exist at a time; a second RING while in
  RINGING/ANSWERED/ECHOING triggers rejection (AT+CHUP on the new
  call or ignore)

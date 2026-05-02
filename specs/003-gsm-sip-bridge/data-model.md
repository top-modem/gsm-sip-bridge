# Data Model: GSM to SIP Audio Bridge

## Entities

### BridgeConfig

Bridge-specific configuration read from the `[bridge]` section of `config.ini`.

| Field | Type | Constraints | Default | Source |
|-------|------|-------------|---------|--------|
| `sip_destination` | `string` | non-empty, 1-32 chars, alphanumeric/`*`/`#`/`+` | `"599"` | `[bridge].sip_destination` |
| `sip_dial_timeout_sec` | `uint16_t` | 5-120 | `30` | `[bridge].sip_dial_timeout_sec` |

**Validation rules**:
- If `[bridge]` section is absent, all fields use defaults.
- If `sip_destination` is empty after trimming, use default `"599"`.
- If `sip_dial_timeout_sec` is out of range, reject with error.

**Relationship**: Composed with `SipConfig` (from feature 002). Both are loaded from the same `config.ini` file.

### BridgedCall

Represents an active GSM-to-SIP bridged call session. Only one instance exists at a time.

| Field | Type | Constraints |
|-------|------|-------------|
| `state` | `BridgeState` | enum, see state machine below |
| `gsm_answered_at` | `steady_clock::time_point` | set when GSM call is answered |
| `sip_connected_at` | `steady_clock::time_point` | set when SIP party answers |
| `sip_call_id` | `int` | PJSUA call ID, -1 when no SIP call |

### BridgeState (enum)

```text
IDLE -> GSM_RINGING -> GSM_ANSWERED -> SIP_DIALING -> SIP_RINGING -> BRIDGED -> ENDING -> IDLE
                                         |               |
                                         +-> SIP_FAILED --+
```

| State | Description |
|-------|-------------|
| `IDLE` | No active call. Waiting for GSM RING. |
| `GSM_RINGING` | RING URC received. About to answer. |
| `GSM_ANSWERED` | GSM call answered (ATA sent). Opening ALSA. |
| `SIP_DIALING` | Outbound SIP INVITE sent. Beep playing to GSM. |
| `SIP_RINGING` | SIP 180 Ringing received. Beep continues. |
| `SIP_FAILED` | SIP call failed (busy/timeout/error). Error tone playing. |
| `BRIDGED` | Both legs connected. Bidirectional audio active. |
| `ENDING` | One leg hung up. Tearing down the other. |

### Transitions

| From | Event | To | Action |
|------|-------|-----|--------|
| IDLE | RING URC | GSM_RINGING | Log ring event |
| GSM_RINGING | ATA success | GSM_ANSWERED | Open ALSA, start beep, init SIP |
| GSM_ANSWERED | makeCall() sent | SIP_DIALING | Start beep on ALSA playback |
| SIP_DIALING | 180 Ringing | SIP_RINGING | Continue beep |
| SIP_DIALING | 200 OK (CONFIRMED) | BRIDGED | Stop beep, connect audio bridge |
| SIP_RINGING | 200 OK (CONFIRMED) | BRIDGED | Stop beep, connect audio bridge |
| SIP_DIALING | Failure/Timeout | SIP_FAILED | Play error tone, schedule hangup |
| SIP_RINGING | Failure/Timeout | SIP_FAILED | Play error tone, schedule hangup |
| SIP_FAILED | Error tone done | ENDING | Hang up GSM |
| BRIDGED | GSM hangup (NO CARRIER) | ENDING | Hang up SIP call |
| BRIDGED | SIP hangup (DISCONNECTED) | ENDING | Hang up GSM call |
| ENDING | Both legs down | IDLE | Release all resources |

## Audio Data Flow

```text
GSM Caller <-> EC20 USB Audio (ALSA) <-> Ring Buffer <-> AudioMediaPort <-> PJSIP Conf Bridge <-> SIP Call <-> SIP Party
```

- **GSM -> SIP**: ALSA capture thread reads PCM frames -> writes to `capture_ring_buffer` -> `onFrameRequested()` reads from buffer -> feeds into SIP call's `AudioMedia`.
- **SIP -> GSM**: SIP call's `AudioMedia` -> `onFrameReceived()` writes to `playback_ring_buffer` -> ALSA playback thread reads from buffer -> writes to ALSA playback device.
- **Beep phase**: ALSA playback thread reads from pre-computed beep buffer instead of `playback_ring_buffer`.

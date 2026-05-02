# Research: GSM to SIP Audio Bridge

## R-001: Bridging ALSA audio into PJSIP's conference bridge

**Decision**: Use `pj::AudioMediaPort` subclass with `onFrameRequested()` / `onFrameReceived()` callbacks.

**Rationale**: PJSIP 2.14+ exposes `AudioMediaPort` as the recommended C++ mechanism for injecting/extracting raw PCM audio into the conference bridge. The port participates in the bridge like any other media object -- `startTransmit()` / `stopTransmit()` wire it to a call's `AudioMedia`. The bridge handles clock-rate and frame-size conversion between connected ports automatically.

- `onFrameRequested()` is called when the bridge needs a frame from us (GSM capture -> SIP call).
- `onFrameReceived()` is called when the bridge delivers a frame to us (SIP call -> GSM playback).

Both run on the conference bridge clock thread and must be non-blocking.

**Alternatives considered**:
- Custom `pjmedia_port` with raw C function pointers: More control but requires manual pool management. Unnecessary since PJSUA2 wraps this cleanly.
- PJSUA sound-device hooks: Read-only, fires for all calls, cannot inject audio. Unsuitable.
- Direct PJMEDIA audio device: Bypasses conference bridge entirely. Overkill and loses routing.

## R-002: Audio format compatibility between EC20 ALSA and PJSIP

**Decision**: Use 8000 Hz, S16_LE, mono, 20ms frames (160 samples) on both sides.

**Rationale**: The EC20's USB Audio Class interface runs at 8000 Hz, S16_LE, mono. PJSIP's conference bridge accepts any clock rate and converts internally. By creating the `AudioMediaPort` at 8000 Hz / 16-bit / mono / 20ms, the PCM frames from ALSA can be passed directly into `onFrameRequested()` without format conversion. PJSIP handles codec negotiation (G.711) with the remote SIP endpoint independently.

**Alternatives considered**:
- Resampling to 16 kHz for PJSIP: Adds latency and complexity with no quality benefit since the source is 8 kHz telephony audio.

## R-003: Thread-safe audio buffer between ALSA and PJSIP callbacks

**Decision**: Use a lock-free ring buffer (SPSC - single producer, single consumer) to decouple ALSA I/O from PJSIP bridge callbacks.

**Rationale**: ALSA capture/playback runs in a dedicated thread (blocking `snd_pcm_readi` / `snd_pcm_writei`). PJSIP's `onFrameRequested` / `onFrameReceived` run on the conference bridge clock thread. A lock-free ring buffer allows the ALSA thread to write captured frames and read playback frames without blocking the PJSIP callbacks, and vice versa. The ring buffer is sized to hold ~100ms of audio (5 frames at 20ms each) to absorb timing jitter.

**Alternatives considered**:
- Mutex-protected queue: Risk of priority inversion on the real-time bridge thread. Lock contention could cause audible glitches.
- Direct ALSA from callbacks: `snd_pcm_readi`/`writei` can block. This would stall the entire PJSIP conference bridge tick.

## R-004: Beep pattern generation

**Decision**: Generate a 400 Hz sine wave programmatically, 200ms on / 200ms off pattern, at 8000 Hz S16_LE.

**Rationale**: The beep provides audible feedback to the GSM caller while the SIP destination is ringing. 400 Hz is the standard European dial tone frequency and is clearly audible on telephony-grade audio. The pattern (200ms on / 200ms off) is distinct from continuous tones and signals "connecting." The tone is pre-computed at startup into a buffer and written to the ALSA playback device (or provided via `onFrameRequested`) during the ringing phase.

**Alternatives considered**:
- Pre-recorded WAV file: Adds a file dependency and build complexity for no benefit.
- Using PJSIP's tone generator: The beep goes to the GSM side (ALSA), not the SIP side. PJSIP's tone generator feeds the conference bridge which goes to SIP, not ALSA.

## R-005: Outbound SIP call mechanism

**Decision**: Use PJSUA2's `pj::Call::makeCall()` from a `pj::Account` to place the outbound SIP call.

**Rationale**: The existing `EchoAccount` manages SIP registration. For the bridge, a new `BridgeAccount` (or reuse with different callbacks) makes an outbound call using `Call::makeCall(dest_uri, CallOpParam)`. Call state changes (`onCallState`) detect when the remote answers (CONFIRMED), fails (DISCONNECTED with non-200 reason), or rings (EARLY). Media state changes (`onCallMediaState`) trigger the switch from beep to bidirectional audio.

**Alternatives considered**:
- Raw PJSIP C API (`pjsua_call_make_call`): Unnecessary complexity when using PJSUA2.

## R-006: Licensing compliance

**Decision**: PJSIP GPL v2+ use is justified (carried forward from feature 002).

**Rationale**: Per the constitution's license gate justification from plan 002: (a) user explicitly specified PJSIP as a binding requirement, (b) this is an internal diagnostic tool not distributed commercially. mINI remains MIT licensed (compliant). ALSA (LGPL 2.1) is dynamically linked (compliant).

**Alternatives considered**: None viable -- PJSIP is the only mature C++ SIP library suitable for this use case.

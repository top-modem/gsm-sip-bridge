# Implementation Plan: GSM to SIP Audio Bridge

**Branch**: `003-gsm-sip-bridge` | **Date**: 2026-05-02 | **Spec**: `specs/003-gsm-sip-bridge/spec.md`
**Input**: Feature specification from `/specs/003-gsm-sip-bridge/spec.md`

## Summary

Build a single binary (`gsm-sip-bridge`) that listens for incoming GSM calls on the EC20 module, auto-answers them, places an outbound SIP call to a configurable extension (default 599), plays a beep pattern to the GSM caller while the SIP side rings, and bridges audio bidirectionally once the SIP party answers. Uses PJSIP's `AudioMediaPort` to bridge between ALSA (EC20 USB audio) and the PJSIP conference bridge, with a lock-free ring buffer decoupling the two I/O threads.

## Technical Context

**Language/Version**: C++17
**Primary Dependencies**: ALSA (`libasound2`), PJSIP (`libpjproject`), mINI (MIT, header-only INI parser), Google Test
**Storage**: N/A (stateless runtime)
**Testing**: Google Test (`gtest`), integration-first per constitution
**Target Platform**: Linux (ARM/x86, Raspberry Pi / similar SBC)
**Project Type**: CLI daemon
**Performance Goals**: <300ms end-to-end voice latency, <2s call answer time, <500ms beep-to-audio switch
**Constraints**: Single active bridge at a time, 8 kHz telephony audio, real-time audio processing
**Scale/Scope**: Sequential calls, >50 calls without restart or resource leak

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Integration-First Testing | PASS | Tests exercise real PJSIP endpoint, real config parser, real PTY serial. Hardware (ALSA/EC20) mocked with justification. |
| II. Green-on-Commit | PASS | All tests must pass before each commit. CI-compatible test targets. |
| III. Frequent Atomic Commits | PASS | Implementation broken into 6 phases with commit points after each task. |
| IV. Makefile-Driven Build | PASS | `run-bridge` target added alongside existing `build`, `test`, `run`, `run-sip`. |
| V. Simplicity & Refactorability | PASS | Reuses existing GSM and SIP modules directly. New code limited to bridge orchestration, ring buffer, and `AudioMediaPort` adapter. No unnecessary abstractions. |

**License gate**: PJSIP is GPL v2+. Justified per plan 002: (a) user explicitly specified PJSIP, (b) internal diagnostic tool not distributed commercially. mINI is MIT. ALSA is LGPL 2.1 (dynamically linked).

## Project Structure

### Documentation (this feature)

```text
specs/003-gsm-sip-bridge/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-interface.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── logger.h                    # Shared (existing)
├── device_discovery.h/cpp      # Existing (GSM)
├── serial_port.h/cpp           # Existing (GSM)
├── at_commander.h/cpp          # Existing (GSM)
├── audio_loop.h/cpp            # Existing (GSM echo, not used directly by bridge)
├── ring_buffer.h               # NEW: Lock-free SPSC ring buffer (header-only)
├── sip/
│   ├── sip_config.h/cpp        # Existing (extended: load_bridge_config)
│   ├── echo_account.h/cpp      # Existing (SIP echo, not used by bridge)
│   └── echo_call.h/cpp         # Existing (SIP echo, not used by bridge)
├── bridge/
│   ├── main.cpp                # NEW: CLI, signal handling, orchestration
│   ├── bridge_config.h         # NEW: BridgeConfig struct + loader
│   ├── bridge_config.cpp       # NEW: INI parser for [bridge] section
│   ├── alsa_media_port.h       # NEW: AudioMediaPort adapter (ALSA <-> PJSIP)
│   ├── alsa_media_port.cpp
│   ├── beep_generator.h        # NEW: 400Hz tone buffer generator
│   ├── beep_generator.cpp
│   ├── bridge_call.h           # NEW: pj::Call subclass for outbound SIP leg
│   ├── bridge_call.cpp
│   ├── bridge_account.h        # NEW: pj::Account subclass for bridge SIP
│   └── bridge_account.cpp

tests/integration/
├── test_bridge_config.cpp      # NEW
├── test_ring_buffer.cpp        # NEW
├── test_beep_generator.cpp     # NEW
├── test_bridge_call.cpp        # NEW
├── ... (existing tests)
```

**Structure Decision**: New `src/bridge/` directory follows the existing `src/sip/` pattern. Shared infrastructure (`logger.h`, `device_discovery`, `serial_port`, `at_commander`, `ring_buffer.h`) lives in `src/`. The bridge reuses GSM modules by linking the same source files and reuses SIP config parsing.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Ring buffer (new data structure) | ALSA I/O and PJSIP callbacks run on different threads with real-time constraints | Mutex would risk priority inversion and audible glitches |
| AudioMediaPort adapter (new abstraction) | PJSIP requires a port subclass to inject/extract audio from the conference bridge | No simpler mechanism exists in PJSIP's API |

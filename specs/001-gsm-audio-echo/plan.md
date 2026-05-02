# Implementation Plan: GSM Audio Echo

**Branch**: `001-gsm-audio-echo` | **Date**: 2026-05-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-gsm-audio-echo/spec.md`

## Summary

Build a C++ CLI program that auto-detects a Quectel EC20 GSM module
over USB, monitors for incoming voice calls via AT commands over
serial, auto-answers calls, and echoes the caller's audio back in
real time using ALSA capture/playback on the module's USB Audio Class
(UAC) device. Audio is looped back at the native 8000 Hz / S16_LE /
mono format with ~40-60 ms latency. The program runs continuously,
handling sequential calls and recovering from module disconnections.

## Technical Context

**Language/Version**: C++17 (GCC 9+)
**Primary Dependencies**: libasound2 (ALSA), libudev (USB discovery)
**Storage**: N/A (no persistent storage)
**Testing**: Google Test (GTest) + CTest, socat (virtual serial),
snd-pcmtest (virtual ALSA)
**Target Platform**: Linux (kernel 3.11+, ALSA support, Debian-based
recommended)
**Project Type**: CLI application (single binary)
**Performance Goals**: <500 ms round-trip audio echo latency; <60 ms
system-introduced latency; answer within 3 seconds of RING
**Constraints**: Single call at a time; 8000 Hz S16_LE mono audio;
EC20 firmware EC20CEFAGR06A15M4G+ required for UAC
**Scale/Scope**: Single device, single concurrent call, long-running
process

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1
design.*

| # | Principle | Status | Evidence |
|---|-----------|--------|----------|
| I | Integration-First Testing | PASS | Tests use real ALSA devices (snd-pcmtest virtual driver) and real serial ports (socat pty pairs). Mocks only for hardware not available in CI (justified). |
| II | Green-on-Commit | PASS | `make test` runs full GTest/CTest suite. All tests must pass before commit. CI enforces this gate. |
| III | Frequent Atomic Commits | PASS | Task breakdown supports per-task commits. Each task produces a compilable, testable increment. |
| IV | Makefile-Driven Build | PASS | Root Makefile wraps CMake with targets: build, test, run, clean, lint, help. |
| V | Simplicity & Refactorability | PASS | No frameworks, no ORM, no abstraction layers. Direct ALSA API + POSIX termios + libudev. Flat source structure. Single binary output. |

No violations. Complexity Tracking section not needed.

## Project Structure

### Documentation (this feature)

```text
specs/001-gsm-audio-echo/
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
├── main.cpp
├── device_discovery.h
├── device_discovery.cpp
├── serial_port.h
├── serial_port.cpp
├── at_commander.h
├── at_commander.cpp
├── audio_loop.h
├── audio_loop.cpp
└── logger.h

tests/
├── integration/
│   ├── test_device_discovery.cpp
│   ├── test_at_commander.cpp
│   ├── test_audio_loop.cpp
│   └── test_end_to_end.cpp
└── CMakeLists.txt

CMakeLists.txt
Makefile
README.md
```

**Structure Decision**: Single project layout. All source in `src/`,
all tests in `tests/integration/`. No `tests/unit/` directory because
the constitution mandates integration-first testing -- isolated unit
tests are not the primary test type. Each source module maps to one
integration test file. Header-only `logger.h` avoids a separate
logging dependency.

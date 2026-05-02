# Tasks: GSM Audio Echo

**Input**: Design documents from `/specs/001-gsm-audio-echo/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Integration tests are included per constitution Principle I (Integration-First Testing). TDD cycle: write failing test -> implement -> green.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Project initialization, build system, and shared infrastructure

- [ ] T001 Create directory structure: src/, tests/integration/ per plan.md
- [ ] T002 Create CMakeLists.txt with C++17, find_package for ALSA, udev, GTest
- [ ] T003 Create Makefile with targets: build, test, run, clean, lint, help
- [ ] T004 [P] Create src/logger.h with timestamped log macros (INFO, WARN, ERROR)

**Checkpoint**: `make build` compiles an empty main.cpp. `make test` runs zero tests and succeeds.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core modules that ALL user stories depend on

- [ ] T005 Create src/device_discovery.h and src/device_discovery.cpp: libudev-based auto-detection of EC20 by USB VID:PID 2c7c:0125, resolving serial port and ALSA card
- [ ] T006 Create tests/integration/test_device_discovery.cpp: test auto-detection with real udev enumeration (verify sysfs parsing returns valid paths or empty when no EC20 present)
- [ ] T007 Create src/serial_port.h and src/serial_port.cpp: POSIX termios wrapper for open/close/read_line/write with RAII, 115200 8N1
- [ ] T008 Create tests/integration/test_serial_port.cpp: test serial read/write using socat pty pair (real file descriptors, no mocks)
- [ ] T009 Create src/at_commander.h and src/at_commander.cpp: send AT commands, parse responses and URCs (RING, NO CARRIER, +CLCC), with line-buffered reading from SerialPort
- [ ] T010 Create tests/integration/test_at_commander.cpp: test AT command/response cycle using socat pty pair simulating EC20 responses

**Checkpoint**: All foundational modules compile and pass tests. `make test` green.

---

## Phase 3: User Story 1 - Auto-Answer and Echo Audio (Priority: P1)

**Goal**: Incoming call is auto-answered and caller hears their own audio echoed back

**Independent Test**: Call the SIM number, verify auto-answer, hear echo, hang up, system returns to idle

### Integration Tests for User Story 1

- [ ] T011 [US1] Create tests/integration/test_audio_loop.cpp: test ALSA capture->playback loopback using snd-pcmtest virtual device or default ALSA device
- [ ] T012 [US1] Create tests/integration/test_end_to_end.cpp: test full call lifecycle (RING->ATA->echo->NO CARRIER) using socat pty for serial and real ALSA device

### Implementation for User Story 1

- [ ] T013 [US1] Create src/audio_loop.h and src/audio_loop.cpp: ALSA PCM open (capture+playback), configure 8000Hz/S16_LE/mono, period=160/buffer=640, read-write loop
- [ ] T014 [US1] Create src/main.cpp: CLI argument parsing (--serial, --audio, --verbose, --help, --version), signal handling (SIGINT/SIGTERM), device discovery, main event loop (idle->ring->answer->echo->hangup->idle)
- [ ] T015 [US1] Wire call state machine in main.cpp: IDLE->RINGING (on RING URC)->ANSWERED (on ATA OK)->ECHOING (audio_loop start)->ENDED (on NO CARRIER)->IDLE
- [ ] T016 [US1] Add call rejection in at_commander: when call active and new RING arrives, send AT+CHUP for new call (FR-007)

**Checkpoint**: `make build` produces audio-echo binary. Full call lifecycle works: ring->answer->echo->hangup->idle. All tests green.

---

## Phase 4: User Story 2 - Optimal Audio Quality (Priority: P2)

**Goal**: Audio uses native device sample rate without resampling; no system-introduced artifacts

**Independent Test**: Call and speak varied tones, verify clear echo without clicks/pops/distortion

### Implementation for User Story 2

- [ ] T017 [US2] Add native sample rate probing in audio_loop.cpp: query ALSA device for supported rates, select highest (prefer 16000 if available, fall back to 8000), log chosen rate
- [ ] T018 [US2] Add buffer underrun/overrun recovery in audio_loop.cpp: detect EPIPE from snd_pcm_readi/snd_pcm_writei, call snd_pcm_prepare to recover, log the event
- [ ] T019 [US2] Update tests/integration/test_audio_loop.cpp: add test for rate probing and xrun recovery

**Checkpoint**: Audio loop auto-detects native rate. Underrun/overrun events are recovered without crash. Tests green.

---

## Phase 5: User Story 3 - Continuous Operation and Error Recovery (Priority: P3)

**Goal**: System runs indefinitely, handles sequential calls, recovers from USB disconnection

**Independent Test**: 10 sequential calls without leaks; USB disconnect/reconnect recovery; 1-hour runtime stability

### Implementation for User Story 3

- [ ] T020 [US3] Add serial port error detection in main event loop: detect read errors / EOF on serial, transition to recovery state
- [ ] T021 [US3] Add device reconnection loop in main.cpp: when serial error detected during IDLE, periodically re-run device discovery (every 5s) until module found again, log attempts
- [ ] T022 [US3] Add RAII cleanup for all resources: ensure ALSA handles, serial fd, and udev context are released on every call end and on shutdown (verify no fd/memory leaks)
- [ ] T023 [US3] Update tests/integration/test_end_to_end.cpp: add test for multiple sequential calls through socat pty (verify no resource leaks via /proc/self/fd count)

**Checkpoint**: System handles 10+ sequential calls. Serial error triggers reconnection. No resource leaks. Tests green.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, code quality, final validation

- [ ] T024 [P] Create README.md with project description, prerequisites, quickstart, Makefile targets, troubleshooting
- [ ] T025 [P] Run make lint and fix any warnings/errors across all source files
- [ ] T026 Validate quickstart.md instructions match actual build/run workflow

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational completion
- **US2 (Phase 4)**: Depends on US1 (extends audio_loop)
- **US3 (Phase 5)**: Depends on US1 (extends main event loop)
- **Polish (Phase 6)**: Depends on all user stories complete

### Within Each Phase

- Tests MUST be written and FAIL before implementation (TDD)
- Headers before implementations
- Core modules before integration points
- Commit after each task (constitution Principle III)

### Parallel Opportunities

- T004 (logger.h) parallel with T002/T003
- T005+T006, T007+T008, T009+T010 are sequential pairs but pairs can overlap
- T011+T012 (tests) parallel, then T013-T016 (implementation)
- T024+T025 parallel in Polish phase

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test with real EC20 hardware
5. Working echo demo ready

### Incremental Delivery

1. Setup + Foundational -> Build system works
2. User Story 1 -> Call echo works (MVP)
3. User Story 2 -> Audio quality optimized
4. User Story 3 -> Reliable long-running operation
5. Polish -> Production-ready documentation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story is independently completable and testable
- Commit after each task with all tests green (constitution Principle II + III)
- Total tasks: 26

# Tasks: GSM to SIP Audio Bridge

**Feature**: 003-gsm-sip-bridge | **Generated**: 2026-05-02

## Phase 1: Setup (Build Infrastructure)

### T-001: Add `gsm-sip-bridge` binary to CMakeLists.txt
- Add `add_executable(gsm-sip-bridge ...)` linking both ALSA and PJSIP
- Include source files from `src/bridge/`, shared GSM sources, and `src/sip/sip_config.cpp`
- Add `bridge-tests` test executable
- **Verify**: `cmake -B build` succeeds with empty stub files
- **Status**: PENDING

### T-002: Add `run-bridge` Makefile target
- Add `BRIDGE_BINARY := $(BUILD_DIR)/gsm-sip-bridge`
- Add `run-bridge: build` target
- Update `.PHONY` line
- **Verify**: `make help` shows `run-bridge`
- **Status**: PENDING

### T-003: Update `config.ini.example` with `[bridge]` section
- Add `[bridge]` section with `sip_destination` and `sip_dial_timeout_sec`
- **Verify**: File is valid INI
- **Status**: PENDING

## Phase 2: Foundational Components

### T-004: Implement `BridgeConfig` (bridge_config.h/cpp)
- Struct with `sip_destination` (string, default "599") and `sip_dial_timeout_sec` (uint16_t, default 30)
- `static LoadResult load(const std::string& path, BridgeConfig& out)` using mINI
- All defaults applied when `[bridge]` section is absent
- **Verify**: Unit test `test_bridge_config.cpp` passes
- **Status**: PENDING

### T-005: Write `test_bridge_config.cpp`
- Test: valid config with all fields
- Test: missing `[bridge]` section uses defaults
- Test: empty `sip_destination` uses default
- Test: `sip_dial_timeout_sec` out of range (too low, too high) returns error
- Test: valid `sip_dial_timeout_sec` boundary values (5, 120)
- **Verify**: All tests pass
- **Status**: PENDING

### T-006: Implement `ring_buffer.h` (lock-free SPSC)
- Template class `RingBuffer<T>` with fixed capacity
- `bool try_write(const T* data, size_t count)` - returns false if full
- `size_t read(T* data, size_t max_count)` - returns frames read
- `size_t available_read() const`
- `size_t available_write() const`
- Uses `std::atomic<size_t>` for head/tail with acquire/release ordering
- **Verify**: `test_ring_buffer.cpp` passes
- **Status**: PENDING

### T-007: Write `test_ring_buffer.cpp`
- Test: write then read returns same data
- Test: write to full buffer returns false
- Test: read from empty buffer returns 0
- Test: wrap-around works correctly
- Test: concurrent producer/consumer (threaded test)
- **Verify**: All tests pass including threaded
- **Status**: PENDING

### T-008: Implement `beep_generator.h/cpp`
- `BeepGenerator` class with configurable frequency (400 Hz), on/off duration (200ms/200ms), sample rate (8000 Hz)
- Pre-computes sine wave buffer at construction
- `void fill_frame(int16_t* buf, size_t frame_count)` - fills with beep pattern, advancing internal position
- `void reset()` - restart pattern from beginning
- **Verify**: `test_beep_generator.cpp` passes
- **Status**: PENDING

### T-009: Write `test_beep_generator.cpp`
- Test: generated samples are within S16_LE range
- Test: tone period has correct number of samples for on/off durations
- Test: reset restarts pattern
- Test: fill_frame produces non-zero samples during tone-on phase
- Test: fill_frame produces zero samples during tone-off phase
- **Verify**: All tests pass
- **Status**: PENDING

## Phase 3: US1 - GSM-to-SIP Bridge Core

### T-010: Implement `AlsaMediaPort` (alsa_media_port.h/cpp)
- Subclass `pj::AudioMediaPort`
- Constructor takes references to capture and playback `RingBuffer<int16_t>`
- `onFrameRequested()`: read from capture ring buffer, provide to PJSIP
- `onFrameReceived()`: write PJSIP audio to playback ring buffer
- `createPort()` with 8000 Hz, mono, 20ms, 16-bit
- **Verify**: Compiles and links. Functional test deferred to integration.
- **Status**: PENDING

### T-011: Implement `BridgeAccount` (bridge_account.h/cpp)
- Subclass `pj::Account`
- `onRegState()`: log registration state changes
- `onIncomingCall()`: reject with 486 Busy (bridge does not accept inbound SIP)
- Method `make_outbound_call(dest_uri, BridgeCall*)` to place call
- **Verify**: Compiles. Integration test `test_bridge_call.cpp`.
- **Status**: PENDING

### T-012: Implement `BridgeCall` (bridge_call.h/cpp)
- Subclass `pj::Call`
- `onCallState()`: track EARLY (ringing), CONFIRMED (answered), DISCONNECTED
- `onCallMediaState()`: when CONFIRMED, connect `AlsaMediaPort` to call's `AudioMedia` bidirectionally
- Callback mechanism to notify bridge main loop of state changes (atomic flag + state enum)
- **Verify**: `test_bridge_call.cpp` passes
- **Status**: PENDING

### T-013: Write `test_bridge_call.cpp`
- Test: PJSIP endpoint initializes with null audio device
- Test: BridgeAccount creates and registers successfully
- Test: BridgeAccount rejects incoming calls with 486
- **Verify**: All tests pass
- **Status**: PENDING

### T-014: Implement ALSA I/O thread for bridge
- In `bridge/main.cpp`: thread function that opens ALSA capture+playback
- Capture loop: `snd_pcm_readi()` -> write to capture ring buffer
- Playback loop: during beep phase, read from `BeepGenerator`; during bridge phase, read from playback ring buffer -> `snd_pcm_writei()`
- Underrun/overrun recovery matching existing `audio_loop.cpp` patterns
- **Verify**: Functional test with hardware
- **Status**: PENDING

### T-015: Implement bridge main loop orchestration
- GSM event loop: poll AT URCs for RING, NO CARRIER, BUSY
- State machine: IDLE -> GSM_ANSWERED -> SIP_DIALING -> BRIDGED -> ENDING -> IDLE
- On RING: answer GSM, open ALSA, start beep thread, make SIP call
- On SIP CONFIRMED: switch from beep to bridge audio
- On either hangup: tear down both legs, return to IDLE
- Signal handling (SIGINT/SIGTERM): clean shutdown
- **Verify**: Full end-to-end test with hardware
- **Status**: PENDING

## Phase 4: US2 - Bridge Configuration

### T-016: Integrate `BridgeConfig` into bridge main
- Load `BridgeConfig` alongside `SipConfig` from same config.ini
- Use `sip_destination` to construct SIP URI for outbound call
- Use `sip_dial_timeout_sec` for SIP call timeout
- **Verify**: Changing config changes behavior
- **Status**: PENDING

## Phase 5: US3 - Error Recovery and Continuous Operation

### T-017: Handle SIP call failure
- On SIP DISCONNECTED before CONFIRMED: set state to SIP_FAILED
- Generate error tone (lower frequency, continuous) via BeepGenerator alternate pattern
- Play error tone for 2 seconds, then hang up GSM
- **Verify**: SIP busy/unreachable plays error tone to GSM caller
- **Status**: PENDING

### T-018: Handle SIP dial timeout
- Start timer when SIP INVITE is sent
- If `sip_dial_timeout_sec` elapses without CONFIRMED: cancel SIP call
- Transition to SIP_FAILED, play error tone
- **Verify**: No-answer scenario handled correctly
- **Status**: PENDING

### T-019: Handle sequential calls
- After ENDING -> IDLE: release all resources (ALSA, ring buffers, call objects)
- Verify no resource leaks (file descriptors, memory)
- **Verify**: Multiple sequential calls work without restart
- **Status**: PENDING

### T-020: Handle SIP re-registration
- Carry forward NAT handling from sip-echo (contactRewriteUse, fixed local port)
- On registration loss: log warning, PJSIP auto-retries per `retryIntervalSec`
- **Verify**: Bridge continues operating after transient SIP registration loss
- **Status**: PENDING

## Phase 6: Polish

### T-021: Update README.md
- Add GSM-SIP Bridge section (overview, config, usage, troubleshooting)
- Add `run-bridge` to Makefile targets table
- Update architecture diagram
- **Verify**: README covers all three binaries
- **Status**: PENDING

### T-022: Final validation
- `make build` compiles all three binaries
- `make test` passes all test suites
- `make lint` clean
- End-to-end: GSM call -> beep -> SIP answer -> two-way audio -> hangup
- **Verify**: All success criteria (SC-001 through SC-007)
- **Status**: PENDING

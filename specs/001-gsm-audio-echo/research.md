# Research: GSM Audio Echo

**Branch**: `001-gsm-audio-echo` | **Date**: 2026-05-02

## 1. EC20 Voice Call AT Commands

**Decision**: Use standard AT command set for call control: `RING`
(unsolicited) for incoming call detection, `ATA` for answer,
`AT+CHUP` for hangup, `AT+CLCC` for call state queries.

**Rationale**: These are the documented Quectel EC20 AT commands for
voice call management. `AT+CHUP` is preferred over `ATH` for hangup
because it reliably disconnects regardless of `AT+CVHU` configuration.
`AT+CLCC` provides detailed call state (active, held, incoming, etc.)
which is essential for state machine correctness.

**Alternatives considered**:
- `ATH` for hangup: behavior depends on `AT+CVHU` setting, less
  reliable. Rejected.
- QuecPython voiceCall API: requires Python runtime, not applicable
  to a C++ program. Rejected.

**Key AT commands**:
- `RING` (URC): unsolicited result code on incoming call
- `ATA`: answer incoming call
- `AT+CHUP`: hang up all calls (reliable)
- `AT+CLCC`: list current calls with state codes (0=active,
  4=incoming)
- `AT+COPS?`: query network registration status
- `AT+QCFG="USBCFG"`: query/set USB configuration including UAC

## 2. EC20 USB Audio (UAC) Configuration

**Decision**: Use USB Audio Class (UAC) mode for voice audio. Enable
via `AT+QCFG="USBCFG"` with UAC flag set. Audio device appears as
ALSA card named "Android" (e.g., `hw:CARD=Android,DEV=0`).

**Rationale**: UAC provides standard ALSA-compatible PCM audio over
USB, avoiding the need for custom audio codecs or proprietary
interfaces. The asterisk-chan-quectel project validates this approach
works reliably with EC20 modules.

**Alternatives considered**:
- PCM digital interface (non-USB): requires additional hardware
  wiring. Rejected for USB simplicity.
- Pulse Audio: adds unnecessary abstraction layer. ALSA direct access
  gives lower latency. Rejected.

**Prerequisites**:
- EC20 firmware version EC20CEFAGR06A15M4G or later (UAC support)
- Linux kernel 3.11+ (USB audio class compatibility)
- UAC must be enabled once via AT command (persists across reboots):
  `AT+QCFG="USBCFG",0x2C7C,0x0125,1,1,1,1,1,0,1`

## 3. Audio Parameters

**Decision**: Use 8000 Hz sample rate, S16_LE (signed 16-bit
little-endian), mono (1 channel). This is the native format of the
EC20 USB audio device for GSM voice calls.

**Rationale**: The asterisk-chan-quectel driver (the most mature open
source EC20 audio integration) uses exactly these parameters:
`DESIRED_RATE = 8000`, `SND_PCM_FORMAT_S16_LE`. The EC20 UAC device
reports 8000 Hz as its native rate for voice calls. Using the native
rate avoids resampling artifacts and aligns with GSM narrowband
codec output.

**Alternatives considered**:
- 16000 Hz (AMR-WB/HD Voice): the EC20 UAC device may not expose
  16 kHz even when the network supports wideband. The audio device
  itself operates at 8000 Hz. Rejected as primary target; may be
  revisited if device probing shows 16 kHz support.
- 44100/48000 Hz: standard audio rates but irrelevant for telephony
  PCM from the modem. Would require resampling. Rejected.

**Audio buffer parameters** (for low-latency loopback):
- Period size: 160 frames (20 ms at 8000 Hz, one GSM frame duration)
- Buffer size: 640 frames (80 ms, 4 periods) for overrun tolerance
- Access: `SND_PCM_ACCESS_RW_INTERLEAVED`
- Expected loopback latency: ~40-60 ms (2-3 periods read + write),
  well within the 500 ms round-trip target

## 4. USB Device Auto-Detection

**Decision**: Use libudev to enumerate USB devices and match by
vendor ID `2c7c` and product ID `0125`. From the matched USB device,
resolve the associated TTY serial ports and ALSA sound card.

**Rationale**: libudev is the standard Linux API for device
enumeration and provides stable, well-documented interfaces. Parsing
sysfs directly is fragile. The EC20 exposes multiple USB interfaces
(serial ports for AT commands, audio for UAC) all under the same
parent USB device, making libudev parent traversal the correct
approach.

**Alternatives considered**:
- Hardcoded `/dev/ttyUSB*` paths: break on different enumeration
  order, multiple USB serial devices. Rejected.
- sysfs direct parsing: works but fragile, not recommended by udev
  maintainers. Rejected.
- libusb: too low-level for device discovery (designed for direct
  USB I/O, not for finding kernel-assigned device nodes). Rejected.

**Implementation approach**:
1. Enumerate udev devices with subsystem "usb", devtype "usb_device"
2. Match `idVendor=2c7c`, `idProduct=0125`
3. From matched parent, enumerate children:
   - Subsystem "tty" -> serial port paths (typically ttyUSB0-3;
     AT command port is usually the second or third)
   - Subsystem "sound" -> ALSA card number -> `hw:X,0` device name
4. CLI overrides (`--serial`, `--audio`) skip auto-detection for
   the respective device

## 5. Serial Port Communication

**Decision**: Use POSIX termios API for serial port communication.
Configure for 115200 baud, 8N1, no flow control. Read AT command
responses and URCs (unsolicited result codes) line-by-line.

**Rationale**: The EC20 AT command interface uses standard serial
communication at 115200 baud. POSIX termios is available on all Linux
systems with zero external dependencies. The AT command protocol is
line-oriented (CR/LF terminated), making simple line-buffered reading
sufficient.

**Alternatives considered**:
- Boost.Asio serial port: adds Boost dependency for minimal gain.
  Rejected per constitution Principle V (simplicity).
- libserialport: external dependency for a trivial use case. Rejected.

## 6. Testing Strategy

**Decision**: Use Google Test (GTest) as the test framework. Tests
are primarily integration tests that exercise real ALSA devices and
serial ports. Use the Linux kernel's `snd-pcmtest` virtual driver
to provide a test ALSA device in CI environments without physical
hardware.

**Rationale**: Constitution Principle I mandates integration-first
testing. GTest is the standard C++ test framework (Apache 2.0
license). The `snd-pcmtest` kernel module creates virtual ALSA
devices that accept any PCM configuration, enabling integration tests
of the audio pipeline without a physical EC20. For serial port
testing, a virtual serial pair (`socat`) can simulate AT command
responses.

**Alternatives considered**:
- Catch2: viable alternative (BSL-1.0 license), but GTest has
  broader ecosystem support and is already the de-facto standard for
  C++ projects. Rejected.
- Pure unit tests with mocked ALSA: violates constitution
  Principle I. Rejected.

**Test infrastructure**:
- `snd-pcmtest` kernel module for virtual ALSA devices
- `socat` for virtual serial port pairs (pty-pty)
- GTest + CTest integration via CMake

## 7. Build System

**Decision**: Use CMake as the build system generator with a
Makefile wrapper at the repository root. CMake generates the actual
build files; the Makefile provides the constitution-mandated targets
(`build`, `test`, `run`, `clean`, `lint`).

**Rationale**: CMake is the standard C++ build system with native
support for dependency discovery (`find_package`), test integration
(CTest), and cross-platform builds. The Makefile wrapper satisfies
constitution Principle IV while delegating actual build logic to
CMake. This avoids writing complex raw Makefiles for C++ compilation.

**Alternatives considered**:
- Raw Makefile only: error-prone for C++ dependency management,
  no CTest integration. Rejected.
- Meson: capable but less ecosystem support for C++ package
  discovery. Rejected.
- Bazel: overkill for a single-binary project. Rejected.

## 8. C++ Standard and Compiler

**Decision**: C++17 with GCC (g++). Minimum GCC version 9.0.

**Rationale**: C++17 provides `std::optional`, `std::string_view`,
structured bindings, and `std::filesystem` -- all useful for this
project without requiring C++20 module support or ranges. GCC 9+ is
widely available on current Linux distributions.

**Alternatives considered**:
- C++20: provides `std::format` and concepts but wider compiler
  support is not needed for this project's scope. Rejected per
  simplicity.
- C++14: lacks `std::optional` and `std::filesystem`. Rejected.
- Clang: viable but GCC is more commonly pre-installed on embedded
  Linux targets. Not rejected, but GCC is primary.

## 9. Dependencies and Licensing

| Dependency | Purpose | License | Required |
|------------|---------|---------|----------|
| libasound2 (ALSA lib) | Audio capture/playback | LGPL-2.1 | Yes |
| libudev | USB device enumeration | LGPL-2.1 | Yes |
| GTest | Testing framework | Apache-2.0 | Dev only |
| CMake | Build system | BSD-3-Clause | Dev only |
| socat | Virtual serial pairs for tests | GPL-2.0 | Dev only |

All runtime dependencies (libasound2, libudev) are LGPL-2.1, which
permits linking from proprietary or permissively-licensed code without
copyleft obligations on the application itself. GTest (Apache-2.0)
and CMake (BSD-3-Clause) are development-only and fully
corporate-friendly.

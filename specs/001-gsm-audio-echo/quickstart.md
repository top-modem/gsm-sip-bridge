# Quickstart: GSM Audio Echo

**Branch**: `001-gsm-audio-echo` | **Date**: 2026-05-02

## Prerequisites

- Linux (Debian/Ubuntu recommended, kernel 3.11+)
- GCC 9+ with C++17 support
- CMake 3.14+
- ALSA development headers (`libasound2-dev`)
- libudev development headers (`libudev-dev`)
- Google Test (`libgtest-dev`)
- Quectel EC20 module connected via USB with SIM card inserted
- EC20 firmware EC20CEFAGR06A15M4G or later (UAC support)

## Install System Dependencies

```bash
sudo apt update
sudo apt install -y build-essential cmake g++ \
    libasound2-dev libudev-dev libgtest-dev \
    socat
```

## One-Time EC20 Setup (enable USB Audio)

Connect to the EC20 AT command port and enable UAC:

```bash
# Find the AT command port (usually ttyUSB2 or ttyUSB3)
ls /dev/ttyUSB*

# Open a serial session (install minicom if needed: sudo apt install minicom)
minicom -D /dev/ttyUSB2 -b 115200

# In minicom, type these AT commands:
# Check current USB config
AT+QCFG="USBCFG"
# Enable UAC (last parameter = 1)
AT+QCFG="USBCFG",0x2C7C,0x0125,1,1,1,1,1,0,1
# Reboot module to apply
AT+CFUN=1,1
```

After reboot, verify the audio device appears:

```bash
arecord -l   # Should show a card named "Android" or similar
aplay -l     # Same card should appear for playback
```

## Build and Run

```bash
git clone <repo-url> audio-echo
cd audio-echo

make build   # Compile the binary
make test    # Run all integration tests
make run     # Start the audio echo program
```

## Verify It Works

1. Start the program: `make run`
2. Call the SIM card's phone number from any phone
3. The program logs `RING` and `call answered, echo active`
4. Speak into your phone -- you hear your own voice echoed back
5. Hang up -- the program logs `call ended` and waits for the next
   call
6. Press Ctrl+C to stop the program

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Compile the audio-echo binary |
| `make test` | Run the full integration test suite |
| `make run` | Build and run audio-echo with auto-detection |
| `make clean` | Remove all build artifacts |
| `make lint` | Run static analysis / linter |
| `make help` | Show all available targets |

## Troubleshooting

**No `/dev/ttyUSB*` devices appear**:
The EC20 USB serial driver may not be loaded. Check `dmesg | grep
ttyUSB` and ensure the `option` and `qcserial` kernel modules are
loaded.

**`arecord -l` does not show an audio device**:
UAC mode is not enabled on the EC20. Follow the one-time setup above.
Ensure firmware is EC20CEFAGR06A15M4G or later.

**"Permission denied" on serial port or audio device**:
Add your user to the `dialout` and `audio` groups:
```bash
sudo usermod -aG dialout,audio $USER
```
Log out and back in for group changes to take effect.

**Audio has clicks or dropouts**:
The system may be under CPU pressure. Ensure no other process is
using the ALSA device (`fuser /dev/snd/*`). Consider running the
program with real-time scheduling priority.

# Quickstart: GSM to SIP Audio Bridge

## Prerequisites

- Linux with ALSA (`libasound2-dev`)
- PJSIP development libraries (`libpjproject-dev`)
- CMake 3.14+, GCC/Clang with C++17
- Quectel EC20 module connected via USB with active SIM
- SIP server account (PBX, Asterisk, FreePBX, etc.)

## Setup

```bash
git clone <repo-url> && cd audio-echo
cp config.ini.example config.ini
```

Edit `config.ini` with SIP credentials and bridge destination:

```ini
[sip]
server = your-pbx.example.com
username = bridge-account
password = your-password

[bridge]
sip_destination = 599
```

## Build and Run

```bash
make build
make run-bridge
```

## Verify

1. Call the GSM number from a phone
2. Hear repeating beeps while the SIP extension rings
3. Answer the SIP extension -- beeps stop, two-way audio begins
4. Hang up from either side -- both legs terminate

## Troubleshooting

| Symptom | Check |
|---------|-------|
| "no EC20 module found" | Verify USB connection: `lsusb \| grep 2c7c` |
| "SIM not registered" | Check antenna and SIM: `minicom -D /dev/ttyUSB2` then `AT+COPS?` |
| "SIP registration failed" | Verify credentials in config.ini, check PBX logs |
| "SIP call failed" | Verify `sip_destination` is a valid extension on the PBX |
| No audio after SIP answers | Check `AT+QPCMV=1,2` succeeded in logs |
| ModemManager warning | Run `sudo systemctl stop ModemManager` |

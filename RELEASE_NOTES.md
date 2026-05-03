# Release Notes

## v4.1.0

- **Call Logging** -- Every incoming GSM call is recorded in a local SQLite database with caller ID, module ID, timestamp, duration, SIP destination, and outcome (answered/missed/failed).
- **SMS Persistence** -- All received SMS messages are stored in SQLite with sender, body, timestamp, module, and Discord forwarding status, surviving restarts and Discord outages.
- **sqlite-web UI** -- Docker Compose stack now includes a read-only web interface for browsing call and SMS records at `http://localhost:8088`.

## v4.0.0

- **SMS-to-Discord Forwarding** -- Captures incoming SMS from all modules, persists to a local SQLite database, and posts rich embed notifications to a configurable Discord webhook.
- **SMS Monitoring** -- Independent SMS polling on all modules via AT commands (`AT+CMGL`), with automatic SIM cleanup after read.
- **Configurable via `[sms]` section** -- Enable/disable SMS, set Discord webhook URL, and configure database path in `config.ini`.

## v3.0.1

- **Build Performance** -- PJSIP Docker build layer is now cached across branches and tags, significantly reducing CI build times.
- **CMake FetchContent** -- Replaced vendored mINI header with CMake FetchContent for cleaner dependency management.
- **License** -- Added GNU GPL v3 license.

## v3.0.0

- **Prometheus Metrics** -- Exposes call counts, SIP registration state, module health, audio errors, and call duration histograms on a `/metrics` endpoint (default port 9091).
- **Grafana Dashboard** -- Ships a pre-provisioned dashboard with panels for system overview, call rates, active calls, duration percentiles, module health, and error rates.
- **Docker Compose Monitoring Stack** -- One-command deployment of the bridge with Prometheus and Grafana in host network mode.

## v2.0.0

- **Multi-Card Support** -- Detects all connected EC20 modules at startup, assigns stable hardware IDs derived from USB serial numbers, and handles concurrent calls across modules independently.
- **Automatic Module Recovery** -- Failed modules (SIM issues, serial errors) are retried every 30 seconds and rejoin the active pool when functional.
- **Single-Card Override** -- Explicit `--serial` and `--audio` flags bypass auto-detection for single-module setups.

## v1.1.0

- **DID Passthrough** -- `sip_destination` is now optional. When empty, the GSM caller's number is used as the SIP DID, letting the PBX inbound route decide the destination extension.
- **SIP Media Renegotiation Fix** -- Audio bridge now reconnects correctly after SIP re-INVITE (media hold/resume scenarios).
- **SIP TCP Transport Fix** -- Fixed connection type when using TCP transport.

## v1.0.0

- **GSM-to-SIP Call Bridging** -- Auto-answers incoming GSM calls on a Quectel EC20 module and bridges audio bidirectionally to a SIP extension via a PBX.
- **SIP Audio Echo** -- Standalone SIP echo server for testing (echoes audio back to caller).
- **GSM Audio Echo** -- Standalone GSM echo tool for hardware validation (echoes modem audio back to caller).
- **Caller ID Forwarding** -- GSM caller's number is forwarded to SIP via P-Asserted-Identity header for DID routing.
- **Lock-Free Audio Pipeline** -- SPSC ring buffers connect ALSA capture/playback to the PJSIP conference bridge with minimal latency.
- **USB Auto-Discovery** -- Detects EC20 modules by scanning the USB bus for vendor/product ID `2c7c:0125`.
- **Docker + CI** -- Multi-platform Docker image (amd64/arm64) with GitHub Actions CI pipeline.

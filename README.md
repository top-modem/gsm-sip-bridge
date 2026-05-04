# gsm-sip-bridge v5.0.0

Bridges incoming GSM calls on Quectel EC20 modules to a SIP extension, forwards SMS to Discord, and exposes Prometheus metrics -- rewritten in Rust for memory safety and performance.

## Quick Start

```bash
git clone <repo-url> audio-echo && cd audio-echo
make build
make test
cp config.toml.example config.toml  # edit with your PBX details
export SIP_PASSWORD=yourpassword
make run
```

## Prerequisites

- Rust stable (pinned by `rust-toolchain.toml`)
- System packages: `build-essential`, `pkg-config`, `clang`, `libclang-dev`
- Libraries: `libasound2-dev`, `libusb-1.0-0-dev`, `libpjproject-dev` (>= 2.14)
- Test utilities: `socat` (for PTY-based integration tests)
- Hardware: One or more Quectel EC20 USB modems

## Architecture

Three-crate Cargo workspace:

| Crate | Role |
|---|---|
| `pjsua-sys` | Auto-generated FFI bindings to PJSIP's C `pjsua` API |
| `pjsua-safe` | Safe Rust wrappers (all `unsafe` blocks carry `// SAFETY:` comments) |
| `gsm-sip-bridge` | The binary crate -- zero `unsafe` |

```text
┌──────────────────────────────────────────────┐
│                  main.rs                      │
├──────────────┬──────────┬────────────────────┤
│  CardPool    │ SipBridge│   SmsHandler       │
│  (modules/)  │ (sip/)   │   (sms/)           │
├──────────────┴──────────┴────────────────────┤
│  config  │  metrics  │  store  │  runtime    │
├──────────┴───────────┴─────────┴─────────────┤
│          pjsua-safe  ←  pjsua-sys            │
└──────────────────────────────────────────────┘
```

## Features

1. Auto-answer incoming GSM calls on all connected EC20 modules
2. Bridge audio bidirectionally to a configured SIP extension
3. Multi-card support (up to 8 simultaneous modules)
4. Module hot-plug and automatic retry on failure
5. DID passthrough or fixed SIP destination routing
6. 400 Hz comfort beep during SIP dial phase
7. SMS capture, SIM cleanup, and Discord webhook forwarding
8. Prometheus metrics endpoint (`/metrics` on port 9091)
9. SQLite persistence for call and SMS records
10. Graceful shutdown with SIGTERM/SIGINT handling
11. Secret redaction in all log output
12. ModemManager conflict detection at startup

## Configuration

See [`docs/configuration.md`](docs/configuration.md) for a full reference.

Secrets support `env:VAR_NAME` syntax to avoid plaintext in config files:

```toml
[sip]
server = "pbx.example.com"
password = "env:SIP_PASSWORD"

[sms]
discord_webhook_url = "env:DISCORD_WEBHOOK_URL"
```

## Docker Compose

```bash
cd docker
cp ../config.toml.example config.toml  # edit
echo "SIP_PASSWORD=secret" > .env
docker compose up -d
```

Services: bridge + Prometheus + Grafana + sqlite-web (read-only DB viewer on port 8088).

## Make Targets

| Target | Description |
|---|---|
| `make build` | Build all crates in release mode |
| `make test` | Run all workspace tests |
| `make run` | Start the bridge |
| `make lint` | Clippy + rustfmt check + cargo-deny |
| `make coverage` | Generate lcov coverage report |
| `make docker-build` | Build the Docker image |
| `make help` | Show all targets |

## Migration from v4.1.x

See [`docs/migrating-from-v4.1.x.md`](docs/migrating-from-v4.1.x.md) for a complete upgrade guide covering configuration, database, metrics, and Docker.

## Documentation

- [Configuration Reference](docs/configuration.md)
- [Operations Guide](docs/operations.md)
- [Migration Guide](docs/migrating-from-v4.1.x.md)
- [Technical Plan](specs/008-rust-rewrite/plan.md)
- [Feature Specification](specs/008-rust-rewrite/spec.md)

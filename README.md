# gsm-sip-bridge v5.0.0

Bridges incoming GSM calls on Quectel EC20 modules to a SIP extension, forwards SMS to Discord, and exposes Prometheus metrics -- rewritten in Rust for memory safety and performance.

**Status**: Under active development on branch `008-rust-rewrite`.

## Quick Start

```bash
git clone <repo-url> audio-echo && cd audio-echo
make build
make test
```

## Architecture

Three-crate Cargo workspace:

| Crate | Role |
|---|---|
| `pjsua-sys` | Auto-generated FFI bindings to PJSIP's C `pjsua` API |
| `pjsua-safe` | Safe Rust wrappers (all `unsafe` blocks carry `// SAFETY:` comments) |
| `gsm-sip-bridge` | The binary crate -- zero `unsafe` |

Full plan: [`specs/008-rust-rewrite/plan.md`](specs/008-rust-rewrite/plan.md)

## Prerequisites

- Rust stable (pinned by `rust-toolchain.toml`)
- `build-essential`, `pkg-config`, `clang`, `libclang-dev`
- `libasound2-dev`, `libusb-1.0-0-dev`, `libpjproject-dev` (>= 2.14)
- `socat` (for integration tests)

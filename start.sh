#!/bin/bash
set -euo pipefail

export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
CARGO_BIN="${CARGO_HOME}/bin/cargo"
if [ ! -x "$CARGO_BIN" ]; then
    CARGO_BIN="/home/ht/.cargo/bin/cargo"
fi
HAS_CONFIG=false
ARGS=()
for arg in "$@"; do
    case "$arg" in
        run)          ;;                     # ignore "run"
        -c|--config)  HAS_CONFIG=true; ARGS+=("$arg") ;;
        *)            ARGS+=("$arg") ;;
    esac
done
if [ "$HAS_CONFIG" = false ]; then
    ARGS=("--config" "config.toml" "${ARGS[@]}")
fi
exec sudo env "PATH=$PATH" stdbuf -oL -eL "$CARGO_BIN" run --bin gsm-sip-bridge --features pjsip-linked -- "${ARGS[@]}" 2>&1 | tee gsm-bridge.log

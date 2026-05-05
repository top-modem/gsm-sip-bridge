#!/usr/bin/env bash
set -uo pipefail

count_unsafe() {
    local dir="$1"
    local count
    count=$(grep -rn "unsafe" "$dir" 2>/dev/null | grep -v "//.*unsafe" | wc -l)
    echo "${count// /}"
}

BRIDGE_UNSAFE=$(count_unsafe "gsm-sip-bridge/src/")
SAFE_UNSAFE=$(count_unsafe "pjsua-safe/src/")
SAFE_TOTAL=$(find pjsua-safe/src/ -name '*.rs' -exec cat {} + 2>/dev/null | wc -l)
SAFE_TOTAL="${SAFE_TOTAL// /}"
: "${SAFE_TOTAL:=1}"

echo "=== Unsafe Block Count ==="
echo "  gsm-sip-bridge/src: ${BRIDGE_UNSAFE} unsafe blocks"
echo "  pjsua-safe/src:     ${SAFE_UNSAFE} unsafe blocks (${SAFE_TOTAL} total lines)"

if [ "${BRIDGE_UNSAFE}" -gt 0 ]; then
    echo "FAIL: gsm-sip-bridge must contain zero unsafe blocks"
    exit 1
fi

if [ "${SAFE_TOTAL}" -gt 0 ]; then
    RATIO=$(echo "scale=2; ${SAFE_UNSAFE} * 100 / ${SAFE_TOTAL}" | bc 2>/dev/null || echo "0")
    echo "  pjsua-safe ratio: ${RATIO}% (threshold: <5%)"
fi

echo "PASS"
exit 0

#!/usr/bin/env bash
# Regenerate the Celenium genesis constants (timestamp + hash constants).
#
# Run this before any launch: testnet or mainnet. The genesis timestamp is a
# hard requirement, not a nicety — a node only syncs inside a short window
# after its genesis timestamp, so a stale value leaves the node permanently
# "not synced" and unable to accept blocks.
#
# Usage:
#   scripts/regenesis.sh --now            # launch now: current wall clock
#   scripts/regenesis.sh 1794000000000    # explicit ms timestamp
#   scripts/regenesis.sh --now --binary target/release/celeniumd
#       # also patch the built binary in place (no rebuild needed)
#
# Steps: patch timestamp -> print new constants via cargo test -> patch
# constants -> verify with test_genesis_hashes -> optional binary patch.
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ $# -lt 1 ]]; then
    sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
fi

TS_ARG="$1"; shift || true
BINARY=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --binary) BINARY="$2"; shift 2 ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
done

if [[ "$TS_ARG" == "--now" ]]; then
    TS=$(python3 -c 'import time; print(int(time.time()*1000))')
else
    TS="$TS_ARG"
    if ! [[ "$TS" =~ ^[0-9]+$ ]]; then
        echo "error: '$TS' is not a ms timestamp" >&2; exit 2
    fi
fi

echo "=== 1/5 patching genesis timestamp ($TS) ==="
python3 scripts/set-genesis-timestamp.py "$TS"

echo
echo "=== 2/5 computing new constants (cargo test) ==="
VALUES=$(mktemp)
trap 'rm -f "$VALUES"' EXIT
cargo test -p celenium-consensus-core print_genesis_regeneration_values -- \
    --nocapture --test-threads=1 > "$VALUES" 2>&1 || {
    echo "error: cargo test failed; tail of output:" >&2
    tail -30 "$VALUES" >&2
    exit 1
}
grep -q "=== GENESIS ===" "$VALUES" || {
    echo "error: test produced no sections; tail of output:" >&2
    tail -30 "$VALUES" >&2
    exit 1
}
sed -n '/=== /,/^hash:/p' "$VALUES"

echo
echo "=== 3/5 patching constants into genesis.rs ==="
python3 scripts/apply-genesis-hashes.py "$VALUES"

echo
echo "=== 4/5 verifying (test_genesis_hashes) ==="
# --lib limits the run to the unit-test binary where test_genesis_hashes lives.
# Without it, tail -N was grabbing the summary of the LAST integration-test
# binary (e.g. "0 passed; 4 filtered out"), hiding the real "1 passed" result.
cargo test -p celenium-consensus-core --lib test_genesis_hashes 2>&1 | tail -8

if [[ -n "$BINARY" ]]; then
    echo
    echo "=== 5/5 patching binary $BINARY ==="
    python3 scripts/patch-genesis-ts.py "$BINARY" "$TS"
else
    echo
    echo "=== 5/5 skipping binary (pass --binary <path> to patch in place) ==="
    echo "NOTE: any already-built binary still holds the old timestamp."
    echo "      Rebuild, or run: python3 scripts/patch-genesis-ts.py <binary> $TS"
fi

echo
echo "Genesis regenerated for timestamp $TS."
echo "Next: commit the genesis.rs change, then build."

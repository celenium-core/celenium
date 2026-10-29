#!/usr/bin/env python3
"""Patch the genesis timestamp inside a built celenium binary.

The timestamp is stored as a raw `u64` (little-endian) in each of the four
network genesis blocks, not as text, so it cannot be found with a decimal
string search. Expect exactly 4 occurrences; anything else means the binary
predates the current constants or was built from different source.

Usage:
    patch-genesis-ts.py <binary> [<new-ts-ms> | --now]

Without a value it reports the occurrences it finds and exits.
"""
import struct
import sys
import time
from pathlib import Path

CURRENT_TS = 1793229666249  # test value baked into the current source
EXPECTED = 4


def find(data: bytes, value: int) -> list[int]:
    needle = struct.pack("<Q", value)
    offsets, i = [], 0
    while True:
        j = data.find(needle, i)
        if j < 0:
            return offsets
        offsets.append(j)
        i = j + 1


def main() -> int:
    if len(sys.argv) < 2:
        print((__doc__ or "").strip())
        return 2

    path = Path(sys.argv[1])
    data = path.read_bytes()

    if len(sys.argv) == 2:
        offs = find(data, CURRENT_TS)
        print(f"{path}: {len(offs)} occurrence(s) of ts {CURRENT_TS} at {[hex(o) for o in offs]}")
        print(f"expected {EXPECTED}: {'OK' if len(offs) == EXPECTED else 'MISMATCH — binary is stale'}")
        return 0 if len(offs) == EXPECTED else 1

    new_ts = int(time.time() * 1000) if sys.argv[2] == "--now" else int(sys.argv[2])
    if not 1_600_000_000_000 < new_ts < 4_000_000_000_000:
        print(f"refusing: {new_ts} is not a plausible ms timestamp")
        return 1

    offs = find(data, CURRENT_TS)
    if len(offs) != EXPECTED:
        print(f"error: found {len(offs)} occurrence(s), expected {EXPECTED}; aborting")
        return 1

    new_bytes = struct.pack("<Q", new_ts)
    out = bytearray(data)
    for off in offs:
        out[off : off + 8] = new_bytes

    path.write_bytes(bytes(out))
    print(f"patched {path}: ts {CURRENT_TS} -> {new_ts} at {[hex(o) for o in offs]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Set the genesis timestamp in consensus/core/src/config/genesis.rs.

Patches the `timestamp:` field of every `pub const <NAME>: GenesisBlock`
block. The struct definition's own `pub timestamp: u64,` field is never
touched because only GenesisBlock blocks are scanned.

Usage:
    set-genesis-timestamp.py <ms-since-epoch>   # explicit value
    set-genesis-timestamp.py --now              # current wall clock
"""
import re
import sys
import time
from pathlib import Path

TARGET = Path("consensus/core/src/config/genesis.rs")

BLOCK_RE = re.compile(
    r"pub const (\w+): GenesisBlock = GenesisBlock \{", re.MULTILINE
)
TS_RE = re.compile(r"(timestamp:\s*)(\d+)(,)")


def block_spans(text: str) -> list[tuple[str, int, int]]:
    """Yield (name, start, end) for each GenesisBlock literal in the file."""
    spans = []
    for m in BLOCK_RE.finditer(text):
        start = m.end() - 1  # include the opening brace
        depth = 0
        i = start
        while i < len(text):
            c = text[i]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    spans.append((m.group(1), m.start(), i + 1))
                    break
            i += 1
    return spans


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in ("--now",) and not sys.argv[1].isdigit():
        print(__doc__.strip())
        return 2

    if sys.argv[1] == "--now":
        ts = int(time.time() * 1000)
    else:
        ts = int(sys.argv[1])

    if not 1_600_000_000_000 < ts < 4_000_000_000_000:
        print(f"refusing: {ts} is not a plausible ms timestamp")
        return 1

    text = TARGET.read_text()
    spans = block_spans(text)
    if not spans:
        print("error: no GenesisBlock literals found")
        return 1

    changed = []
    out = []
    cursor = 0
    for name, start, end in spans:
        block = text[start:end]
        new_block, n = TS_RE.subn(lambda m: f"{m.group(1)}{ts}{m.group(3)}", block, count=1)
        if n != 1:
            print(f"error: no timestamp field in {name}")
            return 1
        changed.append(name)
        out.append(text[cursor:start])
        out.append(new_block)
        cursor = end
    out.append(text[cursor:])

    TARGET.write_text("".join(out))
    print(f"set timestamp {ts} in {len(changed)} block(s): {', '.join(changed)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

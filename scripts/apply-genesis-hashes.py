#!/usr/bin/env python3
"""Apply genesis hash constants printed by the
`print_genesis_regeneration_values` test into genesis.rs.

Run after changing the timestamp (or the premine outputs) and rebuild:

    cargo test -p cenium-consensus-core print_genesis_regeneration_values \
        -- --nocapture > /tmp/genesis-values.txt
    python3 scripts/apply-genesis-hashes.py /tmp/genesis-values.txt

Each `=== NAME ===` section must carry `hash_merkle_root`, `utxo_commitment`
and `hash` as a `{:#04x?}` byte array. The NAME must match a GenesisBlock
literal in genesis.rs.
"""
import re
import sys
from pathlib import Path

TARGET = Path("consensus/core/src/config/genesis.rs")

SECTION_RE = re.compile(r"=== (\w+) ===")
# `field: [\n 0x12, 0x34, ...\n]`
FIELD_RE = re.compile(
    r"^(\w+):\s*\[\s*\n((?:\s*0x[0-9a-fA-F]{2},\s*\n)+)\s*\]", re.MULTILINE
)
BLOCK_RE = re.compile(r"pub const (\w+): GenesisBlock = GenesisBlock \{", re.MULTILINE)

FIELDS = ("hash_merkle_root", "utxo_commitment", "hash")


def parse(text: str) -> dict[str, dict[str, list[str]]]:
    """Return {network: {field: [hex byte strings]}}."""
    sections = {}
    matches = list(SECTION_RE.finditer(text))
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        body = text[m.start() : end]
        fields = {}
        for fm in FIELD_RE.finditer(body):
            name = fm.group(1)
            if name not in FIELDS:
                continue
            bytes_ = re.findall(r"0x([0-9a-fA-F]{2})", fm.group(2))
            if len(bytes_) != 32:
                raise SystemExit(f"error: {m.group(1)}.{name} has {len(bytes_)} bytes, expected 32")
            fields[name] = [f"0x{b.lower()}" for b in bytes_]
        missing = set(FIELDS) - set(fields)
        if missing:
            raise SystemExit(f"error: section {m.group(1)} missing {sorted(missing)}")
        sections[m.group(1)] = fields
    return sections


def format_field(name: str, values: list[str], indent: str) -> str:
    lines = [f"{indent}{name}: Hash::from_bytes(["]
    for i in range(0, 32, 8):
        chunk = ", ".join(values[i : i + 8])
        lines.append(f"{indent}    {chunk},")
    lines.append(f"{indent}]),")
    return "\n".join(lines)


def block_spans(text: str) -> list[tuple[str, int, int]]:
    spans = []
    for m in BLOCK_RE.finditer(text):
        start = m.end() - 1
        depth, i = 0, start
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    spans.append((m.group(1), m.start(), i + 1))
                    break
            i += 1
    return spans


def main() -> int:
    if len(sys.argv) != 2:
        print((__doc__ or "").strip())
        return 2

    sections = parse(Path(sys.argv[1]).read_text())
    if not sections:
        raise SystemExit("error: no sections parsed")

    text = TARGET.read_text()
    spans = {name: (start, end) for name, start, end in block_spans(text)}
    unknown = set(sections) - set(spans)
    if unknown:
        raise SystemExit(f"error: sections not present in genesis.rs: {sorted(unknown)}")

    out, cursor, patched = [], 0, []
    for name, (start, end) in sorted(spans.items(), key=lambda kv: kv[1][0]):
        fields = sections.get(name)
        if fields is None:
            continue
        block = text[start:end]
        for field, values in fields.items():
            pat = re.compile(
                rf"(?P<indent>[ \t]*){field}: Hash::from_bytes\(\[.*?\]\),", re.DOTALL
            )
            new_block, n = pat.subn(
                lambda m: format_field(field, values, m.group("indent")),
                block,
                count=1,
            )
            if n != 1:
                raise SystemExit(f"error: could not patch {name}.{field}")
            block = new_block
        out.append(text[cursor:start])
        out.append(block)
        cursor = end
        patched.append(name)
    out.append(text[cursor:])

    TARGET.write_text("".join(out))
    print(f"patched {', '.join(FIELDS)} in: {', '.join(patched)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

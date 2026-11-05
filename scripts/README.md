# Celenium operational scripts

## `regenesis.sh` — regenerate the genesis constants (required before every launch)

```bash
scripts/regenesis.sh --now                          # current wall clock
scripts/regenesis.sh 1794000000000                  # explicit ms timestamp
scripts/regenesis.sh --now --binary target/release/celeniumd
```

What it does, in order:

1. `set-genesis-timestamp.py` — patches the `timestamp:` field of all four
   `GenesisBlock` literals in `consensus/core/src/config/genesis.rs`.
2. `cargo test -p celenium-consensus-core print_genesis_regeneration_values
   -- --nocapture` — prints the new `hash_merkle_root`, `utxo_commitment` and
   `hash` for each network. These are computed by the consensus code itself,
   never by hand.
3. `apply-genesis-hashes.py` — writes those constants back into `genesis.rs`.
4. `cargo test -p celenium-consensus-core test_genesis_hashes` — proves the
   constants match what the code computes.
5. Optional: `patch-genesis-ts.py <binary>` — patches an already-built binary
   in place so you can test without a rebuild.

The cycle is idempotent: running it with the timestamp already in the source
leaves `genesis.rs` byte-identical. That property is the regression test for
the whole tooling.

## Why the timestamp matters

A node only accepts blocks inside a short window after its genesis timestamp.
A stale value leaves every node permanently "not synced", which looks like a
mining failure but is not. See `PRE-MAINNET-CHECKLIST.md`.

## `patch-genesis-ts.py` — binary-level timestamp patch

```bash
python3 scripts/patch-genesis-ts.py target/release/celeniumd           # report only
python3 scripts/patch-genesis-ts.py target/release/celeniumd --now     # patch
```

The timestamp is a raw `u64` (little-endian) inside each genesis block, not
text, so a decimal string search finds nothing. The script refuses to patch
unless it finds exactly four occurrences — a different count means the binary
predates the current constants and patching it would produce a coin that
cannot sync.

Note that `CURRENT_TS` in the script must match the timestamp baked into the
source. Update it whenever `regenesis.sh` runs, or the check reports a false
mismatch.

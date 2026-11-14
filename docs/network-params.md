# Network Parameters: Kaspa ke CEN

Dokumen ini mendaftar setiap parameter jaringan yang diubah di fork
rusty-kaspa (basis: snapshot upstream commit `35efec7`).

Status: **identitas, port, emisi, genesis, premine CLTV, dan regenesis
konstanta sudah diterapkan** di working copy `~/celenium-build` (terakhir
11 November 2026).

## Identitas jaringan (sudah diterapkan)

| # | File | Yang diubah | Sebelum (Kaspa) | Sesudah (CEN) |
|---|---|---|---|---|
| 1 | `crypto/addresses/src/lib.rs` | Enum `Prefix` + `try_from` + test vector address (crypto/addresses, 302 vektor `wallet/keys`, contoh JS) | `kaspa`, `kaspatest`, `kaspasim`, `kaspadev` | `cen69`, `cen69test`, `cen69sim`, `cen69dev` |
| 2 | `celeniumd/src/daemon.rs` (baris 84-90) | Direktori data default | `.rusty-kaspa` | `.celenium` |
| 3 | `celeniumd/Cargo.toml` | Nama binary | `kaspad` | `celeniumd` |
| 4 | `core/src/celeniumd_env.rs` | Nama env/runtime | `kaspad` | `celeniumd` |

Address pakai HRP `cen69` (varian `cen69test/cen69sim/cen69dev`); bech32 tidak
berubah, checksum 8 karakter polymod 40-bit tetap dihitung atas HRP sehingga
setiap kemunculan alamat di test vector harus di-encode ulang (sudah: 302
vektor wallet + contoh JS + tx_e2e, 1 Nov 2026).

## Port (sudah diterapkan)

| # | Fungsi di `consensus/core/src/network.rs` | Sebelum | Sesudah |
|---|---|---|---|
| 5 | `default_p2p_port` mainnet | 16111 | 26111 |
| 6 | `default_p2p_port` testnet (tanpa suffix, 10, 12) | 16411, 16211, 16311 | 26411, 26211, 26311 |
| 7 | `default_p2p_port` simnet / devnet | 16511, 16611 | 26511, 26611 |
| 8 | `default_rpc_port` main/test/sim/dev | 16110, 16210, 16510, 16610 | 26110, 26210, 26510, 26610 |
| 9 | `default_borsh_rpc_port` | 17110, 17210, 17510, 17610 | 27110, 27210, 27510, 27610 |
| 10 | `default_json_rpc_port` | 18110, 18210, 18510, 18610 | 28110, 28210, 28510, 28610 |

Default di `bridge/config.yaml` dan `rpc/wrpc/server/src/service.rs` juga
ikut disesuaikan (26110/27110).

## Emisi (sudah diterapkan)

| # | File | Parameter | Sebelum | Sesudah |
|---|---|---|---|---|
| 11 | `consensus/src/processes/coinbase.rs` | `SUBSIDY_BY_MONTH_TABLE` | 426 entri (emisi Kaspa) | 817 entri CEN, dari `scripts/data/subsidy_table.rs` |
| 12 | `consensus/src/processes/coinbase.rs` | `SUBSIDY_BY_MONTH_TABLE_SIZE` | 426 | 817 |
| 13 | `consensus/core/src/config/params.rs` | `deflationary_phase_daa_score` (4 network) | `TenBps::deflationary_phase_daa_score()` atau nilai mainnet | 0 (tabel aktif sejak genesis) |
| 14 | `params.rs` | `pre_deflationary_phase_base_subsidy` (4 network) | 50000000000 (50 KAS) | 2079547 (konsisten dengan reward era 0) |
| 15 | `params.rs` | `pre_crescendo_target_time_per_block` (main, test) | 1000 ms | 100 ms (10 BPS dari awal) |
| 16 | `params.rs` | `crescendo_activation` (main, test) | skor aktivasi mainnet | `ForkActivation::never()` |
| 17 | `params.rs` | `dns_seeders` | seed Kaspa | mainnet kosong (private dulu); testnet-10 `seed.celenium.test` (placeholder, ganti saat publik) |

Reward per blok = `table[bulan] / 10` (pembulatan ke atas di kode). Era 0:
rate 0.20795466 CEN/detik, reward 2.079.547 pradha (0.02079547 CEN)/blok,
total era 26,250,199.91 CEN. Total emisi 52,500,000 CEN hingga halving selesai.

## Genesis dan premine (sudah diterapkan)

| # | File | Yang diubah | Status |
|---|---|---|---|
| 18 | `consensus/core/src/config/genesis.rs` | `PREMINE_OUTPUTS`: 264 output coinbase — 132 dev (9,600,000 CEN) + 132 owner (6,900,000 CEN), tiap output P2SH CLTV + multisig 2-of-3 (generator `gen_premine_cltv.rs`) | sudah 11 Nov 2026 |
| 19 | `consensus/src/consensus/utxo_set_override.rs` | Fungsi `genesis_premine_utxos()`: mengubah output genesis coinbase jadi initial UTXO set, karena rusty-kaspa commit UTXO state genesis kosong (premine lewat output saja tidak akan bisa di-spend) | sudah |
| 20 | `consensus/src/consensus/factory.rs` | Panggil `set_genesis_utxo_commitment_from_config` dan `set_initial_utxo_set` tanpa feature gate (sebelumnya hanya devnet) | sudah |
| 21 | `celeniumd/src/args.rs`, `celeniumd/src/daemon.rs`, `consensus/core/src/errors/config.rs` | Hapus pembatasan devnet-only untuk prealloc UTXO | sudah |
| 22 | `consensus/core/src/config/genesis.rs` | Konstanta `GENESIS`, `TESTNET_GENESIS`, `SIMNET_GENESIS`, `DEVNET_GENESIS`: hash, hash_merkle_root, utxo_commitment, nonce PoW | sudah 11 Nov 2026: payload 264 output; nonce di-mine ulang untuk simnet/testnet/devnet (`mine_genesis.rs`, cap 2 miliar); mainnet mempertahankan nonce warisan — `test_genesis_hashes` PASS |

Catatan penting soal premine:

- UTXO premine ditandai `is_coinbase: false` supaya bisa di-spend (setelah lock
  terpenuhi). Lock-nya kini **on-chain**: `OP_CHECKLOCKTIMEVERIFY` (CLTV) +
  multisig 2-of-3 di redeem script tiap output, unlock tiap tanggal 5 bulan.
- Pubkey premine (3 dev + 3 owner) asli; private key disimpan terenkripsi di
  vault pemilik di luar repository. CLTV adalah time-lock konsensus (bukan
  komitmen operator), tapi spend tetap butuh 2-of-3 tanda tangan. Jangan pernah
  commit material kunci.

## Yang tetap (tidak diubah)

- GHOSTDAG: `ghostdag_k = 124`, `max_block_parents = 16`, mergeset limits
  (dari `BlockrateParams::new::<10>()`).
- Block rate 10 BPS dan seluruh parameter blockrate.
- Proof of Work kHeavyHash dan seluruh aturan DAA.
- Mass limits, KIP-0009 storage mass parameter.
- Struktur wallet, derivasi kunci, PSKT.
- Nama crate direname penuh ke `celenium_*` (commit rename fork); merge dari
  upstream Kaspa kini manual. Wire proto juga diganti `Kaspad*` → `Celeniumd*`
  (incompatible dengan klien Kaspa).

## Ringkasan file yang disentuh

```
node/crypto/addresses/src/lib.rs             prefix + test vector
node/crypto/addresses/src/wasm.rs            test vector
node/crypto/addresses/benches/bench.rs       test vector
node/crypto/txscript/src/standard.rs         test vector
node/consensus/core/src/network.rs           port
node/consensus/core/src/config/genesis.rs    genesis + 264 output premine CLTV
node/consensus/core/src/config/params.rs     emisi, seeds, difficulty
node/consensus/src/processes/coinbase.rs     tabel subsidy
node/consensus/src/consensus/utxo_set_override.rs  premine jadi UTXO
node/consensus/src/consensus/factory.rs      wiring premine
node/consensus/core/src/errors/config.rs     hapus error devnet-only
node/celeniumd/src/daemon.rs                  direktori data
node/celeniumd/src/args.rs                    prealloc tanpa gate + help port
node/celeniumd/Cargo.toml                     nama binary celeniumd
node/core/src/celeniumd_env.rs                nama runtime
node/wallet/**                               test vector address
node/bridge/**                               port default, nama folder
crypto/txscript/examples/gen_premine_cltv.rs generator CLTV + multisig premine
```

## Verifikasi genesis (premine)

Genesis constants (hash, hash_merkle_root, utxo_commitment) untuk keempat
network (mainnet, testnet, simnet, devnet) diregenerasi ulang 11 November 2026
setelah premine dipecah jadi 264 output CLTV + multisig 2-of-3; nonce PoW segar
di-mine untuk simnet/testnet/devnet (mine_genesis.rs), mainnet mempertahankan
nonce warisan karena target genesis mainnet membuat pencarian naive tidak
feasible dan pow genesis di-exempt oleh konsensus (paritas dengan rusty-kaspa):

- `hash_merkle_root`: dihitung dari transaksi genesis (termasuk 264 output premine)
- `utxo_commitment`: MuHash atas UTXO premine (264 output CLTV + multisig)
- Verifikasi: `cargo test --lib test_genesis_hashes` di `consensus/core`
  menghitung ulang hash dan membandingkan dengan constant di `genesis.rs` ->
  **semua cocok untuk 4 network**

Premine di-import sebagai initial UTXO set melalui
`genesis_premine_utxos()` di `consensus/src/consensus/utxo_set_override.rs`
(dipanggil dari `factory.rs` tanpa cfg gate, jadi aktif di semua network).
Lebih lanjut lihat `docs/premine-cltv-schedule.md` (jadwal lengkap 264 baris,
kunci publik, dan alamat P2SH tiap tranche).
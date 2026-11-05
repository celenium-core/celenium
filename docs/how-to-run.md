# How To Run

Panduan build dan menjalankan node CEN dari fork rusty-kaspa. Status tiap
langkah ditandai terverifikasi atau belum.

## 1. Toolchain Rust (terverifikasi di WSL Arch)

Environment ini pakai Arch Linux, bukan Ubuntu.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
sudo pacman -Sy --noconfirm base-devel clang cmake protobuf
```

Terverifikasi: rustc 1.99.1, cargo 1.99.1, protobuf ter-install.

## 2. Build (working copy lokal)

Node di Google Drive tidak boleh di-build langsung (filesystem virtual tidak
mendukung chmod, sangat lambat). Working copy ada di `~/celenium-build`:

```bash
source ~/.cargo/env
cd ~/celenium-build
cargo check --workspace      # cek compile semua crate
cargo build --release --bin celeniumd
```

Build pertama lama (bisa 20-60 menit tergantung mesin) karena mengompilasi
seluruh workspace.

Status terverifikasi: `cargo check --workspace` **0 error** (68 crate),
`cargo test` consensus-core **62 passed, 0 failed**, genesis constants
(termasuk utxo_commitment premine) cocok dengan nilai terhitung untuk 4
network. Build pertama `cargo build --release` butuh **1.5-2 jam** di mesin
4-core karena mengompilasi RocksDB C++ (720 file, -O3); ini one-time cost,
build berikutnya cepat.

## 3. Jalankan simnet (jaringan lokal) - TERVERIFIKASI

Hasil verifikasi (28 Okt 2026, binary `celeniumd 2.1.0-270c99c`):

```
[INFO] celeniumd v2.1.0-270c99c
[INFO] Importing the UTXO set of the pruning point eeb6fa5d...   <- premine genesis AKTIF
[INFO] Resyncing the utxoindex...
[INFO] GRPC Server starting on: 127.0.0.1:26510                  <- port CEN (bukan 16110)
[INFO] P2P Server starting on: 0.0.0.0:26511                     <- port CEN (bukan 16111)
```

Port CEN (terverifikasi dari log node):

| Layanan | Port CEN | Port Kaspa asli |
|---|---|---|
| P2P | 26511 | 16111 |
| gRPC (RPC) | 26510 | 16110 |

Simnet: difficulty rendah, `skip_proof_of_work: true`, cocok pengembangan.

```bash
~/celenium-build/target/release/celeniumd --simnet --utxoindex
```

Cek status dari terminal lain:

```bash
cd ~/celenium-build && cargo run --release --bin celenium -- simnet
```

Direktori data default: `~/.celenium/simnet` (sebelumnya `~/.rusty-kaspa`).

## 4. Mining di simnet

Pradana (fork rothschild) adalah transaction generator/miner bawaan:

```bash
cd ~/celenium-build && cargo run --release --bin pradana -- --simnet --mining-address <alamat_cen>
```

Alamat ber-prefix `cen69:` (mainnet), `cen69sim:` (simnet), dan
seterusnya. Contoh alamat simnet valid untuk test:
`cen69sim:` + payload 32 byte versi PubKey (generate lewat CLI kalau
perlu, jangan hardcode dari dokumen ini).

## 5. Verifikasi emisi

Setelah node jalan, reward blok harus sesuai `docs/emission-schedule.md`:
rate awal 0.13665592 CEN/detik, reward per blok 0.0136656 CEN pada 10 BPS.
Era 0 total 17,250,131.44 CEN.

## 6. Genesis baru

Genesis constants di `consensus/core/src/config/genesis.rs` belum
diregenerasi. Alur kerjanya:

1. Set hash sementara ke nol, isi field lain.
2. `cargo test --package kaspa-consensus-core genesis`
3. Baca hash merkle root dan hash blok dari pesan panic.
4. Update konstanta, ulangi sampai test lolos.

```bash
cd ~/celenium-build && cargo run --release --bin kaspa -- get-block 0 --simnet
```

Hash harus sama dengan nilai di genesis.rs. Detail prosedur ada di
`scripts/generate-genesis.md`.

## 7. Verifikasi premine

Premine 34,500,000 CEN (22.2M dev + 12.3M owner) harus muncul sebagai UTXO di
genesis lewat mekanisme initial UTXO set (lihat `network-params.md` bagian
Genesis). Cek saldo alamat premine setelah node sync genesis.

Catatan: pubkey premine sudah asli (a438515); private key-nya offline di
vault pemilik, di luar repo.

## Catatan environment

- Project ada di `G:\My Drive\project\celenium` (mount WSL `/mnt/g`). Entri
  `/etc/fstab` sudah ditambahkan supaya otomatis mount setelah reboot.
- Build selalu di disk native (`~/celenium-build`), jangan di Google Drive.
- Repo di `node/` sudah diset `core.fileMode=false` karena mount 9p membuat
  semua file terlihat executable.
- Setelah perubahan stabil di `~/celenium-build`, salin kembali ke `node/` di
  Google Drive dengan `cp -r` (bukan `cp -a`, timestamp tidak didukung).

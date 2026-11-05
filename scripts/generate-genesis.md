# Generate Genesis Block

Prosedur membuat genesis block baru CEN beserta output premine. Belum
dijalankan; ini rencana kerja Fase 2.

## 1. Tentukan parameter genesis

Untuk setiap network (main, testnet, simnet, devnet):

- `timestamp`: waktu mulai (milidetik sejak epoch).
- `bits`: difficulty awal. Untuk jaringan baru pakai nilai longgar seperti
  devnet (`0x1e21bc1c`) supaya blok bisa ditambang.
- `nonce`: mulai dari 0, dihitung kemudian.
- `daa_score`: 0.
- `coinbase_payload`: payload bebas. Struktur bawaan: blue score (8 byte),
  subsidy (8 byte), script version (2 byte), panjang script (varint), lalu
  script public key. Isi teks bebas di akhir payload (contoh upstream:
  `kaspa-devnet`).
- Output premine: 22,200,000 COIN ke alamat development, 12,300,000 COIN ke
  alamat owner.

## 2. Tambahkan output premine

Fungsi `build_genesis_transactions` di
`node/consensus/core/src/config/genesis.rs` sekarang membuat coinbase tanpa
output. Ubah agar coinbase genesis memuat dua output premine ke alamat dev dan
owner. Tanpa perubahan ini, premine tidak akan pernah masuk UTXO set.

## 3. Regenerasi hash

Genesis Kaspa punya test bawaan `test_genesis_hashes` di file yang sama. Test
ini membangun blok dari konstanta `GenesisBlock`, menghitung ulang merkle root
dan hash blok, lalu panic dengan nilai hasil hitungan kalau tidak cocok.

Alur kerjanya:

1. Set `hash` sementara ke nol, isi field lain (timestamp, bits, nonce,
   payload, outputs).
2. Jalankan:

   ```bash
   cd node
   cargo test --package kaspa-consensus-core genesis
   ```

3. Baca hash merkle root dan hash blok dari pesan panic.
4. Update `hash_merkle_root` dan `hash` dengan nilai tersebut.
5. Ulangi sampai test lolos.

`utxo_commitment` harus konsisten dengan UTXO set hasil genesis; kalau salah,
node akan menolak genesis saat start dan melaporkan nilai yang diharapkan.
Update konstanta dengan nilai tersebut.

## 4. Cari nonce (opsional, untuk mainnet)

Kalau difficulty genesis mau realistis, cari nonce dengan hash block di bawah
target `bits`. Cara termudah: tulis test Rust sementara yang loop nonce dan
print hash yang memenuhi target, lalu pasang nonce itu di konstanta.

## 5. Verifikasi

```bash
cargo test --package kaspa-consensus-core genesis
```

Lalu jalankan node di simnet dan pastikan blok 0 terbaca:

```bash
./target/release/kaspad --simnet --utxoindex
cargo run --release --bin kaspa -- Simnet   # lalu: get-block 0
```

## Catatan

- Jangan pakai private key atau seed asli untuk alamat premine di repo.
  Alamat publik saja yang dicantumkan.
- Genesis berbeda per network (main/testnet/simnet/devnet masing-masing
  konstanta sendiri).
- Setelah genesis berubah, seluruh node harus mulai dari blok 0; tidak ada
  kompatibilitas dengan jaringan lain.

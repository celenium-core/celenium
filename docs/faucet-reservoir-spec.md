# Spesifikasi Faucet Reservoir Celenium (CEN)

Status: **RANCANGAN TERKUNCI** (13 Nov 2026) — belum diimplementasi di kode.
Dokumen ini adalah spesifikasi desain, bukan perubahan yang sudah jalan.

## 1. Tujuan

Memberi *rewards partisipasi* kecil kepada miner ponsel (yang secara praktis
tidak menambang di kHeavyHash) agar keterlibatan tetap hidup, tanpa:

- mengubah emisi / konsensus (tanpa hardfork, tanpa regenesis),
- memotong premine,
- menyebutnya "mining" atau menjanjikan penghasilan.

Reward didanai dari **potongan reward blok (kebijakan pool/miner)**, disangga
**reservoir dinamis** yang menyeimbangkan diri, bukan dari premine.

## 2. Model reservoir

- **Reservoir (wallet floor)** menampung CEN untuk membayar klaim harian.
- **Masuk:** (1) reward bootstrap (coinbase miner inisial → reservoir),
  (2) potongan reward blok (variabel).
- **Keluar:** (1) klaim harian, (2) kelebihan → dikembalikan ke miner.

Aturan kontrol (feedback loop):

- reservoir < 80% target → potongan reward blok **aktif** (hingga 1,5%).
- reservoir > 120% target → potongan **0%** (miner menerima 100% reward).

Dengan ini reservoir menuju target secara otomatis, tanpa angka tetap yang
harus diprediksi, dan tanpa CEN yang menganggur.

## 3. Parameter (default — dapat diubah operator, bukan konsensus)

| Parameter | Default | Keterangan |
|---|---|---|
| Reward dasar | 1000 pradha (0,00001 CEN) / hari / perangkat | dibayar saat klaim lolos |
| Cap anggaran harian | 250 CEN | batas total klaim per hari |
| Target reservoir | 7500 CEN (= 30 hari × cap) | level ideal cadangan |
| Hysteresis | 80% / 120% target (6000 / 9000 CEN) | cegah osilasi potongan |
| Potongan reward blok | 0–1,5% variabel | aktif hanya saat di bawah target |
| Klaim | 1× / hari / perangkat | — |
| Ambang penarikan | 100000 pradha (0,001 CEN) | di atas fee transfer (~25000 pradha) |
| Anti-sybil | atestasi perangkat + 1 klaim/hari | KYC opsional untuk tarik besar |

Semua angka di atas merupakan default yang bisa digeser tanpa perubahan
konsensus (murni kebijakan operator pool/miner).

## 4. Alur

1. **Bootstrap (masa awal):** operator menjalankan miner inisial (CPU/GPU
   murah, di mesin terpisah dari seed node) dengan coinbase diarahkan ke
   wallet reservoir. Blok awal terbit → reservoir terisi.
2. **Top-up (berkelanjutan):** tiap blok, potongan 0–1,5% dialirkan ke
   reservoir bila cadangan di bawah target; 0% bila di atas ambang atas.
3. **Klaim harian:** perangkat membuktikan identitas (Keystore) → server
   mengecek "sudah klaim hari ini?" → kredit 1000 pradha ke ledger off-chain.
4. **Cap:** jika total klaim hari itu × 1000 pradha > 250 CEN, tiap klaim
   dibagi rata sesuai cap (reward per klaim menyusut).
5. **Overflow:** reservoir di atas ambang atas → potongan reward blok = 0%,
   kelebihan otomatis tetap di tangan miner.

## 5. Anti-sybil

- Atestasi perangkat (tanda tangan Keystore atas challenge server) — sulit
  dipalsukan massal.
- 1 perangkat = 1 klaim/hari.
- Rate-limit per IP; aturan 1 IP = 1 sesi mining (sudah di whitepaper).
- KYC opsional untuk penarikan besar.

Catatan: reward sekecil 0,00001 CEN/hari membuat serangan sybil tidak ekonomis
(drain per akun palsu ~0,00365 CEN/tahun), sehingga anti-sybil cukup ringan.

## 6. Settlement

- Klaim menumpuk di **ledger off-chain** (SQLite/Postgres) di server faucet.
- Saat saldo mencapai ambang (100000 pradha), pengguna dapat **menarik**:
  server menandatangani satu transaksi on-chain dari wallet reservoir ke alamat
  pengguna, membayar fee dari reservoir.
- Ambang dibuat di atas fee transfer agar tidak banjir dust/UTXO.

## 7. Arsitektur teknis

Komponen (semua di sisi pool/miner, TANPA ubah konsensus):

- **Stratum bridge** (sudah ada): server stratum untuk worker eksternal.
- **Backend pool + payout engine** (perlu dibangun): mencatat share per worker,
  menghitung reward PPLNS, dan menerapkan potongan reservoir + distribusi.
- **Controller reservoir** (perlu dibangun): memantau saldo reservoir via
  node RPC, menetapkan potongan (0 atau 1,5%) mengikuti hysteresis.
- **Ledger klaim + API faucet** (perlu dibangun): route
  `POST /api/faucet/claim`, `GET /api/faucet/balance`, `POST /api/faucet/withdraw`.
- **Treasury signer**: walet reservoir (multisig 2-of-3; hot wallet isi kecil).
- **Aplikasi miner** (sudah ada, perlu UI baru): tab "Klaim harian" + saldo
  rewards, sebagai ganti counter "share diterima" yang kini sudah jujur.

## 8. Pelabelan & kejujuran

- Sebut "Rewards partisipasi" / "Klaim harian", **bukan** "mining".
- Jangan menjanjikan profit; angka reward kecil (0,00001 CEN/hari) adalah sinyal
  keterlibatan, bukan penghasilan.
- Konsisten dengan whitepaper §5.1 dan baris "sarana partisipasi, bukan profit".

## 9. Status implementasi

| Item | Status |
|---|---|
| Spesifikasi (dokumen ini) | selesai |
| Revisi whitepaper §5.1 | selesai |
| Config pool-mode + faucet | selesai |
| Coinbase routing → alamat pool | selesai |
| Controller reservoir (hysteresis) | selesai (4 unit test) |
| PPLNS tracker | selesai (6 unit test) |
| Payout compute (subsidi → cut → PPLNS → log) | selesai |
| Payout execution (kirim tx ke worker) | belum — butuh kunci pool + ledger akumulasi |
| Ledger + API faucet (claim/balance/withdraw) | belum |
| UI "Klaim harian" di app miner | belum |
| Bootstrap miner (operator) | belum (bergantung deployment) |

## 10. Peta implementasi kode (temuan 13 Nov 2026)

Audit bridge (`bridge/src/`) menunjukkan bridge saat ini berjalan **SOLO**, bukan
pool: coinbase ditanam ke alamat worker sendiri.

- `share_handler.rs::handle_submit` — titik validasi share → blok ditemukan →
  `celenium_api.submit_block(...)`. Coinbase worker: `get_block_template(wallet_addr)`
  memakai `ctx.wallet_addr`, sehingga reward blok masuk ke alamat si penemu.
  Mode pool harus mengganti alamat ini ke alamat pool.
- Fondasi bobot share **sudah ada**: `WorkStats.shares_diff` mengakumulasi
  `hash_value` (nilai difficulty-weighted) tiap share valid — persis "bobot =
  difficulty" untuk PPLNS. `shares_found` = hitungan share.
- `mining_state.rs` menyimpan `Job` (template blok) per koneksi.

Perubahan yang diperlukan untuk pool + faucet:

1. Config "pool mode" (pool address, faucet address, cut 0–1,5%, target,
   hysteresis, ambang penarikan) di `app_config.rs` + `cli.rs`.
2. Arahkan coinbase ke alamat pool (ubah pemanggilan `get_block_template`).
3. Jendela share per worker (PPLNS) — perluas `WorkStats`.
4. Engine payout baru: blok terkonfirmasi → hitung porsi tiap worker + kirim
   potongan faucet bila reservoir di bawah target (controller baca saldo
   reservoir via node RPC `get_balances_by_addresses`).
5. Ledger klaim + route `POST /api/faucet/claim`, `GET /api/faucet/balance`,
   `POST /api/faucet/withdraw` (di `prom.rs` atau modul HTTP baru).
6. UI Flutter "Klaim harian".
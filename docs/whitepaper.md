# Celenium (CEN) — Whitepaper

**Versi 1.0 · 2 November 2026**

Celenium (CEN) adalah cryptocurrency Proof-of-Work berbasis DAG yang di-fork
dari rusty-kaspa (implementasi Rust dari Kaspa). Celenium mempertahankan
konsensus GHOSTDAG, throughput 10 blok per detik sejak genesis, dan algoritma
PoW kHeavyHash, lalu menata ulang ekonomi jaringan: 50% supply ditambang secara
terbuka dan 50% dialokasikan sebagai premine terdokumentasi dengan jadwal
vesting bulanan yang bisa diverifikasi publik.

---

## 1. Ringkasan

| Item | Nilai |
|---|---|
| Nama | Celenium |
| Ticker | CEN |
| Total supply | 69.000.000 CEN |
| Mining (publik) | 34.500.000 CEN (50%) |
| Premine | 34.500.000 CEN (50%) |
| Block rate | 10 blok/detik (100 ms) sejak genesis |
| Halving | setiap 4 tahun, 17 era, horizon ±68 tahun |
| Konsensus | GHOSTDAG (k = 124, max 16 parent) |
| PoW | kHeavyHash |
| Prefix alamat | `cen69` (mainnet), `cen69test`, `cen69sim`, `cen69dev` |
| Node software | `celeniumd` (Rust), lisensi open source |

Satu CEN = 100.000.000 pradha.

## 2. Latar belakang

Kaspa membuktikan bahwa DAG blockDAG GHOSTDAG mampu memberi throughput tinggi
tanpa mengorbankan desentralisasi. Celenium mengambil fondasi teknis itu
sebagai titik awal, lalu mengubah tiga hal: identitas jaringan (nama, prefix,
port, genesis), model distribusi (50% premine terjadwal), dan kurva emisi
(tabel subsidi 817 entri, emisi total mining 34.5 juta CEN dalam ±68 tahun).

Premine 50% adalah deviasi sadar dari model fair-launch Kaspa. Dokumen ini
serta `docs/vesting-schedule.md` dan `docs/tokenomics.md` dipublikasikan
sebagai komitmen transparansi: angka premine, alokasi, dan jadwal unlock bisa
dicek siapa pun, baik dari source code maupun dari UTXO genesis.

## 3. Tokenomics

### 3.1 Distribusi

- **Mining publik: 34.500.000 CEN (50%)** ditambang sepanjang ±68 tahun.
- **Premine: 34.500.000 CEN (50%)**
  - Pengembangan proyek: 22.200.000 CEN (32,17% dari total supply)
  - Pemilik koin: 12.300.000 CEN (17,83% dari total supply)
  - Kedua alokasi ditulis sebagai output coinbase di genesis block dengan
    script P2PK atas public key yang dipublikasikan.

### 3.2 Emisi mining

- Tabel subsidi: 17 era × 48 bulan (halving tiap 4 tahun), total 817 entri.
- Reward era 0: 1.366.560 pradha per blok (0,0136656 CEN), setara
  0,13665592 CEN per detik pada 10 BPS.
- Reward per blok = `tabel[bulan] / 10` dengan pembulatan ke atas.
- Total emisi mining: 34.500.000 CEN (dust pembulatan per blok < 1 CEN).
- Jadwal lengkap per era ada di `docs/emission-schedule.md`;
  `scripts/calculate-emission.py` dapat dijalankan siapa pun untuk verifikasi.

### 3.3 Vesting premine

Unlock bulanan tetap, berlaku proporsional untuk kedua alokasi:

- Pengembangan: 76.590 CEN per bulan (0,345% dari alokasi dev)
- Pemilik: 42.435 CEN per bulan (0,345% dari alokasi owner)
- Total 119.025 CEN per bulan, habis pada bulan ke-290 (±24,2 tahun).

Jadwal lengkap: `docs/vesting-schedule.md`. Status implementasi saat ini:
unlock dijalankan sebagai jadwal off-chain (multisig + publikasi jadwal dan
alamat), karena bahasa script Kaspa tidak memiliki opcode time-lock. Pilihan
on-chain (multisig terverifikasi atau timelock) dievaluasi sebelum mainnet
dan diprioritaskan sebagai syarat kepercayaan komunitas.

## 4. Spesifikasi teknis

### 4.1 Konsensus

- GHOSTDAG: `ghostdag_k = 124`, maksimal 16 parent per blok, mergeset limit
  dan bounded merge depth mengikuti parameter BlockrateParams untuk 10 BPS.
- DAA (difficulty adjustment) mengikuti mekanisme Kaspa.
- Storage mass (KIP-0009) dipertahankan.

### 4.2 Jaringan

| Jaringan | P2P | gRPC | wRPC Borsh | wRPC JSON |
|---|---|---|---|---|
| mainnet | 26111 | 26110 | 27110 | 28110 |
| testnet-10 | 26211 | 26210 | 27210 | 28210 |
| simnet | 26511 | 26510 | 27510 | 28510 |

gRPC default bind ke loopback; wRPC dimatikan secara default; RPC berisiko
(hardcode state) tertutup flag `--unsaferpc`. Untuk VPS, buka hanya port P2P.

### 4.3 Genesis

| Jaringan | Genesis hash |
|---|---|
| mainnet | `55f7735865ee1cd2ddc5140c174fe72b0567c1c0fb11907cca5a196098a1e3de` |
| testnet-10 | `f3d99b3cf22720cc39d7e96a6cd2b0996d4d5a1f23e36b57926517a04205db89` |
| simnet | `f3f0e6037dd16a1c5ae44150efac2eef4acf1ff3788d194cb5c6f1b8d94cd0aa` |
| devnet | `09428456ea1b6c35b24a76e85e5d41e3ed7ec3b4f5a4286a24b6f73f918c6afb` |

Payload coinbase genesis membawa identitas jaringan ("Celenium mainnet
genesis", "celenium-testnet", dst). Timestamp genesis adalah parameter launch:
di menit peluncuran mainnet, genesis diregenerasi dengan timestamp T lalu
node + miner pertama dinyalakan agar blok pertama terbit dalam ~11 menit
(prosedur `scripts/regenesis.sh --now`).

Catatan teknis: validasi PoW tidak berlaku untuk genesis block (paritas dengan
rusty-kaspa). Nonce genesis segar di-mine untuk jaringan yang targetnya
realistis (simnet/testnet/devnet); mainnet mempertahankan nonce warisan.

### 4.4 Alamat

Alamat memakai bech32 varian Kaspa (checksum 8 karakter, polymod 40-bit) dengan
human-readable part `cen69*`. Contoh mainnet: `cen69:qq...`. Perubahan prefix
mengubah checksum, sehingga seluruh test vector alamat di kode di-regenerasi
dan lolos `cargo test`.

### 4.5 Kompatibilitas

Pesan wire gRPC/P2P di-rename dari `Kaspad*` menjadi `Celeniumd*`. Klien dari
ekosistem Kaspa tidak kompatibel secara wire dan harus memakai SDK Celenium
(wasm/Rust). Perubahan ini tercatat di CHANGELOG sebagai breaking change.

## 5. Mining

Mining solo berjalan melalui `celenium-stratum-bridge` dengan CPU miner
internal: bridge mengambil template dari node penuh, menambang, dan reward
blok langsung masuk ke coinbase atas alamat wallet yang dipasang.

- Desktop: `stratum-bridge --internal-cpu-miner --internal-cpu-miner-address
  cen69sim:... --internal-cpu-miner-threads N --web-dashboard-port :3030`
- Dashboard web menampilkan hashrate, share, dan status koneksi node.
- Pool: bridge dapat berjalan sebagai stratum server untuk miner eksternal
  (Android/pool client), dengan penerapan aturan: reward sesi Android dibatasi
  maksimal 10% dari reward desktop pada hashrate yang sama, dan satu alamat IP
  hanya boleh memegang satu sesi mining Android aktif.

Batasan yang diakui terbuka: CPU (desktop maupun ponsel) tidak kompetitif
melawan ASIC/GPU di mainnet. Mining ponsel diposisikan sebagai pool client
ringan dan sarana partisipasi, bukan sumber profit.

## 6. Wallet dan ekosistem

- `celenium`: wallet CLI (buat alamat, kirim/terima, query saldo).
- Aplikasi miner Flutter untuk desktop dan Android (kontrol mining + dashboard
  koneksi ke bridge), distribusi APK arm64/armv7/x86_64 dan bundle Linux.
- `celenium-web`: situs distribusi miner dan dokumentasi.
- SDK wasm (`celenium-wasm`) untuk integrasi web.
- Rencana: block explorer, wallet mobile rilis publik, dokumentasi situs.

## 7. Keamanan

- Audit internal menyeluruh (2 November 2026): konsensus, genesis, API, deploy,
  dan pemindaian rahasia di seluruh riwayat git (bersih; premine hanya berisi
  public key, private key disimpan offline).
- Test: verifikasi hash genesis 4 jaringan, 302 test vector alamat, E2E
  2-node (peering, genesis agreement, utxoindex) 14/14 lulus, DNS seeding
  6/6 lulus, nol warning compiler.
- Praktik keamanan: tidak ada material kunci di repository; guardrail
  `.gitignore` untuk artefak kunci; RPC sensitif tertutup secara default.
- Audit eksternal dan program bug bounty disarankan sebelum mainnet.

## 8. Roadmap

1. Testnet publik: minimal 3 node di lokasi berbeda, sync + mining stabil 1 minggu.
2. Verifikasi mining end-to-end: reward blok masuk ke alamat wallet sesuai tabel emisi.
3. Situs + block explorer + dokumentasi publik.
4. Wallet mobile rilis; miner Android rilis publik.
5. Keputusan vesting on-chain (multisig terverifikasi atau timelock).
6. Audit eksternal + bug bounty.
7. Mainnet launch (regenesis timestamp T, blok pertama < 11 menit, DNS seeder publik).
8. Listing exchange dievaluasi setelah ekosistem matang.

## 9. Risiko dan batasan

- Premine 50% menyimpang dari fair-launch Kaspa; kepercayaan bergantung pada
  transparansi jadwal vesting dan kepatuhan terhadapnya.
- CPU mining tidak kompetitif di mainnet; tidak ada janji profit.
- Emisi dan vesting bergantung pada kejujuran pemilik alokasi selama lock masih
  off-chain.
- Perangkat lunak disediakan "sebagaimana adanya". Dokumen ini bukan nasihat
  finansial. Keterlibatan dalam jaringan kripto memiliki risiko kehilangan aset.

## 10. Referensi

- Repository node: source Celenium (fork rusty-kaspa, basis upstream commit
  `35efec7`).
- `docs/tokenomics.md`, `docs/emission-schedule.md`, `docs/vesting-schedule.md`,
  `docs/network-params.md`, `docs/how-to-run.md`, `docs/launch-runbook.md`.
- `scripts/calculate-emission.py`: kalkulator emisi dan vesting yang dapat
  dijalankan ulang siapa pun.
- Upstream: Kaspa / rusty-kaspa (GHOSTDAG, kHeavyHash). Terima kasih kepada
  komunitas Kaspa atas fondasi open source-nya.

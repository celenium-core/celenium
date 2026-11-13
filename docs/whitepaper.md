# Celenium (CEN) — Whitepaper

**Versi 1.1 · 11 November 2026**

Celenium (CEN) adalah cryptocurrency Proof-of-Work berbasis DAG yang di-fork
dari rusty-kaspa (implementasi Rust dari Kaspa). Celenium mempertahankan
konsensus GHOSTDAG, throughput 10 blok per detik sejak genesis, dan algoritma
PoW kHeavyHash, lalu menata ulang ekonomi jaringan: 76.09% supply ditambang
secara terbuka dan 23.91% dialokasikan sebagai premine terdokumentasi yang
terkunci waktu secara **on-chain** (CLTV + multisig 2-of-3) dengan jadwal unlock
bulanan yang bisa diverifikasi publik.

---

## 1. Ringkasan

| Item | Nilai |
|---|---|
| Nama | Celenium |
| Ticker | CEN |
| Total supply | 69.000.000 CEN |
| Mining (publik) | 52.500.000 CEN (76.09%) |
| Premine | 16.500.000 CEN (23.91%) |
| Block rate | 10 blok/detik (100 ms) sejak genesis |
| Halving | setiap 4 tahun, 17 era, horizon ±68 tahun |
| Konsensus | GHOSTDAG (k = 124, max 16 parent) |
| PoW | kHeavyHash |
| Prefix alamat | `cen69` (mainnet), `cen69test`, `cen69sim`, `cen69dev` |
| Node software | `celeniumd` (Rust), lisensi open source |

Satu CEN = 100.000.000 pradha.

## 2. Latar belakang

Kaspa membuktikan bahwa blockDAG GHOSTDAG mampu memberi throughput tinggi tanpa
mengorbankan desentralisasi. Celenium mengambil fondasi teknis itu sebagai
titik awal, lalu mengubah tiga hal: identitas jaringan (nama, prefix, port,
genesis), model distribusi (23.91% premine terkunci CLTV), dan kurva emisi
(tabel subsidi 817 entri, emisi total mining 52.5 juta CEN dalam ±68 tahun).

Premine 23.91% adalah deviasi sadar dari model fair-launch Kaspa. Dokumen ini
serta `docs/premine-cltv-schedule.md`, `docs/vesting-schedule.md`, dan
`docs/tokenomics.md` dipublikasikan sebagai komitmen transparansi: angka
premine, kunci publik, alamat wallet, dan jadwal unlock bisa dicek siapa pun,
baik dari source code maupun dari UTXO genesis yang terkunci CLTV.

## 3. Tokenomics

### 3.1 Distribusi

- **Mining publik: 52.500.000 CEN (76.09%)** ditambang sepanjang ±68 tahun.
- **Premine: 16.500.000 CEN (23.91%)**
  - Pengembangan proyek: 9.600.000 CEN (13.91% dari total supply)
  - Pemilik koin: 6.900.000 CEN (10.00% dari total supply)
  - Kedua alokasi ditulis di genesis sebagai **264 output coinbase** (132 per
    vault), masing-masing P2SH dengan redeem script CLTV + multisig 2-of-3.

### 3.2 Emisi mining

- Tabel subsidi: 17 era × 48 bulan (halving tiap 4 tahun), total 817 entri.
- Reward era 0: 2.079.547 pradha per blok (0,02079547 CEN), setara
  0,20795466 CEN per detik pada 10 BPS.
- Reward per blok = `tabel[bulan] / 10` dengan pembulatan ke atas.
- Total emisi mining: 52.500.000 CEN (dust pembulatan 0.32 CEN di bawah target
  karena satuan waktu terkecil 1 detik).
- Jadwal lengkap per era ada di `docs/emission-schedule.md`;
  `scripts/calculate-emission.py` dapat dijalankan siapa pun untuk verifikasi.

### 3.3 Vesting premine (CLTV on-chain)

Premine dibuka otomatis **setiap tanggal 5 bulan** (00:00 UTC), linear, tanpa
cliff, selama 132 bulan (11 tahun) mulai **5 Desember 2026**.

- Pengembangan: 72.727,27 CEN per bulan (9.600.000 / 132)
- Pemilik: 52.272,73 CEN per bulan (6.900.000 / 132)
- Total 125.000 CEN per bulan, habis pada **5 November 2037**.

Kunci waktu **di-enforce di layer konsensus**: setiap output genesis memakai
redeem script `<locktime_ms> OP_CHECKLOCKTIMEVERIFY OP_DROP <2> <pubkey_x3>
<3> OP_CHECKMULTISIG`. Sebelum tanggal 5 di bulannya, CLTV menolak spend;
setelahnya tetap wajib **2 dari 3 tanda tangan**. Kunci publik dan alamat
dipublikasikan; private key disimpan terenkripsi di luar repository.

Jadwal lengkap 264 baris + kunci publik + alamat: `docs/premine-cltv-schedule.md`.

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

Payload coinbase genesis membawa identitas jaringan ("Celenium mainnet
genesis", "celenium-testnet", dst) plus **264 output premine** terkunci CLTV +
multisig 2-of-3. Konstanta genesis (hash, merkle root, utxo_commitment, nonce
PoW) dihitung dari payload ini dan diverifikasi oleh
`cargo test test_genesis_hashes` (4 jaringan lulus).

Timestamp genesis adalah parameter launch: di menit peluncuran mainnet, genesis
diregenerasi dengan timestamp T lalu node + miner pertama dinyalakan agar blok
pertama terbit dalam ~11 menit (prosedur `scripts/regenesis.sh --now`). Karena
itu hash genesis mainnet **final ditetapkan saat launch**, bukan nilai statis
di repositori.

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
- Pool: bridge berjalan sebagai stratum server untuk miner eksternal (desktop
  dan Android). Reward blok dibagi ke worker proporsional terhadap bobot share
  (PPLNS); sesi Android tunduk pada aturan 1 IP = 1 sesi mining aktif.

### 5.1 Faucet partisipasi (reservoir dinamis)

CPU (desktop maupun ponsel) tidak kompetitif melawan ASIC/GPU di mainnet,
sehingga mining ponsel tidak dapat menjadi sumber penghasilan yang berarti.
Untuk menjaga partisipasi tetap hidup, Celenium menyalurkan "rewards
partisipasi" — pembayaran harian kecil (dasar 1000 pradha/hari/perangkat ≈
0,00001 CEN) yang didanai dari **potongan reward blok**, bukan dari premine,
lewat reservoir dinamis yang menyeimbangkan diri:

- Bootstrap: miner inisial operator mengarahkan coinbase ke wallet reservoir,
  sehingga reservoir terisi sejak blok pertama di masa awal jaringan.
- Potongan variabel 0–1,5%: sebagian reward blok dialirkan ke reservoir hanya
  saat cadangan di bawah target (default 30 hari pembayaran = 7.500 CEN); saat
  cadangan melebihi ambang atas, potongan menjadi 0% sehingga kelebihan
  otomatis kembali ke miner (nol pemborosan).
- Hysteresis 80%–120% target mencegah potongan berosilasi antar blok.
- Klaim 1×/hari/perangkat dengan cap anggaran 250 CEN/hari yang dipecah rata
  bila jumlah klaim melebihi cap; anti-sybil via atestasi perangkat + 1
  klaim/hari.
- Settlement: saldo klaim tercatat di ledger off-chain dan ditarik ke on-chain
  saat mencapai ambang minimum.

Seluruh mekanisme adalah kebijakan pool/miner, bukan konsensus: emisi tidak
berubah, tidak butuh hardfork, dan premine tetap utuh untuk peruntukannya.
Spesifikasi teknis lengkap: `docs/faucet-reservoir-spec.md`.

## 6. Wallet dan ekosistem

- `celenium`: wallet CLI (buat alamat, kirim/terima, query saldo).
- Aplikasi miner Flutter untuk desktop dan Android (kontrol mining + dashboard
  koneksi ke bridge), distribusi APK arm64/armv7/x86_64 dan bundle Linux.
- `celenium-web`: situs distribusi miner dan dokumentasi.
- SDK wasm (`celenium-wasm`) untuk integrasi web.
- Rencana: block explorer, wallet mobile rilis publik, dokumentasi situs.

## 7. Keamanan

- Audit internal menyeluruh (2 November 2026) + pembaruan 11 November 2026:
  konsensus, genesis, API, deploy, dan pemindaian rahasia di seluruh riwayat
  git (bersih; premine/lock hanya memuat public key, private key disimpan
  terenkripsi di luar repository).
- Test: verifikasi hash genesis 4 jaringan, 302 test vector alamat, E2E
  2-node (peering, genesis agreement, utxoindex), DNS seeding, dan verifikasi
  tabel subsidi & total 264 output CLTV premine.
- Praktik keamanan: tidak ada material kunci di repository; guardrail
  `.gitignore` untuk artefak kunci; RPC sensitif tertutup secara default.
- Audit eksternal dan program bug bounty disarankan sebelum mainnet.

## 8. Roadmap

1. Testnet publik: minimal 3 node di lokasi berbeda, sync + mining stabil 1 minggu.
2. Verifikasi mining end-to-end: reward blok masuk ke alamat wallet sesuai tabel emisi.
3. Situs + block explorer + dokumentasi publik.
4. Wallet mobile rilis; miner Android rilis publik.
5. ~~Keputusan vesting on-chain (multisig terverifikasi atau timelock).~~ **Selesai**: premine kini terkunci CLTV + multisig 2-of-3 di genesis.
6. Audit eksternal + bug bounty.
7. Mainnet launch (regenesis timestamp T, blok pertama < 11 menit, DNS seeder publik).
8. Listing exchange dievaluasi setelah ekosistem matang.

## 9. Risiko dan batasan

- Premine 23.91% menyimpang dari fair-launch Kaspa. Berbeda dari rilis awal,
  jaminan waktunya kini **on-chain** (CLTV), sehingga risiko ketidakpatuhan
  operator dihilangkan; yang tersisa adalah risiko konsentrasi 2-of-3 keyholder.
- CPU mining tidak kompetitif di mainnet; tidak ada janji profit.
- Perangkat lunak disediakan "sebagaimana adanya". Dokumen ini bukan nasihat
  finansial. Keterlibatan dalam jaringan kripto memiliki risiko kehilangan aset.

## 10. Referensi

- Repository node: source Celenium (fork rusty-kaspa, basis upstream commit
  `35efec7`).
- `docs/tokenomics.md`, `docs/emission-schedule.md`, `docs/vesting-schedule.md`,
  `docs/premine-cltv-schedule.md`, `docs/network-params.md`, `docs/how-to-run.md`,
  `docs/launch-runbook.md`.
- `scripts/calculate-emission.py`: kalkulator emisi yang dapat dijalankan ulang
  siapa pun.
- `crypto/txscript/examples/gen_premine_cltv.rs`: generator/verifikasi script
  CLTV + multisig premine.
- Upstream: Kaspa / rusty-kaspa (GHOSTDAG, kHeavyHash). Terima kasih kepada
  komunitas Kaspa atas fondasi open source-nya.

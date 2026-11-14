# Celenium (CEN) Tokenomics

Dokumen ini menjelaskan distribusi supply, mekanisme unlock premine, dan
ringkasan emisi Celenium. Semua angka dihitung dan diverifikasi oleh
`scripts/calculate-emission.py` serta generator `gen_premine_cltv`.

## Supply

| Alokasi | Jumlah (CEN) | Porsi | Status |
|---|---|---|---|
| Mineable (block subsidy) | 52,500,000 | 76.09% | Terdistribusi via mining |
| Premine pengembangan | 9,600,000 | 13.91% | CLTV + multisig 2-of-3, unlock tiap tanggal 5 |
| Premine pemilik coin | 6,900,000 | 10.00% | CLTV + multisig 2-of-3, unlock tiap tanggal 5 |
| **Total** | **69,000,000** | 100% | |

Satuan terkecil: 1 CEN = 100,000,000 base units (disebut **pradha**, mengikuti
konvensi Kaspa).

## Premine dan unlock (CLTV on-chain)

Premine total 16,500,000 CEN dicatat di genesis sebagai **264 output**, masing-
masing P2SH dengan script redeem:

```
<locktime_ms> OP_CHECKLOCKTIMEVERIFY OP_DROP <2> <pubkey_x3> <3> OP_CHECKMULTISIG
```

Artinya setiap output terkunci waktu secara **on-chain** (bukan komitmen
off-chain): sebelum tanggal 5 di bulannya, CLTV menolak spend; setelahnya tetap
wajib **2 dari 3 tanda tangan**. Dua vault terpisah:

- **Dev vault** — 9,600,000 CEN, 2-of-3 (K1/K2/K3)
- **Owner vault** — 6,900,000 CEN, 2-of-3 (K4/K5/K6)

Jadwal unlock **linear, tanpa cliff**, 132 bulan (11 tahun), dimulai
**5 Desember 2026** dan berakhir **5 November 2037**:

- Pengembangan: 9,600,000 / 132 = **72,727.27 CEN/bulan**
  (7,272,727,272,727 pradha; sisa 36 pradha di tranche terakhir).
- Pemilik: 6,900,000 / 132 = **52,272.73 CEN/bulan**
  (5,227,272,727,272 pradha; sisa menyesuaikan di tranche terakhir).
- Total unlock **125,000 CEN/bulan**.

Tabel lengkap 264 baris (tanggal, timestamp, jumlah, alamat) ada di
`docs/premine-cltv-schedule.md`. Kunci publik (bukan private) terpublikasi di
dokumen itu dan di whitepaper; private key disimpan terenkripsi di luar repo.

## Mekanisme lock

- `consensus/core/src/config/genesis.rs` memuat `PREMINE_OUTPUTS` (264 tuple
  `(amount_pradha, p2sh_script)`) yang disisipkan sebagai output coinbase
  genesis.
- `crypto/txscript/examples/gen_premine_cltv.rs` adalah generatornya: membangun
  script CLTV + multisig per tranche, menghitung address P2SH, memverifikasi
  jumlah total, lalu menulis konstanta Rust dan jadwal markdown. Tidak menyentuh
  private key.
- CLTV memakai opcode `OpCheckLockTimeVerify (0xb0)`; lock time di atas
  `LOCK_TIME_THRESHOLD` (500,000,000,000) diinterpretasi sebagai timestamp
  milidetik, sehingga `1796428800000` = 5 Des 2026 00:00 UTC.

## Emisi

- Rate awal: **0.20795466 CEN/detik**, setara block reward awal
  **0.02079547 CEN/blok** (2,079,547 pradha) pada 10 BPS.
- Halving setiap 4 tahun (48 bulan), 17 era, total emisi 52,500,000 CEN
  (dust pembulatan 0.32 CEN di bawah target karena satuan terkecil 1 detik).
- Mining selesai hingga halving selesai (816 bulan).

Detail per era ada di `emission-schedule.md`. Tabel siap pakai untuk kode ada
di `scripts/data/subsidy_table.rs`.

## Deviasi dari Kaspa (wajib dicatat)

Kaspa asli adalah fair-launch tanpa premine. Celenium sengaja menambahkan
premine 23.91% dengan kunci waktu on-chain (CLTV + multisig 2-of-3). Ini
perubahan fundamental terhadap model ekonomi Kaspa dan sudah tercantum di
spesifikasi serta dokumen ini sebagai keputusan desain, bukan kesalahan.

## Status implementasi

Angka di dokumen ini sudah diverifikasi oleh script. Perubahan kode node
sudah diterapkan di working copy lokal (`~/celenium-build`): tabel subsidy,
parameter emisi, prefix, port, 264 output premine genesis terkunci CLTV, dan
regenerasi konstanta genesis (hash/merkle/commitment + nonce PoW). Detail di
`network-params.md`.
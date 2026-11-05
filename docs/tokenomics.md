# Celenium (CEN) Tokenomics

Dokumen ini menjelaskan distribusi supply, aturan unlock premine, dan ringkasan
emisi Celenium. Semua angka dihitung dan diverifikasi oleh
`scripts/calculate-emission.py`.

## Supply

| Alokasi | Jumlah (CEN) | Porsi | Status |
|---|---|---|---|
| Mineable (block subsidy) | 34,500,000 | 50% | Terdistribusi via mining |
| Premine pengembangan | 22,200,000 | 32.17% | Terkunci, unlock bertahap |
| Premine pemilik coin | 12,300,000 | 17.83% | Terkunci, unlock bertahap |
| **Total** | **69,000,000** | 100% | |

Satuan terkecil: 1 CEN = 100,000,000 base units (disebut pradha, mengikuti
konvensi Kaspa).

## Premine dan vesting

Seluruh premine (34,500,000 CEN) dicatat di genesis ke dua alamat khusus (dev
dan owner). Aturan unlock yang dipakai (Model A):

- Setiap alokasi unlock **0.345% dari jumlah aslinya per bulan**.
- Pengembangan: 22,200,000 x 0.345% = **76,590 CEN/bulan**.
- Pemilik: 12,300,000 x 0.345% = **42,435 CEN/bulan**.
- Total unlock **119,025 CEN/bulan**.
- Kedua alokasi habis pada **bulan ke-290** (24.17 tahun). Bulan terakhir
  membawa sisa agar total pas (dev 65,490 CEN, owner 36,285 CEN).

Model ini dipilih karena kalimat spesifikasi "hitung jadwal unlock sampai habis"
mengharuskan jadwal yang benar-benar selesai. Tabel bulanan lengkap ada di
`vesting-schedule.md` dan `scripts/data/vesting_monthly.csv`.

> Catatan: teks spesifikasi juga memuat bacaan alternatif, "0.69% dari total
> coin khusus yang masih terkunci" per bulan. Model itu peluruhan geometri dan
> tidak pernah mencapai nol (setelah 290 bulan masih sisa 4,632,230 CEN,
> 13.4%), sehingga tidak bisa "habis". Perbandingan kuantitatif ada di
> `vesting-schedule.md`.

## Mekanisme lock

Premine dicatat di genesis sebagai dua output coinbase (dev dan owner).
Implementasi di kode:

- `build_genesis_transactions` membuat coinbase genesis dengan 2 output
  premine, jadi alokasi terlihat on-chain sejak blok 0.
- Karena rusty-kaspa commit UTXO state genesis kosong (output genesis coinbase
  tidak akan pernah bisa di-spend), fungsi `genesis_premine_utxos()` di
  `consensus/src/consensus/utxo_set_override.rs` mengubah output tersebut jadi
  initial UTXO set. Entri ditandai `is_coinbase: false` supaya bisa di-spend
  langsung.

Pilihan untuk lock vesting, belum diputuskan:

1. **Multisig + vesting off-chain**: alamat multisig (misal 2-of-3), unlock
   dijalankan manual sesuai jadwal di dokumen ini. Paling sederhana, tapi
   mengandalkan disiplin operator.
2. **Pecah per milestone**: beberapa output ke alamat berbeda (dev, owner,
   cadangan), masing-masing dengan pengawasan terpisah. Tidak menambah
   jaminan teknis, hanya memisah risiko.
3. **Vesting contract**: kalau nanti ada lapisan kontrak, vesting bisa menjadi
   kontrak dengan aliran linear.

Catatan jujur: bahasa script Kaspa tidak punya opcode time-lock seperti
OP_CHECKLOCKTIMEVERIFY di Bitcoin. Jadi lock berbasis waktu tidak bisa
dipaksakan di level consensus node. Jadwal bulanan pada dasarnya adalah
komitmen off-chain yang bisa diaudit publik lewat dokumen dan alamat genesis.

## Emisi

- Rate awal: **0.13665592 CEN/detik**, setara block reward awal
  **0.0136656 CEN/blok** pada 10 BPS.
- Halving setiap 4 tahun (48 bulan), 17 era, total emisi 34,500,000.33 CEN
  (0.33 CEN di atas target karena pembulatan rate per detik).
- Mining selesai dalam 68 tahun, dalam batas spesifikasi 69 tahun.

Detail per era ada di `emission-schedule.md`. Tabel siap pakai untuk kode ada
di `scripts/data/subsidy_table.rs`.

## Deviasi dari Kaspa (wajib dicatat)

Kaspa asli adalah fair-launch tanpa premine. Celenium sengaja menambahkan
premine 50% dengan vesting. Ini perubahan fundamental terhadap model ekonomi
Kaspa dan sudah tercantum di spesifikasi serta dokumen ini sebagai keputusan
desain, bukan kesalahan.

## Status implementasi

Angka di dokumen ini sudah diverifikasi oleh script. Perubahan kode node
sudah diterapkan di working copy lokal (`~/celenium-build`): tabel subsidy,
parameter emisi, prefix, port, output premine genesis, dan regenerasi
konstanta genesis (hash/merkle/commitment + nonce PoW, 1 Nov 2026). Detail di
`network-params.md`.

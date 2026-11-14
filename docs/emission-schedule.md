# Emission Schedule CEN

Parameter emisi (semua bisa diubah di `scripts/calculate-emission.py` lalu
diverifikasi ulang):

| Parameter | Nilai |
|---|---|
| Block rate | 10 BPS (1 blok per 100 ms, sama seperti Kaspa sekarang) |
| Detik per bulan | 2,629,800 (sama seperti konstanta rusty-kaspa) |
| Panjang era halving | 48 bulan (4 tahun) |
| Jumlah era | 17 |
| Rate awal | 0.20795466 CEN/detik |
| Block reward awal | 0.02079547 CEN/blok |
| Model | Geometric halving |

## Cara kerja emisi di rusty-kaspa

`calc_block_subsidy` di `consensus/src/processes/coinbase.rs` bekerja begitu:

1. Kalau `daa_score < deflationary_phase_daa_score`, subsidy konstan
   (`pre_deflationary_phase_base_subsidy`).
2. Setelah itu, subsidy diambil dari tabel bulanan
   `SUBSIDY_BY_MONTH_TABLE`, diindeks oleh `subsidy_month(daa_score)`.
3. Nilai tabel adalah rate **per detik** (pada 1 BPS). Reward per blok =
   `table[bulan] / BPS` (pembulatan ke atas di kode).

Untuk CEN: `deflationary_phase_daa_score = 0` (tabel aktif sejak genesis) dan
tabel diganti dengan tabel CEN 817 entri dari
`scripts/data/subsidy_table.rs`. Entri terakhir bernilai 0 supaya emisi
berhenti setelah bulan ke-816, karena kode memasang index bulan ke entri
terakhir tabel kalau bulan melewati batas. Nilai
`pre_deflationary_phase_base_subsidy = 2,079,547` pradha dijaga agar konsisten
dengan block reward era 0 (`rate[0] / 10` dibulatkan ke atas).

## Tabel reward per era

| Era | Bulan | Rate/detik (CEN) | Reward/blok (CEN) | Total era (CEN) | Kumulatif (CEN) |
|---|---|---|---|---|---|
| 0 | 0-47 | 0.20795466 | 0.02079547 | 26,250,199.913664 | 26,250,199.913664 |
| 1 | 48-95 | 0.10397733 | 0.01039774 | 13,125,099.956832 | 39,375,299.870496 |
| 2 | 96-143 | 0.05198867 | 0.00519887 | 6,562,550.609568 | 45,937,850.480064 |
| 3 | 144-191 | 0.02599433 | 0.00259944 | 3,281,274.673632 | 49,219,125.153696 |
| 4 | 192-239 | 0.01299717 | 0.00129972 | 1,640,637.967968 | 50,859,763.121664 |
| 5 | 240-287 | 0.00649858 | 0.00064986 | 820,318.352832 | 51,680,081.474496 |
| 6 | 288-335 | 0.00324929 | 0.00032493 | 410,159.176416 | 52,090,240.650912 |
| 7 | 336-383 | 0.00162465 | 0.00016247 | 205,080.219360 | 52,295,320.870272 |
| 8 | 384-431 | 0.00081232 | 0.00008124 | 102,539.478528 | 52,397,860.348800 |
| 9 | 432-479 | 0.00040616 | 0.00004062 | 51,269.739264 | 52,449,130.088064 |
| 10 | 480-527 | 0.00020308 | 0.00002031 | 25,634.869632 | 52,474,764.957696 |
| 11 | 528-575 | 0.00010154 | 0.00001016 | 12,817.434816 | 52,487,582.392512 |
| 12 | 576-623 | 0.00005077 | 0.00000508 | 6,408.717408 | 52,493,991.109920 |
| 13 | 624-671 | 0.00002539 | 0.00000254 | 3,204.989856 | 52,497,196.099776 |
| 14 | 672-719 | 0.00001269 | 0.00000127 | 1,601.863776 | 52,498,797.963552 |
| 15 | 720-767 | 0.00000635 | 0.00000064 | 801.563040 | 52,499,599.526592 |
| 16 | 768-815 | 0.00000317 | 0.00000032 | 400.150368 | 52,499,999.676960 |

Setelah bulan ke-816 subsidy = 0 (entri terakhir tabel). Total emisi
52,499,999.68 CEN (0.32 CEN di bawah target 52,500,000 karena pembulatan rate
per detik ke pradha — satuan waktu terkecil 1 detik).

## Jadwal halving

Halving terjadi di batas bulan 48, 96, ..., 768 setelah genesis, yaitu sekitar
setiap 4 tahun sesuai spesifikasi. 17 era selesai hingga halving selesai.

## Catatan teknis

- Rate per detik dibulatkan ke pradha, jadi total emisi 52,499,999.68 CEN,
  0.32 CEN di bawah target 52,500,000. Selisih ini muncul dari pembulatan dan
  tidak bisa dihilangkan selama satuan waktu terkecil adalah 1 detik.
- Tabel lengkap per bulan (817 baris): `scripts/data/subsidy_by_month.csv`.
- Siap salin ke kode: `scripts/data/subsidy_table.rs` (array Rust).
- Tabel di `consensus/src/processes/coinbase.rs` dan
  `scripts/data/subsidy_table.rs` harus identik; diverifikasi oleh test
  `coinbase_tests`.
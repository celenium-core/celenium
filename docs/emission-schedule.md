# Emission Schedule CEN

Parameter emisi (semua bisa diubah di `scripts/calculate-emission.py` lalu
diverifikasi ulang):

| Parameter | Nilai |
|---|---|
| Block rate | 10 BPS (1 blok per 100 ms, sama seperti Kaspa sekarang) |
| Detik per bulan | 2,629,800 (sama seperti konstanta rusty-kaspa) |
| Panjang era halving | 48 bulan (4 tahun) |
| Jumlah era | 17 |
| Rate awal | 0.13665592 CEN/detik |
| Block reward awal | 0.0136656 CEN/blok |
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
terakhir tabel kalau bulan melewati batas.

## Tabel reward per era

| Era | Bulan | Rate/detik (CEN) | Reward/blok (CEN) | Total era (CEN) | Kumulatif (CEN) |
|---|---|---|---|---|---|
| 0 | 0-47 | 0.13665592 | 0.0136656 | 17,250,131.443968 | 17,250,131.443968 |
| 1 | 48-95 | 0.06832796 | 0.0068328 | 8,625,065.721984 | 25,875,197.165952 |
| 2 | 96-143 | 0.03416398 | 0.0034164 | 4,312,532.860992 | 30,187,730.026944 |
| 3 | 144-191 | 0.01708199 | 0.0017082 | 2,156,266.430496 | 32,343,996.457440 |
| 4 | 192-239 | 0.00854100 | 0.0008541 | 1,078,133.846400 | 33,422,130.303840 |
| 5 | 240-287 | 0.00427050 | 0.00042705 | 539,066.923200 | 33,961,197.227040 |
| 6 | 288-335 | 0.00213525 | 0.00021353 | 269,533.461600 | 34,230,730.688640 |
| 7 | 336-383 | 0.00106762 | 0.00010677 | 134,766.099648 | 34,365,496.788288 |
| 8 | 384-431 | 0.00053381 | 0.00005339 | 67,383.049824 | 34,432,879.838112 |
| 9 | 432-479 | 0.00026691 | 0.00002670 | 33,692.156064 | 34,466,571.994176 |
| 10 | 480-527 | 0.00013345 | 0.00001335 | 16,845.446880 | 34,483,417.441056 |
| 11 | 528-575 | 0.00006673 | 0.00000668 | 8,423.354592 | 34,491,840.795648 |
| 12 | 576-623 | 0.00003336 | 0.00000334 | 4,211.046144 | 34,496,051.841792 |
| 13 | 624-671 | 0.00001668 | 0.00000167 | 2,105.523072 | 34,498,157.364864 |
| 14 | 672-719 | 0.00000834 | 0.00000084 | 1,052.761536 | 34,499,210.126400 |
| 15 | 720-767 | 0.00000417 | 0.00000042 | 526.380768 | 34,499,736.507168 |
| 16 | 768-815 | 0.00000209 | 0.00000021 | 263.821536 | 34,500,000.328704 |

Setelah bulan ke-816 subsidy = 0 (entri terakhir tabel).

## Jadwal halving

Halving terjadi di batas bulan 48, 96, ..., 768 setelah genesis, yaitu sekitar
setiap 4 tahun sesuai spesifikasi. 17 era selesai pada tahun ke-68.

## Catatan teknis

- Rate per detik dibulatkan ke pradha, jadi total emisi 34,500,000.33 CEN,
  0.33 CEN di atas target 34,500,000. Selisih ini muncul dari pembulatan dan
  tidak bisa dihilangkan selama satuan waktu terkecil adalah 1 detik.
- Tabel lengkap per bulan (817 baris): `scripts/data/subsidy_by_month.csv`.
- Siap salin ke kode: `scripts/data/subsidy_table.rs` (array Rust).

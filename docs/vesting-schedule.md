# Vesting Schedule CEN

Aturan unlock premine Celenium (Model A):

- Pengembangan: 0.345% dari 22,200,000 CEN per bulan = **76,590 CEN/bulan**.
- Pemilik: 0.345% dari 12,300,000 CEN per bulan = **42,435 CEN/bulan**.
- Total: **119,025 CEN/bulan**.
- Durasi: **290 bulan (24.17 tahun)**. Bulan ke-290 membayar sisa
  (65,490 CEN dev, 36,285 CEN owner) agar total tepat.

Bulan 1 dihitung dari genesis mainnet (atau dari deployment kalau genesis
diganti, sesuai keputusan di `generate-genesis.md`).

## Ringkasan per tahun

| Bulan | Dev unlocked (kumulatif) | Owner unlocked (kumulatif) | Total terkunci (CEN) |
|---|---|---|---|
| 12 | 919,080 | 509,220 | 33,071,700 |
| 24 | 1,838,160 | 1,018,440 | 31,643,400 |
| 36 | 2,757,240 | 1,527,660 | 30,215,100 |
| 48 | 3,676,320 | 2,036,880 | 28,786,800 |
| 60 | 4,595,400 | 2,546,100 | 27,358,500 |
| 72 | 5,514,480 | 3,055,320 | 25,930,200 |
| 84 | 6,433,560 | 3,564,540 | 24,501,900 |
| 96 | 7,352,640 | 4,073,760 | 23,073,600 |
| 108 | 8,271,720 | 4,582,980 | 21,645,300 |
| 120 | 9,190,800 | 5,092,200 | 20,217,000 |
| 132 | 10,109,880 | 5,601,420 | 18,788,700 |
| 144 | 11,028,960 | 6,110,640 | 17,360,400 |
| 156 | 11,948,040 | 6,619,860 | 15,932,100 |
| 168 | 12,867,120 | 7,129,080 | 14,503,800 |
| 180 | 13,786,200 | 7,638,300 | 13,075,500 |
| 192 | 14,705,280 | 8,147,520 | 11,647,200 |
| 204 | 15,624,360 | 8,656,740 | 10,218,900 |
| 216 | 16,543,440 | 9,165,960 | 8,790,600 |
| 228 | 17,462,520 | 9,675,180 | 7,362,300 |
| 240 | 18,381,600 | 10,184,400 | 5,934,000 |
| 252 | 19,300,680 | 10,693,620 | 4,505,700 |
| 264 | 20,219,760 | 11,202,840 | 3,077,400 |
| 276 | 21,138,840 | 11,712,060 | 1,649,100 |
| 288 | 22,057,920 | 12,221,280 | 220,800 |
| 290 | 22,200,000 | 12,300,000 | 0 |

Tabel bulanan lengkap (290 baris): `scripts/data/vesting_monthly.csv`.

## Perbandingan dengan model peluruhan (Model B)

Bacaan literal spesifikasi: 0.69% dari saldo terkunci yang masih ada, per
bulan. Hasil hitung:

| Bulan | Unlocked bulan itu (CEN) | Sisa terkunci (CEN) | Persentase unlocked |
|---|---|---|---|
| 1 | 238,050 | 34,261,950 | 0.69% |
| 12 | 220,592 | 31,749,358 | 7.97% |
| 60 | 158,217 | 22,771,861 | 33.99% |
| 120 | 104,432 | 15,030,665 | 56.43% |
| 290 | 32,184 | 4,632,230 | 86.57% |
| 480 | 8,636 | 1,243,012 | 96.40% |

Model ini tidak pernah mencapai nol, jadi tidak memenuhi syarat "sampai
habis". Karena itu dokumen ini memakai Model A. Kalau pemilik project ingin
Model B, angka di atas adalah konsekuensinya: sebagian coin (sekitar 1.24 juta
CEN setelah 40 tahun) praktis tidak pernah beredar.

## Implementasi

Pilihan mekanisme lock ada di `tokenomics.md` (multisig + off-chain,
OP_CHECKLOCKTIMEVERIFY per milestone, atau kontrak vesting). Belum ada yang
diterapkan di kode.

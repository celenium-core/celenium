# Vesting Schedule CEN (CLTV on-chain)

Jadwal unlock premine Celenium kini di-*enforce* on-chain lewat opcode
`OP_CHECKLOCKTIMEVERIFY` (CLTV) digabung multisig 2-of-3 — bukan sekadar
komitmen off-chain seperti dokumen lama.

## Ringkasan

| Vault | Kunci | Alokasi (CEN) | Per bulan (CEN) | Tranche |
|---|---|---|---|---|
| Pengembangan | K1/K2/K3 (2-of-3) | 9,600,000 | 72,727.27 | 132 |
| Pemilik | K4/K5/K6 (2-of-3) | 6,900,000 | 52,272.73 | 132 |
| **Total** | | **16,500,000** | **125,000** | **264** |

- Unlock otomatis **setiap tanggal 5 bulan** (00:00 UTC), linear, **tanpa cliff**.
- Tranche pertama **5 Desember 2026**, tranche ke-132 **5 November 2037**.
- Durasi 132 bulan (11 tahun).
- Template redeem script per output:
  `<locktime_ms> OP_CHECKLOCKTIMEVERIFY OP_DROP <2> <pubkey_x3> <3> OP_CHECKMULTISIG`

## Kunci publik vault

**Dev vault (2-of-3):**

- K1 = `1b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c`
- K2 = `67d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434`
- K3 = `fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed5`

**Owner vault (2-of-3):**

- K4 = `289c5f5a473b0547ecd89ff5be7d2dedb66a1655077bd34e00b8a2afe847ecdf`
- K5 = `d92d526cd3827d2478ad95d688af5157e6ac92064a22619b34f8dbaa2bc2e2ad`
- K6 = `111b443c9b2510dbe3d3f59a135fbcab376ec1f60f2f8bea97989f3ba4cda21e`

## Alamat wallet (P2SH)

Tiap tranche punya address P2SH sendiri (hash dari redeem script yang berisi
locktime unik). Contoh:

- Dev tranche 1 (5 Des 2026): `cen69:pqz7ztxl8rgkxjlxs893jada33jtwmedd3gmzmpadasrd5gz5namjalz8se46`
- Dev tranche 132 (5 Nov 2037): `cen69:pqpznvfyncptqu9shsee4j84m7nkhyaw7u9f9lf8cphwdrtrxe6v7wt74m69p`
- Owner tranche 1 (5 Des 2026): `cen69:pr3aw5c6vd4jw58a0ae0r5x0k00k6a4s6wu4mzkxqptxwpv2vhy4y8xsn0zpv`
- Owner tranche 132 (5 Nov 2037): `cen69:pr28tfggdpr7tpku5nsypah73s43a7n3mu6m9vrw86c3kcxjse0mj6zmnn7jj`

Tabel lengkap 264 baris (tanggal, timestamp ms, jumlah pradha, jumlah CEN,
address) ada di `docs/premine-cltv-schedule.md`.

## Catatan migrasi

Dokumen ini menggantikan "Model A" (0.345%/bulan, 290 bulan, off-chain) yang
sebelumnya tercatat di sini. Model lama tidak lagi berlaku sejak premine
dipecah menjadi 264 output genesis terkunci CLTV + multisig 2-of-3 — sekarang
jaminan waktunya teknis (konsensus), bukan komitmen operator.
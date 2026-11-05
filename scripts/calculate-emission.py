#!/usr/bin/env python3
"""Celenium (CEN) tokenomics calculator for the rusty-kaspa fork.

rusty-kaspa emission model: a constant pre-deflationary phase, then a monthly
table (SUBSIDY_BY_MONTH_TABLE in consensus/src/processes/coinbase.rs) holding
the per-second reward rate. The node derives the per-block reward as
table[month] / BPS.

CEN parameters: 10 BPS, halving every 4 years (48 months), 17 eras,
deflationary phase from genesis, 34,500,000 CEN mineable.

Usage:
    python3 scripts/calculate-emission.py              print both tables
    python3 scripts/calculate-emission.py --rust FILE  write the Rust table
    python3 scripts/calculate-emission.py --csv DIR    write CSV files
"""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

TOTAL_SUPPLY = 69_000_000
MINING_SUPPLY = 34_500_000
PREMINE_DEV = 22_200_000
PREMINE_OWNER = 12_300_000
PREMINE_TOTAL = PREMINE_DEV + PREMINE_OWNER
BASE_UNITS = 100_000_000

BPS = 10
SECONDS_PER_MONTH = 2_629_800
ERA_MONTHS = 48
NUM_ERAS = 17
TABLE_MONTHS = NUM_ERAS * ERA_MONTHS

# Fixed monthly unlock: 0.345% of each allocation's original size.
VESTING_RATE_PER_MILLE = 345
VESTING_DENOMINATOR = 100_000


def subsidy_table() -> list[int]:
    """Per-second reward rate in pradha for each month, zero after the last era.

    The rate is constant inside an era and halves at each era boundary. The
    trailing zero matters: calc_block_subsidy clamps the month index to the
    last entry, so without it emission would continue forever at the final
    era rate.
    """
    series = sum(0.5**i for i in range(NUM_ERAS))
    initial = MINING_SUPPLY * BASE_UNITS / (SECONDS_PER_MONTH * ERA_MONTHS * series)
    table = [round(initial / 2 ** (month // ERA_MONTHS)) for month in range(TABLE_MONTHS)]
    table.append(0)
    return table


def era_summary(table: list[int]) -> list[tuple[int, int, int, int]]:
    """(era, per-second rate, era total, cumulative) in pradha."""
    rows = []
    cumulative = 0
    for era in range(NUM_ERAS):
        rate = table[era * ERA_MONTHS]
        era_total = rate * SECONDS_PER_MONTH * ERA_MONTHS
        cumulative += era_total
        rows.append((era, rate, era_total, cumulative))
    return rows


def vesting_fixed() -> list[tuple[int, int, int, int, int]]:
    """Monthly unlock at a fixed 0.345% of each original allocation.

    Returns (month, dev_unlocked, owner_unlocked, dev_remaining, owner_remaining).
    Both allocations empty in the same month; the last month carries the remainder.
    """
    dev_month = PREMINE_DEV * VESTING_RATE_PER_MILLE // VESTING_DENOMINATOR
    owner_month = PREMINE_OWNER * VESTING_RATE_PER_MILLE // VESTING_DENOMINATOR
    rows = []
    dev_left, owner_left = PREMINE_DEV, PREMINE_OWNER
    month = 0
    while dev_left > 0 or owner_left > 0:
        month += 1
        dev_now = min(dev_month, dev_left)
        owner_now = min(owner_month, owner_left)
        dev_left -= dev_now
        owner_left -= owner_now
        rows.append((month, dev_now, owner_now, dev_left, owner_left))
    return rows


def vesting_decay(months: int = 480) -> list[tuple[int, int, int]]:
    """Literal spec reading: 0.69% of the still-locked premine per month.

    This schedule asymptotes and never reaches zero, so it cannot be completed.
    Returns (month, unlocked, remaining).
    """
    rows = []
    left = PREMINE_TOTAL
    for month in range(1, months + 1):
        unlocked = left * 69 // 10_000
        left -= unlocked
        rows.append((month, unlocked, left))
    return rows


def fmt_cen(base_units: float) -> str:
    return f"{base_units / BASE_UNITS:,.8f}".rstrip("0").rstrip(".")


def rust_table(table: list[int]) -> str:
    lines = [f"pub const SUBSIDY_BY_MONTH_TABLE: [u64; {len(table)}] = ["]
    for start in range(0, len(table), 12):
        chunk = ", ".join(str(v) for v in table[start:start + 12])
        lines.append(f"    {chunk},")
    lines.append("];")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust", metavar="FILE", help="write the Rust subsidy table to FILE")
    parser.add_argument("--csv", metavar="DIR", help="also write CSV tables to DIR")
    args = parser.parse_args()

    table = subsidy_table()
    eras = era_summary(table)
    vesting = vesting_fixed()
    decay = vesting_decay()

    initial_rate = table[0]
    initial_block = -(-initial_rate // BPS)
    mined_base = sum(v * SECONDS_PER_MONTH for v in table)

    print("=" * 78)
    print("CELENIUM (CEN) TOKENOMICS (rusty-kaspa model)")
    print("=" * 78)
    print(f"Total supply          : {TOTAL_SUPPLY:>15,} CEN")
    print(f"Mineable (mining)     : {MINING_SUPPLY:>15,} CEN  (50%)")
    print(f"Premine (locked)      : {PREMINE_TOTAL:>15,} CEN  (50%)")
    print(f"  development         : {PREMINE_DEV:>15,} CEN")
    print(f"  owner               : {PREMINE_OWNER:>15,} CEN")
    print(f"Block rate            : {BPS} BPS")
    print(f"Halving               : every {ERA_MONTHS} months ({ERA_MONTHS / 12:.0f} years)")
    print(f"Eras                  : {NUM_ERAS} ({NUM_ERAS * ERA_MONTHS / 12:.1f} years)")
    print(f"Initial rate          : {fmt_cen(initial_rate)} CEN/second")
    print(f"Initial block reward  : {fmt_cen(initial_block)} CEN/block")
    print(f"Table size            : {len(table)} months (last entry 0 stops emission)")
    print()
    print("EMISSION SCHEDULE (per era)")
    print("-" * 78)
    print(f"{'era':>3} {'rate/second':>18} {'reward/block':>18} {'era total':>18} {'cumulative':>18}")
    for era, rate, era_total, cumulative in eras:
        block = -(-rate // BPS)
        print(f"{era:>3} {fmt_cen(rate):>18} {fmt_cen(block):>18} {fmt_cen(era_total):>18} {fmt_cen(cumulative):>18}")
    print()

    print("VESTING SCHEDULE (0.345% of each original allocation per month)")
    print("-" * 78)
    print(f"{'month':>5} {'dev unlocked':>15} {'owner unlocked':>15} {'dev left':>15} {'owner left':>15}")
    for month, dev_now, owner_now, dev_left, owner_left in vesting:
        if month <= 3 or month % 12 == 0 or month >= len(vesting) - 2:
            print(f"{month:>5} {dev_now:>15,} {owner_now:>15,} {dev_left:>15,} {owner_left:>15,}")
    dev_month = PREMINE_DEV * VESTING_RATE_PER_MILLE // VESTING_DENOMINATOR
    owner_month = PREMINE_OWNER * VESTING_RATE_PER_MILLE // VESTING_DENOMINATOR
    print(f"  steady monthly unlock: dev {dev_month:,} CEN + owner {owner_month:,} CEN "
          f"= {dev_month + owner_month:,} CEN/month")
    print(f"  full unlock at month {len(vesting)} ({len(vesting) / 12:.2f} years)")
    print()

    print("DECAY MODEL COMPARISON (0.69% of remaining locked per month, literal spec text)")
    print("-" * 78)
    for month, unlocked, left in decay:
        if month in (1, 12, 60, 120, 290, len(decay)):
            pct = 100 * (PREMINE_TOTAL - left) / PREMINE_TOTAL
            print(f"  month {month:>3}: unlocked {unlocked:>10,} CEN, "
                  f"remaining {left:>12,} CEN ({pct:.2f}% unlocked)")
    print("  this model never reaches zero and cannot be completed")
    print()

    print("VERIFICATION")
    print("-" * 78)
    dust = abs(mined_base - MINING_SUPPLY * BASE_UNITS)
    assert dust < 10**11, f"emission dust too large: {fmt_cen(dust)} CEN"
    assert sum(r[1] for r in vesting) == PREMINE_DEV, "dev vesting total mismatch"
    assert sum(r[2] for r in vesting) == PREMINE_OWNER, "owner vesting total mismatch"
    assert MINING_SUPPLY + PREMINE_TOTAL == TOTAL_SUPPLY, "supply split mismatch"
    assert table[-1] == 0, "table must end with a zero entry"
    assert NUM_ERAS * ERA_MONTHS / 12 <= 69, "emission exceeds 69 years"
    print(f"PASS  emission total {fmt_cen(mined_base)} CEN "
          f"(dust {fmt_cen(dust)} CEN from per-second rounding)")
    print(f"PASS  dev vesting sums to exactly {PREMINE_DEV:,} CEN")
    print(f"PASS  owner vesting sums to exactly {PREMINE_OWNER:,} CEN")
    print(f"PASS  mining + premine = {TOTAL_SUPPLY:,} CEN")
    print(f"PASS  mining completes in {NUM_ERAS * ERA_MONTHS / 12:.1f} years (limit 69)")

    if args.rust:
        path = Path(args.rust)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(rust_table(table) + "\n")
        print(f"\nRust table written to {path.resolve()}")

    if args.csv:
        out = Path(args.csv)
        out.mkdir(parents=True, exist_ok=True)
        with open(out / "emission.csv", "w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["era", "rate_per_second", "reward_per_block", "era_total_cen", "cumulative_cen"])
            for era, rate, era_total, cumulative in eras:
                block = -(-rate // BPS)
                writer.writerow([era, rate, block, era_total // BASE_UNITS, cumulative // BASE_UNITS])
        with open(out / "subsidy_by_month.csv", "w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["month", "rate_per_second", "reward_per_block"])
            for month, rate in enumerate(table):
                writer.writerow([month, rate, -(-rate // BPS) if rate else 0])
        with open(out / "vesting_monthly.csv", "w", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["month", "dev_unlocked", "owner_unlocked", "dev_remaining", "owner_remaining"])
            writer.writerows(vesting)
        print(f"CSV tables written to {out.resolve()}")


if __name__ == "__main__":
    main()

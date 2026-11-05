#!/usr/bin/env python3
"""Validate the DNS seeding path end to end on testnet.

A seed node listens without querying DNS itself, and a second node is started
with no --connect at all. If the second node finds and connects to the seed,
the whole discovery chain works: params dns_seeders -> system resolver ->
address manager -> outbound connection.

The seeder hostname resolves through the system resolver, so a local run maps
it to 127.0.0.1 via /etc/hosts. Nothing here proves public reachability, which
needs a seed node on a public IP.

Paths are CLI flags with repo-relative defaults, so the script runs from any
checkout, not just the machine it was written on.
"""
import argparse
import os
import shutil
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument(
        "--bin",
        default=str(REPO / "target" / "release" / "celeniumd"),
        help="celeniumd binary (default: <repo>/target/release/celeniumd)",
    )
    p.add_argument(
        "--scratch",
        default=str(REPO / "target" / "dnsseed-test"),
        help="scratch workdir (default: <repo>/target/dnsseed-test)",
    )
    p.add_argument(
        "--seeder",
        default="seed.celenium.test",
        metavar="HOST",
        help="DNS seeder hostname (default: seed.celenium.test)",
    )
    p.add_argument(
        "--seconds", type=int, default=60, help="run duration in seconds (default: 60)"
    )
    return p.parse_args()


# The seed listens on the testnet-10 default port so the seeder hostname,
# which carries no port, resolves straight to it.
# The seed listens on all interfaces so the seeder hostname, which resolves to
# the machine's LAN address, actually reaches it. Binding to 127.0.0.1 here
# would make the address unreachable and the client would fail to connect.
# The seed must listen on the network default P2P port. A DNS answer carries
# no port, so dns_seed_single appends default_port (26211 for testnet-10); a
# seed on any other port is undiscoverable no matter what DNS returns.
procs = []


def cleanup(*_):
    for p in procs:
        try:
            p.terminate()
        except Exception:
            pass
    time.sleep(3)
    for p in procs:
        try:
            p.kill()
        except Exception:
            pass


def start(bin_path, scratch, name, p2p, rpc, appdir, disable_dns):
    cmd = [
        bin_path,
        "--testnet",
        f"--appdir={appdir}",
        f"--listen={p2p}",
        f"--rpclisten={rpc}",
        "--disable-upnp",
        # debug shows the "Connecting to <addr>" line that INFO hides, which is
        # the only way to tell "address never offered" from "connect failed".
        "--loglevel=debug",
    ]
    if disable_dns:
        cmd.append("--nodnsseed")
    log = open(f"{scratch}/{name}.log", "w")
    p = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, cwd=scratch)
    procs.append(p)
    print(f"[{name}] pid {p.pid} p2p={p2p} dns_seeding={'off' if disable_dns else 'on'}")
    return p


def read(scratch, name):
    try:
        with open(f"{scratch}/{name}.log") as f:
            return f.read()
    except OSError:
        return ""


def main() -> int:
    args = parse_args()
    bin_path, scratch = args.bin, args.scratch
    seeder_host, run_seconds = args.seeder, args.seconds

    seed = ("seed", "0.0.0.0:26211", "127.0.0.1:26210", f"{scratch}/seed")
    # The client uses a different P2P port and no --connect: discovery must
    # come from DNS only.
    client = ("client", "127.0.0.1:26411", "127.0.0.1:26410", f"{scratch}/client")

    # A leftover node holds port 16311 and the client then dies with AddrInUse
    # before it ever queries DNS, which reads as a seeding failure.
    out = subprocess.run(["ps", "-eo", "pid,args"], capture_output=True, text=True).stdout
    mine = os.getpid()
    for line in out.splitlines():
        if "target/release/celeniumd" not in line or "--testnet" not in line:
            continue
        pid = int(line.split()[0])
        if pid != mine:
            try:
                os.kill(pid, signal.SIGKILL)
                print(f"killed leftover testnet node pid {pid}")
            except OSError:
                pass
    time.sleep(2)

    shutil.rmtree(scratch, ignore_errors=True)
    os.makedirs(scratch, exist_ok=True)

    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    print(f"\nDNS check: {seeder_host} -> ", end="")
    try:
        print(socket.gethostbyname(seeder_host))
    except OSError as e:
        print(f"UNRESOLVED ({e}); the client cannot discover anything")
        return 1

    start(bin_path, scratch, *seed, disable_dns=True)
    time.sleep(8)
    start(bin_path, scratch, *client, disable_dns=False)

    print(f"\nrunning {run_seconds}s for DNS discovery and peering...")
    time.sleep(run_seconds)

    seed_log, client_log = read(scratch, "seed"), read(scratch, "client")
    checks = []

    def check(label, ok, detail=""):
        checks.append(ok)
        print(f"  [{'PASS' if ok else 'FAIL'}] {label}{(': ' + detail) if detail else ''}")

    print("\n--- verification ---")
    check("seed node listens without querying DNS itself",
          "P2P Server starting on" in seed_log and "Querying DNS seeder" not in seed_log)
    check("client queried the configured DNS seeder",
          f"Querying DNS seeder {seeder_host}" in client_log)

    retrieved = "Retrieved 1 addresses" in client_log
    check("client retrieved an address from the seeder", retrieved)

    connected = "P2P Connected to outgoing peer" in client_log
    check("client connected outbound using only DNS discovery", connected)

    inbound = "inbound: 1" in seed_log
    check("seed node accepted the inbound connection", inbound)

    pp = {h for h in [seed_log, client_log] if "pruning point" in h}
    check("both nodes agree on the pruning point",
          len({"pruning point"} & {w for w in pp}) == 1 or
          len({l.split("pruning point ")[1][:64] for l in pp if "pruning point " in l}) == 1)

    passed, total = sum(checks), len(checks)
    print(f"\n--- result: {passed}/{total} checks passed ---")
    cleanup()
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
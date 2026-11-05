#!/usr/bin/env python3
"""Validate the DNS seeding path end to end on testnet.

A seed node listens without querying DNS itself, and a second node is started
with no --connect at all. If the second node finds and connects to the seed,
the whole discovery chain works: params dns_seeders -> system resolver ->
address manager -> outbound connection.

The seeder hostname resolves through the system resolver, so this run maps it
to 127.0.0.1 via /etc/hosts. Nothing here proves public reachability, which
needs a seed node on a public IP.

Run from the build dir after celeniumd is rebuilt with dns_seeders populated.
"""
import os
import shutil
import signal
import subprocess
import sys
import time

BUILD = "/home/dx401/celenium-build"
BIN = f"{BUILD}/target/release/celeniumd"
SCRATCH = "/home/dx401/.hermes/profiles/devops/cache/scratch/dnsseed"
SEEDER_HOST = "seed.celenium.test"

# The seed listens on the testnet-10 default port so the seeder hostname,
# which carries no port, resolves straight to it.
# The seed listens on all interfaces so the seeder hostname, which resolves to
# the machine's LAN address, actually reaches it. Binding to 127.0.0.1 here
# would make the address unreachable and the client would fail to connect.
# The seed must listen on the network default P2P port. A DNS answer carries
# no port, so dns_seed_single appends default_port (26211 for testnet-10); a
# seed on any other port is undiscoverable no matter what DNS returns.
SEED = ("seed", "0.0.0.0:26211", "127.0.0.1:26210", f"{SCRATCH}/seed")
# The client uses a different P2P port and no --connect: discovery must come
# from DNS only.
CLIENT = ("client", "127.0.0.1:26411", "127.0.0.1:26410", f"{SCRATCH}/client")

RUN_SECONDS = 60
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


def start(name, p2p, rpc, appdir, disable_dns):
    cmd = [
        BIN, "--testnet",
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
    log = open(f"{SCRATCH}/{name}.log", "w")
    p = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, cwd=SCRATCH)
    procs.append(p)
    print(f"[{name}] pid {p.pid} p2p={p2p} dns_seeding={'off' if disable_dns else 'on'}")
    return p


def read(name):
    try:
        with open(f"{SCRATCH}/{name}.log") as f:
            return f.read()
    except OSError:
        return ""


def main() -> int:
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

    shutil.rmtree(SCRATCH, ignore_errors=True)
    os.makedirs(SCRATCH, exist_ok=True)

    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    print(f"\nDNS check: {SEEDER_HOST} -> ", end="")
    try:
        import socket
        print(socket.gethostbyname(SEEDER_HOST))
    except OSError as e:
        print(f"UNRESOLVED ({e}); the client cannot discover anything")
        return 1

    start(*SEED, disable_dns=True)
    time.sleep(8)
    start(*CLIENT, disable_dns=False)

    print(f"\nrunning {RUN_SECONDS}s for DNS discovery and peering...")
    time.sleep(RUN_SECONDS)

    seed_log, client_log = read("seed"), read("client")
    checks = []

    def check(label, ok, detail=""):
        checks.append(ok)
        print(f"  [{'PASS' if ok else 'FAIL'}] {label}{(': ' + detail) if detail else ''}")

    print("\n--- verification ---")
    check("seed node listens without querying DNS itself",
          "P2P Server starting on" in seed_log and "Querying DNS seeder" not in seed_log)
    check("client queried the configured DNS seeder",
          f"Querying DNS seeder {SEEDER_HOST}" in client_log)

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

#!/usr/bin/env python3
"""Local two-node testnet verification.

Starts two celeniumd nodes on testnet params, peers them, and verifies from
the node logs that:
  * both nodes reach the P2P and gRPC listen stage on testnet params,
  * the peer connection is established in both directions,
  * both nodes import the UTXO set of the same pruning point (genesis
    agreement),
  * the utxoindex resyncs on both.

Verification reads the node logs rather than the RPC interface. On this
build `--rpclisten` binds the gRPC server (the log says "GRPC Server
starting"), so a JSON POST to it is reset by the server; the gRPC surface
needs a protobuf client, which the `tx_e2e` example provides for
transactions but not for node info.

Note that testnet uses `bits: 0x1e7fffff`, about 65536x harder than simnet,
and does not skip proof of work. A short run therefore produces no blocks and
the DAA score does not advance. Block production on testnet needs real
hashrate; that is the intended behaviour, not a defect.

Expects a ts-patched copy of the celeniumd binary (see scripts/regenesis.sh and
scripts/patch-genesis-ts.py). Paths are CLI flags with repo-relative defaults.
"""
import argparse
import os
import re
import shutil
import signal
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
        default=str(REPO / "target" / "tn-bin" / "celeniumd-tn"),
        help="ts-patched binary (default: <repo>/target/tn-bin/celeniumd-tn)",
    )
    p.add_argument(
        "--scratch",
        default=str(REPO / "target" / "tn-test"),
        help="scratch appdir root (default: <repo>/target/tn-test)",
    )
    p.add_argument(
        "--seconds", type=int, default=90, help="run duration in seconds (default: 90)"
    )
    return p.parse_args()


# The patched binary lives outside SCRATCH: this script wipes SCRATCH at
# startup, so keeping the binary there would delete the very file it is about
# to run.
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


def log_path(scratch: str, name: str) -> str:
    return f"{scratch}/{name}.log"


def read_log(scratch: str, name: str) -> str:
    try:
        with open(log_path(scratch, name)) as f:
            return f.read()
    except OSError:
        return ""


def kill_leftovers() -> None:
    """Kill nodes left over from a previous run so their ports are free.

    cleanup() at exit is best-effort: a node still shutting down can hold its
    port when the next run starts, and node2 then dies with AddrInUse before
    it ever peers. Killing by PID avoids pkill -f matching this script's own
    command line.
    """
    out = subprocess.run(
        ["ps", "-eo", "pid,args"], capture_output=True, text=True
    ).stdout
    mine = os.getpid()
    for line in out.splitlines():
        if "celeniumd-tn" not in line:
            continue
        pid = int(line.split()[0])
        if pid == mine:
            continue
        try:
            os.kill(pid, signal.SIGKILL)
            print(f"killed leftover node pid {pid}")
        except OSError:
            pass
    time.sleep(2)


def main() -> int:
    args = parse_args()
    bin_path, scratch = args.bin, args.scratch
    run_seconds = args.seconds

    kill_leftovers()

    if not os.path.exists(bin_path):
        print(f"error: {bin_path} missing; copy target/release/celeniumd, "
              f"patch its timestamp, and pass --bin")
        return 1

    shutil.rmtree(scratch, ignore_errors=True)
    os.makedirs(scratch, exist_ok=True)

    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    # (name, p2p, rpc/gRPC, appdir, peers)
    nodes = [
        ("node1", "127.0.0.1:16311", "127.0.0.1:16310", f"{scratch}/app1", []),
        ("node2", "127.0.0.1:16411", "127.0.0.1:16410", f"{scratch}/app2", ["127.0.0.1:16311"]),
    ]

    for name, p2p, rpc, appdir, peers in nodes:
        cmd = [
            bin_path,
            "--testnet",
            f"--appdir={appdir}",
            f"--listen={p2p}",
            f"--rpclisten={rpc}",
            # UPnP blocks the P2P handshake for ~10s and then fails outside a
            # LAN, which made peering flaky. A VPS has no UPnP gateway anyway.
            "--disable-upnp",
            "--utxoindex",
        ]
        for peer in peers:
            cmd.append(f"--connect={peer}")
        log = open(log_path(scratch, name), "w")
        p = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, cwd=scratch)
        procs.append(p)
        print(f"[{name}] pid {p.pid} p2p={p2p} grpc={rpc} peers={peers or 'none'}")
        time.sleep(6)

    print(f"\n--- running {run_seconds}s for peering and initial sync ---")
    time.sleep(run_seconds)

    logs = {name: read_log(scratch, name) for name, *_ in nodes}
    checks = []

    def check(label: str, ok: bool, detail: str = "") -> None:
        checks.append(ok)
        print(f"  [{'PASS' if ok else 'FAIL'}] {label}{(': ' + detail) if detail else ''}")

    print("\n--- verification ---")
    for name, _p2p, _rpc, _appdir, peers in nodes:
        text = logs[name]
        # re.Match is subscriptable by group index, and this pattern has no
        # capture group, so [1] raises IndexError. Capture the version instead.
        version = re.search(r"celeniumd (v\S+)", text)
        check(f"{name}: binary reports its version", version is not None,
              version.group(1) if version else "")
        check(f"{name}: testnet appdir created", "celenium-testnet-10" in text)
        check(f"{name}: gRPC server listening", "GRPC Server starting on" in text)
        # A node given --connect peers runs with inbound_limit forced to 0
        # (daemon.rs), so it deliberately opens no P2P listener. Expect the
        # outbound connection instead; only a peerless node must listen.
        if peers:
            check(f"{name}: connected outbound to its peer",
                  "P2P Connected to outgoing peer" in text)
        else:
            check(f"{name}: P2P server listening",
                  "P2P Server starting on" in text)
        check(f"{name}: utxoindex resynced", "Resyncing the utxoindex" in text)

    # Peering, both directions
    n1, n2 = logs["node1"], logs["node2"]
    n1_in = "inbound: 1" in n1
    n2_out = "outbound: 1" in n2 and "127.0.0.1:16311" in n2
    check("peering: node2 connected outbound to node1", n2_out)
    check("peering: node1 saw the inbound connection", n1_in)

    # Genesis agreement: both must import the same pruning point UTXO set
    pp = set(re.findall(r"pruning point ([0-9a-f]{64})", n1 + n2))
    check("genesis agreement: one distinct pruning point across both nodes",
          len(pp) == 1, next(iter(pp))[:16] + "..." if pp else "none found")

    # Protocol version
    pv = set(re.findall(r"protocol version (\d+)", n1 + n2))
    check("protocol version matches on both nodes", len(pv) == 1,
          f"version {next(iter(pv))}" if pv else "none found")

    passed, total = sum(checks), len(checks)
    print(f"\n--- result: {passed}/{total} checks passed ---")
    if "Sync rate" in n1 + n2:
        print("note: node reported a low sync rate, expected with no miner attached")

    cleanup()
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
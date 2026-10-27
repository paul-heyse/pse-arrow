#!/usr/bin/env python3
"""Delegated child-cgroup containment probe (stdlib only).

Runs as the main process of a transient systemd user scope. Modes:

  main   steps (a)-(d), controller/no-internal-process checks, charge and limit checks
  oom    MemoryMax enforcement for a leaf child (scope runs with OOMPolicy=continue)
  hold   build a populated leaf, publish its pids, then sleep until the scope is stopped
  nodelegate  xattr and mkdir behaviour in a scope created without Delegate=yes
  tree   (internal) the child process tree placed in the leaf

Each mode prints one JSON document on stdout.
"""

from __future__ import annotations

import errno
import json
import os
import select
import signal
import subprocess
import sys
import time
from pathlib import Path

CG = Path("/sys/fs/cgroup")
SELF = Path(__file__).resolve()


def own_cgroup() -> Path:
    line = Path("/proc/self/cgroup").read_text().strip()
    return CG / line.split("::", 1)[1].lstrip("/")


def read(path: Path) -> str | None:
    try:
        return path.read_text().strip()
    except OSError as exc:
        return f"<{errno.errorcode.get(exc.errno, exc.errno)}>"


def kv(path: Path) -> dict[str, int]:
    text = read(path) or ""
    out: dict[str, int] = {}
    for line in text.splitlines():
        k, _, v = line.partition(" ")
        if v.isdigit():
            out[k] = int(v)
    return out


def xattrs(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    try:
        names = os.listxattr(path)
    except OSError as exc:
        return {"<listxattr>": errno.errorcode.get(exc.errno, str(exc.errno))}
    for name in names:
        try:
            out[name] = os.getxattr(path, name).decode(errors="replace")
        except OSError as exc:
            out[name] = f"<{errno.errorcode.get(exc.errno, exc.errno)}>"
    return out


def write(path: Path, value: str) -> str:
    """Write and return 'ok' or the errno name."""
    try:
        fd = os.open(path, os.O_WRONLY)
        try:
            os.write(fd, value.encode())
        finally:
            os.close(fd)
        return "ok"
    except OSError as exc:
        return errno.errorcode.get(exc.errno, str(exc.errno))


def scope_snapshot(scope: Path) -> dict[str, object]:
    return {
        "memory.current": read(scope / "memory.current"),
        "pids.current": read(scope / "pids.current"),
        "cgroup.procs": (read(scope / "cgroup.procs") or "").split(),
    }


def mover(leaf: Path):
    procs = str(leaf / "cgroup.procs")

    def _pre_exec() -> None:
        # Runs in the forked child before exec: move *self* ("0") into the leaf.
        fd = os.open(procs, os.O_WRONLY)
        try:
            os.write(fd, b"0")
        finally:
            os.close(fd)

    return _pre_exec


def spawn_tree(leaf: Path, *args: str) -> tuple[subprocess.Popen[str], dict[str, object]]:
    child = subprocess.Popen(
        [sys.executable, "-I", str(SELF), "tree", *args],
        stdout=subprocess.PIPE,
        text=True,
        preexec_fn=mover(leaf),
    )
    assert child.stdout is not None
    line = child.stdout.readline()
    return child, json.loads(line)


def wait_unpopulated(leaf: Path, timeout: float) -> dict[str, object]:
    """poll(POLLPRI) on cgroup.events until 'populated 0'."""
    events = os.open(leaf / "cgroup.events", os.O_RDONLY)
    wakeups = 0
    start = time.monotonic()
    try:
        poller = select.poll()
        poller.register(events, select.POLLPRI | select.POLLERR)
        while True:
            os.lseek(events, 0, os.SEEK_SET)
            text = os.read(events, 4096).decode()
            if "populated 0" in text:
                return {
                    "populated0": True,
                    "wakeups": wakeups,
                    "seconds": round(time.monotonic() - start, 4),
                    "events": text.strip(),
                }
            left = timeout - (time.monotonic() - start)
            if left <= 0:
                return {"populated0": False, "wakeups": wakeups, "events": text.strip()}
            got = poller.poll(int(left * 1000))
            if got:
                wakeups += 1
    finally:
        os.close(events)


def kill_and_drain(leaf: Path, pidfds: dict[str, int]) -> dict[str, object]:
    """cgroup.kill, wait for populated 0 through POLLPRI, then check pidfds."""
    # Arm the poll before the kill so the notification cannot be missed.
    events = os.open(leaf / "cgroup.events", os.O_RDONLY)
    os.read(events, 4096)
    poller = select.poll()
    poller.register(events, select.POLLPRI | select.POLLERR)
    t0 = time.monotonic()
    kill = write(leaf / "cgroup.kill", "1")
    wakeups = 0
    text = ""
    while time.monotonic() - t0 < 10:
        if poller.poll(10_000):
            wakeups += 1
        os.lseek(events, 0, os.SEEK_SET)
        text = os.read(events, 4096).decode()
        if "populated 0" in text:
            break
    elapsed = round(time.monotonic() - t0, 4)
    os.close(events)
    exited = {}
    for name, fd in pidfds.items():
        # A pidfd is readable once its process has exited (zombie or reaped).
        exited[name] = bool(select.select([fd], [], [], 2.0)[0])
    return {
        "cgroup.kill": kill,
        "populated0": "populated 0" in text,
        "poll_wakeups": wakeups,
        "seconds_to_populated0": elapsed,
        "events": text.strip(),
        "members_exited_per_pidfd": exited,
    }


def pidfds_for(pids: dict[str, int]) -> dict[str, int]:
    out = {}
    for name, pid in pids.items():
        try:
            out[name] = os.pidfd_open(pid)
        except OSError:
            pass
    return out


def proc_cgroup(pid: int) -> str | None:
    return read(Path(f"/proc/{pid}/cgroup"))


def proc_status(pid: int, key: str) -> str | None:
    text = read(Path(f"/proc/{pid}/status")) or ""
    for line in text.splitlines():
        if line.startswith(key + ":"):
            return line.split(":", 1)[1].strip()
    return None


# ---------------------------------------------------------------- tree (child)


def mode_tree(args: list[str]) -> None:
    kind = args[0]
    report: dict[str, object] = {"child": os.getpid(), "pgid": os.getpgid(0), "sid": os.getsid(0)}
    if kind == "full":
        # Grandchild that leaves the process group and session.
        gc = os.fork()
        if gc == 0:
            os.setsid()
            os.execv("/bin/sleep", ["sleep", "600"])
        report["setsid_grandchild"] = gc
        # Classic double-forked daemon: intermediate exits, daemon is orphaned.
        r, w = os.pipe()
        mid = os.fork()
        if mid == 0:
            os.setsid()
            d = os.fork()
            if d == 0:
                os.close(r)
                os.write(w, str(os.getpid()).encode())
                os.close(w)
                os.execv("/bin/sleep", ["sleep", "600"])
            os._exit(0)
        os.close(w)
        report["daemon"] = int(os.read(r, 64).decode())
        os.close(r)
        os.waitpid(mid, 0)
        mib = int(args[1])
        blob = bytearray(mib << 20)
        for i in range(0, len(blob), 4096):
            blob[i] = 1
        report["allocated_mib"] = mib
        time.sleep(0.2)
        print(json.dumps(report), flush=True)
        time.sleep(600)
    elif kind == "forks":
        n = int(args[0 + 1])
        made, err = 0, None
        for _ in range(n):
            try:
                p = os.fork()
            except OSError as exc:
                err = errno.errorcode.get(exc.errno, str(exc.errno))
                break
            if p == 0:
                time.sleep(600)
                os._exit(0)
            made += 1
        report.update(forked=made, fork_error=err)
        print(json.dumps(report), flush=True)
        time.sleep(600)
    elif kind == "spin":
        print(json.dumps(report), flush=True)
        end = time.monotonic() + float(args[1])
        x = 0
        while time.monotonic() < end:
            x += 1
        time.sleep(600)
    elif kind == "grow":
        print(json.dumps(report), flush=True)
        chunks = []
        for _ in range(int(args[1])):
            chunk = bytearray(32 << 20)
            for i in range(0, len(chunk), 4096):
                chunk[i] = 1
            chunks.append(chunk)
        print(json.dumps({"survived_mib": 32 * len(chunks)}), flush=True)
        time.sleep(600)


# ---------------------------------------------------------------- main


def mode_main() -> dict[str, object]:
    scope = own_cgroup()
    out: dict[str, object] = {"scope": str(scope), "driver_pid": os.getpid()}
    out["scope_xattrs"] = xattrs(scope)
    out["scope_files"] = {
        n: read(scope / n)
        for n in (
            "cgroup.type",
            "cgroup.controllers",
            "cgroup.subtree_control",
            "memory.max",
            "memory.swap.max",
            "pids.max",
            "cpu.max",
            "memory.oom.group",
        )
    }
    out["scope_owner_uid"] = os.stat(scope).st_uid
    out["scope_procs_owner_uid"] = os.stat(scope / "cgroup.procs").st_uid
    out["affinity_count"] = len(os.sched_getaffinity(0))

    # No-internal-process: try enabling each controller while the driver sits in the root.
    enable = {}
    for c in ("memory", "pids", "cpu"):
        res = write(scope / "cgroup.subtree_control", f"+{c}")
        enable[c] = res
        if res == "ok":
            enable[c + "_revert"] = write(scope / "cgroup.subtree_control", f"-{c}")
    out["enable_controllers_with_procs_in_root"] = enable
    out["subtree_control_after"] = read(scope / "cgroup.subtree_control")

    # (a) leaf without controllers
    leaf = scope / "leaf-a"
    try:
        leaf.mkdir()
        out["a_mkdir_leaf"] = "ok"
    except OSError as exc:
        out["a_mkdir_leaf"] = errno.errorcode.get(exc.errno, str(exc.errno))
        return out
    out["a_leaf_files"] = sorted(p.name for p in leaf.iterdir())
    out["a_leaf"] = {
        n: read(leaf / n) for n in ("cgroup.type", "cgroup.controllers", "cgroup.subtree_control")
    }
    out["a_leaf_inode"] = os.stat(leaf).st_ino

    # (b) pre-exec move of the forked child, with memory charge check
    before = scope_snapshot(scope)
    child, rep = spawn_tree(leaf, "full", "256")
    members = {
        "child": rep["child"],
        "setsid_grandchild": rep["setsid_grandchild"],
        "daemon": rep["daemon"],
    }
    after = scope_snapshot(scope)
    out["b_child_report"] = rep
    out["b_member_cgroups"] = {k: proc_cgroup(v) for k, v in members.items()}
    out["b_member_ppid"] = {k: proc_status(v, "PPid") for k, v in members.items()}
    out["b_member_sid"] = {k: os.getsid(v) for k, v in members.items()}
    out["b_leaf_procs"] = (read(leaf / "cgroup.procs") or "").split()
    out["b_leaf_events"] = read(leaf / "cgroup.events")
    out["b_scope_before"] = before
    out["b_scope_after"] = after
    out["b_scope_memory_delta_mib"] = round(
        (int(after["memory.current"]) - int(before["memory.current"])) / 2**20, 1
    )
    out["b_child_rss"] = proc_status(rep["child"], "VmRSS")
    out["b_leaf_has_memory_current"] = (leaf / "memory.current").exists()

    # rmdir while populated must fail
    try:
        leaf.rmdir()
        out["rmdir_while_populated"] = "ok?!"
    except OSError as exc:
        out["rmdir_while_populated"] = errno.errorcode.get(exc.errno, str(exc.errno))

    # (c) + (d) kill subtree; do not reap the direct child first (zombie exclusion).
    fds = pidfds_for(members)
    out["c_d_kill"] = kill_and_drain(leaf, fds)
    out["c_child_state_before_reap"] = proc_status(rep["child"], "State")
    out["c_scope_after_kill"] = scope_snapshot(scope)
    child.wait()
    out["c_child_returncode"] = child.returncode
    for fd in fds.values():
        os.close(fd)
    try:
        leaf.rmdir()
        out["d_rmdir"] = "ok"
    except OSError as exc:
        out["d_rmdir"] = errno.errorcode.get(exc.errno, str(exc.errno))

    # TasksMax enforcement on a leaf child (pids charged to the scope).
    leaf2 = scope / "leaf-pids"
    leaf2.mkdir()
    pids_before = read(scope / "pids.current")
    child2, rep2 = spawn_tree(leaf2, "forks", "200")
    out["tasks"] = {
        "scope_pids_before": pids_before,
        "child_report": rep2,
        "scope_pids.current": read(scope / "pids.current"),
        "scope_pids.max": read(scope / "pids.max"),
        "scope_pids.events": read(scope / "pids.events"),
        "leaf_has_pids_current": (leaf2 / "pids.current").exists(),
    }
    out["tasks"]["kill"] = kill_and_drain(leaf2, {})
    child2.wait()
    leaf2.rmdir()

    # CPUQuota enforcement on a leaf child.
    leaf3 = scope / "leaf-cpu"
    leaf3.mkdir()
    s0 = kv(scope / "cpu.stat")
    child3, rep3 = spawn_tree(leaf3, "spin", "2.0")
    t0 = time.monotonic()
    time.sleep(2.0)
    s1 = kv(scope / "cpu.stat")
    wall = time.monotonic() - t0
    out["cpu"] = {
        "wall_s": round(wall, 3),
        "scope_usage_s": round((s1["usage_usec"] - s0["usage_usec"]) / 1e6, 3),
        "scope_nr_throttled_delta": s1.get("nr_throttled", 0) - s0.get("nr_throttled", 0),
        "leaf_cpu_stat": kv(leaf3 / "cpu.stat"),
        "child_affinity_count": len(os.sched_getaffinity(rep3["child"])),
    }
    out["cpu"]["kill"] = kill_and_drain(leaf3, {})
    child3.wait()
    leaf3.rmdir()

    out["scope_final"] = scope_snapshot(scope)
    out["scope_children_final"] = sorted(p.name for p in scope.iterdir() if p.is_dir())
    return out


def mode_oom() -> dict[str, object]:
    scope = own_cgroup()
    leaf = scope / "leaf-oom"
    leaf.mkdir()
    out: dict[str, object] = {
        "scope": str(scope),
        "memory.max": read(scope / "memory.max"),
        "memory.swap.max": read(scope / "memory.swap.max"),
        "memory.oom.group": read(scope / "memory.oom.group"),
        "events_before": read(scope / "memory.events"),
    }
    child, rep = spawn_tree(leaf, "grow", "24")  # 24 x 32 MiB = 768 MiB > 256 MiB
    assert child.stdout is not None
    rest = child.stdout.read()
    child.wait()
    out["child"] = rep
    out["child_rest_stdout"] = rest.strip()
    out["child_returncode"] = child.returncode
    out["events_after"] = read(scope / "memory.events")
    out["driver_alive"] = True
    out["leaf_events"] = read(leaf / "cgroup.events")
    leaf.rmdir()
    return out


def mode_hold(state: str) -> None:
    scope = own_cgroup()
    leaf = scope / "leaf-hold"
    leaf.mkdir()
    child, rep = spawn_tree(leaf, "full", "16")
    Path(state).write_text(
        json.dumps(
            {
                "scope": str(scope),
                "leaf": str(leaf),
                "leaf_inode": os.stat(leaf).st_ino,
                "driver": os.getpid(),
                "members": {
                    "child": rep["child"],
                    "setsid_grandchild": rep["setsid_grandchild"],
                    "daemon": rep["daemon"],
                },
                "leaf_procs": (read(leaf / "cgroup.procs") or "").split(),
            }
        )
    )
    signal.pause()


def mode_nodelegate() -> dict[str, object]:
    scope = own_cgroup()
    out: dict[str, object] = {"scope": str(scope), "scope_xattrs": xattrs(scope)}
    out["scope_owner_uid"] = os.stat(scope).st_uid
    leaf = scope / "leaf-undelegated"
    try:
        leaf.mkdir()
        out["mkdir"] = "ok"
        leaf.rmdir()
        out["rmdir"] = "ok"
    except OSError as exc:
        out["mkdir"] = errno.errorcode.get(exc.errno, str(exc.errno))
    return out


def main() -> None:
    mode = sys.argv[1]
    if mode == "tree":
        mode_tree(sys.argv[2:])
        return
    if mode == "main":
        result = mode_main()
    elif mode == "oom":
        result = mode_oom()
    elif mode == "nodelegate":
        result = mode_nodelegate()
    elif mode == "hold":
        mode_hold(sys.argv[2])
        return
    else:
        raise SystemExit(f"unknown mode {mode}")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

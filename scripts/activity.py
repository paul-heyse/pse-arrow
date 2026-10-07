# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""What is running for pse-arrow right now: scopes, servers, workers and Cargo processes.

Read-only. It observes systemd units named ``pse-*`` (command and native scopes from
scripts/pse-env, supervised canonical servers and workers), Cargo/rustc/nextest processes
whose working directory is in this checkout, and the effective limits of the agent slice.
It is never a lock or a job ledger: a bare Cargo lock owner cannot be identified, so it
is reported as unknown.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import pse_env  # noqa: E402 -- direct-script path routing

BUILD_TOOLS = {"cargo", "rustc", "cargo-nextest", "clippy-driver", "rustdoc", "maturin"}


def systemctl(*args: str) -> str:
    result = subprocess.run(
        ["systemctl", "--user", *args],
        capture_output=True,
        text=True,
        check=False,
        timeout=10,
    )
    return result.stdout


def units() -> list[dict[str, str]]:
    listed = json.loads(
        systemctl("list-units", "pse-*", "--all", "--no-pager", "--output=json") or "[]"
    )
    found = []
    for unit in listed:
        name = unit["unit"]
        fields = dict(
            line.split("=", 1)
            for line in systemctl(
                "show",
                name,
                "--property=MemoryCurrent",
                "--property=CPUUsageNSec",
                "--property=ActiveEnterTimestampMonotonic",
                "--property=ControlGroup",
                "--property=Slice",
            ).splitlines()
            if "=" in line
        )
        found.append({"unit": name, "active": unit.get("active", ""), **fields})
    return found


def first_command(group: str) -> str:
    procs = Path("/sys/fs/cgroup") / group.lstrip("/") / "cgroup.procs"
    try:
        pid = procs.read_text().split()[0]
        argv = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
    except (OSError, IndexError):
        return "-"
    return " ".join(part.decode(errors="replace") for part in argv if part)[:140]


def elapsed(monotonic_usec: str) -> str:
    try:
        seconds = time.clock_gettime(time.CLOCK_MONOTONIC) - int(monotonic_usec) / 1e6
    except ValueError:
        return "-"
    if seconds < 0 or int(monotonic_usec) == 0:
        return "-"
    hours, rest = divmod(int(seconds), 3600)
    return f"{hours}h{rest // 60:02d}m" if hours else f"{rest // 60}m{rest % 60:02d}s"


def memory(value: str) -> str:
    try:
        return f"{int(value) / (1 << 30):.1f}G"
    except ValueError:
        return "-"


def build_processes(root: Path) -> list[tuple[int, str, str, str]]:
    found = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            name = (entry / "comm").read_text().strip()
            if name not in BUILD_TOOLS:
                continue
            cwd = (entry / "cwd").resolve()
            if cwd != root and root not in cwd.parents:
                continue
            group = (entry / "cgroup").read_text().rsplit("/", 1)[-1].strip()
            argv = (entry / "cmdline").read_bytes().split(b"\0")
            command = " ".join(part.decode(errors="replace") for part in argv if part)
            found.append((int(entry.name), name, group, command[:120]))
        except OSError:
            continue
    return sorted(found)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="structured output")
    args = parser.parse_args(argv)
    observed = units()
    builds = build_processes(ROOT)
    selected = pse_env.slice_name(os.environ)
    bounds = pse_env.limits(selected)
    if args.json:
        print(
            json.dumps(
                {
                    "units": observed,
                    "build_processes": [
                        {"pid": pid, "tool": tool, "cgroup": group, "command": command}
                        for pid, tool, group, command in builds
                    ],
                    "slice": selected,
                    "aggregate_limits": bounds,
                },
                indent=2,
            )
        )
        return 0
    print(f"slice: {selected or 'caller slice'}; aggregate limits: "
          + (", ".join(f"{group} MemoryMax={value}" for group, value in bounds) or "none"))
    if observed:
        print(f"\n{'unit':58} {'state':8} {'memory':>7} {'up':>7}  command")
        for unit in observed:
            print(
                f"{unit['unit']:58} {unit['active']:8} {memory(unit.get('MemoryCurrent', '')):>7}"
                f" {elapsed(unit.get('ActiveEnterTimestampMonotonic', '')):>7}"
                f"  {first_command(unit.get('ControlGroup', ''))}"
            )
    else:
        print("\nno pse-* units")
    if builds:
        print(f"\nbuild processes in this checkout ({len(builds)}); Cargo lock owner: unknown")
        for pid, tool, group, command in builds:
            print(f"  {pid:>8} {tool:14} {group[:44]:44} {command}")
    else:
        print("\nno build processes in this checkout")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

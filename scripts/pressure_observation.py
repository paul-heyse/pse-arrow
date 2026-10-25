# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only Linux pressure and placement facts, with incarnation-bound deltas.

PSI totals are microseconds, not task CPU time. cpu.stat bandwidth counters are
local to that cgroup's quota; ancestor counters describe the ancestor population.
No counter in this module establishes why a particular operation waited.
Kernel interface: kernel.org/doc/html/latest/{accounting/psi,admin-guide/cgroup-v2}.html
"""

from __future__ import annotations

import math
import time
from pathlib import Path
from typing import Literal

import msgspec

from scripts.telemetry_metrics import host_metrics

CGROUP = Path("/sys/fs/cgroup")
PROC = Path("/proc")
COUNTERS = (
    "cpu.stat",
    "cpu.stat.local",
    "io.stat",
    "memory.events",
    "memory.events.local",
)
PLACEMENT = (
    "cpu.max",
    "cpu.weight",
    "cpuset.cpus",
    "cpuset.cpus.effective",
    "cpuset.mems.effective",
    "memory.max",
    "memory.high",
    "memory.swap.max",
    "io.max",
)
PSI = ("cpu", "io", "memory")
MAX_BYTES = 64 << 10


class Reading(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["available", "unavailable"]
    values: dict[str, int | float | str] = msgspec.field(default_factory=dict)
    reason: str | None = None


class Identity(msgspec.Struct, forbid_unknown_fields=True):
    boot: str
    group: str | None
    device: int | None
    inode: int | None


class Restriction(msgspec.Struct, forbid_unknown_fields=True):
    identity: Identity | None
    group: str
    fields: dict[str, Reading]


class Sample(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["observed", "partial", "unavailable"]
    identity: Identity | None
    started_monotonic_ns: int
    finished_monotonic_ns: int
    observed_unix_ns: int
    fields: dict[str, Reading]
    ancestors: list[Restriction] = msgspec.field(default_factory=list)
    reason: str | None = None


class Capture(msgspec.Struct, forbid_unknown_fields=True):
    host: Sample
    selected: Sample


def unavailable(reason: str) -> Reading:
    return Reading("unavailable", reason=reason)


def read(path: Path) -> str:
    with path.open() as stream:
        value = stream.read(MAX_BYTES + 1)
    if len(value) > MAX_BYTES:
        raise ValueError("kernel field exceeds observation bound")
    return value.strip()


def parse(value: str, kind: str) -> dict[str, int | float | str]:
    """Retain named counters, including device identity; reject malformed records."""
    values: dict[str, int | float | str] = {}
    if kind == "placement":
        return {"raw": value}
    for line in value.splitlines():
        parts = line.split()
        if kind == "psi":
            if not parts or parts[0] not in {"some", "full"}:
                raise ValueError("invalid PSI row")
            for pair in parts[1:]:
                key, raw = pair.split("=", 1)
                name = parts[0] + "." + key
                number = int(raw) if key == "total" else float(raw)
                if name in values or number < 0 or not math.isfinite(number):
                    raise ValueError("invalid PSI counter")
                values[name] = number
            if any(
                parts[0] + "." + key not in values
                for key in ("avg10", "avg60", "avg300", "total")
            ):
                raise ValueError("incomplete PSI row")
        elif kind == "io.stat":
            if (
                not parts
                or len(parts[0].split(":")) != 2
                or not all(v.isdecimal() for v in parts[0].split(":"))
            ):
                raise ValueError("invalid IO device")
            for pair in parts[1:]:
                key, raw = pair.split("=", 1)
                name = parts[0] + "." + key
                if name in values or not raw.isdecimal():
                    raise ValueError("invalid IO counter")
                values[name] = int(raw)
        else:
            if len(parts) != 2 or not parts[1].isdecimal() or parts[0] in values:
                raise ValueError("invalid kernel counter")
            values[parts[0]] = int(parts[1])
    if not values and kind not in {"io.stat", "cpu.stat.local"}:
        raise ValueError("empty kernel field")
    return values


def field(path: Path, kind: str) -> Reading:
    try:
        return Reading("available", parse(read(path), kind))
    except (OSError, ValueError) as error:
        return unavailable(f"{type(error).__name__}: {error}")


def identity(directory: Path, group: str | None, proc: Path) -> Identity:
    boot = read(proc / "sys/kernel/random/boot_id")
    if not boot:
        raise ValueError("boot identity unavailable")
    stat = directory.stat() if group is not None else None
    return Identity(
        boot, group, stat.st_dev if stat else None, stat.st_ino if stat else None
    )


def _require_observation(condition: bool, reason: str) -> None:
    """Refuse a mixed or unsupported observation before publishing its fields."""
    if not condition:
        raise ValueError(reason)


def member_affinity(directory: Path, group: str, proc: Path) -> Reading:
    """Actual direct-member scheduler masks, including when cpuset is undelegated.

    Never present an observer's own affinity as the selected workload's placement.
    Identity and membership are checked at both ends; descendant groups are not
    enumerated and their masks are not inferred from these direct members.
    """
    try:
        members = read(directory / "cgroup.procs").split()
        _require_observation(
            not (len(members) > 128 or any(not pid.isdecimal() for pid in members)),
            "direct member affinity observation exceeds bound or is malformed",
        )
        masks: dict[str, int | float | str] = {}
        for pid in members:
            selected = proc / pid
            before = read(selected / "stat").rsplit(")", 1)[1].split()[19]
            _require_observation(
                read(selected / "cgroup") == "0::" + group, "member cgroup changed"
            )
            values = [
                line.split(":", 1)[1].strip()
                for line in read(selected / "status").splitlines()
                if line.startswith("Cpus_allowed_list:")
            ]
            _require_observation(
                not (len(values) != 1 or not values[0]),
                "member scheduler affinity unavailable",
            )
            _require_observation(
                not (
                    read(selected / "stat").rsplit(")", 1)[1].split()[19] != before
                    or read(selected / "cgroup") != "0::" + group
                ),
                "member lifetime changed",
            )
            masks[pid + ":" + before] = values[0]
        return Reading(
            "available", masks, "direct members only; child cgroups unobserved"
        )
    except (OSError, ValueError, IndexError) as error:
        return unavailable(f"{type(error).__name__}: {error}")


def sample(
    group: str | None = None, *, cgroup: Path = CGROUP, proc: Path = PROC
) -> Sample:
    """None selects host PSI; a cgroup path selects that exact kernel population.

    Ancestors retain their own identities and restrictions. No finite quota or
    memory ceiling is replaced by a guessed effective share or remaining budget.
    """
    started = time.monotonic_ns()
    fields: dict[str, Reading] = {}
    ancestors: list[Restriction] = []
    try:
        _require_observation(
            not (
                group is not None
                and (not group.startswith("/") or ".." in Path(group).parts)
            ),
            "expected an absolute cgroup without parent traversal",
        )
        directory = (
            cgroup / group.lstrip("/") if group is not None else proc / "pressure"
        )
        before = identity(directory, group, proc)
        for resource in PSI:
            fields[resource + ".pressure"] = field(
                directory / (resource + ".pressure" if group is not None else resource),
                "psi",
            )
        if group is None:
            # Linux's host CPU 'full' PSI is undefined, even when exported as zero.
            cpu = fields["cpu.pressure"]
            cpu.values = {
                key: value
                for key, value in cpu.values.items()
                if not key.startswith("full.")
            }
        else:
            for name in COUNTERS:
                fields[name] = field(directory / name, name)
            for name in PLACEMENT:
                fields[name] = field(directory / name, "placement")
            fields["direct_member_affinity"] = member_affinity(directory, group, proc)
            parent = directory.parent
            while directory != cgroup and parent.is_relative_to(cgroup):
                _require_observation(
                    not (len(ancestors) >= 256), "ancestor observation exceeds bound"
                )
                relative = (
                    "/" + str(parent.relative_to(cgroup)) if parent != cgroup else "/"
                )
                try:
                    original = identity(parent, relative, proc)
                    limits = {
                        name: field(parent / name, "placement") for name in PLACEMENT
                    }
                    # Ancestor bandwidth counters can reveal ancestor throttling;
                    # they must never be attributed to just the selected child.
                    limits["cpu.stat"] = field(parent / "cpu.stat", "cpu.stat")
                    if identity(parent, relative, proc) != original:
                        raise ValueError("ancestor lifetime changed")
                    ancestors.append(Restriction(original, relative, limits))
                except OSError as error:
                    ancestors.append(
                        Restriction(
                            None, relative, {"identity": unavailable(str(error))}
                        )
                    )
                if parent == cgroup:
                    break
                parent = parent.parent
        _require_observation(
            identity(directory, group, proc) == before,
            "scope incarnation changed during sample",
        )
        partial = any(v.status == "unavailable" for v in fields.values()) or any(
            v.status == "unavailable" for a in ancestors for v in a.fields.values()
        )
        return Sample(
            "partial" if partial else "observed",
            before,
            started,
            time.monotonic_ns(),
            time.time_ns(),
            fields,
            ancestors,
        )
    except (OSError, ValueError) as error:
        # Discard mixed-incarnation fields rather than retain invented continuity.
        return Sample(
            "unavailable",
            None,
            started,
            time.monotonic_ns(),
            time.time_ns(),
            {},
            reason=f"{type(error).__name__}: {error}",
        )


def capture(group: str | None) -> Capture:
    selected = (
        sample(group)
        if group is not None
        else Sample(
            "unavailable",
            None,
            time.monotonic_ns(),
            time.monotonic_ns(),
            time.time_ns(),
            {},
            reason="no selected cgroup",
        )
    )
    captured = Capture(sample(), selected)
    host_metrics().publish_pressure(captured)
    return captured


def delta(previous: Sample, current: Sample) -> dict:
    """Rates only for common cumulative counters in the same kernel incarnation.

    Intervals use sample completion monotonic clocks; read windows are retained in
    the samples. A reset, missing field/device or changed incarnation supplies no rate.
    """
    interval = (current.finished_monotonic_ns - previous.finished_monotonic_ns) / 1e9
    result: dict = {
        "status": "unavailable",
        "interval_seconds": interval,
        "fields": {},
        "previous_identity": msgspec.to_builtins(previous.identity),
        "current_identity": msgspec.to_builtins(current.identity),
        "interval_start_monotonic_ns": previous.finished_monotonic_ns,
        "interval_end_monotonic_ns": current.finished_monotonic_ns,
        "units": "PSI total and *_usec: microseconds; IO *bytes: bytes; other counters: events; per_second retains those units",
        "limits": "cgroup quota counters are local; ancestor populations are separate; counters do not identify a particular operation's wait cause",
    }
    if (
        previous.identity is None
        or previous.identity != current.identity
        or interval <= 0
    ):
        result["reason"] = (
            "scope incarnation changed, unavailable, or nonpositive interval"
        )
        return result
    result["status"] = "available"
    for name in set(previous.fields) | set(current.fields):
        if not (name in COUNTERS or name.endswith(".pressure")):
            continue
        old, new = previous.fields.get(name), current.fields.get(name)
        observed: dict = {}
        result["fields"][name] = observed
        if (
            old is None
            or new is None
            or old.status != "available"
            or new.status != "available"
        ):
            observed.update(
                status="unavailable", reason="field unavailable at an interval endpoint"
            )
            continue
        for key in set(old.values) | set(new.values):
            if name.endswith(".pressure") and not key.endswith(".total"):
                continue
            first, last = old.values.get(key), new.values.get(key)
            if type(first) is not int or type(last) is not int:
                observed[key] = {
                    "status": "unavailable",
                    "reason": "counter absent at an interval endpoint",
                }
            elif last < first:
                observed[key] = {"status": "reset", "previous": first, "current": last}
            else:
                observed[key] = {
                    "status": "available",
                    "delta": last - first,
                    "per_second": (last - first) / interval,
                }
    return result

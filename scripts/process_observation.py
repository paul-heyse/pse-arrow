# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Kernel observations only; each lifetime owner chooses its own release policy.

Absence and unavailable observation are distinct. A process snapshot binds its
optional namespace, executable and cgroup facts to the same boot/start lifetime;
manager invocation identity remains a separately supplied observation.
"""

from __future__ import annotations

import os
import re
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, TypeVar

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping


class Availability(Enum):
    PRESENT = "present"
    ABSENT = "absent"
    UNAVAILABLE = "unavailable"
    CHANGED = "changed"


def boot() -> str:
    return Path("/proc/sys/kernel/random/boot_id").read_text().strip()


def process_stat(pid: int) -> list[str]:
    return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()


def process_group(pid: int) -> str:
    for line in Path(f"/proc/{pid}/cgroup").read_text().splitlines():
        if line.startswith("0::"):
            return line[3:]
    raise ValueError("native operation requires the unified cgroup observation")


def start_identity(pid: int) -> str:
    return process_stat(pid)[19]



def pid_namespace(pid: int) -> str:
    return os.readlink(f"/proc/{pid}/ns/pid")


def executable(pid: int) -> tuple[str, tuple[int, int]]:
    path = Path(f"/proc/{pid}/exe")
    identity = path.stat()
    return os.readlink(path), (identity.st_dev, identity.st_ino)


def group_identity(group: str) -> int | None:
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise ValueError("Invalid registered cgroup")
    try:
        return (Path("/sys/fs/cgroup") / group.lstrip("/")).stat().st_ino
    except FileNotFoundError:
        return None


def populated(group: str) -> bool:
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise ValueError("invalid native scope group")
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    try:
        identity = directory.stat().st_ino
    except FileNotFoundError:
        return False
    try:
        fields = dict(line.split() for line in (directory / "cgroup.events").read_text().splitlines())
    except FileNotFoundError:
        try:
            directory.stat()
        except FileNotFoundError:
            return False
        raise
    if fields.get("populated") not in {"0", "1"}:
        raise ValueError("native scope population unavailable")
    try:
        current = directory.stat().st_ino
    except FileNotFoundError:
        return False
    if current != identity:
        raise ValueError("native scope lifetime changed during population observation")
    return fields["populated"] == "1"


@dataclass(frozen=True)
class ProcessObservation:
    pid: int
    availability: Availability
    boot: str | None = None
    start: str | None = None
    state: str | None = None
    namespace: str | None = None
    executable: tuple[str, tuple[int, int]] | None = None
    group: str | None = None
    invocation: str | None = None
    uncertainty: Mapping[str, str] = field(default_factory=dict)


Observed = TypeVar("Observed")


def _optional(name: str, reader: Callable[[], Observed],
              uncertainty: dict[str, str]) -> Observed | None:
    try:
        return reader()
    except (OSError, ValueError) as error:
        uncertainty[name] = type(error).__name__
        return None


def invocation_identity(group: str, unit: str, manager: Mapping[str, str]) -> str:
    """Authenticate supplied manager identity against actual process membership."""
    control = manager.get("ControlGroup", "")
    invocation = manager.get("InvocationID", "")
    if (manager.get("LoadState") != "loaded" or not control.startswith("/")
            or control == "/" or ".." in Path(control).parts
            or Path(control).name != unit
            or not (group == control or group.startswith(control + "/"))
            or re.fullmatch(r"[a-f0-9]{32}", invocation) is None):
        raise ValueError("Manager invocation does not authenticate process membership")
    return invocation


def observe_process(pid: int, *, unit: str | None = None,
                    manager: Mapping[str, str] | None = None) -> ProcessObservation:
    """Bind optional facts to repeated kernel observations; never infer a manager."""
    uncertainty: dict[str, str] = {}
    try:
        generation = boot()
    except (OSError, ValueError) as error:
        return ProcessObservation(pid, Availability.UNAVAILABLE,
                                  uncertainty={"boot": type(error).__name__})
    try:
        before = process_stat(pid)
        start = before[19]
    except FileNotFoundError:
        return ProcessObservation(pid, Availability.ABSENT, boot=generation)
    except (OSError, ValueError, IndexError) as error:
        return ProcessObservation(pid, Availability.UNAVAILABLE,
                                  uncertainty={"identity": type(error).__name__})
    namespace = _optional("namespace", lambda: pid_namespace(pid), uncertainty)
    binary = _optional("executable", lambda: executable(pid), uncertainty)
    group = _optional("group", lambda: process_group(pid), uncertainty)
    try:
        after = process_stat(pid)
        changed = boot() != generation or after[19] != start
        changed = changed or (namespace is not None and pid_namespace(pid) != namespace)
        changed = changed or (binary is not None and executable(pid) != binary)
        changed = changed or (group is not None and process_group(pid) != group)
        if changed:
            return ProcessObservation(pid, Availability.CHANGED,
                                      uncertainty={"identity": "process facts changed"})
    except (OSError, ValueError, IndexError) as error:
        return ProcessObservation(pid, Availability.UNAVAILABLE,
                                  uncertainty={"identity": type(error).__name__})
    invocation = None
    if unit is not None and manager is not None and group is not None:
        invocation = _optional("invocation", lambda: invocation_identity(group, unit, manager), uncertainty)
    else:
        uncertainty["invocation"] = "manager identity not supplied or process membership unavailable"
    return ProcessObservation(pid, Availability.PRESENT, generation, start,
                              after[0], namespace, binary, group, invocation, uncertainty)

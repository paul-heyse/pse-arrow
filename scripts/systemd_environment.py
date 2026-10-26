# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected user-manager address and kernel placement readback."""

from __future__ import annotations

import os
from pathlib import Path

from scripts import process_observation


class BoundaryError(Exception):
    """This boundary, not the command, failed."""


def slice_group(name: str) -> Path:
    """Systemd nests dashed slice names: pse-x.slice lives inside pse.slice."""
    stem = name.removesuffix(".slice").split("-")
    parts = ["-".join(stem[: index + 1]) + ".slice" for index in range(len(stem))]
    service = f"user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service"
    return Path("/sys/fs/cgroup", service, *parts)


def limits(name: str | None) -> list[tuple[str, str]]:
    """Effective memory limits from the selected slice up to the user manager."""
    if name is None:
        return []
    group = slice_group(name)
    found = []
    while group != Path("/sys/fs/cgroup") and group.name:
        limit = group / "memory.max"
        if limit.is_file():
            value = limit.read_text().strip()
            if value != "max":
                found.append((group.name, value))
        group = group.parent
    return found


def require_unbounded_slice(name: str) -> None:
    """Read the actual selected memory cgroup and every ancestor, including root."""
    require_unbounded_group(slice_group(name))


def require_unbounded_group(
    group: Path, *, root: Path = Path("/sys/fs/cgroup")
) -> None:
    """Verify native files rather than inferring absence of a cap from policy."""
    if not group.is_relative_to(root):
        raise BoundaryError("Unbounded build group is outside the cgroup hierarchy")
    if (group / "memory.max").read_text().strip() != "max":
        raise BoundaryError("Unbounded build allocation has a finite OS memory limit")
    while group.is_relative_to(root):
        path = group / "memory.max"
        if path.is_file() and path.read_text().strip() != "max":
            raise BoundaryError(
                "Unbounded build allocation has a finite ancestor memory limit"
            )
        if group == root:
            break
        group = group.parent


def manager_environment() -> dict[str, str]:
    runtime = Path(f"/run/user/{os.getuid()}")
    return {
        "XDG_RUNTIME_DIR": os.environ.get("XDG_RUNTIME_DIR", str(runtime)),
        "DBUS_SESSION_BUS_ADDRESS": os.environ.get(
            "DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus"
        ),
    }


def current_memory_ceiling() -> int | None:
    """Smallest actual cgroup ancestor, including a narrower observer role."""
    group = process_observation.process_group(os.getpid())
    if not group:
        raise BoundaryError("Cannot observe actual enclosing memory placement")
    root = Path("/sys/fs/cgroup")
    current = root / group.lstrip("/")
    ceilings = []
    while current.is_relative_to(root):
        path = current / "memory.max"
        if path.is_file():
            value = path.read_text().strip()
            if value != "max":
                ceilings.append(int(value))
        if current == root:
            break
        current = current.parent
    return min(ceilings) if ceilings else None

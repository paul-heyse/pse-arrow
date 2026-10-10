# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite local lanes bound to actual supervised lifetimes.

The short ledger lock never spans a systemctl call, workload or drain. A missing
caller is insufficient to release capacity: every registered unit must be observed
empty. Unobservable ownership is retained. Evidence pins are deliberately separate.
"""

from __future__ import annotations

import contextlib
import fcntl
import json
import math
import os
import re
import stat
import subprocess
import tempfile
import time
import tomllib
import uuid
from dataclasses import dataclass, replace
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, TypedDict, cast

from scripts import native_operation as operation

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
GIB = 1 << 30
MARKER = "PSE_HOST_ALLOCATION"


HostPolicy = TypedDict(
    "HostPolicy",
    {
        "version": int,
        "aggregate_gib": int,
        "non_agent_headroom_gib": int,
        "pressure_guard_gib": int,
        "admission_seconds": int,
        "functional_cores": list[int],
        "timing_cores": list[int],
        "light": dict[str, int],
        "compiler-cache": dict[str, int],
        "compile": dict[str, int],
        "functional": dict[str, int],
        "wide": dict[str, int],
        "timing": dict[str, int],
        "exclusive": dict[str, int],
        "reference": dict[str, int],
        "exclusive-observer": dict[str, int],
        "test-runner": dict[str, int],
    },
)


class MetadataLedger(TypedDict):
    """Shared private JSON envelope; each owner interprets its own rows."""

    version: int
    owners: dict[str, dict[str, object]]
    parked_services: NotRequired[list[str]]


class BoundUnit(TypedDict, total=False):
    """Observed kernel and systemd identity, absent until a launch binds."""

    group: str
    invocation: str
    inode: int


AllocationRecord = TypedDict(
    "AllocationRecord",
    {
        "boot": str,
        "pid": int,
        "start": str,
        "class": str,
        "memory": int,
        "lane": str,
        "slots": int,
        "cores": list[int],
        "exclusive": bool,
        "deadline": float,
        "units": dict[str, BoundUnit],
        "released": NotRequired[bool],
        "service": NotRequired[str],
        "parkable": NotRequired[bool],
        "borrowed_services": NotRequired[list[str]],
    },
)


class AllocationLedger(TypedDict):
    """The host admission domain of the shared private envelope."""

    version: int
    owners: dict[str, AllocationRecord]
    parked_services: NotRequired[list[str]]


class AdmissionError(ValueError):
    """No safe finite placement could be established."""


@dataclass(frozen=True)
class Profile:
    name: str
    memory: int
    lane: str
    slots: int
    cores: tuple[int, ...]
    exclusive: bool = False


def policy() -> HostPolicy:
    declared = tomllib.loads((ROOT / ".config/agent-capacity.toml").read_text())
    for key, value in declared.items():
        if key.endswith("_cores") and key in {"functional_cores", "timing_cores"}:
            valid = isinstance(value, list) and all(type(core) is int for core in value)
        elif isinstance(value, dict):
            valid = all(type(amount) is int for amount in value.values())
        else:
            valid = type(value) is int
        if not valid:
            raise AdmissionError(f"Invalid capacity declaration {key}")
    return cast("HostPolicy", declared)


def total_memory() -> int:
    return memory_info()["MemTotal"]


def memory_info() -> dict[str, int]:
    return {
        line.split(":", 1)[0]: int(line.split()[1]) * 1024
        for line in Path("/proc/meminfo").read_text().splitlines()
        if len(line.split()) == 3 and line.split()[2] == "kB"
    }


def finite_bytes(value: str) -> int:
    match = re.fullmatch(r"(\d+)([KMGT%]?)", value.strip().upper())
    if not match:
        raise AdmissionError(
            "Coordinated work requires a positive finite PSE_MEMORY_MAX"
        )
    count, suffix = int(match[1]), match[2]
    size = (
        count * total_memory() // 100
        if suffix == "%"
        else count * {"": 1, "K": 1 << 10, "M": 1 << 20, "G": GIB, "T": 1 << 40}[suffix]
    )
    if size <= 0:
        raise AdmissionError("Memory allocation must be positive")
    return size


def settings(name: str) -> dict[str, int]:
    declared = cast("Mapping[str, object]", policy())[name]
    if not isinstance(declared, dict):
        raise AdmissionError("Missing capacity declaration")
    return cast("dict[str, int]", declared)


def execution(name: str) -> dict[str, int]:
    if name == "reference":
        from scripts.surreal_server import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
            reference_execution,
        )

        return reference_execution()
    selected = settings(name)
    return {
        "pool_memory_bytes": selected["pool_gib"] * GIB,
        "worker_bytes": selected["worker_gib"] * GIB,
        "cpu_threads": selected["cpu_threads"],
        "case_lanes": selected["case_lanes"],
        "math_jobs": selected["math_jobs"],
        "compiler_cores": selected["compiler_cores"],
        "admission_wait_ms": policy()["admission_seconds"] * 1000,
        "process_headroom_bytes": (selected["receiver_gib"] - selected["pool_gib"])
        * GIB,
        "observer_memory_bytes": selected["observer_gib"] * GIB,
    }


def select(name: str, requested: str | None = None) -> Profile:
    config = policy()
    functional = tuple(config["functional_cores"])
    timing = tuple(config["timing_cores"])
    amount = finite_bytes(requested) if requested is not None else None
    if name == "compiler-cache":
        declared = settings(name)["memory_gib"] * GIB
        if amount is not None and amount != declared:
            raise AdmissionError(
                "Compiler-cache capacity is declared by its resident light role"
            )
        return Profile(name, declared, "light", 1, functional)
    if name in {"store-functional", "store-timing"}:
        lane = name.removeprefix("store-")
        declared = settings(lane)
        result = Profile(
            name,
            amount or declared["store_gib"] * GIB,
            name,
            1,
            functional if lane == "functional" else timing,
        )
        if result.memory > declared["store_gib"] * GIB:
            raise AdmissionError(
                "Resident store request exceeds its declared lane; explicitly widen the host profile"
            )
        return result
    if name not in {
        "light",
        "compile",
        "functional",
        "wide",
        "timing",
        "reference",
        "exclusive",
    }:
        raise AdmissionError(f"Unknown resource class {name!r}")
    if name in {"light", "compile", "functional", "wide", "exclusive"} and amount:
        if amount > settings("wide")["slot_gib"] * GIB:
            name = "exclusive"
        elif amount > settings("functional")["slot_gib"] * GIB:
            name = "wide"
        elif name == "light" and amount > settings("light")["memory_gib"] * GIB:
            name = "functional"
    if name == "light":
        result = Profile(
            name, settings(name)["memory_gib"] * GIB, "light", 1, functional
        )
    elif name == "compile":
        result = Profile(
            name, settings(name)["memory_gib"] * GIB, "functional", 1, functional[:4]
        )
    elif name == "functional":
        result = Profile(
            name, settings(name)["slot_gib"] * GIB, name, 1, functional[:4]
        )
    elif name == "wide":
        result = Profile(
            name, settings(name)["slot_gib"] * GIB, "functional", 2, functional
        )
    elif name == "timing":
        result = Profile(name, settings(name)["slot_gib"] * GIB, name, 1, timing)
    elif name == "reference":
        selected = execution(name)
        memory = (
            selected["pool_memory_bytes"]
            + settings(name)["receiver_headroom_gib"] * GIB
        )
        result = Profile(name, memory, "exclusive", 2, timing + functional, True)
        if amount is not None and amount != memory:
            raise AdmissionError(
                "Reference profile requires its unchanged 140GiB primary and 160GiB envelope; select exclusive for another allocation"
            )
    else:
        result = Profile(
            name,
            settings(name)["slot_gib"] * GIB,
            "exclusive",
            2,
            timing + functional,
            True,
        )
    if amount is not None:
        result = replace(result, memory=amount)
    envelope = aggregate(result)
    maximum = total_memory() - config["non_agent_headroom_gib"] * GIB
    if envelope > maximum:
        raise AdmissionError(
            f"Requested aggregate {envelope // GIB}GiB exceeds host capacity {maximum // GIB}GiB after non-agent allowance"
        )
    return result


def aggregate(profile: Profile) -> int:
    config = policy()
    if profile.name == "reference":
        return (
            profile.memory
            + (
                settings("reference")["store_gib"]
                + settings("reference")["observer_gib"]
            )
            * GIB
        )
    if profile.exclusive:
        resident = settings("exclusive")
        return max(
            config["aggregate_gib"] * GIB,
            profile.memory + (resident["store_gib"] + resident["light_gib"]) * GIB,
        )
    return config["aggregate_gib"] * GIB


def classify(command: Sequence[str], native: bool = False) -> str:
    if "--managed-primary-route" in command:
        return "reference"
    if "--unit-only" in command or "py-unit" in command:
        return "compile"
    if any(
        part
        in {
            "py-test",
            "py-unit",
            "native-python",
            "unit-package",
            "unit",
            "native-test",
            "scripts.python_tests",
            "scripts.native_tests",
        }
        for part in command
    ):
        return "exclusive"
    names = {Path(part).name for part in command[:4]}
    if names & {"cargo", "rustc", "maturin", "py_sync.py", "codegen.py"}:
        return "compile"
    if not native and names & {"rg", "cat", "git", "ls", "true", "head", "tail"}:
        return "light"
    return "functional"


def root_path(env: Mapping[str, str] | None = None) -> Path:
    source = os.environ if env is None else env
    home = Path(source.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
    return home / "pse-arrow/host-admission"


def protected(directory: Path) -> None:
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise AdmissionError("Admission state must not traverse symlinks")
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    information = directory.stat()
    if information.st_uid != os.getuid() or stat.S_IMODE(information.st_mode) & 0o077:
        raise AdmissionError("Admission state must be owned with mode0700")


def remaining(deadline: float | None, maximum: float) -> float:
    """A shared absolute clock; ordinary calls retain their existing timeouts."""
    if deadline is None:
        return maximum
    budget = min(maximum, deadline - time.monotonic())
    if budget <= 0:
        raise AdmissionError("Control operation deadline exhausted")
    return budget


def control_run(
    command: Sequence[str],
    *,
    env: Mapping[str, str],
    deadline: float,
    maximum: float = 10,
) -> subprocess.CompletedProcess[str]:
    """Bound every process wait, including cancellation, without descendant pipe waits."""
    remaining(deadline, maximum)
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        child = subprocess.Popen(
            command,
            env=dict(env),
            stdout=output,
            stderr=errors,
        )
        try:
            budget = remaining(deadline, maximum)
            # Reserve part of this same clock for killing/reaping the direct
            # client. The actual scope remains charged independently of it.
            status = child.wait(timeout=budget - min(0.05, budget / 4))
        except BaseException:
            with contextlib.suppress(ProcessLookupError):
                child.kill()
            # The scope stays charged if its client or actual descendants outlive
            # this clock. Never wait indefinitely for their output or their exit.
            with contextlib.suppress(subprocess.TimeoutExpired):
                child.wait(timeout=max(0, deadline - time.monotonic()))
            raise
        output.seek(0)
        errors.seek(0)
        return subprocess.CompletedProcess(
            command,
            status,
            output.read().decode(errors="replace"),
            errors.read().decode(errors="replace"),
        )


def verify_light_control_child(
    env: Mapping[str, str], *, deadline: float
) -> Allocation:
    """Bind only this fixed control child; persistent service ownership stays read-only."""
    marker = env.get(MARKER, "")
    path = Path(marker)
    if not re.fullmatch(r"[a-f0-9]{32}", path.name):
        raise AdmissionError("Missing fixed control ownership")
    owner = readonly_snapshot(path.parent, deadline=deadline)["owners"].get(path.name)
    if owner is None or owner["boot"] != boot() or owner["class"] != "light":
        raise AdmissionError("Missing or stale fixed control ownership")
    if not caller_alive(owner):
        raise AdmissionError("Fixed control launch owner is no longer live")
    group = operation.process_group(os.getpid())
    for unit, binding in owner["units"].items():
        if binding or not re.fullmatch(r"pse-control-[a-f0-9]{32}\.scope", unit):
            continue
        observed = control_unit_observation(unit, deadline)
        control = observed["ControlGroup"]
        if (
            control
            and (group == control or group.startswith(control + "/"))
            and re.fullmatch(r"[a-f0-9]{32}", observed["InvocationID"])
        ):
            allocation = Allocation(
                path.parent,
                path.name,
                Profile(
                    owner["class"],
                    owner["memory"],
                    owner["lane"],
                    owner["slots"],
                    tuple(owner["cores"]),
                    owner["exclusive"],
                ),
                owner["deadline"],
            )
            allocation.bind(unit, deadline=deadline)
            return allocation
    raise AdmissionError("Fixed control child is outside its registered actual scope")


def lock_until(fd: int, kind: int, deadline: float | None) -> None:
    if deadline is None:
        fcntl.flock(fd, kind)
        return
    while True:
        remaining(deadline, 1)
        try:
            fcntl.flock(fd, kind | fcntl.LOCK_NB)
        except BlockingIOError:
            time.sleep(remaining(deadline, 0.01))
        else:
            return


def validate_ledger(record: object) -> MetadataLedger:
    if (
        not isinstance(record, dict)
        or record.get("version") != 1
        or not isinstance(record.get("owners"), dict)
    ):
        raise AdmissionError("Corrupt admission ledger; retain owners for inspection")
    if any(not isinstance(row, dict) for row in record["owners"].values()):
        raise AdmissionError("Corrupt admission owner; retain ledger for inspection")
    return cast("MetadataLedger", record)


def readonly_snapshot(
    directory: Path | None = None, *, deadline: float
) -> AllocationLedger:
    """Detached existing ownership; never create, rewrite, reconcile or repair state."""
    directory = root_path() if directory is None else directory
    remaining(deadline, 1)
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise AdmissionError("Admission state must not traverse symlinks")
    information = directory.stat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or stat.S_IMODE(information.st_mode) & 0o077
    ):
        raise AdmissionError("Admission state must be owned with mode0700")
    fd = os.open(
        directory / ".lock", os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    )
    try:
        validate_private_file(os.fstat(fd), "lock")
        lock_until(fd, fcntl.LOCK_SH, deadline)
        ledger_fd = os.open(
            directory / "allocations.json",
            os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK,
        )
        with os.fdopen(ledger_fd) as source:
            validate_private_file(os.fstat(source.fileno()), "ledger")
            ledger = validate_ledger(json.load(source))
        for nonce, record in ledger["owners"].items():
            if not re.fullmatch(r"[a-f0-9]{32}", nonce):
                raise AdmissionError("Corrupt host allocation identity")
            validate_allocation_record(record)
        parked = ledger.get("parked_services", [])
        if not isinstance(parked, list) or any(not isinstance(x, str) for x in parked):
            raise AdmissionError("Corrupt parked service ownership")
        remaining(deadline, 1)
        return cast("AllocationLedger", ledger)
    finally:
        os.close(fd)


def validate_private_file(information: os.stat_result, label: str) -> None:
    if (
        not stat.S_ISREG(information.st_mode)
        or information.st_uid != os.getuid()
        or information.st_mode & 0o077
    ):
        raise AdmissionError(f"Unsafe admission {label}")


@contextlib.contextmanager
def metadata(
    directory: Path, *, deadline: float | None = None
) -> Generator[MetadataLedger, None, None]:
    remaining(deadline, 1)
    protected(directory)
    fd = os.open(
        directory / ".lock",
        os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        information = os.fstat(fd)
        if information.st_uid != os.getuid() or information.st_mode & 0o077:
            raise AdmissionError("Unsafe admission lock")
        if deadline is not None:
            validate_private_file(information, "lock")
        lock_until(fd, fcntl.LOCK_EX, deadline)
        path = directory / "allocations.json"
        if path.is_symlink():
            raise AdmissionError("Unsafe admission ledger")
        if deadline is not None and path.exists():
            ledger_fd = os.open(
                path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
            )
            with os.fdopen(ledger_fd) as source:
                validate_private_file(os.fstat(source.fileno()), "ledger")
                record = json.load(source)
        else:
            record = (
                json.loads(path.read_text())
                if path.exists()
                else {"version": 1, "owners": {}}
            )
        record = validate_ledger(record)
        yield record
        remaining(deadline, 1)
        operation.write_json(path, record)
    finally:
        os.close(fd)


@contextlib.contextmanager
def allocation_metadata(
    directory: Path, *, deadline: float | None = None
) -> Generator[AllocationLedger, None, None]:
    """Interpret only the host's private rows, preserving the shared lock boundary."""
    with metadata(directory, deadline=deadline) as ledger:
        for record in ledger["owners"].values():
            validate_allocation_record(record)
        yield cast("AllocationLedger", ledger)


def validate_allocation_record(record: Mapping[str, object]) -> None:
    """Malformed host ownership remains charged until explicitly inspected."""
    valid = all(
        isinstance(record.get(key), str) for key in ("boot", "start", "class", "lane")
    )
    valid = valid and bool(
        re.fullmatch(
            r"[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}", str(record.get("boot", ""))
        )
    )
    valid = valid and all(
        type(record.get(key)) is int for key in ("pid", "memory", "slots")
    )
    valid = (
        valid
        and integer(record.get("memory", 0)) > 0
        and integer(record.get("slots", 0)) > 0
    )
    valid = valid and type(record.get("exclusive")) is bool
    cores = record.get("cores")
    valid = (
        valid
        and isinstance(cores, list)
        and bool(cores)
        and all(type(core) is int and core >= 0 for core in cores)
    )
    deadline = record.get("deadline")
    valid = (
        valid
        and type(deadline) in {int, float}
        and math.isfinite(cast("float", deadline))
    )
    units = record.get("units")
    valid = valid and isinstance(units, dict)
    if isinstance(units, dict):
        for name, owner in units.items():
            valid = valid and isinstance(name, str) and isinstance(owner, dict)
            if isinstance(owner, dict):
                valid = valid and all(
                    isinstance(owner[key], str)
                    for key in ("group", "invocation")
                    if key in owner
                )
                valid = valid and ("inode" not in owner or type(owner["inode"]) is int)
    for key in ("released", "parkable"):
        valid = valid and (key not in record or type(record[key]) is bool)
    valid = valid and ("service" not in record or isinstance(record["service"], str))
    services = record.get("borrowed_services", [])
    valid = (
        valid
        and isinstance(services, list)
        and all(isinstance(service, str) for service in services)
    )
    if not valid:
        raise AdmissionError("Corrupt host allocation; retain ownership for inspection")


def boot() -> str:
    return Path("/proc/sys/kernel/random/boot_id").read_text().strip()


def integer(value: object) -> int:
    """Reject an untyped ownership number rather than guessing a live identity."""
    if type(value) is not int:
        raise AdmissionError("Invalid integer in ownership record")
    return value


def caller_alive(record: Mapping[str, object]) -> bool:
    if record["boot"] != boot():
        return False
    try:
        return operation.start_identity(integer(record["pid"])) == record["start"]
    except FileNotFoundError:
        return False


def group_identity(group: str) -> int | None:
    """Kernel identity distinguishes a recreated unit path from its old owner."""
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise AdmissionError("Invalid registered cgroup")
    try:
        return (Path("/sys/fs/cgroup") / group.lstrip("/")).stat().st_ino
    except FileNotFoundError:
        return None


def service_processes_drained(
    record: Mapping[str, object], allocation: Path | None
) -> bool:
    """Do not reclaim an affiliated storage process merely because its cgroup emptied."""
    from scripts import surreal_server  # noqa: PLC0415 -- reciprocal ownership

    raw_services = record.get("borrowed_services", [])
    units = record.get("units")
    if (
        not isinstance(raw_services, list)
        or any(not isinstance(value, str) for value in raw_services)
        or not isinstance(units, dict)
    ):
        return False
    services = list(cast("list[str]", raw_services))
    if record.get("service"):
        if not isinstance(record["service"], str):
            return False
        services.append(record["service"])
    for selected in services:
        state = Path(selected)
        unit = surreal_server.unit_name(state)
        if unit not in units:
            continue  # A caller observing another role did not select this storage unit.
        if allocation is None:
            return False
        launch = None
        try:
            launch = json.loads((state / "service-launch.json").read_text())
            affiliation = launch["allocation"]
            if not isinstance(affiliation, str) or not re.fullmatch(
                r"[a-f0-9]{32}", Path(affiliation).name
            ):
                return False
            if Path(affiliation).absolute() != allocation.absolute():
                continue  # A stale service descriptor belongs to a different owner.
            process = json.loads((state / "server-process.json").read_text())
            generation = launch["generation"]
            if (
                not isinstance(generation, str)
                or not generation
                or process["allocation"] != affiliation
                or process["instance_id"] != generation
                or not isinstance(process["start"], str)
                or not process["start"]
            ):
                return False
            pid = integer(process["pid"])
            if pid <= 0:
                return False
            try:
                if operation.start_identity(pid) == process["start"]:
                    return False
            except FileNotFoundError:
                pass
        except FileNotFoundError:
            # Registration precedes the launch. An unbound, nonexistent unit
            # with no recorded child has no admitted storage process to retain.
            if units[unit] or (launch is not None and "binding" in launch):
                return False
            try:
                (state / "server-process.json").read_text()
            except FileNotFoundError:
                try:
                    if operation.unit_observation(unit).get("LoadState") == "not-found":
                        continue
                except (OSError, ValueError, subprocess.SubprocessError):
                    pass
            except OSError:
                pass
            return False
        except (OSError, ValueError, TypeError, KeyError):
            return False
    return True


def drained(record: Mapping[str, object], *, allocation: Path | None = None) -> bool:
    generation = record.get("boot")
    if not isinstance(generation, str) or not re.fullmatch(
        r"[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}", generation
    ):
        return False
    if generation != boot():
        return True  # No kernel lifetime survives a verified boot generation change.
    units = record.get("units")
    if not isinstance(units, dict):
        return False
    if (
        record.get("service") or record.get("borrowed_services")
    ) and not service_processes_drained(record, allocation):
        return False
    for unit, owner in units.items():
        if not isinstance(owner, dict):
            return False
        try:
            observed = operation.unit_observation(unit)
            if (
                record.get("service")
                and observed.get("LoadState") == "loaded"
                and observed.get("ActiveState") not in {"inactive", "failed"}
            ):
                # RestartSec may retain the old empty cgroup and invocation.
                # The resident owner stays charged until the service stops.
                return False
            if owner.get("group"):
                identity = group_identity(str(owner["group"]))
                if owner.get("inode") is not None and identity != owner["inode"]:
                    # Cgroup destruction proves the previous kernel lifetime ended,
                    # even if systemd has reused this persistent unit name.
                    continue
                if operation.populated(str(owner["group"])):
                    return False
                if (
                    observed.get("LoadState") == "loaded"
                    and observed.get("ActiveState") not in {"inactive", "failed"}
                    and observed.get("InvocationID")
                    not in {"", owner.get("invocation")}
                ):
                    return False
            elif observed.get("LoadState") != "not-found":
                if observed.get("ActiveState") in {
                    "inactive",
                    "failed",
                } and not observed.get("ControlGroup"):
                    continue
                # A registered launch without bound ownership cannot be guessed empty.
                return False
        except (OSError, ValueError, subprocess.SubprocessError):
            return False
    return bool(record.get("released")) or not caller_alive(record)


def drain_borrowed_services(
    directory: Path,
    record: Mapping[str, object],
    *,
    allocation: Path,
    explicit: bool = False,
) -> None:
    """An exclusive caller returns borrowed storage after all other roles drain."""
    raw_services = record.get("borrowed_services", [])
    if not isinstance(raw_services, list) or any(
        not isinstance(value, str) for value in raw_services
    ):
        raise AdmissionError("Invalid borrowed service ownership")
    services = cast("list[str]", raw_services)
    raw_units = record.get("units")
    if not isinstance(raw_units, dict):
        raise AdmissionError("Invalid borrowed service unit ownership")
    bound_units = cast("dict[str, BoundUnit]", raw_units)
    if not services or (not explicit and caller_alive(record)):
        return
    from scripts import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
        surreal_server,
    )

    units = {surreal_server.unit_name(Path(value)) for value in services}
    other = {
        **record,
        "units": {
            unit: owner for unit, owner in bound_units.items() if unit not in units
        },
        "released": True,
    }
    if not drained(other):
        return
    for value in services:
        state = Path(value)
        with surreal_server.lifecycle_reservation(state):
            launch = json.loads((state / "service-launch.json").read_text())
            affiliation = launch["allocation"]
            if not isinstance(affiliation, str) or not re.fullmatch(
                r"[a-f0-9]{32}", Path(affiliation).name
            ):
                raise AdmissionError("Borrowed storage allocation is unobservable")
            if Path(affiliation).absolute() != allocation.absolute():
                continue  # A newer allocation owns this service; the snapshot cannot act on it.
            unit = surreal_server.unit_name(state)
            expected = bound_units.get(unit)
            observed = operation.unit_observation(unit)
            if (
                not expected
                and "binding" not in launch
                and observed.get("LoadState") == "not-found"
            ):
                continue  # No storage lifetime was ever bound for this launch.
            if (
                not isinstance(expected, dict)
                or type(expected.get("inode")) is not int
                or not isinstance(expected.get("group"), str)
                or not expected.get("group")
                or not re.fullmatch(r"[a-f0-9]{32}", expected.get("invocation", ""))
                or launch.get("binding") != expected
            ):
                raise AdmissionError("Borrowed storage binding changed")
            config = surreal_server.config_for(state)
            if config.get("instance_id") != launch.get("generation"):
                raise AdmissionError("Borrowed storage generation changed")
            if observed.get("LoadState") != "not-found":
                stopped = observed.get("ActiveState") in {"inactive", "failed"}
                identity = group_identity(expected["group"])
                if (
                    observed.get("ControlGroup")
                    not in ({"", expected["group"]} if stopped else {expected["group"]})
                    or observed.get("InvocationID")
                    not in (
                        {"", expected["invocation"]}
                        if stopped
                        else {expected["invocation"]}
                    )
                    or (
                        not stopped
                        and observed.get("ActiveState")
                        not in {"active", "activating", "deactivating"}
                    )
                    or identity
                    not in (
                        {None, expected["inode"]} if stopped else {expected["inode"]}
                    )
                ):
                    raise AdmissionError("Borrowed storage lifetime changed")
            # Residency permits borrowing a running service; it does not reverse
            # an intentional stop or a failed service on allocation release.
            if not config.get("resident") or observed.get("ActiveState") != "active":
                surreal_server.workers_drained(state, config)
                surreal_server.stop(state, config)
            else:
                try:
                    surreal_server.park_service(state, expected_binding=expected)
                finally:
                    queue_parked_service(directory, state)


def queue_parked_service(directory: Path, state: Path) -> None:
    """Publish actual parking intent while the caller owns the service lifecycle."""
    from scripts import surreal_server  # noqa: PLC0415 -- reciprocal ownership

    if surreal_server.config_for(state).get("parked"):
        selected = str(state)
        with allocation_metadata(directory) as ledger:
            if selected not in ledger.setdefault("parked_services", []):
                ledger["parked_services"].append(selected)


def withdraw_parked_service(state: Path, *, directory: Path | None = None) -> None:
    """Withdraw only this service's queued resume; retain every allocation owner."""
    directory = root_path() if directory is None else directory
    selected = str(state.resolve())
    with allocation_metadata(directory) as ledger:
        parked = ledger.get("parked_services", [])
        ledger["parked_services"] = [item for item in parked if item != selected]


def reconcile(
    directory: Path, deadline: float | None = None, *, resume_parked: bool = True
) -> None:
    with allocation_metadata(directory) as state:
        snapshot = dict(state["owners"])
    for nonce, item in snapshot.items():
        if (
            isinstance(item, dict)
            and item.get("borrowed_services")
            and not caller_alive(item)
        ):
            try:
                drain_borrowed_services(directory, item, allocation=directory / nonce)
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                # The living service/receiver still owns capacity, even without caller.
                continue
    finished = {
        nonce: item
        for nonce, item in snapshot.items()
        if isinstance(item, dict) and drained(item, allocation=directory / nonce)
    }
    with allocation_metadata(directory) as state:
        owners = state["owners"]
        for nonce, item in finished.items():
            if owners.get(nonce) == item:
                del owners[nonce]
        exclusive_live = any(owner["exclusive"] for owner in owners.values())
        parked = (
            list(state.get("parked_services", []))
            if resume_parked and not exclusive_live
            else []
        )
        if parked:
            state["parked_services"] = []
    if parked:
        from scripts import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
            surreal_server,
        )

        for selected in parked:
            try:
                surreal_server.unpark_service(Path(selected), deadline=deadline)
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                # Retain the explicit parked record; status/recover remains actionable.
                with allocation_metadata(directory) as state:
                    if selected not in state.setdefault("parked_services", []):
                        state["parked_services"].append(selected)
                continue
            with allocation_metadata(directory) as state:
                if selected in state.get("parked_services", []):
                    state["parked_services"].remove(selected)


def conflict(profile: Profile, owners: Mapping[str, object]) -> str | None:
    entries = [item for item in owners.values() if isinstance(item, dict)]
    heavy = [item for item in entries if item["lane"] != "light"]
    if (profile.exclusive and heavy) or (
        profile.lane != "light" and any(item["exclusive"] for item in entries)
    ):
        return "exclusive heavy-work owner"
    used = sum(
        integer(item["slots"]) for item in entries if item["lane"] == profile.lane
    )
    capacity = (
        settings("light")["slots"]
        if profile.lane == "light"
        else 1
        if profile.lane in {"timing", "store-functional", "store-timing"}
        else 2
    )
    if used + profile.slots > capacity:
        return f"{profile.lane} slots occupied"
    # Resident service roles are accounted separately by their originating owner.
    return None


@dataclass
class Allocation:
    directory: Path
    nonce: str
    profile: Profile
    deadline: float

    def environment(self) -> dict[str, str]:
        return {
            MARKER: str(self.directory / self.nonce),
            "PSE_ADMISSION_DEADLINE": str(self.deadline),
            "PSE_RESOURCE_CLASS": self.profile.name,
        }

    def register(self, unit: str, *, deadline: float | None = None) -> None:
        if not re.fullmatch(r"[A-Za-z0-9_.:-]+\.(?:service|scope)", unit):
            raise AdmissionError("Invalid supervised unit")
        with allocation_metadata(self.directory, deadline=deadline) as state:
            owner = state["owners"].get(self.nonce)
            if not isinstance(owner, dict):
                raise AdmissionError("Allocation is no longer live")
            owner["units"].setdefault(unit, {})

    def bind(self, unit: str, *, deadline: float | None = None) -> None:
        observed = control_unit_observation(unit, deadline)
        group = operation.process_group(os.getpid())
        control = observed.get("ControlGroup", "")
        if (
            not control
            or not (group == control or group.startswith(control + "/"))
            or not re.fullmatch(r"[a-f0-9]{32}", observed.get("InvocationID", ""))
        ):
            raise AdmissionError("Child is not in the registered actual unit")
        with allocation_metadata(self.directory, deadline=deadline) as state:
            owner = state["owners"].get(self.nonce)
            if not isinstance(owner, dict) or unit not in owner["units"]:
                raise AdmissionError("Unit was not registered before launch")
            if deadline is not None and owner["units"][unit]:
                raise AdmissionError("Fixed control ownership was already bound")
            identity = group_identity(control)
            if identity is None:
                raise AdmissionError(
                    "Registered child cgroup disappeared before binding"
                )
            owner["units"][unit] = {
                "group": control,
                "invocation": observed["InvocationID"],
                "inode": identity,
            }

    def release(self) -> bool:
        with allocation_metadata(self.directory) as state:
            owner = state["owners"].get(self.nonce)
        if not isinstance(owner, dict):
            return True
        drain_borrowed_services(
            self.directory, owner, allocation=self.directory / self.nonce, explicit=True
        )
        # Explicit owner release only relaxes caller liveness; actual units stay charged.
        snapshot = cast("AllocationRecord", dict(owner))
        snapshot["pid"] = -1
        snapshot["start"] = "released"
        if not drained(snapshot, allocation=self.directory / self.nonce):
            with allocation_metadata(self.directory) as state:
                if state["owners"].get(self.nonce) == owner:
                    state["owners"][self.nonce]["released"] = True
            return False
        with allocation_metadata(self.directory) as state:
            if state["owners"].get(self.nonce) == owner:
                del state["owners"][self.nonce]
        retire_empty_allocation(self)
        return True


def inherit(
    env: Mapping[str, str], *, handoff: bool = False, deadline: float | None = None
) -> Allocation | None:
    marker = env.get(MARKER)
    if not marker:
        return None
    path = Path(marker)
    if not re.fullmatch(r"[a-f0-9]{32}", path.name):
        raise AdmissionError("Invalid inherited allocation")
    if deadline is None:
        with allocation_metadata(path.parent) as state:
            owner = state["owners"].get(path.name)
    else:
        owner = readonly_snapshot(path.parent, deadline=deadline)["owners"].get(
            path.name
        )
    if not isinstance(owner, dict) or owner["boot"] != boot():
        raise AdmissionError("Missing or stale inherited allocation")
    group = operation.process_group(os.getpid())
    member = False
    for unit, value in owner["units"].items():
        if not isinstance(value, dict) or not value.get("group"):
            continue
        if group == value["group"] or group.startswith(value["group"] + "/"):
            observed = control_unit_observation(unit, deadline)
            member = (
                observed.get("InvocationID") == value.get("invocation")
                and observed.get("ControlGroup") == value["group"]
                and (
                    deadline is None
                    or (
                        value.get("inode") is not None
                        and group_identity(value["group"]) == value["inode"]
                    )
                )
            )
            if member:
                break
    own = owner["pid"] == os.getpid() and owner["start"] == operation.start_identity(
        os.getpid()
    )
    if not member and not own and not handoff:
        raise AdmissionError(
            "Inherited allocation does not match actual cgroup ownership"
        )
    if handoff and not member and not caller_alive(owner):
        raise AdmissionError("Handoff lost its launch owner")
    profile = Profile(
        owner["class"],
        owner["memory"],
        owner["lane"],
        owner["slots"],
        tuple(owner["cores"]),
        owner["exclusive"],
    )
    return Allocation(path.parent, path.name, profile, owner["deadline"])


def control_manager_environment() -> dict[str, str]:
    from scripts.pse_env import (  # noqa: PLC0415 -- reciprocal environment owner
        manager_environment,
    )

    return {**os.environ, **manager_environment()}


def control_unit_observation(unit: str, deadline: float | None) -> dict[str, str]:
    if deadline is None:
        return operation.unit_observation(unit)
    result = control_run(
        [
            "systemctl",
            "--user",
            "show",
            unit,
            "--property=InvocationID",
            "--property=ControlGroup",
            "--property=LoadState",
            "--property=ActiveState",
        ],
        env=control_manager_environment(),
        deadline=deadline,
    )
    if result.returncode:
        raise AdmissionError("Control unit observation unavailable")
    fields = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if not all(
        key in fields
        for key in ("LoadState", "ActiveState", "ControlGroup", "InvocationID")
    ):
        raise AdmissionError("Incomplete control unit observation")
    return fields


def acquire_light_control(*, directory: Path, deadline: float) -> Allocation:
    """Ordinary light capacity, without recovery, parking or service reconciliation."""
    profile = select("light")
    reason = "capacity"
    while time.monotonic() < deadline:
        with allocation_metadata(directory, deadline=deadline) as state:
            reason = conflict(profile, state["owners"])
            if reason is None:
                nonce = uuid.uuid4().hex
                state["owners"][nonce] = {
                    "boot": boot(),
                    "pid": os.getpid(),
                    "start": operation.start_identity(os.getpid()),
                    "class": profile.name,
                    "memory": profile.memory,
                    "lane": profile.lane,
                    "slots": profile.slots,
                    "cores": list(profile.cores),
                    "exclusive": profile.exclusive,
                    "deadline": deadline,
                    "units": {},
                }
                selected = Allocation(directory, nonce, profile, deadline)
            else:
                selected = None
        if selected is not None:
            return selected
        time.sleep(remaining(deadline, 0.05))
    raise AdmissionError(f"Control admission deadline exhausted: {reason}")


def retire_light_control(
    allocation: Allocation, owner: AllocationRecord, *, deadline: float
) -> bool:
    """Stop only verified empty control units and their empty originating slice."""
    name = allocation_slice(allocation)
    parent = control_unit_observation(name, deadline)
    if parent["LoadState"] == "not-found" or (
        parent["ActiveState"] in {"inactive", "failed"} and not parent["ControlGroup"]
    ):
        return all(
            control_unit_observation(unit, deadline)["LoadState"] == "not-found"
            for unit in owner["units"]
        )
    parent_group = parent["ControlGroup"]
    if (
        not parent_group
        or Path(parent_group).name != name
        or operation.populated(parent_group)
    ):
        return False
    parent_inode = group_identity(parent_group)
    if parent_inode is None:
        return False
    stop = []
    for unit, binding in owner["units"].items():
        observed = control_unit_observation(unit, deadline)
        if observed["LoadState"] == "not-found":
            continue
        group = binding.get("group", "")
        if not (
            re.fullmatch(r"pse-control-[a-f0-9]{32}\.scope", unit)
            and group.startswith(parent_group + "/")
            and observed["ControlGroup"] == group
            and observed["InvocationID"] == binding.get("invocation")
            and group_identity(group) == binding.get("inode")
            and not operation.populated(group)
        ):
            return False
        stop.append(unit)
    for unit in [*stop, name]:
        if group_identity(parent_group) != parent_inode or operation.populated(
            parent_group
        ):
            return False
        result = control_run(
            ["systemctl", "--user", "stop", unit],
            env=control_manager_environment(),
            deadline=deadline,
            maximum=5,
        )
        if result.returncode:
            return False
    # Unit stop is synchronous; the kernel remains the proof if systemd retains
    # empty metadata for a moment. A missing parent also proves destruction.
    return group_identity(parent_group) is None or not operation.populated(parent_group)


def release_light_control(allocation: Allocation, *, deadline: float) -> bool:
    """Retain unknown/surviving scopes; never drain or resume scientific services."""
    try:
        owner = readonly_snapshot(allocation.directory, deadline=deadline)[
            "owners"
        ].get(allocation.nonce)
        if owner is None:
            return True
        if owner.get("service") or owner.get("borrowed_services"):
            return False
        while True:
            empty = True
            for unit, identity in owner["units"].items():
                observed = control_unit_observation(unit, deadline)
                if identity.get("group"):
                    group = identity["group"]
                    inode = group_identity(group)
                    if identity.get("inode") is not None and inode != identity["inode"]:
                        continue  # Destruction proves the bound kernel lifetime ended.
                    if (
                        observed["LoadState"] != "not-found"
                        and (
                            observed["ControlGroup"] != group
                            or observed["InvocationID"] != identity.get("invocation")
                        )
                    ) or operation.populated(group):
                        empty = False
                        break
                elif observed["LoadState"] != "not-found" and (
                    observed["ActiveState"] not in {"inactive", "failed"}
                    or observed["ControlGroup"]
                ):
                    empty = False
                    break
            if empty:
                break
            # systemd-run can exit before the manager observes its scope empty.
            # Wait only inside the original clock, keeping its charge until then.
            time.sleep(remaining(deadline, 0.01))
        if not retire_light_control(allocation, owner, deadline=deadline):
            return False
        with allocation_metadata(allocation.directory, deadline=deadline) as state:
            if state["owners"].get(allocation.nonce) != owner:
                return False
            del state["owners"][allocation.nonce]
    except (OSError, ValueError, subprocess.SubprocessError):
        return False
    else:
        return True


def require_admission_time(deadline: float, message: str) -> None:
    if time.monotonic() >= deadline:
        raise AdmissionError(message)


def acquire(
    profile: Profile, *, directory: Path | None = None, deadline: float | None = None
) -> Allocation:
    directory = root_path() if directory is None else directory
    deadline = (
        time.monotonic() + policy()["admission_seconds"]
        if deadline is None
        else deadline
    )
    reason = "capacity"
    while time.monotonic() < deadline:
        # Reclaim ended owners without spending this request's original clock on
        # unrelated service startup. Selected store demand owns its own resume.
        reconcile(directory, deadline, resume_parked=False)
        if time.monotonic() >= deadline:
            break
        if (
            profile.lane != "light"
            and memory_info()["MemAvailable"] < policy()["pressure_guard_gib"] * GIB
        ):
            reason = "host MemAvailable below startup pressure guard"
        else:
            selected: Allocation | None = None
            with allocation_metadata(directory) as state:
                parkable = {
                    nonce: owner
                    for nonce, owner in state["owners"].items()
                    if owner.get("service") and owner.get("parkable")
                }
                competing = {
                    nonce: owner
                    for nonce, owner in state["owners"].items()
                    if not profile.exclusive or nonce not in parkable
                }
                reason = conflict(profile, competing)
                if reason is None and time.monotonic() >= deadline:
                    reason = "original admission clock expired"
                if reason is None:
                    nonce = uuid.uuid4().hex
                    # Second normal slot owns the other four physical cores.
                    occupied = {
                        core
                        for value in state["owners"].values()
                        if value["lane"] == "functional"
                        for core in value["cores"]
                    }
                    if profile.lane == "functional" and profile.slots == 1:
                        cores = tuple(core for core in policy()["functional_cores"])
                        profile = replace(
                            profile,
                            cores=next(
                                part
                                for part in (cores[:4], cores[4:])
                                if not occupied.intersection(part)
                            ),
                        )
                    state["owners"][nonce] = {
                        "boot": boot(),
                        "pid": os.getpid(),
                        "start": operation.start_identity(os.getpid()),
                        "class": profile.name,
                        "memory": profile.memory,
                        "lane": profile.lane,
                        "slots": profile.slots,
                        "cores": list(profile.cores),
                        "exclusive": profile.exclusive,
                        "deadline": deadline,
                        "units": {},
                    }
                    selected = Allocation(directory, nonce, profile, deadline)
            if selected is not None:
                if profile.exclusive and parkable:
                    from scripts import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
                        surreal_server,
                    )

                    try:
                        for owner in parkable.values():
                            require_admission_time(
                                deadline,
                                "Original admission clock expired while parking owned services",
                            )
                            service = owner["service"]
                            path = Path(service)
                            binding = owner["units"].get(surreal_server.unit_name(path))
                            if not binding:
                                raise AdmissionError(  # noqa: TRY301 -- release this selected allocation on a missing parking premise
                                    "Parkable service lacks its bound lifetime"
                                )
                            with surreal_server.lifecycle_reservation(path):
                                try:
                                    surreal_server.park_service(
                                        path,
                                        deadline=deadline,
                                        expected_binding=binding,
                                    )
                                finally:
                                    # Publish resume intent under the same lifecycle
                                    # as parking. Preserve partial-stop recovery,
                                    # but a stale stopped premise queues nothing.
                                    queue_parked_service(directory, path)
                        reconcile(directory, deadline, resume_parked=False)
                        require_admission_time(
                            deadline,
                            "Original admission clock expired during service parking",
                        )
                    except Exception:
                        selected.release()
                        raise
                return selected
        time.sleep(min(0.1, max(0, deadline - time.monotonic())))
    raise AdmissionError(f"Admission deadline exhausted: {reason}")


def cpu_set(cores: Sequence[int]) -> tuple[int, ...]:
    selected: set[int] = set()
    for core in cores:
        path = Path(f"/sys/devices/system/cpu/cpu{core}/topology/thread_siblings_list")
        if not path.is_file():
            raise AdmissionError(f"Configured physical core {core} is unavailable")
        siblings = set()
        for part in path.read_text().strip().split(","):
            ends = part.split("-")
            siblings.update(range(int(ends[0]), int(ends[-1]) + 1))
        if min(siblings) != core:
            raise AdmissionError("Host topology differs from physical-core declaration")
        selected.update(siblings)
    return tuple(sorted(selected))


def allocation_slice(allocation: Allocation) -> str:
    return f"pse-allocation-{allocation.nonce}.slice"


def retire_empty_allocation(allocation: Allocation) -> None:
    """Release only this drained runtime cgroup, preserving every stored artifact."""
    name = allocation_slice(allocation)
    observed = operation.unit_observation(name)
    group = observed.get("ControlGroup", "")
    if group and not operation.populated(group):
        from scripts import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
            surreal_server,
        )

        surreal_server.systemctl("stop", name, check=False)


def enforce_allocation(
    allocation: Allocation, env: Mapping[str, str], *, deadline: float | None = None
) -> None:
    """One finite ancestor charges sibling role units to their originating owner."""
    resident = settings("exclusive")
    overhead = (
        (resident["store_gib"] + resident["light_gib"]) * GIB
        if allocation.profile.exclusive
        else 0
    )
    ceiling = (
        aggregate(allocation.profile)
        if allocation.profile.name == "reference"
        else allocation.profile.memory + overhead
    )
    name = allocation_slice(allocation)
    command = ["systemctl", "--user"]
    for arguments in (
        ["start", name],
        [
            "set-property",
            "--runtime",
            name,
            f"MemoryMax={ceiling}",
            "MemorySwapMax=0",
            f"CPUQuota={len(allocation.profile.cores) * 100}%",
            "AllowedCPUs=" + ",".join(map(str, cpu_set(allocation.profile.cores))),
        ],
    ):
        if deadline is None:
            result = subprocess.run(
                [*command, *arguments],
                env=dict(env),
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
        else:
            result = control_run(
                [*command, *arguments], env=env, deadline=deadline, maximum=5
            )
        if result.returncode:
            raise AdmissionError("Cannot enforce originating allocation ancestor")
    from scripts.pse_env import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
        slice_group,
    )

    observed = int((slice_group(name) / "memory.max").read_text().strip())
    if observed != ceiling:
        raise AdmissionError("Allocation memory ancestor readback differs")


@contextlib.contextmanager
def parent_update(
    directory: Path, *, deadline: float | None = None
) -> Generator[None, None, None]:
    """Serialize the finite configuration write, never a workload or a drain."""
    remaining(deadline, 1)
    protected(directory)
    fd = os.open(
        directory / ".parent-update.lock",
        os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        info = os.fstat(fd)
        if info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise AdmissionError("Unsafe aggregate configuration lock")
        lock_until(
            fd,
            fcntl.LOCK_EX,
            min(deadline, time.monotonic() + 5)
            if deadline is not None
            else time.monotonic() + 5,
        )
        yield
    finally:
        os.close(fd)


def enforce_parent(
    profile: Profile, env: Mapping[str, str], *, deadline: float | None = None
) -> None:
    with parent_update(root_path(env), deadline=deadline):
        _enforce_parent(profile, env, deadline=deadline)


def _enforce_parent(
    profile: Profile, env: Mapping[str, str], *, deadline: float | None = None
) -> None:
    cap = aggregate(profile)
    # A concurrent light admission may never lower a widened live heavy envelope.
    with allocation_metadata(root_path(env), deadline=deadline) as state:
        for owner in state["owners"].values():
            existing = Profile(
                owner["class"],
                owner["memory"],
                owner["lane"],
                owner["slots"],
                tuple(owner["cores"]),
                owner["exclusive"],
            )
            cap = max(cap, aggregate(existing))
    command = [
        "systemctl",
        "--user",
        "set-property",
        "--runtime",
        "pse.slice",
        f"MemoryMax={cap}",
        "MemorySwapMax=0",
    ]
    if deadline is None:
        result = subprocess.run(
            command,
            env=dict(env),
            capture_output=True,
            text=True,
            timeout=5,
            check=False,
        )
    else:
        result = control_run(command, env=env, deadline=deadline, maximum=5)
    if result.returncode:
        raise AdmissionError("Cannot enforce admitted aggregate parent")
    from scripts.pse_env import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
        limits,
    )

    smaller = [(name, value) for name, value in limits("pse.slice") if int(value) < cap]
    if smaller:
        raise AdmissionError(
            f"Effective ancestor is below requested aggregate {cap}: {smaller}"
        )


def status() -> dict[str, object]:
    directory = root_path()
    reconcile(directory)
    with allocation_metadata(directory) as state:
        return {**state, "pressure": memory_info(), "policy": policy()}


def measurement_context(env: Mapping[str, str] | None = None) -> dict[str, object]:
    """Record conditions without treating snapshots or CPU ceilings as isolation."""
    source = os.environ if env is None else env
    owner = inherit(source)
    with allocation_metadata(root_path(source)) as state:
        owners = dict(state["owners"])
    ours = owners.get(owner.nonce) if owner else None
    units = ours["units"] if ours else {}
    owned_groups = [value["group"] for value in units.values() if value.get("group")]
    external = []
    for path in Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        try:
            if path.stat().st_uid != os.getuid():
                continue
            fields = (path / "stat").read_text().rsplit(")", 1)[1].split()
            group = operation.process_group(int(path.name))
            if any(
                group == selected or group.startswith(selected + "/")
                for selected in owned_groups
            ):
                continue
            rss = int((path / "statm").read_text().split()[1]) * os.sysconf(
                "SC_PAGE_SIZE"
            )
            ticks = int(fields[11]) + int(fields[12])
            if rss >= 100 * (1 << 20) or (path / "comm").read_text().strip() in {
                "python",
                "rustc",
                "postgres",
                "surreal",
            }:
                external.append(
                    {
                        "pid": int(path.name),
                        "start": fields[19],
                        "name": (path / "comm").read_text().strip(),
                        "rss_bytes": rss,
                        "cpu_ticks": ticks,
                    }
                )
        except (OSError, ValueError, IndexError):
            continue
    service = None
    selected = source.get("PSE_SURREAL_STATE")
    if selected:
        from scripts import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
            surreal_server,
        )

        state_path = Path(selected)
        config = surreal_server.config_for(state_path)
        process = state_path / "server-process.json"
        service = {
            "state": selected,
            "generation": config["instance_id"],
            "resident": bool(config.get("resident")),
            "server_process": surreal_server.read_json(process)
            if process.is_file()
            else None,
            "supervisor": config.get("service_supervisor"),
            "receiver": config.get("primary_receiver"),
            "execution_profile": source.get("PSE_TEST_EXECUTION_PROFILE"),
            "filesystem_cache": "uncontrolled",
            "rocksdb_cache": "not independently measured",
            "schema_provisioning_timed": "declared by selected journey",
        }
    return {
        "clock": time.monotonic(),
        "class": owner.profile.name if owner else None,
        "owner": ours,
        "competing_cooperating_owners": {
            nonce: value
            for nonce, value in owners.items()
            if owner is None or nonce != owner.nonce
        },
        "memory": memory_info(),
        "external_processes": external,
        "service": service,
        "isolation": "unqualified; external load and shared device contention remain possible",
    }

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Local admission lanes bound to actual supervised lifetimes.

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
from typing import TYPE_CHECKING, NotRequired, Protocol, TypedDict, cast

from scripts import operation_lifetime as operation
from scripts import resource_store, systemd_environment, process_observation
from scripts.process_observation import boot
from scripts.host_metadata import (AdmissionError, root_path, protected, remaining, control_run, lock_until, validate_private_file)

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
        "terminal_guards": NotRequired[dict[str, dict[str, object]]],
    },
)


class AllocationLedger(TypedDict):
    """The host admission domain of the shared private envelope."""

    version: int
    owners: dict[str, AllocationRecord]
    parked_services: NotRequired[list[str]]




@dataclass(frozen=True)
class Profile:
    name: str
    memory: int
    lane: str
    slots: int
    cores: tuple[int, ...]
    exclusive: bool = False

    @property
    def unbounded(self) -> bool:
        """Only the explicit ordinary-build class has no OS memory budget."""
        return self.name == "build" and self.lane == "build" and self.memory == 0


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
    functional = declared.get("functional")
    slots = functional.get("slots") if isinstance(functional, dict) else None
    if type(slots) is not int or slots <= 0:
        raise AdmissionError("Invalid capacity declaration functional.slots")
    for lane in ("functional", "timing"):
        selected = declared.get(lane)
        slots = selected.get("store_slots") if isinstance(selected, dict) else None
        if type(slots) is not int or slots <= 0:
            raise AdmissionError(f"Invalid capacity declaration {lane}.store_slots")
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
    functional = tuple(dict.fromkeys(config["functional_cores"]))
    timing = tuple(dict.fromkeys(config["timing_cores"]))
    all_cores = tuple(dict.fromkeys((*timing, *functional)))
    if name == "build":
        if requested is not None:
            raise AdmissionError(
                "The unbounded build class cannot take PSE_MEMORY_MAX; select a finite resource class"
            )
        return Profile(name, 0, "build", 1, all_cores)
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
            name, settings(name)["memory_gib"] * GIB, "functional", 1, functional
        )
    elif name == "functional":
        result = Profile(name, settings(name)["slot_gib"] * GIB, name, 1, functional)
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
        result = Profile(name, memory, "exclusive", 2, all_cores, True)
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
            all_cores,
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
            "unit-libraries",
            "unit-native-selected",
            "unit-native-package",
            "unit-native-capability-package",
            "unit-native-contracts",
            "unit",
            "native-test",
            "scripts.python_tests",
            "scripts.native_tests",
        }
        for part in command
    ):
        return "exclusive"
    if (
        len(command) > 1
        and Path(command[0]).name == "just"
        and command[1]
        in {
            "check",
            "check-package",
            "check-native-package",
            "check-native-test",
            "codegen",
            "codegen-check",
            "py-sync",
            "py-sync-native",
            "py-sync-correctness",
        }
    ):
        return "compile"
    if "scripts.arrow_validation" in command[:4]:
        return "compile"
    names = {Path(part).name for part in command[:4]}
    if names & {"cargo", "rustc", "maturin", "py_sync.py", "codegen.py"}:
        return "compile"
    if not native and names & {"rg", "cat", "git", "ls", "true", "head", "tail"}:
        return "light"
    return "functional"










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
            allocation.bind(unit, deadline=deadline, fresh=True)
            return allocation
    raise AdmissionError("Fixed control child is outside its registered actual scope")




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




@contextlib.contextmanager
def metadata(
    directory: Path, *, deadline: float | None = None, durable: bool = False
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
        if durable:
            resource_store.publish(path, record)
        else:
            operation.write_json(path, record)
    finally:
        os.close(fd)


@contextlib.contextmanager
def allocation_metadata(
    directory: Path, *, deadline: float | None = None, durable: bool = False
) -> Generator[AllocationLedger, None, None]:
    """Interpret only the host's private rows, preserving the shared lock boundary."""
    with metadata(
        directory, deadline=deadline, **({"durable": True} if durable else {})
    ) as ledger:
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
        and (
            integer(record.get("memory", 0)) > 0
            or (
                record.get("class") == "build"
                and record.get("lane") == "build"
                and record.get("memory") == 0
                and record.get("exclusive") is False
            )
        )
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
    try:
        return process_observation.group_identity(group)
    except ValueError as error:
        raise AdmissionError(str(error)) from error







class ServicePolicy(Protocol):
    """Explicit service lifetime decisions composed by the service entry point."""

    def processes_drained(self, record: Mapping[str, object], allocation: Path | None,
                          *, deadline: float | None = None) -> bool: ...

    def return_borrowed(self, directory: Path, record: Mapping[str, object], *,
                        allocation: Path, explicit: bool = False,
                        deadline: float | None = None) -> None: ...

    def park(self, directory: Path, owners: Mapping[str, AllocationRecord], *,
             deadline: float) -> None: ...

    def resume(self, state: Path, *, deadline: float | None = None) -> None: ...


def drained(
    record: Mapping[str, object],
    *,
    allocation: Path | None = None,
    deadline: float | None = None,
    services: ServicePolicy | None = None,
) -> bool:
    if deadline is not None:
        require_admission_time(deadline, "Original drain clock expired")
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
    guards = record.get("terminal_guards", {})
    if not isinstance(guards, dict):
        return False
    for nonce, guard in guards.items():
        scope = guard.get("scope") if isinstance(guard, dict) else None
        binding = units.get(scope.get("unit")) if isinstance(scope, dict) else None
        if (
            not isinstance(scope, dict)
            or not isinstance(binding, dict)
            or type(binding.get("inode")) is not int
            or binding != guard.get("binding")
            or any(
                binding.get(key) != scope.get(key) for key in ("group", "invocation")
            )
            or not operation.terminal_guard_complete(nonce, guard, deadline=deadline)
        ):
            return False
    if (
        record.get("service") or record.get("borrowed_services")
    ) and (services is None or not services.processes_drained(record, allocation, deadline=deadline)):
        return False
    for unit, owner in units.items():
        if not isinstance(owner, dict):
            return False
        try:
            observed = (
                operation.unit_observation(unit)
                if deadline is None
                else operation.unit_observation(unit, deadline=deadline)
            )
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
    if deadline is not None:
        require_admission_time(deadline, "Original drain clock expired")
    return bool(record.get("released")) or not caller_alive(record)






def withdraw_parked_service(state: Path, *, directory: Path | None = None) -> None:
    """Withdraw only this service's queued resume; retain every allocation owner."""
    directory = root_path() if directory is None else directory
    selected = str(state.resolve())
    with allocation_metadata(directory) as ledger:
        parked = ledger.get("parked_services", [])
        ledger["parked_services"] = [item for item in parked if item != selected]


def reconcile(
    directory: Path, deadline: float | None = None, *, resume_parked: bool = True,
    services: ServicePolicy | None = None,
) -> None:
    with allocation_metadata(directory) as state:
        snapshot = dict(state["owners"])
    for nonce, item in snapshot.items():
        if (
            isinstance(item, dict)
            and item.get("borrowed_services")
            and not caller_alive(item)
            and services is not None
        ):
            try:
                services.return_borrowed(directory, item, allocation=directory / nonce)
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
                # The living service/receiver still owns capacity, even without caller.
                continue
    finished = {
        nonce: item
        for nonce, item in snapshot.items()
        if isinstance(item, dict) and drained(item, allocation=directory / nonce, services=services)
    }
    with allocation_metadata(directory) as state:
        owners = state["owners"]
        for nonce, item in finished.items():
            if owners.get(nonce) == item:
                del owners[nonce]
        exclusive_live = any(owner["exclusive"] for owner in owners.values())
        parked = (
            list(state.get("parked_services", []))
            if resume_parked and not exclusive_live and services is not None
            else []
        )
        if parked:
            state["parked_services"] = []
    if parked:
        for selected in parked:
            try:
                services.resume(Path(selected), deadline=deadline)
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
    timing_lanes = {"timing", "store-timing"}
    competing_lanes = {"functional", "store-functional", "build", "exclusive"}
    if (
        profile.lane in competing_lanes
        and any(item["lane"] in timing_lanes for item in entries)
    ) or (
        profile.lane in timing_lanes
        and any(item["lane"] in competing_lanes for item in entries)
    ):
        return "isolated timing owner"
    if (profile.exclusive and heavy) or (
        profile.lane != "light" and any(item["exclusive"] for item in entries)
    ):
        return "exclusive heavy-work owner"
    # Unbounded ordinary work has no owner-count budget. Keep ownership/liveness
    # accounting and the explicit scientific/timing exclusions above, without
    # turning concurrent wrappers or Cargo's waiting builds into admission limits.
    if profile.unbounded:
        return None
    used = sum(
        integer(item["slots"]) for item in entries if item["lane"] == profile.lane
    )
    capacity = (
        settings("light")["slots"]
        if profile.lane == "light"
        else settings(profile.lane.removeprefix("store-"))["store_slots"]
        if profile.lane in {"store-functional", "store-timing"}
        else settings("functional")["slots"]
        if profile.lane == "functional"
        else 1
        if profile.lane == "timing"
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
    services: ServicePolicy | None = None

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

    def bind(
        self, unit: str, *, deadline: float | None = None, fresh: bool = False
    ) -> None:
        """Authenticate actual membership; fixed controls additionally require freshness."""
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
            if fresh and owner["units"][unit]:
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

    def release(self, *, deadline: float | None = None) -> bool:
        with allocation_metadata(self.directory, deadline=deadline) as state:
            owner = state["owners"].get(self.nonce)
        if not isinstance(owner, dict):
            return True
        if self.services is not None:
            self.services.return_borrowed(
                self.directory, owner, allocation=self.directory / self.nonce,
                explicit=True, **operation.deadline_options(deadline),
            )
        # Returning a stopped borrowed service can retire only that affiliation.
        # Judge and release the current record, including any other surviving work.
        with allocation_metadata(self.directory, deadline=deadline) as state:
            owner = state["owners"].get(self.nonce)
        if not isinstance(owner, dict):
            return True
        # Explicit owner release only relaxes caller liveness; actual units stay charged.
        snapshot = cast("AllocationRecord", dict(owner))
        snapshot["pid"] = -1
        snapshot["start"] = "released"
        if not drained(
            snapshot,
            allocation=self.directory / self.nonce,
            services=self.services,
            **operation.deadline_options(deadline),
        ):
            with allocation_metadata(self.directory, deadline=deadline) as state:
                if state["owners"].get(self.nonce) == owner:
                    state["owners"][self.nonce]["released"] = True
            return False
        with allocation_metadata(self.directory, deadline=deadline) as state:
            current = state["owners"].get(self.nonce)
            if current is not None and current != owner:
                return False
            if current == owner:
                del state["owners"][self.nonce]
        retire_empty_allocation(self, **operation.deadline_options(deadline))
        return True


def inherit(
    env: Mapping[str, str], *, handoff: bool = False, deadline: float | None = None,
    services: ServicePolicy | None = None,
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
    return Allocation(path.parent, path.name, profile, owner["deadline"], services)


def control_manager_environment() -> dict[str, str]:
    from scripts.systemd_environment import (  # noqa: PLC0415 -- reciprocal environment owner
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


class AdmissionCheckpoint:
    """Derived diagnostic image, never consulted by admission decisions."""

    def __init__(self, directory: Path, profile: Profile, deadline: float) -> None:
        self.path = (
            directory / "observations" / "admission" / (uuid.uuid4().hex + ".json")
        )
        self.identity = {
            "version": 1,
            "attempt": self.path.stem,
            "pid": os.getpid(),
            "start": operation.start_identity(os.getpid()),
            "boot": boot(),
            "profile": profile.name,
            "deadline": deadline,
            "invocation_declaration": os.environ.get("PSE_TEST_INVOCATION"),
        }
        self.previous: object = None
        self.last = 0.0

    def publish(
        self,
        status: str,
        reason: str | None,
        owners: Mapping[str, object] | None = None,
        allocation: str | None = None,
    ) -> None:
        facts = {
            "status": status,
            "reason": reason,
            "conflicting_owners": dict(owners or {}),
            "allocation": allocation,
        }
        now = time.monotonic()
        if facts == self.previous and now - self.last < 5:
            return
        # Observation failure must not change the admitted command's semantics.
        try:
            protected(self.path.parents[2])
            protected(self.path.parents[1])
            protected(self.path.parent)
            operation.write_json(
                self.path,
                {
                    **self.identity,
                    **facts,
                    "observed_monotonic": now,
                    "observed_at": time.time(),
                },
            )
        except (OSError, AdmissionError):
            return
        self.previous, self.last = facts, now


def admission_observations(
    directory: Path, *, attempt: str | None = None
) -> list[dict]:
    """Detached atomic producer checkpoints; no startup, locks or reconciliation."""
    if attempt is not None and not re.fullmatch(r"[a-f0-9]{32}", attempt):
        raise AdmissionError("Invalid exact admission attempt")
    area = directory / "observations" / "admission"
    paths = (
        [area / (attempt + ".json")]
        if attempt is not None
        else sorted(area.glob("*.json"))[-100:]
    )
    result = []
    for path in paths:
        value = resource_store.read(path)
        if value.get("version") != 1 or value.get("attempt") != path.stem:
            raise AdmissionError("Invalid admission observation")
        value["age_seconds"] = max(0, time.time() - value["observed_at"])
        value["caller_alive"] = caller_alive(value)
        result.append(value)
    return result


def acquire(
    profile: Profile, *, directory: Path | None = None, deadline: float | None = None,
    services: ServicePolicy | None = None,
) -> Allocation:
    directory = root_path() if directory is None else directory
    deadline = (
        time.monotonic() + policy()["admission_seconds"]
        if deadline is None
        else deadline
    )
    reason = "capacity"
    checkpoint = AdmissionCheckpoint(directory, profile, deadline)
    checkpoint.publish("waiting", reason)
    while time.monotonic() < deadline:
        # Reclaim ended owners without spending this request's original clock on
        # unrelated service startup. Selected store demand owns its own resume.
        reconcile(directory, deadline, resume_parked=False, services=services)
        if time.monotonic() >= deadline:
            break
        if (
            profile.lane != "light"
            and not profile.unbounded
            and memory_info()["MemAvailable"] < policy()["pressure_guard_gib"] * GIB
        ):
            reason = "host MemAvailable below startup pressure guard"
            checkpoint.publish("waiting", reason)
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
                    selected = Allocation(directory, nonce, profile, deadline, services)
            if selected is not None:
                checkpoint.publish(
                    "parking" if profile.exclusive and parkable else "admitted",
                    None,
                    allocation=selected.nonce,
                )
                if profile.exclusive and parkable:
                    try:
                        if services is None:
                            raise AdmissionError("Service parking requires its composed lifetime policy")
                        services.park(directory, parkable, deadline=deadline)
                        reconcile(directory, deadline, resume_parked=False, services=services)
                        require_admission_time(
                            deadline,
                            "Original admission clock expired during service parking",
                        )
                    except Exception:
                        selected.release()
                        checkpoint.publish(
                            "failed",
                            "service parking failed",
                            allocation=selected.nonce,
                        )
                        raise
                checkpoint.publish("admitted", None, allocation=selected.nonce)
                return selected
            checkpoint.publish(
                "waiting",
                reason,
                {
                    nonce: {"class": owner["class"], "units": owner["units"]}
                    for nonce, owner in competing.items()
                },
            )
        time.sleep(min(0.1, max(0, deadline - time.monotonic())))
    checkpoint.publish("expired", reason)
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
    # Dashed slice names create real parents: psebuild is deliberately outside pse.slice.
    prefix = "psebuild" if allocation.profile.unbounded else "pse"
    return f"{prefix}-allocation-{allocation.nonce}.slice"


def retire_empty_allocation(
    allocation: Allocation, *, deadline: float | None = None
) -> None:
    """Release only this drained runtime cgroup, preserving every stored artifact."""
    name = allocation_slice(allocation)
    options = operation.deadline_options(deadline)
    observed = operation.unit_observation(name, **options)
    group = observed.get("ControlGroup", "")
    if group and not operation.populated(group):
        result = control_run(["systemctl", "--user", "stop", name],
                             env=operation.manager_control_environment(),
                             deadline=time.monotonic() + 10 if deadline is None else deadline)
        if result.returncode:
            raise AdmissionError("Cannot retire empty allocation cgroup")


def enforce_allocation(
    allocation: Allocation, env: Mapping[str, str], *, deadline: float | None = None
) -> None:
    """Enforce the selected CPU/OS-memory policy on the authentic originating owner."""
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
    properties = [
        "set-property",
        "--runtime",
        name,
        f"CPUQuota={len(cpu_set(allocation.profile.cores)) * 100}%",
        "AllowedCPUs=" + ",".join(map(str, cpu_set(allocation.profile.cores))),
    ]
    if not allocation.profile.unbounded:
        properties += [f"MemoryMax={ceiling}", "MemorySwapMax=0"]
    for arguments in (["start", name], properties):
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
    from scripts.systemd_environment import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
        slice_group,
    )

    if allocation.profile.unbounded:
        systemd_environment.require_unbounded_slice(name)
    else:
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
    if profile.unbounded:
        command = ["systemctl", "--user", "start", "psebuild.slice"]
        result = (
            subprocess.run(
                command,
                env=dict(env),
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            if deadline is None
            else control_run(command, env=env, deadline=deadline, maximum=5)
        )
        if result.returncode:
            raise AdmissionError("Cannot start unbounded build parent")
        systemd_environment.require_unbounded_slice("psebuild.slice")
        return
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
    from scripts.systemd_environment import (  # noqa: PLC0415 -- admission and supervisor/environment have reciprocal ownership
        limits,
    )

    smaller = [(name, value) for name, value in limits("pse.slice") if int(value) < cap]
    if smaller:
        raise AdmissionError(
            f"Effective ancestor is below requested aggregate {cap}: {smaller}"
        )


def status(*, services: ServicePolicy | None = None) -> dict[str, object]:
    directory = root_path()
    reconcile(directory, services=services)
    with allocation_metadata(directory) as state:
        return {**state, "pressure": memory_info(), "policy": policy()}


def measurement_context(env: Mapping[str, str] | None = None, *, service: Mapping[str, object] | None = None) -> dict[str, object]:
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
            fields = process_observation.process_stat(int(path.name))
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

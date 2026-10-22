# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native setup admission owned by one actual process operation and child scope."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import json
import math
import os
import re
import shlex
import signal
import stat
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, Self, TypedDict, TypeVar, cast

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping
    from types import FrameType, TracebackType

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import build_environment  # noqa: E402 -- direct-script path routing

MARKER = "PSE_NATIVE_OPERATION"
# Private handshake with the build-measurement observer, never an admission input.
BUILD_TIMING = "PSE_BUILD_MEASUREMENT_TIMING"
VERSION = 2
CAPABILITIES = ("compiler", "solver", "klu", "isolation", "uno", "petsc")
# Native setup supplies these unless the caller chose a value; `off` removes one.
# The thread budget stays "1" by default because assessment receipts require it.
THREAD_VARIABLES = ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS")
OVERRIDABLE = (*THREAD_VARIABLES, "OMP_PROC_BIND", "OMP_PLACES", "HWLOC_COMPONENTS")
# Names a caller turned `off`, carried to nested native setup so it does not restore them.
OFF_MARKER = "PSE_NATIVE_OFF"
# Native setup owns these; a differing caller value is replaced and reported.
AUTHORITATIVE = {
    "MKL_CBWR": "reproducible MKL kernels (ADR-0108)",
    "MKL_DYNAMIC": "fixed MKL threading (ADR-0108)",
    "OMP_CANCELLATION": "SPRAL requires OpenMP cancellation",
    "SCIPOPTDIR": "follows the admitted solver prefix",
    "SUITESPARSE_INCLUDE_DIR": "follows the prepared KLU prefix",
    "SUITESPARSE_LIBRARY_DIR": "follows the prepared KLU prefix",
    "PSE_KLU_PROVIDER_RECEIPT": "follows the native qualifier's managed KLU generation",
}
CAPABILITY_PATHS = {
    "compiler": ("CLANG_PATH", "BINDGEN_EXTRA_CLANG_ARGS"),
    "solver": ("IPOPT_DIR", "SCIPOPTDIR"),
    "klu": (
        "SUITESPARSE_INCLUDE_DIR",
        "SUITESPARSE_LIBRARY_DIR",
        "PSE_KLU_PROVIDER_RECEIPT",
    ),
    "isolation": ("PSE_ROOT_ISOLATION_DIR",),
    "uno": ("UNO_DIR", "IPOPT_DIR"),
    "petsc": ("PETSC_DIR", "IPOPT_DIR"),
}


class DeadlineOptions(TypedDict, total=False):
    """Forward an existing clock only when its caller supplied one."""

    deadline: float


def deadline_options(deadline: float | None) -> DeadlineOptions:
    return {"deadline": deadline} if deadline is not None else {}


class ScopeOwner(TypedDict):
    unit: str
    group: str
    invocation: str


class Admission(TypedDict, total=False):
    value: object
    prefix: str
    identity: Mapping[str, object]
    required: list[str]
    managed: bool
    files: dict[str, str]
    inputs: dict[str, str | None]


class RegistrationOwner(TypedDict):
    path: str
    nonce: str


class OperationRecord(TypedDict):
    version: int
    scope: ScopeOwner | None
    admissions: dict[str, Admission]
    generations: list[str]
    pid: NotRequired[int]
    start: NotRequired[str]
    python: NotRequired[str]
    handoff: NotRequired[str]
    foreground: NotRequired[bool]
    parent: NotRequired[str]
    foreground_children: NotRequired[list[str]]
    cancelled: NotRequired[bool]
    scope_inode: NotRequired[int]
    registration: NotRequired[RegistrationOwner]
    operation: NotRequired[str]
    selected_composers: NotRequired[dict[str, dict[str, object]]]
    cancel_deadline: NotRequired[float]
    cancel_signal: NotRequired[int]


Observed = TypeVar("Observed")


def write_json(path: Path, value: Mapping[str, object]) -> None:
    temporary = path.with_suffix(f".{os.getpid()}.tmp")
    descriptor = os.open(
        temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "w") as output:
        output.write(json.dumps(value, sort_keys=True) + "\n")
    temporary.chmod(0o600)
    temporary.replace(path)


def process_group(pid: int) -> str:
    for line in Path(f"/proc/{pid}/cgroup").read_text().splitlines():
        if line.startswith("0::"):
            return line[3:]
    raise ValueError("native operation requires the unified cgroup observation")


def start_identity(pid: int) -> str:
    return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19]


class DeadlineExpiredError(ValueError):
    """An offered absolute native admission/control clock expired."""


def original_cancellation_clock(value: object) -> float:
    """Narrow an existing cancellation deadline without choosing a new clock."""
    if (
        isinstance(value, (int, float))
        and not isinstance(value, bool)
        and math.isfinite(value)
    ):
        return value
    raise ValueError("invalid original cancellation clock")


def admission_remaining(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise DeadlineExpiredError("native operation original admission clock expired")
    return remaining


def manager_control_environment() -> dict[str, str]:
    # The foreground observer can run outside the composed child environment.
    # Observation, stop and mandatory kill must address the same user manager.
    from scripts import pse_env  # noqa: PLC0415 -- reciprocal environment boundary

    return {**os.environ, **pse_env.manager_environment()}


def _manager_detail(value: str | bytes | None, limit: int) -> str:
    raw = value.encode() if isinstance(value, str) else value or b""
    suffix = " [truncated]" if len(raw) > limit else ""
    return raw[:limit].decode(errors="replace").strip() + suffix


def _manager_endpoint(environment: Mapping[str, str]) -> str:
    return ", ".join(
        f"{name}={_manager_detail(environment.get(name), 256)}"
        for name in ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS")
    )


class ManagerObservationTimeout(subprocess.TimeoutExpired):
    """A manager timeout retaining its selected endpoint and bounded diagnostics."""

    def __init__(self, error: subprocess.TimeoutExpired, context: str) -> None:
        super().__init__(
            error.cmd, error.timeout, stderr=_manager_detail(error.stderr, 4096)
        )
        self.context = context

    def __str__(self) -> str:
        return (
            f"{super().__str__()} ({self.context}); stderr: {self.stderr or '(empty)'}"
        )


def manager_observation(*arguments: str, deadline: float | None = None) -> str:
    """Read the selected user manager without preparation or execution admission."""
    environment = manager_control_environment()
    context = _manager_endpoint(environment)
    timeout = 10 if deadline is None else min(10, admission_remaining(deadline))
    try:
        result = subprocess.run(
            ["systemctl", "--user", *arguments],
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=environment,
        )
    except subprocess.TimeoutExpired as error:
        if deadline is not None and time.monotonic() >= deadline:
            raise DeadlineExpiredError(
                "native operation original observation clock expired"
            ) from error
        raise ManagerObservationTimeout(error, context) from error
    except OSError as error:
        raise ValueError(
            f"systemctl {arguments[0]} unavailable ({context}): {type(error).__name__}: {error}"
        ) from error
    if deadline is not None:
        admission_remaining(deadline)
    if result.returncode:
        detail = _manager_detail(result.stderr, 4096) or "(empty)"
        raise ValueError(
            f"systemctl {arguments[0]} failed ({result.returncode}; {context}); stderr: {detail}"
        )
    return result.stdout


def unit_observation(
    unit: str, *, deadline: float | None = None, properties: tuple[str, ...] = ()
) -> dict[str, str]:
    requested = tuple(
        dict.fromkeys(
            ("InvocationID", "ControlGroup", "LoadState", "ActiveState", *properties)
        )
    )
    value = manager_observation(
        "show", *(f"--property={key}" for key in requested), unit, deadline=deadline
    )
    context = _manager_endpoint(manager_control_environment())
    fields: dict[str, str] = {}
    for line in value.splitlines():
        if "=" not in line:
            raise ValueError(f"malformed systemctl properties for {unit} ({context})")
        key, observed = line.split("=", 1)
        if key in fields:
            raise ValueError(
                f"duplicate systemctl property {key} for {unit} ({context})"
            )
        fields[key] = observed
    if any(key not in fields for key in requested) or not all(
        fields.get(key) for key in ("LoadState", "ActiveState")
    ):
        raise ValueError(f"incomplete systemctl properties for {unit} ({context})")
    return fields


def scope_owner(*, deadline: float | None = None) -> ScopeOwner | None:
    """Only real scope membership and its invocation grant operation reuse."""
    if deadline is not None:
        admission_remaining(deadline)
    try:
        group = process_group(os.getpid())
        unit = next(
            part
            for part in Path(group).parts
            if re.fullmatch(
                r"(?:pse-native-[a-f0-9]{32}\.scope|pse-surreal-worker-[a-f0-9]{16}-[0-9]+\.(?:scope|service))",
                part,
            )
        )
        observed = unit_observation(unit, **deadline_options(deadline))
        control = observed["ControlGroup"]
        invocation = observed["InvocationID"]
        if (
            observed["LoadState"] != "loaded"
            or not re.fullmatch(r"[a-f0-9]{32}", invocation)
            or not control.startswith("/")
            or control == "/"
            or ".." in Path(control).parts
            or not (group == control or group.startswith(control + "/"))
        ):
            return None
    except DeadlineExpiredError:
        raise
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        if deadline is not None:
            admission_remaining(deadline)
        return None

    else:
        if deadline is not None:
            admission_remaining(deadline)
        return {"unit": unit, "group": control, "invocation": invocation}


def populated(group: str) -> bool:
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise ValueError("invalid native scope group")
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    if not directory.exists():
        return False
    try:
        fields = dict(
            line.split()
            for line in (directory / "cgroup.events").read_text().splitlines()
        )
    except FileNotFoundError:
        if not directory.exists():
            return False
        raise
    if fields.get("populated") not in {"0", "1"}:
        raise ValueError("native scope population unavailable")
    return fields["populated"] == "1"


class DrainPhase(Enum):
    COMPLETE = "complete"
    POPULATED = "populated"
    ACTIVE = "active"
    MISMATCH = "mismatch"
    UNAVAILABLE = "unavailable"


@dataclass(frozen=True)
class DrainObservation:
    owner: Mapping[str, object] | None
    phase: DrainPhase
    cause: str
    observed: Mapping[str, str] | None = None

    @property
    def complete(self) -> bool:
        return self.phase is DrainPhase.COMPLETE

    def diagnostic(self) -> str:
        return f"native drain {self.phase.value}: {self.cause}; selected={self.owner}; observed={self.observed}"

    def __bool__(self) -> bool:
        raise TypeError("Consume DrainObservation.complete or its phase explicitly")


def drained(
    record: Mapping[str, object], *, deadline: float | None = None
) -> DrainObservation:
    """Observe this native owner; unknown premises keep its obligations pinned."""
    owner = record.get("scope")
    if not isinstance(owner, dict) or any(
        not isinstance(owner.get(key), str) for key in ("unit", "group", "invocation")
    ):
        return DrainObservation(None, DrainPhase.UNAVAILABLE, "missing scope identity")
    selected = dict(owner)
    try:
        if deadline is not None:
            admission_remaining(deadline)
        inode = record.get("scope_inode")
        if type(inode) is int:
            from scripts import host_admission  # noqa: PLC0415 -- bound kernel lifetime

            if host_admission.group_identity(owner["group"]) != inode:
                return DrainObservation(
                    selected, DrainPhase.COMPLETE, "bound kernel scope lifetime ended"
                )
        if populated(owner["group"]):
            return DrainObservation(
                selected, DrainPhase.POPULATED, "owned descendants remain"
            )
        observed = unit_observation(owner["unit"], **deadline_options(deadline))
        if observed.get("LoadState") == "not-found":
            # A group can repopulate while the manager is being observed.
            if populated(owner["group"]):
                return DrainObservation(
                    selected,
                    DrainPhase.POPULATED,
                    "group populated during observation",
                    observed,
                )
            return DrainObservation(
                selected,
                DrainPhase.COMPLETE,
                "unit absent and owned group empty",
                observed,
            )
        if observed.get("LoadState") != "loaded" or any(
            key not in observed
            for key in ("InvocationID", "ControlGroup", "ActiveState")
        ):
            return DrainObservation(
                selected,
                DrainPhase.UNAVAILABLE,
                "unit observation lacks required identity or phase",
                observed,
            )
        if observed.get("InvocationID") != owner["invocation"] or observed.get(
            "ControlGroup"
        ) not in {owner["group"], ""}:
            return DrainObservation(
                selected,
                DrainPhase.MISMATCH,
                "unit identity differs from selected owner",
                observed,
            )
        if observed.get("ActiveState") not in {"inactive", "failed"}:
            return DrainObservation(
                selected, DrainPhase.ACTIVE, "owned unit is not terminal", observed
            )
        if populated(owner["group"]):
            return DrainObservation(
                selected,
                DrainPhase.POPULATED,
                "group populated during observation",
                observed,
            )
        return DrainObservation(
            selected,
            DrainPhase.COMPLETE,
            "owned unit terminal and group empty",
            observed,
        )
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        return DrainObservation(
            selected, DrainPhase.UNAVAILABLE, f"{type(error).__name__}: {error}"
        )


def _record(path: Path) -> OperationRecord:
    value = json.loads(path.read_text())
    if value.get("version") != VERSION or not isinstance(value.get("admissions"), dict):
        raise ValueError("unknown native operation admission record")
    return cast("OperationRecord", value)


def owner_record(*, deadline: float | None = None, reserve: float = 0) -> Path | None:
    if deadline is not None:
        admission_remaining(deadline)
    marker = os.environ.get(MARKER)
    if not marker:
        return None
    path = Path(marker)
    try:
        from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle

        directory = cache.cache_root(dict(os.environ)) / ".operations"
        if (
            not path.is_absolute()
            or path.parent != directory
            or path != path.resolve()
            or any(parent.is_symlink() for parent in path.parents)
        ):
            return None
        metadata = path.lstat()
        parent = directory.lstat()
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.getuid()
            or stat.S_IMODE(metadata.st_mode) != 0o600
            or parent.st_uid != os.getuid()
            or stat.S_IMODE(parent.st_mode) != 0o700
        ):
            return None
        record = _record(path)
        if deadline is not None and record.get("cancel_deadline") is not None:
            original = record["cancel_deadline"]
            if type(original) not in {int, float}:
                raise ValueError("invalid original cancellation clock")  # noqa: TRY301 -- invalid owner metadata is refused by the enclosing probe
            deadline = min(deadline, original - reserve)
            admission_remaining(deadline)
        owner = record.get("scope")
        if owner:
            return path if scope_owner(**deadline_options(deadline)) == owner else None
        pid = record["pid"]
        if start_identity(pid) != record["start"]:
            return None
        descendant = os.getpid()
        while descendant > 1:
            if deadline is not None:
                admission_remaining(deadline)
            if descendant == pid:
                return path
            status = Path(f"/proc/{descendant}/status").read_text().splitlines()
            descendant = int(
                next(line.split()[1] for line in status if line.startswith("PPid:"))
            )
    except DeadlineExpiredError:
        raise
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        if deadline is not None:
            admission_remaining(deadline)
        return None
    if deadline is not None:
        admission_remaining(deadline)
    return None


def current() -> Path | None:
    path = owner_record()
    return path if path is not None and _record(path).get("scope") else None


@contextlib.contextmanager
def record_lock(
    path: Path, *, deadline: float | None = None
) -> Generator[None, None, None]:
    # A synchronous cancellation handler must not reacquire our own flock.
    previous = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGTERM, signal.SIGINT})
    try:
        if deadline is not None:
            admission_remaining(deadline)
        with path.with_suffix(".lock").open("a") as lock:
            if deadline is None:
                fcntl.flock(lock, fcntl.LOCK_EX)
            else:
                while True:
                    admission_remaining(deadline)
                    try:
                        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                        break
                    except BlockingIOError:
                        time.sleep(min(0.02, admission_remaining(deadline)))
                admission_remaining(deadline)
            yield
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous)


def remembered(key: str) -> Admission | None:
    path = current()
    if path is None:
        return None
    with record_lock(path):
        return _record(path)["admissions"].get(key)


def admit(key: str, value: Admission) -> None:
    path = owner_record()
    if path is None:
        return
    with record_lock(path):
        record = _record(path)
        prior = record["admissions"].get(key)
        if prior is not None and prior != value:
            raise ValueError("native operation admission changed inside its lifetime")
        record["admissions"][key] = value
        write_json(path, record)


def observe(key: str, producer: Callable[[], Observed]) -> Observed:
    """Reuse input observation only inside the verified operation owner."""
    prior = remembered("observation:" + key)
    if prior is not None:
        return cast("Observed", prior["value"])
    value = producer()
    admit("observation:" + key, {"value": value})
    return value


def pin(base: Path, generation: Path) -> None:
    """Register before selection unlock, including conservative fallback."""
    path = owner_record()
    if path is None:
        directory = base / ".operations"
        directory.mkdir(parents=True, exist_ok=True)
        directory.chmod(0o700)
        path = directory / f"unscoped-{uuid.uuid4().hex}.json"
        write_json(
            path,
            {
                "version": VERSION,
                "scope": None,
                "admissions": {},
                "generations": [str(generation)],
            },
        )
        return
    with record_lock(path):
        record = _record(path)
        if str(generation) not in record["generations"]:
            record["generations"].append(str(generation))
            write_json(path, record)


def pinned_generations(base: Path) -> set[str] | None:
    """Observe live pins once; discard only authentically settled operation owners.

    Publication coordination held by the caller excludes a new selection of the
    generation being reclaimed. Record exclusion protects handoff/pin mutations.
    Unknown records conservatively refuse reclamation. Settled records no longer
    pin generation bytes; exact cancellation/registration owners may still need
    their proof until those owners retire.
    """
    pinned: set[str] = set()
    for path in (base / ".operations").glob("*.json"):
        try:
            with record_lock(path):
                record = _record(path)
                generations = record.get("generations")
                if not isinstance(generations, list) or any(
                    not isinstance(generation, str) for generation in generations
                ):
                    return None
                if drained(record).complete:
                    # Allocation guards refer to these exact acknowledgments.
                    # Retaining metadata does not keep drained generation bytes
                    # pinned, and avoids a collector racing host release.
                    if record.get("selected_composers"):
                        continue
                    # The live parent may still need this exact child binding
                    # during cancellation. Its settled child no longer pins bytes.
                    if registration_pending(record):
                        continue
                    if (
                        record.get("foreground") is True
                        and foreground_parent(record, path).exists()
                    ):
                        continue
                    path.unlink()
                    path.with_suffix(".lock").unlink(missing_ok=True)
                else:
                    pinned.update(generations)
        except FileNotFoundError:
            # Another collector finished this exact operation under exclusion.
            continue
        except (OSError, ValueError, KeyError):
            return None
    return pinned


def foreground_parent(record: OperationRecord, path: Path) -> Path:
    name = record.get("parent")
    if isinstance(name, str):
        parent = Path(name)
        if parent.parent == path.parent and re.fullmatch(
            r"[a-f0-9]{32}\.json", parent.name
        ):
            return parent
    raise ValueError("invalid foreground parent association")


def foreground_children(record: OperationRecord) -> list[str]:
    children = record.get("foreground_children", [])
    if isinstance(children, list):
        return children
    raise ValueError("invalid foreground child associations")


def registration_pending(record: OperationRecord) -> bool:
    """Settled launch proof belongs to its matching unresolved admission owner."""
    selected = record.get("registration")
    if selected is None:
        return False
    if (
        not isinstance(selected, dict)
        or not isinstance(selected.get("path"), str)
        or not isinstance(selected.get("nonce"), str)
        or not Path(selected["path"]).is_absolute()
    ):
        raise ValueError("invalid native registration association")
    try:
        pending = json.loads(Path(selected["path"]).read_text())
    except FileNotFoundError:
        return False
    if not isinstance(pending, dict):
        raise ValueError("invalid native registration owner")  # noqa: TRY004 -- malformed persisted records use ValueError
    return pending.get("nonce") == selected["nonce"]


def prepare_handoff(
    expected_unit: str,
    *,
    foreground: bool = False,
    reservation: tuple[Path, str] | None = None,
    deadline: float | None = None,
) -> Path | None:
    """Pin before launch; a dead launcher leaves a conservative pending guard."""
    parent = owner_record(**deadline_options(deadline))
    if parent is None and reservation is None:
        return None
    if parent is None:
        from scripts import native_cache as cache  # noqa: PLC0415 -- reciprocal owner

        directory = cache.cache_root(dict(os.environ)) / ".operations"
        if deadline is not None:
            admission_remaining(deadline)
        directory.mkdir(parents=True, exist_ok=True, mode=0o700)
        directory.chmod(0o700)
    else:
        directory = parent.parent
    with (
        record_lock(parent, deadline=deadline)
        if parent is not None
        else contextlib.nullcontext()
    ):
        if deadline is not None:
            admission_remaining(deadline)
        registered = _record(parent) if parent is not None else None
        if foreground and (registered is None or parent is None):
            raise ValueError("native foreground parent is unavailable")
        if foreground and registered is not None and registered.get("cancelled"):
            raise ValueError("native foreground parent is cancelled")
        path = directory / f"handoff-{uuid.uuid4().hex}.json"
        record: OperationRecord = {
            "version": VERSION,
            "scope": None,
            "handoff": expected_unit,
            "admissions": {},
            "generations": registered["generations"][:]
            if registered is not None
            else [],
        }
        if reservation is not None:
            selected, nonce = reservation
            if not selected.is_absolute() or not nonce:
                raise ValueError("native handoff requires an exact registration owner")
            pending = json.loads(selected.read_text())
            if not isinstance(pending, dict) or pending.get("nonce") != nonce:
                raise ValueError("native registration owner changed before handoff")
            record["registration"] = {"path": str(selected), "nonce": nonce}
        children: list[str] = []
        if foreground and registered is not None:
            if not re.fullmatch(r"pse-native-[a-f0-9]{32}\.scope", expected_unit):
                raise ValueError("foreground handoff requires an observer scope")
            record.update(foreground=True, parent=str(parent))
            children = foreground_children(registered)
        if deadline is not None:
            admission_remaining(deadline)
        write_json(path, record)
        if foreground and registered is not None and parent is not None:
            registered["foreground_children"] = [*children, str(path)]
            if deadline is not None:
                admission_remaining(deadline)
            write_json(parent, registered)
    return path


def bind_handoff(path: Path) -> None:
    """Only the child can prove its kernel membership and invocation generation."""
    owner = scope_owner()
    if owner is None:
        raise ValueError("native child handoff requires an authentic managed scope")
    initial = _record(path)
    if type(initial.get("foreground", False)) is not bool:
        raise ValueError("invalid foreground handoff marker")
    parent = (
        foreground_parent(initial, path) if initial.get("foreground") is True else None
    )
    # Parent -> child is also cancellation's lock order. Binding cannot admit
    # payload after the parent's cancellation fence, even before child IPC.
    with record_lock(parent) if parent is not None else contextlib.nullcontext():
        registered = _record(parent) if parent is not None and parent.exists() else None
        with record_lock(path):
            record = _record(path)
            if record.get("handoff") != owner["unit"] or record.get("scope") not in (
                None,
                owner,
            ):
                raise ValueError(
                    "native child handoff differs from its registered scope"
                )
            record["scope"] = owner
            operation = owner_record()
            if operation is None and record.get("foreground"):
                raise ValueError("native handoff lost its actual operation")
            if operation is not None:
                record["operation"] = str(operation)
            if parent is not None or record.get("registration") is not None:
                from scripts import host_admission  # noqa: PLC0415 -- reciprocal owner

                if parent is not None and (
                    record.get("parent") != str(parent)
                    or (
                        registered is not None
                        and str(path) not in foreground_children(registered)
                    )
                ):
                    raise ValueError("native foreground parent association changed")
                inode = host_admission.group_identity(owner["group"])
                if inode is None or record.get("scope_inode", inode) != inode:
                    raise ValueError("native handoff scope generation changed")
                record["scope_inode"] = inode
                if parent is not None and (
                    registered is None or registered.get("cancelled")
                ):
                    record["cancelled"] = True
            write_json(path, record)
            if record.get("foreground") and record.get("cancelled"):
                raise ValueError("native foreground handoff is cancelled")


def fence_foreground_handoff(
    name: object,
    parent: Path,
    *,
    deadline: float | None = None,
    signum: int | None = None,
    cancellation_deadline: float | None = None,
) -> OperationRecord:
    if isinstance(name, str):
        path = Path(name)
        if path.parent == parent.parent and re.fullmatch(
            r"handoff-[a-f0-9]{32}\.json", path.name
        ):
            with record_lock(path, deadline=deadline):
                record = _record(path)
                if record.get("foreground") is not True or record.get("parent") != str(
                    parent
                ):
                    raise ValueError("foreground handoff parent changed")
                record["cancelled"] = True
                original = (
                    deadline if cancellation_deadline is None else cancellation_deadline
                )
                if original is not None and original < record.get(
                    "cancel_deadline", float("inf")
                ):
                    record["cancel_deadline"] = original
                    if signum is not None:
                        record["cancel_signal"] = signum
                elif signum is not None:
                    record.setdefault("cancel_signal", signum)
                write_json(path, record)
            return record
    raise ValueError("invalid foreground handoff association")


def stop_foreground_handoff(
    record: OperationRecord, *, deadline: float | None = None
) -> None:
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal owner

    owner = record.get("scope", False)
    if owner is None:
        # A late child still binds its actual lifetime, then refuses payload.
        return
    if not isinstance(owner, dict) or any(
        not isinstance(owner.get(key), str) for key in ("unit", "group", "invocation")
    ):
        raise ValueError("invalid foreground scope identity")
    options = deadline_options(deadline)
    initial = drained(record, **options)
    if initial.complete:
        return
    if initial.phase in {DrainPhase.UNAVAILABLE, DrainPhase.MISMATCH}:
        raise ValueError(initial.diagnostic() + "; foreground cancellation refused")
    observed = unit_observation(owner["unit"], **options)
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("InvocationID") != owner["invocation"]
        or observed.get("ControlGroup") != owner["group"]
        or type(record.get("scope_inode")) is not int
        or host_admission.group_identity(owner["group"]) != record["scope_inode"]
    ):
        raise ValueError(
            f"foreground observer scope identity changed; selected={owner}; observed={observed}"
        )
    # Stop waits for this authenticated scope. Its native handler first settles
    # any nested foreground handoffs before its own scope kill.
    subprocess.run(
        ["systemctl", "--user", "stop", owner["unit"]],
        check=True,
        timeout=10 if deadline is None else min(10, admission_remaining(deadline)),
        env=manager_control_environment(),
    )
    completion = drained(record, **options)
    if not completion.complete:
        raise ValueError(
            completion.diagnostic() + "; foreground observer scope did not drain"
        )


def retire_unlaunched_handoff(
    path: Path, unit: str, *, deadline: float
) -> tuple[float, int] | None:
    """Retire only a fresh handoff whose creator has not attempted Popen."""
    parent = owner_record(deadline=deadline)
    if parent is None:
        raise ValueError("unlaunched handoff lost its native parent")
    with record_lock(parent, deadline=deadline):
        registered = _record(parent)
        children = foreground_children(registered)
        if str(path) not in children:
            raise ValueError("unlaunched handoff lost its registered association")
        with record_lock(path, deadline=deadline):
            selected = _record(path)
            if (
                foreground_parent(selected, path) != parent
                or selected.get("foreground") is not True
                or selected.get("handoff") != unit
                or "scope" not in selected
                or selected["scope"] is not None
                or selected.get("operation") is not None
                or selected.get("admissions") != {}
                or selected.get("registration") is not None
            ):
                raise ValueError("unlaunched handoff identity or phase changed")
            original: tuple[float, int] | None = None
            for record in (registered, selected):
                if record.get("cancel_deadline") is not None:
                    clock = original_cancellation_clock(record["cancel_deadline"])
                    deadline = min(deadline, clock)
                    if "cancel_signal" in record:
                        signum = record["cancel_signal"]
                        if signum not in {signal.SIGINT, signal.SIGTERM}:
                            raise ValueError("invalid unlaunched cancellation signal")
                        if original is None or clock < original[0]:
                            original = clock, signum
        # Binding takes parent -> child exclusion too. The existing fence
        # prevents any delayed entry from admitting payload before retirement.
        fenced = fence_foreground_handoff(str(path), parent, deadline=deadline)
        with record_lock(path, deadline=deadline):
            if _record(path) != fenced:
                raise ValueError("unlaunched handoff changed after its fence")
            registered["foreground_children"] = [
                name for name in children if name != str(path)
            ]
            write_json(parent, registered)
            path.unlink()
        return original


def cancel_foreground_handoffs(
    *,
    deadline: float | None = None,
    signum: int | None = None,
    drain_deadline: float | None = None,
    cancellation_deadline: float | None = None,
) -> None:
    """Settle only the foreground observers explicitly owned by this operation."""
    parent = owner_record(**deadline_options(deadline))
    if parent is None:
        return
    failures: list[str] = []
    bound: list[OperationRecord] = []
    with record_lock(parent, deadline=deadline):
        registered = _record(parent)
        registered["cancelled"] = True
        original = deadline if cancellation_deadline is None else cancellation_deadline
        if original is not None and original < registered.get(
            "cancel_deadline", float("inf")
        ):
            registered["cancel_deadline"] = original
            if signum is not None:
                registered["cancel_signal"] = signum
        elif signum is not None:
            registered.setdefault("cancel_signal", signum)
        write_json(parent, registered)
        children = foreground_children(registered)[:]
        for name in children:
            try:
                bound.append(
                    fence_foreground_handoff(
                        name,
                        parent,
                        deadline=deadline,
                        signum=signum,
                        cancellation_deadline=(
                            original if cancellation_deadline is not None else None
                        ),
                    )
                )
            except (OSError, ValueError, KeyError) as error:
                failures.append(str(error))
    for record in bound:
        try:
            selected_deadline = deadline if drain_deadline is None else drain_deadline
            stop_foreground_handoff(
                record,
                **(
                    {"deadline": selected_deadline}
                    if selected_deadline is not None
                    else {}
                ),
            )
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            failures.append(str(error))
    if failures:
        raise ValueError("foreground cancellation incomplete: " + "; ".join(failures))


def composer_registrations(
    record: Mapping[str, object],
) -> dict[str, dict[str, object]]:
    selected = record.get("selected_composers", {})
    if not isinstance(selected, dict) or any(
        not isinstance(nonce, str)
        or not re.fullmatch(r"[a-f0-9]{32}", nonce)
        or not isinstance(value, dict)
        for nonce, value in selected.items()
    ):
        raise ValueError("invalid selected terminal associations")
    return selected


def register_composer(
    invocation: Path, *, deadline: float | None = None
) -> tuple[Path, str, str] | None:
    """Publish only a selected live composer under its authentic operation fence.

    The caller installs interruption handling before this transition. The outer
    owner only requests and observes completion; it never composes test outcomes.
    """
    from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
        test_resources as resources,
    )

    deadline = resources.resource_store.clock(deadline)
    parent = owner_record(deadline=deadline)
    if parent is None or scope_owner(deadline=deadline) is None:
        return None

    declaration = resources.read_invocation(invocation)
    nonce = declaration["nonce"]
    registry = resources.registry().resolve()
    if (
        invocation != registry / "invocations" / f"{nonce}.json"
        or invocation != invocation.resolve()
    ):
        raise ValueError("selected terminal requires its exact private invocation")
    token = uuid.uuid4().hex
    with record_lock(parent, deadline=deadline):
        record = _record(parent)
        if record.get("cancelled"):
            raise ValueError("native operation cancelled before selected payload")
        composers = composer_registrations(record)
        if nonce in composers:
            raise ValueError("selected invocation already has a terminal composer")
        register_terminal_guard(parent, nonce, token, deadline=deadline)
        composers[nonce] = {
            "token": token,
            "pid": os.getpid(),
            "start": start_identity(os.getpid()),
            "scope": record["scope"],
            "invocation": str(invocation),
            "registry": str(registry),
            "phase": "registered",
        }
        record["selected_composers"] = composers
        write_json(parent, record)
    return parent, nonce, token


def register_terminal_guard(
    parent: Path, nonce: str, token: str, *, deadline: float | None = None
) -> None:
    from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
        host_admission as host,
    )

    allocation = host.inherit(dict(os.environ), deadline=deadline)
    if allocation is None:
        raise ValueError("selected terminal lacks its host allocation")
    owner = _record(parent)["scope"]
    if owner is None:
        raise ValueError("selected terminal lacks its authentic native scope")
    with host.allocation_metadata(
        allocation.directory, deadline=deadline, durable=True
    ) as ledger:
        allocated = ledger["owners"].get(allocation.nonce)
        binding = (
            allocated["units"].get(owner["unit"]) if allocated is not None else None
        )
        if (
            allocated is None
            or not binding
            or binding.get("group") != owner["group"]
            or binding.get("invocation") != owner["invocation"]
            or type(binding.get("inode")) is not int
        ):
            raise ValueError("selected terminal is outside its bound allocation")
        guards = allocated.setdefault("terminal_guards", {})
        if nonce in guards:
            raise ValueError("selected terminal already has an allocation guard")
        guards[nonce] = {
            "operation": str(parent),
            "token": token,
            "scope": owner,
            "binding": dict(binding),
            "phase": "registered",
        }


def terminal_guard_complete(
    nonce: str, guard: object, *, deadline: float | None = None
) -> bool:
    """An exact acknowledgment is an independent premise of host drain."""
    try:
        from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
            resource_store,
        )

        if not isinstance(guard, dict) or not re.fullmatch(r"[a-f0-9]{32}", nonce):
            return False
        scope = guard.get("scope")
        if (
            not isinstance(scope, dict)
            or not re.fullmatch(
                r"(?:pse-native-[a-f0-9]{32}\.scope|pse-surreal-worker-[a-f0-9]{16}-[0-9]+\.(?:scope|service))",
                str(scope.get("unit", "")),
            )
            or not isinstance(scope.get("group"), str)
            or not scope["group"].startswith("/")
            or not re.fullmatch(r"[a-f0-9]{32}", str(scope.get("invocation", "")))
            or not re.fullmatch(r"[a-f0-9]{32}", str(guard.get("token", "")))
        ):
            return False
        path = Path(str(guard["operation"]))
        if (
            not path.is_absolute()
            or path != path.resolve()
            or path.parent.name != ".operations"
            or not re.fullmatch(r"[a-f0-9]{32}\.json", path.name)
        ):
            return False
        if guard.get("phase") == "acknowledged":
            # The existing admitted row records only verification of immutable
            # terminal identities, never another copy of outcomes. Temporary
            # evidence retirement cannot erase an acknowledgment already proved.
            return all(
                re.fullmatch(r"[a-f0-9]{64}", str(guard.get(key, ""))) is not None
                for key in ("terminal_identity", "evidence_identity")
            )
        record = resource_store.read(path)
        if record.get("version") != VERSION or record.get("scope") != guard["scope"]:
            return False
        composer = composer_registrations(record).get(nonce)
        if (
            composer is None
            or composer.get("token") != guard["token"]
            or composer.get("phase") != "complete"
        ):
            return False
        registry = Path(str(composer["registry"]))
        invocation = registry / "invocations" / f"{nonce}.json"
        if (
            composer.get("invocation") != str(invocation)
            or invocation != invocation.resolve()
        ):
            return False
        identity, _terminal = resource_store.FileStore(registry).terminal(
            nonce, deadline=deadline
        )
        evidence = resource_store.read(invocation.with_suffix(".terminal.json"))
        return composer.get("terminal_identity") == identity and composer.get(
            "evidence_identity"
        ) == resource_store.digest(evidence)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError):
        return False


def composer_deadline(registration: tuple[Path, str, str] | None) -> float | None:
    if registration is None:
        return None
    value = _record(registration[0]).get("cancel_deadline")
    if value is None:
        inherited = inherited_cancellation(deadline=time.monotonic() + 10)
        value = inherited[0] if inherited is not None else None
    if value is not None and type(value) not in {int, float}:
        raise ValueError("invalid original cancellation clock")
    return value


def composer_signal(registration: tuple[Path, str, str] | None) -> int | None:
    if registration is None:
        return None
    value = _record(registration[0]).get("cancel_signal")
    if value is None:
        inherited = inherited_cancellation(
            deadline=composer_deadline(registration) or time.monotonic() + 10
        )
        value = inherited[1] if inherited is not None else None
    if value is not None and value not in {signal.SIGINT, signal.SIGTERM}:
        raise ValueError("invalid original cancellation signal")
    return value


def note_composer_cancellation(
    signum: int, deadline: float, *, control_deadline: float | None = None
) -> tuple[float, int] | None:
    """Carry the earliest request through this Operation and its bound parent handoff."""
    control_deadline = (
        deadline if control_deadline is None else min(deadline, control_deadline)
    )
    reserve = deadline - control_deadline
    parent = owner_record(
        deadline=control_deadline, **({"reserve": reserve} if reserve else {})
    )
    if parent is None:
        return None
    # Authentication can discover an earlier clock on this Operation even
    # without an inherited handoff. Carry it into the first exclusion wait.
    known = _record(parent)
    if known.get("cancel_deadline", float("inf")) <= deadline:
        deadline, signum = known["cancel_deadline"], known["cancel_signal"]
        control_deadline = min(control_deadline, deadline - reserve)
    inherited = inherited_cancellation(
        deadline=control_deadline, **({"reserve": reserve} if reserve else {})
    )
    if inherited is not None and inherited[0] <= deadline:
        deadline, signum = inherited
        control_deadline = min(control_deadline, deadline - reserve)
    with record_lock(parent, deadline=control_deadline):
        record = _record(parent)
        if deadline < record.get("cancel_deadline", float("inf")):
            record.update(cancel_deadline=deadline, cancel_signal=signum)
        else:
            deadline, signum = record["cancel_deadline"], record["cancel_signal"]
            control_deadline = min(control_deadline, deadline - reserve)
        write_json(parent, record)
    name = os.environ.get("PSE_NATIVE_HANDOFF")
    if name:
        path = Path(name)
        selected = _record(path)
        owner = scope_owner(deadline=control_deadline)
        if selected.get("scope") != owner or selected.get("operation") != str(parent):
            raise ValueError("cancellation parent handoff changed")
        ancestor = foreground_parent(selected, path)
        with record_lock(ancestor, deadline=control_deadline):
            if str(path) not in foreground_children(_record(ancestor)):
                raise ValueError("cancellation lost its outer foreground association")
            with record_lock(path, deadline=control_deadline):
                selected = _record(path)
                if deadline < selected.get("cancel_deadline", float("inf")):
                    selected.update(
                        cancelled=True, cancel_deadline=deadline, cancel_signal=signum
                    )
                    write_json(path, selected)
    return deadline, signum


def inherited_cancellation(
    *, deadline: float, reserve: float = 0
) -> tuple[float, int] | None:
    """An exact bound foreground handoff can carry its original control clock."""
    name = os.environ.get("PSE_NATIVE_HANDOFF")
    if not name:
        return None
    path = Path(name)
    child = _record(path)
    if (
        not child.get("cancelled")
        or "cancel_deadline" not in child
        or "cancel_signal" not in child
    ):
        return None
    if type(child["cancel_deadline"]) not in {int, float}:
        raise ValueError("invalid inherited original cancellation clock")
    deadline = min(deadline, child["cancel_deadline"] - reserve)
    parent = foreground_parent(child, path)
    with record_lock(parent, deadline=deadline):
        registered = _record(parent)
        owner = scope_owner(deadline=deadline)
        from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
            host_admission,
        )

        if (
            str(path) not in foreground_children(registered)
            or child.get("foreground") is not True
            or owner is None
            or child.get("scope") != owner
            or child.get("handoff") != owner["unit"]
            or child.get("scope_inode") != host_admission.group_identity(owner["group"])
            or type(child["cancel_deadline"]) not in {int, float}
            or child["cancel_signal"] not in {signal.SIGINT, signal.SIGTERM}
        ):
            raise ValueError("original foreground cancellation authority changed")
    return child["cancel_deadline"], child["cancel_signal"]


def acknowledge_composer(registration: tuple[Path, str, str] | None) -> None:
    if registration is None:
        return
    from scripts import resource_store  # noqa: PLC0415 -- durable terminal evidence

    parent, nonce, token = registration
    deadline = composer_deadline(registration)
    with record_lock(parent, deadline=deadline):
        record = _record(parent)
        composer = composer_registrations(record).get(nonce)
        if (
            composer is None
            or composer.get("token") != token
            or composer.get("pid") != os.getpid()
            or composer.get("start") != start_identity(os.getpid())
        ):
            raise ValueError("selected terminal composer identity changed")
        invocation = Path(str(composer["invocation"]))
        store = resource_store.FileStore(Path(str(composer["registry"])))
        identity, _terminal = store.terminal(nonce, deadline=deadline)
        evidence = resource_store.read(invocation.with_suffix(".terminal.json"))
        evidence_identity = resource_store.digest(evidence)
        composer.update(
            phase="complete",
            terminal_identity=identity,
            evidence_identity=evidence_identity,
        )
        write_json(parent, record)
        acknowledge_terminal_guard(
            parent,
            nonce,
            token,
            record["scope"],
            identity,
            evidence_identity,
            deadline=deadline,
        )


def acknowledge_terminal_guard(
    parent: Path,
    nonce: str,
    token: str,
    scope: object,
    terminal_identity: str,
    evidence_identity: str,
    *,
    deadline: float | None = None,
) -> None:
    from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
        host_admission as host,
    )

    allocation = host.inherit(dict(os.environ), deadline=deadline)
    if allocation is None:
        raise ValueError("selected acknowledgment lost its host allocation")
    with host.allocation_metadata(
        allocation.directory, deadline=deadline, durable=True
    ) as ledger:
        allocated = ledger["owners"].get(allocation.nonce)
        guard = (
            allocated.get("terminal_guards", {}).get(nonce)
            if allocated is not None
            else None
        )
        if (
            allocated is None
            or not isinstance(scope, dict)
            or guard is None
            or any(
                guard.get(key) != value
                for key, value in (
                    ("operation", str(parent)),
                    ("token", token),
                    ("scope", scope),
                )
            )
        ):
            raise ValueError("selected acknowledgment lost its exact host guard")
        if guard.get("binding") != allocated["units"].get(scope["unit"]):
            raise ValueError("selected acknowledgment host binding changed")
        guard.update(
            phase="acknowledged",
            terminal_identity=terminal_identity,
            evidence_identity=evidence_identity,
        )


def request_composer_completion(signum: int, *, deadline: float) -> None:
    """Request authenticated composers after child drain, before mandatory kill."""
    parent = owner_record(deadline=deadline)
    if parent is None:
        return
    with record_lock(parent, deadline=deadline):
        record = _record(parent)
        selected = {
            nonce: dict(value)
            for nonce, value in composer_registrations(record).items()
        }
    for nonce, composer in selected.items():
        admission_remaining(deadline)
        if composer.get("phase") == "complete":
            continue
        pid = composer.get("pid")
        owner = composer.get("scope")
        if (
            type(pid) is not int
            or owner != record.get("scope")
            or not isinstance(owner, dict)
            or composer.get("invocation")
            != str(
                Path(str(composer.get("registry"))) / "invocations" / f"{nonce}.json"
            )
        ):
            raise ValueError("contradictory selected terminal process authority")
        try:
            descriptor = os.pidfd_open(pid)
        except ProcessLookupError:
            with record_lock(parent, deadline=deadline):
                current = composer_registrations(_record(parent)).get(nonce)
                if (
                    current is not None
                    and current.get("phase") == "complete"
                    and current.get("token") == composer.get("token")
                ):
                    continue
            raise
        try:
            if (
                start_identity(pid) != composer.get("start")
                or process_group(pid) != owner["group"]
            ):
                raise ValueError("selected terminal process lifetime changed")
            signal.pidfd_send_signal(descriptor, signum)
        finally:
            os.close(descriptor)
    pending = set(selected)
    while pending:
        admission_remaining(deadline)
        with record_lock(parent, deadline=deadline):
            current = composer_registrations(_record(parent))
            for nonce in list(pending):
                composer = current.get(nonce)
                if composer is None or any(
                    composer.get(key) != selected[nonce].get(key)
                    for key in (
                        "token",
                        "pid",
                        "start",
                        "scope",
                        "invocation",
                        "registry",
                    )
                ):
                    raise ValueError(
                        "selected terminal association changed during cancellation"
                    )
                if composer.get("phase") == "complete":
                    from scripts import (  # noqa: PLC0415 -- reciprocal admission/resource ownership
                        resource_store,
                    )

                    evidence = resource_store.read(
                        Path(str(composer["invocation"])).with_suffix(".terminal.json")
                    )
                    identity, _terminal = resource_store.FileStore(
                        Path(str(composer["registry"]))
                    ).terminal(nonce, deadline=deadline)
                    if composer.get("terminal_identity") != identity or composer.get(
                        "evidence_identity"
                    ) != resource_store.digest(evidence):
                        raise ValueError(
                            "selected terminal acknowledgment contradicts durable evidence"
                        )
                    pending.remove(nonce)
        if pending:
            time.sleep(min(0.02, admission_remaining(deadline)))


class Operation:
    """Kernel scope owns generation pins beyond this supervisor's lifetime."""

    def __init__(self, base: Path) -> None:
        self.base = base
        self.path = base / ".operations" / f"{uuid.uuid4().hex}.json"
        self.previous: str | None = None
        self.previous_cache: str | None = None
        self.previous_python: str | None = None

    def __enter__(self) -> Self:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.parent.chmod(0o700)
        write_json(
            self.path,
            {
                "version": VERSION,
                "pid": os.getpid(),
                "start": start_identity(os.getpid()),
                "scope": scope_owner(),
                "python": sys.executable,
                "admissions": {},
                "generations": [],
            },
        )
        self.previous = os.environ.get(MARKER)
        self.previous_cache = os.environ.get("PSE_NATIVE_CACHE")
        self.previous_python = os.environ.get("PSE_NATIVE_SETUP_PYTHON")
        os.environ["PSE_NATIVE_CACHE"] = str(self.base.resolve())
        os.environ["PSE_NATIVE_SETUP_PYTHON"] = sys.executable
        os.environ[MARKER] = str(self.path)
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc: BaseException | None,
        _traceback: TracebackType | None,
    ) -> None:
        try:
            # The payload can return after its validation process was interrupted
            # without signaling this supervisor. Only the creator retires the
            # operation; nested run() calls continue borrowing its live owner.
            deadline = _record(self.path).get("cancel_deadline")
            cancel_foreground_handoffs(deadline=deadline)
        finally:
            if self.previous is None:
                os.environ.pop(MARKER, None)
            else:
                os.environ[MARKER] = self.previous
            if self.previous_cache is None:
                os.environ.pop("PSE_NATIVE_CACHE", None)
            else:
                os.environ["PSE_NATIVE_CACHE"] = self.previous_cache
            if self.previous_python is None:
                os.environ.pop("PSE_NATIVE_SETUP_PYTHON", None)
            else:
                os.environ["PSE_NATIVE_SETUP_PYTHON"] = self.previous_python
            # Do not unlink records on return or cancellation while children may survive.


def enforce(env: dict[str, str], name: str, value: str) -> None:
    """Set a native-owned value, reporting a caller value it replaces."""
    prior = env.get(name)
    if prior not in (None, "", value):
        print(
            f"pse-env: refused {name}={prior}: {AUTHORITATIVE[name]}; using {value}",
            file=sys.stderr,
        )
    env[name] = value


def environment(requested: list[str], env: dict[str, str]) -> dict[str, str]:
    timing = env.get(BUILD_TIMING)
    if timing is None:
        return _environment(requested, env)
    owner = current()
    if owner is None:
        raise ValueError(
            "setup-inclusive build timing requires an authentic native scope"
        )
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal ownership

    allocation = host_admission.inherit(env)
    if allocation is None:
        raise ValueError("setup-inclusive build timing requires its host allocation")
    receipt: dict[str, object] = {
        "version": 1,
        "requested_capabilities": requested,
        "setup_started_monotonic": time.monotonic(),
        "owner_record_path": str(owner),
        "setup_status": "running",
        "host_allocation": {
            "path": str(allocation.directory / allocation.nonce),
            "class": allocation.profile.name,
            "memory_bytes": allocation.profile.memory,
            "cores": allocation.profile.cores,
        },
    }
    with record_lock(owner):
        receipt["owner"] = _record(owner)
    write_json(Path(timing), receipt)
    try:
        configured = _environment(requested, env)
    except BaseException as error:
        receipt["setup_status"] = "failed"
        receipt["error_type"] = type(error).__name__
        raise
    else:
        receipt["setup_status"] = "passed"
        return configured
    finally:
        receipt["setup_finished_monotonic"] = time.monotonic()
        with record_lock(owner):
            receipt["owner"] = _record(owner)
        write_json(Path(timing), receipt)


def runtime_selection(
    requested: list[str], env: Mapping[str, str], *, root: Path = ROOT
) -> dict[str, str]:
    """Select runtime defaults and explicit removals without preparing a provider."""
    from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle

    result = dict(env)
    off = {name for name in result.get(OFF_MARKER, "").split(",") if name}
    for name in OVERRIDABLE:
        if result.get(name) == "off":
            off.add(name)
        elif name in result:
            off.discard(name)
    if any(kind in requested for kind in ("solver", "uno", "petsc")):
        for name, value in cache.runtime_env(
            root / "docker/solvers/Dockerfile"
        ).items():
            if name in AUTHORITATIVE:
                result[name] = value
            elif name not in off:
                result.setdefault(name, value)
    for name in THREAD_VARIABLES:
        if name not in off:
            result.setdefault(name, "1")
    for name in off:
        result.pop(name, None)
    if off:
        result[OFF_MARKER] = ",".join(sorted(off))
    else:
        result.pop(OFF_MARKER, None)
    result.pop("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER", None)
    return result


def _environment(requested: list[str], env: dict[str, str]) -> dict[str, str]:
    from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle
    from scripts import (  # noqa: PLC0415 -- owner cycle
        native_pipeline_cache as pipeline,
    )

    base = cache.cache_root(env)
    result = (
        cache.compiler_env(env)
        if any(
            kind in requested
            for kind in ("compiler", "klu", "isolation", "uno", "petsc")
        )
        else env.copy()
    )
    selected_runtime = runtime_selection(requested, result)
    for name in AUTHORITATIVE:
        if name in selected_runtime and selected_runtime[name] != result.get(name):
            enforce(result, name, selected_runtime[name])
    result = selected_runtime
    if "solver" in requested or "uno" in requested or "petsc" in requested:
        if result.get("IPOPT_DIR"):
            prefix = Path(result["IPOPT_DIR"])
            cache.admit_external(prefix, cache.SOLVER_FILES)
        else:
            prefix = cache.solver(base)
        result["IPOPT_DIR"] = str(prefix)
        enforce(result, "SCIPOPTDIR", str(prefix))
        result["LD_LIBRARY_PATH"] = cache.prepend_library_path(
            str(prefix / "lib"), result
        )
    if "klu" in requested:
        include_value = result.get("SUITESPARSE_INCLUDE_DIR")
        library_value = result.get("SUITESPARSE_LIBRARY_DIR")
        if "SUITESPARSE_INCLUDE_DIR" in result or "SUITESPARSE_LIBRARY_DIR" in result:
            if not include_value or not library_value:
                raise ValueError(
                    "explicit KLU paths require both include and library directories"
                )
            include = Path(include_value)
            library = Path(library_value)
            if not include.is_absolute() or not library.is_absolute():
                raise ValueError("explicit KLU paths must be absolute")
            prefix = include.parent.parent.resolve()
            if (
                include.resolve() != prefix / "include/suitesparse"
                or library.resolve() != prefix / "lib"
            ):
                raise ValueError(
                    "explicit KLU paths must select one coherent native prefix"
                )
            managed = cache.admit_external(prefix, cache.KLU_FILES)
        else:
            prefix = cache.klu(base, result)
            managed = True
        enforce(result, "SUITESPARSE_INCLUDE_DIR", str(prefix / "include/suitesparse"))
        enforce(result, "SUITESPARSE_LIBRARY_DIR", str(prefix / "lib"))
        enforce(
            result,
            "PSE_KLU_PROVIDER_RECEIPT",
            str(prefix / ".complete.json") if managed else "",
        )
    if "isolation" in requested:
        if result.get("PSE_ROOT_ISOLATION_DIR"):
            cache.admit_external(
                Path(result["PSE_ROOT_ISOLATION_DIR"]), cache.ISOLATION_FILES
            )
        else:
            result["PSE_ROOT_ISOLATION_DIR"] = str(cache.isolation(base, result))
    for kind, variable in (("uno", "UNO_DIR"), ("petsc", "PETSC_DIR")):
        if kind in requested:
            if result.get(variable):
                cache.admit_external(
                    Path(result[variable]), pipeline.required_files(kind)
                )
            else:
                result[variable] = str(pipeline.prepare(kind, result))
            result["LD_LIBRARY_PATH"] = cache.prepend_library_path(
                f"{result[variable]}/lib", result
            )
    for capability in requested:
        # Thread budgets do not change what is prepared, so a nested caller may
        # choose another one without disturbing the operation's admission.
        names = (*cache.INPUT_ENV, *CAPABILITY_PATHS[capability])
        admit(
            "capability:" + capability,
            {"inputs": {name: result.get(name) for name in names}},
        )
    return result


def cancel_children(
    child: subprocess.Popen, signum: int, *, deadline: float | None = None
) -> None:
    deadline = time.monotonic() + 10 if deadline is None else deadline
    completion_deadline = deadline - 0.5
    failure = None
    try:
        inherited = inherited_cancellation(deadline=completion_deadline, reserve=0.5)
        if inherited is not None:
            deadline = min(deadline, inherited[0])
            completion_deadline = deadline - 0.5
            signum = inherited[1]
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
        print(
            f"native-operation: original cancellation authority unresolved: {error}",
            file=sys.stderr,
            flush=True,
        )
    owner = None
    try:
        owner = scope_owner(deadline=completion_deadline)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
        print(
            f"native-operation: own scope authority unresolved: {error}",
            file=sys.stderr,
            flush=True,
        )
    # Reserve part of the same fixed cancellation clock for the mandatory kill.
    # Waiting for an acknowledgment may never consume that final kill budget.
    try:
        cancel_foreground_handoffs(
            deadline=completion_deadline,
            signum=signum,
            drain_deadline=completion_deadline,
            cancellation_deadline=deadline,
        )
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
        print(
            f"native-operation: {error}; unresolved scopes remain pinned",
            file=sys.stderr,
            flush=True,
        )
    try:
        request_composer_completion(signum, deadline=completion_deadline)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
        print(
            f"native-operation: selected terminal acknowledgment unresolved: {error}; invocation remains identified and pinned",
            file=sys.stderr,
            flush=True,
        )
    if owner is not None:
        # A cancelled native operation drains the whole authentic child scope,
        # including workers which changed sessions after the supervisor spawned.
        try:
            subprocess.run(
                [
                    "systemctl",
                    "--user",
                    "kill",
                    "--kill-whom=all",
                    "--signal=SIGKILL",
                    owner["unit"],
                ],
                check=False,
                env=manager_control_environment(),
                # Mandatory kill still occurs when the completion clock expired.
                # Its IPC is bounded by the same remaining clock, with a minimal
                # nonblocking launch budget after expiry; no child gets a reset.
                timeout=max(0.001, min(10, deadline - time.monotonic())),
            )
        except (OSError, subprocess.SubprocessError) as error:
            failure = error
            print(
                f"native-operation: scope cancellation failed: {error}",
                file=sys.stderr,
                flush=True,
            )
    with contextlib.suppress(ProcessLookupError):
        os.killpg(child.pid, signum)
    if failure is not None:
        raise ValueError("native foreground cancellation incomplete") from failure


def cancel_foreground(
    child: subprocess.Popen,
    signum: int,
    handoff: Path,
    *,
    deadline: float | None = None,
) -> tuple[float, int]:
    """Cancel one authenticated child scope, preserving its terminal supervisor."""
    failure = None
    deadline = time.monotonic() + 10 if deadline is None else deadline
    try:
        completion_deadline = deadline - 0.5
        inherited = inherited_cancellation(deadline=completion_deadline, reserve=0.5)
        if inherited is not None:
            deadline, signum = min(deadline, inherited[0]), inherited[1]
            completion_deadline = min(completion_deadline, deadline - 0.5)
        parent = owner_record(deadline=completion_deadline, reserve=0.5)
        if parent is None:
            raise ValueError("foreground cancellation lost its native parent")  # noqa: TRY301 -- retain contextual cancellation failure after final signal
        original = note_composer_cancellation(
            signum, deadline, control_deadline=completion_deadline
        )
        if isinstance(original, tuple):
            deadline, signum = original
            completion_deadline = min(completion_deadline, deadline - 0.5)
        with record_lock(parent, deadline=completion_deadline):
            registered = _record(parent)
            deadline = min(deadline, registered["cancel_deadline"])
            completion_deadline = min(completion_deadline, deadline - 0.5)
            signum = registered["cancel_signal"]
            if str(handoff) not in foreground_children(registered):
                raise ValueError("foreground cancellation lost its registered child")  # noqa: TRY301 -- retain contextual cancellation failure after final signal
            selected = fence_foreground_handoff(
                str(handoff),
                parent,
                deadline=completion_deadline,
                cancellation_deadline=deadline,
                signum=signum,
            )
            owner = selected.get("scope")
            if owner is not None and (
                not isinstance(owner, dict)
                or owner.get("unit") != selected.get("handoff")
                or owner == registered.get("scope")
            ):
                raise ValueError(  # noqa: TRY301 -- retain contextual cancellation failure after final signal
                    "foreground cancellation has contradictory child ownership"
                )
        stop_foreground_handoff(selected, deadline=completion_deadline)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
    finally:
        # Cleanup can receive its first signal after wait() reaped this launcher.
        # Its numeric process group is then no longer owned by this Popen. The
        # authenticated handoff above remains the authority for descendant drain.
        if child.returncode is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signum)
    if failure is not None:
        raise ValueError("native foreground cancellation incomplete") from failure
    return deadline, signum


def run_child_scope(
    command: list[str],
    env: dict[str, str],
    *,
    cwd: Path | None = None,
    observation: dict[str, object] | None = None,
    deadline: float | None = None,
    stdout: int | None = None,
    stderr: int | None = None,
) -> int:
    """Borrow the existing supervised allocation for a separately drained workload.

    The selected terminal supervisor stays in its enclosing scope. The existing
    native child entry authenticates its allocation and handoff before payload.
    No allocation is acquired or released here.
    """
    if deadline is not None:
        if not math.isfinite(deadline):
            raise ValueError("Invalid native workload deadline")
        if time.monotonic() >= deadline:
            raise subprocess.TimeoutExpired(command, 0)
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal ownership
    from scripts.pse_env import (  # noqa: PLC0415 -- reciprocal ownership
        current_memory_ceiling,
    )

    process_observation = observation if observation is not None else {}

    allocation = host_admission.inherit(env, **deadline_options(deadline))
    if allocation is None or scope_owner(**deadline_options(deadline)) is None:
        raise ValueError("selected native workload requires its admitted parent")
    role_memory = current_memory_ceiling()
    if allocation.profile.unbounded:
        from scripts.pse_env import require_unbounded_slice  # noqa: PLC0415 -- placement owner

        require_unbounded_slice(host_admission.allocation_slice(allocation))
        # A genuinely bounded scientific child of this build owner retains its
        # actual role cap; only the ordinary uncapped launcher omits one.
        if role_memory is not None and role_memory <= 0:
            raise ValueError("Invalid originating role cap")
    else:
        if role_memory is None or role_memory <= 0:
            raise ValueError("selected native workload requires a finite originating role cap")
        # A sibling scope preserves its narrower observer/control role.
        role_memory = min(role_memory, allocation.profile.memory)
    unit = f"pse-native-{uuid.uuid4().hex}.scope"
    allocation.register(unit, **deadline_options(deadline))
    handoff = prepare_handoff(unit, foreground=True, **deadline_options(deadline))
    if handoff is None:
        raise ValueError("selected native workload has no authenticated handoff")
    environment = dict(env)
    environment.pop(MARKER, None)
    environment["PSE_NATIVE_HANDOFF"] = str(handoff)
    launch = [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={unit}",
        f"--slice={host_admission.allocation_slice(allocation)}",
        *([] if role_memory is None else [f"--property=MemoryMax={role_memory}", "--property=MemorySwapMax=0"]),
        "--",
        sys.executable,
        str(Path(__file__).resolve()),
        "--capabilities",
        "",
        "--",
        *command,
    ]

    normal_deadline: float | None = None

    def settle_not_started() -> tuple[float, int] | None:
        nonlocal normal_deadline
        normal_deadline = time.monotonic() + 10
        clocks = [normal_deadline]
        record = _record(handoff)
        selected_clocks = [
            process_observation.get("cancellation_deadline"),
            record.get("cancel_deadline"),
            _record(foreground_parent(record, handoff)).get("cancel_deadline"),
        ]
        for value in selected_clocks:
            if value is not None:
                clocks.append(original_cancellation_clock(value))
        original = retire_unlaunched_handoff(handoff, unit, deadline=min(clocks))
        process_observation["owned_drain"] = "not-started"
        return original

    def settle_scope() -> None:
        nonlocal normal_deadline
        # run still owns non-raising signal handling here. A first signal may
        # establish cancellation; repeated signals cannot unwind this drain.
        # Reaping can precede the manager's terminal transition. Join that
        # transition under one cleanup clock, never extending cancellation.
        normal_deadline = time.monotonic() + 10
        cancellation_deadline: float | None = None
        previous_mask: set[int | signal.Signals] | None = None
        try:
            while True:
                if observation is not None:
                    observation["owned_drain"] = "unresolved"
                record = _record(handoff)
                clocks = [
                    cancellation_deadline,
                    process_observation.get("cancellation_deadline"),
                    record.get("cancel_deadline"),
                ]
                if record.get("parent") is not None:
                    clocks.append(
                        _record(foreground_parent(record, handoff)).get(
                            "cancel_deadline"
                        )
                    )
                selected_clocks = [
                    original_cancellation_clock(value)
                    for value in clocks
                    if value is not None
                ]
                cancellation_deadline = (
                    min(selected_clocks) if selected_clocks else None
                )
                deadline = (
                    min(normal_deadline, cancellation_deadline)
                    if cancellation_deadline is not None
                    else normal_deadline
                )
                if cancellation_deadline is not None and previous_mask is None:
                    # Keep the first signal observable during normal cleanup;
                    # defer repeats only after its original clock is recorded.
                    previous_mask = signal.pthread_sigmask(
                        signal.SIG_BLOCK, {signal.SIGINT, signal.SIGTERM}
                    )
                completion = drained(record, deadline=deadline)
                if observation is not None:
                    observation["owned_drain"] = (
                        "complete" if completion.complete else "unresolved"
                    )
                    observation["scope"] = completion.owner
                if completion.complete:
                    return
                if (
                    cancellation_deadline is None
                    and completion.phase in {DrainPhase.POPULATED, DrainPhase.ACTIVE}
                    and process_observation.get("cancellation_deadline") is not None
                ):
                    # A first signal arrived during the observation. Read that
                    # owner's original clock before deciding whether to join.
                    continue
                if completion.phase not in {
                    DrainPhase.POPULATED,
                    DrainPhase.ACTIVE,
                }:
                    raise ValueError(
                        completion.diagnostic() + "; selected workload remains charged"
                    )
                time.sleep(min(0.02, admission_remaining(deadline)))
        finally:
            if previous_mask is not None:
                signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)

    return run(
        launch,
        environment,
        cwd=cwd,
        observation=process_observation,
        foreground=handoff,
        on_reaped=settle_scope,
        on_not_started=settle_not_started,
        cleanup_clock=lambda: normal_deadline,
        stdout=stdout,
        stderr=stderr,
        **deadline_options(deadline),
    )


def run(
    command: list[str],
    env: dict[str, str],
    *,
    cwd: Path | None = None,
    observation: dict[str, object] | None = None,
    foreground: Path | None = None,
    on_reaped: Callable[[], None] | None = None,
    on_not_started: Callable[[], tuple[float, int] | None] | None = None,
    cleanup_clock: Callable[[], float | None] | None = None,
    deadline: float | None = None,
    stdout: int | None = None,
    stderr: int | None = None,
) -> int:
    if deadline is not None and not math.isfinite(deadline):
        raise ValueError("Invalid native workload deadline")
    workload_timeout = (
        max(0, deadline - time.monotonic()) if deadline is not None else 0.0
    )
    timed_out = False
    spawn_attempted = False
    # Only the measured native owner consumes this marker. Its descendants must
    # not overwrite the initial setup or supervisor-return boundaries.
    timing = env.get(BUILD_TIMING)
    if timing is not None and current() is None:
        timing = None
    receipt = json.loads(Path(timing).read_text()) if timing is not None else None
    child_env = env.copy()
    if receipt is not None:
        child_env.pop(BUILD_TIMING, None)
        receipt["child_started_monotonic"] = time.monotonic()
    child: subprocess.Popen | None = None
    status: int | None = None
    child_finished: float | None = None
    if observation is not None:
        observation.update(
            child_pid=None,
            child_status=None,
            forwarded_signal=None,
            owned_drain="unobserved",
        )
    old_handlers = {}
    cancelling = False
    received_signal: int | None = None
    cancel_failure: Exception | None = None
    cancellation_deadline: float | None = None

    def cancel(signum: int, _frame: FrameType | None) -> None:
        nonlocal \
            cancelling, \
            cancel_failure, \
            received_signal, \
            cancellation_deadline, \
            timed_out
        if cancelling:
            return
        if received_signal is None:
            received_signal = signum
            cancellation_deadline = time.monotonic() + 10
            cleanup_deadline = cleanup_clock() if cleanup_clock is not None else None
            if cleanup_deadline is not None:
                cancellation_deadline = min(
                    cancellation_deadline,
                    original_cancellation_clock(cleanup_deadline),
                )
        if observation is not None:
            observation["forwarded_signal"] = signum
            observation["cancellation_deadline"] = cancellation_deadline
        # Popen may be interrupted before returning its owned PID. Remember the
        # request without raising through creation, then cancel that exact child.
        if child is None:
            return
        cancelling = True
        try:
            if foreground is None:
                cancel_children(child, received_signal, deadline=cancellation_deadline)
            else:
                original = cancel_foreground(
                    child, received_signal, foreground, deadline=cancellation_deadline
                )
                if isinstance(original, tuple):
                    if timed_out and (
                        original[1] != received_signal
                        or (
                            cancellation_deadline is not None
                            and original[0] < cancellation_deadline
                        )
                    ):
                        # A timeout cannot replace a cancellation already owned
                        # by this exact handoff, including its signal meaning.
                        timed_out = False
                    cancellation_deadline, received_signal = original
                    if observation is not None:
                        observation.update(
                            cancellation_deadline=cancellation_deadline,
                            forwarded_signal=received_signal,
                        )
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            cancel_failure = error

    previous_mask = signal.pthread_sigmask(
        signal.SIG_BLOCK, {signal.SIGINT, signal.SIGTERM}
    )
    try:
        for signum in (signal.SIGTERM, signal.SIGINT):
            old_handlers[signum] = signal.signal(signum, cancel)
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)
    try:
        expired_before_spawn = deadline is not None and time.monotonic() >= deadline
        if expired_before_spawn and on_not_started is not None:
            # The exact handoff owner can retire this proven no-launch phase.
            # Select timeout/signal truth after its authenticated settlement.
            if received_signal is None:
                timed_out = True
                cancel(signal.SIGTERM, None)
        else:
            if expired_before_spawn and received_signal is None:
                timed_out = True
                cancel(signal.SIGTERM, None)
                raise subprocess.TimeoutExpired(command, workload_timeout)
            spawn_attempted = True
            child = subprocess.Popen(
                command,
                env=child_env,
                cwd=cwd,
                start_new_session=True,
                stdout=stdout,
                stderr=stderr,
            )
            if observation is not None:
                observation["child_pid"] = child.pid
            if received_signal is not None:
                cancel(received_signal, None)
            escalated = False
            while True:
                # Work expiry initiates the existing owned cancellation/drain. An
                # earlier caller signal keeps its original clock and exit meaning.
                if (
                    deadline is not None
                    and received_signal is None
                    and time.monotonic() >= deadline
                ):
                    timed_out = True
                    cancel(signal.SIGTERM, None)
                timeout = 0.1
                if deadline is not None and received_signal is None:
                    timeout = min(timeout, max(0, deadline - time.monotonic()))
                if cancellation_deadline is not None:
                    completion_deadline = cancellation_deadline - 0.5
                    if time.monotonic() >= completion_deadline and not escalated:
                        # Only this Popen's exact new session is escalation authority.
                        # Keep reaping out of signal handlers: Popen's waitpid lock
                        # may belong to the interrupted outer wait at that instant.
                        with contextlib.suppress(ProcessLookupError):
                            os.killpg(child.pid, signal.SIGKILL)
                        escalated = True
                    timeout = max(
                        0,
                        min(
                            0.1,
                            (
                                cancellation_deadline
                                if escalated
                                else completion_deadline
                            )
                            - time.monotonic(),
                        ),
                    )
                try:
                    status = child.wait(timeout=timeout)
                    child_finished = time.monotonic()
                    break
                except subprocess.TimeoutExpired:
                    if (
                        cancellation_deadline is not None
                        and time.monotonic() >= cancellation_deadline
                    ):
                        raise ValueError(
                            "Owned direct child did not reap within original cancellation clock"
                        ) from cancel_failure
            if observation is not None:
                observation["child_status"] = status
    finally:
        try:
            # Also runs on spawn/wait failure. Callback invocation is not a
            # reap witness; it must authenticate and observe its own drain.
            if not spawn_attempted and on_not_started is not None:
                original = on_not_started()
                if original is not None and (
                    cancellation_deadline is None
                    or original[0] <= cancellation_deadline
                ):
                    if timed_out and (
                        original[1] != received_signal
                        or cancellation_deadline is None
                        or original[0] < cancellation_deadline
                    ):
                        timed_out = False
                    cancellation_deadline, received_signal = original
                    if observation is not None:
                        observation.update(
                            cancellation_deadline=cancellation_deadline,
                            forwarded_signal=received_signal,
                        )
            elif on_reaped is not None:
                on_reaped()
        finally:
            for signum, handler in old_handlers.items():
                signal.signal(signum, handler)
            if receipt is not None and timing is not None:
                receipt["child_finished_monotonic"] = child_finished
                child_status = child.returncode if child is not None else None
                receipt["child_exit_code"] = (
                    128 - child_status
                    if child_status is not None and child_status < 0
                    else child_status
                )
                owner = current()
                if owner is not None:
                    with record_lock(owner):
                        receipt["owner"] = _record(owner)
                write_json(Path(timing), receipt)
    if cancel_failure is not None:
        raise ValueError(
            "native foreground cancellation incomplete"
        ) from cancel_failure
    if timed_out:
        raise subprocess.TimeoutExpired(command, workload_timeout)
    # Cleanup can receive the first signal after wait() returned. Choose exit
    # truth only after its physical drain and signal-handler restoration.
    if received_signal is not None:
        return 128 + received_signal
    if status is None:
        raise ValueError("native workload has no terminal outcome")
    return 128 - status if status < 0 else status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capabilities", default="")
    parser.add_argument("--shell", action="store_true")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    requested = [value for value in args.capabilities.split(",") if value]
    if any(value not in CAPABILITIES for value in requested):
        parser.error("unknown native capability request")
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command and not args.shell:
        parser.error("an actual child command is required")
    from scripts import native_cache as cache  # noqa: PLC0415 -- owner cycle

    original = dict(os.environ)
    if not args.shell and original.get("PSE_HOST_ALLOCATION"):
        from scripts import (  # noqa: PLC0415 -- reciprocal operation/admission ownership
            host_admission,
        )

        allocation = host_admission.inherit(original, handoff=True)
        owner = scope_owner()
        if allocation is None or owner is None:
            raise ValueError("Managed native role lacks actual allocation membership")
        allocation.bind(owner["unit"])
    if args.shell:
        configured = environment(requested, build_environment.configure(ROOT, original))
        for name, value in sorted(configured.items()):
            if original.get(name) != value:
                print(f"export {name}={shlex.quote(value)}")
        for name in sorted(original.keys() - configured.keys()):
            print(f"unset {name}")
        return 0
    if owner_record() is not None:
        return run(
            command, environment(requested, build_environment.configure(ROOT, original))
        )
    with Operation(cache.cache_root(original)):
        if original.get("PSE_NATIVE_HANDOFF"):
            bind_handoff(Path(original["PSE_NATIVE_HANDOFF"]))
        return run(
            command,
            environment(requested, build_environment.configure(ROOT, dict(os.environ))),
        )


if __name__ == "__main__":
    raise SystemExit(main())

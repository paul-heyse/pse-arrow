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
from pathlib import Path
from typing import TYPE_CHECKING, NotRequired, Self, TypedDict, TypeVar, cast

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping
    from types import FrameType, TracebackType


ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import (  # noqa: E402 -- direct-script path routing
    build_environment,
    host_admission,
    native_cache,
    operation_lifetime,
    systemd_environment,
)

MARKER = "PSE_NATIVE_OPERATION"
# Private handshake with the build-measurement observer, never an admission input.
BUILD_TIMING = "PSE_BUILD_MEASUREMENT_TIMING"
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
    scope: operation_lifetime.ScopeOwner | None
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


def original_cancellation_clock(value: object) -> float:
    """Narrow an existing cancellation deadline without choosing a new clock."""
    if (
        isinstance(value, (int, float))
        and not isinstance(value, bool)
        and math.isfinite(value)
    ):
        return value
    raise ValueError("invalid original cancellation clock")


def _record(path: Path) -> OperationRecord:
    value = json.loads(path.read_text())
    if value.get("version") != operation_lifetime.VERSION or not isinstance(
        value.get("admissions"), dict
    ):
        raise ValueError("unknown native operation admission record")
    return cast("OperationRecord", value)


def owner_record(*, deadline: float | None = None, reserve: float = 0) -> Path | None:
    if deadline is not None:
        operation_lifetime.admission_remaining(deadline)
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
            operation_lifetime.admission_remaining(deadline)
        owner = record.get("scope")
        if owner:
            return (
                path
                if operation_lifetime.scope_owner(
                    **operation_lifetime.deadline_options(deadline)
                )
                == owner
                else None
            )
        pid = record["pid"]
        if operation_lifetime.start_identity(pid) != record["start"]:
            return None
        descendant = os.getpid()
        while descendant > 1:
            if deadline is not None:
                operation_lifetime.admission_remaining(deadline)
            if descendant == pid:
                return path
            status = Path(f"/proc/{descendant}/status").read_text().splitlines()
            descendant = int(
                next(line.split()[1] for line in status if line.startswith("PPid:"))
            )
    except operation_lifetime.DeadlineExpiredError:
        raise
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        if deadline is not None:
            operation_lifetime.admission_remaining(deadline)
        return None
    if deadline is not None:
        operation_lifetime.admission_remaining(deadline)
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
            operation_lifetime.admission_remaining(deadline)
        with path.with_suffix(".lock").open("a") as lock:
            if deadline is None:
                fcntl.flock(lock, fcntl.LOCK_EX)
            else:
                while True:
                    operation_lifetime.admission_remaining(deadline)
                    try:
                        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                        break
                    except BlockingIOError:
                        time.sleep(
                            min(0.02, operation_lifetime.admission_remaining(deadline))
                        )
                operation_lifetime.admission_remaining(deadline)
            yield
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous)


def remembered(key: str) -> Admission | None:
    # Preparation can belong to an authenticated parent process before a native
    # execution scope exists. Use the same live owner as admit(); current() keeps
    # its narrower meaning for scope allocation and physical drain authority.
    path = owner_record()
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
        operation_lifetime.write_json(path, record)


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
        operation_lifetime.write_json(
            path,
            {
                "version": operation_lifetime.VERSION,
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
            operation_lifetime.write_json(path, record)


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
                if operation_lifetime.drained(record).complete:
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
    parent = owner_record(**operation_lifetime.deadline_options(deadline))
    if parent is None and reservation is None:
        return None
    if parent is None:
        from scripts import native_cache as cache  # noqa: PLC0415 -- reciprocal owner

        directory = cache.cache_root(dict(os.environ)) / ".operations"
        if deadline is not None:
            operation_lifetime.admission_remaining(deadline)
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
            operation_lifetime.admission_remaining(deadline)
        registered = _record(parent) if parent is not None else None
        if foreground and (registered is None or parent is None):
            raise ValueError("native foreground parent is unavailable")
        if foreground and registered is not None and registered.get("cancelled"):
            raise ValueError("native foreground parent is cancelled")
        path = directory / f"handoff-{uuid.uuid4().hex}.json"
        record: OperationRecord = {
            "version": operation_lifetime.VERSION,
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
            operation_lifetime.admission_remaining(deadline)
        operation_lifetime.write_json(path, record)
        if foreground and registered is not None and parent is not None:
            registered["foreground_children"] = [*children, str(path)]
            if deadline is not None:
                operation_lifetime.admission_remaining(deadline)
            operation_lifetime.write_json(parent, registered)
    return path


def bind_handoff(path: Path) -> None:
    """Only the child can prove its kernel membership and invocation generation."""
    owner = operation_lifetime.scope_owner()
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
            operation_lifetime.write_json(path, record)
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
                operation_lifetime.write_json(path, record)
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
    options = operation_lifetime.deadline_options(deadline)
    initial = operation_lifetime.drained(record, **options)
    if initial.complete:
        return
    if initial.phase in {
        operation_lifetime.DrainPhase.UNAVAILABLE,
        operation_lifetime.DrainPhase.MISMATCH,
    }:
        raise ValueError(initial.diagnostic() + "; foreground cancellation refused")
    observed = operation_lifetime.unit_observation(owner["unit"], **options)
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
        timeout=10
        if deadline is None
        else min(10, operation_lifetime.admission_remaining(deadline)),
        env=operation_lifetime.manager_control_environment(),
    )
    completion = operation_lifetime.drained(record, **options)
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
            operation_lifetime.write_json(parent, registered)
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
    parent = owner_record(**operation_lifetime.deadline_options(deadline))
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
        operation_lifetime.write_json(parent, registered)
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
        operation_lifetime.write_json(parent, record)
    name = os.environ.get("PSE_NATIVE_HANDOFF")
    if name:
        path = Path(name)
        selected = _record(path)
        owner = operation_lifetime.scope_owner(deadline=control_deadline)
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
                    operation_lifetime.write_json(path, selected)
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
        owner = operation_lifetime.scope_owner(deadline=deadline)
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
        composer = operation_lifetime.composer_registrations(record).get(nonce)
        if (
            composer is None
            or composer.get("token") != token
            or composer.get("pid") != os.getpid()
            or composer.get("start") != operation_lifetime.start_identity(os.getpid())
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
        operation_lifetime.write_json(parent, record)
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
            for nonce, value in operation_lifetime.composer_registrations(
                record
            ).items()
        }
    for nonce, composer in selected.items():
        operation_lifetime.admission_remaining(deadline)
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
                current = operation_lifetime.composer_registrations(
                    _record(parent)
                ).get(nonce)
                if (
                    current is not None
                    and current.get("phase") == "complete"
                    and current.get("token") == composer.get("token")
                ):
                    continue
            raise
        try:
            if (
                operation_lifetime.start_identity(pid) != composer.get("start")
                or operation_lifetime.process_group(pid) != owner["group"]
            ):
                raise ValueError("selected terminal process lifetime changed")
            signal.pidfd_send_signal(descriptor, signum)
        finally:
            os.close(descriptor)
    pending = set(selected)
    while pending:
        operation_lifetime.admission_remaining(deadline)
        with record_lock(parent, deadline=deadline):
            current = operation_lifetime.composer_registrations(_record(parent))
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
            time.sleep(min(0.02, operation_lifetime.admission_remaining(deadline)))


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
        operation_lifetime.write_json(
            self.path,
            {
                "version": operation_lifetime.VERSION,
                "pid": os.getpid(),
                "start": operation_lifetime.start_identity(os.getpid()),
                "scope": operation_lifetime.scope_owner(),
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
    operation_lifetime.write_json(Path(timing), receipt)
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
        operation_lifetime.write_json(Path(timing), receipt)


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
                cache.admit_external(Path(result[variable]), cache.pipeline_files(kind))
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
        owner = operation_lifetime.scope_owner(deadline=completion_deadline)
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
                env=operation_lifetime.manager_control_environment(),
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


def require_command_parent(
    allocation: host_admission.Allocation, *, deadline: float | None = None
) -> None:
    """Authenticate a bound ordinary command without granting native reuse."""
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal ownership

    if deadline is None:
        with host_admission.allocation_metadata(allocation.directory) as ledger:
            registered = ledger["owners"].get(allocation.nonce)
    else:
        registered = host_admission.readonly_snapshot(
            allocation.directory, deadline=deadline
        )["owners"].get(allocation.nonce)
    if (
        not isinstance(registered, dict)
        or registered.get("boot") != host_admission.boot()
    ):
        raise ValueError("selected workload lost its registered command allocation")
    group = operation_lifetime.process_group(os.getpid())
    unit = next(
        (
            part
            for part in reversed(Path(group).parts)
            if re.fullmatch(r"pse-cmd-[a-f0-9]{32}\.scope", part)
        ),
        None,
    )
    if unit is None:
        raise ValueError("selected workload requires a bound command parent")
    binding = registered["units"].get(unit)
    if not isinstance(binding, dict) or not binding.get("group"):
        raise ValueError("selected workload requires a bound command parent")
    control = binding["group"]
    inode = binding.get("inode")
    if (
        not control.startswith("/")
        or ".." in Path(control).parts
        or Path(control).name != unit
        or not (group == control or group.startswith(control + "/"))
        or type(inode) is not int
        or inode <= 0
        or not re.fullmatch(r"[a-f0-9]{32}", binding.get("invocation", ""))
    ):
        raise ValueError("selected workload has contradictory command ownership")
    observed = host_admission.control_unit_observation(unit, deadline)
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("ActiveState") != "active"
        or observed.get("ControlGroup") != control
        or observed.get("InvocationID") != binding["invocation"]
        or host_admission.group_identity(control) != inode
    ):
        raise ValueError("selected workload command parent identity changed")
    if deadline is not None:
        operation_lifetime.admission_remaining(deadline)


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

    allocation = host_admission.inherit(
        env, **operation_lifetime.deadline_options(deadline)
    )
    if allocation is None:
        raise ValueError("selected native workload requires its admitted parent")
    with contextlib.ExitStack() as lifetime:
        if (
            operation_lifetime.scope_owner(
                **operation_lifetime.deadline_options(deadline)
            )
            is None
        ):
            require_command_parent(
                allocation, **operation_lifetime.deadline_options(deadline)
            )
            if owner_record(**operation_lifetime.deadline_options(deadline)) is None:
                if os.environ.get(MARKER) or env.get(MARKER):
                    raise ValueError(
                        "selected workload has a contradictory operation parent"
                    )
                # PID ownership is sufficient for foreground supervision, but
                # scope=None deliberately cannot authorize native setup reuse.
                lifetime.enter_context(Operation(native_cache.cache_root(env)))
        return _run_child_scope(
            command,
            env,
            allocation,
            cwd=cwd,
            observation=observation,
            deadline=deadline,
            stdout=stdout,
            stderr=stderr,
        )


def _run_child_scope(
    command: list[str],
    env: dict[str, str],
    allocation: host_admission.Allocation,
    *,
    cwd: Path | None,
    observation: dict[str, object] | None,
    deadline: float | None,
    stdout: int | None,
    stderr: int | None,
) -> int:
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal ownership

    process_observation = observation if observation is not None else {}
    role_memory = systemd_environment.current_memory_ceiling()
    if allocation.profile.unbounded:
        systemd_environment.require_unbounded_slice(
            host_admission.allocation_slice(allocation)
        )
        # A genuinely bounded scientific child of this build owner retains its
        # actual role cap; only the ordinary uncapped launcher omits one.
        if role_memory is not None and role_memory <= 0:
            raise ValueError("Invalid originating role cap")
    else:
        if role_memory is None or role_memory <= 0:
            raise ValueError(
                "selected native workload requires a finite originating role cap"
            )
        # A sibling scope preserves its narrower observer/control role.
        role_memory = min(role_memory, allocation.profile.memory)
    unit = f"pse-native-{uuid.uuid4().hex}.scope"
    allocation.register(unit, **operation_lifetime.deadline_options(deadline))
    handoff = prepare_handoff(
        unit, foreground=True, **operation_lifetime.deadline_options(deadline)
    )
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
        *(
            []
            if role_memory is None
            else [f"--property=MemoryMax={role_memory}", "--property=MemorySwapMax=0"]
        ),
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
                completion = operation_lifetime.drained(record, deadline=deadline)
                if observation is not None:
                    observation["owned_drain"] = (
                        "complete" if completion.complete else "unresolved"
                    )
                    observation["scope"] = completion.owner
                if completion.complete:
                    return
                if (
                    cancellation_deadline is None
                    and completion.phase
                    in {
                        operation_lifetime.DrainPhase.POPULATED,
                        operation_lifetime.DrainPhase.ACTIVE,
                    }
                    and process_observation.get("cancellation_deadline") is not None
                ):
                    # A first signal arrived during the observation. Read that
                    # owner's original clock before deciding whether to join.
                    continue
                if completion.phase not in {
                    operation_lifetime.DrainPhase.POPULATED,
                    operation_lifetime.DrainPhase.ACTIVE,
                }:
                    raise ValueError(
                        completion.diagnostic() + "; selected workload remains charged"
                    )
                time.sleep(min(0.02, operation_lifetime.admission_remaining(deadline)))
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
        **operation_lifetime.deadline_options(deadline),
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
                operation_lifetime.write_json(Path(timing), receipt)
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
    parser.add_argument("--prepared-environment", type=Path)
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
    if args.prepared_environment is not None:
        if args.shell:
            parser.error("a prepared receiver runs only its selected child")
        from scripts import (  # noqa: PLC0415 -- immutable launch boundary
            receiver_environment,
            surreal_server,
        )

        if args.prepared_environment != ROOT / "native-environment.json":
            raise ValueError(
                "prepared receiver inputs must belong to this immutable helper"
            )
        surreal_server.activatable_generation(ROOT)
        selected = receiver_environment.read(args.prepared_environment)
        original.update(selected.values)

    def selected_environment(source: dict[str, str]) -> dict[str, str]:
        if args.prepared_environment is not None:
            from scripts import (
                receiver_environment,
            )

            return receiver_environment.restore(
                args.prepared_environment, requested, source
            )
        return environment(requested, build_environment.configure(ROOT, source))

    if not args.shell and original.get("PSE_HOST_ALLOCATION"):
        from scripts import (  # noqa: PLC0415 -- reciprocal operation/admission ownership
            host_admission,
        )

        allocation = host_admission.inherit(original, handoff=True)
        owner = operation_lifetime.scope_owner()
        if allocation is None or owner is None:
            raise ValueError("Managed native role lacks actual allocation membership")
        allocation.bind(owner["unit"])
    if args.shell:
        configured = selected_environment(original)
        for name, value in sorted(configured.items()):
            if original.get(name) != value:
                print(f"export {name}={shlex.quote(value)}")
        for name in sorted(original.keys() - configured.keys()):
            print(f"unset {name}")
        return 0
    if owner_record() is not None:
        return run(command, selected_environment(original))
    with Operation(cache.cache_root(original)):
        if original.get("PSE_NATIVE_HANDOFF"):
            bind_handoff(Path(original["PSE_NATIVE_HANDOFF"]))
        return run(
            command,
            selected_environment(dict(os.environ)),
        )


if __name__ == "__main__":
    raise SystemExit(main())

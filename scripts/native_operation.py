# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native setup admission owned by one actual process operation and child scope."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import json
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
}
CAPABILITY_PATHS = {
    "compiler": ("CLANG_PATH", "BINDGEN_EXTRA_CLANG_ARGS"),
    "solver": ("IPOPT_DIR", "SCIPOPTDIR"),
    "klu": ("SUITESPARSE_INCLUDE_DIR", "SUITESPARSE_LIBRARY_DIR"),
    "isolation": ("PSE_ROOT_ISOLATION_DIR",),
    "uno": ("UNO_DIR", "IPOPT_DIR"),
    "petsc": ("PETSC_DIR", "IPOPT_DIR"),
}


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


def unit_observation(unit: str) -> dict[str, str]:
    environment = dict(os.environ)
    runtime = Path(f"/run/user/{os.getuid()}")
    if (runtime / "bus").is_socket():
        environment.setdefault("XDG_RUNTIME_DIR", str(runtime))
        environment.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    result = subprocess.run(
        [
            "systemctl",
            "--user",
            "show",
            "--property=InvocationID",
            "--property=ControlGroup",
            "--property=LoadState",
            "--property=ActiveState",
            unit,
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
        env=environment,
    )
    if result.returncode:
        raise ValueError("native operation unit observation unavailable")
    return dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )


def scope_owner() -> ScopeOwner | None:
    """Only real scope membership and its invocation grant operation reuse."""
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
        observed = unit_observation(unit)
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
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        return None

    else:
        return {"unit": unit, "group": control, "invocation": invocation}


def populated(group: str) -> bool:
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise ValueError("invalid native scope group")
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    if not directory.exists():
        return False
    fields = dict(
        line.split() for line in (directory / "cgroup.events").read_text().splitlines()
    )
    if fields.get("populated") not in {"0", "1"}:
        raise ValueError("native scope population unavailable")
    return fields["populated"] == "1"


def drained(record: Mapping[str, object]) -> bool:
    """Parent exit never releases a populated group. Unknown owners stay pinned."""
    owner = record.get("scope")
    if not isinstance(owner, dict):
        return False
    try:
        if populated(owner["group"]):
            return False
        observed = unit_observation(owner["unit"])
        if observed.get("LoadState") == "not-found":
            return True
        return (
            observed.get("InvocationID") == owner["invocation"]
            and observed.get("ControlGroup") in {owner["group"], ""}
            and observed.get("ActiveState") in {"inactive", "failed"}
            and not populated(owner["group"])
        )
    except (OSError, ValueError, KeyError, subprocess.SubprocessError):
        return False


def _record(path: Path) -> OperationRecord:
    value = json.loads(path.read_text())
    if value.get("version") != VERSION or not isinstance(value.get("admissions"), dict):
        raise ValueError("unknown native operation admission record")
    return cast("OperationRecord", value)


def owner_record() -> Path | None:
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
        owner = record.get("scope")
        if owner:
            return path if scope_owner() == owner else None
        pid = record["pid"]
        if start_identity(pid) != record["start"]:
            return None
        descendant = os.getpid()
        while descendant > 1:
            if descendant == pid:
                return path
            status = Path(f"/proc/{descendant}/status").read_text().splitlines()
            descendant = int(
                next(line.split()[1] for line in status if line.startswith("PPid:"))
            )
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError):
        return None
    return None


def current() -> Path | None:
    path = owner_record()
    return path if path is not None and _record(path).get("scope") else None


@contextlib.contextmanager
def record_lock(path: Path) -> Generator[None, None, None]:
    # A synchronous cancellation handler must not reacquire our own flock.
    previous = signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGTERM, signal.SIGINT})
    try:
        with path.with_suffix(".lock").open("a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
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
    Unknown records conservatively refuse reclamation; completed operations have
    no consumer after drain and are deleted rather than becoming an archive.
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
                if drained(record):
                    # The live parent may still need this exact child binding
                    # during cancellation. Its settled child no longer pins bytes.
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


def prepare_handoff(expected_unit: str, *, foreground: bool = False) -> Path | None:
    """Pin before launch; a dead launcher leaves a conservative pending guard."""
    parent = owner_record()
    if parent is None:
        return None
    with record_lock(parent):
        registered = _record(parent)
        if foreground and registered.get("cancelled"):
            raise ValueError("native foreground parent is cancelled")
        path = parent.parent / f"handoff-{uuid.uuid4().hex}.json"
        record: OperationRecord = {
            "version": VERSION,
            "scope": None,
            "handoff": expected_unit,
            "admissions": {},
            "generations": registered["generations"][:],
        }
        children: list[str] = []
        if foreground:
            if not re.fullmatch(r"pse-native-[a-f0-9]{32}\.scope", expected_unit):
                raise ValueError("foreground handoff requires an observer scope")
            record.update(foreground=True, parent=str(parent))
            children = foreground_children(registered)
        write_json(path, record)
        if foreground:
            registered["foreground_children"] = [*children, str(path)]
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
            if parent is not None:
                from scripts import host_admission  # noqa: PLC0415 -- reciprocal owner

                if record.get("parent") != str(parent) or (
                    registered is not None
                    and str(path) not in foreground_children(registered)
                ):
                    raise ValueError("native foreground parent association changed")
                inode = host_admission.group_identity(owner["group"])
                if inode is None or record.get("scope_inode", inode) != inode:
                    raise ValueError("native foreground scope generation changed")
                record["scope_inode"] = inode
                if registered is None or registered.get("cancelled"):
                    record["cancelled"] = True
            write_json(path, record)
            if record.get("foreground") and record.get("cancelled"):
                raise ValueError("native foreground handoff is cancelled")


def fence_foreground_handoff(name: object, parent: Path) -> OperationRecord:
    if isinstance(name, str):
        path = Path(name)
        if path.parent == parent.parent and re.fullmatch(
            r"handoff-[a-f0-9]{32}\.json", path.name
        ):
            with record_lock(path):
                record = _record(path)
                if record.get("foreground") is not True or record.get("parent") != str(
                    parent
                ):
                    raise ValueError("foreground handoff parent changed")
                record["cancelled"] = True
                write_json(path, record)
            return record
    raise ValueError("invalid foreground handoff association")


def stop_foreground_handoff(record: OperationRecord) -> None:
    from scripts import host_admission  # noqa: PLC0415 -- reciprocal owner

    owner = record.get("scope", False)
    if owner is None:
        # A late child still binds its actual lifetime, then refuses payload.
        return
    if not isinstance(owner, dict) or any(
        not isinstance(owner.get(key), str) for key in ("unit", "group", "invocation")
    ):
        raise ValueError("invalid foreground scope identity")
    if drained(record):
        return
    observed = unit_observation(owner["unit"])
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("InvocationID") != owner["invocation"]
        or observed.get("ControlGroup") != owner["group"]
        or type(record.get("scope_inode")) is not int
        or host_admission.group_identity(owner["group"]) != record["scope_inode"]
    ):
        raise ValueError("foreground observer scope identity changed")
    # Stop waits for this authenticated scope. Its native handler first settles
    # any nested foreground handoffs before its own scope kill.
    subprocess.run(
        ["systemctl", "--user", "stop", owner["unit"]],
        check=True,
        timeout=10,
    )
    if not drained(record):
        raise ValueError("foreground observer scope did not drain")


def cancel_foreground_handoffs() -> None:
    """Settle only the foreground observers explicitly owned by this operation."""
    parent = owner_record()
    if parent is None:
        return
    failures: list[str] = []
    bound: list[OperationRecord] = []
    with record_lock(parent):
        registered = _record(parent)
        registered["cancelled"] = True
        write_json(parent, registered)
        children = foreground_children(registered)[:]
        for name in children:
            try:
                bound.append(fence_foreground_handoff(name, parent))
            except (OSError, ValueError, KeyError) as error:
                failures.append(str(error))
    for record in bound:
        try:
            stop_foreground_handoff(record)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            failures.append(str(error))
    if failures:
        raise ValueError("foreground cancellation incomplete: " + "; ".join(failures))


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
            cancel_foreground_handoffs()
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
    off = {name for name in result.get(OFF_MARKER, "").split(",") if name}
    for name in OVERRIDABLE:
        if result.get(name) == "off":
            off.add(name)
        elif name in result:
            off.discard(name)
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
        for name, value in cache.runtime_env(
            ROOT / "docker/solvers/Dockerfile"
        ).items():
            if name in AUTHORITATIVE:
                enforce(result, name, value)
            elif name not in off:
                result.setdefault(name, value)
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
            cache.admit_external(prefix, cache.KLU_FILES)
        else:
            prefix = cache.klu(base, result)
        enforce(result, "SUITESPARSE_INCLUDE_DIR", str(prefix / "include/suitesparse"))
        enforce(result, "SUITESPARSE_LIBRARY_DIR", str(prefix / "lib"))
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
    for capability in requested:
        # Thread budgets do not change what is prepared, so a nested caller may
        # choose another one without disturbing the operation's admission.
        names = (*cache.INPUT_ENV, *CAPABILITY_PATHS[capability])
        admit(
            "capability:" + capability,
            {"inputs": {name: result.get(name) for name in names}},
        )
    return result


def cancel_children(child: subprocess.Popen, signum: int) -> None:
    failure = None
    try:
        cancel_foreground_handoffs()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        failure = error
        print(
            f"native-operation: {error}; unresolved scopes remain pinned",
            file=sys.stderr,
            flush=True,
        )
    owner = scope_owner()
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
                timeout=10,
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


def run(command: list[str], env: dict[str, str]) -> int:
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
    child = subprocess.Popen(command, env=child_env, start_new_session=True)
    old_handlers = {}
    cancelling = False
    cancel_failure: Exception | None = None

    def cancel(signum: int, _frame: FrameType | None) -> None:
        nonlocal cancelling, cancel_failure
        if cancelling:
            return
        cancelling = True
        try:
            cancel_children(child, signum)
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            cancel_failure = error

    for signum in (signal.SIGTERM, signal.SIGINT):
        old_handlers[signum] = signal.signal(signum, cancel)
    try:
        status = child.wait()
        if cancel_failure is not None:
            raise ValueError(
                "native foreground cancellation incomplete"
            ) from cancel_failure
        # A child killed by signal N reports 128+N, as a shell would, rather than
        # a negative status that `SystemExit` would turn into 256-N.
        return 128 - status if status < 0 else status
    finally:
        for signum, handler in old_handlers.items():
            signal.signal(signum, handler)
        if receipt is not None and timing is not None:
            receipt["child_finished_monotonic"] = time.monotonic()
            status = child.returncode
            receipt["child_exit_code"] = (
                128 - status if status is not None and status < 0 else status
            )
            owner = current()
            if owner is not None:
                with record_lock(owner):
                    receipt["owner"] = _record(owner)
            write_json(Path(timing), receipt)


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

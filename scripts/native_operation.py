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
    with path.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


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


def prepare_handoff(expected_unit: str) -> Path | None:
    """Pin before launch; a dead launcher leaves a conservative pending guard."""
    parent = owner_record()
    if parent is None:
        return None
    with record_lock(parent):
        generations = _record(parent)["generations"][:]
    path = parent.parent / f"handoff-{uuid.uuid4().hex}.json"
    write_json(
        path,
        {
            "version": VERSION,
            "scope": None,
            "handoff": expected_unit,
            "admissions": {},
            "generations": generations,
        },
    )
    return path


def bind_handoff(path: Path) -> None:
    """Only the child can prove its kernel membership and invocation generation."""
    owner = scope_owner()
    if owner is None:
        raise ValueError("native child handoff requires an authentic managed scope")
    with record_lock(path):
        record = _record(path)
        if record.get("handoff") != owner["unit"] or record.get("scope") not in (
            None,
            owner,
        ):
            raise ValueError("native child handoff differs from its registered scope")
        record["scope"] = owner
        write_json(path, record)


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
    owner = scope_owner()
    if owner is not None:
        # A cancelled native operation drains the whole authentic child scope,
        # including workers which changed sessions after the supervisor spawned.
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
    with contextlib.suppress(ProcessLookupError):
        os.killpg(child.pid, signum)


def run(command: list[str], env: dict[str, str]) -> int:
    child = subprocess.Popen(command, env=env, start_new_session=True)
    old_handlers = {}

    def cancel(signum: int, _frame: FrameType | None) -> None:
        cancel_children(child, signum)

    for signum in (signal.SIGTERM, signal.SIGINT):
        old_handlers[signum] = signal.signal(signum, cancel)
    try:
        status = child.wait()
        # A child killed by signal N reports 128+N, as a shell would, rather than
        # a negative status that `SystemExit` would turn into 256-N.
        return 128 - status if status < 0 else status
    finally:
        for signum, handler in old_handlers.items():
            signal.signal(signum, handler)


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

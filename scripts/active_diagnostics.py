# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected active tools with explicit effects, exact subjects and retained action output.

These actions own their helper scope. An attached process is never their child,
never a cancellation target, and never restarted or resumed by this adapter.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import threading
import time
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, model_validator

from scripts import process_observation
from scripts import operation_lifetime
from scripts import (
    diagnostic_trace,
    doctor,
    native_operation,
    resource_store,
    run_inspection,
)

SCHEMA = "active-diagnostic-action-v1"
CPYTHON_SYMBOL_DIRECTORY = Path(os.path.sep) / "tmp"


class ActionInterruptedError(RuntimeError):
    """The owned helper drained after caller cancellation; do not start another step."""


class ActionRequest(BaseModel):
    """Validated host intent; process/tool identities are admitted at execution."""

    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )

    action: Literal[
        "python-snapshot",
        "python-sample",
        "python-perf",
        "gdb-launch",
        "gdb-core",
        "sched",
        "heaptrack",
        "stat",
        "feasibility",
        "memray",
        "rr-record",
        "rr-replay",
    ]
    output: str = Field(min_length=1)
    seconds: float = Field(default=10.0, gt=0, le=300, allow_inf_nan=False)
    output_bytes: int = Field(default=8 << 20, ge=0, le=64 << 20)
    pid: int | None = Field(default=None, gt=0)
    start: str | None = Field(default=None, pattern=r"^[0-9]+$")
    native: bool = False
    nonblocking: bool = False
    jit: bool = False
    faulthandler: bool = False
    core: str | None = Field(default=None, min_length=1)
    executable: str | None = Field(default=None, min_length=1)
    executable_sha256: str | None = Field(default=None, pattern=r"^[a-f0-9]{64}$")
    events: str | None = None
    memray_python: str | None = Field(default=None, min_length=1)
    debuginfod: bool = False
    rr_binary: str | None = Field(default=None, min_length=1)
    trace: str | None = Field(default=None, min_length=1)
    trace_sha256: str | None = Field(default=None, pattern=r"^[a-f0-9]{64}$")
    breakpoint: str | None = None
    perf_binary: str | None = Field(default=None, min_length=1)
    operation: str | None = None
    invocation: str | None = None
    registry: str | None = Field(default=None, min_length=1)
    command: list[str] = Field(default_factory=list)

    @model_validator(mode="after")
    def selected_subject(self) -> ActionRequest:
        attachment = self.action in {"python-snapshot", "python-sample"}
        if self.action == "python-sample" and not self.seconds.is_integer():
            raise ValueError("py-spy sampling requires a whole number of seconds")
        if attachment:
            self.attachment_subject()
        if not attachment and (self.pid is not None or self.start is not None):
            raise ValueError(
                "live process attachment is supported only by explicit Python snapshot/sample actions; GDB uses owned launch or supplied core"
            )
        if self.registry is not None and self.invocation is None:
            raise ValueError(
                "--registry requires an authentic --invocation association"
            )
        selected_options = {
            "native": {"python-snapshot", "python-sample"},
            "nonblocking": {"python-snapshot", "python-sample"},
            "jit": {"python-perf"},
            "faulthandler": {"gdb-launch"},
            "core": {"gdb-core"},
            "executable": {"gdb-core"},
            "executable_sha256": {"gdb-core"},
            "events": {"stat"},
            "memray_python": {"memray"},
            "debuginfod": {"python-perf", "memray"},
            "rr_binary": {"rr-record", "rr-replay"},
            "trace": {"rr-replay"},
            "trace_sha256": {"rr-replay"},
            "breakpoint": {"rr-replay"},
            "perf_binary": {"python-perf", "sched", "stat"},
        }
        for field, actions in selected_options.items():
            value = getattr(self, field)
            if value is not None and value is not False and self.action not in actions:
                raise ValueError(f"{field} does not apply to {self.action}")
        if any(not value or "\x00" in value for value in self.command):
            raise ValueError("workload argv must contain nonempty strings without NUL")
        if (
            attachment or self.action in {"feasibility", "gdb-core", "rr-replay"}
        ) and self.command:
            raise ValueError("selected action does not launch supplied workload argv")
        # Omitted, explicit null and an explicit empty event selection are not
        # interchangeable: only absence/null selects the standard perf events.
        if self.events is not None and not re.fullmatch(
            r"[A-Za-z0-9_:,./=-]+", self.events
        ):
            raise ValueError("invalid explicit perf event selection")
        return self

    def attachment_subject(self) -> tuple[int, str]:
        if self.pid is None or self.start is None:
            raise ValueError("Python attachment requires exact --pid and --start")
        return self.pid, self.start


def regular(
    path: Path, expected: str | None = None, *, hash_contents: bool = True
) -> dict:
    path = path.absolute()
    if ".." in path.parts or any(item.is_symlink() for item in (path, *path.parents)):
        raise ValueError("selected artifact must not traverse symlinks")
    descriptor = os.open(
        path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    )
    with os.fdopen(descriptor, "rb") as source:
        before = os.fstat(source.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise ValueError("selected artifact is not an ordinary file")
        digest = (
            hashlib.file_digest(source, "sha256").hexdigest() if hash_contents else None
        )
        after = os.fstat(source.fileno())
        if (before.st_ino, before.st_size, before.st_mtime_ns) != (
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
        ):
            raise ValueError("selected artifact changed during identification")
    if expected is not None and digest != expected:
        raise ValueError(
            "selected executable identity differs from the supplied digest"
        )
    return {
        "path": str(path),
        "sha256": digest,
        "bytes": before.st_size,
        "device": before.st_dev,
        "inode": before.st_ino,
    }


def retain_symbol(
    original: Path, destination: Path, wall_ns: int, *, maximum_bytes: int | None = None
) -> None:
    """Copy a fresh owned CPython symbol file through one verified descriptor."""
    descriptor = os.open(original, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(descriptor, "rb") as source:
        before = os.fstat(source.fileno())
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_uid != os.getuid()
            or before.st_mtime_ns < wall_ns
            or (maximum_bytes is not None and before.st_size > maximum_bytes)
        ):
            raise ValueError(
                "symbol file is not fresh bounded output from the selected process"
            )
        with destination.open("xb") as target:
            shutil.copyfileobj(source, target, 1 << 20)
        after = os.fstat(source.fileno())
        if (before.st_ino, before.st_size, before.st_mtime_ns) != (
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
        ):
            raise ValueError("symbol file changed during retention")


def process_subject(pid: int, start: str) -> dict:
    if pid <= 0 or not re.fullmatch(r"[0-9]+", start):
        raise ValueError(
            "live attachment requires an exact PID and /proc start identity"
        )
    before = operation_lifetime.start_identity(pid)
    if before != start:
        raise ValueError("selected process start identity changed")
    directory = Path("/proc") / str(pid)
    status = (directory / "status").read_text()
    state = next(
        line.split(":", 1)[1].strip()
        for line in status.splitlines()
        if line.startswith("State:")
    )
    threads = []
    for task in (directory / "task").iterdir():
        if len(threads) >= 1024:
            break
        try:
            threads.append(
                {
                    "tid": int(task.name),
                    "start": operation_lifetime.start_identity(int(task.name)),
                }
            )
        except OSError:
            continue
    executable = (directory / "exe").resolve(strict=True)
    if operation_lifetime.start_identity(pid) != start:
        raise ValueError("process changed while selecting its subject")
    return {
        "kind": "live-process",
        "pid": pid,
        "start": start,
        "state": state,
        "boot": process_observation.boot(),
        "observed_monotonic_ns": time.monotonic_ns(),
        "threads": threads,
        "thread_page_limit": 1024,
        "executable": regular(executable),
        "operation_association": "not supplied",
    }


def selected_tool(name: str, supplied: Path | None = None) -> dict:
    binary = str(supplied.absolute()) if supplied is not None else shutil.which(name)
    if binary is None:
        return {
            "name": name,
            "status": "unavailable",
            "reason": "not installed on selected PATH",
        }
    binary = str(Path(binary).resolve(strict=True))
    try:
        version = subprocess.run(
            [binary, "--version"],
            capture_output=True,
            text=True,
            timeout=3,
            check=False,
        )
        return {
            "name": name,
            "status": "available",
            "path": binary,
            "identity": regular(Path(binary)),
            "version": (version.stdout + version.stderr)[:4096].strip(),
            "version_exit": version.returncode,
        }
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        return {
            "name": name,
            "status": "unavailable",
            "path": binary,
            "reason": str(error),
        }


def trace_reference(directory: Path, expected: str | None = None) -> dict:
    """Fingerprint only this selected rr trace, with explicit file/byte bounds."""
    if ".." in directory.parts or any(
        path.is_symlink() for path in (directory, *directory.parents)
    ):
        raise ValueError("rr trace must be an exact directory without symlinks")
    before = directory.stat()
    members = sorted(directory.iterdir())
    if not stat.S_ISDIR(before.st_mode) or len(members) > 1024:
        raise ValueError("rr trace directory exceeds the selected 1024-file bound")
    total = 0
    references = []
    for path in members:
        total += path.lstat().st_size
        if total > 1 << 30:
            raise ValueError(
                "rr trace fingerprint exceeds the selected 1GiB byte bound"
            )
        references.append({"member": path.name, **regular(path)})
    after = directory.stat()
    if (before.st_ino, before.st_mtime_ns) != (after.st_ino, after.st_mtime_ns):
        raise ValueError("rr trace membership changed during identification")
    digest = hashlib.sha256(json.dumps(references, sort_keys=True).encode()).hexdigest()
    if expected is not None and digest != expected:
        raise ValueError("rr trace differs from the selected trace fingerprint")
    return {
        "path": str(directory),
        "device": before.st_dev,
        "inode": before.st_ino,
        "sha256": digest,
        "files": references,
        "bytes": total,
    }


def memray_runtime(path: Path | None) -> dict:
    if path is None:
        raise ValueError(
            "Memray requires explicit --memray-python from the isolated pinned tool environment"
        )
    selected = path.absolute()
    result = subprocess.run(
        [
            str(selected),
            "-c",
            "import json,sys,memray; print(json.dumps({'version':memray.__version__,'prefix':sys.prefix,'executable':sys.executable,'python':sys.version}))",
        ],
        capture_output=True,
        text=True,
        timeout=5,
        check=True,
    )
    runtime = json.loads(result.stdout)
    pins = []
    for declaration in (
        doctor.pyproject().get("dependency-groups", {}).get("diagnostics", [])
    ):
        if isinstance(declaration, str) and (
            matched := re.fullmatch(r"memray==([^; ]+)(?:;.*)?", declaration)
        ):
            pins.append(matched[1])
    if len(pins) != 1:
        raise ValueError(
            "diagnostics dependency group must declare exactly one exact Memray pin"
        )
    if runtime["version"] != pins[0]:
        raise ValueError(
            f"selected Memray environment must contain declared memray=={pins[0]}"
        )
    return {
        "name": "memray",
        "status": "available",
        "path": str(selected),
        "identity": regular(selected.resolve(strict=True)),
        "runtime": runtime,
    }


def identified_launch(command: list[str], subject_file: Path) -> list[str]:
    """Publish this owned exec request's kernel identity without interpreting workload argv."""
    selected_environment = (
        {"LD_LIBRARY_PATH": os.environ["LD_LIBRARY_PATH"]}
        if "LD_LIBRARY_PATH" in os.environ
        else {}
    )
    # A capability-bearing perf executable enters glibc secure mode, which strips
    # LD_LIBRARY_PATH. Restore this exact selected value only after perf has
    # exec'd the unprivileged relay, then re-exec the real target so its loader
    # consumes it. Never forward other LD_* values or compose a new default.
    launcher = (
        "import json,os,sys,time; from pathlib import Path; "
        "from scripts.process_observation import start_identity, boot; "
        "selected=json.loads(sys.argv[2]); os.environ.update(selected); "
        "p=Path(sys.argv[1]); "
        "f=os.fdopen(os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'w'); "
        "json.dump({'pid':os.getpid(),'start':start_identity(os.getpid()),"
        "'wall_ns':time.time_ns(),'monotonic_ns':time.monotonic_ns(),"
        "'boot':boot(),"
        "'restored_environment':list(selected),"
        "'exec_observation':'exec requested; workload outcome is separate'},f); "
        "f.close(); os.execvp(sys.argv[3],sys.argv[3:])"
    )
    return [
        sys.executable,
        "-c",
        launcher,
        str(subject_file),
        json.dumps(selected_environment),
        *command,
    ]


def collect(
    command: list[str],
    output: Path,
    label: str,
    *,
    seconds: float,
    output_bytes: int,
    cwd: Path | None = None,
    environment_remove: tuple[str, ...] = (),
) -> dict:
    """Use the existing exact child scope; cap retained text while draining both pipes."""
    observation: dict[str, object] = {}
    retained = {"stdout": 0, "stderr": 0}
    seen = {"stdout": 0, "stderr": 0}
    errors: list[str] = []
    readers = []
    descriptors = []
    started = time.monotonic()

    def reader(descriptor: int, name: str) -> None:
        try:
            with (
                os.fdopen(descriptor, "rb") as source,
                (output / f"{label}.{name}.txt").open("xb") as destination,
            ):
                while chunk := source.read(65536):
                    seen[name] += len(chunk)
                    selected = chunk[: max(0, output_bytes - retained[name])]
                    destination.write(selected)
                    retained[name] += len(selected)
        except OSError as error:
            errors.append(f"{name}: {error}")

    for name in ("stdout", "stderr"):
        read_fd, write_fd = os.pipe()
        descriptors.append(write_fd)
        thread = threading.Thread(target=reader, args=(read_fd, name), daemon=True)
        thread.start()
        readers.append(thread)
    code = None
    status = "returned"
    try:
        from scripts.telemetry_process import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            helper_environment,
        )

        environment = helper_environment()
        for name in environment_remove:
            environment.pop(name, None)
        code = native_operation.run_child_scope(
            command,
            environment,
            stdout=descriptors[0],
            stderr=descriptors[1],
            observation=observation,
            cwd=cwd,
            deadline=started + seconds,
        )
    except subprocess.TimeoutExpired:
        status = "timed_out"
    except (OSError, ValueError, RuntimeError) as error:
        status = "unavailable"
        errors.append(str(error))
    finally:
        for descriptor in descriptors:
            os.close(descriptor)
        for thread in readers:
            thread.join(timeout=5)
        if any(thread.is_alive() for thread in readers):
            errors.append(
                "owned helper output writers remain; read coverage is incomplete"
            )
    return {
        "argv": command,
        "status": status,
        "exit_code": code,
        "process": observation,
        "elapsed_seconds": time.monotonic() - started,
        "output_seen_bytes": seen,
        "output_retained_bytes": retained,
        "truncated": any(seen[key] > retained[key] for key in seen),
        "errors": errors,
        "environment_removed": list(environment_remove),
    }


def read_action(
    directory: Path,
    *,
    tail: int = 0,
    summary_only: bool = False,
    action_sha256: str | None = None,
) -> dict:
    if (
        tail < 0
        or ".." in directory.parts
        or any(item.is_symlink() for item in (directory, *directory.parents))
    ):
        raise ValueError("select an exact action directory and nonnegative tail")
    reference = regular(directory / "action.json", action_sha256)
    record = resource_store.read(directory / "action.json")
    regular(directory / "action.json", reference["sha256"])
    record["action_reference"] = reference
    if record.get("schema") != SCHEMA or not isinstance(record.get("artifacts"), list):
        raise ValueError("not an active diagnostic action document")
    if summary_only:
        for artifact in record["artifacts"]:
            name = artifact.get("member")
            if not isinstance(name, str) or Path(name).name != name:
                raise ValueError("invalid owned action artifact reference")
        return record
    result = {**record, "artifact_observations": []}
    for artifact in record["artifacts"][:100]:
        name = artifact.get("member")
        if not isinstance(name, str) or Path(name).name != name:
            raise ValueError("invalid owned action artifact reference")
        if not isinstance(artifact.get("sha256"), str):
            observed = {
                "path": str(directory / name),
                "status": "unavailable",
                "error": "artifact has no recorded digest; original collection was incomplete",
            }
            result["artifact_observations"].append(observed)
            continue
        try:
            observed = run_inspection.artifact_observation(
                directory / name,
                artifact["sha256"],
                tail if name.endswith(".txt") else 0,
                None,
            )
        except (OSError, ValueError) as error:
            observed = {
                "path": str(directory / name),
                "status": "unavailable",
                "error": str(error),
            }
        result["artifact_observations"].append(observed)
    return result


def publish(output: Path, record: dict) -> dict:
    artifacts = []
    for path in sorted(output.iterdir()):
        if path.name == "action.json" or not path.is_file():
            continue
        try:
            artifacts.append({"member": path.name, **regular(path)})
        except (OSError, ValueError) as error:
            artifacts.append(
                {"member": path.name, "status": "unavailable", "error": str(error)}
            )
    record["artifacts"] = artifacts
    record["finished_monotonic_ns"] = time.monotonic_ns()
    resource_store.publish(output / "action.json", record)
    association = record.get("invocation")
    if isinstance(association, dict):
        store = resource_store.FileStore(Path(association["registry"]))
        reference = {
            "path": str(output / "action.json"),
            "sha256": regular(output / "action.json")["sha256"],
            "action": record["action"],
        }
        with store.invocation_edit(association["identity"]) as state:
            references = state.setdefault("diagnostic_actions", [])
            if reference not in references:
                references.append(reference)
    return record


def execute(request: ActionRequest) -> dict:
    """Run one admitted action; its helper scope never owns an attached subject."""
    output = Path(request.output).absolute()
    if any(path.is_symlink() for path in (output, *output.parents)):
        raise ValueError("action output must not traverse symlinks")
    output.mkdir(mode=0o700, parents=False, exist_ok=False)
    record: dict = {
        "schema": SCHEMA,
        "output": str(output),
        "action_record": str(output / "action.json"),
        "action": request.action,
        "started_monotonic_ns": time.monotonic_ns(),
        "target_outcome": "not observed; collection success is independent",
        "operation": request.operation,
        "invocation": None,
        "steps": [],
        "status": "incomplete",
        "bounds": {
            "seconds_per_step": request.seconds,
            "text_bytes_per_stream": request.output_bytes,
            "raw_artifact_bytes": "not capped; duration bounds execution, not raw file size",
        },
    }

    def step(
        command: list[str],
        label: str,
        seconds: float | None = None,
        cwd: Path | None = None,
        environment_remove: tuple[str, ...] = (),
    ) -> dict:
        value = collect(
            command,
            output,
            label,
            seconds=request.seconds if seconds is None else seconds,
            output_bytes=request.output_bytes,
            cwd=cwd,
            environment_remove=environment_remove,
        )
        record["steps"].append(value)
        process = value.get("process", {})
        if (
            value["status"] != "timed_out"
            and process.get("forwarded_signal") is not None
        ):
            raise ActionInterruptedError(
                "owned diagnostic helper interrupted; subject ownership unchanged"
            )
        return value

    command = request.command

    def perform() -> None:
        if request.invocation is not None:
            root, declaration, _state = run_inspection.resolve(
                request.invocation,
                Path(request.registry) if request.registry is not None else None,
            )
            record["invocation"] = {
                "registry": str(root),
                "identity": declaration["nonce"],
            }
        if request.action in {"python-snapshot", "python-sample"}:
            pid, start = request.attachment_subject()
            subject = process_subject(pid, start)
            if subject["state"].startswith(("T", "t")):
                raise ValueError(
                    "attachment to an already stopped target is refused; offline/owned launch is available"
                )
            if request.native and request.nonblocking:
                raise ValueError(
                    "py-spy native sampling requires target pauses and cannot be nonblocking"
                )
            record.update(
                subject=subject,
                effect="temporary native sampling pauses"
                if request.native
                else "nonblocking Python snapshot"
                if request.nonblocking
                else "temporary Python sampling pauses",
                coverage={
                    "population": "CPython interpreter threads; Rust-owned threads without Python state are not promised",
                    "native_workers": "not a complete native-thread census",
                    "released_gil": "included interpreter threads; attribution is sample-dependent",
                },
            )
            tool = selected_tool("py-spy")
            record["tool"] = tool
            if tool["status"] != "available":
                raise ValueError(tool["reason"])
            if operation_lifetime.start_identity(pid) != start:
                raise ValueError("selected target identity changed before attachment")
            argv = (
                [tool["path"], "dump", "--pid", str(pid), "--json"]
                if request.action == "python-snapshot"
                else [
                    tool["path"],
                    "record",
                    "--pid",
                    str(pid),
                    "--duration",
                    str(int(request.seconds)),
                    "--threads",
                    "--idle",
                    "--format",
                    "speedscope",
                    "--output",
                    str(output / "python.speedscope.json"),
                ]
            )
            argv += ["--native"] if request.native else []
            argv += ["--nonblocking"] if request.nonblocking else []
            value = step(argv, "py-spy", request.seconds + 5)
            try:
                record["subject_after"] = process_subject(pid, start)
            except (OSError, ValueError) as error:
                record["subject_after"] = {
                    "status": "unavailable",
                    "reason": str(error),
                }
            record["status"] = "collected" if value["exit_code"] == 0 else "unavailable"
        elif request.action in {"gdb-launch", "gdb-core"}:
            tool = selected_tool("gdb")
            record.update(
                tool=tool,
                effect="owned stopped-process experiment"
                if request.action == "gdb-launch"
                else "offline core interpretation",
            )
            if tool["status"] != "available":
                raise ValueError(tool["reason"])
            # --args performs GDB's own shell escaping. Disabling its startup
            # shell leaves those escapes literal in GDB 15 and corrupts argv.
            base = [
                tool["path"],
                "--nx",
                "--nh",
                "--batch",
                "-iex",
                "set auto-load off",
                "-ex",
                "set pagination off",
            ]
            if request.action == "gdb-core":
                if (
                    request.core is None
                    or request.executable is None
                    or request.executable_sha256 is None
                ):
                    raise ValueError(
                        "offline core requires supplied core, executable and executable digest"
                    )
                executable = regular(
                    Path(request.executable), request.executable_sha256
                )
                record["subject"] = {
                    "kind": "offline-core",
                    "executable": executable,
                    "core": regular(Path(request.core), hash_contents=False),
                }
                value = step(
                    [
                        *base,
                        "-ex",
                        "thread apply all bt full",
                        "-ex",
                        "info sharedlibrary",
                        request.executable,
                        request.core,
                    ],
                    "gdb",
                )
            else:
                if not command:
                    raise ValueError("gdb-launch requires literal executable argv")
                executable = Path(shutil.which(command[0]) or command[0]).resolve(
                    strict=True
                )
                record["subject"] = {
                    "kind": "owned-launch",
                    "argv": command,
                    "executable": regular(executable),
                }
                events = "python import gdb,json; gdb.events.stop.connect(lambda e: print('PSE_ACTION_TARGET '+json.dumps({'signal':getattr(e,'stop_signal',None)}))); gdb.events.exited.connect(lambda e: print('PSE_ACTION_TARGET '+json.dumps({'exit_code':getattr(e,'exit_code',None)})))"
                if request.faulthandler:
                    base += ["-ex", "set environment PYTHONFAULTHANDLER 1"]
                value = step(
                    [
                        *base,
                        "-ex",
                        events,
                        "-ex",
                        "run",
                        "-ex",
                        "thread apply all bt full",
                        "-ex",
                        "info sharedlibrary",
                        "-ex",
                        "python gdb.execute('continue') if gdb.selected_inferior().pid else None",
                        "--args",
                        *command,
                    ],
                    "gdb",
                )
                text = (output / "gdb.stdout.txt").read_text(errors="replace")
                record["target_outcome"] = [
                    json.loads(line.split("PSE_ACTION_TARGET ", 1)[1])
                    for line in text.splitlines()
                    if line.startswith("PSE_ACTION_TARGET ")
                ]
                record["faulthandler"] = (
                    "selected CPython fault handler; Python/C stack support is interpreter/platform dependent"
                    if request.faulthandler
                    else "not selected"
                )
            record["coverage"] = (
                "GDB-reported threads/libraries and symbols; optimized-out/missing frames remain raw; no portable core or library copying"
            )
            record["status"] = "collected" if value["exit_code"] == 0 else "unavailable"
        elif request.action in {"python-perf", "sched", "heaptrack", "stat"}:
            if not command:
                raise ValueError(
                    "selected launch action requires literal workload argv"
                )
            tool = (
                selected_tool("heaptrack")
                if request.action == "heaptrack"
                else selected_tool(
                    "perf",
                    Path(request.perf_binary)
                    if request.perf_binary is not None
                    else None,
                )
            )
            record.update(
                tool=tool,
                subject={"kind": "owned-launch", "argv": command},
                effect="allocation interception"
                if request.action == "heaptrack"
                else "selected owned experimental capture",
            )
            if tool["status"] != "available":
                raise ValueError(tool["reason"])
            required = []
            subject: dict = {}
            if request.action == "python-perf":
                if Path(command[0]).resolve() != Path(sys.executable).resolve():
                    raise ValueError(
                        "Python perf requires the selected interpreter as the literal executable"
                    )
                instrumented = [
                    command[0],
                    "-X",
                    "perf_jit" if request.jit else "perf",
                    *command[1:],
                ]
                raw = output / "perf.data"
                subject_file = output / "python-subject.json"
                selected_launch = identified_launch(instrumented, subject_file)
                value = step(
                    [
                        tool["path"],
                        "record",
                        "-e",
                        "cpu-clock:u",
                        "-F",
                        "99",
                        *(["-k", "1"] if request.jit else []),
                        "--call-graph",
                        "dwarf,16384" if request.jit else "fp",
                        "-o",
                        str(raw),
                        "--",
                        *selected_launch,
                    ],
                    "perf-record",
                )
                if subject_file.exists():
                    subject = resource_store.read(subject_file)
                    record["subject"]["runtime_identity"] = subject
                    retained_symbols = []
                    for extension, prefix in (("map", "perf"), ("dump", "jit")):
                        original = (
                            CPYTHON_SYMBOL_DIRECTORY
                            / f"{prefix}-{subject['pid']}.{extension}"
                        )
                        try:
                            retain_symbol(
                                original,
                                output / f"python.{extension}",
                                subject["wall_ns"],
                            )
                            retained_symbols.append(
                                {
                                    "original": str(original),
                                    "member": f"python.{extension}",
                                    "status": "retained",
                                }
                            )
                        except (OSError, ValueError) as error:
                            retained_symbols.append(
                                {
                                    "original": str(original),
                                    "status": "unavailable",
                                    "reason": str(error),
                                }
                            )
                    record["python_symbols"] = retained_symbols
                decoded = raw
                if request.jit and raw.exists():
                    decoded = output / "perf.jit.data"
                    required.append(
                        step(
                            [
                                tool["path"],
                                "inject",
                                "--jit",
                                "-i",
                                str(raw),
                                "-o",
                                str(decoded),
                            ],
                            "perf-inject",
                            cwd=output,
                        )
                    )
                    if subject_file.exists():
                        candidates = sorted(
                            CPYTHON_SYMBOL_DIRECTORY.glob(
                                f"jitted-{subject['pid']}-*.so"
                            )
                        )
                        record["jit_symbol_retention"] = {
                            "status": "complete"
                            if len(candidates) <= 512
                            else "partial",
                            "files": [],
                        }
                        symbols = output / "jit-symbols"
                        symbols.mkdir(mode=0o700)
                        for original in candidates[:512]:
                            selected = symbols / original.name
                            retain_symbol(
                                original,
                                selected,
                                subject["wall_ns"],
                                maximum_bytes=1 << 20,
                            )
                            record["jit_symbol_retention"]["files"].append(
                                {
                                    "original": str(original),
                                    "member": f"jit-symbols/{original.name}",
                                    **regular(selected),
                                }
                            )
                if decoded.exists() and decoded.stat().st_size:
                    required.append(
                        step(
                            [tool["path"], "script", "-i", str(decoded)],
                            "perf-stacks",
                            environment_remove=()
                            if request.debuginfod
                            else ("DEBUGINFOD_URLS",),
                        )
                    )
                else:
                    required.append(
                        {
                            "status": "unavailable",
                            "exit_code": None,
                            "reason": "no nonempty perf data for required stack decoding",
                        }
                    )
                stacks = output / "perf-stacks.stdout.txt"
                observed_python = stacks.exists() and "py::" in stacks.read_text(
                    errors="replace"
                )
                record["python_attribution"] = {
                    "observed_python_symbols": observed_python,
                    "meaning": "decoded Python trampoline names actually observed"
                    if observed_python
                    else "decoder ran without established Python symbol attribution; native/unknown stacks remain raw",
                }
                if not observed_python:
                    required.append(
                        {
                            "status": "unavailable",
                            "exit_code": None,
                            "reason": "Python symbol attribution was not observed in decoded stacks",
                        }
                    )
                record["coverage"] = (
                    "actual decoded CPU stacks; retained map/JIT data and local interpreter/library identities; raw perf data references original symbol paths, retained copies do not rewrite them; Samply import not established"
                )
            elif request.action == "sched":
                raw = output / "sched.data"
                subject_file = output / "launch-subject.json"
                value = step(
                    [
                        tool["path"],
                        "sched",
                        "record",
                        "-o",
                        str(raw),
                        "--",
                        *identified_launch(command, subject_file),
                    ],
                    "sched-record",
                )
                if value["exit_code"] == 0 and raw.exists() and subject_file.exists():
                    subject = resource_store.read(subject_file)
                    record["subject"]["runtime_identity"] = subject
                    required.append(
                        step(
                            [
                                tool["path"],
                                "sched",
                                "timehist",
                                "-i",
                                str(raw),
                                "--pid",
                                str(subject["pid"]),
                                "--state",
                                "--wakeups",
                                "--with-summary",
                            ],
                            "sched-timehist",
                        )
                    )
                else:
                    required.append(
                        {
                            "status": "unavailable",
                            "exit_code": None,
                            "reason": "no scheduler capture for required timehist",
                        }
                    )
                record["coverage"] = (
                    "perf sched record defaults to system-wide (-a) scheduler tracepoints, including unrelated host tasks; owned launch identifies the experiment, not an isolated captured population; timehist and summary filter the owned exec PID, with recorded PID/start/boot and actual thread IDs in the trace; permission refusal is a prerequisite, never CPU/proc substitution"
                )
            elif request.action == "heaptrack":
                value = step(
                    [
                        tool["path"],
                        "--record-only",
                        "-o",
                        str(output / "heaptrack.data"),
                        *command,
                    ],
                    "heaptrack",
                )
                printer = selected_tool("heaptrack_print")
                record["decoder"] = printer
                raw_files = list(output.glob("heaptrack.data*"))
                if printer["status"] == "available" and raw_files:
                    required.append(
                        step(
                            [
                                printer["path"],
                                "-f",
                                str(raw_files[0]),
                                "-n",
                                "10",
                                "-s",
                                "3",
                            ],
                            "heaptrack-summary",
                        )
                    )
                else:
                    required.append(
                        {
                            "status": "unavailable",
                            "exit_code": None,
                            "reason": "required Heaptrack summary decoder/data unavailable",
                        }
                    )
                record["coverage"] = (
                    "libc allocation functions and observed allocating frames; excludes custom pools/suballocations; not RSS, reservations or operation-private peaks"
                )
            else:
                events = (
                    request.events
                    or "cycles:u,instructions:u,branches:u,branch-misses:u,cache-misses:u"
                )
                if not re.fullmatch(r"[A-Za-z0-9_:,./=-]+", events):
                    raise ValueError("invalid explicit perf event selection")
                value = step(
                    [
                        tool["path"],
                        "stat",
                        "--json-output",
                        "-o",
                        str(output / "stat.json"),
                        "-e",
                        events,
                        "--",
                        *command,
                    ],
                    "perf-stat",
                )
                record["coverage"] = (
                    "selected event counts with actual running/multiplexing fractions; unsupported counters remain raw refusals"
                )
            record["capture_status"] = (
                "collected" if value["exit_code"] == 0 else "unavailable"
            )
            record["decoders"] = required
            record["status"] = (
                "unavailable"
                if value["exit_code"] != 0
                else "partial"
                if any(
                    item["exit_code"] != 0 or item["status"] != "returned"
                    for item in required
                )
                else "collected"
            )
        elif request.action == "memray":
            tool = memray_runtime(
                Path(request.memray_python)
                if request.memray_python is not None
                else None
            )
            if not command or Path(command[0]).absolute() != Path(tool["path"]):
                raise ValueError(
                    "Memray workload must select the exact isolated interpreter, followed by a script, -m or -c"
                )
            record.update(
                tool=tool,
                subject={"kind": "owned-launch", "argv": command},
                effect="Python allocator and native allocation interception",
            )
            raw = output / "memray.bin"
            value = step(
                [
                    tool["path"],
                    "-m",
                    "memray",
                    "run",
                    "--native",
                    "--trace-python-allocators",
                    "-o",
                    str(raw),
                    *command[1:],
                ],
                "memray-record",
            )
            record["symbol_resolution"] = (
                "configured debuginfod queries allowed"
                if request.debuginfod
                else "local symbols only; DEBUGINFOD_URLS removed for the decoder"
            )
            decoded = (
                step(
                    [tool["path"], "-m", "memray", "stats", str(raw)],
                    "memray-stats",
                    environment_remove=()
                    if request.debuginfod
                    else ("DEBUGINFOD_URLS",),
                )
                if raw.exists()
                else {
                    "exit_code": None,
                    "status": "unavailable",
                    "reason": "capture absent",
                }
            )
            record["capture_status"] = "collected" if raw.exists() else "unavailable"
            record["target_outcome"] = {
                "launcher_exit": value["exit_code"],
                "meaning": "Memray propagates workload failures; capture and decoder statuses remain independent",
            }
            record["decoders"] = [decoded]
            record["coverage"] = (
                "Python allocator suballocations plus Python/native allocating frames; new Python threads supported; Rust-owned threads have native allocation attribution only, custom allocator pools and exec descendants are excluded"
            )
            record["status"] = (
                "unavailable"
                if not raw.exists()
                else "partial"
                if decoded["exit_code"] != 0
                else "collected"
            )
        elif request.action in {"rr-record", "rr-replay"}:
            tool = selected_tool(
                "rr",
                Path(request.rr_binary)
                if request.rr_binary is not None
                else diagnostic_trace.binary_path("rr"),
            )
            record.update(
                tool=tool,
                effect="owned record/replay experiment; no attachment to the original process",
            )
            if tool["status"] != "available":
                raise ValueError(tool["reason"])
            runtime_root = Path(tool["path"]).parent.parent
            if request.rr_binary is None:
                diagnostic_trace.verify_bundle(
                    runtime_root, diagnostic_trace.tool_spec("rr")
                )
            record["runtime_bundle"] = {
                "path": str(runtime_root),
                "libraries": [
                    regular(path)
                    for path in sorted((runtime_root / "lib/rr").glob("*.so"))[:32]
                ],
                "stub": regular(runtime_root / "bin/rr_exec_stub"),
            }
            if not record["runtime_bundle"]["libraries"]:
                raise ValueError("rr requires its complete portable runtime bundle")
            if request.action == "rr-record":
                if not command:
                    raise ValueError("rr-record requires literal executable argv")
                executable = Path(shutil.which(command[0]) or command[0]).resolve(
                    strict=True
                )
                record["subject"] = {
                    "kind": "owned-launch",
                    "argv": command,
                    "executable": regular(executable),
                }
                trace = output / "trace"
                value = step(
                    [tool["path"], "record", "-o", str(trace), *command], "rr-record"
                )
                if not trace.exists():
                    raise ValueError(
                        "rr did not produce a trace; recorder prerequisites or target launch failed"
                    )
                record["trace"] = trace_reference(trace)
                decoded = step([tool["path"], "ps", str(trace)], "rr-processes")
                record["target_outcome"] = {
                    "record_exit": value["exit_code"],
                    "process_table": "rr-processes.stdout.txt",
                    "meaning": "recorder exit can propagate target failure; rr ps retains each recorded process exit",
                }
            else:
                if request.trace is None or request.trace_sha256 is None:
                    raise ValueError(
                        "rr-replay requires exact --trace and --trace-sha256 from the recording action"
                    )
                trace = Path(request.trace).absolute()
                record["trace"] = trace_reference(trace, request.trace_sha256)
                record["subject"] = {
                    "kind": "offline-recorded-processes",
                    "trace": str(trace),
                }
                gdb = selected_tool("gdb")
                if gdb["status"] != "available":
                    raise ValueError(gdb["reason"])
                record["decoder"] = gdb
                if request.breakpoint is None or not re.fullmatch(
                    r"[A-Za-z_][A-Za-z0-9_:]*", request.breakpoint
                ):
                    raise ValueError(
                        "rr reverse control requires an explicit symbol --breakpoint"
                    )
                value = step(
                    [
                        tool["path"],
                        "replay",
                        "-d",
                        gdb["path"],
                        str(trace),
                        "--",
                        "-nx",
                        "-batch",
                        "-iex",
                        "set auto-load off",
                        "-ex",
                        "set debuginfod enabled off",
                        "-ex",
                        "set breakpoint pending on",
                        "-ex",
                        "break " + request.breakpoint,
                        "-ex",
                        "continue",
                        "-ex",
                        "reverse-stepi",
                        "-ex",
                        "bt",
                    ],
                    "rr-reverse",
                )
                decoded = value
                record["trace_after"] = trace_reference(trace, request.trace_sha256)
            record["coverage"] = (
                "selected Linux process tree, retained rr trace and local executable/library identities; CPU/kernel/syscall/shared-memory feasibility is experiment-specific; complete portable rr bundle required"
            )
            record["status"] = (
                "collected"
                if decoded["exit_code"] == 0 and value["status"] == "returned"
                else "partial"
            )
        elif request.action == "feasibility":
            record.update(
                effect="read installed optional tool and host prerequisites",
                subject={"kind": "host-feasibility"},
            )
            record["tools"] = {
                name: selected_tool(name)
                for name in ("rr", "memray", "perf", "gdb", "py-spy", "heaptrack")
            }
            record["host"] = {
                name: Path(path).read_text().strip()
                for name, path in (
                    ("perf_event_paranoid", "/proc/sys/kernel/perf_event_paranoid"),
                    ("ptrace_scope", "/proc/sys/kernel/yama/ptrace_scope"),
                    ("core_pattern", "/proc/sys/kernel/core_pattern"),
                )
            }
            record["rr"] = {
                "decision": "optional selected recorder",
                "qualification": "presence is not record/replay qualification; use rr-record then rr-replay on the actual process tree",
                "scope": "current official rr supports selected AMD Zen hosts; exact CPU/kernel/syscall/shared-memory support requires an actual recorder probe",
            }
            record["memray"] = {
                "decision": "optional isolated pinned Python allocation route",
                "qualification": "requires --memray-python matching the diagnostics dependency group and an actual capture",
                "coverage_gap": "Python allocator suballocations and Python-source allocation attribution beyond libc allocation summaries; native mode attributes observed native allocator frames",
            }
            record["status"] = "observed"
        else:
            raise ValueError("unsupported active action")

    try:
        perform()
    except ActionInterruptedError as error:
        record.update(status="interrupted", error=str(error))
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        record.update(status="unavailable", error=str(error))
    finally:
        publish(output, record)
    return record


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__)
    command.add_argument(
        "action",
        choices=(
            "read",
            "python-snapshot",
            "python-sample",
            "python-perf",
            "gdb-launch",
            "gdb-core",
            "sched",
            "heaptrack",
            "stat",
            "feasibility",
            "memray",
            "rr-record",
            "rr-replay",
        ),
    )
    command.add_argument(
        "--output",
        type=Path,
        required=True,
        help="fresh private action directory; existing directory for read",
    )
    command.add_argument("--seconds", type=float, default=10)
    command.add_argument("--output-bytes", type=int, default=8 << 20)
    command.add_argument("--pid", type=int)
    command.add_argument("--start")
    command.add_argument("--native", action="store_true")
    command.add_argument("--nonblocking", action="store_true")
    command.add_argument("--jit", action="store_true")
    command.add_argument("--faulthandler", action="store_true")
    command.add_argument("--core", type=Path)
    command.add_argument("--executable", type=Path)
    command.add_argument("--executable-sha256")
    command.add_argument("--events")
    command.add_argument("--memray-python", type=Path)
    command.add_argument(
        "--debuginfod",
        action="store_true",
        help="allow configured symbol downloads for perf/Memray decoders; default local symbols only",
    )
    command.add_argument(
        "--rr-binary", type=Path, help="bin/rr from the complete pinned portable bundle"
    )
    command.add_argument("--trace", type=Path)
    command.add_argument("--trace-sha256")
    command.add_argument(
        "--breakpoint", help="exact symbol for the selected rr reverse-step control"
    )
    command.add_argument(
        "--perf-binary",
        type=Path,
        help="explicit perf ELF/tool selection; SHA and version retained",
    )
    command.add_argument(
        "--operation", help="optional attribution only; does not authorize attachment"
    )
    command.add_argument(
        "--invocation", help="optional authentic invocation identity/declaration"
    )
    command.add_argument("--registry", type=Path)
    command.add_argument("--tail", type=int, default=20)
    return command


def main(argv: list[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    split = arguments.index("--") if "--" in arguments else len(arguments)
    args = parser().parse_args(arguments[:split])
    args.command = arguments[split + 1 :]

    try:
        if args.action == "read":
            record = read_action(args.output.absolute(), tail=args.tail)
        else:
            values = vars(args).copy()
            values.pop("tail")
            for field in (
                "output",
                "core",
                "executable",
                "memray_python",
                "rr_binary",
                "trace",
                "perf_binary",
                "registry",
            ):
                if values[field] is not None:
                    values[field] = str(values[field])
            record = execute(ActionRequest.model_validate(values))
        print(json.dumps(record, indent=2))
        return (
            0
            if args.action == "read" or record["status"] in {"collected", "observed"}
            else 1
        )
    except (OSError, ValueError, RuntimeError) as error:
        print(json.dumps({"schema": SCHEMA, "error": str(error)}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

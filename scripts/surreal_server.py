# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Own the released local SurrealDB server, its private state and resource budget.

Application admission and native-worker drain belong to runtime composition. The
supervisor records that admission state; privileged direct database clients must
not be used to bypass it. See docs/dev/surreal-substrate.md.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import errno
import fcntl
import hashlib
import http.client
import json
import math
import os
import platform
import re
import secrets
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
import tomllib
import urllib.error
import urllib.request
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

# Recorded receivers launch this file directly, independently of the caller's
# working directory. Its sibling capability owners belong to this installation.
if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts import (
    host_admission,
    native_operation,
    test_resources,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping


MIB = 1024 * 1024
GIB = 1024 * MIB
_POLICY_PATH = (
    Path(__file__).resolve().parents[1]
    / "crates/pse-operations/src/generated/surreal-policy.json"
)
_POLICY = json.loads(_POLICY_PATH.read_text())
MESSAGE_BYTES = _POLICY["max_message_bytes"]
SUBSTRATE_INTERPRETATION = _POLICY["interpretation"]
OWNER = "pse-arrow-surreal-v1"
RELEASE_API = "https://api.github.com/repos/surrealdb/surrealdb/releases"
SCRIPT = Path(__file__).resolve()
_STARTUP = threading.local()
_LIFECYCLE = threading.local()
WORKER_CAPABILITIES = ("solver", "klu", "isolation", "uno", "petsc")


class SupervisorError(RuntimeError):
    """An actionable lifecycle error with no credential-bearing output."""


def read_json(path: Path, *, deadline: float | None = None) -> dict[str, object]:
    if deadline is not None:
        return mcp_private_record(path, deadline)[0]
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SupervisorError(f"Expected an object in {path.name}")
    return value


def object_mapping(value: object) -> dict[str, object]:
    """Narrow an owned metadata object before consuming its named fields."""
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise SupervisorError("Expected a string-keyed owned metadata object")
    return value


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise SupervisorError("Expected an integer in owned configuration")
    return value


def write_json(path: Path, value: Mapping[str, object]) -> None:
    """Replace private metadata atomically and persist the directory entry."""
    fd, name = tempfile.mkstemp(prefix=f".{path.name}-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        Path(name).replace(path)
        directory = os.open(path.parent, os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        Path(name).unlink(missing_ok=True)


def lexical_absolute(path: Path) -> Path:
    """Normalize lexical aliases without following symlinks or hiding their identity."""
    # Path.resolve follows links; Path.absolute alone retains '..' aliases. This
    # lexical normalization preserves the precheck and stable systemd slot names.
    return Path(os.path.abspath(path))  # noqa: PTH100 -- Must normalize '..' without following symlinks.


def checked_directory(path: Path, *, empty: bool = False) -> Path:
    path = path.absolute()
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise SupervisorError("State and backup paths must not traverse symlinks")
    path = lexical_absolute(path)
    if path.exists():
        if not path.is_dir() or path.stat().st_uid != os.getuid():
            raise SupervisorError("Directory must be owned by the current user")
        if empty and any(path.iterdir()):
            raise SupervisorError(
                "Refusing to initialize or replace a nonempty directory"
            )
    else:
        path.mkdir(parents=True, mode=0o700)
    path.chmod(0o700)
    return path


@contextlib.contextmanager
def state_lock(state: Path) -> Generator[None, None, None]:
    fd = os.open(
        state / ".supervisor.lock", os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600
    )
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        os.close(fd)


def publish_generation(
    state: Path, worker: Path | None = None, producer: Path | None = None
) -> dict[str, str]:
    """Freeze the complete owned Python/configuration closure before launch."""
    root = SCRIPT.parents[1]
    sources = [*sorted((root / "scripts").glob("*.py")), root / "scripts/sccache"]
    for relative in (
        ".config/agent-capacity.toml",
        ".config/build.toml",
        ".config/sccache.toml",
        ".cargo/config.toml",
        "packages/reference/conformance.toml",
        "docker/solvers/Dockerfile",
        "Cargo.toml",
        "rust-toolchain.toml",
        ".python-version",
        "crates/pse-operations/src/generated/surreal-policy.json",
    ):
        sources.append(root / relative)
    entries = {str(path.relative_to(root)): file_digest(path) for path in sources}
    if worker is not None:
        entries["bin/pse-worker"] = file_digest(worker)
    if producer is not None:
        entries["producer/worker.json"] = file_digest(producer)
    identity = hashlib.sha256(
        json.dumps(entries, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    generations = state / ".generations"
    generations.mkdir(mode=0o700, exist_ok=True)
    destination = generations / identity
    if not destination.exists():
        pending = generations / ("pending-" + uuid.uuid4().hex)
        pending.mkdir(mode=0o700)
        for source in sources:
            target = pending / source.relative_to(root)
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            target.chmod(0o700 if source == root / "scripts/sccache" else 0o600)
        if worker is not None:
            target = pending / "bin/pse-worker"
            target.parent.mkdir(mode=0o700, exist_ok=True)
            shutil.copyfile(worker, target)
            target.chmod(0o700)
        if producer is not None:
            target = pending / "producer/worker.json"
            target.parent.mkdir(mode=0o700, exist_ok=True)
            shutil.copyfile(producer, target)
            target.chmod(0o600)
        if any(
            file_digest(pending / name) != expected
            for name, expected in entries.items()
        ):
            raise SupervisorError(
                "Source changed while publishing immutable generation; pending bytes preserved"
            )
        write_json(
            pending / "generation.json",
            {"version": 1, "identity": identity, "files": entries},
        )
        # Concurrent complete generations have identical content; preserve pending.
        try:
            pending.rename(destination)
        except OSError as error:
            if error.errno not in {errno.EEXIST, errno.ENOTEMPTY}:
                raise
    verify_generation(destination)
    return {
        "supervisor_executable": str(Path(sys.executable).resolve()),
        "supervisor_script": str(destination / "scripts/surreal_server.py"),
        "supervisor_sha256": entries["scripts/surreal_server.py"],
        **(
            {
                "worker_executable": str(destination / "bin/pse-worker"),
                "worker_sha256": entries["bin/pse-worker"],
            }
            if worker is not None
            else {}
        ),
    }


@contextlib.contextmanager
def lifecycle_reservation(state: Path) -> Generator[None, None, None]:
    """Reserve an operation with short metadata exclusion, never an IPC-held lock."""
    state = service_directory(state)
    if getattr(_LIFECYCLE, "state", None) == state:
        yield
        return

    path = state / "lifecycle-owner.json"
    selected: dict[str, object] = {
        "nonce": uuid.uuid4().hex,
        "pid": os.getpid(),
        "start": native_operation.start_identity(os.getpid()),
        "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
    }
    with state_lock(state):
        if path.exists():
            previous = read_json(path)
            live = previous["boot"] == selected["boot"] and reservation_live(previous)
            if live:
                raise SupervisorError(
                    "Another live owner is changing this service; retry after its operation finishes"
                )
        write_json(path, selected)
    previous_state = getattr(_LIFECYCLE, "state", None)
    _LIFECYCLE.state = state
    try:
        yield
    finally:
        try:
            drain_maintenance_listener(state)
            with state_lock(state):
                if path.exists() and read_json(path).get("nonce") == selected["nonce"]:
                    path.unlink()
        finally:
            _LIFECYCLE.state = previous_state


def reservation_live(reservation: dict[str, object]) -> bool:

    if (
        reservation.get("boot") is not None
        and reservation["boot"]
        != Path("/proc/sys/kernel/random/boot_id").read_text().strip()
    ):
        return False
    try:
        # One kernel observation binds process state and start identity. An
        # unreaped zombie retains its PID/start tick but owns no live authority.
        process = (
            Path(f"/proc/{integer(reservation['pid'])}/stat")
            .read_text()
            .rsplit(")", 1)[1]
            .split()
        )
        return process[0] not in {"Z", "X", "x"} and process[19] == reservation["start"]
    except FileNotFoundError:
        return False


@contextlib.contextmanager
def context_admission(state: Path) -> Generator[None, None, None]:
    """Serialize only admission metadata against all service lifecycle changes."""
    service = service_directory(state)
    with state_lock(service):
        lifecycle = service / "lifecycle-owner.json"
        if lifecycle.exists() and reservation_live(read_json(lifecycle)):
            raise SupervisorError(
                "Service lifecycle operation has closed context admission"
            )
        config = config_for(service)
        if config["admission"] != "open" or not config["accepting_writes"]:
            raise SupervisorError("Service context admission is closed")
        if service == state:
            yield
        else:
            with state_lock(state):
                yield


def managed_contexts(state: Path) -> list[Path]:
    """Inventory registered generations while lifecycle admission is reserved."""
    contexts = [state]
    for descriptor in sorted((state / ".contexts").glob("*.json")):
        if descriptor.is_symlink():
            raise SupervisorError("Context descriptor must not traverse symlinks")
        context = read_json(descriptor)
        selected = receiver_context(state, str(context["database"]))
        if selected == state:
            raise SupervisorError("Registered receiver context is missing")
        contexts.append(selected)
    return contexts


def close_context_admission(state: Path, *, admission: str = "quiesced") -> None:
    # No admission can pass the service reservation while these small writes run.
    for context in managed_contexts(state):
        with state_lock(context):
            config = config_for(context)
            config["accepting_writes"] = False
            if config["admission"] != "validation_required":
                config["admission"] = admission
            write_json(context / "config.json", config)


def open_context_admission(state: Path) -> None:
    for context in managed_contexts(state):
        with state_lock(context):
            config = config_for(context)
            if config.get("derived_rebuild_pending"):
                raise SupervisorError("Incomplete derived cutover remains closed")
            config.update(accepting_writes=True, admission="open")
            write_json(context / "config.json", config)


def require_borrowers_drained(state: Path) -> None:
    """Native-free registered contexts also own an admitted service lifetime."""
    owner = service_directory(state)
    for record in test_resources.resource_status().values():
        if not isinstance(record, dict):
            raise SupervisorError("Unresolved service resource registry entry")
        if record.get("kind") == "evidence" or record.get("cleanup") == "removed":
            continue
        selected = Path(record.get("state", ""))
        if selected != owner and owner / ".receivers" not in selected.parents:
            continue
        if selected != owner and service_directory(selected) != owner:
            raise SupervisorError("Live context has an unresolved service association")
        cleaner = record.get("cleaner")
        if (
            record.get("cleanup") == "removing"
            and isinstance(cleaner, dict)
            and test_resources.borrower_alive(cleaner)
        ):
            raise SupervisorError(
                "A live context cleanup still borrows this storage service"
            )
        if not record.get("drained") and test_resources.borrower_alive(record):
            raise SupervisorError(
                "A live test context still borrows this storage service"
            )


def all_contexts_drained(state: Path) -> None:
    require_borrowers_drained(state)
    for context in managed_contexts(state):
        for pending in [
            context / "primary-admission.json",
            context / "observer-launch.json",
            context / "primary-observer.json",
            *context.glob("worker-admission-*.json"),
        ]:
            if pending.exists() and reservation_live(read_json(pending)):
                raise SupervisorError(
                    "Context launch reservation remains active; wait for admission to settle"
                )
        if observer_launch_busy(context):
            raise SupervisorError(
                "Context observer launch remains active; drain before offline action"
            )
        observer = context / "primary-observer.json"
        if observer.exists():
            registration = read_json(observer)
            # verify_observer records actual membership for an existing process,
            # which need not have a supervisor-created unit name. A dead leader
            # does not free that recorded group while descendants survive.
            group = registration.get("group", "")
            if registration.get("unit"):
                group = systemctl(
                    "show",
                    "--property=ControlGroup",
                    "--value",
                    str(registration["unit"]),
                    check=False,
                ).stdout.strip()
            if not isinstance(group, str):
                raise SupervisorError(
                    "Context observer lacks recorded kernel membership"
                )
            if group.startswith("/") and group_populated(group):
                raise SupervisorError(
                    "Context observer process group remains populated; drain before offline action"
                )
        workers_drained(context, config_for(context))


def owned_generations(state: Path) -> set[Path]:
    generations: set[Path] = set()
    for context in managed_contexts(state):
        config = config_for(context)
        for role in ("service_supervisor", "primary_receiver"):
            receiver = config.get(role)
            if isinstance(receiver, dict) and receiver.get("supervisor_script"):
                generation = Path(str(receiver["supervisor_script"])).parents[1]
                if generation.parent != state / ".generations":
                    raise SupervisorError(
                        "Admitted source generation is outside this service state"
                    )
                if generation not in generations:
                    verify_generation(generation)
                    generations.add(generation)
    return generations


def readmit_supervisor(state: Path) -> dict[str, object]:
    """Explicitly publish current owned code without modifying retained generations."""
    with lifecycle_reservation(state):
        return _readmit_supervisor(state)


def _readmit_supervisor(state: Path) -> dict[str, object]:
    config = config_for(state)
    if (
        active(state)
        or config.get("accepting_writes")
        or config.get("admission") != "quiesced"
    ):
        raise SupervisorError(
            "Readmission requires this selected service stopped and quiesced"
        )
    close_context_admission(state)
    all_contexts_drained(state)
    history = state / ".profile-history"
    history.mkdir(mode=0o700, exist_ok=True)
    write_json(history / (uuid.uuid4().hex + ".json"), config)
    credentials = read_json(state / "credentials.json")
    if (
        "selection_username" not in credentials
        or "selection_password" not in credentials
    ):
        write_json(history / (uuid.uuid4().hex + "-credentials.json"), credentials)
        credentials.update(
            selection_username="pse-selection",
            selection_password=secrets.token_urlsafe(36),
        )
        write_json(state / "credentials.json", credentials)
    supervisor = publish_generation(state)
    server = dict(object_mapping(config["server"]))
    server["binary_sha256"] = file_digest(Path(str(server["binary"])))
    with state_lock(state):
        config = config_for(state)
        config.update(
            service_supervisor=supervisor, server=server, restart_qualified=False
        )
        write_json(state / "config.json", config)
    return public_status(state, config)


def verify_generation(directory: Path) -> None:
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise SupervisorError("Immutable generation must not traverse symlinks")
    receipt = read_json(directory / "generation.json")
    if (
        receipt.get("version") != 1
        or receipt.get("identity") != directory.name
        or not isinstance(receipt.get("files"), dict)
    ):
        raise SupervisorError("Missing immutable generation identity")
    files = receipt["files"]
    identity = hashlib.sha256(
        json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    if identity != directory.name:
        raise SupervisorError("Immutable generation declaration changed")
    for name, expected in files.items():
        path = directory / name
        if (
            not path.is_relative_to(directory)
            or ".." in Path(name).parts
            or path.is_symlink()
            or file_digest(path) != expected
        ):
            raise SupervisorError("Immutable supervisor/receiver closure changed")
        if (
            name in {"scripts/sccache", "bin/pse-worker"}
            and path.stat().st_mode & 0o700 != 0o700
        ):
            raise SupervisorError(
                "Immutable executable closure lost its admitted execution mode"
            )


def execution_resources(name: str) -> dict[str, object]:
    if name == "plan28-reference" or name == "reference":
        return reference_resources()

    declared = host_admission.settings(name)
    execution = host_admission.execution(name)
    server = (
        host_admission.settings("timing" if name == "timing" else "functional")[
            "store_gib"
        ]
        * GIB
    )
    worker = declared["receiver_gib"] * GIB
    total = server + declared["slot_gib"] * GIB
    return resources(total, server, 1, worker, execution)


def register_context(
    state: Path, database: str, profile: str, worker: Path | None = None
) -> dict[str, object]:
    """Create an explicit receiver/context descriptor without mutating service selection."""
    config = config_for(state)
    timing_service = config.get("service_class") == "timing"
    if (profile == "timing") != timing_service:
        raise SupervisorError("Timing contexts require their dedicated timing service")
    if (
        not database
        or len(database) > 128
        or any(
            not (character.isascii() and (character.isalnum() or character == "_"))
            for character in database
        )
    ):
        raise SupervisorError("Context database requires a bounded ASCII identity")
    with context_admission(state):
        if (state / ".contexts" / f"{database}.json").exists():
            raise SupervisorError(
                "Context is already registered; use its explicit owner"
            )
    allocation = execution_resources(profile)
    producer, producer_digest = selected_worker_receipt(systemd_environment())
    receiver = (
        publish_generation(state, worker, Path(producer) if producer else None)
        if worker is not None
        else None
    )
    directory = state / ".contexts"
    target = directory / f"{database}.json"
    if target.exists():
        raise SupervisorError("Context is already registered; use its explicit owner")
    identity = hashlib.sha256(
        json.dumps(
            {
                "receiver": receiver,
                "resources": allocation,
                "producer": producer,
                "producer_digest": producer_digest,
            },
            sort_keys=True,
        ).encode()
    ).hexdigest()
    receiver_state = state / ".receivers" / identity / database
    selected = dict(config)
    selected.update(
        database=database,
        resources=allocation,
        primary_receiver=receiver,
        service_state=str(state),
        receiver_generation=identity,
        resident=False,
    )
    if producer is not None and receiver is not None:
        selected["worker_producer_receipt"] = {
            "path": str(
                Path(receiver["supervisor_script"]).parents[1] / "producer/worker.json"
            ),
            "sha256": producer_digest,
            "origin": producer,
        }
    with context_admission(state):
        if target.exists():
            raise SupervisorError(
                "Context was registered concurrently; use its explicit owner"
            )
        directory.mkdir(mode=0o700, exist_ok=True)
        receiver_state.mkdir(mode=0o700, parents=True, exist_ok=False)
        selected.update(config_for(state))
        selected.update(
            database=database,
            resources=allocation,
            primary_receiver=receiver,
            service_state=str(state),
            receiver_generation=identity,
            resident=False,
        )
        if producer is not None and receiver is not None:
            selected["worker_producer_receipt"] = {
                "path": str(
                    Path(receiver["supervisor_script"]).parents[1]
                    / "producer/worker.json"
                ),
                "sha256": producer_digest,
                "origin": producer,
            }
        write_json(receiver_state / "config.json", selected)
        write_json(
            target,
            {
                "version": 1,
                "database": database,
                "resources": allocation,
                "primary_receiver": receiver,
                "receiver_state": str(receiver_state),
                "receiver_generation": identity,
            },
        )
    return read_json(target)


def receiver_context(state: Path, database: str | None) -> Path:
    if database is None:
        return state
    path = state / ".contexts" / f"{database}.json"
    if not path.exists():
        return state
    context = read_json(path)
    selected = Path(str(context["receiver_state"]))
    if (
        selected.is_symlink()
        or state / ".receivers" not in selected.parents
        or context["database"] != database
    ):
        raise SupervisorError("Invalid registered receiver/context association")
    config = config_for(selected)
    if (
        config.get("service_state") != str(state)
        or config["database"] != database
        or config.get("receiver_generation") != context["receiver_generation"]
    ):
        raise SupervisorError(
            "Receiver generation does not match selected service/context"
        )
    return selected


def service_directory(state: Path, *, deadline: float | None = None) -> Path:
    path = state / "config.json"
    if not path.is_file():
        return state
    config = read_json(path, deadline=deadline)
    selected = config.get("service_state")
    if selected is None:
        return state
    result = Path(str(selected))
    if result / ".receivers" not in state.parents:
        raise SupervisorError("Receiver state is not owned by the recorded service")
    return result


def upgrade_profile(state: Path) -> dict[str, object]:
    """Explicit selected v1 readmission; leave every other saved profile untouched."""
    state = checked_directory(state)
    with state_lock(state):
        old = read_json(state / "config.json")
        if old.get("owner") != OWNER or old.get("profile_version") != 1:
            raise SupervisorError("upgrade selects one owned revision1 profile")
        if (
            active(state)
            or old.get("admission") != "quiesced"
            or old.get("accepting_writes")
        ):
            raise SupervisorError(
                "Upgrade requires the selected profile stopped, quiesced and drained"
            )
        workers_drained(state, old)
        allocation = old["resources"]
        if not isinstance(allocation, dict):
            raise SupervisorError("Missing revision1 allocation")
        expected = resources(
            integer(allocation["total_memory_bytes"]),
            integer(allocation["server_memory_bytes"]),
            integer(allocation["native_workers"]),
            integer(allocation["native_worker_memory_bytes"]),
            allocation.get("execution"),
        )
        if (
            expected != allocation
            or old.get("grpc_max_message_bytes") != MESSAGE_BYTES
            or old.get("endpoint") != f"grpc://127.0.0.1:{old['port']}"
        ):
            raise SupervisorError(
                "Revision1 profile does not match its exact admission"
            )
        write_json(state / "profile-v1-preserved.json", old)
        updated = dict(old)
        updated.update(
            profile_version=2,
            endpoint=f"ws://127.0.0.1:{old['port']}",
            websocket_max_message_bytes=MESSAGE_BYTES,
        )
        updated.pop("grpc_max_message_bytes", None)
        receiver = old.get("primary_receiver")
        if isinstance(receiver, dict):
            updated["primary_receiver"] = publish_generation(
                state, Path(str(receiver["worker_executable"]))
            )
        updated["service_supervisor"] = publish_generation(state)
        updated["resident"] = False
        write_json(state / "config.json", updated)
    return public_status(state, config_for(state))


def config_for(state: Path, *, deadline: float | None = None) -> dict[str, object]:
    if (
        any(path.is_symlink() for path in (state, *state.parents))
        or (state / "config.json").is_symlink()
    ):
        raise SupervisorError("Owned state must not traverse symlinks")
    config = read_json(state / "config.json", deadline=deadline)
    if config.get("owner") != OWNER or config.get("profile_version") != 2:
        raise SupervisorError("Not an owned, supported SurrealDB state directory")
    if config.get("websocket_max_message_bytes") != MESSAGE_BYTES:
        raise SupervisorError("Unsupported native WebSocket message profile")
    budget = config.get("resources")
    if not isinstance(budget, dict):
        raise SupervisorError("Missing resource allocation")
    execution = budget.get("execution")
    if execution is not None:
        validate_execution(execution)
    expected = resources(
        integer(budget["total_memory_bytes"]),
        integer(budget["server_memory_bytes"]),
        integer(budget["native_workers"]),
        integer(budget["native_worker_memory_bytes"]),
        execution,
    )
    if budget != expected:
        raise SupervisorError(
            "Resource overrides do not match the allocated server budget"
        )
    port = integer(config["port"])
    if not 1 <= port <= 65535 or config["endpoint"] != f"ws://127.0.0.1:{port}":
        raise SupervisorError(
            "Only the recorded authenticated loopback endpoint is supported"
        )
    if (
        config["max_message_bytes"] != MESSAGE_BYTES
        or config["schema_interpretation"] != config["interpretation"]
    ):
        raise SupervisorError("Inconsistent substrate interpretation or message limit")
    if (
        not isinstance(config["accepting_writes"], bool)
        or not isinstance(config["admission"], str)
        or config["admission"]
        not in {"open", "quiescing", "quiesced", "validation_required"}
    ):
        raise SupervisorError("Invalid application admission state")
    return config


def resources(
    total: int, server: int, workers: int, worker: int, execution: object = None
) -> dict[str, object]:
    if (
        server < 512 * MIB
        or not server < total < 2**63
        or not 1 <= workers <= 32
        or worker < 64 * MIB
    ):
        raise SupervisorError(
            "Require server >=512MiB, 1..32 workers and positive finite budgets"
        )
    observer = 0
    if execution is not None:
        validate_execution(execution)
        if not isinstance(execution, dict):
            raise SupervisorError("Invalid execution configuration")
        observer = integer(execution["observer_memory_bytes"])
        if workers != 1 or worker != integer(execution["pool_memory_bytes"]) + integer(
            execution["process_headroom_bytes"]
        ):
            raise SupervisorError(
                "Execution requires one exact primary process allocation"
            )
    if execution is not None and server + worker + observer > total:
        raise SupervisorError(
            "Reference execution requires its exact shared memory envelope"
        )
    if server + workers * worker + observer > total:
        raise SupervisorError(
            "Server and managed native workers exceed the shared memory budget"
        )
    allocation: dict[str, object] = {
        "total_memory_bytes": total,
        "server_memory_bytes": server,
        "native_workers": workers,
        "native_worker_memory_bytes": worker,
        "rocksdb_block_cache_bytes": server // 4,
        "rocksdb_write_buffer_bytes": min(32 * MIB, server // 64),
        "rocksdb_write_buffers": 2,
        "memory_threshold_bytes": server * 3 // 4,
    }
    if execution is not None:
        allocation["execution"] = execution
    return allocation


def validate_execution(execution: object) -> None:

    allowed = [
        reference_execution(),
        *(
            host_admission.execution(name)
            for name in (
                "functional",
                "wide",
                "timing",
                "exclusive",
                "exclusive-observer",
            )
        ),
    ]
    if not isinstance(execution, dict) or execution not in allowed:
        raise SupervisorError(
            "Execution profile differs from its selected finite declaration"
        )


def reference_execution() -> dict[str, int]:
    """Materialize scientific capacities from their declaration; headroom is policy."""
    path = SCRIPT.parents[1] / "packages/reference/conformance.toml"
    settings = tomllib.loads(path.read_text())["settings"]
    execution = {
        "pool_memory_bytes": integer(settings["memory_limit_bytes"]),
        "worker_bytes": integer(settings["math_worker_bytes"]),
        "cpu_threads": integer(settings["threads"]),
        "case_lanes": integer(settings["threads"]),
        "math_jobs": integer(settings["math_jobs"]),
        "compiler_cores": integer(
            settings["preparation"]["compiler"]["optimization"]["cores"]
        ),
        "admission_wait_ms": 30_000,
        "process_headroom_bytes": 12 * GIB,
        "observer_memory_bytes": 4 * GIB,
    }
    if any(value <= 0 for value in execution.values()) or execution["case_lanes"] != 16:
        raise SupervisorError(
            "Selected reference execution requires finite sixteen-lane settings"
        )
    return execution


def reference_resources() -> dict[str, object]:
    execution = reference_execution()
    worker = execution["pool_memory_bytes"] + execution["process_headroom_bytes"]
    server = 16 * GIB
    total = server + worker + execution["observer_memory_bytes"]
    return resources(total, server, 1, worker, execution)


def primary_receiver(executable: Path | None) -> dict[str, str]:
    if executable is None:
        raise SupervisorError(
            "Selecting reference execution requires --worker-executable"
        )
    worker = executable.resolve(strict=True)
    interpreter = Path(sys.executable).resolve(strict=True)
    if not worker.is_file() or not os.access(worker, os.X_OK):
        raise SupervisorError("Primary receiver must name a built executable")
    return {
        "supervisor_executable": str(interpreter),
        "supervisor_script": str(SCRIPT),
        "worker_executable": str(worker),
        "supervisor_sha256": file_digest(SCRIPT),
        "worker_sha256": file_digest(worker),
    }


def fetched_json(url: str) -> dict[str, object]:
    if not url.startswith(f"{RELEASE_API}/"):
        raise SupervisorError(
            "Release metadata is outside the official HTTPS API origin"
        )
    request = urllib.request.Request(  # noqa: S310 -- Validated official HTTPS API origin above.
        url, headers={"User-Agent": "pse-arrow-local-supervisor"}
    )
    with urllib.request.urlopen(request, timeout=60) as response:  # noqa: S310 -- Only the validated official HTTPS API request is opened.
        value = json.load(response)
    if not isinstance(value, dict):
        raise SupervisorError("Release metadata is not an object")
    return value


def install(version: str | None, tool_root: Path) -> dict[str, str]:
    if platform.system() != "Linux" or platform.machine() not in {"x86_64", "aarch64"}:
        raise SupervisorError("Initial supervisor supports Linux amd64/arm64 only")
    arch = "amd64" if platform.machine() == "x86_64" else "arm64"
    if version is not None and (
        not version.startswith("v")
        or not all(c.isdigit() or c in "v.-" for c in version)
    ):
        raise SupervisorError(
            "Server version must be an official stable vX.Y.Z release"
        )
    release = fetched_json(
        f"{RELEASE_API}/latest" if version is None else f"{RELEASE_API}/tags/{version}"
    )
    tag = str(release["tag_name"])
    if release.get("prerelease") or release.get("draft"):
        raise SupervisorError("Only stable official releases are supported")
    filename = f"surreal-{tag}.linux-{arch}.tgz"
    assets = release["assets"]
    if not isinstance(assets, list):
        raise SupervisorError("Release has no assets")
    asset = next(
        (a for a in assets if isinstance(a, dict) and a.get("name") == filename), None
    )
    if not isinstance(asset, dict):
        raise SupervisorError("Official release has no binary for this platform")
    digest = str(asset.get("digest", ""))
    if not digest.startswith("sha256:") or len(digest) != 71:
        raise SupervisorError(
            "Official asset lacks a SHA256 digest; refusing an unchecked install"
        )
    url = str(asset["browser_download_url"])
    if not url.startswith("https://github.com/surrealdb/surrealdb/releases/download/"):
        raise SupervisorError("Release asset is outside the official download origin")
    directory = checked_directory(tool_root / tag / f"linux-{arch}")
    binary = directory / "surreal"
    receipt = directory / "release.json"
    with state_lock(directory):
        if receipt.exists() or binary.exists():
            if not receipt.exists() or not binary.exists():
                raise SupervisorError(
                    "Installed release is incomplete; use a new tool directory"
                )
            prior = read_json(receipt)
            if prior.get("archive_sha256") == digest.removeprefix(
                "sha256:"
            ) and prior.get("binary_sha256") == file_digest(binary):
                return {
                    "version": tag.removeprefix("v"),
                    "binary": str(binary),
                    "binary_sha256": file_digest(binary),
                    "archive_sha256": digest.removeprefix("sha256:"),
                }
            raise SupervisorError(
                "Installed binary or release receipt changed; use a new tool directory"
            )
        with tempfile.TemporaryDirectory(dir=directory) as scratch:
            archive = Path(scratch) / filename
            request = urllib.request.Request(  # noqa: S310 -- Official HTTPS asset origin validated above; content SHA256 verified below.
                url, headers={"User-Agent": "pse-arrow-local-supervisor"}
            )
            with (
                urllib.request.urlopen(request, timeout=60) as response,  # noqa: S310 -- Validated official HTTPS asset; bytes verified below.
                archive.open("wb") as output,
            ):
                shutil.copyfileobj(response, output)
            if file_digest(archive) != digest.removeprefix("sha256:"):
                raise SupervisorError(
                    "Official release archive failed SHA256 verification"
                )
            with tarfile.open(archive) as bundle:
                candidates = [
                    m
                    for m in bundle.getmembers()
                    if m.isfile() and Path(m.name).name == "surreal"
                ]
                if len(candidates) != 1:
                    raise SupervisorError(
                        "Official archive must contain exactly one server binary"
                    )
                source = bundle.extractfile(candidates[0])
                if source is None:
                    raise SupervisorError("Cannot extract official server binary")
                with source, binary.open("xb") as output:
                    shutil.copyfileobj(source, output)
            binary.chmod(0o700)
        actual = subprocess.run(
            [str(binary), "version"], capture_output=True, text=True, check=True
        ).stdout.strip()
        if tag.removeprefix("v") not in actual:
            raise SupervisorError(
                "Official binary version does not match release metadata"
            )
        write_json(
            receipt,
            {
                "version": tag.removeprefix("v"),
                "platform": f"linux-{arch}",
                "url": url,
                "archive_sha256": digest.removeprefix("sha256:"),
                "binary_sha256": file_digest(binary),
            },
        )
    return {
        "version": tag.removeprefix("v"),
        "binary": str(binary),
        "binary_sha256": file_digest(binary),
        "archive_sha256": digest.removeprefix("sha256:"),
    }


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def setup(args: argparse.Namespace) -> dict[str, object]:
    args.interpretation = args.interpretation or SUBSTRATE_INTERPRETATION
    if (args.state / "config.json").exists():
        config = config_for(args.state.absolute())
        if config["interpretation"] != args.interpretation:
            raise SupervisorError("Existing state has a different interpretation")
        allocation = config["resources"]
        if args.execution_profile:
            if not isinstance(allocation, dict):
                raise SupervisorError("Missing recorded execution allocation")
            if "execution" not in allocation:
                raise SupervisorError(
                    "Existing state requires offline reconfigure to select reference execution"
                )
            if allocation != execution_resources(args.execution_profile):
                raise SupervisorError(
                    "Existing execution profile differs; reconfigure offline"
                )
            expected_class = (
                "timing" if args.execution_profile == "timing" else "functional"
            )
            if config.get("service_class", "functional") != expected_class:
                raise SupervisorError(
                    "Existing storage lane differs; reconfigure offline"
                )
        return public_status(args.state.absolute(), config)
    # Download first so network failure leaves no half-initialized application state.
    allocation = resources(
        (49152 if args.memory_mib is None else args.memory_mib) * MIB,
        (8192 if args.server_memory_mib is None else args.server_memory_mib) * MIB,
        1 if args.native_workers is None else args.native_workers,
        (
            32768
            if args.native_worker_memory_mib is None
            else args.native_worker_memory_mib
        )
        * MIB,
    )
    receiver = None
    if args.execution_profile:
        if any(
            value is not None
            for value in (
                args.memory_mib,
                args.server_memory_mib,
                args.native_workers,
                args.native_worker_memory_mib,
            )
        ):
            raise SupervisorError(
                "Reference execution supplies exact capacities; memory overrides are refused"
            )
        allocation = execution_resources(args.execution_profile)
        receiver = primary_receiver(args.worker_executable)
    port = 18240 if args.port is None else args.port
    if (
        not 1 <= port <= 65535
        or not args.interpretation
        or not args.interpretation.strip()
    ):
        raise SupervisorError(
            "Require a valid loopback port and an explicit interpretation identity"
        )
    tool_root = args.tool_root
    if tool_root is None:
        tool_root = (
            Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
            / "pse-arrow/tools/surreal"
        )
    server = install(args.version or "v3.3.0", tool_root)
    state = checked_directory(args.state, empty=True)
    with state_lock(state):
        if any(path.name != ".supervisor.lock" for path in state.iterdir()):
            raise SupervisorError(
                "State was initialized concurrently; refusing to replace it"
            )
        config: dict[str, object] = {
            "owner": OWNER,
            "profile_version": 2,
            "instance_id": str(uuid.uuid4()),
            "server": server,
            "port": port,
            "namespace": "pse",
            "database": "canonical",
            "endpoint": f"ws://127.0.0.1:{port}",
            "credentials_file": str(state / "credentials.json"),
            "schema_interpretation": args.interpretation,
            "accepting_writes": True,
            "interpretation": args.interpretation,
            "admission": "open",
            "max_message_bytes": MESSAGE_BYTES,
            "websocket_max_message_bytes": MESSAGE_BYTES,
            "resources": allocation,
            "log_max_bytes": 8 * MIB,
            "log_backups": 2,
            "service_class": "timing"
            if args.execution_profile == "timing"
            else "functional",
        }
        write_json(
            state / "credentials.json",
            {
                "username": "pse-local",
                "password": secrets.token_urlsafe(36),
                "selection_username": "pse-selection",
                "selection_password": secrets.token_urlsafe(36),
            },
        )
        if receiver is not None:
            config["primary_receiver"] = publish_generation(
                state, Path(receiver["worker_executable"])
            )
        config["service_supervisor"] = publish_generation(state)
        config["resident"] = (
            args.resident
            if args.resident is not None
            else not bool(args.execution_profile)
            or args.execution_profile in {"functional", "wide"}
        )
        write_json(state / "config.json", config)
        (state / "tmp").mkdir(mode=0o700)
    return public_status(state, config)


def reconfigure(
    state: Path, config: dict[str, object], args: argparse.Namespace
) -> None:
    """Replace only the allocation after proving the owned lifecycle is offline."""
    if any(
        value is not None
        for value in (
            args.native_workers,
            args.port,
            args.version,
            args.interpretation,
            args.tool_root,
        )
    ):
        raise SupervisorError(
            "reconfigure accepts memory options only; worker count and state identity are fixed"
        )
    if config["admission"] != "quiesced" or config["accepting_writes"]:
        raise SupervisorError(
            "Quiesce admission and stop the server before reconfigure"
        )
    prior = config["resources"]
    if not isinstance(prior, dict):
        raise SupervisorError("Invalid resource configuration")
    allocation = resources(
        integer(prior["total_memory_bytes"])
        if args.memory_mib is None
        else args.memory_mib * MIB,
        integer(prior["server_memory_bytes"])
        if args.server_memory_mib is None
        else args.server_memory_mib * MIB,
        integer(prior["native_workers"]),
        integer(prior["native_worker_memory_bytes"])
        if args.native_worker_memory_mib is None
        else args.native_worker_memory_mib * MIB,
        prior.get("execution"),
    )
    receiver = config.get("primary_receiver")
    if args.execution_profile:
        if any(
            value is not None
            for value in (
                args.memory_mib,
                args.server_memory_mib,
                args.native_worker_memory_mib,
            )
        ):
            raise SupervisorError(
                "Reference execution supplies exact capacities; memory overrides are refused"
            )
        allocation = execution_resources(args.execution_profile)
        receiver = primary_receiver(args.worker_executable)
    elif args.worker_executable is not None:
        raise SupervisorError(
            "Receiver selection requires an explicit execution profile"
        )
    observed = systemctl(
        "show",
        "--property=ActiveState",
        "--property=ControlGroup",
        unit_name(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in observed.stdout.splitlines() if "=" in line
    )
    if (
        observed.returncode
        or fields.get("ActiveState") not in {"inactive", "failed"}
        or "ControlGroup" not in fields
        or group_populated(fields["ControlGroup"])
    ):
        raise SupervisorError("Server must be verified stopped before reconfigure")
    workers_drained(state, config)
    if args.execution_profile:
        receiver = publish_generation(
            state, Path(str(object_mapping(receiver)["worker_executable"]))
        )
    updated = dict(config)
    updated["resources"] = allocation
    if args.execution_profile:
        updated["service_class"] = (
            "timing" if args.execution_profile == "timing" else "functional"
        )
    if receiver is not None:
        updated["primary_receiver"] = receiver
    write_json(state / "config.json", updated)
    config.update(updated)


def systemd_environment() -> dict[str, str]:
    environment = os.environ.copy()
    runtime = Path(f"/run/user/{os.getuid()}")
    if (runtime / "bus").is_socket():
        environment.setdefault("XDG_RUNTIME_DIR", str(runtime))
        environment.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    return environment


def systemctl(
    *arguments: str, check: bool = True, cleanup_timeout: float | None = None
) -> subprocess.CompletedProcess[str]:
    deadline = getattr(_STARTUP, "deadline", None)
    timeout = (
        cleanup_timeout
        if cleanup_timeout is not None
        else (60 if deadline is None else remaining(deadline))
    )
    result = subprocess.run(
        ["systemctl", "--user", *arguments],
        env=systemd_environment(),
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode:
        raise SupervisorError(
            "User systemd operation failed; a user manager is required for capped supervision"
        )
    return result


def unit_name(state: Path) -> str:
    state = service_directory(state)
    return f"pse-surreal-{hashlib.sha256(str(lexical_absolute(state)).encode()).hexdigest()[:16]}.service"


def active(state: Path) -> bool:
    return (
        systemctl("is-active", "--quiet", unit_name(state), check=False).returncode == 0
    )


def reset_failure_window(state: Path) -> None:
    """A collected inactive unit has no live failure window to reset."""
    unit = unit_name(state)
    result = systemctl("reset-failed", unit, check=False)
    if result.returncode:
        result = systemctl(
            "show",
            "--property=ActiveState",
            "--property=ControlGroup",
            unit,
            check=False,
        )
        observed = dict(
            line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
        )
        if (
            result.returncode
            or observed.get("ActiveState") not in {"inactive", "failed"}
            or group_populated(observed.get("ControlGroup", ""))
        ):
            raise SupervisorError(
                "Cannot reset an unresolved live service failure window"
            )


def worker_unit(state: Path, slot: int) -> str:
    """A slot has one persistent systemd name, including across launcher death."""
    return f"pse-surreal-worker-{hashlib.sha256(str(lexical_absolute(state)).encode()).hexdigest()[:16]}-{slot}.scope"


def group_populated(group: str) -> bool:
    if not group:
        return False
    if not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise SupervisorError("Cannot verify an invalid managed worker cgroup")
    path = Path("/sys/fs/cgroup") / group.lstrip("/")
    if not path.exists():
        return False
    events = (path / "cgroup.events").read_text().splitlines()
    populated = next(
        (line.split()[1] for line in events if line.startswith("populated ")), None
    )
    if populated not in {"0", "1"}:
        raise SupervisorError("Cannot verify managed worker cgroup population")
    return populated == "1"


def worker_observation(state: Path, slot: int) -> dict[str, str]:
    result = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=MemoryMax",
        worker_unit(state, slot),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if result.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot verify managed worker slot; a user systemd manager is required"
        )
    return fields


def worker_busy(observation: dict[str, str]) -> bool:
    return observation["ActiveState"] not in {"inactive", "failed"} or group_populated(
        observation["ControlGroup"]
    )


def settled_worker(state: Path, slot: int) -> dict[str, str]:
    observation = worker_observation(state, slot)
    # A scope whose launcher died may remain loaded after its processes exit.
    # Only a proven empty kernel group can be collected before slot reuse.
    if (
        observation["ActiveState"] not in {"inactive", "failed"}
        and observation["ControlGroup"]
        and not group_populated(observation["ControlGroup"])
    ):
        systemctl("stop", worker_unit(state, slot))
        observation = worker_observation(state, slot)
    return observation


def workers_drained(state: Path, config: dict[str, object]) -> None:
    """Caller acknowledgment alone cannot establish a stopped process inventory."""
    systemctl("show-environment")
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    if "execution" in allocation and worker_busy(primary_observation(state)):
        raise SupervisorError(
            "Primary receiver remains active; quiesce and drain before offline copy"
        )
    for slot in range(integer(allocation["native_workers"])):
        if worker_busy(settled_worker(state, slot)):
            raise SupervisorError(
                "Managed native worker slots remain active; quiesce and drain before offline copy"
            )


def worker_environment(
    state: Path, slot: int, allocation: dict[str, object], handoff: Path | None = None
) -> dict[str, str]:
    environment = systemd_environment()
    # Never inherit the ordinary ad hoc workload's optional/off/default cap policy.
    environment.pop("PSE_MEMORY_MAX", None)
    environment.pop("PSE_NATIVE_OPERATION", None)
    environment.pop("PSE_NATIVE_HANDOFF", None)
    if handoff is not None:
        environment["PSE_NATIVE_HANDOFF"] = str(handoff)
    environment.update(
        {
            "PSE_SURREAL_STATE": str(state),
            "PSE_NATIVE_WORKER_SLOT": str(slot),
            "PSE_NATIVE_WORKER_MEMORY_BYTES": str(
                integer(allocation["native_worker_memory_bytes"])
            ),
        }
    )
    return environment


def primary_environment(
    state: Path,
    allocation: dict[str, object],
    receiver: dict[str, str],
    handoff: Path | None = None,
) -> tuple[dict[str, str], tuple[str, ...]]:
    """Carry role-specific inputs to the actual service; the worker validates them."""
    environment = worker_environment(state, 0, allocation, handoff)
    selected = environment.get("PSE_WORKER_BINARY")
    if selected is not None:
        try:
            actual = Path(selected).resolve(strict=True)
        except OSError as error:
            raise SupervisorError(
                "Selected worker requires offline receiver readmission"
            ) from error
        if file_digest(actual) != receiver["worker_sha256"]:
            raise SupervisorError(
                "Selected worker differs from the configured receiver; drain and readmit it offline"
            )
    # A Python caller's generic receipt is its own role. A service must neither
    # inherit it nor fall back to a stale user-manager generic receipt.
    environment.pop("PSE_PRODUCER_RECEIPT", None)
    admitted = config_for(state).get("worker_producer_receipt")
    if isinstance(admitted, dict):
        frozen = Path(str(admitted["path"]))
        if file_digest(frozen) != admitted["sha256"]:
            raise SupervisorError("Immutable worker producer receipt changed")
        environment["PSE_WORKER_PRODUCER_RECEIPT"] = str(frozen)
    receipt, _ = selected_worker_receipt(environment)
    if receipt is not None:
        environment["PSE_PRODUCER_RECEIPT"] = receipt
    capabilities = ("solver", "klu", "isolation", "uno", "petsc")
    names = tuple(
        dict.fromkeys(
            (
                "PSE_PRODUCER_RECEIPT",
                "PSE_NATIVE_OPERATION",
                "PSE_NATIVE_HANDOFF",
                "PSE_NATIVE_CACHE",
                "PSE_HOST_ALLOCATION",
                "PSE_ADMISSION_DEADLINE",
                "PSE_RESOURCE_CLASS",
                "PSE_TEST_EXECUTION_PROFILE",
                "PSE_NATIVE_SETUP_PYTHON",
                "PSE_SOLVER_IMAGE",
                "PSE_NATIVE_PROVIDER_RECEIPT",
                "SYMBOLICA_LICENSE",
                "LD_LIBRARY_PATH",
                "PSE_LLVM_PREFIX",
                "LIBCLANG_PATH",
                native_operation.OFF_MARKER,
                *native_operation.OVERRIDABLE,
                *(
                    name
                    for capability in capabilities
                    for name in native_operation.CAPABILITY_PATHS[capability]
                ),
            )
        )
    )
    return environment, names


def selected_worker_receipt(
    environment: dict[str, str],
) -> tuple[str | None, str | None]:
    selected = environment.get("PSE_WORKER_PRODUCER_RECEIPT")
    if selected is None:
        return None, None
    if not selected:
        raise SupervisorError("An explicit worker producer receipt must name a file")
    try:
        path = Path(selected).resolve(strict=True)
        return str(path), file_digest(path)
    except OSError as error:
        raise SupervisorError(
            "Cannot associate the selected worker producer receipt"
        ) from error


def primary_receipt_ready(
    process: Path, marker: dict[str, object], environment: dict[str, str]
) -> bool:
    """Associate actual receiving inputs; header/artifact admission stays in the worker."""
    selected, digest = selected_worker_receipt(environment)
    assignments = (process / "environ").read_bytes().split(b"\0")
    generic = [
        item.removeprefix(b"PSE_PRODUCER_RECEIPT=")
        for item in assignments
        if item.startswith(b"PSE_PRODUCER_RECEIPT=")
    ]
    if selected is None:
        return (
            not generic
            and marker.get("producer_receipt_path") is None
            and marker.get("producer_receipt_sha256") is None
        )
    return (
        generic == [os.fsencode(selected)]
        and marker.get("producer_receipt_path") == selected
        and marker.get("producer_receipt_sha256") == digest
    )


def primary_service_environment(
    environment: dict[str, str], names: tuple[str, ...]
) -> list[str]:
    """Use caller values without putting sensitive values in the command line."""
    arguments = [f"--setenv={name}" for name in names if name in environment]
    absent = [name for name in names if name not in environment]
    if absent:
        arguments.append("--property=UnsetEnvironment=" + " ".join(absent))
    return arguments


def placement_slice() -> list[str]:
    """Workers and servers launch their own units, so they name the agent slice too."""
    root = str(Path(__file__).resolve().parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)
    from scripts import pse_env  # noqa: PLC0415 -- after root selection

    selected = pse_env.slice_name(os.environ)
    return [] if selected is None else [f"--slice={selected}"]


def execution_slice(state: Path) -> str:

    placement = placement_observation(state)
    if placement is not None and placement.get("allocation_slice"):
        return str(placement["allocation_slice"])
    allocation = host_admission.inherit(os.environ)
    if allocation is None:
        raise SupervisorError("Scientific placement requires an admitted host owner")
    return host_admission.allocation_slice(allocation)


def physical_cpus(count: int) -> list[int]:
    """Select declared physical cores independently of the launcher's affinity."""
    allocation = host_admission.inherit(os.environ)
    if allocation is None:
        raise SupervisorError("Scientific role requires an actual host allocation")
    cores = list(allocation.profile.cores)
    host_admission.cpu_set(cores)  # Validate this machine's physical topology.
    if len(cores) < count:
        raise SupervisorError("Admitted host lane cannot fit the scientific CPU width")
    return cores[:count]


def group_for_slice(name: str) -> Path:
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)
    from scripts import pse_env  # noqa: PLC0415 -- existing placement owner

    return pse_env.slice_group(name)


def effective_limits(group: Path) -> tuple[int | None, float | None]:
    memory = None
    cpu = None
    root = Path("/sys/fs/cgroup")
    while group != root and group.name:
        memory_path = group / "memory.max"
        if memory_path.is_file():
            value = memory_path.read_text().strip()
            if value != "max":
                memory = min(memory, int(value)) if memory is not None else int(value)
        cpu_path = group / "cpu.max"
        if cpu_path.is_file():
            quota, period = cpu_path.read_text().split()
            if quota != "max":
                value = int(quota) / int(period)
                cpu = min(cpu, value) if cpu is not None else value
        group = group.parent
    return memory, cpu


def host_memory() -> tuple[int, int]:
    fields = dict(
        line.split(":", 1)
        for line in Path("/proc/meminfo").read_text().splitlines()
        if ":" in line
    )
    return int(fields["MemTotal"].split()[0]) * 1024, int(
        fields["MemAvailable"].split()[0]
    ) * 1024


def placement_observation(state: Path) -> dict[str, object] | None:
    path = state / "execution-placement.json"
    return read_json(path) if path.is_file() else None


def ensure_execution_placement(state: Path, allocation: dict[str, object]) -> None:
    execution = allocation.get("execution")
    if not isinstance(execution, dict):
        return

    owner = host_admission.inherit(os.environ)
    if owner is None:
        raise SupervisorError("Scientific placement requires actual host ownership")
    width = integer(execution["cpu_threads"])
    cpus = physical_cpus(width)
    host_admission.enforce_allocation(owner, systemd_environment())
    group = group_for_slice(host_admission.allocation_slice(owner))
    total = int((group / "memory.max").read_text().strip())
    if (
        integer(allocation["native_worker_memory_bytes"])
        + integer(execution["observer_memory_bytes"])
        > total
    ):
        raise SupervisorError("Scientific roles exceed their originating allocation")
    physical, available = host_memory()
    current = group / "memory.current"
    owned = int(current.read_text()) if current.is_file() else 0
    observed_memory, observed_cpu = effective_limits(group)
    if observed_memory != total or observed_cpu is None or observed_cpu < width:
        raise SupervisorError(
            "Execution placement readback differs from the exact reference envelope"
        )
    effective = group / "cpuset.cpus.effective"
    if effective.is_file() and not set(cpus) <= cpu_list(effective.read_text().strip()):
        raise SupervisorError(
            "Execution physical CPU placement readback differs from its selection"
        )
    # User managers may have cpu/memory delegation without cpuset. Every role
    # also starts with inherited scheduler affinity, verified on its actual PID.
    # A recorded AllowedCPUs property alone never establishes enforcement.
    if shutil.which("taskset") is None:
        raise SupervisorError("Reference role placement requires taskset")
    # Caps are ceilings, not upfront physical reservations or measured demand.
    write_json(
        state / "execution-placement.json",
        {
            "allocation_slice": host_admission.allocation_slice(owner),
            "allocation": str(owner.directory / owner.nonce),
            "host_memory_bytes": physical,
            "host_available_bytes": available,
            "owned_resident_bytes": owned,
            "available_plus_owned_bytes": available + owned,
            "memory_max_bytes": observed_memory,
            "cpu_quota_cores": observed_cpu,
            "physical_cpus": cpus,
            "cpu_placement": "cpuset-and-process-affinity"
            if effective.is_file()
            else "process-affinity",
            "physically_reserved": False,
        },
    )


def cpu_list(value: str) -> set[int]:
    result: set[int] = set()
    for item in value.split(","):
        lower, separator, upper = item.partition("-")
        result.update(range(int(lower), int(upper) + 1) if separator else [int(lower)])
    return result


def selected_slice(state: Path, allocation: dict[str, object]) -> list[str]:
    if "execution" in allocation:
        return [f"--slice={execution_slice(state)}"]

    owner = host_admission.inherit(os.environ)
    if owner is None:
        raise SupervisorError(
            "Managed role requires an admitted originating host allocation"
        )
    return [f"--slice={host_admission.allocation_slice(owner)}"]


def role_command(allocation: dict[str, object], command: list[str]) -> list[str]:
    """Set affinity before the role creates children or native worker threads."""
    execution = allocation.get("execution")
    if not isinstance(execution, dict):
        return command
    executable = shutil.which("taskset")
    if executable is None:
        raise SupervisorError("Reference role placement requires taskset")
    cpus = physical_cpus(integer(execution["cpu_threads"]))
    return [executable, "--cpu-list", ",".join(map(str, cpus)), *command]


def role_affinity_ready(
    pid: int, execution: dict[str, object], cpus: list[int] | None = None
) -> bool:
    """Read the actual leader and every current thread's kernel affinity."""
    try:
        allowed = set(
            physical_cpus(integer(execution["cpu_threads"])) if cpus is None else cpus
        )
        if os.sched_getaffinity(pid) != allowed:
            return False
        return all(
            bool(actual := os.sched_getaffinity(int(thread.name))) and actual <= allowed
            for thread in (Path("/proc") / str(pid) / "task").iterdir()
        )
    except (OSError, ValueError, SupervisorError):
        return False


def worker_scope_command(
    state: Path,
    slot: int,
    allocation: dict[str, object],
    command: list[str],
    *,
    capabilities: tuple[str, ...] = WORKER_CAPABILITIES,
) -> list[str]:
    return [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={worker_unit(state, slot)}",
        *selected_slice(state, allocation),
        f"--property=MemoryMax={integer(allocation['native_worker_memory_bytes'])}",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=2048"
        if "execution" in allocation
        else "--property=TasksMax=128",
        "--",
        *role_command(
            allocation,
            [
                str(Path(__file__).resolve().parents[1] / ".venv/bin/python"),
                str(Path(__file__).resolve().with_name("native_operation.py")),
                "--capabilities",
                ",".join(capabilities),
                "--",
                *command,
            ],
        ),
    ]


def worker(
    state: Path,
    command: list[str],
    *,
    capabilities: tuple[str, ...] = WORKER_CAPABILITIES,
) -> int:
    if not command:
        raise SupervisorError(
            "worker requires --worker-command followed by an executable and arguments"
        )
    config_for(state)
    state = checked_directory(state)
    # Observe systemd outside exclusion, then reserve only the selected metadata.
    config = config_for(state)
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    systemctl("show-environment")
    ensure_execution_placement(state, allocation)
    if "execution" in allocation and worker_busy(primary_observation(state)):
        raise SupervisorError(
            "The primary process already owns the shared reference allocation"
        )
    available = [
        n
        for n in range(integer(allocation["native_workers"]))
        if not worker_busy(settled_worker(state, n))
    ]
    token = uuid.uuid4().hex

    if any(
        capability not in native_operation.CAPABILITIES for capability in capabilities
    ):
        raise SupervisorError("Unknown managed worker native capability")
    with context_admission(state):
        config = config_for(state)
        if config["admission"] != "open" or not config["accepting_writes"]:
            raise SupervisorError("Worker admission is closed")
        if config["resources"] != allocation:
            raise SupervisorError("Worker allocation changed during admission")
        primary_pending = state / "primary-admission.json"
        if (
            "execution" in allocation
            and primary_pending.exists()
            and reservation_live(read_json(primary_pending))
        ):
            raise SupervisorError(
                "Primary receiver admission already owns this allocation"
            )
        slot = next(
            (
                n
                for n in available
                if not (state / f"worker-admission-{n}.json").exists()
                or not reservation_live(read_json(state / f"worker-admission-{n}.json"))
            ),
            None,
        )
        if slot is None:
            raise SupervisorError("All configured native worker slots are occupied")
        pending = state / f"worker-admission-{slot}.json"
        write_json(
            pending,
            {
                "nonce": token,
                "pid": os.getpid(),
                "start": native_operation.start_identity(os.getpid()),
                "unit": worker_unit(state, slot),
            },
        )
    try:
        # Once reserved, an offline operation refuses this pending launch rather
        # than racing its process registration. No metadata lock crosses IPC.
        if worker_busy(settled_worker(state, slot)) or (
            "execution" in allocation and worker_busy(primary_observation(state))
        ):
            raise SupervisorError(
                "Selected worker allocation became occupied during admission"
            )
        if not ready(state, config):
            start(state, config)
        # Pin the launch window before crossing into an independent worker scope.
        # The common child owner binds this guard using actual kernel membership.
        root = str(Path(__file__).resolve().parents[1])
        if root not in sys.path:
            sys.path.insert(0, root)

        handoff = native_operation.prepare_handoff(worker_unit(state, slot))

        host_owner = host_admission.inherit(os.environ)
        if host_owner is None:
            raise SupervisorError(
                "Managed worker must belong to an admitted host allocation"
            )
        host_owner.register(worker_unit(state, slot))
        child = subprocess.Popen(
            worker_scope_command(
                state, slot, allocation, command, capabilities=capabilities
            ),
            env=worker_environment(state, slot, allocation, handoff),
        )
        deadline = time.monotonic() + 10
        while child.poll() is None:
            observation = worker_observation(state, slot)
            if observation["ActiveState"] == "active":
                if observation.get("MemoryMax") != str(
                    allocation["native_worker_memory_bytes"]
                ):
                    systemctl("stop", worker_unit(state, slot))
                    raise SupervisorError(
                        "Managed worker memory cap differs from its configured allocation"
                    )
                break
            if time.monotonic() >= deadline:
                child.terminate()
                systemctl("stop", worker_unit(state, slot), check=False)
                raise SupervisorError("Managed worker scope did not register")
            time.sleep(0.02)
    finally:
        with state_lock(state):
            if pending.exists() and read_json(pending).get("nonce") == token:
                pending.unlink()
    result = child.wait()
    return result if result >= 0 else 128 - result


def primary_unit(state: Path) -> str:
    return worker_unit(state, 0).removesuffix(".scope") + ".service"


def observer_scope_command(
    state: Path, allocation: dict[str, object], unit: str, command: list[str]
) -> list[str]:
    execution = allocation.get("execution")
    if not isinstance(execution, dict) or not command:
        raise SupervisorError(
            "Observer requires a selected execution profile and command"
        )
    return [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={unit}",
        *selected_slice(state, allocation),
        f"--property=MemoryMax={integer(execution['observer_memory_bytes'])}",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=2048",
        "--",
        *role_command(
            allocation,
            [
                sys.executable,
                str(SCRIPT.with_name("native_operation.py")),
                "--capabilities",
                "solver,klu,isolation,uno,petsc",
                "--",
                *command,
            ],
        ),
    ]


def observer_launch_busy(state: Path) -> bool:
    path = state / "observer-launch.json"
    if not path.exists():
        return False
    launch = read_json(path)
    process = Path(f"/proc/{integer(launch['pid'])}/stat")
    if (
        process.exists()
        and process.read_text().rsplit(")", 1)[1].split()[19] == launch["start"]
    ):
        return True
    result = systemctl(
        "show", "--property=ControlGroup", str(launch["unit"]), check=False
    )
    group = next(
        (
            line.removeprefix("ControlGroup=")
            for line in result.stdout.splitlines()
            if line.startswith("ControlGroup=")
        ),
        "",
    )
    return bool(group and group_populated(group))


def reference_state(selected: Path) -> Path:
    """Demand the explicit unchanged reference store before observer admission."""
    config_for(selected)
    selected = service_directory(selected)
    config = config_for(selected)
    if config["resources"] == reference_resources():
        state = selected
    else:
        state_home = Path(
            os.environ.get("XDG_STATE_HOME", str(Path.home() / ".local/state"))
        )
        state = state_home / "pse-arrow/surreal-reference-v2"
    if not (state / "config.json").exists():
        worker = os.environ.get("PSE_WORKER_BINARY")
        if not worker:
            raise SupervisorError(
                "Reference setup requires an explicitly qualified PSE_WORKER_BINARY"
            )
        args = parser().parse_args(
            [
                "setup",
                "--state",
                str(state),
                "--port",
                "18241",
                "--execution-profile",
                "plan28-reference",
                "--worker-executable",
                worker,
                "--interpretation",
                SUBSTRATE_INTERPRETATION,
            ]
        )
        setup(args)
    config = config_for(state)
    if config["resources"] != reference_resources():
        raise SupervisorError(
            "Selected reference service must retain the exact reference allocation"
        )
    # Explicit reference demand uses the ordinary owned startup boundary. It
    # preserves restored validation, active-unit readiness and parked guards.
    start(state, config)
    return state


class ObserverControlScope:
    """The exact caller lifetime and finite cap borrowed by one observer."""

    __slots__ = (
        "group",
        "inode",
        "invocation",
        "memory",
        "nonce",
        "observer",
        "temporary_memory",
        "unit",
    )

    def __init__(
        self,
        unit: str,
        group: str,
        invocation: str,
        inode: int,
        observer: str,
    ) -> None:
        self.unit = unit
        self.group = group
        self.invocation = invocation
        self.inode = inode
        self.memory = 0
        self.temporary_memory = 0
        self.nonce = uuid.uuid4().hex
        self.observer = observer


def observer_control_reservation(
    owner: host_admission.Allocation, scope: ObserverControlScope, action: str
) -> None:
    """Exclude overlapping cap borrowers without holding a lock through execution."""
    expected = {
        "owner": owner.nonce,
        "nonce": scope.nonce,
        "observer": scope.observer,
        "group": scope.group,
        "invocation": scope.invocation,
        "inode": scope.inode,
    }
    path = owner.directory / "observer-control-borrows.json"
    with host_admission.allocation_metadata(owner.directory) as ledger:
        current = ledger["owners"].get(owner.nonce)
        if current is None:
            raise SupervisorError("Observer host allocation is no longer live")
        registered = object_mapping(
            object_mapping(current.get("units")).get(scope.unit)
        )
        if registered != {
            "group": scope.group,
            "invocation": scope.invocation,
            "inode": scope.inode,
        }:
            raise SupervisorError("Observer registered control identity changed")
        if path.is_symlink():
            raise SupervisorError("Unsafe observer control borrow metadata")
        reservations = read_json(path) if path.exists() else {}
        if action == "claim":
            if scope.unit in reservations:
                raise SupervisorError(
                    "Observer control scope already has a cap borrower"
                )
            reservations[scope.unit] = expected
        elif reservations.get(scope.unit) != expected:
            raise SupervisorError("Observer control borrow identity changed")
        elif action == "release":
            del reservations[scope.unit]
        elif action != "verify":
            raise SupervisorError("Unknown observer control borrow action")
        if action != "verify":
            write_json(path, reservations)


def observer_control_observation(scope: ObserverControlScope) -> dict[str, str]:
    """Refuse to change a replaced unit or a caller outside its original group."""
    result = systemctl(
        "show",
        "--property=LoadState,ActiveState,ControlGroup,InvocationID,MemoryMax",
        scope.unit,
    )
    observed = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    caller = native_operation.process_group(os.getpid())
    if (
        observed.get("LoadState") != "loaded"
        or observed.get("ActiveState") != "active"
        or observed.get("ControlGroup") != scope.group
        or observed.get("InvocationID") != scope.invocation
        or host_admission.group_identity(scope.group) != scope.inode
        or not (caller == scope.group or caller.startswith(scope.group + "/"))
    ):
        raise SupervisorError("Observer control scope identity changed")
    return observed


def observer_allocation_drained(owner: host_admission.Allocation, unit: str) -> bool:
    """Use the existing registered kernel lifetime, never guess an empty launch."""
    with host_admission.allocation_metadata(owner.directory) as ledger:
        current = ledger["owners"].get(owner.nonce)
        if current is None:
            raise SupervisorError("Observer host allocation is no longer live")
        record = dict(current)
        units = object_mapping(current.get("units"))
        bound = object_mapping(units.get(unit, {}))
        group = bound.get("group")
        invocation = bound.get("invocation")
        if (
            not isinstance(group, str)
            or not group
            or not isinstance(invocation, str)
            or not re.fullmatch(r"[a-f0-9]{32}", invocation)
            or type(bound.get("inode")) is not int
        ):
            return False
        record["units"] = {unit: dict(bound)}
        record["released"] = True
    return host_admission.drained(record)


@contextlib.contextmanager
def observer_control_limit(
    owner: host_admission.Allocation,
    unit: str,
    profile: str | None,
    *,
    launch_attempted: Callable[[], bool] | None = None,
) -> Generator[None, None, None]:
    """Borrow the caller's control cap only for the observer's verified lifetime."""
    scope = None
    if profile is not None and profile != "reference":
        caller = native_operation.process_group(os.getpid())
        with host_admission.allocation_metadata(owner.directory) as ledger:
            units = dict(object_mapping(ledger["owners"][owner.nonce].get("units")))
        for control, value in units.items():
            registered = object_mapping(value)
            group = registered.get("group", "")
            if not isinstance(group, str):
                raise SupervisorError("Observer control group is invalid")
            if group and (caller == group or caller.startswith(group + "/")):
                invocation = registered.get("invocation")
                inode = registered.get("inode")
                if (
                    not isinstance(invocation, str)
                    or not re.fullmatch(r"[a-f0-9]{32}", invocation)
                    or not isinstance(inode, int)
                    or isinstance(inode, bool)
                ):
                    raise SupervisorError("Observer control scope is not bound")
                scope = ObserverControlScope(control, group, invocation, inode, unit)
                break
        if scope is None:
            raise SupervisorError("Observer caller has no registered control scope")
    if scope is not None:
        observer_control_reservation(owner, scope, "claim")
    captured = False
    yielded = False
    try:
        if scope is not None and profile is not None:
            memory = observer_control_observation(scope).get("MemoryMax", "")
            if not memory.isdecimal() or int(memory) <= 0:
                raise SupervisorError("Observer control cap must be finite")
            scope.memory = int(memory)
            ceiling = host_admission.settings(profile)["control_gib"] * GIB
            scope.temporary_memory = min(scope.memory, ceiling)
            captured = True
            systemctl(
                "set-property",
                "--runtime",
                scope.unit,
                f"MemoryMax={scope.temporary_memory}",
                "MemorySwapMax=0",
            )
            if observer_control_observation(scope).get("MemoryMax") != str(
                scope.temporary_memory
            ):
                raise SupervisorError("Observer control cap readback differs")
        yielded = True
        yield
    finally:
        never_launched = not yielded or (
            launch_attempted is not None and not launch_attempted()
        )
        if scope is not None and (
            never_launched or observer_allocation_drained(owner, unit)
        ):
            observer_control_reservation(owner, scope, "verify")
            if captured:
                current = observer_control_observation(scope).get("MemoryMax")
                if current not in {str(scope.temporary_memory), str(scope.memory)}:
                    raise SupervisorError(
                        "Observer control cap changed before restoration"
                    )
                if current != str(scope.memory):
                    systemctl(
                        "set-property",
                        "--runtime",
                        scope.unit,
                        f"MemoryMax={scope.memory}",
                    )
                if observer_control_observation(scope).get("MemoryMax") != str(
                    scope.memory
                ):
                    raise SupervisorError(
                        "Observer control cap restoration readback differs"
                    )
            observer_control_reservation(owner, scope, "release")


def observer(state: Path, command: list[str], profile: str | None = None) -> int:
    """Run one foreground observer in the same finite primary/server envelope."""
    config_for(state)
    state = checked_directory(state)
    unit = f"pse-native-{uuid.uuid4().hex}.scope"
    if observer_launch_busy(state):
        raise SupervisorError("The observer allocation is occupied")
    with context_admission(state):
        config = config_for(state)
        allocation = (
            config["resources"] if profile is None else execution_resources(profile)
        )
        if not isinstance(allocation, dict) or not isinstance(
            allocation.get("execution"), dict
        ):
            raise SupervisorError(
                "Observer requires the explicit shared execution profile"
            )
        if (
            not command
            or config["admission"] != "open"
            or not config["accepting_writes"]
        ):
            raise SupervisorError("Observer command requires open admission")
        # Check only recorded caller liveness under exclusion. Actual unit/drain
        # observation above never holds this metadata lock.
        pending = state / "observer-launch.json"
        if pending.exists():
            previous = read_json(pending)
            process = Path(f"/proc/{integer(previous['pid'])}/stat")
            if (
                process.exists()
                and process.read_text().rsplit(")", 1)[1].split()[19]
                == previous["start"]
            ):
                raise SupervisorError("The observer allocation is occupied")
        registration = state / "primary-observer.json"
        if registration.exists():
            prior = read_json(registration)
            process = Path(f"/proc/{integer(prior['pid'])}/stat")
            if (
                process.exists()
                and process.read_text().rsplit(")", 1)[1].split()[19] == prior["start"]
            ):
                raise SupervisorError("The reference observer allocation is occupied")
        start = (
            Path(f"/proc/{os.getpid()}/stat").read_text().rsplit(")", 1)[1].split()[19]
        )
        write_json(
            state / "observer-launch.json",
            {"pid": os.getpid(), "start": start, "unit": unit},
        )
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)

    ensure_execution_placement(state, allocation)

    owner = host_admission.inherit(os.environ)
    if owner is None:
        raise SupervisorError("Observer must belong to an admitted host allocation")
    attempted = False
    try:
        with observer_control_limit(
            owner, unit, profile, launch_attempted=lambda: attempted
        ):
            owner.register(unit)
            handoff = native_operation.prepare_handoff(unit, foreground=True)
            environment = systemd_environment()
            for key in (
                "PSE_NATIVE_OPERATION",
                "PSE_NATIVE_HANDOFF",
                "PSE_NATIVE_WORKER_SLOT",
                "PSE_NATIVE_WORKER_MEMORY_BYTES",
                "PSE_MEMORY_MAX",
            ):
                environment.pop(key, None)
            if handoff is not None:
                environment["PSE_NATIVE_HANDOFF"] = str(handoff)
            environment["PSE_SURREAL_STATE"] = str(state)
            if profile is not None:
                environment["PSE_TEST_EXECUTION_PROFILE"] = profile
            selected_command = observer_scope_command(state, allocation, unit, command)
            attempted = True
            # The operation creator owns cancellation of this handoff. Keep its
            # native supervisor outside a validation gate's process-group kill,
            # so it can drain runners that establish their own sessions.
            return subprocess.call(
                selected_command, env=environment, start_new_session=handoff is not None
            )
    finally:
        # Keep launch exclusion through the control-scope restoration attempt.
        # Surviving or unbound scopes retain launch metadata and host charge.
        if not attempted or observer_allocation_drained(owner, unit):
            with state_lock(state):
                launch = read_json(state / "observer-launch.json")
                if launch.get("unit") == unit:
                    (state / "observer-launch.json").unlink()


def primary_observation(state: Path) -> dict[str, str]:
    result = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=MemoryMax",
        "--property=MainPID",
        primary_unit(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if result.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot verify the primary receiver's systemd association"
        )
    return fields


def recorded_primary(config: dict[str, object]) -> dict[str, str]:
    """Read the admitted association without opening its replacement artifacts."""
    receiver = config.get("primary_receiver")
    if not isinstance(receiver, dict) or not all(
        isinstance(value, str) for value in receiver.values()
    ):
        raise SupervisorError(
            "Selected execution requires a configured primary receiver"
        )
    expected = {
        "supervisor_executable",
        "supervisor_script",
        "worker_executable",
        "supervisor_sha256",
        "worker_sha256",
    }
    if set(receiver) != expected:
        raise SupervisorError("Primary receiver association has unsupported fields")
    result = {key: str(value) for key, value in receiver.items()}
    if any(
        not Path(result[key]).is_absolute()
        for key in ("supervisor_executable", "supervisor_script", "worker_executable")
    ):
        raise SupervisorError("Primary receiver paths must be explicit and absolute")
    verify_generation(Path(result["supervisor_script"]).parents[1])
    return result


def checked_primary(config: dict[str, object]) -> dict[str, str]:
    receiver = recorded_primary(config)
    if (
        file_digest(Path(receiver["supervisor_script"]))
        != receiver["supervisor_sha256"]
        or file_digest(Path(receiver["worker_executable"])) != receiver["worker_sha256"]
    ):
        raise SupervisorError(
            "Primary receiver bytes changed; offline profile readmission is required"
        )
    return receiver


def primary_ready(
    state: Path,
    config: dict[str, object],
    observation: dict[str, str],
    database: str | None = None,
    qualification: Path | None = None,
) -> bool:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        return False
    return (
        _primary_process(
            state,
            config,
            observation,
            checked_primary(config),
            database,
            qualification,
            receiving=True,
        )
        is not None
    )


def primary_drain_pid(
    state: Path, config: dict[str, object], observation: dict[str, str]
) -> int | None:
    """Identify the admitted live receiver even after its disk files are replaced."""
    return _primary_process(state, config, observation, recorded_primary(config))


def _primary_process(
    state: Path,
    config: dict[str, object],
    observation: dict[str, str],
    receiver: dict[str, str],
    database: str | None = None,
    qualification: Path | None = None,
    *,
    receiving: bool = False,
) -> int | None:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        return None
    execution = allocation["execution"]
    try:
        launch = read_json(state / "primary-launch.json")
        marker = read_json(state / "primary-receiver.json")
        pid = integer(marker["pid"])
        if qualification is not None and marker.get(
            "qualification_native_entry"
        ) != str(qualification):
            return None
        selected = marker.get("canonical_database")
        if (
            not isinstance(selected, str)
            or not selected
            or (database is not None and selected != database)
        ):
            return None
        if (
            observation["ActiveState"] != "active"
            or not group_populated(observation["ControlGroup"])
            or marker.get("ready") is not True
            or marker.get("nonce") != launch["nonce"]
        ):
            return None
        if observation.get("MemoryMax") != str(
            allocation["native_worker_memory_bytes"]
        ):
            return None
        if any(
            marker.get(key) != execution[key]
            for key in (
                "pool_memory_bytes",
                "worker_bytes",
                "cpu_threads",
                "case_lanes",
                "math_jobs",
            )
        ):
            return None
        process = Path(f"/proc/{pid}")
        relative = next(
            line.removeprefix("0::")
            for line in (process / "cgroup").read_text().splitlines()
            if line.startswith("0::")
        )
        if relative != observation["ControlGroup"] and not relative.startswith(
            observation["ControlGroup"] + "/"
        ):
            return None
        executable = process / "exe"
        # Linux keeps the admitted executable open after atomic replacement. Its
        # proc link gains this suffix; resolving it against the replacement path
        # would confuse the running admission with the next one.
        actual_path = str(executable.readlink()).removesuffix(" (deleted)")
        if file_digest(executable) != receiver["worker_sha256"] or Path(
            actual_path
        ) != Path(receiver["worker_executable"]):
            return None
        assignments = (process / "environ").read_bytes().split(b"\0")
        if [
            item.removeprefix(b"PSE_PRIMARY_NONCE=")
            for item in assignments
            if item.startswith(b"PSE_PRIMARY_NONCE=")
        ] != [os.fsencode(str(launch["nonce"]))]:
            return None
        if receiving and not primary_receipt_ready(
            process, marker, systemd_environment()
        ):
            return None
        placement = placement_observation(state)
        if not isinstance(placement, dict) or not isinstance(
            placement.get("physical_cpus"), list
        ):
            return None
        if not role_affinity_ready(pid, execution, placement["physical_cpus"]):
            return None
        memory, cpu = effective_limits(Path("/sys/fs/cgroup") / relative.lstrip("/"))
        if (
            memory != integer(allocation["native_worker_memory_bytes"])
            or cpu is None
            or cpu < integer(execution["cpu_threads"])
        ):
            return None
    except (OSError, KeyError, StopIteration, SupervisorError):
        return None
    else:
        return pid


def verify_observer(state: Path, config: dict[str, object], pid: int) -> None:
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        raise SupervisorError("Observer requires the selected shared execution profile")
    execution = allocation["execution"]
    if pid <= 0:
        raise SupervisorError("Observer PID must be positive")
    cgroups = Path(f"/proc/{pid}/cgroup").read_text()
    relative = next(
        (
            line.removeprefix("0::")
            for line in cgroups.splitlines()
            if line.startswith("0::")
        ),
        "",
    )
    group = Path("/sys/fs/cgroup") / relative.lstrip("/")
    profile = group_for_slice(execution_slice(state))
    placement = placement_observation(state)
    if not isinstance(placement, dict):
        raise SupervisorError("Observer lacks actual execution placement")
    cpus = placement.get("physical_cpus")
    if not isinstance(cpus, list) or not all(isinstance(value, int) for value in cpus):
        raise SupervisorError("Observer lacks declared physical CPU placement")
    memory, cpu = effective_limits(group)
    if (
        not group.is_relative_to(profile)
        or memory is None
        or memory > integer(execution["observer_memory_bytes"])
        or cpu is None
        or cpu < integer(execution["cpu_threads"])
        or not role_affinity_ready(pid, execution, cpus)
    ):
        raise SupervisorError(
            f"Observer placement required: PSE_SLICE={execution_slice(state)} PSE_MEMORY_MAX=4G scripts/pse-env -- <observer command>"
        )
    # The envelope includes one observer process, not arbitrarily many independent
    # four-GiB ceilings. Kernel membership and PID start time establish live reuse.
    start = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19]
    registration = state / "primary-observer.json"
    if registration.exists():
        prior = read_json(registration)
        previous = integer(prior["pid"])
        previous_stat = Path(f"/proc/{previous}/stat")
        if previous != pid and previous_stat.exists():
            observed = previous_stat.read_text().rsplit(")", 1)[1].split()[19]
            if observed == prior["start"]:
                raise SupervisorError(
                    "The reference envelope's observer allocation is occupied"
                )
    write_json(registration, {"pid": pid, "start": start, "group": relative})


def _checked_current_observer(state: Path) -> dict[str, object]:
    """Validate the live caller's own registration while holding state_lock."""
    registration = read_json(state / "primary-observer.json")
    pid = os.getpid()
    if integer(registration["pid"]) != pid:
        raise SupervisorError("Current caller does not own the observer registration")
    process = Path(f"/proc/{pid}")
    start = (process / "stat").read_text().rsplit(")", 1)[1].split()[19]
    group = next(
        (
            line.removeprefix("0::")
            for line in (process / "cgroup").read_text().splitlines()
            if line.startswith("0::")
        ),
        "",
    )
    if (
        not group
        or registration.get("start") != start
        or registration.get("group") != group
    ):
        raise SupervisorError("Current observer process identity changed")
    return registration


def release_current_observer(state: Path) -> None:
    """Relinquish only this caller's registration after primary capacity drains.

    The observer's live PID is never overridden by a second caller. A sequential
    owner may explicitly hand off within its existing aggregate group after the
    associated primary is stopped and the kernel confirms its allocation empty.
    """
    config_for(state)
    state = checked_directory(state)
    with state_lock(state):
        registration = _checked_current_observer(state)
        # Keep the control request finite without changing general manager policy.
        prior = getattr(_STARTUP, "deadline", None)
        deadline = time.monotonic() + 10
        _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
        try:
            observed = primary_observation(state)
        finally:
            _STARTUP.deadline = prior
        group = observed["ControlGroup"]
        if observed["ActiveState"] not in {"inactive", "failed"} or (
            group and group_populated(group)
        ):
            raise SupervisorError(
                "Observer handoff requires a stopped, drained primary"
            )
        if _checked_current_observer(state) != registration:
            raise SupervisorError("Observer registration changed before handoff")
        (state / "primary-observer.json").unlink()


def qualification_directory(state: Path, selected: Path) -> Path:
    """Accept an explicit private diagnostic control, never ambient execution policy."""
    selected = lexical_absolute(selected)
    state = lexical_absolute(state)
    if selected == state or not selected.is_relative_to(state):
        raise SupervisorError("Qualification control must be beneath the owned state")
    if any(path.is_symlink() for path in (selected, *selected.parents)):
        raise SupervisorError("Qualification control must not traverse symlinks")
    if (
        not selected.is_dir()
        or selected.stat().st_uid != os.getuid()
        or selected.stat().st_mode & 0o077
    ):
        raise SupervisorError(
            "Qualification control must be an existing private owned directory"
        )
    return selected


def ensure_primary(
    state: Path,
    observer_pid: int | None = None,
    database: str | None = None,
    qualification: Path | None = None,
) -> dict[str, object]:
    state = receiver_context(state, database)
    # Existing nested control calls consume this owner's clock; they do not renew it.
    deadline = time.monotonic() + 30
    prior = getattr(_STARTUP, "deadline", None)
    reservation = state / "primary-admission.json"
    token = uuid.uuid4().hex

    with context_admission(state):
        if any(
            reservation_live(read_json(pending))
            for pending in state.glob("worker-admission-*.json")
        ):
            raise SupervisorError(
                "A pending native worker launch owns this receiver allocation"
            )
        if reservation.exists():
            owner = read_json(reservation)
            try:
                alive = (
                    native_operation.start_identity(integer(owner["pid"]))
                    == owner["start"]
                )
            except FileNotFoundError:
                alive = False
            if alive:
                raise SupervisorError("Another live owner is admitting this receiver")
        write_json(
            reservation,
            {
                "nonce": token,
                "pid": os.getpid(),
                "start": native_operation.start_identity(os.getpid()),
            },
        )
    _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
    try:
        return _ensure_primary(
            state, observer_pid, database, _STARTUP.deadline, qualification
        )
    finally:
        with state_lock(state):
            if reservation.exists() and read_json(reservation).get("nonce") == token:
                reservation.unlink()
        _STARTUP.deadline = prior


def _ensure_primary(
    state: Path,
    observer_pid: int | None,
    database: str | None,
    deadline: float,
    qualification: Path | None = None,
) -> dict[str, object]:
    config_for(state)
    state = checked_directory(state)
    if qualification is not None:
        qualification = qualification_directory(state, qualification)
    config = config_for(state)
    if config["admission"] != "open" or not config["accepting_writes"]:
        raise SupervisorError("Primary admission is closed")
    allocation = config["resources"]
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get("execution"), dict
    ):
        raise SupervisorError(
            "Primary serving requires an explicit reference execution profile"
        )
    receiver = checked_primary(config)
    primary_environment(state, allocation, receiver)
    selected_database = database if database is not None else str(config["database"])
    if not selected_database or len(selected_database) > 1024:
        raise SupervisorError(
            "Primary receiver requires a bounded canonical database identity"
        )
    ensure_execution_placement(state, allocation)
    if observer_pid is not None:
        verify_observer(state, config, observer_pid)
    observed = primary_observation(state)
    ready_arguments = () if qualification is None else (qualification,)
    if primary_ready(state, config, observed, selected_database, *ready_arguments):
        if not ready(state, config):
            start(state, config, deadline=deadline)
        return {
            "ready": True,
            "unit": primary_unit(state),
            "execution": allocation["execution"],
            "canonical_database": selected_database,
            "placement": placement_observation(state),
        }
    if worker_busy(observed) or worker_busy(settled_worker(state, 0)):
        raise SupervisorError(
            "An active unmatched receiver or canonical database owns the primary allocation; quiesce and drain it before reuse"
        )
    if not ready(state, config):
        start(state, config, deadline=deadline)
    root = str(SCRIPT.parents[1])
    if root not in sys.path:
        sys.path.insert(0, root)

    host_owner = host_admission.inherit(os.environ)
    if host_owner is None:
        raise SupervisorError("Primary must belong to an admitted host allocation")
    host_owner.register(primary_unit(state))
    handoff = native_operation.prepare_handoff(primary_unit(state))
    nonce = uuid.uuid4().hex
    write_json(state / "primary-launch.json", {"nonce": nonce})
    (state / "primary-receiver.json").unlink(missing_ok=True)
    environment, names = primary_environment(state, allocation, receiver, handoff)
    environment["PSE_PRIMARY_NONCE"] = nonce
    command = [
        "systemd-run",
        "--user",
        "--quiet",
        "--collect",
        "--no-ask-password",
        "--expand-environment=no",
        f"--unit={primary_unit(state)}",
        *selected_slice(state, allocation),
        "--service-type=exec",
        "--property=KillMode=control-group",
        "--property=Restart=no",
        "--property=TimeoutStopSec=45",
        "--property=TasksMax=2048",
        "--property=MemorySwapMax=0",
        f"--property=MemoryMax={allocation['native_worker_memory_bytes']}",
    ]
    for key in (
        "PSE_SURREAL_STATE",
        "PSE_NATIVE_WORKER_SLOT",
        "PSE_NATIVE_WORKER_MEMORY_BYTES",
        "PSE_NATIVE_HANDOFF",
        "PSE_PRIMARY_NONCE",
    ):
        command.append(f"--setenv={key}={environment[key]}")
    command.extend(primary_service_environment(environment, names))
    command.extend(
        [
            "--",
            *role_command(
                allocation,
                [
                    receiver["supervisor_executable"],
                    str(SCRIPT.with_name("native_operation.py")),
                    "--capabilities",
                    "solver,klu,isolation,uno,petsc",
                    "--",
                    receiver["worker_executable"],
                    "--maximum-in-flight",
                    str(allocation["execution"]["case_lanes"]),
                    "--canonical-database",
                    selected_database,
                ],
            ),
        ]
    )
    if qualification is not None:
        command.extend(["--qualification-native-entry", str(qualification)])
    try:
        result = subprocess.run(
            command,
            env=environment,
            capture_output=True,
            timeout=remaining(deadline),
            check=False,
        )
    except subprocess.TimeoutExpired:
        systemctl("stop", primary_unit(state), check=False, cleanup_timeout=10)
        raise SupervisorError(
            "Primary launch exceeded its original admission clock"
        ) from None
    if result.returncode:
        raise SupervisorError("Cannot launch the capped primary receiver")
    while time.monotonic() < deadline:
        observed = primary_observation(state)
        if primary_ready(state, config, observed, selected_database, *ready_arguments):
            return {
                "ready": True,
                "unit": primary_unit(state),
                "execution": allocation["execution"],
                "canonical_database": selected_database,
                "placement": placement_observation(state),
            }
        if observed["ActiveState"] == "failed":
            break
        time.sleep(0.05)
    systemctl("stop", primary_unit(state), check=False, cleanup_timeout=10)
    raise SupervisorError(
        "Primary receiver did not establish its actual runtime readiness"
    )


def drain_context(
    state: Path, database: str, *, deadline: float | None = None
) -> list[dict[str, str]]:
    """Cooperatively drain only the receiver associated with this exact context."""
    if not (state / ".contexts" / f"{database}.json").exists():
        base = config_for(state)
        if (
            "execution" not in object_mapping(base["resources"])
            and base.get("primary_receiver") is None
        ):
            # A registered codec-only database never admitted a native receiver.
            return []
    selected = receiver_context(state, database)
    config = config_for(selected)
    if config["database"] != database:
        raise SupervisorError(
            "Drain requires an explicit receiver database association"
        )
    deadline = time.monotonic() + 45 if deadline is None else deadline
    previous = getattr(_STARTUP, "deadline", None)
    _STARTUP.deadline = deadline if previous is None else min(previous, deadline)
    try:
        observed = primary_observation(selected)
        group = observed.get("ControlGroup", "")
        if observed["ActiveState"] in {"inactive", "failed"} and (
            not group or not group_populated(group)
        ):
            return []
        pid = primary_drain_pid(selected, config, observed)
        if pid is None:
            raise SupervisorError(
                "Unmatched live receiver remains retained; drain refused"
            )
        owner = native_operation.unit_observation(primary_unit(selected))
        association = {
            "unit": primary_unit(selected),
            "group": owner["ControlGroup"],
            "invocation": owner["InvocationID"],
        }
        descriptor = os.pidfd_open(pid)
        try:
            if (
                primary_drain_pid(selected, config, primary_observation(selected))
                != pid
            ):
                raise SupervisorError(
                    "Receiver identity changed before cooperative drain"
                )
            signal.pidfd_send_signal(descriptor, signal.SIGINT)
            while not native_operation.drained({"scope": association}):
                remaining(_STARTUP.deadline)
                time.sleep(0.02)
        finally:
            os.close(descriptor)
        return [association]
    finally:
        _STARTUP.deadline = previous


def public_status(state: Path, config: dict[str, object]) -> dict[str, object]:
    port = integer(config["port"])
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    return {
        "state": str(state),
        "active": active(state),
        "listener_owned": owns_listener(state, config),
        "authenticated_websocket_ready": protocol_ready(state, config),
        "service_generation": config["instance_id"],
        "supervisor_generation": config.get("service_supervisor"),
        "parked": bool(config.get("parked")),
        "unit": unit_name(state),
        "endpoint": f"127.0.0.1:{port}",
        "websocket_endpoint": config["endpoint"],
        "credentials_file": str(state / "credentials.json"),
        "config_file": str(state / "config.json"),
        "server": config["server"],
        "admission": config["admission"],
        "interpretation": config["interpretation"],
        "resources": config["resources"],
        "execution_placement": placement_observation(state),
        "accepting_writes": config["accepting_writes"],
        "websocket_max_message_bytes": MESSAGE_BYTES,
        "worker_units": [
            worker_unit(state, slot)
            for slot in range(integer(allocation["native_workers"]))
        ],
    }


def listener_ready(state: Path, config: dict[str, object]) -> bool:
    owner = service_directory(state)
    if owner != state:
        return listener_ready(owner, config_for(owner))
    if not owns_listener(state, config):
        return False
    try:
        with urllib.request.urlopen(
            f"http://127.0.0.1:{config['port']}/health", timeout=1
        ) as response:
            return response.status == 200 and active(state)
    except (OSError, urllib.error.URLError):
        return False


def protocol_ready(state: Path, config: dict[str, object]) -> bool:
    owner = service_directory(state)
    if owner != state:
        return protocol_ready(owner, config_for(owner))
    path = state / "protocol-readiness.json"
    if not path.is_file():
        return False
    proof = read_json(path)
    observed = systemctl(
        "show", "--property=InvocationID", "--value", unit_name(state), check=False
    )
    return (
        proof.get("schema") == "native-ws-readiness-v1"
        and proof.get("instance_id") == config["instance_id"]
        and proof.get("invocation") == observed.stdout.strip()
        and proof.get("binary_sha256")
        == object_mapping(config["server"]).get("binary_sha256")
        and proof.get("credentials_sha256") == file_digest(state / "credentials.json")
    )


def establish_protocol_readiness(
    state: Path, config: dict[str, object], deadline: float
) -> None:
    """Authenticate native WS under the startup clock without installing schema."""
    state = checked_directory(state)
    credentials = read_json(state / "credentials.json")
    if (
        credentials.get("selection_username") != "pse-selection"
        or not isinstance(credentials.get("selection_password"), str)
        or not credentials["selection_password"]
    ):
        raise SupervisorError(
            "Selection credentials are missing; stop, quiesce and explicitly readmit this service"
        )
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("SURREAL_")
    }
    environment.update(
        SURREAL_USER=str(credentials["username"]),
        SURREAL_PASS=str(credentials["password"]),
    )
    query = f"DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD {json.dumps(credentials['selection_password'])} ROLES VIEWER; INFO FOR ROOT;\n"
    result = subprocess.run(
        [
            str(object_mapping(config["server"])["binary"]),
            "sql",
            "--endpoint",
            str(config["endpoint"]),
            "--json",
            "--hide-welcome",
        ],
        input=query,
        env=environment,
        cwd=state,
        capture_output=True,
        text=True,
        timeout=remaining(deadline),
        check=False,
    )
    try:
        # The pinned CLI prints a REPL prompt even for non-interactive pipes.
        lines = [line.removeprefix("> ").strip() for line in result.stdout.splitlines()]
        returned = json.loads("\n".join(line for line in lines if line and line != ">"))
        valid = (
            isinstance(returned, list)
            and len(returned) == 2
            and returned[0] is None
            and isinstance(returned[1], dict)
            and isinstance(returned[1].get("namespaces"), dict)
        )
    except ValueError:
        valid = False
    if result.returncode or not valid:
        raise SupervisorError(
            "Authenticated native WebSocket readiness failed; admission remains closed"
        )
    invocation = systemctl(
        "show", "--property=InvocationID", "--value", unit_name(state)
    ).stdout.strip()
    write_json(
        state / "protocol-readiness.json",
        {
            "schema": "native-ws-readiness-v1",
            "instance_id": config["instance_id"],
            "invocation": invocation,
            "binary_sha256": object_mapping(config["server"]).get("binary_sha256"),
            "credentials_sha256": file_digest(state / "credentials.json"),
        },
    )


def ready(state: Path, config: dict[str, object]) -> bool:
    return listener_ready(state, config) and protocol_ready(state, config)


def process_owns_listening_socket(proc: Path, inodes: set[str]) -> bool:
    sockets = {f"socket:[{inode}]" for inode in inodes}
    for descriptor in (proc / "fd").iterdir():
        try:
            target = str(descriptor.readlink())
        except FileNotFoundError:
            # Other connections may close after enumeration. Only a live
            # descriptor for the selected listening socket establishes ownership.
            continue
        if target in sockets:
            return True
    return False


def owns_listener(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> bool:
    """Linux listener ownership, independent of another server's /health response."""
    owner_state = service_directory(state, deadline=deadline)
    if owner_state != state:
        return owns_listener(
            owner_state, config_for(owner_state, deadline=deadline), deadline=deadline
        )
    try:
        if deadline is not None:
            remaining(deadline)
        process = read_json(state / "server-process.json", deadline=deadline)
        if process.get("instance_id") != config["instance_id"]:
            return False
        pid = integer(process["pid"])
        proc = Path("/proc") / str(pid)
        if process.get("start") != native_operation.start_identity(pid):
            return False
        membership = (proc / "cgroup").read_text()
        owner, launch = recorded_storage_owner(state, config, deadline=deadline)
        placement = storage_placement(config, owner)
        observed = storage_unit_observation(state, deadline=deadline)
        binding = object_mapping(launch["binding"])
        relative = next(
            line.removeprefix("0::")
            for line in membership.splitlines()
            if line.startswith("0::")
        )
        group = Path("/sys/fs/cgroup") / relative.lstrip("/")
        memory, cpu = effective_limits(group)
        quota, period = (group / "cpu.max").read_text().split()
        if (
            observed.get("ActiveState") != "active"
            or observed.get("InvocationID") != binding["invocation"]
            or observed.get("ControlGroup") != binding["group"]
            or relative != binding["group"]
            or host_admission.group_identity(relative) != binding["inode"]
            or process.get("allocation") != launch["allocation"]
            or memory != placement["memory_bytes"]
            or cpu != placement["cpu_threads"]
            or (group / "memory.max").read_text().strip()
            != str(placement["memory_bytes"])
            or quota == "max"
            or int(quota) / int(period) != placement["cpu_threads"]
            or not group.is_relative_to(group_for_slice(str(placement["slice"])))
            or not role_affinity_ready(
                pid,
                {"cpu_threads": placement["cpu_threads"]},
                list(owner.profile.cores),
            )
        ):
            return False
        address = f"0100007F:{integer(config['port']):04X}"
        inodes = {
            row.split()[9]
            for row in Path("/proc/net/tcp").read_text().splitlines()[1:]
            if row.split()[1] == address and row.split()[3] == "0A"
        }
        matched = process_owns_listening_socket(proc, inodes)
        if deadline is not None:
            remaining(deadline)
    except (
        OSError,
        KeyError,
        ValueError,
        StopIteration,
        SupervisorError,
        host_admission.AdmissionError,
    ):
        return False
    return matched


def remaining(deadline: float) -> float:
    value = deadline - time.monotonic()
    if value <= 0:
        raise SupervisorError("Managed startup exceeded its original admission clock")
    return value


def storage_placement(
    config: dict[str, object], owner: host_admission.Allocation
) -> dict[str, object]:
    """Derive storage caps from its host owner, independently of science roles."""
    memory = integer(object_mapping(config["resources"])["server_memory_bytes"])
    service_class = config.get("service_class", "functional")
    if service_class not in {"functional", "timing"}:
        raise SupervisorError("Unknown storage service class")
    if not owner.profile.exclusive:
        expected = host_admission.select("store-" + str(service_class), str(memory))
        if owner.profile != expected:
            raise SupervisorError(
                "Storage owner differs from the selected service lane"
            )
    elif memory > host_admission.settings(owner.profile.name)["store_gib"] * GIB:
        raise SupervisorError("Storage exceeds its borrowed exclusive partition")
    host_admission.cpu_set(owner.profile.cores)
    return {
        "memory_bytes": memory,
        "cpu_threads": len(owner.profile.cores),
        "physical_cpus": list(owner.profile.cores),
        "slice": host_admission.allocation_slice(owner)
        if owner.profile.exclusive
        else "pse.slice",
    }


def storage_launch(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> dict[str, object]:
    """Require the exact immutable service selection associated with a launch."""
    launch = read_json(state / "service-launch.json", deadline=deadline)
    launch_deadline = launch.get("deadline")
    if not isinstance(launch_deadline, (int, float)) or not math.isfinite(
        launch_deadline
    ):
        raise SupervisorError("Storage launch lacks a finite admission clock")
    if (
        launch.get("generation") != config["instance_id"]
        or launch.get("supervisor_generation") != config.get("service_supervisor")
        or launch.get("server_generation") != config.get("server")
        or launch.get("service_class") != config.get("service_class", "functional")
        or launch.get("server_memory_bytes")
        != object_mapping(config["resources"])["server_memory_bytes"]
    ):
        raise SupervisorError(
            "Storage launch differs from the admitted service generation"
        )
    return launch


def recorded_storage_owner(
    state: Path, config: dict[str, object], *, deadline: float | None = None
) -> tuple[host_admission.Allocation, dict[str, object]]:
    """Read actual charged storage ownership without adopting the caller's lane."""
    launch = storage_launch(state, config, deadline=deadline)
    path = Path(str(launch["allocation"]))
    if not re.fullmatch(r"[a-f0-9]{32}", path.name):
        raise SupervisorError("Invalid storage allocation marker")
    snapshot = (
        contextlib.nullcontext(
            host_admission.readonly_snapshot(path.parent, deadline=deadline)
        )
        if deadline is not None
        else host_admission.allocation_metadata(path.parent)
    )
    with snapshot as ledger:
        record = ledger["owners"].get(path.name)
        if (
            record is None
            or record["boot"] != host_admission.boot()
            or record.get("released")
        ):
            raise SupervisorError("Storage allocation is absent or stale")
        bound = dict(record["units"].get(unit_name(state), {}))
        if (
            (
                record.get("service") != str(state)
                and str(state) not in record.get("borrowed_services", [])
            )
            or not isinstance(bound.get("group"), str)
            or not isinstance(bound.get("invocation"), str)
            or not re.fullmatch(r"[a-f0-9]{32}", str(bound.get("invocation", "")))
            or type(bound.get("inode")) is not int
        ):
            raise SupervisorError("Storage allocation does not own this service")
        profile = host_admission.Profile(
            record["class"],
            record["memory"],
            record["lane"],
            record["slots"],
            tuple(record["cores"]),
            record["exclusive"],
        )
        deadline = record["deadline"]
    if bound != launch.get("binding"):
        raise SupervisorError("Storage launch does not match its bound kernel lifetime")
    return host_admission.Allocation(path.parent, path.name, profile, deadline), launch


def storage_unit_observation(
    state: Path, *, deadline: float | None = None
) -> dict[str, str]:
    """Observe storage identity through the supervisor's remaining original clock."""
    if deadline is not None:
        return host_admission.control_unit_observation(unit_name(state), deadline)
    result = systemctl(
        "show",
        "--property=InvocationID",
        "--property=ControlGroup",
        "--property=ActiveState",
        unit_name(state),
    )
    observed = dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )
    if not re.fullmatch(r"[a-f0-9]{32}", observed.get("InvocationID", "")):
        raise SupervisorError("Storage unit lacks an actual invocation identity")
    return observed


def resume_storage_allocation(
    state: Path, config: dict[str, object]
) -> host_admission.Allocation:
    """Reuse charged capacity only after the exact owned predecessor has drained."""
    owner, launch = recorded_storage_owner(state, config)
    observed = storage_unit_observation(state)
    group = observed.get("ControlGroup", "")
    binding = object_mapping(launch["binding"])
    if (
        observed.get("ActiveState") not in {"active", "activating"}
        or not group.startswith("/")
        or group == "/"
        or ".." in Path(group).parts
        or native_operation.process_group(os.getpid()) != group
        or observed["InvocationID"] == binding.get("invocation")
        or group != binding.get("group")
    ):
        raise SupervisorError("Cannot adopt an uncertain storage restart lifetime")
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    processes = {
        int(pid)
        for path in directory.rglob("cgroup.procs")
        for pid in path.read_text().split()
    }
    if processes != {os.getpid()}:
        raise SupervisorError("Storage predecessor or descendants remain undrained")
    return owner


def materialize_service(
    state: Path,
    config: dict[str, object],
    owner: host_admission.Allocation | None = None,
) -> None:
    """Persistent owned unit; ExecStart only starts storage, never science."""
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict):
        raise SupervisorError(
            "Profile has no immutable service supervisor; explicitly readmit it"
        )
    verify_generation(Path(str(supervisor["supervisor_script"])).parents[1])
    directory = (
        Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
        / "systemd/user"
    )
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / unit_name(state)
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Missing service allocation")

    # Escaping is systemd ExecStart syntax, never shell syntax.
    def argument(value: object) -> str:
        text = str(value)
        if any(character in text for character in "\n\r\0"):
            raise SupervisorError("Invalid systemd unit argument")
        return (
            '"'
            + text.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
            + '"'
        )

    executable = " ".join(
        argument(value)
        for value in (
            supervisor["supervisor_executable"],
            supervisor["supervisor_script"],
            "_serve",
            "--state",
            state,
        )
    )
    config["restart_qualified"] = recovery_qualified(state, config)
    maintenance = (state / ".maintenance-listener.json").exists()
    restart = "on-failure" if config["restart_qualified"] and not maintenance else "no"

    if owner is None:
        admitted = host_admission.inherit(os.environ)
        if admitted is not None and admitted.profile.exclusive:
            owner = admitted
        else:
            profile = host_admission.select(
                "store-" + str(config.get("service_class", "functional")),
                str(allocation["server_memory_bytes"]),
            )
            owner = host_admission.Allocation(state, "", profile, 0)
    placement = storage_placement(config, owner)
    text = f"[Unit]\nDescription=pse-arrow owned storage {config['instance_id']}\nStartLimitIntervalSec=300\nStartLimitBurst=3\n\n[Service]\nType=exec\nExecStart={executable}\nKillMode=control-group\nTimeoutStopSec=45\nRestart={restart}\nRestartSec=5\nSlice={placement['slice']}\nMemoryMax={placement['memory_bytes']}\nCPUQuota={integer(placement['cpu_threads']) * 100}%\nMemorySwapMax=0\nTasksMax=128\nStandardOutput=null\nStandardError=null\n\n[Install]\nWantedBy=default.target\n"
    if path.exists() and not config.get("unit_materialized"):
        raise SupervisorError(
            "Existing user unit is not this generation's materialization"
        )
    temporary = path.with_suffix(".pending")
    temporary.write_text(text)
    temporary.chmod(0o600)
    temporary.replace(path)
    config["unit_materialized"] = True
    write_json(state / "config.json", config)
    systemctl("daemon-reload")
    if config.get("resident") and not maintenance:
        systemctl("enable", unit_name(state))


def service_allocation(
    state: Path, config: dict[str, object], deadline: float
) -> host_admission.Allocation:

    inherited = host_admission.inherit(os.environ)
    if inherited is not None and inherited.profile.exclusive:
        allocation = inherited
    else:
        profile = (
            "store-timing"
            if config.get("service_class") == "timing"
            else "store-functional"
        )
        requested = str(object_mapping(config["resources"])["server_memory_bytes"])
        allocation = host_admission.acquire(
            host_admission.select(profile, requested), deadline=deadline
        )
    storage_placement(config, allocation)
    allocation.register(unit_name(state))
    if allocation.profile.exclusive:
        with host_admission.allocation_metadata(allocation.directory) as ledger:
            borrowed = ledger["owners"][allocation.nonce].setdefault(
                "borrowed_services", []
            )
            if str(state) not in borrowed:
                borrowed.append(str(state))
    if not allocation.profile.exclusive:
        with host_admission.allocation_metadata(allocation.directory) as ledger:
            ledger["owners"][allocation.nonce].update(
                service=str(state), parkable=bool(config.get("resident"))
            )
    write_json(
        state / "service-launch.json",
        {
            "allocation": str(allocation.directory / allocation.nonce),
            "deadline": deadline,
            "generation": config["instance_id"],
            "supervisor_generation": config.get("service_supervisor"),
            "server_generation": config.get("server"),
            "service_class": config.get("service_class", "functional"),
            "server_memory_bytes": object_mapping(config["resources"])[
                "server_memory_bytes"
            ],
        },
    )
    return allocation


def park_service(
    state: Path,
    *,
    deadline: float | None = None,
    expected_binding: host_admission.BoundUnit | None = None,
) -> bool:
    """Park only this materialized idle service; preserve all disk and evidence."""
    previous = getattr(_STARTUP, "deadline", None)
    if deadline is not None:
        _STARTUP.deadline = deadline if previous is None else min(previous, deadline)
    try:
        with lifecycle_reservation(state):
            return (
                _park_service(state)
                if expected_binding is None
                else _park_service(state, expected_binding=expected_binding)
            )
    finally:
        _STARTUP.deadline = previous


def _park_service(
    state: Path, *, expected_binding: host_admission.BoundUnit | None = None
) -> bool:
    config = config_for(state)
    if not config.get("resident") or not config.get("unit_materialized"):
        raise SupervisorError(
            "Service lifecycle is not eligible for coordinated parking"
        )
    deadline = getattr(_STARTUP, "deadline", None)
    observed = host_admission.control_unit_observation(unit_name(state), deadline)
    if observed.get("LoadState") == "not-found" or observed.get("ActiveState") in {
        "inactive",
        "failed",
    }:
        return False
    group = observed.get("ControlGroup", "")
    if (
        observed.get("ActiveState") != "active"
        or not group.startswith("/")
        or group == "/"
        or ".." in Path(group).parts
        or not re.fullmatch(r"[a-f0-9]{32}", observed.get("InvocationID", ""))
    ):
        raise SupervisorError("Service parking requires its actual running lifetime")
    if expected_binding is not None and (
        type(expected_binding.get("inode")) is not int
        or group != expected_binding.get("group")
        or observed["InvocationID"] != expected_binding.get("invocation")
        or host_admission.group_identity(group) != expected_binding.get("inode")
    ):
        raise SupervisorError("Service parking lifetime changed since host admission")
    all_contexts_drained(state)
    if host_admission.control_unit_observation(
        unit_name(state), deadline
    ) != observed or (
        expected_binding is not None
        and host_admission.group_identity(group) != expected_binding.get("inode")
    ):
        raise SupervisorError("Service parking lifetime changed while draining")
    with state_lock(state):
        config = config_for(state)
        config["parked"] = True
        write_json(state / "config.json", config)
    stop(state, config)
    return True


def unpark_service(state: Path, *, deadline: float | None = None) -> None:
    with lifecycle_reservation(state):
        config = config_for(state)
        if config.get("parked"):
            start(state, config, deadline=deadline)


def start(
    state: Path,
    config: dict[str, object],
    *,
    validation: bool = False,
    deadline: float | None = None,
) -> None:
    deadline = time.monotonic() + 40 if deadline is None else deadline
    previous = getattr(_STARTUP, "deadline", None)
    deadline = deadline if previous is None else min(previous, deadline)
    _STARTUP.deadline = deadline
    try:
        remaining(deadline)
        with lifecycle_reservation(state):
            _start(state, config, validation=validation, deadline=deadline)
    finally:
        _STARTUP.deadline = previous


def _start(
    state: Path,
    config: dict[str, object],
    *,
    validation: bool,
    deadline: float,
) -> None:
    owner = service_directory(state)
    if owner != state:
        return start(owner, config_for(owner), validation=validation, deadline=deadline)
    if deadline is None:
        deadline = time.monotonic() + 40
    if (
        config["admission"] == "validation_required"
        or config.get("derived_rebuild_pending")
    ) and not validation:
        raise SupervisorError(
            "Restored database requires validate before normal server start"
        )
    maintenance = (state / ".maintenance-listener.json").exists()
    if maintenance:
        require_maintenance_owner(state, config)
        if not validation:
            raise SupervisorError("Maintenance listener refuses ordinary startup")
    parked = bool(config.get("parked"))
    resumed = False
    try:
        if active(state):
            if not listener_ready(state, config):
                raise SupervisorError("Owned server unit is active but unhealthy")
            if not maintenance and not protocol_ready(state, config):
                establish_protocol_readiness(state, config, deadline)
            if parked:
                with state_lock(state):
                    config.update(config_for(state))
                    config["parked"] = False
                    write_json(state / "config.json", config)
            if not validation:
                open_context_admission(state)
                config.update(config_for(state))
            resumed = True
            return None
        systemctl("show-environment")
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise SupervisorError("Invalid resource configuration")
        remaining(deadline)
        allocation_owner = service_allocation(state, config, deadline)
        environment = systemd_environment()
        host_admission.enforce_parent(allocation_owner.profile, environment)
        materialize_service(state, config, allocation_owner)
        if parked:
            # _serve may proceed only after deliberate startup owns its allocation.
            # Failed admission must retain the premise used by queued reconciliation.
            with state_lock(state):
                config.update(config_for(state))
                config["parked"] = False
                write_json(state / "config.json", config)
            # An admitted park/unpark is a new startup, not another failure retry.
            systemctl("reset-failed", unit_name(state), check=False)
        result = systemctl("start", unit_name(state), check=False)
        if result.returncode:
            allocation_owner.release()
            raise SupervisorError("Cannot start owned persistent service unit")
        while time.monotonic() < deadline:
            if listener_ready(state, config):
                if not maintenance:
                    establish_protocol_readiness(state, config, deadline)
                if not validation:
                    config["admission"] = "open"
                    config["accepting_writes"] = True
                    write_json(state / "config.json", config)
                    open_context_admission(state)
                resumed = True
                return None
            if not active(state):
                break
            time.sleep(0.1)
        systemctl("stop", unit_name(state), check=False)
        raise SupervisorError(
            "Server did not become healthy; inspect the private bounded server.log"
        )
    finally:
        if parked and not resumed:
            # Keep any partial lifetime charged through its existing owner. Closing
            # admission and retaining parking intent does not prove a service drain.
            with state_lock(state):
                config.update(config_for(state))
                config["parked"] = True
                write_json(state / "config.json", config)
            close_context_admission(state)
            config.update(config_for(state))


def stop(state: Path, config: dict[str, object], *, abrupt: bool = False) -> None:
    prior = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 90
    _STARTUP.deadline = deadline if prior is None else min(deadline, prior)
    try:
        with lifecycle_reservation(state):
            close_context_admission(state)
            config.update(config_for(state))
            _stop(state, config, abrupt=abrupt)
    finally:
        _STARTUP.deadline = prior


def _stop(state: Path, config: dict[str, object], *, abrupt: bool = False) -> None:
    group_result = systemctl(
        "show", "--property=ControlGroup", "--value", unit_name(state), check=False
    )
    group = group_result.stdout.strip()
    group_path = (
        Path("/sys/fs/cgroup") / group.lstrip("/")
        if group.startswith("/") and group != "/"
        else None
    )
    config["accepting_writes"] = False
    if config["admission"] != "validation_required":
        config["admission"] = "quiesced"
        write_json(state / "config.json", config)
    if not abrupt:
        all_contexts_drained(state)
    if abrupt:
        if active(state):
            killed = systemctl(
                "kill",
                "--kill-whom=all",
                "--signal=SIGKILL",
                unit_name(state),
                check=False,
            )
            if killed.returncode and active(state):
                raise SupervisorError("Abrupt fixture SIGKILL was refused")
            # Do not enqueue a competing graceful stop while the kill is being
            # processed. Wait for collection of the killed process group.
        while True:
            remaining(_STARTUP.deadline)
            observation = systemctl(
                "show",
                "--property=ActiveState",
                "--value",
                unit_name(state),
                check=False,
            )
            if observation.returncode == 0 and observation.stdout.strip() in {
                "inactive",
                "failed",
            }:
                break
            time.sleep(0.05)
    else:
        # is-active excludes activating/deactivating, but they still require an
        # actual stop job before an offline action can claim terminal state.
        systemctl("stop", unit_name(state), check=False)
    result = systemctl(
        "show", "--property=ActiveState", "--value", unit_name(state), check=False
    )
    if result.returncode or result.stdout.strip() not in {"inactive", "failed"}:
        raise SupervisorError("Server remains active; database copy is unsafe")
    if group_path is not None:
        while group_path.exists():
            events = group_path / "cgroup.events"
            if not events.exists() or "populated 1" not in events.read_text():
                break
            remaining(_STARTUP.deadline)
            time.sleep(0.05)

    release_stopped_service(state, config)


def release_stopped_service(state: Path, config: dict[str, object]) -> None:
    """Release the drained storage owner, never its caller's borrowed host lane."""
    launch_path = state / "service-launch.json"
    if not launch_path.exists():
        return
    launch = read_json(launch_path)
    if launch.get("generation") != config["instance_id"]:
        raise SupervisorError("Stopped service allocation has a different generation")

    try:
        owner = host_admission.inherit(
            {host_admission.MARKER: str(launch["allocation"])}, handoff=True
        )
    except host_admission.AdmissionError:
        # A dead launch caller is settled by host reconciliation using the
        # recorded cgroup inode/invocation, not by guessing a live replacement.
        return
    if owner is None or owner.profile.exclusive:
        return
    with host_admission.allocation_metadata(owner.directory) as ledger:
        record = ledger["owners"].get(owner.nonce)
        if (
            not isinstance(record, dict)
            or record.get("service") != str(state)
            or unit_name(state) not in record["units"]
        ):
            raise SupervisorError(
                "Stopped storage allocation does not own this service"
            )
    if not owner.release():
        raise SupervisorError(
            "Stopped storage allocation still owns an undrained lifetime"
        )


def server_environment(
    config: dict[str, object], credentials: dict[str, object], *, bootstrap: bool = True
) -> dict[str, str]:
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid allocation")
    # Inherited SURREAL_* settings cannot silently override the owned profile.
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("SURREAL_")
    }
    environment.update(
        {
            "SURREAL_USER": str(credentials["username"]),
            "SURREAL_PASS": str(credentials["password"]),
            "SURREAL_WEBSOCKET_MAX_MESSAGE_SIZE": str(MESSAGE_BYTES),
            "SURREAL_WEBSOCKET_MAX_WRITE_BUFFER_SIZE": str(8 * MIB),
            # SurrealDB counts its implicit connection session against this cap.
            # Two permits exactly one SDK application session on each socket.
            "SURREAL_WEBSOCKET_MAX_ATTACHED_SESSIONS": "2",
            "SURREAL_MEMORY_THRESHOLD": str(allocation["memory_threshold_bytes"]),
            "SURREAL_ROCKSDB_BLOCK_CACHE_SIZE": str(
                allocation["rocksdb_block_cache_bytes"]
            ),
            "SURREAL_ROCKSDB_WRITE_BUFFER_SIZE": str(
                allocation["rocksdb_write_buffer_bytes"]
            ),
            "SURREAL_ROCKSDB_MAX_WRITE_BUFFER_NUMBER": "2",
            "SURREAL_ROCKSDB_THREAD_COUNT": "2",
            "SURREAL_ROCKSDB_JOBS_COUNT": "4",
            "SURREAL_ROCKSDB_MAX_CONCURRENT_SUBCOMPACTIONS": "2",
            "SURREAL_ROCKSDB_COMPACTION_READAHEAD_SIZE": str(4 * MIB),
            "SURREAL_ROCKSDB_KEEP_LOG_FILE_NUM": "2",
            "SURREAL_ROCKSDB_STORAGE_LOG_LEVEL": "error",
            "SURREAL_RUNTIME_WORKER_THREADS": "4",
            "TOKIO_WORKER_THREADS": "4",
            "RAYON_NUM_THREADS": "4",
        }
    )
    if not bootstrap:
        environment.pop("SURREAL_USER", None)
        environment.pop("SURREAL_PASS", None)
    return environment


class BoundedLog:
    """Bound each write and retain a fixed number of private rotated files."""

    def __init__(self, path: Path, maximum: int, backups: int, secret: str):
        self.path, self.maximum, self.backups, self.secret = (
            path,
            maximum,
            backups,
            secret.encode(),
        )

    def append(self, block: bytes) -> None:
        block = block.replace(self.secret, b"[REDACTED]")
        for offset in range(0, len(block), self.maximum):
            part = block[offset : offset + self.maximum]
            if (
                self.path.exists()
                and self.path.stat().st_size + len(part) > self.maximum
            ):
                for index in range(self.backups, 0, -1):
                    source = (
                        self.path
                        if index == 1
                        else self.path.with_suffix(f".log.{index - 1}")
                    )
                    destination = self.path.with_suffix(f".log.{index}")
                    if source.exists():
                        source.replace(destination)
            fd = os.open(
                self.path, os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW, 0o600
            )
            with os.fdopen(fd, "ab") as output:
                output.write(part)


def serve(state: Path) -> int:
    """Give each manager startup one finite clock, independently of science."""
    prior = getattr(_STARTUP, "deadline", None)
    deadline = time.monotonic() + 40
    _STARTUP.deadline = deadline if prior is None else min(prior, deadline)
    try:
        return _serve(state)
    finally:
        _STARTUP.deadline = prior


def _serve(state: Path) -> int:
    config = config_for(state)
    if config.get("parked"):
        return 0
    maintenance = (state / ".maintenance-listener.json").exists()
    if maintenance:
        require_maintenance_owner(state, config)
    deadline = _STARTUP.deadline
    launch_path = state / "service-launch.json"
    if launch_path.is_file():
        launch = storage_launch(state, config)
        if "binding" in launch:
            allocation_owner = resume_storage_allocation(state, config)
        else:
            allocation_owner = host_admission.inherit(
                {host_admission.MARKER: str(launch["allocation"])}, handoff=True
            )
        if allocation_owner is None:
            raise SupervisorError("Storage launch has no admitted owner")
    else:
        allocation_owner = service_allocation(state, config, deadline)
        launch = storage_launch(state, config)
    storage_placement(config, allocation_owner)
    if "binding" not in launch:
        launch_deadline = launch["deadline"]
        if not isinstance(launch_deadline, (int, float)):
            raise SupervisorError("Storage launch lacks a finite admission clock")
        _STARTUP.deadline = min(deadline, launch_deadline)
    remaining(_STARTUP.deadline)
    allocation_owner.bind(unit_name(state))
    with host_admission.allocation_metadata(allocation_owner.directory) as ledger:
        binding = dict(
            ledger["owners"][allocation_owner.nonce]["units"][unit_name(state)]
        )
    launch["binding"] = binding
    write_json(launch_path, launch)
    os.environ.update(allocation_owner.environment())
    host_admission.enforce_parent(allocation_owner.profile, systemd_environment())
    os.sched_setaffinity(0, list(allocation_owner.profile.cores))
    remaining(_STARTUP.deadline)
    credentials_path = state / "credentials.json"
    if (
        credentials_path.stat().st_mode & 0o077
        or credentials_path.stat().st_uid != os.getuid()
    ):
        raise SupervisorError(
            "Credentials must be owned by the current user with mode 0600"
        )
    credentials = read_json(credentials_path)
    server = config["server"]
    if not isinstance(server, dict):
        raise SupervisorError("Invalid server receipt")
    if server.get("binary_sha256") != file_digest(Path(str(server["binary"]))):
        raise SupervisorError(
            "Storage binary differs from its admitted generation; explicitly readmit"
        )
    database = database_endpoint(state)
    command = [
        str(server["binary"]),
        "start",
        "--no-banner",
        "--log=warn",
        "--no-defaults",
        "--deny-guests",
        "--deny-net",
        "--deny-scripting",
        "--bind",
        f"127.0.0.1:{config['port']}",
        "--query-timeout=90s",
        "--transaction-timeout=90s",
        "--temporary-directory",
        str(state / "tmp"),
        database,
    ]
    log = BoundedLog(
        state / "server.log",
        integer(config["log_max_bytes"]),
        integer(config["log_backups"]),
        str(credentials["password"]),
    )
    # A recovery listener observes existing ROOT authorities; it must never
    # bootstrap a supplied candidate into an empty or unknown catalog.
    environment = server_environment(config, credentials, bootstrap=not maintenance)
    child = subprocess.Popen(
        command,
        env=environment,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    write_json(
        state / "server-process.json",
        {
            "instance_id": config["instance_id"],
            "pid": child.pid,
            "start": native_operation.start_identity(child.pid),
            "allocation": launch["allocation"],
        },
    )

    def forward(signum: int, _frame: object) -> None:
        if child.poll() is None:
            child.send_signal(signum)

    signal.signal(signal.SIGTERM, forward)
    signal.signal(signal.SIGINT, forward)
    stream = child.stdout
    if stream is None:
        raise SupervisorError("Server output pipe was not created")

    def capture() -> None:
        # Retain overlap so a secret split across pipe reads is still redacted.
        tail = b""
        overlap = len(log.secret)
        while block := stream.read(8192):
            tail += block
            safe = len(tail) - overlap
            if safe > 0:
                # Redact before splitting; retaining the overlap handles partial tokens.
                redacted = tail.replace(log.secret, b"[REDACTED]")
                if redacted != tail:
                    tail = redacted
                    safe = max(0, len(tail) - overlap)
                log.append(tail[:safe])
                tail = tail[safe:]
        log.append(tail)

    if maintenance:
        threading.Thread(
            target=watch_maintenance_owner, args=(state, child), daemon=True
        ).start()
    reader = threading.Thread(target=capture, daemon=True)
    reader.start()
    result = child.wait()
    reader.join(timeout=5)
    stream.close()
    return result


def backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
    with lifecycle_reservation(state):
        return _backup(state, config, destination)


def _backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
    if config.get("derived_rebuild_pending"):
        raise SupervisorError(
            "Incomplete derived rebuild cannot be backed up as current state"
        )
    if destination.absolute() == state or state in destination.absolute().parents:
        raise SupervisorError(
            "Backup destination must be outside the live state directory"
        )
    stop(state, config)
    if not (state / "database").is_dir():
        raise SupervisorError("No initialized database to back up")
    destination = checked_directory(destination, empty=True)
    for name in ("database", "config.json", "credentials.json"):
        source = state / name
        if source.is_dir():
            shutil.copytree(source, destination / name, symlinks=False)
        else:
            shutil.copy2(source, destination / name)
    generations = owned_generations(state)
    for context in managed_contexts(state):
        if context != state:
            target = destination / context.relative_to(state)
            target.mkdir(mode=0o700, parents=True)
            shutil.copy2(context / "config.json", target / "config.json")
    for generation in sorted(generations):
        shutil.copytree(generation, destination / ".generations" / generation.name)
    if (state / ".contexts").exists():
        shutil.copytree(state / ".contexts", destination / ".contexts")
    files = {
        str(path.relative_to(destination)): file_digest(path)
        for path in sorted(destination.rglob("*"))
        if path.is_file()
    }
    write_json(
        destination / "backup.json",
        {
            "owner": OWNER,
            "backup_version": 2,
            "state_root": str(state),
            "interpretation": config["interpretation"],
            "files": files,
        },
    )
    return {
        "backup": str(destination),
        "interpretation": config["interpretation"],
        "server_stopped": True,
    }


def private_offline_state(state: Path, config: dict[str, object]) -> None:
    """Only the stopped service's sole database may be replaced by its owner."""
    if service_directory(state) != state or managed_contexts(state) != [state]:
        raise SupervisorError("Derived cutover refuses shared service contexts")
    if config["accepting_writes"] or config["admission"] not in {
        "quiesced",
        "validation_required",
    }:
        raise SupervisorError("Derived cutover requires closed admission")
    all_contexts_drained(state)
    if (state / ".maintenance-listener.json").exists():
        # Recovery first drains a predecessor invocation; no ordinary listener
        # or live competing lifecycle owner may be adopted.
        drain_maintenance_listener(state)
        config.update(config_for(state))
    if active(state):
        raise SupervisorError("Derived cutover requires the owned server stopped")
    # is-active alone does not establish process-group drain.
    stop(state, config)


def database_endpoint(state: Path) -> str:
    return f"rocksdb://{state / 'database'}?sync=every&versioned=false"


def sql_identifier(value: object) -> str:
    if (
        not isinstance(value, str)
        or re.fullmatch(r"[A-Za-z0-9_-]{1,128}", value) is None
    ):
        raise SupervisorError("Unsupported owned database identifier")
    return "`" + value + "`"


def require_maintenance_owner(state: Path, config: dict[str, object]) -> None:
    selected = read_json(state / ".maintenance-listener.json")
    owner = object_mapping(selected.get("owner"))
    if (
        config.get("accepting_writes") is not False
        or config.get("admission") not in {"quiesced", "validation_required"}
        or selected.get("instance_id") != config.get("instance_id")
        or selected.get("server") != config.get("server")
        or selected.get("resources") != config.get("resources")
        or selected.get("supervisor") != config.get("service_supervisor")
        or selected.get("credentials_sha256") != file_digest(state / "credentials.json")
        or not reservation_live(owner)
        or read_json(state / "lifecycle-owner.json") != owner
        or service_directory(state) != state
        or managed_contexts(state) != [state]
    ):
        raise SupervisorError(
            "Maintenance listener requires its exact live closed owner"
        )


def watch_maintenance_owner(state: Path, child: subprocess.Popen[bytes]) -> None:
    """Owner loss closes the child; recovery still owns exact drain/release."""
    while child.poll() is None:
        try:
            require_maintenance_owner(state, config_for(state))
        except (SupervisorError, OSError, KeyError, ValueError):
            if child.poll() is None:
                child.terminate()
            return
        time.sleep(0.1)


def drain_maintenance_listener(state: Path) -> None:
    path = state / ".maintenance-listener.json"
    if not path.exists():
        return
    config = config_for(state)
    selected = read_json(path)
    owner = object_mapping(selected.get("owner"))
    if (
        selected.get("instance_id") != config.get("instance_id")
        or selected.get("server") != config.get("server")
        or selected.get("resources") != config.get("resources")
        or (
            reservation_live(owner)
            and read_json(state / "lifecycle-owner.json") != owner
        )
    ):
        raise SupervisorError(
            "Unknown maintenance listener retained; admission remains closed"
        )
    observation = systemctl(
        "show",
        "--property=LoadState",
        "--property=ActiveState",
        "--property=ControlGroup",
        "--property=InvocationID",
        unit_name(state),
        check=False,
    )
    fields = dict(
        line.split("=", 1) for line in observation.stdout.splitlines() if "=" in line
    )
    if observation.returncode or any(
        key not in fields for key in ("LoadState", "ActiveState", "ControlGroup")
    ):
        raise SupervisorError(
            "Cannot observe maintenance lifetime; admission remains closed"
        )
    group = fields["ControlGroup"]
    # is-active excludes activating/deactivating. Every extant group or busy
    # invocation needs exact authority before stop can target its unit name.
    if fields["ActiveState"] not in {"inactive", "failed"} or group:
        launch = storage_launch(state, config)
        binding = object_mapping(launch.get("binding"))
        if (
            fields.get("InvocationID") != binding.get("invocation")
            or group != binding.get("group")
            or host_admission.group_identity(group) != binding.get("inode")
        ):
            raise SupervisorError(
                "Maintenance invocation changed; admission remains closed"
            )
    # _stop proves the exact service cgroup/process drain before releasing its
    # allocation. Failure preserves the durable listener and pending phase.
    stop(state, config)
    path.unlink()
    descriptor = os.open(state, os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def maintenance_endpoint(state: Path) -> str:
    if getattr(_LIFECYCLE, "state", None) != state:
        raise SupervisorError("Maintenance requires a reserved lifecycle owner")
    config = config_for(state)
    if (state / ".maintenance-listener.json").exists():
        require_maintenance_owner(state, config)
        if not listener_ready(state, config):
            raise SupervisorError(
                "Maintenance listener became unavailable; admission remains closed"
            )
    else:
        private_offline_state(state, config)
        config["service_supervisor"] = publish_generation(state)
        write_json(state / "config.json", config)
        write_json(
            state / ".maintenance-listener.json",
            {
                "owner": read_json(state / "lifecycle-owner.json"),
                "instance_id": config["instance_id"],
                "server": config["server"],
                "resources": config["resources"],
                "supervisor": config["service_supervisor"],
                "credentials_sha256": file_digest(state / "credentials.json"),
            },
        )
        start(state, config, validation=True)
    return "http://127.0.0.1:" + str(config["port"])


def maintenance_sql(
    state: Path, config: dict[str, object], credentials: dict[str, object], body: str
) -> tuple[str, subprocess.CompletedProcess[str]]:
    """Fresh authenticated CLI session on the reserved supervised maintenance child."""
    endpoint = maintenance_endpoint(state)
    selection = []
    for key in ("namespace", "database"):
        if config.get(key):
            selection.extend(["--" + key, str(config[key])])
    nonce = uuid.uuid4().hex
    query = (
        "BEGIN; "
        + body
        + f" RETURN {{maintenance: '{nonce}', value: $value}}; COMMIT;\n"
    )
    result = subprocess.run(
        [
            str(object_mapping(config["server"])["binary"]),
            "sql",
            "--log=none",
            "--endpoint",
            endpoint,
            *selection,
            "--json",
            "--hide-welcome",
        ],
        input=query,
        env=server_environment(config, credentials),
        cwd=state,
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    return nonce, result


def maintenance_results(config: dict[str, object], output: str) -> list[object]:
    """Decode the pinned native SQL prompt framing without accepting extra results."""
    prompt = (
        "/".join(
            str(config[key]) for key in ("namespace", "database") if config.get(key)
        )
        + ">"
    )
    lines = [line.strip().removeprefix(prompt).strip() for line in output.splitlines()]
    returned = json.loads("\n".join(line for line in lines if line))
    if not isinstance(returned, list):
        raise TypeError("Native SQL output is not a result list")
    return [item for item in returned if item is not None]


def maintenance_failure(
    state: Path,
    credentials: dict[str, object],
    result: subprocess.CompletedProcess[str],
) -> None:
    # Keep bounded private failure evidence without exposing query text,
    # authentication arguments, or candidate account passwords.
    diagnostics = {"stdout": result.stdout, "stderr": result.stderr}
    authorities = [credentials]
    candidate = state / ".maintenance-credentials.json"
    if candidate.is_file():
        authorities.append(read_json(candidate))
    for name, raw in diagnostics.items():
        value = raw
        for authority in authorities:
            for field in ("password", "selection_password"):
                secret = authority.get(field)
                if isinstance(secret, str) and secret:
                    value = value.replace(secret, "[REDACTED]")
        diagnostics[name] = value[-MESSAGE_BYTES:]
    slot = state / "maintenance-cli-failure.json"
    if slot.is_symlink():
        raise SupervisorError(
            "Unknown maintenance diagnostic retained; admission remains closed"
        )
    if slot.exists():
        identity = slot.stat()
        # Each retained Unicode character needs at most two six-byte JSON
        # surrogate escapes. The envelope includes the full signed exit-code range.
        envelope = {
            "owner": OWNER,
            "kind": "maintenance-cli-failure-v1",
            "returncode": -2147483648,
            "stdout": "",
            "stderr": "",
        }
        maximum_size = (
            24 * MESSAGE_BYTES
            + len(json.dumps(envelope, indent=2, sort_keys=True).encode())
            + 1
        )
        if (
            not stat.S_ISREG(identity.st_mode)
            or identity.st_uid != os.getuid()
            or stat.S_IMODE(identity.st_mode) != 0o600
            or identity.st_nlink != 1
            or identity.st_size > maximum_size
        ):
            raise SupervisorError(
                "Unknown maintenance diagnostic retained; admission remains closed"
            )
        previous = read_json(slot)
        if (
            previous.get("owner") != OWNER
            or previous.get("kind") != "maintenance-cli-failure-v1"
        ):
            raise SupervisorError(
                "Unknown maintenance diagnostic retained; admission remains closed"
            )
    write_json(
        slot,
        {
            "owner": OWNER,
            "kind": "maintenance-cli-failure-v1",
            "returncode": result.returncode,
            **diagnostics,
        },
    )
    raise SupervisorError("Maintenance acknowledgment failed; admission remains closed")


def maintenance_query(
    state: Path, config: dict[str, object], credentials: dict[str, object], body: str
) -> dict[str, object]:
    """Require exactly one transaction acknowledgment from an output-neutral query."""
    nonce, result = maintenance_sql(state, config, credentials, body)
    nonempty: list[object] = []
    try:
        nonempty = maintenance_results(config, result.stdout)
        valid = (
            len(nonempty) == 1
            and isinstance(nonempty[0], dict)
            and nonempty[0].get("maintenance") == nonce
        )
    except (ValueError, TypeError):
        valid = False
    if result.returncode or not valid:
        maintenance_failure(state, credentials, result)
    return object_mapping(nonempty[0])


MAINTENANCE_CATALOG_FIELDS = {
    "ROOT": frozenset(
        {"accesses", "defaults", "namespaces", "nodes", "system", "users", "config"}
    ),
    "NS": frozenset({"accesses", "databases", "users"}),
    "DB": frozenset(
        {
            "accesses",
            "apis",
            "analyzers",
            "buckets",
            "functions",
            "modules",
            "models",
            "params",
            "tables",
            "users",
            "configs",
            "sequences",
        }
    ),
}


def maintenance_catalog(
    state: Path, config: dict[str, object], credentials: dict[str, object], scope: str
) -> dict[str, object]:
    """Read one pinned catalog and its exact transaction acknowledgment.

    INFO must be a statement: assigning unselected ROOT INFO requires a namespace
    in the pinned release. Only select NS/DB after the caller observes it exists.
    """
    if scope not in MAINTENANCE_CATALOG_FIELDS:
        raise SupervisorError("Unknown maintenance catalog scope")
    selected = dict(config)
    if scope == "ROOT":
        selected.update(namespace=None, database=None)
    elif scope == "NS":
        selected["database"] = None
        if not selected.get("namespace"):
            raise SupervisorError("Namespace catalog requires an observed namespace")
    elif not selected.get("namespace") or not selected.get("database"):
        raise SupervisorError("Database catalog requires an observed database")
    nonce, result = maintenance_sql(
        state, selected, credentials, f"INFO FOR {scope}; LET $value=true;"
    )
    nonempty: list[object] = []
    try:
        nonempty = maintenance_results(selected, result.stdout)
        valid = (
            len(nonempty) == 2
            and isinstance(nonempty[0], dict)
            and set(nonempty[0]) == MAINTENANCE_CATALOG_FIELDS[scope]
            and all(isinstance(value, dict) for value in nonempty[0].values())
            and isinstance(nonempty[1], dict)
            and set(nonempty[1]) == {"maintenance", "value"}
            and nonempty[1].get("maintenance") == nonce
            and nonempty[1].get("value") is True
        )
    except (ValueError, TypeError):
        valid = False
    if result.returncode or not valid:
        maintenance_failure(state, credentials, result)
    return object_mapping(nonempty[0])


def remove_analysis_state(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Drain only owned derived rows in fixed pages; retain every unrelated input."""
    response = maintenance_query(
        state,
        config,
        credentials,
        "LET $pins=SELECT key FROM canonical_protections WHERE !released AND expires_at>time::micros() LIMIT 1; "
        "IF array::len($pins)!=0 { THROW 'live source protections refuse derived cutover'; }; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError("Live protection check was not acknowledged")
    # Explicit edge deletion bounds graph-pointer work before any endpoint deletion.
    for table, predicate in (
        ("canonical_analysis_edges", "true"),
        ("canonical_analysis_nodes", "true"),
        ("canonical_analysis_inputs", "true"),
        ("canonical_roots", "owner_kind='analysis'"),
        ("canonical_analyses", "true"),
        ("canonical_guards", "string::starts_with(key,'analysis:')"),
    ):
        while True:
            guards = ""
            if table == "canonical_roots":
                guards = "FOR $row IN $rows { LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$row.problem) FOR UPDATE; IF $guard=NONE { THROW 'source authority missing'; }; }; "
            if table == "canonical_analysis_inputs":
                guards = "FOR $row IN $rows { LET $guard=SELECT * FROM ONLY type::record('canonical_guards','execution-run:'+$row.run) FOR UPDATE; IF $guard=NONE { THROW 'run authority missing'; }; }; "
            response = maintenance_query(
                state,
                config,
                credentials,
                f"LET $rows=SELECT * FROM {table} WHERE {predicate} ORDER BY key LIMIT 64; "  # noqa: S608 -- Fixed internal table and predicate declarations.
                + guards
                + "LET $deleted=DELETE $rows.id RETURN NONE; LET $value=array::len($rows);",
            )
            count = response.get("value")
            if type(count) is not int or not 0 <= count <= 64:
                raise SupervisorError("Invalid bounded derived cleanup acknowledgment")
            if count == 0:
                break
    after = ""
    while True:
        response = maintenance_query(
            state,
            config,
            credentials,
            "LET $rows=SELECT * FROM canonical_guards WHERE string::starts_with(key,'retention:') AND key>"  # noqa: S608 -- Cursor is encoded as a JSON string.
            + json.dumps(after)
            + " ORDER BY key LIMIT 64; "
            "FOR $row IN $rows { UPDATE ONLY $row.id SET incarnation=<string>rand::uuid::v4(),generation=generation+1dec; }; "
            "LET $value={count:array::len($rows),after:array::last($rows.key) ?? ''};",
        )
        value = object_mapping(response.get("value"))
        count, cursor = value.get("count"), value.get("after")
        if (
            type(count) is not int
            or not 0 <= count <= 64
            or not isinstance(cursor, str)
            or (count and cursor <= after)
        ):
            raise SupervisorError("Invalid source authority rotation acknowledgment")
        if count == 0:
            break
        after = cursor


def fresh_database_identity(config: dict[str, object]) -> dict[str, object]:
    selected = dict(config)
    identity = uuid.uuid4().hex
    selected.update(
        namespace="pse_" + identity,
        database="canonical_" + identity,
        instance_id=str(uuid.uuid4()),
        admission="validation_required",
        accepting_writes=False,
        unit_materialized=False,
        restart_qualified=False,
    )
    return selected


def fresh_owned_database_identity(
    state: Path, config: dict[str, object]
) -> dict[str, object]:
    """Rebind a proved-offline owned service to a new database generation."""
    if (
        getattr(_LIFECYCLE, "state", None) != state
        or active(state)
        or (state / ".maintenance-listener.json").exists()
        or config.get("accepting_writes") is not False
        or config_for(state)["instance_id"] != config["instance_id"]
    ):
        raise SupervisorError(
            "Fresh database identity requires its drained lifecycle owner"
        )
    launch = state / "service-launch.json"
    if launch.exists():
        # The old binding cannot be carried into another service generation.
        # Preserve unknown receipts rather than hiding an ownership mismatch.
        storage_launch(state, config)
        launch.unlink()
        descriptor = os.open(state, os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    selected = fresh_database_identity(config)
    # The state-path unit is still this owner's materialization. Its immutable
    # executable and generation are replaced normally on the next start.
    selected["unit_materialized"] = config.get("unit_materialized", False)
    return selected


def fresh_credentials() -> dict[str, object]:
    return {
        "username": "pse-local-" + uuid.uuid4().hex,
        "password": secrets.token_urlsafe(36),
        "selection_username": "pse-selection",
        "selection_password": secrets.token_urlsafe(36),
    }


def require_private_catalog(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    """Global credential changes require actual sole ownership of stored content."""
    pending = object_mapping(config.get("derived_rebuild_pending", {}))
    allowed = {(config["namespace"], config["database"])}
    if pending:
        allowed.add((pending["namespace"], pending["database"]))
    root = maintenance_catalog(state, config, credentials, "ROOT")
    for namespace in object_mapping(root["namespaces"]):
        selected = {**config, "namespace": namespace}
        namespace_catalog = maintenance_catalog(state, selected, credentials, "NS")
        for database in object_mapping(namespace_catalog["databases"]):
            if (namespace, database) in allowed:
                continue
            content = maintenance_catalog(
                state, {**selected, "database": database}, credentials, "DB"
            )
            # Keep even empty databases/namespaces; refuse any unknown content.
            if any(content.values()):
                raise SupervisorError(
                    "Derived cutover refuses an unknown nonempty database"
                )


def rotate_database_accounts(
    state: Path,
    config: dict[str, object],
    old: dict[str, object],
    new: dict[str, object],
) -> None:
    pending = state / ".maintenance-credentials.json"
    write_json(pending, new)
    require_private_catalog(state, config, old)
    response = maintenance_query(
        state,
        config,
        old,
        f"DEFINE USER {sql_identifier(new['username'])} ON ROOT PASSWORD {json.dumps(new['password'])} ROLES OWNER; "
        f"DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD {json.dumps(new['selection_password'])} ROLES VIEWER; "
        f"REMOVE USER {sql_identifier(old['username'])} ON ROOT; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError(
            "Account rotation was not acknowledged; admission remains closed"
        )
    drain_maintenance_listener(state)
    config.update(config_for(state))
    write_json(state / "credentials.json", new)
    pending.unlink()


def discard_database(
    state: Path,
    selected: dict[str, object],
    old: dict[str, object],
    credentials: dict[str, object],
) -> None:
    # REMOVE DATABASE requires only its existing namespace (Base::Ns). Never
    # select the disposed database: native CLI session followup can resolve its
    # selected context after the removal. Preserve its namespace and service.
    response = maintenance_query(
        state,
        {**selected, "namespace": old["namespace"], "database": None},
        credentials,
        f"REMOVE DATABASE {sql_identifier(old['database'])}; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError(
            "Old database disposal was not acknowledged; admission remains closed"
        )

    if database_present(state, old, credentials):
        raise SupervisorError(
            "Old database remains addressable; admission remains closed"
        )


def database_present(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> bool:
    # Select no unknown namespace/database during an observational catalog read.
    root = maintenance_catalog(state, config, credentials, "ROOT")
    if config["namespace"] not in object_mapping(root["namespaces"]):
        return False
    namespace = maintenance_catalog(state, config, credentials, "NS")
    return config["database"] in object_mapping(namespace["databases"])


def maintenance_phase(state: Path, config: dict[str, object], phase: str) -> None:
    pending = object_mapping(config["derived_rebuild_pending"])
    config["service_supervisor"] = config_for(state)["service_supervisor"]
    pending["phase"] = phase
    config["derived_rebuild_pending"] = pending
    config["accepting_writes"] = False
    write_json(state / "config.json", config)


def current_marker(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    response = maintenance_query(
        state,
        config,
        credentials,
        "LET $marker=SELECT * FROM ONLY canonical_interpretations:current; "  # noqa: S608 -- Fixed policy interpretation is JSON quoted.
        f"IF $marker=NONE OR $marker.interpretation!={json.dumps(SUBSTRATE_INTERPRETATION)} {{ THROW 'current interpretation unavailable'; }}; LET $value=true;",
    )
    if response.get("value") is not True:
        raise SupervisorError("Current interpretation was not acknowledged")


def finish_maintenance(
    state: Path, config: dict[str, object], credentials: dict[str, object]
) -> None:
    pending = object_mapping(config["derived_rebuild_pending"])
    if pending["kind"] == "rebuild":
        verified_preserved_inputs(config)
    require_private_catalog(state, config, credentials)
    maintenance_phase(state, config, "disposing")
    old = {**config, "namespace": pending["namespace"], "database": pending["database"]}
    if database_present(state, old, credentials):
        discard_database(state, config, old, credentials)
    drain_maintenance_listener(state)
    config.update(config_for(state))
    config.pop("derived_rebuild_pending")
    config["admission"] = (
        "quiesced" if pending["kind"] == "rebuild" else "validation_required"
    )
    write_json(state / "config.json", config)


def copy_current_database(
    state: Path,
    source: dict[str, object],
    target: dict[str, object],
    credentials: dict[str, object],
) -> None:
    # Reserve the exact path durably before its exclusive creation. An empty,
    # still-unbound file at that reservation is the only pre-inode crash state.
    pending = object_mapping(target["derived_rebuild_pending"])
    nonce = pending.get("export_nonce")
    if nonce is None:
        if any(state.glob(".maintenance-export*.surql")):
            raise SupervisorError(
                "Unknown maintenance export retained; admission remains closed"
            )
        nonce = uuid.uuid4().hex
        pending["export_nonce"] = nonce
        pending["export_path"] = ".maintenance-export-" + nonce + ".surql"
        target["derived_rebuild_pending"] = pending
        write_json(state / "config.json", target)
    if (
        not isinstance(nonce, str)
        or re.fullmatch(r"[a-f0-9]{32}", nonce) is None
        or pending.get("export_path") != ".maintenance-export-" + nonce + ".surql"
    ):
        raise SupervisorError("Unknown maintenance export reservation")
    exported = state / str(pending["export_path"])
    if exported.is_symlink():
        raise SupervisorError("Maintenance export must not traverse symlinks")
    if exported.exists():
        descriptor = os.open(exported, os.O_WRONLY | os.O_NOFOLLOW)
        observed = os.fstat(descriptor)
        bound = pending.get("export_identity")
        private = (
            stat.S_ISREG(observed.st_mode)
            and observed.st_uid == os.getuid()
            and stat.S_IMODE(observed.st_mode) == 0o600
            and observed.st_nlink == 1
        )
        if (
            not private
            or (bound is None and observed.st_size != 0)
            or (bound is not None and bound != [observed.st_dev, observed.st_ino])
        ):
            os.close(descriptor)
            raise SupervisorError(
                "Unknown maintenance export retained; admission remains closed"
            )
    else:
        descriptor = os.open(
            exported, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
        )
    identity = os.fstat(descriptor)
    os.close(descriptor)
    pending["export_identity"] = [identity.st_dev, identity.st_ino]
    target["derived_rebuild_pending"] = pending
    write_json(state / "config.json", target)
    # A prior interrupted export is overwritten only after binding its identity.
    descriptor = os.open(exported, os.O_WRONLY | os.O_NOFOLLOW)
    try:
        observed = os.fstat(descriptor)
        if [observed.st_dev, observed.st_ino] != pending["export_identity"]:
            raise SupervisorError(
                "Maintenance export ownership changed; admission remains closed"
            )
        os.ftruncate(descriptor, 0)
    finally:
        os.close(descriptor)
    try:
        endpoint = maintenance_endpoint(state)
        binary = str(object_mapping(source["server"])["binary"])
        for action, identity in (("export", source), ("import", target)):
            result = subprocess.run(
                [
                    binary,
                    action,
                    "--log=none",
                    "--endpoint",
                    endpoint,
                    "--namespace",
                    str(identity["namespace"]),
                    "--database",
                    str(identity["database"]),
                    str(exported),
                ],
                env=server_environment(source, credentials),
                cwd=state,
                capture_output=True,
                check=False,
            )
            if result.returncode:
                raise SupervisorError(
                    "Native current restore copy failed; admission remains closed"
                )
    finally:
        drain_maintenance_listener(state)
        target.update(config_for(state))
        pending = object_mapping(target["derived_rebuild_pending"])
        if exported.exists():
            observed = exported.stat()
            if (
                exported.is_symlink()
                or [observed.st_dev, observed.st_ino] != pending["export_identity"]
            ):
                raise SupervisorError(
                    "Maintenance export ownership changed; admission remains closed"
                )
            exported.unlink()
        for field in ("export_identity", "export_path", "export_nonce"):
            pending.pop(field, None)
        write_json(state / "config.json", target)


def input_inventory(source: Path) -> dict[str, str]:
    """Read explicit inputs without changing their owner's paths or permissions."""
    if (
        any(path.is_symlink() for path in (source, *source.parents))
        or not source.is_dir()
    ):
        raise SupervisorError("Preserved authored inputs must be external real files")
    files = {}
    for path in sorted(source.rglob("*")):
        if path.is_symlink() or (not path.is_file() and not path.is_dir()):
            raise SupervisorError(
                "Preserved authored inputs must be external real files"
            )
        if path.is_file():
            files[str(path.relative_to(source))] = file_digest(path)
    if not files:
        raise SupervisorError("Explicit authored/external input inventory is required")
    return files


def preserve_inputs(source: Path, destination: Path, state: Path) -> Path:
    inventory = input_inventory(source.absolute())
    if any(
        path.is_symlink()
        for path in (destination.absolute(), *destination.absolute().parents)
    ):
        raise SupervisorError("Preserved input destination must not traverse symlinks")
    source = lexical_absolute(source)
    destination = lexical_absolute(destination)
    if source == state or state in source.parents or source in state.parents:
        raise SupervisorError("Preserved authored inputs must be external real files")
    if (
        destination in (source, state)
        or source in destination.parents
        or destination in source.parents
        or state in destination.parents
        or destination in state.parents
    ):
        raise SupervisorError(
            "Preserved input destination must be external and disjoint"
        )
    destination = checked_directory(destination, empty=True)
    preserved = destination / "inputs"
    shutil.copytree(source, preserved)
    if input_inventory(preserved) != inventory or input_inventory(source) != inventory:
        raise SupervisorError("Authored/external inputs changed during preservation")
    # The manifest may authorize disposal only after the copied bytes and names
    # are durable; do not change permissions or metadata on the input authority.
    for path in sorted(preserved.rglob("*")):
        if path.is_file():
            descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
    directories = [preserved, *(path for path in preserved.rglob("*") if path.is_dir())]
    for path in sorted(directories, key=lambda item: len(item.parts), reverse=True):
        descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    write_json(destination / "inputs.json", {"owner": OWNER, "files": inventory})
    descriptor = os.open(
        destination.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    )
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return preserved


def rebuild(
    state: Path, source: Path, destination: Path, initializer: list[str]
) -> dict[str, object]:
    """Explicit current reconstruction, requiring externally preserved input authority."""
    if not initializer:
        raise SupervisorError("Fresh rebuild requires a current initializer command")
    with lifecycle_reservation(state):
        old = config_for(state)
        private_offline_state(state, old)
        if old.get("derived_rebuild_pending"):
            raise SupervisorError(
                "Incomplete rebuild remains closed; explicit owner recovery is required"
            )
        preserved = preserve_inputs(source, destination, state)
        inventory = read_json(preserved.parent / "inputs.json")
        selected = fresh_owned_database_identity(state, old)
        initializer_owner = read_json(state / "lifecycle-owner.json")
        credentials = fresh_credentials()
        prior = read_json(state / "credentials.json")
        selected.update(
            interpretation=SUBSTRATE_INTERPRETATION,
            schema_interpretation=SUBSTRATE_INTERPRETATION,
            derived_rebuild_pending={
                "namespace": old["namespace"],
                "database": old["database"],
                "inputs": str(preserved),
                "initializer": initializer_owner,
                "kind": "rebuild",
                "phase": "accounts",
                "inputs_digest": file_digest(preserved.parent / "inputs.json"),
                "target_namespace": selected["namespace"],
                "target_database": selected["database"],
                "previous_username": prior["username"],
                "next_username": credentials["username"],
            },
        )
        write_json(state / "config.json", selected)
        rotate_database_accounts(state, selected, prior, credentials)
        initialize_rebuild(state, selected, initializer, preserved, inventory)
        finish_maintenance(state, selected, credentials)
        return public_status(state, selected)


def initialize_rebuild(
    state: Path,
    selected: dict[str, object],
    initializer: list[str],
    preserved: Path,
    inventory: dict[str, object],
) -> None:
    drain_maintenance_listener(state)
    selected["service_supervisor"] = config_for(state)["service_supervisor"]
    initializer_owner = read_json(state / "lifecycle-owner.json")
    pending = object_mapping(selected["derived_rebuild_pending"])
    pending["initializer"] = initializer_owner
    selected["derived_rebuild_pending"] = pending
    maintenance_phase(state, selected, "initializing")
    # Credentials and fresh target are durable before any listener can start.
    try:
        start(state, selected, validation=True)
        # Ordinary admission stays closed, including after owner death. Only
        # this explicit child can borrow the exact live lifecycle authority.
        result = subprocess.run(
            initializer,
            env={
                **os.environ,
                "PSE_SURREAL_STATE": str(state),
                "PSE_PRESERVED_INPUTS": str(preserved),
                "PSE_CANONICAL_INITIALIZER": str(initializer_owner["nonce"]),
            },
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if result.returncode:
            raise SupervisorError(
                "Current initializer failed; preserved inputs retained and admission closed"
            )
        validate_interpretation(state, selected, SUBSTRATE_INTERPRETATION)
        if (
            inventory.get("owner") != OWNER
            or input_inventory(preserved) != inventory.get("files")
            or read_json(preserved.parent / "inputs.json") != inventory
        ):
            raise SupervisorError(
                "Preserved authored inputs changed; old database retained"
            )
        maintenance_phase(state, selected, "validated")
    finally:
        selected["accepting_writes"] = False
        write_json(state / "config.json", selected)
        stop(state, selected)


def recover_credentials(
    state: Path, config: dict[str, object]
) -> tuple[dict[str, object], bool]:
    """Observe an atomic account change before deciding whether it committed."""
    pending = object_mapping(config["derived_rebuild_pending"])
    path = state / ".maintenance-credentials.json"
    current = read_json(state / "credentials.json")
    candidate = read_json(path) if path.exists() else None
    if candidate is not None and candidate.get("username") != pending["next_username"]:
        raise SupervisorError(
            "Unknown pending maintenance credentials; admission remains closed"
        )
    candidates = (
        [current] if candidate is None or candidate == current else [current, candidate]
    )
    admitted = []
    for credentials in candidates:
        try:
            root = maintenance_catalog(state, config, credentials, "ROOT")
        except SupervisorError:
            continue
        users = object_mapping(root["users"])
        value = {
            "current": credentials["username"] in users,
            "previous": pending["previous_username"] in users,
            "next": pending["next_username"] in users,
        }
        if (
            any(
                type(value.get(name)) is not bool
                for name in ("current", "previous", "next")
            )
            or value["previous"] == value["next"]
        ):
            raise SupervisorError(
                "Maintenance root-account state is uncertain; admission remains closed"
            )
        if value["current"]:
            admitted.append((credentials, value))
    drain_maintenance_listener(state)
    config.update(config_for(state))
    if len(admitted) != 1:
        raise SupervisorError(
            "Maintenance account authority is uncertain; admission remains closed"
        )
    credentials, value = admitted[0]
    if credentials["username"] == pending["next_username"] and value["next"]:
        write_json(state / "credentials.json", credentials)
        path.unlink(missing_ok=True)
        return credentials, True
    if credentials["username"] != pending["previous_username"] or not value["previous"]:
        raise SupervisorError("Maintenance account does not match recorded authority")
    if pending["phase"] in {"cleanup", "copy"}:
        if candidate is not None:
            raise SupervisorError("Unexpected account phase; admission remains closed")
        return credentials, False
    if pending["phase"] != "accounts":
        raise SupervisorError("Recorded account rotation was not observed")
    # The old root is still present and authenticates, while the exact candidate
    # is absent. The atomic transaction is established not to have committed.
    if candidate is None:
        candidate = fresh_credentials()
        pending["next_username"] = candidate["username"]
        config["derived_rebuild_pending"] = pending
        write_json(state / "config.json", config)
    rotate_database_accounts(state, config, credentials, candidate)
    return candidate, True


def verified_preserved_inputs(
    config: dict[str, object],
) -> tuple[Path, dict[str, object]]:
    pending = object_mapping(config["derived_rebuild_pending"])
    preserved = Path(str(pending.get("inputs", "")))
    manifest = preserved.parent / "inputs.json"
    if (
        not preserved.is_absolute()
        or manifest.is_symlink()
        or file_digest(manifest) != pending.get("inputs_digest")
    ):
        raise SupervisorError(
            "Preserved input inventory differs; admission remains closed"
        )
    inventory = read_json(manifest)
    if inventory.get("owner") != OWNER or input_inventory(preserved) != inventory.get(
        "files"
    ):
        raise SupervisorError(
            "Preserved authored inputs differ; admission remains closed"
        )
    return preserved, inventory


def recover_maintenance(state: Path, initializer: list[str]) -> dict[str, object]:
    with lifecycle_reservation(state):
        config = config_for(state)
        pending = object_mapping(config.get("derived_rebuild_pending"))
        if (
            pending.get("kind") == "current-restore"
            and pending.get("phase") == "backup-copy"
        ):
            complete_backup_copy(state, config)
            config = config_for(state)
            pending = object_mapping(config["derived_rebuild_pending"])
        private_offline_state(state, config)
        kind, phase = pending.get("kind"), pending.get("phase")
        phases = {
            "rebuild": {"accounts", "initializing", "validated", "disposing"},
            "current-restore": {
                "cleanup",
                "copy",
                "accounts",
                "validated",
                "disposing",
            },
        }
        if (
            not isinstance(kind, str)
            or not isinstance(phase, str)
            or kind not in phases
            or phase not in phases[kind]
            or config["interpretation"] != SUBSTRATE_INTERPRETATION
        ):
            raise SupervisorError("Unknown maintenance phase; admission remains closed")
        for name in (
            "namespace",
            "database",
            "target_namespace",
            "target_database",
            "previous_username",
            "next_username",
        ):
            sql_identifier(pending.get(name))
        if (config["namespace"], config["database"]) != (
            pending["target_namespace"],
            pending["target_database"],
        ) or (pending["namespace"], pending["database"]) == (
            config["namespace"],
            config["database"],
        ):
            raise SupervisorError(
                "Maintenance source/target identity differs; admission remains closed"
            )
        preserved = None
        inventory = None
        if kind == "rebuild":
            preserved, inventory = verified_preserved_inputs(config)
            if phase in {"accounts", "initializing"} and not initializer:
                raise SupervisorError(
                    "Interrupted initializer requires explicit current initializer command"
                )
        credentials, rotated = recover_credentials(state, config)
        require_private_catalog(state, config, credentials)
        if kind == "rebuild" and phase in {"accounts", "initializing"}:
            if database_present(state, config, credentials):
                discard_database(state, config, config, credentials)
            drain_maintenance_listener(state)
            config.update(config_for(state))
            selected = fresh_owned_database_identity(state, config)
            pending.update(
                target_namespace=selected["namespace"],
                target_database=selected["database"],
            )
            selected["derived_rebuild_pending"] = pending
            config = selected
            if preserved is None or inventory is None:
                raise SupervisorError("Missing preserved initializer inputs")
            initialize_rebuild(state, config, initializer, preserved, inventory)
        elif kind == "current-restore" and phase in {"cleanup", "copy"}:
            if rotated:
                raise SupervisorError("Unexpected account rotation before current copy")
            source = {
                **config,
                "namespace": pending["namespace"],
                "database": pending["database"],
            }
            if not database_present(state, source, credentials):
                raise SupervisorError(
                    "Current restore source is absent; admission remains closed"
                )
            current_marker(state, source, credentials)
            # Re-mint source authorities before copying again: no old creation
            # capability from an unadmitted partial target survives recovery.
            remove_analysis_state(state, source, credentials)
            if database_present(state, config, credentials):
                discard_database(state, config, config, credentials)
            drain_maintenance_listener(state)
            config.update(config_for(state))
            selected = fresh_owned_database_identity(state, config)
            pending.update(
                target_namespace=selected["namespace"],
                target_database=selected["database"],
            )
            selected["derived_rebuild_pending"] = pending
            config = selected
            maintenance_phase(state, config, "copy")
            copy_current_database(state, source, config, credentials)
            new_credentials = fresh_credentials()
            pending["next_username"] = new_credentials["username"]
            config["derived_rebuild_pending"] = pending
            maintenance_phase(state, config, "accounts")
            rotate_database_accounts(state, config, credentials, new_credentials)
            credentials = new_credentials
            current_marker(state, config, credentials)
            maintenance_phase(state, config, "validated")
        elif kind == "current-restore" and phase == "accounts":
            current_marker(state, config, credentials)
            maintenance_phase(state, config, "validated")
        else:
            current_marker(state, config, credentials)
        finish_maintenance(state, config, credentials)
        return public_status(state, config)


def validated_restore_backup(
    source: Path, interpretation: str
) -> tuple[dict[str, object], dict[str, object]]:
    if any(path.is_symlink() for path in (source, *source.parents)) or any(
        path.is_symlink() for path in source.rglob("*")
    ):
        raise SupervisorError("Backup must not traverse symlinks")
    manifest = read_json(source / "backup.json")
    if (
        manifest.get("owner") != OWNER
        or manifest.get("backup_version") != 2
        or manifest.get("interpretation") != interpretation
    ):
        raise SupervisorError("Unknown backup or incompatible interpretation")
    files = manifest.get("files")
    if not isinstance(files, dict):
        raise SupervisorError("Invalid backup file inventory")
    original = manifest.get("state_root")
    if not isinstance(original, str) or not Path(original).is_absolute():
        raise SupervisorError("Missing backup source-state identity")
    actual = {
        str(p.relative_to(source))
        for p in source.rglob("*")
        if p.is_file() and p != source / "backup.json"
    }
    if actual != set(files):
        raise SupervisorError("Backup file inventory differs from its manifest")
    for name, digest in files.items():
        path = source / name
        if (
            Path(name).is_absolute()
            or ".." in Path(name).parts
            or path.is_symlink()
            or file_digest(path) != digest
        ):
            raise SupervisorError("Backup path or content failed validation")
    config = config_for(source)
    if config.get("derived_rebuild_pending"):
        raise SupervisorError(
            "Interrupted maintenance must be recovered by its original owner"
        )
    if (
        config["interpretation"] != interpretation
        or config["schema_interpretation"] != interpretation
    ):
        raise SupervisorError("Backup metadata has an incompatible interpretation")
    if service_directory(source) != source or managed_contexts(source) != [source]:
        raise SupervisorError("Restore refuses shared service contexts")
    if interpretation != SUBSTRATE_INTERPRETATION:
        raise SupervisorError(
            "Unsupported restore requires preserved authored inputs and explicit rebuild"
        )
    return manifest, config


def reroot_restored(value: object, original: str, state: Path) -> object:
    if isinstance(value, dict):
        return {
            key: reroot_restored(item, original, state) for key, item in value.items()
        }
    if isinstance(value, list):
        return [reroot_restored(item, original, state) for item in value]
    if isinstance(value, str) and (
        value == original or value.startswith(original + "/")
    ):
        return str(state) + value[len(original) :]
    return value


def metadata_write_scratch(path: Path, state: Path) -> bool:
    """Identify private atomic-write residue; never consume or delete its bytes."""
    if (
        path.parent != state
        or re.fullmatch(
            r"\.(config\.json|lifecycle-owner\.json)-[a-z0-9_]{8}", path.name
        )
        is None
        or path.is_symlink()
    ):
        return False
    observed = path.stat()
    return (
        stat.S_ISREG(observed.st_mode)
        and observed.st_uid == os.getuid()
        and stat.S_IMODE(observed.st_mode) == 0o600
        and observed.st_nlink == 1
    )


def complete_backup_copy(state: Path, config: dict[str, object]) -> None:
    """Resume only exact declared backup files, under the still-closed owner."""
    pending = object_mapping(config["derived_rebuild_pending"])
    source = Path(str(pending.get("backup", "")))
    if (
        not source.is_absolute()
        or source == state
        or source in state.parents
        or state in source.parents
    ):
        raise SupervisorError("Invalid preserved backup location")
    if file_digest(source / "backup.json") != pending.get("backup_digest"):
        raise SupervisorError(
            "Preserved backup manifest changed; admission remains closed"
        )
    manifest, original_config = validated_restore_backup(
        source, SUBSTRATE_INTERPRETATION
    )
    if (original_config["namespace"], original_config["database"]) != (
        pending["namespace"],
        pending["database"],
    ):
        raise SupervisorError("Preserved backup source identity changed")
    if (config["namespace"], config["database"]) != (
        pending["target_namespace"],
        pending["target_database"],
    ):
        raise SupervisorError("Preserved backup target identity changed")
    if active(state):
        raise SupervisorError(
            "Interrupted backup copy requires the owned server stopped"
        )
    all_contexts_drained(state)
    files = object_mapping(manifest["files"])
    roots = {"database", "credentials.json", ".generations", ".contexts", ".receivers"}
    selected = {
        name: digest for name, digest in files.items() if Path(name).parts[0] in roots
    }
    if "credentials.json" not in selected or not (source / "database").is_dir():
        raise SupervisorError("Backup is missing owned state")
    permitted = set(selected) | {
        "config.json",
        ".supervisor.lock",
        "lifecycle-owner.json",
    }
    for path in state.rglob("*"):
        metadata_scratch = metadata_write_scratch(path, state)
        # Never parse, adopt or delete interrupted atomic-write scratch. Its
        # known basename is disjoint from every copied file and authority.
        if path.is_symlink() or (
            path.is_file()
            and str(path.relative_to(state)) not in permitted
            and not metadata_scratch
        ):
            raise SupervisorError(
                "Unknown interrupted backup content retained; admission remains closed"
            )
    # Known partial file copies can be overwritten from the immutable verified
    # backup. Never import its old, normally startable config.json.
    for directory in sorted(source.rglob("*")):
        if directory.is_dir() and directory.relative_to(source).parts[0] in roots:
            (state / directory.relative_to(source)).mkdir(
                mode=0o700, parents=True, exist_ok=True
            )
    for name, digest in sorted(selected.items()):
        target = state / name
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        shutil.copy2(source / name, target)
        with target.open("rb") as stream:
            os.fsync(stream.fileno())
        if file_digest(target) != digest:
            raise SupervisorError(
                "Restored backup copy differs; admission remains closed"
            )
    validated_restore_backup(source, SUBSTRATE_INTERPRETATION)
    if file_digest(source / "backup.json") != pending["backup_digest"]:
        raise SupervisorError("Preserved backup changed during copy")
    original = str(manifest["state_root"])
    for descriptor in (state / ".contexts").glob("*.json"):
        selected_context = reroot_restored(read_json(descriptor), original, state)
        if not isinstance(selected_context, dict):
            raise SupervisorError("Invalid restored context descriptor")
        write_json(descriptor, selected_context)
    for receiver in (state / ".receivers").glob("*/*/config.json"):
        selected_receiver = reroot_restored(read_json(receiver), original, state)
        if not isinstance(selected_receiver, dict):
            raise SupervisorError("Invalid restored receiver profile")
        selected_receiver.update(
            instance_id=config["instance_id"],
            admission="validation_required",
            accepting_writes=False,
            unit_materialized=False,
            restart_qualified=False,
        )
        write_json(receiver, selected_receiver)
    for generation in (
        (state / ".generations").iterdir() if (state / ".generations").exists() else []
    ):
        verify_generation(generation)
    owned_generations(state)
    (state / "tmp").mkdir(mode=0o700, exist_ok=True)
    # Flush copied directory entries before advancing the durable owner phase.
    for directory in [state, *(path for path in state.rglob("*") if path.is_dir())]:
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    maintenance_phase(state, config, "cleanup")


def restore(source: Path, state: Path, interpretation: str) -> dict[str, object]:
    manifest, original = validated_restore_backup(source, interpretation)
    state = checked_directory(state)
    with lifecycle_reservation(state):
        if any(
            path.name not in {".supervisor.lock", "lifecycle-owner.json"}
            and not metadata_write_scratch(path, state)
            for path in state.iterdir()
        ):
            raise SupervisorError(
                "Refusing to initialize or replace a nonempty directory"
            )
        selected = reroot_restored(
            fresh_database_identity(original), str(manifest["state_root"]), state
        )
        if not isinstance(selected, dict):
            raise SupervisorError("Invalid restored profile")
        selected["credentials_file"] = str(state / "credentials.json")
        credentials = read_json(source / "credentials.json")
        selected["derived_rebuild_pending"] = {
            "kind": "current-restore",
            "phase": "backup-copy",
            "namespace": original["namespace"],
            "database": original["database"],
            "target_namespace": selected["namespace"],
            "target_database": selected["database"],
            "previous_username": credentials["username"],
            "next_username": fresh_credentials()["username"],
            "backup": str(source.absolute()),
            "backup_digest": file_digest(source / "backup.json"),
        }
        # Publish the fresh, closed identity before any bytes of the old database or
        # credentials. Every interruption is recoverable only through this owner.
        write_json(state / "config.json", selected)
        return recover_maintenance(state, [])


def validate(
    state: Path, config: dict[str, object], interpretation: str, command: list[str]
) -> dict[str, object]:
    if (
        config["admission"] != "validation_required"
        or config.get("derived_rebuild_pending")
        or config["interpretation"] != interpretation
        or not command
    ):
        raise SupervisorError(
            "Validation requires a gated restore, matching interpretation and a check command"
        )
    start(state, config, validation=True)
    environment = os.environ.copy()
    environment["PSE_SURREAL_STATE"] = str(state)
    # The validator owns exact values/schema/operation identities; no check is inferred from /health.
    try:
        validate_interpretation(state, config, interpretation)
        result = subprocess.run(
            command,
            env=environment,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=120,
            check=False,
        )
        if result.returncode:
            raise SupervisorError(
                "Restored database semantic validation failed; admission remains closed"
            )
    finally:
        stop(state, config)
    config["admission"] = "quiesced"
    write_json(state / "config.json", config)
    return public_status(state, config)


def validate_interpretation(
    state: Path, config: dict[str, object], interpretation: str
) -> None:
    """Authenticate and verify the restored canonical reader interpretation."""
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    request = urllib.request.Request(
        f"http://127.0.0.1:{config['port']}/sql",
        data=b"SELECT VALUE interpretation FROM canonical_interpretations:current;",
        headers={
            "Authorization": f"Basic {token}",
            "Accept": "application/json",
            "surreal-ns": str(config["namespace"]),
            "surreal-db": str(config["database"]),
        },
    )
    with urllib.request.urlopen(request, timeout=10) as response:  # noqa: S310 -- Literal authenticated loopback HTTP endpoint from owned configuration.
        result = json.load(response)
    if (
        not isinstance(result, list)
        or len(result) != 1
        or not isinstance(result[0], dict)
        or result[0].get("status") != "OK"
        or result[0].get("result") != [interpretation]
    ):
        raise SupervisorError(
            "Canonical database interpretation was not validated; writes remain closed"
        )


def recovery_generation(state: Path, config: dict[str, object]) -> str:
    """Bind recovery evidence to the exact owned storage and service closure."""
    supervisor = config.get("service_supervisor")
    if not isinstance(supervisor, dict) or not supervisor.get("supervisor_script"):
        raise SupervisorError(
            "Recovery qualification requires an immutable service supervisor"
        )
    directory = Path(str(supervisor["supervisor_script"])).parents[1]
    if directory.parent != state / ".generations":
        raise SupervisorError(
            "Recovery service closure is outside its owned generation"
        )
    verify_generation(directory)
    binary = config.get("server")
    if not isinstance(binary, dict) or file_digest(
        Path(str(binary["binary"]))
    ) != binary.get("binary_sha256"):
        raise SupervisorError("Recovery qualification storage artifact changed")
    selected = {
        key: config.get(key)
        for key in (
            "instance_id",
            "server",
            "service_supervisor",
            "resources",
            "namespace",
            "database",
            "endpoint",
            "schema_interpretation",
            "websocket_max_message_bytes",
        )
    }
    selected["credentials_sha256"] = file_digest(state / "credentials.json")
    return hashlib.sha256(
        json.dumps(selected, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def recovery_qualified(state: Path, config: dict[str, object]) -> bool:
    proof = state / "recovery-qualification.json"
    if not config.get("restart_qualified") or not proof.is_file():
        return False
    result = read_json(proof)
    return (
        result.get("version") == 1
        and result.get("outcome") == "passed"
        and result.get("generation") == recovery_generation(state, config)
    )


def administrative_headers(
    config: dict[str, object],
    token: str,
    namespace: str | None,
    database: str | None,
    *,
    select_context: bool = True,
) -> dict[str, str]:
    """Select one validated administrative context or explicitly remain unselected."""
    if (namespace is None) != (database is None):
        raise SupervisorError(
            "Administrative context overrides require both identifiers"
        )
    selected = (
        (config["namespace"], config["database"])
        if namespace is None
        else (namespace, database)
    )
    if any(
        not isinstance(value, str)
        or not 1 <= len(value) <= 128
        or not all(
            character.isascii() and (character.isalnum() or character == "_")
            for character in value
        )
        for value in selected
    ):
        raise SupervisorError(
            "Administrative context requires bounded ASCII identifiers"
        )
    headers = {"Authorization": f"Basic {token}", "Accept": "application/json"}
    if select_context:
        headers.update({"surreal-ns": str(selected[0]), "surreal-db": str(selected[1])})
    return headers


def administrative_query(
    state: Path,
    query: str,
    deadline: float,
    *,
    namespace: str | None = None,
    database: str | None = None,
    select_context: bool = True,
) -> list[dict[str, object]]:
    """Bounded authenticated administrative control; scientific RPC remains native WS."""
    config = config_for(state)
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    request = urllib.request.Request(
        f"http://127.0.0.1:{config['port']}/sql",
        data=query.encode(),
        headers=administrative_headers(
            config, token, namespace, database, select_context=select_context
        ),
    )
    with urllib.request.urlopen(request, timeout=remaining(deadline)) as response:  # noqa: S310 -- literal authenticated loopback endpoint; config_for validates port
        payload = response.read(MESSAGE_BYTES + 1)
    if len(payload) > MESSAGE_BYTES:
        raise SupervisorError("Administrative response exceeded its bounded extent")
    results = json.loads(payload)
    if (
        not isinstance(results, list)
        or not results
        or any(
            not isinstance(row, dict) or row.get("status") != "OK" for row in results
        )
    ):
        raise SupervisorError(
            "Administrative query contained a failed or incomplete statement"
        )
    remaining(deadline)
    return results


def abandon_administrative_response(
    state: Path,
    query: str,
    deadline: float,
    *,
    namespace: str | None = None,
    database: str | None = None,
) -> None:
    """Submit the complete small HTTP probe and abandon its unknown acknowledgment."""
    config = config_for(state)
    credentials = read_json(state / "credentials.json")
    token = base64.b64encode(
        f"{credentials['username']}:{credentials['password']}".encode()
    ).decode()
    headers = administrative_headers(config, token, namespace, database)
    connection = http.client.HTTPConnection(
        "127.0.0.1", integer(config["port"]), timeout=remaining(deadline)
    )
    try:
        connection.request(
            "POST",
            "/sql",
            body=query.encode(),
            headers=headers,
        )
        # No getresponse(): complete request transmission does not establish commit.
    finally:
        connection.close()


def qualify_recovery(state: Path) -> dict[str, object]:
    """Destructive, explicitly selected administrative recovery qualification."""
    deadline = time.monotonic() + 90
    prior = getattr(_STARTUP, "deadline", None)
    _STARTUP.deadline = deadline if prior is None else min(prior, deadline)
    deadline = _STARTUP.deadline
    try:
        remaining(deadline)
        with lifecycle_reservation(state):
            close_context_admission(state)
            all_contexts_drained(state)
            config = config_for(state)
            config["restart_qualified"] = False
            write_json(state / "config.json", config)
            generation = recovery_generation(state, config)
            operation = uuid.uuid4().hex
            unknown_operation = uuid.uuid4().hex
            namespace = "pse_recovery"
            database = "recovery_" + operation
            controls = checked_directory(state / ".recovery-controls")
            context_directory = controls / operation
            context_directory.mkdir(mode=0o700)
            context = {
                "owner": "supervisor-recovery-v1",
                "generation": generation,
                "namespace": namespace,
                "database": database,
                "operation": operation,
                "unknown_operation": unknown_operation,
                "outcome": "incomplete",
            }
            write_json(context_directory / "context.json", context)
            key = "supervisor_recovery_probe:" + operation
            unknown_key = "supervisor_recovery_probe:" + unknown_operation
            acknowledged = {"operation": operation, "payload": "acknowledged-v1"}
            unknown = {"operation": unknown_operation, "payload": "unknown-v1"}
            try:
                stop(state, config)
                reset_failure_window(state)
                start(state, config, validation=True, deadline=deadline)
                provisioned = administrative_query(
                    state,
                    f"DEFINE NAMESPACE IF NOT EXISTS `{namespace}`; USE NS `{namespace}`; DEFINE DATABASE `{database}`;",
                    deadline,
                    namespace=namespace,
                    database=database,
                    select_context=False,
                )
                # Released 3.3 USE returns the selected context; DEFINE returns NONE.
                if (
                    len(provisioned) != 3
                    or any(
                        row.get("status") != "OK" or "result" not in row
                        for row in provisioned
                    )
                    or [row["result"] for row in provisioned]
                    != [None, {"namespace": namespace, "database": None}, None]
                ):
                    raise SupervisorError(
                        "Recovery context provisioning lacked complete acknowledgments"
                    )
                results = administrative_query(
                    state,
                    f"UPSERT {key} CONTENT {json.dumps(acknowledged)}; SELECT operation,payload FROM ONLY {key};",  # noqa: S608 -- local UUID records and JSON-encoded payload
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                if len(results) != 2 or results[1].get("result") != acknowledged:
                    raise SupervisorError(
                        "Recovery acknowledged receipt did not match its original identity"
                    )
                abandon_administrative_response(
                    state,
                    f"UPSERT {unknown_key} CONTENT {json.dumps(unknown)};",
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                stop(state, config, abrupt=True)
                reset_failure_window(state)
                start(state, config, validation=True, deadline=deadline)
                readback = administrative_query(
                    state,
                    f"SELECT operation,payload FROM ONLY {key}; SELECT operation,payload FROM ONLY {unknown_key};",  # noqa: S608 -- identifiers are local generated UUID records
                    deadline,
                    namespace=namespace,
                    database=database,
                )
                if (
                    len(readback) != 2
                    or readback[0].get("result") != acknowledged
                    or readback[1].get("result") not in (None, unknown)
                ):
                    raise SupervisorError(
                        "Recovery lost the acknowledged original operation or returned conflicting bytes"
                    )
                # Old process-group drain plus reopen bounds settlement of this
                # exact probe. No absent-ack replay or scientific claim occurs.
                recovered = readback[1]["result"]
            finally:
                cleanup_prior = _STARTUP.deadline
                _STARTUP.deadline = time.monotonic() + 90
                try:
                    stop(state, config)
                finally:
                    _STARTUP.deadline = cleanup_prior
            remaining(deadline)
            if recovery_generation(state, config_for(state)) != generation:
                raise SupervisorError(
                    "Service generation changed during recovery qualification"
                )
            write_json(
                state / "recovery-qualification.json",
                {
                    "version": 1,
                    "outcome": "passed",
                    "generation": generation,
                    "operation": operation,
                    "unknown_operation": unknown_operation,
                    "administrative_context": {
                        "namespace": namespace,
                        "database": database,
                        "evidence": str(context_directory / "context.json"),
                    },
                    "acknowledged_survived": True,
                    "unknown_outcome": "committed"
                    if recovered == unknown
                    else "not_committed",
                    "scope": "process-crash/reopen; administrative fixed probe; no scientific or host-crash qualification",
                },
            )
            context["outcome"] = "passed"
            write_json(context_directory / "context.json", context)
            config.update(config_for(state))
            config["restart_qualified"] = True
            write_json(state / "config.json", config)
            materialize_service(state, config)
            reset_failure_window(state)
            return public_status(state, config)
    finally:
        _STARTUP.deadline = prior


def mcp_private_record(path: Path, deadline: float) -> tuple[dict[str, object], str]:
    """Read an existing private record without following links or repairing modes."""
    remaining(deadline)
    descriptor = os.open(
        path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    )
    with os.fdopen(descriptor, "rb") as stream:
        information = os.fstat(stream.fileno())
        if (
            not stat.S_ISREG(information.st_mode)
            or information.st_uid != os.getuid()
            or stat.S_IMODE(information.st_mode) != 0o600
            or information.st_size > 64 * 1024
        ):
            raise SupervisorError("MCP private record is unavailable")
        raw = stream.read(64 * 1024 + 1)
    remaining(deadline)
    if len(raw) > 64 * 1024:
        raise SupervisorError("MCP private record exceeds its bound")
    value = object_mapping(json.loads(raw))
    return value, hashlib.sha256(raw).hexdigest()


def mcp_authorization(state: Path, endpoint: str, deadline: float) -> dict[str, str]:
    """Return only the existing VIEWER header after observing the exact live owner."""
    remaining(deadline)
    state = lexical_absolute(state)
    if any(path.is_symlink() for path in (state, *state.parents)):
        raise SupervisorError("MCP state must not traverse links")
    information = state.stat()
    if (
        not stat.S_ISDIR(information.st_mode)
        or information.st_uid != os.getuid()
        or stat.S_IMODE(information.st_mode) != 0o700
    ):
        raise SupervisorError("MCP state must already be private and owned")
    config = config_for(state, deadline=deadline)
    if (
        endpoint != f"http://127.0.0.1:{integer(config['port'])}/mcp"
        or config.get("service_state") is not None
        or config.get("credentials_file") != str(state / "credentials.json")
        or not owns_listener(state, config, deadline=deadline)
    ):
        raise SupervisorError("MCP endpoint does not match its live owned service")
    credentials, credential_digest = mcp_private_record(
        state / "credentials.json", deadline
    )
    proof, _ = mcp_private_record(state / "protocol-readiness.json", deadline)
    observed = storage_unit_observation(state, deadline=deadline)
    if (
        proof.get("schema") != "native-ws-readiness-v1"
        or proof.get("instance_id") != config["instance_id"]
        or proof.get("invocation") != observed.get("InvocationID")
        or proof.get("binary_sha256")
        != object_mapping(config["server"]).get("binary_sha256")
        or proof.get("credentials_sha256") != credential_digest
        or credentials.get("selection_username") != "pse-selection"
        or not isinstance(credentials.get("selection_password"), str)
        or not credentials["selection_password"]
    ):
        raise SupervisorError("MCP VIEWER readiness identity is unavailable")
    # Recheck live ownership and credential/config identity before releasing the header.
    if (
        config_for(state, deadline=deadline) != config
        or mcp_private_record(state / "credentials.json", deadline)[1]
        != credential_digest
        or not owns_listener(state, config, deadline=deadline)
    ):
        raise SupervisorError("MCP service changed while reading credentials")
    remaining(deadline)
    token = base64.b64encode(
        f"pse-selection:{credentials['selection_password']}".encode()
    ).decode()
    return {"Authorization": "Basic " + token}


def mcp_headers_main(arguments: list[str]) -> int:
    """Fixed, admitted control path with one eight-second clock and redacted errors."""
    deadline = time.monotonic() + 8
    child = arguments[0] == "_mcp-headers-child"
    command = argparse.ArgumentParser(prog="surreal-server mcp-headers")
    command.add_argument("--state", type=Path, required=True)
    command.add_argument("--expected-endpoint", required=True)
    if child:
        command.add_argument("--deadline", type=float, required=True)
    args = command.parse_args(arguments[1:])
    if not re.fullmatch(r"http://127\.0\.0\.1:[0-9]{1,5}/mcp", args.expected_endpoint):
        command.error(
            "--expected-endpoint must be an explicit native loopback /mcp endpoint"
        )
    try:
        from scripts import pse_env  # noqa: PLC0415 -- fixed control environment owner

        if child:
            if not math.isfinite(args.deadline) or args.deadline > deadline:
                raise SupervisorError("Invalid fixed helper clock")  # noqa: TRY301 -- fixed boundary redacts every failure
            deadline = args.deadline
            remaining(deadline)
            try:
                owner = host_admission.inherit(os.environ, deadline=deadline)
            except host_admission.AdmissionError:
                owner = host_admission.verify_light_control_child(
                    os.environ, deadline=deadline
                )
            if owner is None:
                raise SupervisorError("Missing admitted fixed helper ownership")  # noqa: TRY301 -- fixed boundary redacts every failure
            headers = mcp_authorization(args.state, args.expected_endpoint, deadline)
        else:
            environment = pse_env.control_environment(
                SCRIPT.parents[1], os.environ, deadline=deadline
            )
            result = pse_env.run_light_control(
                [
                    sys.executable,
                    str(SCRIPT),
                    "_mcp-headers-child",
                    "--state",
                    str(args.state),
                    "--expected-endpoint",
                    args.expected_endpoint,
                    "--deadline",
                    str(deadline),
                ],
                environment,
                deadline=deadline,
            )
            remaining(deadline)
            if result.returncode:
                raise SupervisorError("Fixed helper child failed")  # noqa: TRY301 -- fixed boundary redacts every failure
            headers = json.loads(result.stdout)
            if (
                not isinstance(headers, dict)
                or set(headers) != {"Authorization"}
                or not isinstance(headers["Authorization"], str)
                or not headers["Authorization"].startswith("Basic ")
                or not base64.b64decode(headers["Authorization"][6:], validate=True)
                .decode()
                .startswith("pse-selection:")
            ):
                raise SupervisorError("Invalid fixed helper output")  # noqa: TRY301 -- fixed boundary redacts every failure
    except Exception:
        # Child output, HTTP errors and JSON parse errors can contain credentials.
        # Failure must never echo any of them or return partial headers.
        print(
            "pse-env: MCP credentials unavailable; verify the existing owned service and private VIEWER readiness",
            file=sys.stderr,
        )
        return 125
    print(json.dumps(headers))
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument(
        "command",
        choices=[
            "setup",
            "upgrade",
            "readmit",
            "ensure",
            "recover",
            "qualify-recovery",
            "drain",
            "reconfigure",
            "start",
            "status",
            "quiesce",
            "stop",
            "kill",
            "backup",
            "restore",
            "rebuild",
            "recover-maintenance",
            "validate",
            "worker",
            "ensure-primary",
            "observer",
            "_serve",
        ],
    )
    result.add_argument(
        "--state",
        type=Path,
        default=Path(
            os.environ.get(
                "PSE_SURREAL_STATE",
                str(
                    Path(
                        os.environ.get(
                            "XDG_STATE_HOME", str(Path.home() / ".local/state")
                        )
                    )
                    / "pse-arrow/surreal-functional-v2"
                ),
            )
        ),
    )
    result.add_argument(
        "--tool-root",
        type=Path,
    )
    result.add_argument(
        "--version", help="Official release vX.Y.Z; omitted selects current stable"
    )
    result.add_argument("--port", type=int)
    result.add_argument(
        "--resident",
        action=argparse.BooleanOptionalAction,
        default=None,
        help="Resident services may be parked and resumed across host lanes; disable for owned disposable controls",
    )
    result.add_argument("--memory-mib", type=int)
    result.add_argument("--server-memory-mib", type=int)
    result.add_argument("--native-workers", type=int)
    result.add_argument("--native-worker-memory-mib", type=int)
    result.add_argument(
        "--execution-profile",
        choices=[
            "plan28-reference",
            "functional",
            "wide",
            "timing",
            "exclusive",
            "exclusive-observer",
        ],
    )
    result.add_argument("--worker-executable", type=Path)
    result.add_argument("--observer-pid", type=int)
    result.add_argument("--canonical-database")
    result.add_argument(
        "--qualification-native-entry",
        type=Path,
        help="Explicit private native-entry control for a qualification-feature worker",
    )
    result.add_argument("--interpretation")
    result.add_argument(
        "--drained",
        action="store_true",
        help="Caller has quiesced admission and drained managed native workers",
    )
    result.add_argument("--destination", type=Path)
    result.add_argument("--source", type=Path)
    result.add_argument("--initializer-command", nargs=argparse.REMAINDER)
    result.add_argument("--check-command", nargs=argparse.REMAINDER)
    result.add_argument("--worker-command", nargs=argparse.REMAINDER)
    result.add_argument("--worker-capabilities", default=",".join(WORKER_CAPABILITIES))
    result.add_argument("--observer-command", nargs=argparse.REMAINDER)
    return result


def dispatch(args: argparse.Namespace) -> int:
    if args.qualification_native_entry is not None and args.command != "ensure-primary":
        raise SupervisorError(
            "Qualification control is only available with ensure-primary"
        )
    if args.command == "upgrade":
        output = upgrade_profile(args.state.absolute())
    elif args.command == "setup":
        output = setup(args)
    elif args.command == "rebuild":
        if args.source is None or args.destination is None:
            raise SupervisorError(
                "rebuild requires --source authored-inputs and --destination preserved-inputs"
            )
        output = rebuild(
            args.state.absolute(),
            args.source.absolute(),
            args.destination.absolute(),
            args.initializer_command or [],
        )
    elif args.command == "recover-maintenance":
        output = recover_maintenance(
            args.state.absolute(), args.initializer_command or []
        )
    elif args.command == "restore":
        if args.source is None:
            raise SupervisorError("restore requires --source")
        output = restore(
            args.source.absolute(), args.state.absolute(), args.interpretation or ""
        )
    elif args.command == "worker":
        return worker(
            args.state.absolute(),
            args.worker_command or [],
            capabilities=tuple(
                value for value in args.worker_capabilities.split(",") if value
            ),
        )
    elif args.command == "observer":
        profile = (
            "reference"
            if args.execution_profile == "plan28-reference"
            else args.execution_profile
        )
        return observer(
            args.state.absolute(), args.observer_command or [], profile=profile
        )
    elif args.command == "ensure-primary":
        output = ensure_primary(
            args.state.absolute(),
            args.observer_pid,
            args.canonical_database,
            args.qualification_native_entry,
        )
    elif args.command == "_serve":
        return serve(args.state.absolute())
    else:
        state = args.state.absolute()
        config_for(
            state
        )  # Reject unowned state before changing permissions or writing a lock.
        state = checked_directory(state)
        with (
            contextlib.nullcontext()
            if args.command == "status"
            else lifecycle_reservation(state)
        ):
            config = config_for(state)
            if args.command == "readmit":
                output = readmit_supervisor(state)
                print(json.dumps(output, sort_keys=True))
                return 0
            if args.command == "qualify-recovery":
                output = qualify_recovery(state)
                print(json.dumps(output, sort_keys=True))
                return 0
            if args.command in {"start", "ensure", "recover"}:
                if args.command == "recover":
                    reset_failure_window(state)
                start(state, config)
            elif args.command == "reconfigure":
                reconfigure(state, config, args)
            elif args.command in {"quiesce", "drain"}:
                close_context_admission(state, admission="quiescing")
                config.update(config_for(state))
                for context in managed_contexts(state):
                    selected = config_for(context)
                    allocation = selected["resources"]
                    if (
                        not isinstance(allocation, dict)
                        or "execution" not in allocation
                    ):
                        continue
                    observed = primary_observation(context)
                    pid = primary_drain_pid(context, selected, observed)
                    if pid is not None:
                        # Signal the actual receiver only. The installation wrapper's
                        # cancellation path kills its scope, which is recovery, not
                        # cooperative native drain.
                        descriptor = os.pidfd_open(pid)
                        try:
                            if (
                                primary_drain_pid(
                                    context, selected, primary_observation(context)
                                )
                                == pid
                            ):
                                signal.pidfd_send_signal(descriptor, signal.SIGINT)
                        finally:
                            os.close(descriptor)
            elif args.command in {"stop", "kill", "backup"}:
                if args.command != "kill" and not args.drained:
                    raise SupervisorError(
                        "Quiesce admission and drain managed workers, then supply --drained"
                    )
                if args.command == "backup":
                    if args.destination is None:
                        raise SupervisorError("backup requires --destination")
                    output = backup(state, config, args.destination)
                    print(json.dumps(output, sort_keys=True))
                    return 0
                # An operator stop/kill withdraws coordinated resume intent. The
                # internal parking path still calls stop directly and retains it.
                # Clear the service premise before the host queue so a concurrent
                # reconciler's old snapshot cannot resurrect this explicit stop.
                service = service_directory(state)
                with state_lock(service):
                    selected = config_for(service)
                    selected["parked"] = False
                    write_json(service / "config.json", selected)
                host_admission.withdraw_parked_service(service)
                config.update(config_for(state))
                stop(state, config, abrupt=args.command == "kill")
            elif args.command == "validate":
                output = validate(
                    state, config, args.interpretation or "", args.check_command or []
                )
                print(json.dumps(output, sort_keys=True))
                return 0
            output = public_status(state, config)
    print(json.dumps(output, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    # Fixed credential retrieval must never reach general lifecycle dispatch.
    if arguments and arguments[0] in {"mcp-headers", "_mcp-headers-child"}:
        return mcp_headers_main(arguments)
    args = parser().parse_args(arguments)
    try:
        return dispatch(args)
    except (
        SupervisorError,
        OSError,
        ValueError,
        KeyError,
        subprocess.SubprocessError,
    ) as error:
        # Raw subprocess errors may contain secret-bearing arguments from a validator.
        message = (
            str(error)
            if isinstance(error, SupervisorError)
            else f"{type(error).__name__}: supervisor operation failed"
        )
        print(message, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

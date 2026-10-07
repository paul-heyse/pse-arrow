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
import fcntl
import hashlib
import json
import os
import platform
import secrets
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Generator


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


class SupervisorError(RuntimeError):
    """An actionable lifecycle error with no credential-bearing output."""


def read_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SupervisorError(f"Expected an object in {path.name}")
    return value


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise SupervisorError("Expected an integer in owned configuration")
    return value


def write_json(path: Path, value: dict[str, object]) -> None:
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


def config_for(state: Path) -> dict[str, object]:
    if (
        any(path.is_symlink() for path in (state, *state.parents))
        or (state / "config.json").is_symlink()
    ):
        raise SupervisorError("Owned state must not traverse symlinks")
    config = read_json(state / "config.json")
    if config.get("owner") != OWNER or config.get("profile_version") != 1:
        raise SupervisorError("Not an owned, supported SurrealDB state directory")
    if config.get("grpc_max_message_bytes") != MESSAGE_BYTES:
        raise SupervisorError("Unsupported gRPC message profile")
    budget = config.get("resources")
    if not isinstance(budget, dict):
        raise SupervisorError("Missing resource allocation")
    expected = resources(
        integer(budget["total_memory_bytes"]),
        integer(budget["server_memory_bytes"]),
        integer(budget["native_workers"]),
        integer(budget["native_worker_memory_bytes"]),
    )
    if budget != expected:
        raise SupervisorError(
            "Resource overrides do not match the allocated server budget"
        )
    port = integer(config["port"])
    if not 1 <= port <= 65535 or config["endpoint"] != f"grpc://127.0.0.1:{port}":
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


def resources(total: int, server: int, workers: int, worker: int) -> dict[str, int]:
    if (
        server < 512 * MIB
        or not server < total < 2**63
        or not 1 <= workers <= 32
        or worker < 64 * MIB
    ):
        raise SupervisorError(
            "Require server >=512MiB, 1..32 workers and positive finite budgets"
        )
    if server + workers * worker > total:
        raise SupervisorError(
            "Server and managed native workers exceed the shared memory budget"
        )
    return {
        "total_memory_bytes": total,
        "server_memory_bytes": server,
        "native_workers": workers,
        "native_worker_memory_bytes": worker,
        "rocksdb_block_cache_bytes": server // 4,
        "rocksdb_write_buffer_bytes": min(32 * MIB, server // 64),
        "rocksdb_write_buffers": 2,
        "memory_threshold_bytes": server * 3 // 4,
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
        "archive_sha256": digest.removeprefix("sha256:"),
    }


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def setup(args: argparse.Namespace) -> dict[str, object]:
    if (args.state / "config.json").exists():
        config = config_for(args.state.absolute())
        if config["interpretation"] != args.interpretation:
            raise SupervisorError("Existing state has a different interpretation")
        return public_status(args.state.absolute(), config)
    # Download first so network failure leaves no half-initialized application state.
    allocation = resources(
        (4096 if args.memory_mib is None else args.memory_mib) * MIB,
        (2048 if args.server_memory_mib is None else args.server_memory_mib) * MIB,
        2 if args.native_workers is None else args.native_workers,
        (
            1024
            if args.native_worker_memory_mib is None
            else args.native_worker_memory_mib
        )
        * MIB,
    )
    port = 18080 if args.port is None else args.port
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
    server = install(args.version, tool_root)
    state = checked_directory(args.state, empty=True)
    with state_lock(state):
        if any(path.name != ".supervisor.lock" for path in state.iterdir()):
            raise SupervisorError(
                "State was initialized concurrently; refusing to replace it"
            )
        config: dict[str, object] = {
            "owner": OWNER,
            "profile_version": 1,
            "instance_id": str(uuid.uuid4()),
            "server": server,
            "port": port,
            "namespace": "pse",
            "database": "canonical",
            "endpoint": f"grpc://127.0.0.1:{port}",
            "credentials_file": str(state / "credentials.json"),
            "schema_interpretation": args.interpretation,
            "accepting_writes": True,
            "interpretation": args.interpretation,
            "admission": "open",
            "max_message_bytes": MESSAGE_BYTES,
            "grpc_max_message_bytes": MESSAGE_BYTES,
            "resources": allocation,
            "log_max_bytes": 8 * MIB,
            "log_backups": 2,
        }
        write_json(
            state / "credentials.json",
            {"username": "pse-local", "password": secrets.token_urlsafe(36)},
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
    updated = dict(config)
    updated["resources"] = allocation
    write_json(state / "config.json", updated)
    config.update(updated)


def systemd_environment() -> dict[str, str]:
    environment = os.environ.copy()
    runtime = Path(f"/run/user/{os.getuid()}")
    if (runtime / "bus").is_socket():
        environment.setdefault("XDG_RUNTIME_DIR", str(runtime))
        environment.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    return environment


def systemctl(*arguments: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["systemctl", "--user", *arguments],
        env=systemd_environment(),
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if check and result.returncode:
        raise SupervisorError(
            "User systemd operation failed; a user manager is required for capped supervision"
        )
    return result


def unit_name(state: Path) -> str:
    return f"pse-surreal-{hashlib.sha256(str(lexical_absolute(state)).encode()).hexdigest()[:16]}.service"


def active(state: Path) -> bool:
    return (
        systemctl("is-active", "--quiet", unit_name(state), check=False).returncode == 0
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


def placement_slice() -> list[str]:
    """Workers and servers launch their own units, so they name the agent slice too."""
    from scripts import pse_env  # noqa: PLC0415 -- placement owner

    selected = pse_env.slice_name(os.environ)
    return [] if selected is None else [f"--slice={selected}"]


def worker_scope_command(
    state: Path,
    slot: int,
    allocation: dict[str, object],
    command: list[str],
    *,
    capabilities: tuple[str, ...] = ("solver", "klu", "isolation", "uno", "petsc"),
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
        *placement_slice(),
        f"--property=MemoryMax={integer(allocation['native_worker_memory_bytes'])}",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=128",
        "--",
        str(Path(__file__).resolve().parents[1] / ".venv/bin/python"),
        str(Path(__file__).resolve().with_name("native_operation.py")),
        "--capabilities",
        ",".join(capabilities),
        "--",
        *command,
    ]


def worker(state: Path, command: list[str]) -> int:
    if not command:
        raise SupervisorError(
            "worker requires --worker-command followed by an executable and arguments"
        )
    config_for(state)
    state = checked_directory(state)
    # The lifecycle lock covers admission and slot registration only. Quiesce/stop
    # remain available while the native process runs and drains its current claim.
    with state_lock(state):
        config = config_for(state)
        if config["admission"] != "open" or not config["accepting_writes"]:
            raise SupervisorError("Worker admission is closed")
        systemctl("show-environment")
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise SupervisorError("Invalid resource configuration")
        slot = next(
            (
                n
                for n in range(integer(allocation["native_workers"]))
                if not worker_busy(settled_worker(state, n))
            ),
            None,
        )
        if slot is None:
            raise SupervisorError("All configured native worker slots are occupied")
        if not ready(state, config):
            start(state, config)
        # Pin the launch window before crossing into an independent worker scope.
        # The common child owner binds this guard using actual kernel membership.
        root = str(Path(__file__).resolve().parents[1])
        if root not in sys.path:
            sys.path.insert(0, root)
        from scripts.native_operation import (  # noqa: PLC0415 -- standalone supervisor loads shared owner after root selection
            prepare_handoff,
        )

        handoff = prepare_handoff(worker_unit(state, slot))
        child = subprocess.Popen(
            worker_scope_command(state, slot, allocation, command),
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
    result = child.wait()
    return result if result >= 0 else 128 - result


def public_status(state: Path, config: dict[str, object]) -> dict[str, object]:
    port = integer(config["port"])
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    return {
        "state": str(state),
        "active": active(state),
        "unit": unit_name(state),
        "endpoint": f"127.0.0.1:{port}",
        "grpc_endpoint": f"grpc://127.0.0.1:{port}",
        "credentials_file": str(state / "credentials.json"),
        "config_file": str(state / "config.json"),
        "server": config["server"],
        "admission": config["admission"],
        "interpretation": config["interpretation"],
        "resources": config["resources"],
        "accepting_writes": config["accepting_writes"],
        "grpc_max_message_bytes": MESSAGE_BYTES,
        "worker_units": [
            worker_unit(state, slot)
            for slot in range(integer(allocation["native_workers"]))
        ],
    }


def ready(state: Path, config: dict[str, object]) -> bool:
    if not owns_listener(state, config):
        return False
    try:
        with urllib.request.urlopen(
            f"http://127.0.0.1:{config['port']}/health", timeout=1
        ) as response:
            return response.status == 200 and active(state)
    except (OSError, urllib.error.URLError):
        return False


def owns_listener(state: Path, config: dict[str, object]) -> bool:
    """Linux listener ownership, independent of another server's /health response."""
    try:
        process = read_json(state / "server-process.json")
        if process.get("instance_id") != config["instance_id"]:
            return False
        pid = integer(process["pid"])
        proc = Path("/proc") / str(pid)
        if unit_name(state) not in (proc / "cgroup").read_text():
            return False
        address = f"0100007F:{integer(config['port']):04X}"
        inodes = {
            row.split()[9]
            for row in Path("/proc/net/tcp").read_text().splitlines()[1:]
            if row.split()[1] == address and row.split()[3] == "0A"
        }
        return any(
            str(fd.readlink()) in {f"socket:[{inode}]" for inode in inodes}
            for fd in (proc / "fd").iterdir()
        )
    except (OSError, KeyError, ValueError, SupervisorError):
        return False


def start(state: Path, config: dict[str, object], *, validation: bool = False) -> None:
    if config["admission"] == "validation_required" and not validation:
        raise SupervisorError(
            "Restored database requires validate before normal server start"
        )
    if active(state):
        if not ready(state, config):
            raise SupervisorError("Owned server unit is active but unhealthy")
        return
    systemctl("show-environment")
    allocation = config["resources"]
    if not isinstance(allocation, dict):
        raise SupervisorError("Invalid resource configuration")
    command = [
        "systemd-run",
        "--user",
        "--quiet",
        "--collect",
        f"--unit={unit_name(state)}",
        *placement_slice(),
        "--service-type=exec",
        "--property=KillMode=control-group",
        "--property=TimeoutStopSec=45",
        "--property=Restart=no",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=128",
        f"--property=MemoryMax={allocation['server_memory_bytes']}",
        "--property=StandardOutput=null",
        "--property=StandardError=null",
        str(Path(sys.executable).resolve()),
        str(SCRIPT),
        "_serve",
        "--state",
        str(state),
    ]
    result = subprocess.run(
        command, env=systemd_environment(), capture_output=True, timeout=60, check=False
    )
    if result.returncode:
        raise SupervisorError("Cannot create capped server unit")
    deadline = time.monotonic() + 40
    while time.monotonic() < deadline:
        if ready(state, config):
            if not validation:
                config["admission"] = "open"
                config["accepting_writes"] = True
                write_json(state / "config.json", config)
            return
        if not active(state):
            break
        time.sleep(0.1)
    systemctl("stop", unit_name(state), check=False)
    raise SupervisorError(
        "Server did not become healthy; inspect the private bounded server.log"
    )


def stop(state: Path, config: dict[str, object], *, abrupt: bool = False) -> None:
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
        workers_drained(state, config)
    if active(state):
        if abrupt:
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
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
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
            systemctl("stop", unit_name(state), check=False)
    result = systemctl(
        "show", "--property=ActiveState", "--value", unit_name(state), check=False
    )
    if result.returncode or result.stdout.strip() not in {"inactive", "failed"}:
        raise SupervisorError("Server remains active; database copy is unsafe")
    if group_path is not None:
        deadline = time.monotonic() + 15
        while group_path.exists():
            events = group_path / "cgroup.events"
            if not events.exists() or "populated 1" not in events.read_text():
                break
            if time.monotonic() >= deadline:
                raise SupervisorError(
                    "Server process group remains populated; database copy is unsafe"
                )
            time.sleep(0.05)


def server_environment(
    config: dict[str, object], credentials: dict[str, object]
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
            "SURREAL_GRPC_MAX_MESSAGE_SIZE": str(MESSAGE_BYTES),
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
    config = config_for(state)
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
    database = f"rocksdb://{state / 'database'}?sync=every&versioned=false"
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
    child = subprocess.Popen(
        command,
        env=server_environment(config, credentials),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    write_json(
        state / "server-process.json",
        {"instance_id": config["instance_id"], "pid": child.pid},
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

    reader = threading.Thread(target=capture, daemon=True)
    reader.start()
    result = child.wait()
    reader.join(timeout=5)
    stream.close()
    return result


def backup(
    state: Path, config: dict[str, object], destination: Path
) -> dict[str, object]:
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
    files = {
        str(path.relative_to(destination)): file_digest(path)
        for path in sorted(destination.rglob("*"))
        if path.is_file()
    }
    write_json(
        destination / "backup.json",
        {
            "owner": OWNER,
            "backup_version": 1,
            "interpretation": config["interpretation"],
            "files": files,
        },
    )
    return {
        "backup": str(destination),
        "interpretation": config["interpretation"],
        "server_stopped": True,
    }


def restore(source: Path, state: Path, interpretation: str) -> dict[str, object]:
    if any(path.is_symlink() for path in (source, *source.parents)) or any(
        path.is_symlink() for path in source.rglob("*")
    ):
        raise SupervisorError("Backup must not traverse symlinks")
    manifest = read_json(source / "backup.json")
    if (
        manifest.get("owner") != OWNER
        or manifest.get("backup_version") != 1
        or manifest.get("interpretation") != interpretation
    ):
        raise SupervisorError("Unknown backup or incompatible interpretation")
    files = manifest.get("files")
    if not isinstance(files, dict):
        raise SupervisorError("Invalid backup file inventory")
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
    if (
        config["interpretation"] != interpretation
        or config["schema_interpretation"] != interpretation
    ):
        raise SupervisorError("Backup metadata has an incompatible interpretation")
    state = checked_directory(state, empty=True)
    for name in ("database", "config.json", "credentials.json"):
        path = source / name
        if path.is_dir():
            shutil.copytree(path, state / name)
        else:
            shutil.copy2(path, state / name)
    config["instance_id"] = str(uuid.uuid4())
    config["admission"] = "validation_required"
    config["accepting_writes"] = False
    config["credentials_file"] = str(state / "credentials.json")
    write_json(state / "config.json", config)
    (state / "tmp").mkdir(mode=0o700)
    return public_status(state, config)


def validate(
    state: Path, config: dict[str, object], interpretation: str, command: list[str]
) -> dict[str, object]:
    if (
        config["admission"] != "validation_required"
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


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument(
        "command",
        choices=[
            "setup",
            "reconfigure",
            "start",
            "status",
            "quiesce",
            "stop",
            "kill",
            "backup",
            "restore",
            "validate",
            "worker",
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
                    / "pse-arrow/surreal"
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
    result.add_argument("--memory-mib", type=int)
    result.add_argument("--server-memory-mib", type=int)
    result.add_argument("--native-workers", type=int)
    result.add_argument("--native-worker-memory-mib", type=int)
    result.add_argument("--interpretation")
    result.add_argument(
        "--drained",
        action="store_true",
        help="Caller has quiesced admission and drained managed native workers",
    )
    result.add_argument("--destination", type=Path)
    result.add_argument("--source", type=Path)
    result.add_argument("--check-command", nargs=argparse.REMAINDER)
    result.add_argument("--worker-command", nargs=argparse.REMAINDER)
    return result


def dispatch(args: argparse.Namespace) -> int:
    if args.command == "setup":
        output = setup(args)
    elif args.command == "restore":
        if args.source is None:
            raise SupervisorError("restore requires --source")
        output = restore(
            args.source.absolute(), args.state.absolute(), args.interpretation or ""
        )
    elif args.command == "worker":
        return worker(args.state.absolute(), args.worker_command or [])
    elif args.command == "_serve":
        return serve(args.state.absolute())
    else:
        state = args.state.absolute()
        config_for(
            state
        )  # Reject unowned state before changing permissions or writing a lock.
        state = checked_directory(state)
        with state_lock(state):
            config = config_for(state)
            if args.command == "start":
                start(state, config)
            elif args.command == "reconfigure":
                reconfigure(state, config, args)
            elif args.command == "quiesce":
                config["accepting_writes"] = False
                if config["admission"] != "validation_required":
                    config["admission"] = "quiescing"
                write_json(state / "config.json", config)
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


def main() -> int:
    args = parser().parse_args()
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

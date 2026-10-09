# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One owned foreground cache service; compilers stay in their caller's cgroup.

sccache 0.17 has no client-only switch. NO_DAEMON keeps a raced lazy-start
child in the wrapper's process group. Service death refuses the result and
terminates that group; a transient fork/cache effect remains possible.
"""

from __future__ import annotations

import contextlib
import dataclasses
import fcntl
import hashlib
import json
import os
import re
import selectors
import shutil
import signal
import socket
import stat
import struct
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, NoReturn

import msgspec

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import host_admission as host  # noqa: E402 -- direct-script root routing
from scripts import native_operation as operation  # noqa: E402 -- script routing


class CacheSignal(BaseException):
    """Preserve caller signals after draining the owned client group."""

    def __init__(self, number: int):
        self.number = number


class CacheError(Exception):
    """An unproved cache owner cannot admit a compilation."""


def refuse(message: str) -> NoReturn:
    """Raise a fixed boundary refusal, including inside owner-cleanup scopes."""
    raise CacheError(message)


def saved_binary_identity(value: object) -> tuple[int, int, int, int]:
    return msgspec.convert(value, type=tuple[int, int, int, int])


def remaining(deadline: float) -> float:
    value = deadline - time.monotonic()
    if value <= 0:
        raise CacheError("compiler cache original startup clock exhausted")
    return value


def private(directory: Path) -> None:
    if any(path.is_symlink() for path in (directory, *directory.parents)):
        raise CacheError("compiler cache state cannot traverse symlinks")
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = directory.stat()
    if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) & 0o077:
        raise CacheError("compiler cache state must be private and owned")


def read(path: Path) -> dict[str, object]:
    info = path.lstat()
    if (
        not stat.S_ISREG(info.st_mode)
        or info.st_uid != os.getuid()
        or info.st_mode & 0o077
        or info.st_size > 64 * 1024
    ):
        raise CacheError("compiler cache descriptor must be a bounded private file")
    result = json.loads(path.read_text())
    if not isinstance(result, dict):
        raise CacheError("compiler cache descriptor must be an object")
    return result


@contextlib.contextmanager
def metadata(directory: Path, deadline: float) -> Generator[None, None, None]:
    private(directory)
    fd = os.open(
        directory / ".lock",
        os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o600,
    )
    try:
        info = os.fstat(fd)
        if info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise CacheError("compiler cache metadata lock is not private")
        while True:
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                time.sleep(min(0.01, remaining(deadline)))
        yield
    finally:
        os.close(fd)


def real_binary(env: Mapping[str, str]) -> Path:
    selected = env.get("PSE_SCCACHE_BINARY") or shutil.which(
        "sccache", path=env.get("PATH")
    )
    if not selected:
        raise CacheError("configure PSE_SCCACHE_BINARY with installed sccache")
    binary = Path(selected).resolve(strict=True)
    if (
        binary == (ROOT / "scripts/sccache").resolve()
        or not binary.is_file()
        or not os.access(binary, os.X_OK)
    ):
        raise CacheError("PSE_SCCACHE_BINARY must select the real executable")
    with binary.open("rb") as source:
        if source.read(4) != b"\x7fELF":
            raise CacheError("PSE_SCCACHE_BINARY must select the installed ELF binary")
    return binary


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def identity(path: Path) -> tuple[int, int, int, int]:
    info = path.stat()
    return info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns


def system_environment(env: Mapping[str, str]) -> dict[str, str]:
    result = dict(env)
    runtime = Path(f"/run/user/{os.getuid()}")
    result.setdefault("XDG_RUNTIME_DIR", str(runtime))
    result.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    return result


def manager(
    arguments: Sequence[str], env: Mapping[str, str], deadline: float
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["systemctl", "--user", *arguments],
        env=system_environment(env),
        capture_output=True,
        text=True,
        timeout=min(5, remaining(deadline)),
        check=False,
    )


def observe(unit: str, env: Mapping[str, str], deadline: float) -> dict[str, str]:
    result = manager(
        ["show", "--property=MainPID,InvocationID,ControlGroup,ActiveState", unit],
        env,
        deadline,
    )
    if result.returncode:
        raise CacheError("compiler cache unit observation failed")
    return dict(
        line.split("=", 1) for line in result.stdout.splitlines() if "=" in line
    )


def peer(path: Path, deadline: float) -> int | None:
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(min(0.25, remaining(deadline)))
            connection.connect(str(path))
            pid, uid, _ = struct.unpack(
                "3i",
                connection.getsockopt(
                    socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i")
                ),
            )
            if uid != os.getuid():
                raise CacheError("compiler cache socket has a foreign owner")
            return pid
    except (FileNotFoundError, ConnectionRefusedError):
        return None
    except TimeoutError:
        raise CacheError("compiler cache socket owner is unobservable") from None


@dataclasses.dataclass(frozen=True)
class Service:
    descriptor: Mapping[str, object]
    pid: int
    start: str
    invocation: str
    group: str
    socket_inode: tuple[int, int]

    @property
    def endpoint(self) -> str:
        return str(self.descriptor["socket"])


def registered(record: Mapping[str, object], group: str, invocation: str) -> bool:
    allocation = record.get("allocation")
    if not isinstance(allocation, dict) or not isinstance(
        allocation.get(host.MARKER), str
    ):
        return False
    marker = Path(allocation[host.MARKER])
    if not re.fullmatch(r"[a-f0-9]{32}", marker.name):
        return False
    with host.metadata(marker.parent) as ledger:
        owners = ledger["owners"]
        if not isinstance(owners, dict):
            raise CacheError("compiler cache host owner ledger is invalid")
        owner = owners.get(marker.name)
        if (
            not isinstance(owner, dict)
            or owner.get("class") != "compiler-cache"
            or owner.get("lane") != "light"
        ):
            return False
        units = owner.get("units")
        if not isinstance(units, dict):
            return False
        association = units.get(str(record["unit"]))
        if not isinstance(association, dict):
            return False
        expected = dict(association)
    return (
        expected.get("group") == group
        and expected.get("invocation") == invocation
        and expected.get("inode") == host.group_identity(group)
    )


def ready(
    record: Mapping[str, object], env: Mapping[str, str], deadline: float
) -> Service | None:
    try:
        binary = Path(str(record["binary"]))
        endpoint = Path(str(record["socket"]))
        observation = observe(str(record["unit"]), env, deadline)
        pid = int(observation.get("MainPID", "0"))
        group = observation.get("ControlGroup", "")
        invocation = observation.get("InvocationID", "")
        if (
            record.get("version") != 1
            or observation.get("ActiveState") != "active"
            or pid <= 0
            or not group
            or not re.fullmatch(r"[a-f0-9]{32}", invocation)
        ):
            return None
        if (
            identity(Path(f"/proc/{pid}/exe"))
            != saved_binary_identity(record["binary_identity"])
            or Path(f"/proc/{pid}/exe").resolve() != binary
        ):
            return None
        if (
            operation.process_group(pid) != group
            or peer(endpoint, deadline) != pid
            or not registered(record, group, invocation)
        ):
            return None
        info = endpoint.lstat()
        if (
            not stat.S_ISSOCK(info.st_mode)
            or info.st_uid != os.getuid()
            or info.st_mode & 0o077
        ):
            raise CacheError("compiler cache socket must be private and owned")
        return Service(
            record,
            pid,
            operation.start_identity(pid),
            invocation,
            group,
            (info.st_dev, info.st_ino),
        )
    except (FileNotFoundError, ProcessLookupError):
        return None


def daemon_environment(record: Mapping[str, object]) -> dict[str, str]:
    # An allowlist cannot inherit job/native/handoff, cloud credentials or local secrets.
    return {
        "HOME": str(Path.home()),
        "PATH": "/usr/bin:/bin",
        "LANG": "C.UTF-8",
        "SCCACHE_START_SERVER": "1",
        "SCCACHE_NO_DAEMON": "1",
        "SCCACHE_IDLE_TIMEOUT": "0",
        "SCCACHE_CLIENT_SIDE": "0",
        "SCCACHE_DIRECT": "false",
        "SCCACHE_DIR": str(record["cache"]),
        "SCCACHE_CACHE_SIZE": str(record["cache_size"]),
        "SCCACHE_CONF": str(record["configuration"]),
        "SCCACHE_SERVER_UDS": str(record["socket"]),
    }


def quote(value: str) -> str:
    if "\n" in value or "\r" in value or "\0" in value:
        raise CacheError("invalid compiler cache unit field")
    return (
        '"' + value.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%") + '"'
    )


def startup_inputs() -> dict[str, str]:
    # A manually restarted unit must refuse changed admission code rather than
    # consume a different owner under the old generation descriptor.
    paths = [
        Path(__file__).resolve(),
        ROOT / "scripts/host_admission.py",
        ROOT / "scripts/native_operation.py",
        ROOT / "scripts/build_environment.py",
        ROOT / ".config/agent-capacity.toml",
        Path(sys.executable).resolve(),
    ]
    return {str(path): digest(path) for path in paths}


def unit_text(record: Mapping[str, object], allocation: host.Allocation) -> str:
    command = " ".join(
        quote(value)
        for value in [
            sys.executable,
            str(Path(__file__).resolve()),
            "--pse-cache-serve",
            str(record["record"]),
        ]
    )
    return "\n".join(
        [
            "[Unit]",
            "Description=Owned PSE compiler cache",
            "[Service]",
            "Type=exec",
            "Restart=no",
            f"Slice={host.allocation_slice(allocation)}",
            f"MemoryMax={allocation.profile.memory}",
            "MemorySwapMax=0",
            f"CPUAffinity={' '.join(map(str, host.cpu_set(allocation.profile.cores)))}",
            "TasksMax=128",
            "KillMode=control-group",
            "TimeoutStartSec=30",
            "TimeoutStopSec=15",
            "UMask=0077",
            f"ExecStart={command}",
            "",
        ]
    )


def ensure(env: Mapping[str, str], *, deadline: float | None = None) -> Service:
    deadline = time.monotonic() + 30 if deadline is None else deadline
    remaining(deadline)
    binary = real_binary(env)
    cache = Path(env["SCCACHE_DIR"]).resolve()
    configuration = Path(
        env.get("SCCACHE_CONF", str(ROOT / ".config/sccache.toml"))
    ).resolve(strict=True)
    signature = hashlib.sha256(
        json.dumps(
            [
                str(cache),
                digest(binary),
                digest(configuration),
                env.get("SCCACHE_CACHE_SIZE", "100G"),
            ]
        ).encode()
    ).hexdigest()[:16]
    state = (
        Path(env.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse-arrow/compiler-cache"
        / signature
    )
    remaining(deadline)
    private(state)
    current = state / "current.json"
    nonce = uuid.uuid4().hex
    while True:
        remaining(deadline)
        candidates = [read(current)] if current.exists() else []
        candidates.extend(
            read(path)
            for path in state.glob("*/service.json")
            if not candidates or str(path) != candidates[0].get("record")
        )
        waiting = False
        for candidate in candidates:
            existing = ready(candidate, env, deadline)
            if existing is not None:
                return existing
            # Stock sccache unlinks a filesystem UDS before binding. Never start
            # another owner while a live replacement still holds this endpoint.
            observation = observe(str(candidate["unit"]), env, deadline)
            if (
                int(observation.get("MainPID", "0")) > 0
                or peer(Path(str(candidate["socket"])), deadline) is not None
            ):
                waiting = True
        if waiting:
            time.sleep(min(0.05, remaining(deadline)))
            continue
        reserved = False
        with metadata(state, deadline):
            owner_file = state / "reservation.json"
            owner = read(owner_file) if owner_file.exists() else {}
            alive = False
            with contextlib.suppress(FileNotFoundError, ProcessLookupError):
                alive = (
                    owner.get("boot") == host.boot()
                    and operation.start_identity(
                        msgspec.convert(owner.get("pid", -1), type=int)
                    )
                    == owner.get("start")
                    and msgspec.convert(owner.get("deadline", 0), type=float)
                    > time.monotonic()
                )
            if not alive:
                operation.write_json(
                    owner_file,
                    {
                        "nonce": nonce,
                        "pid": os.getpid(),
                        "start": operation.start_identity(os.getpid()),
                        "boot": host.boot(),
                        "deadline": deadline,
                    },
                )
                reserved = True
        if reserved:
            break
        time.sleep(min(0.05, remaining(deadline)))
    allocation = None
    try:
        profile = host.select("compiler-cache")
        allocation = host.acquire(
            profile, directory=host.root_path(env), deadline=deadline
        )
        placement_env = system_environment(env)
        remaining(deadline)
        host.enforce_parent(profile, placement_env)
        remaining(deadline)
        host.enforce_allocation(allocation, placement_env)
        remaining(deadline)
        generation = state / nonce
        private(generation)
        frozen_binary = generation / "sccache"
        shutil.copyfile(binary, frozen_binary)
        frozen_binary.chmod(0o500)
        frozen_configuration = generation / f"sccache{configuration.suffix}"
        shutil.copyfile(configuration, frozen_configuration)
        frozen_configuration.chmod(0o600)
        runtime = (
            Path(env.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}"))
            / "pse-compiler-cache"
        )
        private(runtime)
        endpoint = runtime / f"{nonce}.sock"
        if len(os.fsencode(endpoint)) >= 108:
            refuse("compiler cache socket path exceeds native UDS bound")
        unit = f"pse-compiler-cache-{nonce}.service"
        record: dict[str, object] = {
            "version": 1,
            "generation": nonce,
            "unit": unit,
            "startup_inputs": startup_inputs(),
            "binary": str(frozen_binary),
            "binary_identity": list(identity(frozen_binary)),
            "binary_sha256": digest(frozen_binary),
            "configuration": str(frozen_configuration),
            "configuration_sha256": digest(frozen_configuration),
            "cache": str(cache),
            "cache_size": env.get("SCCACHE_CACHE_SIZE", "100G"),
            "socket": str(endpoint),
            "allocation": allocation.environment(),
            "record": str(generation / "service.json"),
        }
        operation.write_json(generation / "service.json", record)
        units = (
            Path(env.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
            / "systemd/user"
        )
        units.mkdir(parents=True, exist_ok=True)
        unit_path = units / unit
        if unit_path.exists():
            refuse("compiler cache generation unit already exists")
        unit_path.write_text(unit_text(record, allocation))
        unit_path.chmod(0o600)
        allocation.register(unit)
        for arguments in (["daemon-reload"], ["start", unit]):
            if manager(arguments, env, deadline).returncode:
                refuse("compiler cache foreground service launch failed")
        while True:
            admitted = ready(record, env, deadline)
            if admitted is not None:
                with metadata(state, deadline):
                    if read(state / "reservation.json").get("nonce") != nonce:
                        refuse("compiler cache startup reservation changed")
                    operation.write_json(current, record)
                    operation.write_json(state / "reservation.json", {})
                return admitted
            time.sleep(min(0.02, remaining(deadline)))
    except BaseException:
        if allocation is not None:
            allocation.release()  # Actual live groups remain charged.
        raise
    finally:
        with metadata(state, deadline + 1):
            owner_file = state / "reservation.json"
            if owner_file.exists() and read(owner_file).get("nonce") == nonce:
                operation.write_json(owner_file, {})


def serve(path: Path) -> None:
    record = read(path)
    if (
        Path(str(record["record"])) != path
        or identity(Path(str(record["binary"])))
        != saved_binary_identity(record["binary_identity"])
        or digest(Path(str(record["binary"]))) != record["binary_sha256"]
        or digest(Path(str(record["configuration"]))) != record["configuration_sha256"]
    ):
        raise CacheError("compiler cache executable generation changed")
    if record.get("startup_inputs") != startup_inputs():
        raise CacheError(
            "compiler cache startup admission inputs changed; readmit a new generation"
        )
    inherited = msgspec.convert(record["allocation"], type=dict[str, str])
    allocation = host.inherit(inherited, handoff=True)
    if allocation is None or allocation.profile.name != "compiler-cache":
        raise CacheError("compiler cache requires its independent admitted owner")
    allocation.bind(str(record["unit"]))
    os.execve(  # noqa: S606 -- verified generation ELF, fixed argv, scrubbed environment
        str(record["binary"]), [str(record["binary"])], daemon_environment(record)
    )


def same(service: Service, env: Mapping[str, str]) -> bool:
    observed = ready(service.descriptor, env, time.monotonic() + 5)
    return observed is not None and (
        observed.pid,
        observed.start,
        observed.invocation,
        observed.group,
        observed.socket_inode,
    ) == (
        service.pid,
        service.start,
        service.invocation,
        service.group,
        service.socket_inode,
    )


def finish_group(process: subprocess.Popen[bytes]) -> int:
    # Keep the leader unreaped until group termination, so its PGID cannot be reused.
    for number in (signal.SIGTERM, signal.SIGKILL):
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, number)
        if number == signal.SIGTERM:
            time.sleep(0.05)
    return process.wait(timeout=5)


@contextlib.contextmanager
def pidfd(pid: int) -> Generator[int, None, None]:
    descriptor = os.pidfd_open(pid)
    try:
        yield descriptor
    finally:
        os.close(descriptor)


@contextlib.contextmanager
def caller_signals() -> Generator[None, None, None]:
    def interrupted(number: int, _frame: object) -> None:
        raise CacheSignal(number)

    previous = {
        number: signal.getsignal(number)
        for number in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM)
    }
    try:
        for number in previous:
            signal.signal(number, interrupted)
        yield
    finally:
        for number, handler in previous.items():
            signal.signal(number, handler)


def compile_command(
    arguments: Sequence[str], env: Mapping[str, str], service: Service
) -> int:
    with caller_signals():
        return supervised_compile(arguments, env, service)


def supervised_compile(
    arguments: Sequence[str], env: Mapping[str, str], service: Service
) -> int:
    if not same(service, env):
        raise CacheError("compiler cache service identity changed before invocation")
    child_env = dict(env)
    child_env.update(
        SCCACHE_SERVER_UDS=service.endpoint,
        SCCACHE_CLIENT_SIDE="1",
        SCCACHE_DIRECT="false",
        SCCACHE_NO_DAEMON="1",
    )
    child_env.pop("SCCACHE_START_SERVER", None)
    child_env.pop("SCCACHE_STARTUP_NOTIFY", None)
    if "SCCACHE_ERROR_LOG" in child_env:
        raise CacheError("SCCACHE_ERROR_LOG disables supervised client compilation")
    with pidfd(service.pid) as descriptor:
        if not same(service, env):
            raise CacheError("compiler cache service changed during lifetime binding")
        with selectors.DefaultSelector() as monitor:
            monitor.register(descriptor, selectors.EVENT_READ)
            process = subprocess.Popen(
                [str(real_binary(env)), *arguments],
                env=child_env,
                start_new_session=True,
            )
            completed = False
            try:
                while True:
                    if monitor.select(0.05):
                        raise CacheError(
                            "compiler cache service died; compilation refused"
                        )
                    if (
                        os.waitid(
                            os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT
                        )
                        is not None
                    ):
                        completed = True
                        break
                if not same(service, env):
                    raise CacheError(
                        "compiler cache service identity changed after invocation"
                    )
            finally:
                result = finish_group(process)
            if not completed:
                raise CacheError("compiler cache client did not complete")
            return result if result >= 0 else 128 - result


def main(arguments: Sequence[str] | None = None) -> int:
    argv = list(sys.argv[1:] if arguments is None else arguments)
    env = dict(os.environ)
    try:
        if len(argv) == 2 and argv[0] == "--pse-cache-serve":
            serve(Path(argv[1]))
            return 0
        if env.get("SCCACHE_START_SERVER") == "1" or (
            argv and argv[0].startswith("--")
        ):
            if not env.get("SCCACHE_SERVER_UDS"):
                refuse("explicit cache control requires its selected endpoint")
            if (
                env.get("SCCACHE_START_SERVER") == "1"
                and env.get("SCCACHE_NO_DAEMON") != "1"
            ):
                refuse("explicit cache server requires foreground ownership")
            os.execve(str(real_binary(env)), [str(real_binary(env)), *argv], env)  # noqa: S606 -- explicitly selected local cache control
            return 0
        service = ensure(env)
        return compile_command(argv, env, service)
    except CacheSignal as interrupted:
        return 128 + interrupted.number
    except (
        CacheError,
        host.AdmissionError,
        OSError,
        ValueError,
        subprocess.TimeoutExpired,
    ) as error:
        print(f"pse-env: compiler cache boundary: {error}", file=sys.stderr)
        return 125


if __name__ == "__main__":
    raise SystemExit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private external-primary native-entry controls for public Python studies."""

import math
import os
import signal
import stat
import subprocess
import sys
import time
import uuid
from collections.abc import Callable, Sequence
from contextlib import suppress
from pathlib import Path
from typing import Self

import msgspec
from scripts import native_operation, test_resources
from scripts import surreal_server as server


class NativeEntry(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    thread: str
    released: bool
    stop_observed: bool


class NativeEntries(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    nonce: str
    canonical_database: str
    pid: int
    entries: tuple[NativeEntry, ...]
    active: int
    maximum: int


class CallerStarted(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    nonce: str
    started: float


def _read_private(path: Path) -> bytes:
    metadata = path.lstat()
    assert stat.S_ISREG(metadata.st_mode)
    assert metadata.st_uid == os.getuid()
    assert metadata.st_mode & 0o077 == 0
    assert metadata.st_size <= 8192
    with path.open("rb") as stream:
        content = stream.read(8193)
    assert len(content) <= 8192
    return content


def _read_object(path: Path) -> dict[str, object]:
    return msgspec.json.decode(_read_private(path), type=dict[str, object])


def _write_private(path: Path, value: object) -> None:
    content = msgspec.json.encode(value)
    assert len(content) <= 8192
    temporary = path.with_suffix(f".{os.getpid()}-temporary")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


class ManagedStudyFixture:
    """Own one nonce-bound barrier and drain its primary before removing controls.

    This mirrors the Rust managed study fixture's existing private seam. It never
    starts local native work or gives the observer a primary worker allocation.
    """

    def __init__(
        self,
        state: str,
        *,
        database: str | None = None,
        resource: str | None = None,
        test: str | None = None,
    ) -> None:
        self.state = server.receiver_context(Path(state).absolute(), database)
        self.resource = resource
        self.test = test
        self.controls_resource: str | None = None
        self.lock = self.state / "managed-study-qualification.lock"
        self.nonce = uuid.uuid4().hex
        self.directory = self.state / f"native-entry-qualification-{self.nonce}"
        self.database = ""
        self.observer_group = ""
        self.worker_pid = 0
        self.lock_owned = False
        self.primary_attempted = False
        self.caller: subprocess.Popen[bytes] | None = None
        self.drain_deadline: float | None = None

    def __enter__(self) -> Self:
        with server.state_lock(self.state):
            if self.lock.exists():
                prior = _read_object(self.lock)
                try:
                    alive = (
                        native_operation.start_identity(
                            msgspec.convert(prior["pid"], type=int)
                        )
                        == prior["start"]
                    )
                except FileNotFoundError:
                    alive = False
                assert not alive, "another live owner has this qualification context"
            _write_private(
                self.lock,
                {
                    "nonce": self.nonce,
                    "pid": os.getpid(),
                    "start": native_operation.start_identity(os.getpid()),
                },
            )
            self.lock_owned = True
        try:
            config = _read_object(self.state / "config.json")
            resources = msgspec.convert(config["resources"], type=dict[str, object])
            profile = msgspec.convert(resources["execution"], type=dict[str, int])
            assert resources["native_workers"] == 1
            assert tuple(
                profile[key]
                for key in (
                    "pool_memory_bytes",
                    "cpu_threads",
                    "case_lanes",
                    "math_jobs",
                )
            ) == (128 << 30, 16, 16, 32)
            assert 0 < profile["observer_memory_bytes"] <= 4 << 30
            # The managed Python route places this caller in the observer group.
            cgroup = next(
                line.removeprefix("0::")
                for line in Path("/proc/self/cgroup").read_text().splitlines()
                if line.startswith("0::")
            )
            group = Path("/sys/fs/cgroup") / cgroup.lstrip("/")
            self.observer_group = cgroup
            limits = [
                (ancestor / "memory.max").read_text().strip()
                for ancestor in (group, *group.parents)
                if (ancestor / "memory.max").is_file()
            ]
            assert any(
                limit != "max" and 0 < int(limit) <= profile["observer_memory_bytes"]
                for limit in limits
            ), "public Python caller must retain the four GiB observer placement"
            self.database = msgspec.convert(config["database"], type=str)
            receiver = msgspec.convert(config["primary_receiver"], type=dict[str, str])
            self._handoff_prior_observer()
            self.controls_resource = test_resources.register(
                {
                    "state": str(self.state),
                    "database": self.database,
                    "kind": "controls",
                    "directory": str(self.directory),
                    "test": self.test or "",
                }
            )
            self.directory.mkdir(mode=0o700)
            _write_private(
                self.directory / "request.json",
                {
                    "nonce": self.nonce,
                    "canonical_database": self.database,
                    "entries": 16,
                    "entry_timeout_ms": 90_000,
                },
            )
            self.primary_attempted = True
            output = subprocess.run(
                [
                    receiver["supervisor_executable"],
                    receiver["supervisor_script"],
                    "ensure-primary",
                    "--state",
                    str(self.state),
                    "--canonical-database",
                    self.database,
                    "--qualification-native-entry",
                    str(self.directory),
                ],
                capture_output=True,
                timeout=45,
                check=False,
            )
            assert len(output.stdout) <= 64 << 10
            assert len(output.stderr) <= 64 << 10
            assert output.returncode == 0, output.stderr.decode(errors="replace")
            ready = msgspec.json.decode(output.stdout, type=dict[str, object])
            assert ready["ready"] is True
            assert ready["canonical_database"] == self.database
            marker = _read_object(self.state / "primary-receiver.json")
            assert marker["canonical_database"] == self.database
            assert marker["qualification_native_entry"] == str(self.directory)
            for key in ("pool_memory_bytes", "cpu_threads", "case_lanes", "math_jobs"):
                assert marker[key] == profile[key]
            self.worker_pid = msgspec.convert(marker["pid"], type=int)
            assert self.worker_pid > 0
            assert self.worker_pid != os.getpid()
        except BaseException:
            self.close()
            raise
        else:
            return self

    def signal(self, name: str) -> None:
        _write_private(self.directory / name, self.nonce)

    def _handoff_prior_observer(self) -> None:
        """Drain only the prior primary owned by this sequential pytest caller."""
        root = str(Path(__file__).resolve().parents[3])
        if root not in sys.path:
            sys.path.insert(0, root)
        from scripts import (  # noqa: PLC0415 -- load after source root registration
            surreal_server as server,
        )
        from scripts.case_measure import (  # noqa: PLC0415 -- private drain owner
            stop_managed_primary,
        )

        owned = False
        with server.state_lock(self.state):
            registration = self.state / "primary-observer.json"
            if registration.exists():
                record = _read_object(registration)
                pid = msgspec.convert(record["pid"], type=int)
                if pid == os.getpid():
                    owned = True
                else:
                    process = Path(f"/proc/{pid}/stat")
                    assert (
                        not process.exists()
                        or process.read_text().rsplit(")", 1)[1].split()[19]
                        != record["start"]
                    ), "another live caller owns the observer allocation"
        if owned:
            server._checked_current_observer(self.state)
        prior = getattr(server._STARTUP, "deadline", None)
        deadline = time.monotonic() + 10
        server._STARTUP.deadline = deadline if prior is None else min(deadline, prior)
        try:
            observed = server.primary_observation(self.state)
        finally:
            server._STARTUP.deadline = prior
        group = observed["ControlGroup"]
        live = observed["ActiveState"] not in {"inactive", "failed"} or bool(
            group and server.group_populated(group)
        )
        assert not live or owned, "refuse to drain a primary owned by another observer"
        if live:
            stop_managed_primary(self.state, self.database)
        if owned:
            server.release_current_observer(self.state)

    def call(
        self,
        command: Sequence[str],
        assert_held: Callable[[NativeEntries], None],
        *,
        cancel: bool = False,
        setup_timeout: float = 45,
        timeout: float = 90,
    ) -> None:
        """Supervise the actual public caller; signals retain its native future.

        Setup has a separate finite clock. The operation clock starts in the
        child immediately before the public call, and is never renewed after
        releasing the sixteen owners. A nonzero child exit preserves assertions.
        """
        log = self.directory / "caller.log"
        descriptor = os.open(log, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "wb") as output:
            self.caller = subprocess.Popen(
                command, stdout=output, stderr=subprocess.STDOUT, start_new_session=True
            )
        _write_private(
            self.directory / "caller.json",
            {"pid": self.caller.pid, "nonce": self.nonce},
        )
        try:
            deadline = time.monotonic() + setup_timeout
            while not (self.directory / "calling.json").exists():
                self._check_caller()
                assert self.caller.poll() is None, (
                    "public caller exited before the public call"
                )
                assert time.monotonic() < deadline, "public caller setup timed out"
                time.sleep(0.02)
            started = msgspec.json.decode(
                _read_private(self.directory / "calling.json"), type=CallerStarted
            )
            assert started.nonce == self.nonce
            assert math.isfinite(started.started)
            assert 0 < started.started <= time.monotonic()
            deadline = started.started + timeout
            held = self.observation(
                lambda value: value.active == 16,
                timeout=max(0, deadline - time.monotonic()),
            )
            assert_held(held)
            self.signal("cancel.json" if cancel else "release.json")
            while self.caller.poll() is None:
                assert time.monotonic() < deadline, "public study operation timed out"
                time.sleep(0.02)
            self._check_caller()
        except BaseException:
            # close shares one cancellation/drain clock with __exit__; it never
            # kills or abandons a caller whose native future has not joined.
            self.close()
            raise

    def _check_caller(self) -> None:
        assert self.caller is not None
        status = self.caller.poll()
        if status is not None:
            assert (self.directory / "caller.log").stat().st_size <= 64 << 10, (
                "public caller output exceeded its bound"
            )
            with (self.directory / "caller.log").open("rb") as output:
                detail = output.read(64 << 10).decode(errors="replace")
            assert status == 0, f"public caller exited {status}: {detail}"

    def _primary_alive(self, deadline: float) -> bool:
        """Check the supervisor-owned service even before the entry snapshot.

        A service association, rather than an entry file, retains the allocation
        during worker/runtime startup and after a malformed entry observation.
        An unrelated populated service is never adopted or signalled.
        """
        root = str(Path(__file__).resolve().parents[3])
        if root not in sys.path:
            sys.path.insert(0, root)
        from scripts import (  # noqa: PLC0415 -- load after source root registration
            surreal_server as server,
        )

        remaining = deadline - time.monotonic()
        assert remaining > 0, "qualified primary did not drain; controls retained"
        observed = subprocess.run(
            [
                "systemctl",
                "--user",
                "show",
                "--property=LoadState",
                "--property=ActiveState",
                "--property=ControlGroup",
                "--property=MainPID",
                server.primary_unit(self.state),
            ],
            env=server.systemd_environment(),
            capture_output=True,
            text=True,
            check=False,
            timeout=min(2, remaining),
        )
        assert observed.returncode == 0, (
            "primary service ownership is unproved; controls retained"
        )
        fields = dict(
            line.split("=", 1) for line in observed.stdout.splitlines() if "=" in line
        )
        assert all(
            key in fields
            for key in ("LoadState", "ActiveState", "ControlGroup", "MainPID")
        )
        group = fields["ControlGroup"]
        if not group:
            assert fields["ActiveState"] in {"inactive", "failed"}, (
                "primary service is still starting"
            )
            return False
        assert group.startswith("/")
        assert group != "/"
        assert ".." not in Path(group).parts
        directory = Path("/sys/fs/cgroup") / group.lstrip("/")
        try:
            events = (directory / "cgroup.events").read_text().splitlines()
        except FileNotFoundError:
            return False
        populated = next(
            (line.split()[1] for line in events if line.startswith("populated ")), ""
        )
        assert populated in {"0", "1"}, "primary group population is unproved"
        if populated == "0":
            return False
        pid = int(fields["MainPID"])
        assert pid > 0, (
            "populated primary service has no associated leader; controls retained"
        )
        arguments = (Path(f"/proc/{pid}") / "cmdline").read_bytes().split(b"\0")
        relative = next(
            line.removeprefix("0::")
            for line in (Path(f"/proc/{pid}") / "cgroup").read_text().splitlines()
            if line.startswith("0::")
        )
        assert relative == group or relative.startswith(group + "/"), (
            "primary leader is outside its service"
        )
        assert any(
            arguments[index : index + 2]
            == [b"--qualification-native-entry", os.fsencode(self.directory)]
            for index in range(len(arguments) - 1)
        ), "live primary is not associated with this fixture; controls retained"
        assert any(
            arguments[index : index + 2]
            == [b"--canonical-database", self.database.encode()]
            for index in range(len(arguments) - 1)
        ), "live primary is not associated with this fixture; controls retained"
        return True

    def observation(
        self, predicate: Callable[[NativeEntries], bool], *, timeout: float = 90
    ) -> NativeEntries:
        deadline = time.monotonic() + timeout
        while True:
            if self.caller is not None:
                self._check_caller()
            failure = self.directory / "failure.json"
            assert not failure.exists(), (
                _read_private(failure).decode() if failure.exists() else ""
            )
            assert Path(f"/proc/{self.worker_pid}").exists(), "qualified primary exited"
            try:
                content = _read_private(self.directory / "entries.json")
            except FileNotFoundError:
                assert Path(f"/proc/{self.worker_pid}").exists(), (
                    "qualified primary exited"
                )
                assert time.monotonic() < deadline, "native entry observation timed out"
                time.sleep(0.02)
                continue
            value = msgspec.json.decode(content, type=NativeEntries)
            assert value.nonce == self.nonce
            assert value.canonical_database == self.database
            assert value.pid == self.worker_pid
            assert 0 <= value.active <= value.maximum <= 16
            assert len(value.entries) <= 16
            if predicate(value):
                return value
            assert Path(f"/proc/{self.worker_pid}").exists(), "qualified primary exited"
            assert time.monotonic() < deadline, "native entry observation timed out"
            time.sleep(0.02)

    def close(self) -> None:
        if not self.lock_owned:
            return
        if self.directory.exists():
            if self.drain_deadline is None:
                self.drain_deadline = time.monotonic() + 45
            deadline = self.drain_deadline
            if self.worker_pid == 0 and (self.directory / "entries.json").exists():
                try:
                    value = msgspec.json.decode(
                        _read_private(self.directory / "entries.json"),
                        type=NativeEntries,
                    )
                    if (
                        value.nonce == self.nonce
                        and value.canonical_database == self.database
                        and value.pid > 0
                        and value.pid != os.getpid()
                    ):
                        self.worker_pid = value.pid
                except (AssertionError, OSError, msgspec.DecodeError):
                    # A malformed startup observation must not prevent release
                    # and stop signals from reaching this nonce's controller.
                    pass
            if self.caller is not None and self.caller.poll() is None:
                with suppress(ProcessLookupError):
                    self.caller.send_signal(signal.SIGINT)
            if self.primary_attempted:
                self._primary_alive(deadline)
            # Failure cleanup releases blocked owners and asks the existing primary
            # driver to cancel/drain; never kill an owner during native teardown.
            for name in ("release.json", "stop.json"):
                if not (self.directory / name).exists():
                    self.signal(name)
            while True:
                caller_alive = self.caller is not None and self.caller.poll() is None
                worker_alive = bool(
                    self.worker_pid and Path(f"/proc/{self.worker_pid}").exists()
                )
                service_alive = self.primary_attempted and self._primary_alive(deadline)
                if not (caller_alive or worker_alive or service_alive):
                    break
                assert time.monotonic() < deadline, (
                    "public caller or qualified primary did not drain; "
                    "controls retained"
                )
                time.sleep(0.02)
            if self.worker_pid and (self.directory / "entries.json").exists():
                final = msgspec.json.decode(
                    _read_private(self.directory / "entries.json"), type=NativeEntries
                )
                assert final.nonce == self.nonce
                assert final.canonical_database == self.database
                assert final.pid == self.worker_pid
                assert final.active == 0
            if self.controls_resource is not None:
                test_resources.record_drain(self.controls_resource)
        with server.state_lock(self.state):
            if (
                self.lock.exists()
                and _read_object(self.lock).get("nonce") == self.nonce
            ):
                self.lock.unlink()
        self.lock_owned = False

    def __exit__(self, _kind: object, _error: object, _traceback: object) -> None:
        self.close()

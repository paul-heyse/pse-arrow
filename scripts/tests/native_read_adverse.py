# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Test-only authenticated process control and reversible owned image replacement."""

from __future__ import annotations

import asyncio
import fcntl
import hashlib
import json
import os
import select
import signal
import stat
import struct
import tempfile
import termios
import time
from contextlib import contextmanager, suppress
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import (
    artifact_publication,
    host_admission,
    operation_lifetime,
    process_observation,
    python_extension,
)
from scripts.capabilities.native_read_protocol import SESSION_ENVIRONMENT

if TYPE_CHECKING:
    from collections.abc import Generator

    from scripts.capabilities.native_read_sessions import ReadSession


def _sha(path: Path, expected: tuple[int, int] | None = None) -> str:
    descriptor = os.open(
        path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC
    )
    with os.fdopen(descriptor, "rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode) or (
            expected is not None and (before.st_dev, before.st_ino) != expected
        ):
            raise ReplacementRefusedError(
                "image observation no longer names the expected ordinary inode"
            )
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
        after = os.fstat(stream.fileno())
        if (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
        ) != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
        ):
            raise ReplacementRefusedError(
                "image bytes changed during their bounded observation"
            )
        return digest


def _matches(path: Path, identity: tuple[int, int], digest: str) -> bool:
    try:
        current = path.lstat()
        return (
            stat.S_ISREG(current.st_mode)
            and (current.st_dev, current.st_ino) == identity
            and _sha(path, identity) == digest
        )
    except (OSError, ReplacementRefusedError):
        return False


def image_stamp(path: Path) -> tuple[int, int, int, int, int]:
    value = path.stat()
    return (
        value.st_dev,
        value.st_ino,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


class ReplacementRefusedError(RuntimeError):
    """Preserve the original backup and any concurrent path for explicit inspection."""


@dataclass(frozen=True)
class ImageSelection:
    binary: Path
    sha256: str
    stamp: tuple[int, int, int, int, int]

    @classmethod
    def selected(cls, root: Path) -> ImageSelection:
        raw = os.environ.get("PSE_NATIVE_READ_IMAGE_REPLACEMENT")
        if raw is None or len(raw) > 16 << 10:
            raise ReplacementRefusedError(
                "root must explicitly select the freshly installed image and its source-hold window"
            )
        value = json.loads(raw)
        if not isinstance(value, dict) or set(value) != {
            "binary",
            "sha256",
            "artifact_stamp",
        }:
            raise ReplacementRefusedError(
                "malformed explicit installed image selection"
            )
        path, digest, stamp = value["binary"], value["sha256"], value["artifact_stamp"]
        if (
            not isinstance(path, str)
            or not isinstance(digest, str)
            or len(digest) != 64
            or any(c not in "0123456789abcdef" for c in digest)
            or not isinstance(stamp, list)
            or len(stamp) != 5
            or any(type(item) is not int for item in stamp)
        ):
            raise ReplacementRefusedError("invalid explicit installed image identity")
        binary = Path(path)
        if (
            not binary.is_absolute()
            or binary.is_symlink()
            or binary.parent.resolve() != (root / "python/pse").resolve()
            or not binary.name.startswith("_native.")
            or binary.suffix != ".so"
        ):
            raise ReplacementRefusedError(
                "image replacement is limited to the selected checkout's new ordinary extension"
            )
        stamps = [int(item) for item in stamp]
        return cls(
            binary, digest, (stamps[0], stamps[1], stamps[2], stamps[3], stamps[4])
        )


@dataclass(frozen=True)
class ReplacedImage:
    original: ImageSelection
    backup: Path
    replacement: tuple[int, int]


@contextmanager
def replaced_image(selected: ImageSelection) -> Generator[ReplacedImage]:
    """Never write the mapped original inode; never overwrite a concurrent restore path.

    A restoration refusal intentionally retains the private original backup. Renames and
    hardlinks can change ctime; inode/bytes are restored, not a fabricated old timestamp.
    """
    binary = selected.binary
    before = binary.lstat()
    if (
        binary.is_symlink()
        or not stat.S_ISREG(before.st_mode)
        or before.st_uid != os.getuid()
        or image_stamp(binary) != selected.stamp
        or _sha(binary, selected.stamp[:2]) != selected.sha256
    ):
        raise ReplacementRefusedError(
            "selected installed image changed before the owned control"
        )
    directory = Path(
        tempfile.mkdtemp(prefix=".pse-read-image-control-", dir=binary.parent)
    )
    backup, candidate, displaced = (
        directory / "original",
        directory / "replacement",
        directory / "displaced",
    )
    selected_path = directory / "selected-path"
    os.link(binary, backup, follow_symlinks=False)
    if not _matches(backup, selected.stamp[:2], selected.sha256):
        raise ReplacementRefusedError(
            "concurrent source change; original evidence retained at " + str(directory)
        )
    candidate.write_bytes(b"pse-owned-wrong-image-control\n")
    candidate.chmod(stat.S_IMODE(before.st_mode))
    wrong_identity = image_stamp(candidate)[:2]
    wrong_digest = _sha(candidate)
    # Moving the actual path aside and inspecting it prevents overwriting a
    # concurrent installation. Exclusive link publication refuses a new destination.
    binary.rename(selected_path)
    if not _matches(selected_path, selected.stamp[:2], selected.sha256):
        with suppress(FileExistsError):
            os.link(selected_path, binary, follow_symlinks=False)
        raise ReplacementRefusedError(
            "concurrent install before replacement; all evidence retained at "
            + str(directory)
        )
    try:
        os.link(candidate, binary, follow_symlinks=False)
    except FileExistsError as error:
        raise ReplacementRefusedError(
            "concurrent installer owns destination; original retained at " + str(backup)
        ) from error
    candidate.unlink()
    selected_path.unlink()
    original_error: BaseException | None = None
    try:
        yield ReplacedImage(selected, backup, wrong_identity)
    except BaseException as error:
        original_error = error

    def restore() -> None:
        if not _matches(binary, wrong_identity, wrong_digest):
            raise ReplacementRefusedError(
                "concurrent installed-path mutation; original backup retained at "
                + str(backup)
            )
        # Move only our expected replacement aside, then inspect what actually moved.
        # Exclusive linking restores an absent path and refuses a concurrent installer.
        binary.rename(displaced)
        if not _matches(displaced, wrong_identity, wrong_digest):
            with suppress(FileExistsError):
                os.link(displaced, binary, follow_symlinks=False)
            raise ReplacementRefusedError(
                "concurrent restore race; all image evidence retained at "
                + str(directory)
            )
        try:
            os.link(backup, binary, follow_symlinks=False)
        except FileExistsError as error:
            raise ReplacementRefusedError(
                "concurrent installer owns destination; original backup retained at "
                + str(backup)
            ) from error
        if not _matches(binary, selected.stamp[:2], selected.sha256):
            raise ReplacementRefusedError(
                "restored image failed original inode/byte check; backup retained at "
                + str(backup)
            )
        displaced.unlink()
        backup.unlink()
        directory.rmdir()

    try:
        restore()
    except BaseException as restoration_error:
        if original_error is not None:
            raise BaseExceptionGroup(
                "Image control and restoration both failed",
                [original_error, restoration_error],
            ) from None
        raise
    if original_error is not None:
        raise original_error


def _environment(pid: int) -> dict[str, str]:
    with Path(f"/proc/{pid}/environ").open("rb") as stream:
        raw = stream.read((1 << 20) + 1)
    if len(raw) > 1 << 20:
        raise ValueError("worker environment exceeds test observation extent")
    # Only the two lifecycle markers survive this local observation, never credentials.
    names = {
        SESSION_ENVIRONMENT.encode(),
        host_admission.MARKER.encode(),
        b"_PSE_NATIVE_PUBLICATION",
    }
    result = {}
    for item in raw.split(b"\0"):
        name, separator, value = item.partition(b"=")
        if separator and name in names:
            result[name.decode()] = value.decode()
    return result


class AuthenticatedReadWorker:
    """pidfd control of exactly the worker admitted by this test's private session."""

    def __init__(self, session: ReadSession) -> None:
        temporary = session._temporary  # noqa: SLF001 -- test authenticates its own actual private session admission
        if temporary is None:
            raise ValueError("actual native read session did not publish admission")
        admission = Path(temporary.name) / "session.json"
        self._authenticate(admission, session.incarnation)

    def _authenticate(self, admission: Path, incarnation: str) -> None:
        self.admission = admission
        document = json.loads(admission.read_bytes())
        if document.get("incarnation") != incarnation:
            raise ValueError("actual session admission incarnation differs")
        process_matches: list[tuple[int, dict[str, str]]] = []
        for path in Path("/proc").iterdir():
            if not path.name.isdigit():
                continue
            pid = int(path.name)
            try:
                command = (path / "cmdline").read_bytes().split(b"\0")
                if not any(
                    command[index : index + 2]
                    == [b"-m", b"scripts.capabilities.native_read_worker"]
                    for index in range(len(command))
                ):
                    continue
                environment = _environment(pid)
                if environment.get(SESSION_ENVIRONMENT) == str(admission):
                    process_matches.append((pid, environment))
            except (OSError, ValueError):
                continue
        if len(process_matches) != 1:
            raise ValueError(
                "private session did not authenticate exactly one actual read worker"
            )
        self.pid, environment = process_matches[0]
        marker = environment.get(host_admission.MARKER)
        if marker is None:
            raise ValueError("actual read worker lacks its original host allocation")
        self.allocation = Path(marker)
        first = process_observation.observe_process(self.pid)
        if (
            first.availability is not process_observation.Availability.PRESENT
            or first.group is None
        ):
            raise ValueError("actual read worker process observation unavailable")
        record = self._allocation_record()
        unit_matches = [
            (unit, bound)
            for unit, bound in record["units"].items()
            if isinstance(bound, dict)
            and isinstance(bound.get("group"), str)
            and (
                first.group == bound["group"]
                or first.group.startswith(bound["group"] + "/")
            )
        ]
        if not unit_matches:
            raise ValueError(
                "read worker is outside its original bound allocation units"
            )
        self.unit, self.bound = max(
            unit_matches, key=lambda item: len(item[1]["group"])
        )
        parent_group = process_observation.process_group(os.getpid())
        self.borrowed_parent = os.environ.get(host_admission.MARKER) == str(
            self.allocation
        ) and (
            parent_group == self.bound["group"]
            or parent_group.startswith(self.bound["group"] + "/")
        )
        manager = operation_lifetime.unit_observation(
            self.unit, deadline=time.monotonic() + 5
        )
        observed = process_observation.observe_process(
            self.pid, unit=self.unit, manager=manager
        )
        if (
            observed.availability is not process_observation.Availability.PRESENT
            or observed.invocation != self.bound.get("invocation")
            or process_observation.group_identity(self.bound["group"])
            != self.bound.get("inode")
        ):
            raise ValueError(
                "actual worker manager/allocation identity is not authenticated"
            )
        self.observed = observed
        self.descriptor = os.pidfd_open(self.pid)
        self.stopped = False
        try:
            self.require_charged()
        except BaseException:
            os.close(self.descriptor)
            raise

    @classmethod
    def from_foreground(
        cls, publication: Path, foreground_pid: int
    ) -> AuthenticatedReadWorker:
        """Authenticate the CLI descendant against its private publication and session."""
        matches = []
        for process in Path("/proc").iterdir():
            if not process.name.isdigit():
                continue
            try:
                environment = _environment(int(process.name))
                if environment.get("_PSE_NATIVE_PUBLICATION") != str(publication):
                    continue
                marker = environment.get(SESSION_ENVIRONMENT)
                if marker is None:
                    continue
                current = int(process.name)
                seen = set()
                while current > 1 and current not in seen:
                    seen.add(current)
                    stat_fields = (
                        Path(f"/proc/{current}/stat")
                        .read_text()
                        .rsplit(")", 1)[1]
                        .split()
                    )
                    current = int(stat_fields[1])
                    if current == foreground_pid:
                        admission = Path(marker)
                        document = json.loads(admission.read_bytes())
                        matches.append((admission, document["incarnation"]))
                        break
            except (OSError, ValueError, KeyError):
                continue
        unique = set(matches)
        if len(unique) != 1:
            raise ValueError(
                "foreground CLI did not authenticate exactly one private native session"
            )
        admission, incarnation = unique.pop()
        selected = cls.__new__(cls)
        cls._authenticate(selected, admission, incarnation)
        return selected

    def require_image(self, publication: artifact_publication.Publication) -> None:
        """Observe actual loaded path for acceptance, without consumer revalidation."""
        self.require_charged()
        paths = {
            line.split(maxsplit=5)[5]
            for line in Path(f"/proc/{self.pid}/maps").read_text().splitlines()
            if len(line.split(maxsplit=5)) == 6
        }
        if publication.artifact.binary not in paths:
            raise ValueError("actual native worker did not map its issued generation")

    async def require_settled(self) -> None:
        """Observe session-family exit; preserve an authenticated borrowed parent charge."""
        deadline = time.monotonic() + 10

        async def settled() -> None:
            while not self.settlement_observed(deadline):
                await asyncio.sleep(0.01)

        await asyncio.wait_for(settled(), timeout=10)

    def session_members(self) -> list[int]:
        """Exact inherited session marker also finds reparented session descendants."""
        members = []
        for process in Path("/proc").iterdir():
            if not process.name.isdigit():
                continue
            try:
                if _environment(int(process.name)).get(SESSION_ENVIRONMENT) == str(
                    self.admission
                ):
                    members.append(int(process.name))
            except (FileNotFoundError, ProcessLookupError, PermissionError):
                continue
        return members

    def settlement_observed(self, deadline: float) -> bool:
        if not self.exited() or self.session_members():
            return False
        snapshot = host_admission.readonly_snapshot(
            self.allocation.parent, deadline=deadline
        )
        record = snapshot["owners"].get(self.allocation.name)
        if not self.borrowed_parent:
            return record is None or self.unit not in record["units"]
        if record is None or self.unit not in record["units"]:
            raise ValueError(
                "borrowed parent allocation disappeared before command drain"
            )
        parent = host_admission.inherit(os.environ, deadline=deadline)
        if (
            parent is None
            or parent.directory / parent.nonce != self.allocation
            or record["units"].get(self.unit) != self.bound
            or process_observation.group_identity(self.bound["group"])
            != self.bound["inode"]
        ):
            raise ValueError("borrowed parent allocation changed during session drain")
        # The outer pse-env owner must separately prove scope drain and release
        # when the whole test command finishes. This child cannot release it.
        return True

    def _allocation_record(self) -> host_admission.AllocationRecord:
        record = host_admission.readonly_snapshot(
            self.allocation.parent, deadline=time.monotonic() + 5
        )["owners"].get(self.allocation.name)
        if record is None:
            raise ValueError(
                "original native read allocation was released before physical drain"
            )
        return record

    def require_charged(self) -> None:
        manager = operation_lifetime.unit_observation(
            self.unit, deadline=time.monotonic() + 5
        )
        current = process_observation.observe_process(
            self.pid, unit=self.unit, manager=manager
        )
        original = self.observed
        if current.availability is not process_observation.Availability.PRESENT or (
            current.boot,
            current.start,
            current.namespace,
            current.executable,
            current.group,
            current.invocation,
        ) != (
            original.boot,
            original.start,
            original.namespace,
            original.executable,
            original.group,
            original.invocation,
        ):
            raise ValueError("authenticated read worker lifetime changed")
        record = self._allocation_record()
        if record["units"].get(
            self.unit
        ) != self.bound or not process_observation.populated(self.bound["group"]):
            raise ValueError(
                "original read allocation/unit was released before physical drain"
            )

    async def stop(self) -> None:
        self.require_charged()
        self.stopped = True
        signal.pidfd_send_signal(self.descriptor, signal.SIGSTOP)

        async def stopped() -> None:
            while process_observation.observe_process(self.pid).state not in {"T", "t"}:
                await asyncio.sleep(0.01)

        await asyncio.wait_for(stopped(), timeout=5)
        self.require_charged()

    def queued_bytes(self) -> int:
        if not self.stopped:
            raise ValueError(
                "queued pipe observation requires the authenticated stopped worker"
            )
        descriptor = os.open(
            f"/proc/{self.pid}/fd/0", os.O_RDONLY | os.O_NONBLOCK | os.O_CLOEXEC
        )
        try:
            if not stat.S_ISFIFO(os.fstat(descriptor).st_mode):
                raise ValueError(
                    "actual native request input is not an observable pipe"
                )
            result = fcntl.ioctl(descriptor, termios.FIONREAD, struct.pack("I", 0))
            return struct.unpack("I", result)[0]
        finally:
            os.close(descriptor)

    def resume(self) -> None:
        if self.stopped:
            with suppress(ProcessLookupError):
                signal.pidfd_send_signal(self.descriptor, signal.SIGCONT)
            self.stopped = False

    def exited(self) -> bool:
        poll = select.poll()
        poll.register(self.descriptor, select.POLLIN)
        return bool(poll.poll(0))

    def close(self) -> None:
        try:
            self.resume()
        finally:
            os.close(self.descriptor)


class PublishedReadGenerations:
    """Actual constructor publication into one test-private receipt; no ordinary alias edits."""

    def __init__(self, root: Path, directory: Path) -> None:
        if root.resolve() != python_extension.ROOT.resolve():
            raise ValueError(
                "generation control must use the selected checkout's constructor"
            )
        self.environment = python_extension.bind_interpreter(dict(os.environ))
        original = python_extension.read_publication("extension", self.environment)
        if original is None or original.mode != "native" or not original.validated:
            raise ValueError(
                "root must select an existing successful native correctness publication"
            )
        compatible = python_extension.currency_check(
            self.environment, mode="native", profile=original.profile, validated=True
        )
        if compatible.state != "current":
            raise ValueError(
                "selected native publication is incompatible with the acceptance interpreter"
            )
        self.source = Path(original.artifact.binary)
        self.original = original
        self.receipt = directory / "native-read-publication.json"
        self.environment["_PSE_NATIVE_PUBLICATION"] = str(self.receipt)

    def publish(self) -> artifact_publication.Publication:
        return artifact_publication.publish(
            self.receipt,
            kind="extension",
            source=self.source,
            mode="native",
            profile=self.original.profile,
            validated=True,
            interpreter=str(python_extension.interpreter(self.environment).absolute()),
            features=lambda candidate: python_extension._capture_candidate_features(  # noqa: SLF001 -- actual constructor feature observation
                self.environment,
                candidate,
                validated=True,
                native=True,
            ),
        )

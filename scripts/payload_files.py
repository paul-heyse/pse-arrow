# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private file-tier placement and exact Linux process-death observations."""

from __future__ import annotations

import fcntl
import hashlib
import os
import re
import stat
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping


class PayloadError(ValueError):
    """File closure or process ownership could not be established."""


def incarnation(value: object) -> str:
    if not isinstance(value, str):
        raise PayloadError("Missing payload incarnation; explicitly regenerate this profile")
    try:
        parsed = uuid.UUID(value)
    except ValueError as error:
        raise PayloadError("Invalid payload incarnation") from error
    if str(parsed) != value:
        raise PayloadError("Payload incarnation must be a canonical UUID")
    return value


def root(state: Path, selected: Mapping[str, object]) -> Path:
    database = selected.get("database")
    if not isinstance(database, str) or re.fullmatch(r"[A-Za-z0-9_]{1,128}", database) is None:
        raise PayloadError("Invalid payload database placement")
    return state / "payloads" / incarnation(selected.get("payload_incarnation")) / database


def instance_paths(state: Path, selected: Mapping[str, object], key: object) -> tuple[Path, Path]:
    identity = incarnation(key)
    directory = root(state, selected)
    return directory / "instances" / (identity + ".sealed"), directory / ".pending" / (identity + ".pending")


def checked_file(path: Path) -> tuple[str, int]:
    """Hash one immutable regular file with bounded working memory and stable identity."""
    if any(part.is_symlink() for part in (path, *path.parents)):
        raise PayloadError("Payload path must not traverse symlinks")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(descriptor, "rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid() or before.st_nlink != 1:
            raise PayloadError("Payload file lost its private regular-file identity")
        digest = hashlib.sha256()
        while block := stream.read(1024 * 1024):
            digest.update(block)
        after = os.fstat(stream.fileno())
        def identity(value: os.stat_result) -> tuple[int, ...]:
            return (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_nlink, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
        if identity(before) != identity(after) or identity(path.lstat()) != identity(after):
            raise PayloadError("Payload file changed during inventory")
        return digest.hexdigest(), before.st_size


def recognized_locks(directory: Path, instances: set[str]) -> set[Path]:
    """Validate persistent operational locks without copying or replacing their inodes."""
    locks = directory / ".locks"
    if any(path.is_symlink() for path in (locks, *locks.parents)):
        raise PayloadError("Payload lock directory must not traverse symlinks")
    try:
        observed = locks.stat()
    except FileNotFoundError:
        return set()
    if not stat.S_ISDIR(observed.st_mode) or observed.st_uid != os.getuid():
        raise PayloadError("Payload lock directory lost its private ownership")
    expected = {locks / (incarnation(instance) + ".lock") for instance in instances}
    recognized: set[Path] = set()
    for path in locks.iterdir():
        if path not in expected or path.is_symlink():
            raise PayloadError("Unknown payload instance lock retained; inventory refused")
        initial = path.lstat()
        if not stat.S_ISREG(initial.st_mode):
            raise PayloadError("Payload instance lock must be a private empty regular file")
        descriptor = os.open(path, os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
        try:
            before = os.fstat(descriptor)
            if not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid() or stat.S_IMODE(before.st_mode) != 0o600 or before.st_nlink != 1 or before.st_size != 0:
                raise PayloadError("Payload instance lock must be a private empty regular file")
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise PayloadError("Payload instance lock still has a live operational borrower") from error
            after = path.lstat()
            if (before.st_dev, before.st_ino, before.st_mode, before.st_uid, before.st_nlink, before.st_size) != (after.st_dev, after.st_ino, after.st_mode, after.st_uid, after.st_nlink, after.st_size):
                raise PayloadError("Payload instance lock changed during validation")
            recognized.add(path)
        finally:
            # Closing releases only this observer's flock; the persistent inode
            # remains authoritative throughout its live storage incarnation.
            os.close(descriptor)
    return recognized


def owner_dead(owner: Mapping[str, object]) -> bool:
    """Only exact observable death is positive; inaccessible and foreign namespaces refuse."""
    required = {"boot", "pid_namespace", "pid", "start"}
    if not required <= owner.keys() or not all(isinstance(owner[key], str) for key in ("boot", "pid_namespace", "start")):
        raise PayloadError("Incomplete file owner identity")
    raw_pid = owner["pid"]
    if isinstance(raw_pid, bool) or not isinstance(raw_pid, (str, int)):
        raise PayloadError("Invalid file owner PID")
    try:
        pid = int(raw_pid)
    except ValueError as error:
        raise PayloadError("Invalid file owner PID") from error
    if pid < 1 or str(pid) != str(raw_pid):
        raise PayloadError("Invalid file owner PID")
    boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
    if boot != owner["boot"]:
        return True
    namespace = os.readlink("/proc/self/ns/pid")
    if namespace != owner["pid_namespace"]:
        raise PayloadError("File owner PID namespace cannot be authenticated")
    process = Path("/proc") / str(pid)
    try:
        actual_namespace = os.readlink(process / "ns/pid")
        actual_start = (process / "stat").read_text().rsplit(")", 1)[1].split()[19]
    except FileNotFoundError:
        # A missing stat with an extant process directory is a racing or unknown observation.
        try:
            process.stat()
        except FileNotFoundError:
            return True
        raise PayloadError("File owner identity observation is uncertain") from None
    if actual_namespace != namespace:
        raise PayloadError("File owner namespace changed during observation")
    return actual_start != owner["start"]

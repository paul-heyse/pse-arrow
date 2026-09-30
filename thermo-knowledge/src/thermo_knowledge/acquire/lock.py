# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`sources.lock`: what actually resolved, committed as JSON with sorted keys."""

from __future__ import annotations

import fcntl
import os
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

import msgspec
from msgspec import Struct

from thermo_knowledge.acquire.errors import AcquireError

LOCK_VERSION = 1


class LockEntry(Struct, forbid_unknown_fields=True, omit_defaults=True):
    """What one source resolved to.

    `resolved` is the full commit or checksum behind `pin`; `path` (kind `local`) and `reason`
    (kind `none`) are present for those kinds only.
    """

    kind: str
    pin: str | None
    retrieved: str
    file_count: int
    total_bytes: int
    tree_hash: str | None
    resolved: str | None = None
    path: str | None = None
    reason: str | None = None


class Lock(Struct, forbid_unknown_fields=True):
    sources: dict[str, LockEntry]
    version: int


def read_lock(path: Path) -> dict[str, LockEntry]:
    """The entries of the lock; an absent lock has none."""
    try:
        data = path.read_bytes()
    except FileNotFoundError:
        return {}
    try:
        lock = msgspec.json.decode(data, type=Lock)
    except (msgspec.DecodeError, msgspec.ValidationError) as error:
        raise AcquireError(f"{path}: invalid lock: {error}") from error
    if lock.version != LOCK_VERSION:
        raise AcquireError(f"{path}: lock version {lock.version} is not {LOCK_VERSION}")
    return lock.sources


def encode_lock(entries: dict[str, LockEntry]) -> bytes:
    """JSON with sorted keys and a trailing newline."""
    lock = Lock(sources=entries, version=LOCK_VERSION)
    return msgspec.json.format(msgspec.json.encode(lock, order="sorted"), indent=2) + b"\n"


def write_lock(path: Path, entries: dict[str, LockEntry]) -> None:
    """Replace the lock atomically: write a temporary file beside it, then rename it over the
    lock, so no reader ever sees a partial file."""
    temporary = path.with_name(f"{path.name}.{os.getpid()}.tmp")
    try:
        with temporary.open("wb") as handle:
            handle.write(encode_lock(entries))
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def lock_guard_path(path: Path) -> Path:
    """The sidecar file whose `flock` serialises writers of the lock at `path`."""
    return path.with_name(f"{path.name}.flock")


@contextmanager
def exclusive(path: Path) -> Iterator[None]:
    """Hold the exclusive lock on the sidecar of the lock at `path`, waiting for other
    processes that hold it. The lock is released when the block ends or the process dies."""
    descriptor = os.open(lock_guard_path(path), os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o644)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        yield
    finally:
        os.close(descriptor)


def record_entry(path: Path, source_id: str, entry: LockEntry) -> None:
    """Set one source's entry: under the exclusive lock, re-read the current file, replace only
    this source's entry and write the result atomically. Entries other runs recorded since this
    process last looked are kept."""
    with exclusive(path):
        entries = read_lock(path)
        entries[source_id] = entry
        write_lock(path, entries)

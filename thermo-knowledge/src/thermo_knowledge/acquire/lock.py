# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`sources.lock`: what actually resolved, committed as JSON with sorted keys."""

from __future__ import annotations

import os
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
    """Replace the lock atomically."""
    temporary = path.with_name(f"{path.name}.tmp")
    temporary.write_bytes(encode_lock(entries))
    os.replace(temporary, path)

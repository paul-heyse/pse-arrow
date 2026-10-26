# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Copy an exact stopped metadata image without leaving named partial files."""

from __future__ import annotations

import hashlib
import os
import stat
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping


class ImageError(ValueError):
    """An original source image or reserved destination coordinate changed."""


def _real(path: Path) -> None:
    if not path.is_absolute() or any(part.is_symlink() for part in (path, *path.parents)):
        raise ImageError("Backup image paths must be absolute and must not traverse symlinks")


def _identity(value: os.stat_result) -> tuple[int, ...]:
    return (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_size,
            value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)


def _owned(value: os.stat_result) -> None:
    # The enclosing service/backup directory supplies privacy. Database engines
    # may retain 0644 metadata; preserve that original mode inside the private root.
    if not stat.S_ISREG(value.st_mode) or value.st_uid != os.getuid():
        raise ImageError("Backup image must remain an owned regular file")


def _private_directory(path: Path) -> tuple[int, os.stat_result]:
    """Create every missing component privately, relative to held owned parents."""
    missing: list[str] = []
    ancestor = path
    while not ancestor.exists():
        missing.append(ancestor.name)
        ancestor = ancestor.parent
    _real(ancestor)
    directory = os.open(ancestor, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        for name in [*reversed(missing), None]:
            information = os.fstat(directory)
            if information.st_uid != os.getuid() or information.st_mode & 0o077:
                raise ImageError("Backup destination directory must remain private and owned")
            if name is None:
                _real(path)
                named = path.stat()
                if (information.st_dev, information.st_ino) != (named.st_dev, named.st_ino):
                    raise ImageError("Backup destination directory changed during creation")
                return directory, information
            try:
                os.mkdir(name, mode=0o700, dir_fd=directory)
            except FileExistsError:
                # A concurrent name is never repaired or granted weaker ownership.
                pass
            child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                            dir_fd=directory)
            try:
                os.fsync(directory)
            except BaseException:
                os.close(child)
                raise
            os.close(directory)
            directory = child
    except BaseException:
        os.close(directory)
        raise
    raise ImageError("Backup destination directory was not established")


def digest(path: Path) -> str:
    _real(path)
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    with os.fdopen(descriptor, "rb") as source:
        before = os.fstat(descriptor)
        _owned(before)
        result = hashlib.file_digest(source, "sha256").hexdigest()
        if _identity(before) != _identity(os.fstat(descriptor)) or _identity(before) != _identity(path.stat()):
            raise ImageError("Backup source image changed during verification")
        return result


def files(root: Path, selected: Iterable[Path]) -> dict[str, str]:
    result: dict[str, str] = {}
    for original in selected:
        _real(original)
        for path in sorted(original.rglob("*")) if original.is_dir() else (original,):
            _real(path)
            if path.is_file():
                relative = path.relative_to(root).as_posix()
                result[relative] = digest(path)
            elif not path.is_dir():
                raise ImageError("Backup source contains an unsupported physical entry")
    return result


def check_destination(root: Path, files: Mapping[str, str], manifest_sha256: str) -> None:
    _real(root)
    for path in root.rglob("*"):
        _real(path)
        if path.is_dir():
            continue
        relative = path.relative_to(root)
        if relative.parts[0] == "payloads":
            # Native complete-tree and exact-inventory checks own physical payloads.
            continue
        expected = manifest_sha256 if relative.as_posix() == "backup.json" else files.get(relative.as_posix())
        if expected is None or path.stat().st_nlink != 1 or digest(path) != expected:
            raise ImageError("Unknown or changed interrupted backup content is retained")


def copy(source: Path, target: Path, expected: str) -> None:
    """Reverify existing exact outputs or atomically publish a new complete inode."""
    _real(source)
    _real(target)
    directory, parent = _private_directory(target.parent)
    try:
        exists = target.exists()
    except BaseException:
        os.close(directory)
        raise
    if exists:
        try:
            if (target.stat().st_nlink != 1 or digest(target) != expected
                    or stat.S_IMODE(target.stat().st_mode) != stat.S_IMODE(source.stat().st_mode)):
                raise ImageError("Existing backup image differs from the original snapshot")
            _real(target.parent)
            if (parent.st_dev, parent.st_ino) != (target.parent.stat().st_dev, target.parent.stat().st_ino):
                raise ImageError("Backup destination directory changed during resume")
            return
        finally:
            os.close(directory)
    try:
        source_fd = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    except BaseException:
        os.close(directory)
        raise
    try:
        before = os.fstat(source_fd)
        _owned(before)
        # O_EXCL deliberately stays absent: only linkat on this held anonymous
        # descriptor may publish its final name after complete byte verification.
        output_fd = os.open(target.parent, os.O_TMPFILE | os.O_RDWR | os.O_CLOEXEC, 0o600)
        with os.fdopen(output_fd, "w+b") as output, os.fdopen(source_fd, "rb", closefd=False) as original:
            observed = hashlib.sha256()
            while block := original.read(1024 * 1024):
                output.write(block)
                observed.update(block)
            if observed.hexdigest() != expected or _identity(before) != _identity(os.fstat(source_fd)):
                raise ImageError("Stopped backup source differs from its exact snapshot")
            output.flush()
            os.fchmod(output_fd, stat.S_IMODE(before.st_mode))
            os.fsync(output_fd)
            output.seek(0)
            if hashlib.file_digest(output, "sha256").hexdigest() != expected:
                raise ImageError("Unpublished backup image differs from its original source")
            _real(target.parent)
            if (parent.st_dev, parent.st_ino) != (target.parent.stat().st_dev, target.parent.stat().st_ino):
                raise ImageError("Backup destination directory changed before publication")
            os.link(f"/proc/self/fd/{output_fd}", target.name, dst_dir_fd=directory, follow_symlinks=True)
            os.fsync(output_fd)
            os.fsync(directory)
        if digest(target) != expected:
            raise ImageError("Published backup image differs from its original source")
        if _identity(before) != _identity(os.fstat(source_fd)) or _identity(before) != _identity(source.stat()):
            raise ImageError("Backup source identity changed during publication")
    finally:
        os.close(source_fd)
        os.close(directory)

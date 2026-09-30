# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The raw store: layout, file inventory, tree hash and `ACQUISITION.json`.

```
<raw>/<id>/<resolved-pin>/tree/             the acquired files, untouched
<raw>/<id>/<resolved-pin>/ACQUISITION.json  pin, retrieval time, tool versions, every file
<raw>/<id>/.partial/                        work in progress; never a pin
```

A pin directory is written once: the acquisition is built in `.partial` and renamed into place
when complete, so an interrupted run leaves nothing that looks complete.
"""

from __future__ import annotations

import hashlib
import os
import shutil
from dataclasses import dataclass
from pathlib import Path

import msgspec
from msgspec import Struct

from thermo_knowledge.acquire.errors import AcquireError

TREE_DIR_NAME = "tree"
ACQUISITION_NAME = "ACQUISITION.json"
PARTIAL_NAME = ".partial"
PIN_LENGTH = 12
SCHEMA_VERSION = 1
_CHUNK = 1024 * 1024


@dataclass(frozen=True)
class FileEntry:
    """One file of a tree. A symbolic link is an entry whose hash is that of its target text."""

    path: str
    size: int
    sha256: str
    link: str | None = None


class AcquisitionFile(Struct, forbid_unknown_fields=True):
    """One file of `ACQUISITION.json`."""

    path: str
    size: int
    sha256: str
    link: str | None = None


class Acquisition(Struct, forbid_unknown_fields=True):
    """The content of `ACQUISITION.json`."""

    schema: int
    id: str
    kind: str
    pin: str
    resolved: str | None
    retrieved: str
    tool_versions: dict[str, str]
    urls: list[str]
    details: dict[str, str]
    tree_hash: str
    file_count: int
    total_bytes: int
    files: list[AcquisitionFile]


def source_dir(raw_dir: Path, source_id: str) -> Path:
    return raw_dir / source_id


def pin_dir(raw_dir: Path, source_id: str, pin: str) -> Path:
    return raw_dir / source_id / pin


def partial_dir(raw_dir: Path, source_id: str) -> Path:
    return raw_dir / source_id / PARTIAL_NAME


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(_CHUNK):
            digest.update(chunk)
    return digest.hexdigest()


def scan_tree(root: Path) -> list[FileEntry]:
    """Every file under `root`, sorted by relative POSIX path; links are not followed."""
    entries: list[FileEntry] = []

    def walk(directory: Path, prefix: str) -> None:
        with os.scandir(directory) as scanner:
            children = sorted(scanner, key=lambda child: child.name)
        for child in children:
            relative = f"{prefix}{child.name}"
            if child.is_symlink():
                target = os.readlink(child.path)
                data = target.encode("utf-8", "surrogateescape")
                entries.append(
                    FileEntry(relative, len(data), hashlib.sha256(data).hexdigest(), link=target)
                )
            elif child.is_dir(follow_symlinks=False):
                walk(Path(child.path), f"{relative}/")
            elif child.is_file(follow_symlinks=False):
                size = child.stat(follow_symlinks=False).st_size
                entries.append(FileEntry(relative, size, sha256_file(Path(child.path))))

    if root.is_dir():
        walk(root, "")
    entries.sort(key=lambda entry: entry.path.encode("utf-8", "surrogateescape"))
    return entries


def tree_hash(entries: list[FileEntry]) -> str:
    """SHA-256 over the lines `<sha256>  <relative path>\\n`, sorted by path."""
    digest = hashlib.sha256()
    for entry in sorted(entries, key=lambda e: e.path.encode("utf-8", "surrogateescape")):
        digest.update(f"{entry.sha256}  {entry.path}\n".encode("utf-8", "surrogateescape"))
    return digest.hexdigest()


def total_bytes(entries: list[FileEntry]) -> int:
    return sum(entry.size for entry in entries)


@dataclass(frozen=True)
class TreeDifference:
    """Files of a tree compared with the inventory recorded at acquisition."""

    missing: list[str]
    extra: list[str]
    changed: list[str]

    @property
    def empty(self) -> bool:
        return not (self.missing or self.extra or self.changed)

    def lines(self) -> list[str]:
        return (
            [f"missing file: {path}" for path in self.missing]
            + [f"extra file: {path}" for path in self.extra]
            + [f"changed file: {path}" for path in self.changed]
        )


def compare_tree(recorded: list[AcquisitionFile], actual: list[FileEntry]) -> TreeDifference:
    expected = {entry.path: entry for entry in recorded}
    found = {entry.path: entry for entry in actual}
    return TreeDifference(
        missing=sorted(set(expected) - set(found)),
        extra=sorted(set(found) - set(expected)),
        changed=sorted(
            path
            for path in set(expected) & set(found)
            if (expected[path].sha256, expected[path].size)
            != (found[path].sha256, found[path].size)
        ),
    )


def build_acquisition(
    *,
    source_id: str,
    kind: str,
    pin: str,
    resolved: str | None,
    retrieved: str,
    tool_versions: dict[str, str],
    urls: list[str],
    details: dict[str, str],
    entries: list[FileEntry],
) -> Acquisition:
    return Acquisition(
        schema=SCHEMA_VERSION,
        id=source_id,
        kind=kind,
        pin=pin,
        resolved=resolved,
        retrieved=retrieved,
        tool_versions=tool_versions,
        urls=urls,
        details=details,
        tree_hash=tree_hash(entries),
        file_count=len(entries),
        total_bytes=total_bytes(entries),
        files=[AcquisitionFile(e.path, e.size, e.sha256, e.link) for e in entries],
    )


def write_acquisition(directory: Path, acquisition: Acquisition) -> None:
    payload = msgspec.json.format(msgspec.json.encode(acquisition, order="sorted"), indent=2)
    (directory / ACQUISITION_NAME).write_bytes(payload + b"\n")


def read_acquisition(directory: Path) -> Acquisition:
    """`ACQUISITION.json` of a pin directory; raises `AcquireError` when absent or invalid."""
    path = directory / ACQUISITION_NAME
    try:
        return msgspec.json.decode(path.read_bytes(), type=Acquisition)
    except FileNotFoundError as error:
        raise AcquireError(f"{path}: missing") from error
    except (msgspec.DecodeError, msgspec.ValidationError) as error:
        raise AcquireError(f"{path}: invalid: {error}") from error


def finalize(work: Path, destination: Path) -> None:
    """Rename the completed `work` directory to `destination`; never writes into an existing one."""
    if destination.exists():
        raise AcquireError(
            f"{destination}: refusing to write into an existing resolved-pin directory; "
            "an existing acquisition is never modified in place"
        )
    work.rename(destination)


def remove_tree(path: Path) -> None:
    shutil.rmtree(path, ignore_errors=True)


def prune_empty(directory: Path) -> None:
    """Remove `directory` when nothing is left in it."""
    try:
        directory.rmdir()
    except OSError:
        pass

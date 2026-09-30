# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kinds `archive` and `file`: verified downloads.

A download is streamed to a temporary file in the work directory and its published checksum is
verified before anything is moved into the tree; the work directory itself only becomes a pin
directory when the whole acquisition completed.
"""

from __future__ import annotations

import hashlib
import re
import tarfile
import zipfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urlsplit

import httpx

from thermo_knowledge import __version__
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.errors import AcquireError
from thermo_knowledge.acquire.manifest import ArchiveSpec, FileItem, FileSpec
from thermo_knowledge.acquire.outcome import Outcome
from thermo_knowledge.acquire.runtime import Runtime
from thermo_knowledge.acquire.store import PIN_LENGTH, TREE_DIR_NAME

EXTRACTED_DIR_NAME = "extracted"
"""Where `extract = true` puts the archive's members, inside the tree next to the archive."""

_CHUNK = 1024 * 256
_SCHEMES = {"http", "https"}


@dataclass(frozen=True)
class Downloaded:
    sha256: str
    md5: str
    size: int
    final_url: str


def download(runtime: Runtime, source_id: str, url: str, destination: Path) -> Downloaded:
    """Stream `url` (following redirects) into `destination`, hashing on the way."""
    if urlsplit(url).scheme not in _SCHEMES:
        raise AcquireError(f"{source_id}: {url}: only http and https URLs can be downloaded")
    sha256 = hashlib.sha256()
    md5 = hashlib.md5(usedforsecurity=False)
    size = 0
    try:
        with (
            runtime.client_factory() as client,
            client.stream("GET", url, follow_redirects=True) as response,
        ):
            if response.status_code != 200:
                raise AcquireError(
                    f"{source_id}: {url}: expected HTTP 200, the server answered "
                    f"{response.status_code} {response.reason_phrase}"
                )
            with destination.open("wb") as handle:
                for chunk in response.iter_bytes(_CHUNK):
                    handle.write(chunk)
                    sha256.update(chunk)
                    md5.update(chunk)
                    size += len(chunk)
            final_url = str(response.url)
    except httpx.HTTPError as error:
        raise AcquireError(f"{source_id}: {url}: download failed: {error!r}") from error
    return Downloaded(sha256.hexdigest(), md5.hexdigest(), size, final_url)


def _verify(
    source_id: str, url: str, got: Downloaded, *, sha256: str | None, md5: str | None
) -> None:
    for algorithm, expected, found in (("sha256", sha256, got.sha256), ("md5", md5, got.md5)):
        if expected is not None and expected != found:
            raise AcquireError(
                f"{source_id}: {url}: the published {algorithm} is {expected} but the "
                f"delivered bytes have {algorithm} {found}; nothing was stored"
            )


def _archive_name(url: str) -> str:
    name = PurePosixPath(unquote(urlsplit(url).path)).name
    return name if name and name not in {".", ".."} else "archive"


def _check_member(source_id: str, archive: str, name: str) -> None:
    portable = name.replace("\\", "/")
    path = PurePosixPath(portable)
    if path.is_absolute() or ".." in path.parts or re.match(r"^[A-Za-z]:", portable):
        raise AcquireError(
            f"{source_id}: {archive}: refusing archive member {name!r}: it would escape the "
            "target directory"
        )


def extract_archive(source_id: str, archive: Path, target: Path) -> None:
    """Extract a tar or zip archive into `target`, refusing members that escape it."""
    target.mkdir()
    if tarfile.is_tarfile(archive):
        try:
            with tarfile.open(archive) as bundle:
                members = bundle.getmembers()
                for member in members:
                    _check_member(source_id, archive.name, member.name)
                    if member.issym() or member.islnk():
                        _check_member(source_id, archive.name, member.linkname)
                bundle.extractall(target, members=members, filter="data")
        except tarfile.FilterError as error:
            raise AcquireError(
                f"{source_id}: {archive.name}: refusing archive member "
                f"{getattr(error.tarinfo, 'name', '?')!r}: {error}"
            ) from error
        except (tarfile.TarError, OSError) as error:
            raise AcquireError(f"{source_id}: {archive.name}: cannot extract: {error}") from error
    elif zipfile.is_zipfile(archive):
        try:
            with zipfile.ZipFile(archive) as bundle:
                for name in bundle.namelist():
                    _check_member(source_id, archive.name, name)
                bundle.extractall(target)
        except (zipfile.BadZipFile, OSError) as error:
            raise AcquireError(f"{source_id}: {archive.name}: cannot extract: {error}") from error
    else:
        raise AcquireError(
            f"{source_id}: {archive.name}: `extract = true` but the file is neither a tar nor a "
            "zip archive"
        )


def _tools() -> dict[str, str]:
    return {"httpx": httpx.__version__, "thermo-knowledge": __version__}


def acquire_archive(runtime: Runtime, source_id: str, spec: ArchiveSpec, work: Path) -> Outcome:
    """Build `<work>/tree/` from one archive: the file as delivered and, optionally, its members."""
    tree = work / TREE_DIR_NAME
    tree.mkdir()
    name = _archive_name(spec.url)
    partial = work / "download"
    got = download(runtime, source_id, spec.url, partial)
    _verify(source_id, spec.url, got, sha256=spec.sha256, md5=spec.md5)
    partial.rename(tree / name)
    if spec.extract:
        extract_archive(source_id, tree / name, tree / EXTRACTED_DIR_NAME)
    declared = spec.sha256 or spec.md5
    assert declared is not None
    details = {"archive": name, "sha256": got.sha256, "final_url": got.final_url}
    if spec.md5 is not None:
        details["md5"] = got.md5
    if spec.doi is not None:
        details["doi"] = spec.doi
    return Outcome(
        pin=declared[:PIN_LENGTH],
        resolved=got.sha256,
        urls=[spec.url],
        tool_versions=_tools(),
        details=details,
    )


def file_pin(items: list[FileItem]) -> str | None:
    """The pin known from the manifest alone, or `None` until the files are downloaded.

    One file: the prefix of its declared checksum. Several: the prefix of the tree hash, known
    in advance only when every file declares a `sha256`.
    """
    if len(items) == 1:
        declared = items[0].sha256 or items[0].md5
        return declared[:PIN_LENGTH] if declared else None
    if all(item.sha256 for item in items):
        entries = [store.FileEntry(item.name, 0, item.sha256 or "") for item in items]
        return store.tree_hash(entries)[:PIN_LENGTH]
    return None


def acquire_files(runtime: Runtime, source_id: str, spec: FileSpec, work: Path) -> Outcome:
    """Build `<work>/tree/` from each file as delivered."""
    tree = work / TREE_DIR_NAME
    tree.mkdir()
    entries: list[store.FileEntry] = []
    details: dict[str, str] = {}
    for item in spec.files:
        partial = work / "download"
        got = download(runtime, source_id, item.url, partial)
        _verify(source_id, item.url, got, sha256=item.sha256, md5=item.md5)
        partial.rename(tree / item.name)
        entries.append(store.FileEntry(item.name, got.size, got.sha256))
        details[f"file {item.name}"] = f"sha256 {got.sha256}; final url {got.final_url}"
    if len(spec.files) == 1:
        item = spec.files[0]
        pin_source = item.sha256 or item.md5 or entries[0].sha256
        resolved = entries[0].sha256
    else:
        pin_source = store.tree_hash(entries)
        resolved = pin_source
    return Outcome(
        pin=pin_source[:PIN_LENGTH],
        resolved=resolved,
        urls=[item.url for item in spec.files],
        tool_versions=_tools(),
        details=details,
    )

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Source manifests: typed declarations decoded from `sources/<id>.toml`.

Unknown keys are refused at every level. Problems are reported with the file and the key path.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path, PurePosixPath
from typing import Annotated, Literal

import msgspec
from msgspec import Meta, Struct

from thermo_knowledge import config
from thermo_knowledge.acquire.errors import ManifestError

ID_PATTERN = re.compile(r"^[a-z][a-z0-9_]*$")

NonEmpty = Annotated[str, Meta(min_length=1)]
Commit = Annotated[str, Meta(pattern=r"^[0-9a-f]{40}$")]
Sha256 = Annotated[str, Meta(pattern=r"^[0-9a-fA-F]{64}$")]
Md5 = Annotated[str, Meta(pattern=r"^[0-9a-fA-F]{32}$")]

Tier = Literal["A", "B", "held"]
Basis = Literal["licence_grant", "public_domain", "permission", "terms_of_use", "not_stated"]
Grant = Literal["yes", "no", "not_stated"]
Conditional = Literal["yes", "no", "not_stated", "with_conditions"]

_STRICT = {"forbid_unknown_fields": True}


def _relative_path(value: str, what: str) -> None:
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or re.match(r"^[A-Za-z]:", value):
        raise ValueError(f"{what} {value!r} must be a relative path without '..'")


class GitSpec(Struct, tag_field="kind", tag="git", **_STRICT):
    """A checkout of exactly one commit, without history."""

    url: NonEmpty
    commit: Commit
    tag: NonEmpty | None = None
    submodules: list[NonEmpty] = []
    sparse: list[NonEmpty] = []

    def __post_init__(self) -> None:
        for path in self.submodules:
            _relative_path(path, "submodule path")
        for pattern in self.sparse:
            _relative_path(pattern, "sparse path")


class ArchiveSpec(Struct, tag_field="kind", tag="archive", **_STRICT):
    """One downloaded archive, verified against the publisher's checksum."""

    url: NonEmpty
    sha256: Sha256 | None = None
    md5: Md5 | None = None
    doi: NonEmpty | None = None
    extract: bool = False

    def __post_init__(self) -> None:
        if (self.sha256 is None) == (self.md5 is None):
            raise ValueError(
                "exactly one of `sha256` and `md5` (the publisher's checksum) is required"
            )
        if self.sha256 is not None:
            self.sha256 = self.sha256.lower()
        if self.md5 is not None:
            self.md5 = self.md5.lower()


class FileItem(Struct, **_STRICT):
    """One downloaded file, optionally verified against the publisher's checksum."""

    url: NonEmpty
    name: NonEmpty
    sha256: Sha256 | None = None
    md5: Md5 | None = None

    def __post_init__(self) -> None:
        if self.sha256 is not None and self.md5 is not None:
            raise ValueError(f"file {self.name!r}: give at most one of `sha256` and `md5`")
        if "/" in self.name or "\\" in self.name or self.name in {".", ".."}:
            raise ValueError(f"file name {self.name!r} must be a plain file name")
        if self.sha256 is not None:
            self.sha256 = self.sha256.lower()
        if self.md5 is not None:
            self.md5 = self.md5.lower()


class FileSpec(Struct, tag_field="kind", tag="file", **_STRICT):
    """Several downloaded files, each stored as delivered."""

    files: Annotated[list[FileItem], Meta(min_length=1)]

    def __post_init__(self) -> None:
        names = [item.name for item in self.files]
        duplicates = sorted({name for name in names if names.count(name) > 1})
        if duplicates:
            raise ValueError(f"duplicate file names: {', '.join(duplicates)}")


class PagesSpec(Struct, tag_field="kind", tag="pages", **_STRICT):
    """Rate-limited retrieval of pages that have no bulk download.

    `urls` is read by the enumerators that take their entry points from the manifest: `url_list`
    (the pages themselves) and `janaf` and `iapws` (the one index page).
    """

    enumerator: NonEmpty
    min_interval_seconds: Annotated[float, Meta(ge=0)]
    limit: Annotated[int, Meta(ge=1)] | None = None
    urls: list[NonEmpty] = []


class LocalSpec(Struct, tag_field="kind", tag="local", **_STRICT):
    """An existing checkout in the repository, verified and not copied."""

    path: NonEmpty
    commit: Commit | None = None
    tag: NonEmpty | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path, "path")
        if self.commit is None and self.tag is None:
            raise ValueError("one of `commit` and `tag` is required")


class NoneSpec(Struct, tag_field="kind", tag="none", **_STRICT):
    """A source that is recorded and not acquired, with the reason."""

    reason: NonEmpty


AcquireSpec = GitSpec | ArchiveSpec | FileSpec | PagesSpec | LocalSpec | NoneSpec

KINDS: dict[type[Struct], str] = {
    GitSpec: "git",
    ArchiveSpec: "archive",
    FileSpec: "file",
    PagesSpec: "pages",
    LocalSpec: "local",
    NoneSpec: "none",
}


def kind_name(spec: AcquireSpec) -> str:
    return KINDS[type(spec)]


class Payload(Struct, **_STRICT):
    """What the source delivers and which reader handles it (read by later stages)."""

    reader: NonEmpty | None = None
    environment: NonEmpty = "core"
    include: list[str] = []
    exclude: list[str] = []


class Rights(Struct, **_STRICT):
    """One rights determination; competing statements are separate entries."""

    scope: NonEmpty
    basis: Basis
    statement: NonEmpty
    store: Grant
    redistribute: Conditional
    commercial: Conditional
    attribution: bool
    share_alike: bool
    observed: NonEmpty
    spdx: NonEmpty | None = None
    url: NonEmpty | None = None


class Manifest(Struct, **_STRICT):
    """One source declaration."""

    id: NonEmpty
    title: NonEmpty
    tier: Tier
    acquire: AcquireSpec
    rights: Annotated[list[Rights], Meta(min_length=1)]
    homepage: NonEmpty | None = None
    waves: list[int] = []
    notes: str = ""
    payload: Payload = msgspec.field(default_factory=Payload)

    @property
    def kind(self) -> str:
        return kind_name(self.acquire)


def default_sources_dir() -> Path:
    """`sources/` of the tree."""
    return config.TREE_DIR / "sources"


def default_lock_path() -> Path:
    """`sources.lock` of the tree."""
    return config.TREE_DIR / "sources.lock"


_LOCATION = re.compile(r"\s+- at `\$([^`]*)`$")
_UNKNOWN_FIELD = re.compile(r"^Object contains unknown field `([^`]*)`$")


def _describe(error: msgspec.ValidationError) -> str:
    """`key.path: message` from a msgspec validation error."""
    message = str(error)
    location = ""
    match = _LOCATION.search(message)
    if match:
        message = message[: match.start()]
        location = match.group(1).lstrip(".")
    unknown = _UNKNOWN_FIELD.match(message)
    if unknown:
        key = unknown.group(1)
        return f"{location + '.' if location else ''}{key}: unknown key"
    message = message.replace("got `date`", "got a TOML date (quote the value as a string)")
    return f"{location}: {message}" if location else message


def parse_manifest(text: str, *, file: str, stem: str | None = None) -> Manifest:
    """Decode one manifest; raises `ManifestError` naming `file` and the offending key."""
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise ManifestError([f"{file}: invalid TOML: {error}"]) from error

    acquire = data.get("acquire")
    if not isinstance(acquire, dict):
        raise ManifestError([f"{file}: acquire: the [acquire] table is required"])
    kind = acquire.get("kind")
    if kind not in KINDS.values():
        known = ", ".join(sorted(KINDS.values()))
        raise ManifestError(
            [f"{file}: acquire.kind: unknown kind {kind!r}; expected one of {known}"]
        )
    rights = data.get("rights")
    if not isinstance(rights, list) or not rights:
        raise ManifestError(
            [f'{file}: rights: at least one [[rights]] entry is required (even for kind = "none")']
        )

    try:
        manifest = msgspec.convert(data, Manifest, strict=True)
    except msgspec.ValidationError as error:
        raise ManifestError([f"{file}: {_describe(error)}"]) from error

    if not ID_PATTERN.match(manifest.id):
        raise ManifestError(
            [f"{file}: id: {manifest.id!r} is not lowercase snake_case (^[a-z][a-z0-9_]*$)"]
        )
    if stem is not None and manifest.id != stem:
        raise ManifestError([f"{file}: id: {manifest.id!r} must equal the file name {stem!r}"])
    _check_enumerator(manifest, file)
    return manifest


def _check_enumerator(manifest: Manifest, file: str) -> None:
    if not isinstance(manifest.acquire, PagesSpec):
        return
    from thermo_knowledge.acquire.enumerators import ENUMERATORS

    if manifest.acquire.enumerator not in ENUMERATORS:
        known = ", ".join(sorted(ENUMERATORS))
        raise ManifestError(
            [
                f"{file}: acquire.enumerator: unknown enumerator "
                f"{manifest.acquire.enumerator!r}; known: {known}"
            ]
        )


def load_sources(directory: Path | None = None) -> dict[str, Manifest]:
    """Every `<id>.toml` of a sources directory, sorted by id.

    Problems of all files are reported together in one `ManifestError`.
    """
    directory = default_sources_dir() if directory is None else directory
    if not directory.is_dir():
        raise ManifestError([f"{directory}: the sources directory does not exist"])
    manifests: dict[str, Manifest] = {}
    problems: list[str] = []
    for path in sorted(directory.glob("*.toml")):
        try:
            manifest = parse_manifest(
                path.read_text(encoding="utf-8"), file=str(path), stem=path.stem
            )
        except ManifestError as error:
            problems.extend(error.problems)
        except UnicodeDecodeError as error:
            problems.append(f"{path}: not valid UTF-8: {error}")
        else:
            manifests[manifest.id] = manifest
    if problems:
        raise ManifestError(problems)
    return manifests

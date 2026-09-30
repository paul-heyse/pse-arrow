# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What the writer knows of a carrier: enough to create its provenance rows.

A `CarrierInfo` is built from the source manifest, the lock entry and the staged manifest of one
acquisition (`from_source`), or decoded from the phase-1 output of a mapping (`decode`), so that
resolution, which sees many carriers, needs no source tree. The writer turns it into the
`carrier`, `artifact`, `import_record`, `licence` and `rights_determination` rows, and each
`Origin` into an `import_record` row and a `record_origin` row.
"""

from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

import msgspec
from msgspec import Struct

from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import LockEntry
from thermo_knowledge.acquire.manifest import Manifest
from thermo_knowledge.staging.manifest import StagedManifest


class ArtifactInfo(Struct, forbid_unknown_fields=True):
    """One file of a carrier: its content hash (lowercase hexadecimal SHA-256) and size."""

    sha256: str
    size: int


class RightsInfo(Struct, forbid_unknown_fields=True):
    """One `[[rights]]` entry of the source manifest."""

    scope: str
    basis: str
    statement: str
    store: str
    redistribute: str
    commercial: str
    attribution: bool
    share_alike: bool
    observed: str
    spdx: str | None = None
    url: str | None = None


class CarrierInfo(Struct, forbid_unknown_fields=True):
    """One acquisition of one source at one resolved pin, and the files cited so far."""

    manifest_id: str
    title: str
    pin: str
    tree_hash: str
    retrieved: str
    reader: str
    reader_version: str
    rights: list[RightsInfo]
    artifacts: dict[str, ArtifactInfo] = {}

    @property
    def key(self) -> str:
        """The carrier's `source.key`: the manifest id and the resolved pin."""
        return f"{self.manifest_id}@{self.pin}"

    @property
    def retrieved_at(self) -> datetime:
        return datetime.fromisoformat(self.retrieved)


def decode(data: bytes) -> CarrierInfo:
    return msgspec.json.decode(data, type=CarrierInfo)


def encode(carrier: CarrierInfo) -> bytes:
    return msgspec.json.encode(carrier, order="sorted")


def from_source(manifest: Manifest, entry: LockEntry, staged: StagedManifest) -> CarrierInfo:
    """The carrier the staged tables of `manifest` were read from."""
    return CarrierInfo(
        manifest_id=manifest.id,
        title=manifest.title,
        pin=staged.pin,
        tree_hash=entry.tree_hash or staged.tree_hash,
        retrieved=entry.retrieved,
        reader=staged.reader.name,
        reader_version=staged.reader.version,
        rights=[
            RightsInfo(
                scope=item.scope,
                basis=item.basis,
                statement=item.statement,
                store=item.store,
                redistribute=item.redistribute,
                commercial=item.commercial,
                attribution=item.attribution,
                share_alike=item.share_alike,
                observed=item.observed,
                spdx=item.spdx,
                url=item.url,
            )
            for item in manifest.rights
        ],
    )


def artifact_info(tree: Path, path: str) -> ArtifactInfo:
    """Hash and size of one file of the acquired tree (a link counts by its target text)."""
    target = tree / path
    if target.is_symlink():
        data = os.readlink(target).encode("utf-8", "surrogateescape")
        return ArtifactInfo(hashlib.sha256(data).hexdigest(), len(data))
    return ArtifactInfo(store.sha256_file(target), target.stat().st_size)


@dataclass(frozen=True)
class SourceRef:
    """A position in a source-faithful table: the file and the locator of one row."""

    carrier: str  # the manifest id
    artifact: str
    locator: str


@dataclass(frozen=True)
class Origin:
    """A record was produced from the row `ref` of a carrier, which presents it in `role`."""

    ref: SourceRef
    role: str


@dataclass
class Carriers:
    """The carriers a writer may cite, by manifest id, and where to hash a file not yet cited."""

    infos: dict[str, CarrierInfo] = field(default_factory=dict)
    trees: dict[str, Path] = field(default_factory=dict)

    def add(self, carrier: CarrierInfo, tree: Path | None = None) -> None:
        self.infos[carrier.manifest_id] = carrier
        if tree is not None:
            self.trees[carrier.manifest_id] = tree

    def artifact(self, manifest_id: str, path: str) -> ArtifactInfo:
        carrier = self.infos[manifest_id]
        found = carrier.artifacts.get(path)
        if found is None:
            tree = self.trees.get(manifest_id)
            if tree is None:
                raise KeyError(
                    f"the carrier {manifest_id} records no hash for {path} and has no tree to "
                    "hash it from"
                )
            found = artifact_info(tree, path)
            carrier.artifacts[path] = found
        return found

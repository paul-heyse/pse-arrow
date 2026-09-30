# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`manifest.json` of a staged source: what was read, from which inputs, into which tables.

```
<staged>/<id>/<resolved-pin>/manifest.json
<staged>/<id>/<resolved-pin>/<table>.parquet
```
"""

from __future__ import annotations

from pathlib import Path
from typing import Literal

import msgspec
from msgspec import Struct

from thermo_knowledge.staging.errors import StagingError

MANIFEST_NAME = "manifest.json"
MANIFEST_SCHEMA = 1
FileStatus = Literal["read", "partly_read", "skipped"]


class TableRecord(Struct, forbid_unknown_fields=True):
    """One staged table: its Parquet file, row count, schema fingerprint and content hash (the
    SHA-256 of the Parquet file)."""

    file: str
    rows: int
    schema_fingerprint: str
    content_hash: str


class FileRecord(Struct, forbid_unknown_fields=True, omit_defaults=True):
    """One payload file and how it was handled; a reason accompanies `partly_read` and
    `skipped`."""

    path: str
    status: FileStatus
    reason: str | None = None


class ReaderRecord(Struct, forbid_unknown_fields=True):
    name: str
    version: str
    environment: str
    source_hash: str


class StagedManifest(Struct, forbid_unknown_fields=True):
    schema: int
    source_id: str
    pin: str
    reader: ReaderRecord
    reuse_key: str
    tree_hash: str
    tables: dict[str, TableRecord]
    payload: list[FileRecord]


def encode(manifest: StagedManifest) -> bytes:
    """JSON with sorted keys and a trailing newline."""
    return msgspec.json.format(msgspec.json.encode(manifest, order="sorted"), indent=2) + b"\n"


def write(directory: Path, manifest: StagedManifest) -> None:
    (directory / MANIFEST_NAME).write_bytes(encode(manifest))


def read(directory: Path) -> StagedManifest:
    """The manifest of a staged directory; raises `StagingError` when absent or invalid."""
    path = directory / MANIFEST_NAME
    try:
        manifest = msgspec.json.decode(path.read_bytes(), type=StagedManifest)
    except FileNotFoundError as error:
        raise StagingError(f"{path}: missing") from error
    except (msgspec.DecodeError, msgspec.ValidationError) as error:
        raise StagingError(f"{path}: invalid: {error}") from error
    if manifest.schema != MANIFEST_SCHEMA:
        raise StagingError(f"{path}: manifest schema {manifest.schema} is not {MANIFEST_SCHEMA}")
    return manifest

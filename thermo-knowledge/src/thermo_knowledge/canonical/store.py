# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Canonical Parquet on disk, and its manifest.

```
<canonical>/<id>/<schema>.<table>.parquet     phase 2 of a mapping
<canonical>/<id>/manifest.json
<canonical>/<id>/_identity/                   phase 1 of a mapping (mapping/identity_claims.py)
<canonical>/_resolution/<schema>.<table>.parquet
<canonical>/_resolution/manifest.json
```

A directory is built in a temporary sibling and renamed into place when complete, so a failed run
leaves nothing that looks finished. The manifest's `reuse_key` hashes the run's complete inputs;
a directory whose recorded key equals the current key is reused. Files are Parquet with zstd
compression and no timestamp, so the same tables are the same bytes.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import tempfile
from collections.abc import Mapping
from pathlib import Path

import msgspec
import pyarrow as pa
import pyarrow.parquet as pq
from msgspec import Struct

from thermo_knowledge.canonical.provenance import CarrierInfo
from thermo_knowledge.staging import schema as staging_schema
from thermo_knowledge.staging.manifest import TableRecord
from thermo_knowledge.staging.writer import sha256_file

MANIFEST_NAME = "manifest.json"
MANIFEST_SCHEMA = 1
COMPRESSION = "zstd"


class CanonicalError(Exception):
    """A refusal a person can act on: a stale or missing input, a file the manifest disowns."""


class FormulaScopeRecord(Struct, forbid_unknown_fields=True):
    """A scope of a carrier whose source entities a formula and charge identify (rule 4)."""

    scope: str
    discriminator: str | None = None


class CanonicalManifest(Struct, forbid_unknown_fields=True):
    """What one canonical directory holds and the inputs it was made from. A phase-1 manifest
    also carries the carrier (with the hash of every file its claims cite) and the formula
    scopes the mapping declares, so resolution needs no source tree and no mapping."""

    schema: int
    source_id: str
    phase: str
    reuse_key: str
    inputs: dict[str, str]
    tables: dict[str, TableRecord]
    summary: dict[str, int] = {}
    carrier: CarrierInfo | None = None
    formula_scopes: list[FormulaScopeRecord] = []


def file_name(table: str) -> str:
    return f"{table}.parquet"


def reuse_key(description: Mapping[str, object]) -> str:
    """SHA-256 over the canonical JSON of the inputs that decide a run's output."""
    text = json.dumps(description, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(text.encode()).hexdigest()


def write_tables(directory: Path, tables: Mapping[str, pa.Table]) -> dict[str, TableRecord]:
    """Write each table as `<schema>.<table>.parquet` under `directory`."""
    directory.mkdir(parents=True, exist_ok=True)
    records: dict[str, TableRecord] = {}
    for name in sorted(tables):
        table = tables[name]
        path = directory / file_name(name)
        pq.write_table(table, path, compression=COMPRESSION)
        records[name] = TableRecord(
            file=path.name,
            rows=table.num_rows,
            schema_fingerprint=staging_schema.fingerprint(table.schema),
            content_hash=sha256_file(path),
        )
    return records


def encode(manifest: CanonicalManifest) -> bytes:
    return msgspec.json.format(msgspec.json.encode(manifest, order="sorted"), indent=2) + b"\n"


def write_manifest(directory: Path, manifest: CanonicalManifest) -> None:
    (directory / MANIFEST_NAME).write_bytes(encode(manifest))


def read_manifest(directory: Path) -> CanonicalManifest:
    path = directory / MANIFEST_NAME
    try:
        manifest = msgspec.json.decode(path.read_bytes(), type=CanonicalManifest)
    except FileNotFoundError as error:
        raise CanonicalError(f"{path}: missing") from error
    except (msgspec.DecodeError, msgspec.ValidationError) as error:
        raise CanonicalError(f"{path}: invalid: {error}") from error
    if manifest.schema != MANIFEST_SCHEMA:
        raise CanonicalError(f"{path}: manifest schema {manifest.schema} is not {MANIFEST_SCHEMA}")
    return manifest


def manifest_hash(directory: Path) -> str:
    """The SHA-256 of a directory's manifest file: what a later stage records of its input."""
    return sha256_file(directory / MANIFEST_NAME)


def verify_directory(directory: Path, manifest: CanonicalManifest) -> dict[str, Path]:
    """Every Parquet file of `directory` against its manifest (presence, content hash, rows);
    returns table name to path."""
    problems: list[str] = []
    paths: dict[str, Path] = {}
    for name, record in manifest.tables.items():
        path = directory / record.file
        if not path.is_file():
            problems.append(f"{name}: {record.file} is missing")
        elif sha256_file(path) != record.content_hash:
            problems.append(f"{name}: {record.file} differs from the manifest's content hash")
        elif pq.ParquetFile(path).metadata.num_rows != record.rows:
            problems.append(f"{name}: {record.file} does not have the {record.rows} rows recorded")
        else:
            paths[name] = path
    listed = {record.file for record in manifest.tables.values()}
    for path in sorted(directory.glob("*.parquet")):
        if path.name not in listed:
            problems.append(f"{path.name}: a Parquet file the manifest does not list")
    if problems:
        joined = "\n  ".join(problems)
        raise CanonicalError(f"{directory}: does not match its manifest:\n  {joined}")
    return paths


def read_table(path: Path) -> pa.Table:
    return pq.read_table(path)


def new_work_directory(parent: Path) -> Path:
    """A fresh temporary directory beside where the result will be installed."""
    parent.mkdir(parents=True, exist_ok=True)
    return Path(tempfile.mkdtemp(prefix=".partial-", dir=parent))


def install_directory(work: Path, destination: Path) -> None:
    """Rename the completed `work` into place, replacing an earlier result."""
    if destination.exists():
        retired = destination.with_name(f".retired-{work.name}")
        destination.rename(retired)
        try:
            work.rename(destination)
        except BaseException:
            retired.rename(destination)
            raise
        shutil.rmtree(retired, ignore_errors=True)
    else:
        work.rename(destination)

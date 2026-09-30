# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers of the FeOS, ThermoML schema, IDAES and JANAF reader tests: run a reader over a
synthetic tree written by the test, and stage a real acquired source into a temporary directory.
Contains no tests."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import payload, stage
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.staging.writer import Writer, WriterResult


@dataclass(frozen=True)
class Run:
    """The tables and payload accounting of one reader run."""

    tables: dict[str, pa.Table]
    result: WriterResult

    def rows(self, name: str) -> list[dict]:
        return self.tables[name].to_pylist()


def run_reader(module: ModuleType, tmp_path: Path, files: Mapping[str, str]) -> Run:
    """Run `module.read` over a tree holding `files` (relative path to text); every file is
    a payload file."""
    tree = tmp_path / "tree"
    for relative, content in files.items():
        path = tree / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8", newline="")
    out = tmp_path / "out"
    out.mkdir()
    writer = Writer(out, module.TABLES, payload.tree_files(tree))
    try:
        module.read(tree, writer)
    except BaseException:
        writer.abort()
        raise
    result = writer.finish()
    tables = {name: pq.read_table(out / f"{name}.parquet") for name in module.TABLES}
    return Run(tables, result)


def stage_real(source_id: str, staged_dir: Path) -> Path:
    """`tk read <source_id>` into `staged_dir`; the staged directory of the lock's pin."""
    ctx = StageContext(staged_dir=staged_dir)
    manifests = load_sources(default_sources_dir())
    outcome = stage.read_source(ctx, manifests[source_id], read_lock(default_lock_path()))
    assert outcome.status == "read"
    entry = read_lock(default_lock_path())[source_id]
    assert entry.pin is not None
    return stage.staged_path(ctx, source_id, entry.pin)


def staged_table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def staged_manifest_of(staged: Path) -> staged_manifest.StagedManifest:
    return staged_manifest.read(staged)


def field_metadata(schema: pa.Schema, name: str) -> dict[str, str]:
    raw = schema.field(name).metadata or {}
    return {key.decode(): value.decode() for key, value in raw.items()}

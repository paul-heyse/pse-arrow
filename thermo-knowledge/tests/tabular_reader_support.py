# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers of the chemicals and thermo reader tests: stage synthetic payload files through a
reader module and the real writer. Contains no tests."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from types import ModuleType

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge.staging.manifest import FileRecord
from thermo_knowledge.staging.writer import Writer


@dataclass
class Staged:
    directory: Path
    tables: dict[str, int]
    payload: dict[str, FileRecord]

    def table(self, name: str) -> pa.Table:
        return pq.read_table(self.directory / f"{name}.parquet")

    def rows(self, name: str) -> list[dict]:
        return self.table(name).to_pylist()


def stage(module: ModuleType, tmp_path: Path, files: dict[str, str | bytes]) -> Staged:
    """Write `files` (payload path to content) under a tree and read them with `module`."""
    tree = tmp_path / "tree"
    for relative, content in files.items():
        path = tree / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode() if isinstance(content, str) else content)
    out = tmp_path / "out"
    out.mkdir()
    writer = Writer(out, module.TABLES, sorted(files))
    try:
        module.read(tree, writer)
    except BaseException:
        writer.abort()
        raise
    result = writer.finish()
    return Staged(
        out,
        {name: record.rows for name, record in result.tables.items()},
        {record.path: record for record in result.payload},
    )


def lines(*rows: str) -> str:
    """Tab-separated text from rows written with `|` between cells, ending with a newline."""
    return "\n".join(row.replace("|", "\t") for row in rows) + "\n"

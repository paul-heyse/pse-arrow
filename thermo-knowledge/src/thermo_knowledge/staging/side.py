# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Side-environment readers: the protocol between the core and a reader in `envs/<name>/`.

The core runs

```
envs/tk-env.sh run <env> python envs/<env>/readers/<reader>.py --tree <tree> --out <dir>
```

in the tree directory. The script cannot import `thermo_knowledge`. It writes into `<dir>` one
Parquet file per table and a `tables.json`:

```json
{
  "protocol": 1,
  "reader_version": "3",
  "tables": {
    "<table>": {
      "file": "<table>.parquet",
      "schema": [
        {"name": "_artifact", "type": "string", "nullable": false,
         "metadata": {"source_name": "...", "unit": "not applicable"}},
        ...
      ]
    }
  },
  "files": {"<payload path>": {"status": "read|partly_read|skipped", "reason": "..."}}
}
```

Column types are `string`, `int16`, `int32`, `int64`, `float32`, `float64`, `bool` and
`list<...>` of those. The declared schema is the script's own statement, made without looking at
the data; the core checks each Parquet file against it and then passes the batches through the
same `Writer` as an in-process reader, so every rule (declared schema, `_artifact` and `_locator`,
unique locators, payload accounting) applies unchanged and the manifest is written the same way.
`files` lists the payload files the script read in part or not at all, with reasons; a file listed
as `read` (or one cited by a row) is read; any payload file nothing accounts for is an error.
`reader_version` must equal the script's `READER_VERSION`.
"""

from __future__ import annotations

import subprocess
from collections.abc import Callable
from pathlib import Path
from typing import Literal

import msgspec
import pyarrow as pa
import pyarrow.parquet as pq
from msgspec import Struct

from thermo_knowledge import config
from thermo_knowledge.staging import schema as schema_module
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.reader import ResolvedReader, envs_dir
from thermo_knowledge.staging.writer import Writer

PROTOCOL = 1
TABLES_NAME = "tables.json"

Command = Callable[[str, Path, Path, Path], list[str]]
"""`(environment, script, tree, out)` to the argument vector that runs the script."""


def tk_env_command(environment: str, script: Path, tree: Path, out: Path) -> list[str]:
    """The invocation through `envs/tk-env.sh` (the side environment's own interpreter)."""
    return [
        "bash",
        str(envs_dir() / "tk-env.sh"),
        "run",
        environment,
        "python",
        str(script),
        "--tree",
        str(tree),
        "--out",
        str(out),
    ]


class SideColumn(Struct, forbid_unknown_fields=True):
    name: str
    type: str
    nullable: bool = True
    metadata: dict[str, str] = {}


class SideTable(Struct, forbid_unknown_fields=True):
    file: str
    schema: list[SideColumn]


class SideFile(Struct, forbid_unknown_fields=True):
    status: Literal["read", "partly_read", "skipped"]
    reason: str | None = None


class SideTables(Struct, forbid_unknown_fields=True):
    protocol: int
    reader_version: str
    tables: dict[str, SideTable]
    files: dict[str, SideFile] = {}


def run_side(
    reader: ResolvedReader,
    tree: Path,
    out: Path,
    *,
    command: Command = tk_env_command,
) -> SideTables:
    """Run the script and parse its `tables.json`; raises `StagingError` on any failure."""
    script = reader.script
    assert script is not None
    arguments = command(reader.environment, script, tree, out)
    try:
        completed = subprocess.run(
            arguments, cwd=config.TREE_DIR, capture_output=True, text=True, check=False
        )
    except OSError as error:
        raise StagingError(f"cannot run the side reader {reader.describe()}: {error}") from error
    if completed.returncode != 0:
        tail = "\n".join(completed.stderr.strip().splitlines()[-20:])
        raise StagingError(
            f"side reader {reader.describe()} exited with status {completed.returncode}:\n{tail}"
        )
    path = out / TABLES_NAME
    try:
        described = msgspec.json.decode(path.read_bytes(), type=SideTables)
    except FileNotFoundError as error:
        raise StagingError(f"side reader {reader.describe()} wrote no {TABLES_NAME}") from error
    except (msgspec.DecodeError, msgspec.ValidationError) as error:
        raise StagingError(f"{path}: invalid: {error}") from error
    if described.protocol != PROTOCOL:
        raise StagingError(f"{path}: protocol {described.protocol} is not {PROTOCOL}")
    if described.reader_version != reader.version:
        raise StagingError(
            f"{path}: reader_version {described.reader_version!r} differs from the script's "
            f"READER_VERSION {reader.version!r}"
        )
    return described


def declared_schemas(described: SideTables) -> dict[str, pa.Schema]:
    """The schemas `tables.json` declares."""
    schemas: dict[str, pa.Schema] = {}
    for table, entry in described.tables.items():
        fields = []
        for item in entry.schema:
            fields.append(
                pa.field(
                    item.name,
                    schema_module.parse_type(item.type),
                    nullable=item.nullable,
                    metadata=dict(item.metadata) or None,
                )
            )
        schemas[table] = pa.schema(fields)
    return schemas


def feed(described: SideTables, out: Path, writer: Writer, schemas: dict[str, pa.Schema]) -> None:
    """Validate the script's Parquet files against its declaration and pass them through
    `writer`; record the file statuses."""
    listed = {entry.file for entry in described.tables.values()}
    for extra in sorted(path.name for path in out.glob("*.parquet")):
        if extra not in listed:
            raise StagingError(f"{extra}: a Parquet file {TABLES_NAME} does not declare")
    for table, entry in described.tables.items():
        if entry.file != f"{table}.parquet":
            raise StagingError(f"table {table}: file must be {table}.parquet, not {entry.file!r}")
        path = out / entry.file
        if not path.is_file():
            raise StagingError(f"table {table}: {entry.file} is missing")
        parquet = pq.ParquetFile(path)
        if not schema_module.schemas_equal(parquet.schema_arrow, schemas[table]):
            raise StagingError(
                f"table {table}: {entry.file} does not have the schema {TABLES_NAME} declares\n"
                f"  declared: {schemas[table].to_string(show_field_metadata=False)}\n"
                f"  file:     {parquet.schema_arrow.to_string(show_field_metadata=False)}"
            )
        for batch in parquet.iter_batches():
            writer.batch(table, batch)
    for artifact, file in described.files.items():
        if file.status == "read":
            writer.opened(artifact)
        elif file.status == "partly_read":
            writer.partly_read(artifact, file.reason or "")
        else:
            writer.skipped(artifact, file.reason or "")

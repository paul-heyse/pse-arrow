# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk load-src`: load staged Parquet into the PostgreSQL schema `src_<id>`.

The schema is dropped and recreated, one table is created per staged table with the column
types mapped from the Arrow schema, the rows are bulk-loaded with ADBC, `_locator` becomes the
primary key, each column's documentation (the Arrow field metadata) becomes its comment, and the
manifest is recorded in `src_<id>._manifest`. Loading refuses staged data whose manifest does not
match its Parquet files, any schema not named `src_<something>`, and the operational database.

| Arrow | PostgreSQL |
|---|---|
| string | text |
| int16, int32, int64 | smallint, integer, bigint |
| float32, float64 | real, double precision |
| bool | boolean |
| list of one of those | the matching array |
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import LiteralString

import adbc_driver_postgresql.dbapi as adbc
import msgspec
import psycopg
import pyarrow as pa
import pyarrow.parquet as pq
from psycopg import sql

from thermo_knowledge import config, db
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import schema as schema_module
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.manifest import StagedManifest
from thermo_knowledge.staging.schema import LOCATOR
from thermo_knowledge.staging.writer import sha256_file

SCHEMA_PATTERN = re.compile(r"^src_[a-z][a-z0-9_]*$")
MANIFEST_TABLE = "_manifest"
BATCH_ROWS = 50_000

_SCALAR_SQL: dict[str, LiteralString] = {
    "string": "text",
    "int16": "smallint",
    "int32": "integer",
    "int64": "bigint",
    "float32": "real",
    "float64": "double precision",
    "bool": "boolean",
}


@dataclass(frozen=True)
class LoadOutcome:
    schema: str
    tables: dict[str, int]


def sql_type(dtype: pa.DataType) -> LiteralString:
    """The PostgreSQL type of a supported Arrow type; refuses any other."""
    name = schema_module.type_name(dtype)  # raises StagingError with the supported list
    if name.startswith("list<"):
        return f"{_SCALAR_SQL[name[5:-1]]}[]"
    return _SCALAR_SQL[name]


def refuse_schema(schema: str) -> str:
    """Return `schema` when it is a `src_<something>` name; refuse any other."""
    if not SCHEMA_PATTERN.match(schema) or len(schema.encode()) > 63:
        raise StagingError(
            f"refusing to touch schema {schema!r}: load-src only manages schemas named "
            "src_<something> (lowercase letters, digits and underscores)"
        )
    return schema


def verify_staged(directory: Path) -> tuple[StagedManifest, dict[str, pa.Schema]]:
    """The manifest of a staged directory and its tables' schemas, after checking every Parquet
    file against the manifest (presence, content hash, row count, schema fingerprint) and
    every column's type against what a staging table can hold."""
    manifest = staged_manifest.read(directory)
    schemas: dict[str, pa.Schema] = {}
    problems: list[str] = []
    for table, record in manifest.tables.items():
        path = directory / record.file
        if not path.is_file():
            problems.append(f"{table}: {record.file} is missing")
            continue
        if sha256_file(path) != record.content_hash:
            problems.append(f"{table}: {record.file} differs from the manifest's content hash")
            continue
        parquet = pq.ParquetFile(path)
        if parquet.metadata.num_rows != record.rows:
            problems.append(
                f"{table}: {record.file} has {parquet.metadata.num_rows} rows, "
                f"the manifest records {record.rows}"
            )
            continue
        schema = parquet.schema_arrow
        if schema_module.fingerprint(schema) != record.schema_fingerprint:
            problems.append(f"{table}: {record.file} has a schema the manifest does not record")
            continue
        schemas[table] = schema
    listed = {record.file for record in manifest.tables.values()}
    for path in sorted(directory.glob("*.parquet")):
        if path.name not in listed:
            problems.append(f"{path.name}: a Parquet file the manifest does not list")
    if problems:
        joined = "\n  ".join(problems)
        raise StagingError(f"{directory}: the staged data does not match its manifest:\n  {joined}")
    for table, schema in schemas.items():
        try:
            schema_module.validate_schema(table, schema)
        except StagingError as error:
            raise StagingError(f"{directory}: {error}") from error
    return manifest, schemas


def _create_table(schema_name: str, table: str, schema: pa.Schema) -> sql.Composed:
    columns = [
        sql.SQL("{} {}{}").format(
            sql.Identifier(field.name),
            sql.SQL(sql_type(field.type)),
            sql.SQL("" if field.nullable else " NOT NULL"),
        )
        for field in schema
    ]
    return sql.SQL("CREATE TABLE {}.{} ({})").format(
        sql.Identifier(schema_name), sql.Identifier(table), sql.SQL(", ").join(columns)
    )


def column_comment(field: pa.Field) -> str:
    """`source_name: ...; unit: ...[; note: ...]` from the field metadata."""
    metadata = schema_module.field_metadata(field)
    parts = [
        f"source_name: {metadata[schema_module.SOURCE_NAME]}",
        f"unit: {metadata[schema_module.UNIT]}",
    ]
    if metadata.get(schema_module.NOTE):
        parts.append(f"note: {metadata[schema_module.NOTE]}")
    return "; ".join(parts)


def load_staged(url: str, directory: Path, schema_name: str) -> LoadOutcome:
    """Load the staged directory into `schema_name` of the database `url` names."""
    refuse_schema(schema_name)
    config.refuse_production(config.database_name(url))
    manifest, schemas = verify_staged(directory)
    # Every statement below names only the validated schema, the reader's validated table names
    # and column identifiers quoted by psycopg.
    schema_id = sql.Identifier(schema_name)
    with db.connect(url, autocommit=True) as conn:
        conn.execute(sql.SQL("DROP SCHEMA IF EXISTS {} CASCADE").format(schema_id))
        conn.execute(sql.SQL("CREATE SCHEMA {}").format(schema_id))
        for table, schema in schemas.items():
            conn.execute(_create_table(schema_name, table, schema))
    try:
        counts = _ingest(url, directory, schema_name, manifest, schemas)
        with db.connect(url, autocommit=True) as conn:
            with conn.transaction():
                _finish(conn, schema_name, manifest, schemas)
    except BaseException:
        with db.connect(url, autocommit=True) as conn:
            conn.execute(sql.SQL("DROP SCHEMA IF EXISTS {} CASCADE").format(schema_id))
        raise
    return LoadOutcome(schema_name, counts)


def _ingest(
    url: str,
    directory: Path,
    schema_name: str,
    manifest: StagedManifest,
    schemas: dict[str, pa.Schema],
) -> dict[str, int]:
    counts: dict[str, int] = {}
    with adbc.connect(url) as connection:
        with connection.cursor() as cursor:
            for table, record in manifest.tables.items():
                schema = schemas[table]
                parquet = pq.ParquetFile(directory / record.file)
                reader = pa.RecordBatchReader.from_batches(
                    schema, parquet.iter_batches(batch_size=BATCH_ROWS)
                )
                loaded = cursor.adbc_ingest(
                    table, reader, mode="append", db_schema_name=schema_name
                )
                if loaded not in (-1, record.rows):
                    raise StagingError(
                        f"{table}: loaded {loaded} rows, the manifest records {record.rows}"
                    )
                counts[table] = record.rows
        connection.commit()
    return counts


def _finish(
    conn: psycopg.Connection,
    schema_name: str,
    manifest: StagedManifest,
    schemas: dict[str, pa.Schema],
) -> None:
    schema_id = sql.Identifier(schema_name)
    for table, schema in schemas.items():
        table_id = sql.Identifier(table)
        conn.execute(
            sql.SQL("ALTER TABLE {}.{} ADD PRIMARY KEY ({})").format(
                schema_id, table_id, sql.Identifier(LOCATOR)
            )
        )
        for field in schema:
            conn.execute(
                sql.SQL("COMMENT ON COLUMN {}.{}.{} IS {}").format(
                    schema_id,
                    table_id,
                    sql.Identifier(field.name),
                    sql.Literal(column_comment(field)),
                )
            )
    conn.execute(
        sql.SQL(
            "CREATE TABLE {}.{} (source_id text NOT NULL, pin text NOT NULL, reader text NOT NULL, "
            "reader_version text NOT NULL, reuse_key text NOT NULL, "
            "loaded_at timestamptz NOT NULL DEFAULT now(), manifest jsonb NOT NULL)"
        ).format(schema_id, sql.Identifier(MANIFEST_TABLE))
    )
    conn.execute(
        sql.SQL(
            "INSERT INTO {}.{} (source_id, pin, reader, reader_version, reuse_key, manifest) "
            "VALUES (%s, %s, %s, %s, %s, %s)"
        ).format(schema_id, sql.Identifier(MANIFEST_TABLE)),
        (
            manifest.source_id,
            manifest.pin,
            manifest.reader.name,
            manifest.reader.version,
            manifest.reuse_key,
            msgspec.json.encode(manifest, order="sorted").decode(),
        ),
    )

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The one transaction of `tk build` (pipeline section 3).

Everything happens on one ADBC connection, because PostgreSQL has no transaction that spans two:
the generated DDL, the reified declaration and declared entities, the bulk load of the union
and the build record all belong to it, and nothing is visible until the commit that validates
every deferred constraint.

* The DDL is a script of many statements, which ADBC's prepared execution refuses; it runs as
  the argument of one `EXECUTE` in a `DO` block.
* The reified `meta` rows and the declared entities are Python values of many PostgreSQL types
  (ranges, arrays, enums, domains); they are sent as one JSON document per table that the server
  parses into the table's own column types (`json_populate_recordset`), as `COPY ... FROM STDIN`
  text would be.
* The canonical Parquet is ingested with `adbc_ingest` in append mode.

The six canonical schemas are dropped and recreated; the database's other schemas (`src_<id>`
above all) are not touched, and a build refuses, before dropping anything, when an object in
another schema depends on a canonical one, since `DROP ... CASCADE` would remove it.
"""

from __future__ import annotations

import json
import uuid
from datetime import date, datetime
from pathlib import Path

import adbc_driver_manager
import adbc_driver_postgresql.dbapi as adbc
import pyarrow as pa
import pyarrow.parquet as pq
from psycopg import sql

from thermo_knowledge.build import record
from thermo_knowledge.build.plan import BuildPlan
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.generate.entity_rows import TableRows
from thermo_knowledge.generate.plan import GENERATED_SCHEMAS, SCHEMA_DOCS
from thermo_knowledge.generate.fingerprint import fingerprint_comment

BATCH_ROWS = 50_000
_SCRIPT_TAG = "$tk_build$"


class DatabaseRefusedError(CanonicalError):
    """PostgreSQL refused a statement or the commit; the transaction was rolled back."""


class ForeignDependentsError(CanonicalError):
    """An object outside the canonical schemas depends on one inside them."""


def _json_cell(value: object) -> object:
    if value is None or isinstance(value, bool | int | float | str):
        return value
    if isinstance(value, uuid.UUID):
        return str(value)
    if isinstance(value, datetime | date):
        return value.isoformat()
    if isinstance(value, bytes):
        return "\\x" + value.hex()
    if isinstance(value, list | tuple):
        return [_json_cell(item) for item in value]
    raise TypeError(f"cannot send {type(value).__name__} to the database")


def _canonical_schemas() -> sql.Composable:
    return sql.SQL("ARRAY[{}]").format(
        sql.SQL(", ").join(sql.Literal(name) for name in GENERATED_SCHEMAS)
    )


def _script(text: str) -> str:
    """`text`, a script of several statements, as one statement."""
    if _SCRIPT_TAG in text:
        raise ValueError(f"the script contains the quote tag {_SCRIPT_TAG}")
    return (
        sql.SQL(f"DO {_SCRIPT_TAG} BEGIN EXECUTE {{}}; END {_SCRIPT_TAG}")
        .format(sql.Literal(text))
        .as_string()
    )


def refuse_foreign_dependents(cursor: adbc.Cursor) -> None:
    """Raise when an object outside the canonical schemas depends on one inside them.

    The schema of a dependent is its own when `pg_identify_object` knows one; a rewrite rule (a
    view), trigger, column default or policy belongs to the schema of its table.
    """
    schemas = _canonical_schemas()
    query = sql.SQL(
        """
        SELECT DISTINCT pg_describe_object(d.classid, d.objid, d.objsubid)
        FROM pg_depend d
        WHERE d.deptype IN ('n', 'a')
          AND (pg_identify_object(d.refclassid, d.refobjid, d.refobjsubid)).schema = ANY({schemas})
          AND NOT coalesce(
                CASE d.classid
                  WHEN 'pg_rewrite'::regclass THEN (
                    SELECT n.nspname FROM pg_rewrite x JOIN pg_class c ON c.oid = x.ev_class
                    JOIN pg_namespace n ON n.oid = c.relnamespace WHERE x.oid = d.objid)
                  WHEN 'pg_trigger'::regclass THEN (
                    SELECT n.nspname FROM pg_trigger x JOIN pg_class c ON c.oid = x.tgrelid
                    JOIN pg_namespace n ON n.oid = c.relnamespace WHERE x.oid = d.objid)
                  WHEN 'pg_attrdef'::regclass THEN (
                    SELECT n.nspname FROM pg_attrdef x JOIN pg_class c ON c.oid = x.adrelid
                    JOIN pg_namespace n ON n.oid = c.relnamespace WHERE x.oid = d.objid)
                  WHEN 'pg_policy'::regclass THEN (
                    SELECT n.nspname FROM pg_policy x JOIN pg_class c ON c.oid = x.polrelid
                    JOIN pg_namespace n ON n.oid = c.relnamespace WHERE x.oid = d.objid)
                  ELSE (pg_identify_object(d.classid, d.objid, d.objsubid)).schema
                END = ANY({schemas}),
                true)
        ORDER BY 1
        """
    ).format(schemas=schemas)
    cursor.execute(query.as_string())
    found = [name for (name,) in cursor.fetchall()]
    if found:
        listed = "\n  ".join(found[:10])
        raise ForeignDependentsError(
            "objects outside the canonical schemas depend on objects a build drops; drop or "
            f"move them first ({len(found)}):\n  {listed}"
        )


def drop_canonical(cursor: adbc.Cursor) -> None:
    cursor.execute(
        sql.SQL("DROP SCHEMA IF EXISTS {} CASCADE")
        .format(sql.SQL(", ").join(sql.Identifier(name) for name in GENERATED_SCHEMAS))
        .as_string()
    )


def apply_script(cursor: adbc.Cursor, text: str) -> None:
    """Run a script of several statements."""
    cursor.execute(_script(text))


def insert_rows(cursor: adbc.Cursor, batch: TableRows) -> None:
    """Insert Python rows by sending them as JSON for the server to parse into the columns'
    own types."""
    if not batch.rows:
        return
    document = json.dumps(
        [dict(zip(batch.columns, (_json_cell(v) for v in row))) for row in batch.rows],
        ensure_ascii=False,
        allow_nan=False,
    )
    target = sql.Identifier(batch.schema, batch.table)
    columns = sql.SQL(", ").join(sql.Identifier(name) for name in batch.columns)
    statement = sql.SQL(
        "INSERT INTO {target} ({columns}) "
        "SELECT {columns} FROM json_populate_recordset(NULL::{target}, $1::json)"
    ).format(target=target, columns=columns)
    cursor.execute(statement.as_string(), (document,))


def ingest_file(cursor: adbc.Cursor, table: str, path: Path) -> int:
    """Append a canonical Parquet file to its table; returns its rows."""
    schema, name = table.split(".", 1)
    parquet = pq.ParquetFile(path)
    reader = pa.RecordBatchReader.from_batches(
        parquet.schema_arrow, parquet.iter_batches(batch_size=BATCH_ROWS)
    )
    cursor.adbc_ingest(name, reader, mode="append", db_schema_name=schema)
    return parquet.metadata.num_rows


def record_fingerprint(cursor: adbc.Cursor, fingerprint: str) -> None:
    comment = fingerprint_comment(SCHEMA_DOCS["tk"], fingerprint)
    cursor.execute(
        sql.SQL("COMMENT ON SCHEMA {} IS {}")
        .format(sql.Identifier("tk"), sql.Literal(comment))
        .as_string()
    )


def apply(url: str, plan: BuildPlan, built_at: datetime) -> None:
    """Replace the canonical schemas with `plan` in one transaction and commit, or change
    nothing.

    The order is: the generated DDL and `sql/physical.sql`; the reified `meta` rows and the
    declared entities; the union of the sources' files (constraints are deferred, so tables load
    in any order); the build record; the schema fingerprint; the commit.
    """
    with adbc.connect(url) as connection:
        try:
            with connection.cursor() as cursor:
                refuse_foreign_dependents(cursor)
                drop_canonical(cursor)
                apply_script(cursor, plan.ddl)
                if plan.physical.strip():
                    apply_script(cursor, plan.physical)
                cursor.execute("SET CONSTRAINTS ALL DEFERRED")
                for batch in plan.batches:
                    insert_rows(cursor, batch)
                for table, path in plan.parts():
                    ingest_file(cursor, table, path)
                for statement in record.create_statements():
                    cursor.execute(statement)
                insert_rows(cursor, record.rows(plan.inputs, built_at))
                record_fingerprint(cursor, plan.fingerprint)
            connection.commit()
        except adbc_driver_manager.Error as error:
            connection.rollback()
            raise DatabaseRefusedError(f"the database refused the build: {error}") from error
        except BaseException:
            connection.rollback()
            raise

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Declared table schemas of a reader.

A reader declares an explicit `pyarrow` schema for every table it emits; nothing is inferred from
data. Every table has the provenance columns `_artifact` (path of the file relative to the
acquired tree) and `_locator` (`<artifact>#<position>`; unique per row within the table), and
every column carries the documentation the source allows as Arrow field metadata:

- `source_name`: the source's own name for the value (a key, a column heading, a key path),
- `unit`: the unit exactly as the source states it, or `not stated`,
- `note`: optional free text (for example that an integer-or-float field is stored as float64).

The column types are those a PostgreSQL staging table can hold: strings, integers, floats,
booleans and one-dimensional lists of those.
"""

from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Mapping

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError

ARTIFACT = "_artifact"
LOCATOR = "_locator"
SOURCE_NAME = "source_name"
UNIT = "unit"
NOTE = "note"
NOT_STATED = "not stated"

_NAME = re.compile(r"^[a-z][a-z0-9_]*$")
_MAX_IDENTIFIER_BYTES = 63
POSTGRES_SYSTEM_COLUMNS = frozenset({"tableoid", "xmin", "cmin", "xmax", "cmax", "ctid"})
"""Names PostgreSQL reserves for system columns; a staged column cannot carry them, so a reader
renames the column and keeps the source's name in the `source_name` metadata."""

STRING = pa.string()
INT64 = pa.int64()
FLOAT64 = pa.float64()
BOOL = pa.bool_()

_SCALAR_TYPES: dict[str, pa.DataType] = {
    "string": pa.string(),
    "int16": pa.int16(),
    "int32": pa.int32(),
    "int64": pa.int64(),
    "float32": pa.float32(),
    "float64": pa.float64(),
    "bool": pa.bool_(),
}
_LIST = re.compile(r"^list<([a-z0-9]+)>$")


def type_name(dtype: pa.DataType) -> str:
    """The stable spelling of a supported type (`string`, `float64`, `list<int64>`, ...);
    raises `StagingError` for anything a staging table cannot hold."""
    if pa.types.is_list(dtype):
        inner = _scalar_name(dtype.value_type)
        if inner is None:
            raise StagingError(_unsupported(dtype))
        return f"list<{inner}>"
    name = _scalar_name(dtype)
    if name is None:
        raise StagingError(_unsupported(dtype))
    return name


def _scalar_name(dtype: pa.DataType) -> str | None:
    for name, candidate in _SCALAR_TYPES.items():
        if dtype == candidate:
            return name
    return None


def _unsupported(dtype: pa.DataType) -> str:
    supported = ", ".join([*_SCALAR_TYPES, *(f"list<{name}>" for name in _SCALAR_TYPES)])
    return f"Arrow type {dtype} is not supported; supported types: {supported}"


def parse_type(name: str) -> pa.DataType:
    """The Arrow type a spelling of `type_name` denotes."""
    match = _LIST.match(name)
    if match:
        inner = match.group(1)
        if inner in _SCALAR_TYPES:
            return pa.list_(_SCALAR_TYPES[inner])
    elif name in _SCALAR_TYPES:
        return _SCALAR_TYPES[name]
    raise StagingError(f"unknown column type {name!r}")


def column(
    name: str,
    dtype: pa.DataType,
    *,
    source_name: str | None = None,
    unit: str = NOT_STATED,
    note: str | None = None,
    nullable: bool = True,
) -> pa.Field:
    """A documented column. `source_name` defaults to the column name."""
    metadata = {SOURCE_NAME: source_name if source_name is not None else name, UNIT: unit}
    if note:
        metadata[NOTE] = note
    return pa.field(name, dtype, nullable=nullable, metadata=metadata)


PROVENANCE_FIELDS: tuple[pa.Field, ...] = (
    column(
        ARTIFACT,
        STRING,
        source_name="file path relative to the acquired tree",
        unit="not applicable",
        nullable=False,
    ),
    column(
        LOCATOR,
        STRING,
        source_name="<artifact>#<position>: a JSON pointer, [row] suffix, or L<line>",
        unit="not applicable",
        nullable=False,
    ),
)


def table_schema(*columns: pa.Field) -> pa.Schema:
    """The schema of a table: the provenance columns, then `columns`."""
    return pa.schema([*PROVENANCE_FIELDS, *columns])


def validate_table_name(name: str) -> None:
    if not _NAME.match(name) or len(name.encode()) > _MAX_IDENTIFIER_BYTES:
        raise StagingError(
            f"table name {name!r} must match ^[a-z][a-z0-9_]*$ and be at most "
            f"{_MAX_IDENTIFIER_BYTES} bytes"
        )


def validate_schema(table: str, schema: pa.Schema) -> None:
    """Refuse a declared schema that lacks the provenance columns, documentation or a
    supported type."""
    validate_table_name(table)
    names = [field.name for field in schema]
    if len(set(names)) != len(names):
        duplicates = sorted({name for name in names if names.count(name) > 1})
        raise StagingError(f"table {table}: duplicate column names: {', '.join(duplicates)}")
    for required in (ARTIFACT, LOCATOR):
        if required not in names:
            raise StagingError(f"table {table}: the required column {required} is not declared")
        field = schema.field(required)
        if field.type != STRING or field.nullable:
            raise StagingError(f"table {table}: {required} must be a non-nullable string")
    for field in schema:
        if not field.name or len(field.name.encode()) > _MAX_IDENTIFIER_BYTES or "\0" in field.name:
            raise StagingError(
                f"table {table}: column name {field.name!r} must be 1 to "
                f"{_MAX_IDENTIFIER_BYTES} bytes without NUL"
            )
        if field.name in POSTGRES_SYSTEM_COLUMNS:
            raise StagingError(
                f"table {table}: column name {field.name!r} is a PostgreSQL system column; "
                "rename the column and keep the source's name in the source_name metadata"
            )
        try:
            type_name(field.type)
        except StagingError as error:
            raise StagingError(f"table {table}, column {field.name}: {error}") from error
        metadata = _metadata(field)
        for key in (SOURCE_NAME, UNIT):
            if not metadata.get(key):
                raise StagingError(
                    f"table {table}, column {field.name}: field metadata {key!r} is missing; "
                    "every column documents the source's name and unit (or `not stated`)"
                )


def _metadata(field: pa.Field) -> dict[str, str]:
    raw = field.metadata or {}
    return {key.decode(): value.decode() for key, value in raw.items()}


def field_metadata(field: pa.Field) -> dict[str, str]:
    """The field's metadata as strings."""
    return _metadata(field)


def _spelling(dtype: pa.DataType) -> str:
    try:
        return type_name(dtype)
    except StagingError:
        return str(dtype)  # an unsupported type still has a fingerprint; validation refuses it


def fingerprint(schema: pa.Schema) -> str:
    """SHA-256 over names, types, nullability and metadata of the columns, in order."""
    description = [
        [field.name, _spelling(field.type), field.nullable, sorted(_metadata(field).items())]
        for field in schema
    ]
    text = json.dumps(description, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(text.encode()).hexdigest()


def schemas_equal(left: pa.Schema, right: pa.Schema) -> bool:
    """Names, types, nullability and metadata equal, in order (the name of a list's item field,
    which Parquet spells differently, does not count)."""
    return fingerprint(left) == fingerprint(right)


def check_declared(schemas: Mapping[str, pa.Schema]) -> None:
    if not schemas:
        raise StagingError("the reader declares no tables")
    for table, schema in schemas.items():
        validate_schema(table, schema)

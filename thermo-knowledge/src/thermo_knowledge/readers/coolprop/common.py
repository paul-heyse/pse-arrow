# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Building blocks of the CoolProp reader: documented columns, JSON objects split into a row of
scalar fields plus long-form coefficient rows, and a buffered sink over the writer.

Reading is strict: a JSON object is consumed key by key, and a key no column declares raises an
error instead of being dropped, so a new field in a later CoolProp release is a visible failure.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import (
    BOOL,
    FLOAT64,
    INT64,
    NOT_STATED,
    STRING,
    column,
    table_schema,
)
from thermo_knowledge.staging.writer import Writer

NUMBER_NOTE = "JSON number stored as float64; the source writes some values as JSON integers"
LENGTHS = "array_lengths"
NOT_APPLICABLE = "not applicable"

JsonValue = object


@dataclass(frozen=True)
class Col:
    """One documented column."""

    name: str
    dtype: pa.DataType
    source_name: str | None = None
    unit: str = NOT_STATED
    note: str | None = None
    unit_from: str | None = None
    """Name of the sibling field in which the source states this column's unit."""
    nullable: bool = True

    def field(self) -> pa.Field:
        return column(
            self.name,
            self.dtype,
            source_name=self.source_name,
            unit=self.unit,
            note=self.note,
            nullable=self.nullable,
        )


def num(
    name: str,
    unit: str = NOT_STATED,
    *,
    source_name: str | None = None,
    unit_from: str | None = None,
    note: str | None = None,
) -> Col:
    text = NUMBER_NOTE if note is None else f"{NUMBER_NOTE}; {note}"
    return Col(name, FLOAT64, source_name, unit, text, unit_from)


def text(
    name: str, *, source_name: str | None = None, nullable: bool = True, note: str | None = None
) -> Col:
    return Col(name, STRING, source_name, NOT_STATED, note, nullable=nullable)


def integer(name: str, *, source_name: str | None = None, unit: str = NOT_STATED) -> Col:
    return Col(name, INT64, source_name, unit)


def flag(name: str, *, source_name: str | None = None) -> Col:
    return Col(name, BOOL, source_name, NOT_STATED)


def texts(name: str, *, source_name: str | None = None) -> Col:
    """A list of strings kept as one column."""
    return Col(name, pa.list_(STRING), source_name, NOT_STATED)


def index(name: str, source_name: str) -> Col:
    return Col(name, INT64, source_name, NOT_APPLICABLE)


def pointer(*tokens: object) -> str:
    """A JSON pointer from its tokens (object keys are escaped, array indexes are numbers)."""
    return "".join(f"/{str(token).replace('~', '~0').replace('/', '~1')}" for token in tokens)


def locator(artifact: str, position: str) -> str:
    return f"{artifact}#{position}"


def is_scalar(value: object) -> bool:
    return value is None or isinstance(value, str | int | float | bool)


class Fields:
    """The keys of one JSON object, taken one by one; `done` refuses what nobody took."""

    def __init__(self, obj: object, where: str) -> None:
        if not isinstance(obj, dict):
            raise StagingError(f"{where}: expected a JSON object, found {type(obj).__name__}")
        self._rest = dict(obj)
        self.where = where

    def take(self, key: str) -> JsonValue:
        return self._rest.pop(key, None)

    def present(self, key: str) -> bool:
        return key in self._rest

    def take_object(self, key: str) -> dict[str, JsonValue] | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not isinstance(value, dict):
            raise StagingError(f"{self.where}/{key}: expected an object")
        return value

    def take_list(self, key: str) -> list[JsonValue] | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not isinstance(value, list):
            raise StagingError(f"{self.where}/{key}: expected a list")
        return value

    def remaining(self) -> dict[str, JsonValue]:
        return self._rest

    def done(self) -> None:
        if self._rest:
            keys = ", ".join(sorted(self._rest))
            raise StagingError(
                f"{self.where}: fields no column declares: {keys}; extend the CoolProp reader "
                "and bump its READER_VERSION"
            )


def check_unit(where: str, column_spec: Col, record: Mapping[str, JsonValue]) -> None:
    """The unit the source states beside a value must be the one the column documents."""
    if column_spec.unit_from is None:
        return
    stated = record.get(column_spec.unit_from)
    if stated is not None and stated != column_spec.unit:
        raise StagingError(
            f"{where}: {column_spec.unit_from} is {stated!r} but the column documents "
            f"{column_spec.unit!r}"
        )


@dataclass(frozen=True)
class ObjectSpec:
    """An object kind that becomes one row of a scalar table plus, per coefficient row, rows of
    a long table.

    Scalar-valued JSON fields go to columns of `scalars_table`; list-valued fields (parallel
    coefficient arrays) go to the columns of `rows_table`, one row per index, null where an array
    is shorter or absent. `array_lengths` records each array's length in source order, so an
    empty array and an absent one stay distinct. A field named in `lift` is moved from the
    object into the key columns (the term `type`).
    """

    scalars_table: str
    keys: tuple[Col, ...]
    scalars: tuple[Col, ...]
    rows_table: str | None = None
    arrays: tuple[Col, ...] = ()
    lift: tuple[str, ...] = ()
    row_index: str = "row_index"
    lengths: bool = True
    """Whether the scalar table carries `array_lengths` (tables whose objects have no arrays
    do not)."""

    def schemas(self) -> dict[str, pa.Schema]:
        lengths = Col(
            LENGTHS,
            STRING,
            "length of each list-valued field of the object, in source order (JSON)",
            NOT_APPLICABLE,
        )
        scalar_columns = (*self.keys, *self.scalars, *((lengths,) if self.lengths else ()))
        out = {self.scalars_table: table_schema(*(c.field() for c in scalar_columns))}
        if self.rows_table is not None:
            position = index(self.row_index, "position within the parallel arrays")
            out[self.rows_table] = table_schema(
                *(c.field() for c in (*self.keys, position, *self.arrays))
            )
        return out

    def rows(
        self,
        artifact: str,
        position: str,
        key_values: Mapping[str, JsonValue],
        obj: object,
    ) -> tuple[dict[str, JsonValue], list[dict[str, JsonValue]]]:
        """The scalar row and the coefficient rows of one object."""
        where = locator(artifact, position)
        body = Fields(obj, where)
        keys = dict(key_values)
        for name in self.lift:
            keys[name] = body.take(name)
        rest = body.remaining()
        scalar_names = {c.name: c for c in self.scalars}
        array_names = {c.name: c for c in self.arrays}
        scalar_row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": where,
            **keys,
        }
        arrays: dict[str, list[JsonValue]] = {}
        for key, value in rest.items():
            if (
                isinstance(value, list)
                and key in scalar_names
                and pa.types.is_list(scalar_names[key].dtype)
            ):
                scalar_row[key] = value
            elif isinstance(value, list):
                if key not in array_names:
                    raise StagingError(f"{where}/{key}: a list no column declares")
                for item in value:
                    if not is_scalar(item):
                        raise StagingError(f"{where}/{key}: nested lists and objects are not read")
                arrays[key] = value
            elif isinstance(value, dict):
                raise StagingError(f"{where}/{key}: an object no column declares")
            else:
                if key not in scalar_names:
                    raise StagingError(f"{where}/{key}: a field no column declares")
                check_unit(where, scalar_names[key], rest)
                scalar_row[key] = value
        if self.lengths:
            scalar_row[LENGTHS] = json.dumps(
                {k: len(v) for k, v in arrays.items()}, separators=(",", ":")
            )
        rows: list[dict[str, JsonValue]] = []
        if arrays and self.rows_table is None:
            raise StagingError(f"{where}: lists without a coefficient table")
        width = max((len(v) for v in arrays.values()), default=0)
        for row in range(width):
            entry: dict[str, JsonValue] = {
                "_artifact": artifact,
                "_locator": locator(artifact, f"{position}[{row}]"),
                **keys,
                self.row_index: row,
            }
            for name, values in arrays.items():
                if row < len(values):
                    entry[name] = values[row]
            rows.append(entry)
        return scalar_row, rows


@dataclass
class Sink:
    """Collects rows per table and hands them to the writer in batches."""

    writer: Writer
    batch: int = 20_000
    _buffers: dict[str, list[dict[str, JsonValue]]] = field(default_factory=dict)

    def add(self, table: str, row: dict[str, JsonValue]) -> None:
        buffer = self._buffers.setdefault(table, [])
        buffer.append(row)
        if len(buffer) >= self.batch:
            self._flush(table)

    def add_object(
        self,
        spec: ObjectSpec,
        artifact: str,
        position: str,
        key_values: Mapping[str, JsonValue],
        obj: object,
    ) -> None:
        scalar_row, rows = spec.rows(artifact, position, key_values, obj)
        self.add(spec.scalars_table, scalar_row)
        if spec.rows_table is not None:
            for row in rows:
                self.add(spec.rows_table, row)

    def _flush(self, table: str) -> None:
        buffer = self._buffers.get(table)
        if buffer:
            self.writer.rows(table, buffer)
            self._buffers[table] = []

    def flush(self) -> None:
        for table in list(self._buffers):
            self._flush(table)


def merge_schemas(*parts: Mapping[str, pa.Schema]) -> dict[str, pa.Schema]:
    merged: dict[str, pa.Schema] = {}
    for part in parts:
        for name, schema in part.items():
            if name in merged:
                raise StagingError(f"table {name} is declared twice")
            merged[name] = schema
    return merged


def load_json(tree: Path, artifact: str) -> object:
    """The parsed JSON file `artifact` of the tree."""
    try:
        return json.loads((tree / artifact).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as JSON: {error}") from error


def load_list(tree: Path, artifact: str) -> list[object]:
    """The parsed JSON file `artifact`, which must hold a list."""
    document = load_json(tree, artifact)
    if not isinstance(document, list):
        raise StagingError(f"{artifact}: expected a JSON array at the top level")
    return document

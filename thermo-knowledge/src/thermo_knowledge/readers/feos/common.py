# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Building blocks of the FeOS reader: documented columns, strict consumption of JSON objects and
a buffered sink over the writer.

Reading is strict. A JSON object is consumed key by key and a key no column declares raises an
error instead of being dropped, so a field a later FeOS release adds is a visible failure; an
explicit JSON `null` is refused, because the files never write one and a null column means
"the key is absent".
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
NOT_APPLICABLE = "not applicable"
IDENTIFIER_KEYS = ("cas", "name", "iupac_name", "smiles", "inchi", "formula")
"""The keys of FeOS's identifier object, in the order the files write them."""

JsonValue = object


def num(
    name: str,
    unit: str = NOT_STATED,
    *,
    source_name: str | None = None,
    note: str | None = None,
) -> pa.Field:
    text = NUMBER_NOTE if note is None else f"{NUMBER_NOTE}; {note}"
    return column(name, FLOAT64, source_name=source_name, unit=unit, note=text)


def text(
    name: str,
    *,
    source_name: str | None = None,
    note: str | None = None,
    nullable: bool = True,
    unit: str = NOT_STATED,
) -> pa.Field:
    return column(name, STRING, source_name=source_name, unit=unit, note=note, nullable=nullable)


def integer(
    name: str, *, source_name: str | None = None, unit: str = NOT_STATED, nullable: bool = True
) -> pa.Field:
    return column(name, INT64, source_name=source_name, unit=unit, nullable=nullable)


def position(name: str, source_name: str, *, nullable: bool = False) -> pa.Field:
    """An index the reader derives from position in the file (a record or element number)."""
    return column(name, INT64, source_name=source_name, unit=NOT_APPLICABLE, nullable=nullable)


def flag(name: str, *, source_name: str | None = None) -> pa.Field:
    return column(name, BOOL, source_name=source_name, unit=NOT_STATED)


def identifier_columns(prefix: str, source_prefix: str) -> list[pa.Field]:
    """The six columns of an identifier object under `source_prefix` (`identifier`, `id1`, ...)."""
    return [
        text(f"{prefix}_{key}", source_name=f"{source_prefix}/{key}") for key in IDENTIFIER_KEYS
    ]


def locator(artifact: str, pointer: str) -> str:
    return f"{artifact}#{pointer}"


def is_number(value: object) -> bool:
    return isinstance(value, int | float) and not isinstance(value, bool)


class Fields:
    """The keys of one JSON object, taken one by one; `done` refuses what nobody took."""

    def __init__(self, obj: object, where: str) -> None:
        if not isinstance(obj, dict):
            raise StagingError(f"{where}: expected a JSON object, found {type(obj).__name__}")
        for key, value in obj.items():
            if value is None:
                raise StagingError(f"{where}/{key}: explicit null; the files never write one")
        self._rest = dict(obj)
        self.where = where

    def present(self, key: str) -> bool:
        return key in self._rest

    def keys(self) -> list[str]:
        return list(self._rest)

    def peek(self, key: str) -> JsonValue:
        return self._rest.get(key)

    def take(self, key: str) -> JsonValue:
        return self._rest.pop(key, None)

    def number(self, key: str) -> float | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not is_number(value):
            raise StagingError(f"{self.where}/{key}: expected a number, found {value!r}")
        return float(value)

    def integer(self, key: str) -> int | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if isinstance(value, bool) or not isinstance(value, int):
            raise StagingError(f"{self.where}/{key}: expected an integer, found {value!r}")
        return value

    def string(self, key: str) -> str | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not isinstance(value, str):
            raise StagingError(f"{self.where}/{key}: expected a string, found {value!r}")
        return value

    def number_list(self, key: str) -> list[float] | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        return number_list(value, f"{self.where}/{key}")

    def object(self, key: str) -> dict[str, JsonValue] | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not isinstance(value, dict):
            raise StagingError(f"{self.where}/{key}: expected an object")
        return value

    def array(self, key: str) -> list[JsonValue] | None:
        value = self._rest.pop(key, None)
        if value is None:
            return None
        if not isinstance(value, list):
            raise StagingError(f"{self.where}/{key}: expected an array")
        return value

    def done(self) -> None:
        if self._rest:
            keys = ", ".join(sorted(self._rest))
            raise StagingError(
                f"{self.where}: fields no column declares: {keys}; extend the FeOS reader and "
                "bump its READER_VERSION"
            )


def number_list(value: object, where: str) -> list[float]:
    if not isinstance(value, list):
        raise StagingError(f"{where}: expected an array of numbers")
    out: list[float] = []
    for index, item in enumerate(value):
        if not is_number(item):
            raise StagingError(f"{where}[{index}]: expected a number, found {item!r}")
        out.append(float(item))  # type: ignore[arg-type]
    return out


def identifier_row(prefix: str, block: object, where: str) -> dict[str, JsonValue]:
    """The six identifier columns of an identifier object, null for a key the object lacks."""
    fields = Fields(block, where)
    row: dict[str, JsonValue] = {}
    for key in IDENTIFIER_KEYS:
        row[f"{prefix}_{key}"] = fields.string(key)
    fields.done()
    return row


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

    def _flush(self, table: str) -> None:
        buffer = self._buffers.get(table)
        if buffer:
            self.writer.rows(table, buffer)
            self._buffers[table] = []

    def flush(self) -> None:
        for table in list(self._buffers):
            self._flush(table)


def schema_of(*columns: pa.Field) -> pa.Schema:
    return table_schema(*columns)


def merge_schemas(*parts: Mapping[str, pa.Schema]) -> dict[str, pa.Schema]:
    merged: dict[str, pa.Schema] = {}
    for part in parts:
        for name, schema in part.items():
            if name in merged:
                raise StagingError(f"table {name} is declared twice")
            merged[name] = schema
    return merged


def load_array(tree: Path, artifact: str) -> list[JsonValue]:
    """The parsed JSON file `artifact`, which must hold an array."""
    try:
        document = json.loads((tree / artifact).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as JSON: {error}") from error
    if not isinstance(document, list):
        raise StagingError(f"{artifact}: expected a JSON array at the top level")
    return document

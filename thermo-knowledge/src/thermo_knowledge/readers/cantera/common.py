# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Building blocks of the Cantera reader: documented columns, JSON-pointer locators, typed
extraction of dictionary keys (a key whose value has another type stays in the dictionary and
reaches the long-form parameter table) and the leaf rows of any YAML value.
"""

from __future__ import annotations

from collections.abc import Iterator, Mapping
from dataclasses import dataclass, field

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import (
    BOOL,
    FLOAT64,
    INT64,
    NOT_STATED,
    STRING,
    column,
)
from thermo_knowledge.staging.writer import Writer

NOT_APPLICABLE = "not applicable"
UNIT_NOTE = (
    "Cantera resolves the unit of a quantity from the units mapping of its scope (entry, "
    "section, file), else SI with the kilomole; see units_entries"
)
MAX_EXACT_INT = 2**53
MAX_INT64 = 2**63

Value = object


# -- columns -----------------------------------------------------------------------------------


def col(
    name: str,
    dtype: pa.DataType,
    source_name: str | None = None,
    unit: str = NOT_STATED,
    note: str | None = None,
) -> pa.Field:
    return column(name, dtype, source_name=source_name, unit=unit, note=note)


def text(name: str, source_name: str | None = None, note: str | None = None) -> pa.Field:
    return col(name, STRING, source_name, note=note)


def number(name: str, source_name: str | None = None, note: str | None = None) -> pa.Field:
    """A float64 column for a YAML number (an integer spelling is stored as its float value)."""
    extra = "YAML number stored as float64; an integer spelling is stored as its float value"
    return col(name, FLOAT64, source_name, note=extra if note is None else f"{extra}; {note}")


def quantity(name: str, source_name: str | None = None) -> tuple[pa.Field, pa.Field]:
    """The pair of columns for a value the source writes either as a number (in the unit of its
    scope) or as a string with its own unit (`-393.5 kJ/mol`)."""
    return (
        number(name, source_name, UNIT_NOTE),
        text(
            f"{name}_text",
            source_name,
            "the value verbatim when the source writes it as a string with a unit; "
            "null when it is a number",
        ),
    )


def flag(name: str, source_name: str | None = None) -> pa.Field:
    return col(name, BOOL, source_name)


def integer(name: str, source_name: str, unit: str = NOT_APPLICABLE) -> pa.Field:
    return col(name, INT64, source_name, unit)


def strings(name: str, source_name: str | None = None) -> pa.Field:
    return col(name, pa.list_(STRING), source_name)


def floats(name: str, source_name: str | None = None, note: str | None = None) -> pa.Field:
    return col(name, pa.list_(FLOAT64), source_name, note=note)


VALUE_KINDS = (
    "string",
    "integer",
    "float",
    "boolean",
    "null",
    "empty_list",
    "empty_mapping",
)

VALUE_FIELDS: tuple[pa.Field, ...] = (
    col(
        "value_kind",
        STRING,
        "YAML type of the value",
        NOT_APPLICABLE,
        "one of " + ", ".join(VALUE_KINDS) + "; null is a key present with a null value",
    ),
    col("value_text", STRING, "string value, verbatim", note="set when value_kind is string"),
    col(
        "value_integer",
        INT64,
        "integer value",
        note="set when value_kind is integer; exact",
    ),
    col(
        "value_number",
        FLOAT64,
        "float value",
        note="set when value_kind is float; the unit is stated by the units mapping of the scope",
    ),
    col("value_bool", BOOL, "boolean value", NOT_APPLICABLE, "set when value_kind is boolean"),
)


def pointer(*tokens: object) -> str:
    """A JSON pointer from its tokens (keys are escaped, indexes are numbers)."""
    return "".join(f"/{str(token).replace('~', '~0').replace('/', '~1')}" for token in tokens)


def locator(artifact: str, position: str) -> str:
    return f"{artifact}#{position}"


# -- typed extraction --------------------------------------------------------------------------


def as_float(value: Value) -> float | None:
    """The YAML number `value` as a float; `None` for any other type."""
    if isinstance(value, bool) or not isinstance(value, int | float):
        return None
    if isinstance(value, int) and abs(value) > MAX_EXACT_INT:
        return None
    return float(value)


def as_floats(value: Value) -> list[float] | None:
    """A YAML list of numbers as floats; `None` for any other value."""
    if not isinstance(value, list):
        return None
    converted = [as_float(item) for item in value]
    if any(item is None for item in converted):
        return None
    return [item for item in converted if item is not None]


class Fields:
    """The keys of one YAML mapping. `string`, `number`, ... take a key when its value has the
    type and leave it otherwise; what is left is `rest`, which goes to the parameter table."""

    def __init__(self, mapping: Mapping[object, Value]) -> None:
        self.rest: dict[object, Value] = dict(mapping)

    def string(self, key: str) -> str | None:
        value = self.rest.get(key)
        if isinstance(value, str):
            del self.rest[key]
            return value
        return None

    def number(self, key: str) -> float | None:
        value = as_float(self.rest.get(key))
        if value is not None:
            del self.rest[key]
        return value

    def boolean(self, key: str) -> bool | None:
        value = self.rest.get(key)
        if isinstance(value, bool):
            del self.rest[key]
            return value
        return None

    def quantity(self, key: str) -> tuple[float | None, str | None]:
        value = self.rest.get(key)
        if isinstance(value, str):
            del self.rest[key]
            return None, value
        return self.number(key), None

    def strings(self, key: str) -> list[str] | None:
        value = self.rest.get(key)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            del self.rest[key]
            return list(value)
        return None

    def numbers(self, key: str) -> list[float] | None:
        converted = as_floats(self.rest.get(key))
        if converted is not None:
            del self.rest[key]
        return converted

    def mapping(self, key: str) -> dict[object, Value] | None:
        value = self.rest.get(key)
        if isinstance(value, dict):
            del self.rest[key]
            return value
        return None

    def take(self, key: str) -> Value:
        return self.rest.pop(key, None)


# -- leaves ------------------------------------------------------------------------------------


def leaf_columns(artifact: str, where: str, value: Value) -> dict[str, Value]:
    """The `value_*` columns of a scalar (or empty container) YAML value."""
    if value is None:
        return {"value_kind": "null"}
    if isinstance(value, bool):
        return {"value_kind": "boolean", "value_bool": value}
    if isinstance(value, int):
        if abs(value) >= MAX_INT64:
            raise StagingError(f"{artifact}#{where}: integer {value} does not fit 64 bits")
        return {"value_kind": "integer", "value_integer": value}
    if isinstance(value, float):
        return {"value_kind": "float", "value_number": value}
    if isinstance(value, str):
        return {"value_kind": "string", "value_text": value}
    if isinstance(value, list) and not value:
        return {"value_kind": "empty_list"}
    if isinstance(value, dict) and not value:
        return {"value_kind": "empty_mapping"}
    raise StagingError(
        f"{artifact}#{where}: a value of type {type(value).__name__} is not a YAML scalar"
    )


def walk(value: Value, base: str, relative: str = "") -> Iterator[tuple[str, str, str, Value]]:
    """The leaves of a YAML value, and the `units` mappings inside it.

    Yields `("leaf", pointer, relative pointer, leaf)` for every scalar, `null`, empty list or
    empty mapping, and `("units", units pointer, owner pointer, mapping)` for a mapping key
    named `units` whose value is a mapping: a units block is not a parameter of its owner (it
    belongs to `units_entries`), at any depth."""
    if isinstance(value, dict) and value:
        for key, child in value.items():
            token = pointer(key)
            if key == "units" and isinstance(child, dict):
                yield "units", base + token, base, child
            else:
                yield from walk(child, base + token, relative + token)
    elif isinstance(value, list) and value:
        for position, child in enumerate(value):
            token = pointer(position)
            yield from walk(child, base + token, relative + token)
    else:
        yield "leaf", base, relative, value


# -- sink --------------------------------------------------------------------------------------


@dataclass
class Sink:
    """Collects rows per table and hands them to the writer in batches."""

    writer: Writer
    batch: int = 20_000
    _buffers: dict[str, list[dict[str, Value]]] = field(default_factory=dict)

    def add(self, table: str, row: dict[str, Value]) -> None:
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


def merge_schemas(*parts: Mapping[str, pa.Schema]) -> dict[str, pa.Schema]:
    merged: dict[str, pa.Schema] = {}
    for part in parts:
        for name, schema in part.items():
            if name in merged:
                raise StagingError(f"table {name} is declared twice")
            merged[name] = schema
    return merged

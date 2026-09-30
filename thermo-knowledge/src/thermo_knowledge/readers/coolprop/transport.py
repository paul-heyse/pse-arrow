# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `TRANSPORT` block of a fluid file.

Record layouts vary widely between fluids, so the block is decomposed by its regular parts and
kept whole besides:

- `transport_objects`: one row per JSON object of the block (the block itself, `viscosity`,
  `viscosity/dilute`, each element of a model list, ...) with its type tag, reference keys, notes
  and the object verbatim as JSON text (`json`, nested objects included),
- `transport_scalars`: every other scalar field of each object, typed in long form (one row per
  field; the unit the source states sits in the object's own `<field>_units` row),
- `transport_rows`: the coefficient arrays of each object in long form, one nullable column per
  array name that occurs anywhere in the block.
"""

from __future__ import annotations

import json

import pyarrow as pa

from thermo_knowledge.readers.coolprop.common import (
    LENGTHS,
    NOT_APPLICABLE,
    Col,
    Sink,
    index,
    is_scalar,
    locator,
    num,
    pointer,
    text,
)
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import BOOL, STRING, table_schema

FLUID = text("fluid", source_name="INFO/NAME", nullable=False)
OBJECT_PATH = Col(
    "object_path",
    STRING,
    "JSON pointer of the object relative to TRANSPORT ('' for the block itself)",
    NOT_APPLICABLE,
    nullable=False,
)

TAGS = ("type", "BibTeX", "reference_fluid", "hardcoded", "_note", "note")
ARRAYS = (
    "A",
    "Aa",
    "Aaa",
    "Aaaa",
    "Adrdr",
    "Ai",
    "Aii",
    "Ar",
    "Arr",
    "Arrr",
    "B",
    "a",
    "b",
    "c_liq",
    "c_vap",
    "d",
    "d1",
    "d2",
    "f",
    "g",
    "gamma",
    "h",
    "l",
    "m",
    "n",
    "p",
    "q",
    "t",
    "t1",
    "t2",
)

SCHEMAS: dict[str, pa.Schema] = {
    "transport_objects": table_schema(
        FLUID.field(),
        OBJECT_PATH.field(),
        Col("parent_path", STRING, "JSON pointer of the enclosing object", NOT_APPLICABLE).field(),
        Col(
            "field", STRING, "key or list position of the object in its parent", NOT_APPLICABLE
        ).field(),
        *(text(name, source_name=name).field() for name in TAGS),
        Col(
            LENGTHS,
            STRING,
            "length of each list-valued field of the object, in source order (JSON)",
            NOT_APPLICABLE,
        ).field(),
        Col(
            "json",
            STRING,
            "the object as JSON text, nested objects included (keys in source order)",
            NOT_APPLICABLE,
        ).field(),
    ),
    "transport_scalars": table_schema(
        FLUID.field(),
        OBJECT_PATH.field(),
        Col("field", STRING, "key of the scalar field", NOT_APPLICABLE, nullable=False).field(),
        Col(
            "value_kind",
            STRING,
            "number, string, bool or null",
            NOT_APPLICABLE,
            nullable=False,
        ).field(),
        num(
            "value_number", "as stated by the object's <field>_units row where the source gives one"
        ).field(),
        text("value_string").field(),
        Col("value_bool", BOOL, "value").field(),
    ),
    "transport_rows": table_schema(
        FLUID.field(),
        OBJECT_PATH.field(),
        index("row_index", "position within the parallel arrays").field(),
        *(num(name, source_name=name).field() for name in ARRAYS),
    ),
}


def read_transport(sink: Sink, artifact: str, fluid: str, block: dict[str, object]) -> None:
    """Every object of the TRANSPORT block of one fluid."""
    _object(sink, artifact, fluid, block, path=(), parent=None, field_name="TRANSPORT")


def _object(
    sink: Sink,
    artifact: str,
    fluid: str,
    obj: dict[str, object],
    *,
    path: tuple[object, ...],
    parent: tuple[object, ...] | None,
    field_name: str,
) -> None:
    relative = pointer(*path)
    position = pointer("TRANSPORT", *path)
    where = locator(artifact, position)
    tags: dict[str, object] = {}
    arrays: dict[str, list[object]] = {}
    children: list[tuple[tuple[object, ...], str, dict[str, object]]] = []
    scalars: list[tuple[str, object]] = []
    for key, value in obj.items():
        if key in TAGS:
            if not isinstance(value, str):
                raise StagingError(f"{where}/{key}: expected a string")
            tags[key] = value
        elif isinstance(value, dict):
            children.append(((*path, key), key, value))
        elif isinstance(value, list):
            if value and all(isinstance(item, dict) for item in value):
                for item_index, item in enumerate(value):
                    children.append(((*path, key, item_index), f"{key}/{item_index}", item))
            elif all(is_scalar(item) for item in value):
                if key not in ARRAYS:
                    raise StagingError(f"{where}/{key}: a coefficient array no column declares")
                arrays[key] = value
            else:
                raise StagingError(f"{where}/{key}: a list that mixes objects and values")
        elif is_scalar(value):
            scalars.append((key, value))
        else:  # pragma: no cover - json.loads yields nothing else
            raise StagingError(f"{where}/{key}: unsupported value")

    sink.add(
        "transport_objects",
        {
            "_artifact": artifact,
            "_locator": where,
            "fluid": fluid,
            "object_path": relative,
            "parent_path": None if parent is None else pointer(*parent),
            "field": field_name,
            **tags,
            LENGTHS: json.dumps({k: len(v) for k, v in arrays.items()}, separators=(",", ":")),
            "json": json.dumps(obj, ensure_ascii=False, separators=(",", ":")),
        },
    )
    for key, value in scalars:
        row: dict[str, object] = {
            "_artifact": artifact,
            "_locator": locator(artifact, f"{position}{pointer(key)}"),
            "fluid": fluid,
            "object_path": relative,
            "field": key,
        }
        if value is None:
            row["value_kind"] = "null"
        elif isinstance(value, bool):
            row["value_kind"], row["value_bool"] = "bool", value
        elif isinstance(value, int | float):
            row["value_kind"], row["value_number"] = "number", value
        else:
            row["value_kind"], row["value_string"] = "string", value
        sink.add("transport_scalars", row)
    width = max((len(v) for v in arrays.values()), default=0)
    for row_index in range(width):
        entry: dict[str, object] = {
            "_artifact": artifact,
            "_locator": locator(artifact, f"{position}[{row_index}]"),
            "fluid": fluid,
            "object_path": relative,
            "row_index": row_index,
        }
        for name, values in arrays.items():
            if row_index < len(values):
                entry[name] = values[row_index]
        sink.add("transport_rows", entry)
    for child_path, child_field, child in children:
        _object(sink, artifact, fluid, child, path=child_path, parent=path, field_name=child_field)

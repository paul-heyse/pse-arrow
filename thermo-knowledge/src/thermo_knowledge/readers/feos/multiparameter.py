# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The multiparameter (Helmholtz) fluid records of `parameters/multiparameter/coolprop.json`.

A fluid record holds an identifier object (the name only), `molarweight`, `tc`, `rhoc` and two
lists of type-tagged term objects, `ideal_gas` and `residual`. A term object is flat: a `type`
string, scalar fields and parallel arrays. Each term is one row of `multiparameter_terms`
(scalars, plus `array_lengths`, the length of each array in source order, so an empty array and
an absent one stay distinct) and each index of its arrays is one row of
`multiparameter_term_rows`, one nullable column per array name; a term whose arrays differ in
length is refused. The JSON writes some numbers as integers, stored here as float64.
"""

from __future__ import annotations

import json
from pathlib import Path

from thermo_knowledge.readers.feos.common import (
    NOT_APPLICABLE,
    Fields,
    JsonValue,
    Sink,
    identifier_columns,
    identifier_row,
    integer,
    is_number,
    load_array,
    locator,
    num,
    position,
    schema_of,
    text,
)
from thermo_knowledge.staging.errors import StagingError

MULTIPARAMETER_FILE = "parameters/multiparameter/coolprop.json"

SECTIONS = ("ideal_gas", "residual")
SCALARS = ("a", "a1", "a2", "T0", "Tc", "Tcrit", "R", "cp_over_R")
TEXTS = ("Tcrit_units", "reference")
NOTE_FIELD = "_note"
ARRAYS = (
    "n",
    "t",
    "c",
    "v",
    "d",
    "l",
    "m",
    "g",
    "beta",
    "epsilon",
    "eta",
    "gamma",
    "A",
    "B",
    "C",
    "D",
    "a",
    "b",
    "gd",
    "gt",
    "ld",
    "lt",
)

_RECORD = position("record_index", "position of the fluid record in the file's JSON array")
_SECTION = text(
    "section", source_name="ideal_gas or residual (the key of the term list)", nullable=False
)
_TERM = position("term_index", "position of the term object in its list")
_TYPE = text("type", source_name="type", nullable=False)

MULTIPARAMETER_FLUIDS = schema_of(
    _RECORD,
    *identifier_columns("identifier", "identifier"),
    num("molarweight", "g/mol", note="FeOS reads it as g/mol"),
    num("tc", note="reducing temperature; FeOS states no unit"),
    num("rhoc", note="reducing density; FeOS states no unit"),
    integer("ideal_gas_term_count", source_name="ideal_gas (length of the array)"),
    integer("residual_term_count", source_name="residual (length of the array)"),
)

MULTIPARAMETER_TERMS = schema_of(
    _RECORD,
    text("fluid", source_name="identifier/name", unit=NOT_APPLICABLE),
    _SECTION,
    _TERM,
    _TYPE,
    *(num(name, note="scalar field of a term object") for name in SCALARS),
    text("Tcrit_units", source_name="Tcrit_units", note="the unit the record states for Tcrit"),
    text("reference", source_name="reference", note="named reference state of an offset term"),
    text("note_field", source_name="_note"),
    text(
        "array_lengths",
        source_name="length of each array field of the term, in source order (JSON)",
        unit=NOT_APPLICABLE,
    ),
)

MULTIPARAMETER_TERM_ROWS = schema_of(
    _RECORD,
    text("fluid", source_name="identifier/name", unit=NOT_APPLICABLE),
    _SECTION,
    _TERM,
    _TYPE,
    position("row_index", "position within the parallel arrays of the term"),
    *(num(name, note="element of the array field of a term object") for name in ARRAYS),
)

SCHEMAS = {
    "multiparameter_fluids": MULTIPARAMETER_FLUIDS,
    "multiparameter_terms": MULTIPARAMETER_TERMS,
    "multiparameter_term_rows": MULTIPARAMETER_TERM_ROWS,
}


def read_multiparameter(tree: Path, artifact: str, sink: Sink) -> None:
    for index, record in enumerate(load_array(tree, artifact)):
        where = locator(artifact, f"/{index}")
        fields = Fields(record, where)
        identifier = identifier_row("identifier", fields.take("identifier"), f"{where}/identifier")
        fluid = identifier["identifier_name"]
        row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": where,
            "record_index": index,
            **identifier,
            "molarweight": fields.number("molarweight"),
            "tc": fields.number("tc"),
            "rhoc": fields.number("rhoc"),
        }
        for section in SECTIONS:
            terms = fields.array(section)
            if terms is None:
                raise StagingError(f"{where}/{section}: missing")
            row[f"{section}_term_count"] = len(terms)
            for term_index, term in enumerate(terms):
                _term(artifact, index, str(fluid), section, term_index, term, sink)
        fields.done()
        sink.add("multiparameter_fluids", row)


def _term(
    artifact: str,
    index: int,
    fluid: str,
    section: str,
    term_index: int,
    term: JsonValue,
    sink: Sink,
) -> None:
    where = locator(artifact, f"/{index}/{section}/{term_index}")
    fields = Fields(term, where)
    kind = fields.string("type")
    if kind is None:
        raise StagingError(f"{where}: a term without type")
    keys = {
        "_artifact": artifact,
        "record_index": index,
        "fluid": fluid,
        "section": section,
        "term_index": term_index,
        "type": kind,
    }
    scalar_row: dict[str, JsonValue] = {"_locator": where, **keys}
    for name in SCALARS:
        scalar_row[name] = None
    for name in TEXTS:
        scalar_row[name] = None
    scalar_row["note_field"] = None
    arrays: dict[str, list[float]] = {}
    for name in fields.keys():
        if isinstance(fields.peek(name), list):
            if name not in ARRAYS:
                raise StagingError(f"{where}/{name}: an array no column declares")
            values = fields.array(name) or []
            if not all(is_number(item) for item in values):
                raise StagingError(f"{where}/{name}: expected an array of numbers")
            arrays[name] = [float(item) for item in values]  # type: ignore[arg-type]
        elif name in SCALARS:
            scalar_row[name] = fields.number(name)
        elif name in TEXTS:
            scalar_row[name] = fields.string(name)
        elif name == NOTE_FIELD:
            scalar_row["note_field"] = fields.string(name)
    fields.done()
    lengths = {len(values) for values in arrays.values()}
    if len(lengths) > 1:
        shown = {name: len(values) for name, values in arrays.items()}
        raise StagingError(f"{where}: parallel arrays of different lengths: {shown}")
    scalar_row["array_lengths"] = json.dumps(
        {name: len(values) for name, values in arrays.items()}, separators=(",", ":")
    )
    sink.add("multiparameter_terms", scalar_row)
    for row_index in range(max(lengths, default=0)):
        sink.add(
            "multiparameter_term_rows",
            {
                "_locator": f"{where}[{row_index}]",
                **keys,
                "row_index": row_index,
                **{name: values[row_index] for name, values in arrays.items()},
            },
        )

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Fitted incompressible-liquid records (`dev/incompressible_liquids/json/*.json`).

Each record holds validity ranges and base values (`incompressible_fluids`) and one function object
per property `{type, coeffs, NRMS}` (`incompressible_functions`). `coeffs` is a number list
(one-dimensional), a list of number lists (two-dimensional, rows of equal length) or the string
`notdefined`; the numbers are in `incompressible_coefficients`, one row per number.
"""

from __future__ import annotations

from pathlib import Path

from thermo_knowledge.readers.coolprop.common import (
    NOT_APPLICABLE,
    Col,
    Fields,
    Sink,
    index,
    load_json,
    locator,
    merge_schemas,
    num,
    pointer,
    text,
)
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import INT64, STRING, table_schema

FLUID = text("fluid", source_name="name", nullable=False)
PROPERTY = Col(
    "property", STRING, "key of the function object in the record", NOT_APPLICABLE, nullable=False
)

SCALARS = (
    ("name", text("name")),
    ("description", text("description")),
    ("reference", text("reference")),
    ("xid", text("xid")),
    ("xmin", num("x_min", source_name="xmin")),
    ("xmax", num("x_max", source_name="xmax")),
    ("xbase", num("xbase")),
    ("Tbase", num("Tbase")),
    ("Tmin", num("Tmin")),
    ("Tmax", num("Tmax")),
    ("TminPsat", num("TminPsat")),
)
FUNCTIONS = (
    "density",
    "specific_heat",
    "viscosity",
    "conductivity",
    "saturation_pressure",
    "T_freeze",
    "mass2input",
    "volume2input",
    "mole2input",
)

SCHEMAS = merge_schemas(
    {
        "incompressible_fluids": table_schema(*(spec.field() for _, spec in SCALARS)),
        "incompressible_functions": table_schema(
            FLUID.field(),
            PROPERTY.field(),
            text("type").field(),
            num("NRMS").field(),
            Col(
                "coeffs_kind",
                STRING,
                "shape of coeffs: list, list_of_lists or string",
                NOT_APPLICABLE,
                nullable=False,
            ).field(),
            text(
                "coeffs_text", source_name="coeffs", note="the value when coeffs is a string"
            ).field(),
            Col("n_rows", INT64, "length of coeffs", NOT_APPLICABLE).field(),
            Col("n_cols", INT64, "length of each row of a list of lists", NOT_APPLICABLE).field(),
        ),
        "incompressible_coefficients": table_schema(
            FLUID.field(),
            PROPERTY.field(),
            index("row_index", "position in coeffs").field(),
            index("col_index", "position in the row of a list of lists").field(),
            num("value", source_name="coeffs element").field(),
        ),
    }
)


def read_incompressible(tree: Path, artifact: str, sink: Sink) -> None:
    body = Fields(load_json(tree, artifact), artifact)
    record: dict[str, object] = {"_artifact": artifact, "_locator": locator(artifact, "")}
    for key, spec in SCALARS:
        record[spec.name] = body.take(key)
    functions = {name: body.take_object(name) for name in FUNCTIONS}
    body.done()
    fluid = record["name"]
    if not isinstance(fluid, str):
        raise StagingError(f"{artifact}: name is required")
    sink.add("incompressible_fluids", record)
    for name, function in functions.items():
        if function is not None:
            _function(sink, artifact, fluid, name, function)


def _function(
    sink: Sink, artifact: str, fluid: str, name: str, function: dict[str, object]
) -> None:
    where = locator(artifact, pointer(name))
    body = Fields(function, where)
    coeffs = body.take("coeffs")
    row: dict[str, object] = {
        "_artifact": artifact,
        "_locator": where,
        "fluid": fluid,
        "property": name,
        "type": body.take("type"),
        "NRMS": body.take("NRMS"),
    }
    body.done()
    cells: list[tuple[int, int | None, object]] = []
    if isinstance(coeffs, str):
        row["coeffs_kind"], row["coeffs_text"] = "string", coeffs
    elif isinstance(coeffs, list) and all(isinstance(item, list) for item in coeffs) and coeffs:
        widths = {len(item) for item in coeffs}
        if len(widths) != 1:
            raise StagingError(f"{where}/coeffs: rows of different lengths")
        row["coeffs_kind"], row["n_rows"], row["n_cols"] = (
            "list_of_lists",
            len(coeffs),
            widths.pop(),
        )
        for i, item in enumerate(coeffs):
            cells.extend((i, j, value) for j, value in enumerate(item))
    elif isinstance(coeffs, list) and not any(isinstance(item, list | dict) for item in coeffs):
        row["coeffs_kind"], row["n_rows"] = "list", len(coeffs)
        cells.extend((i, None, value) for i, value in enumerate(coeffs))
    else:
        raise StagingError(f"{where}/coeffs: expected a string, a list or a list of lists")
    sink.add("incompressible_functions", row)
    for i, j, value in cells:
        position = pointer(name, "coeffs", i) if j is None else pointer(name, "coeffs", i, j)
        sink.add(
            "incompressible_coefficients",
            {
                "_artifact": artifact,
                "_locator": locator(artifact, position),
                "fluid": fluid,
                "property": name,
                "row_index": i,
                "col_index": j,
                "value": value,
            },
        )

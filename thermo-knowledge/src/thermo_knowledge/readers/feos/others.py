# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""FeOS files whose records are neither pure nor binary parameter records: the DIPPR ideal-gas
records, the Joback group records, the group-contribution chemical records and the SMARTS
records.

- DIPPR record: an identifier object and one key naming the equation (`DIPPR100`, `DIPPR107`)
  that holds the coefficient list; the key is kept as `equation`.
- Joback record: the identifier is the segment name, then `molarweight` and the coefficients
  `a` to `e`.
- Chemical record: an identifier object, a `segments` list of segment names and an optional
  `bonds` list of index pairs (one row per bond in `chemical_record_bonds`; `bond_count` is null
  when the key is absent).
- SMARTS record: `group`, `smarts` and an optional integer `max`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.feos.common import (
    Fields,
    JsonValue,
    Sink,
    identifier_columns,
    identifier_row,
    integer,
    load_array,
    locator,
    num,
    position,
    schema_of,
    text,
)
from thermo_knowledge.readers.feos.records import MODEL_DIRECTORY, RECORD_INDEX, model_directory
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import column

DIPPR_FILES = (
    "parameters/ideal_gas/poling2000.json",
    "parameters/ideal_gas/burkhardt2025.json",
)
JOBACK_FILE = "parameters/ideal_gas/joback1987.json"
CHEMICAL_FILE = "parameters/pcsaft/gc_substances.json"
SMARTS_FILE = "parameters/pcsaft/sauer2014_smarts.json"

DIPPR_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    *identifier_columns("identifier", "identifier"),
    text("equation", source_name="the key that holds the coefficient list (DIPPR100, DIPPR107)"),
    column(
        "coefficients",
        pa.list_(pa.float64()),
        source_name="<equation key>",
        unit="not stated",
        note=(
            "coefficients in source order; FeOS states that its DIPPR equations use T in K and "
            "give the isobaric heat capacity in J/kmol/K, and states no unit per coefficient"
        ),
    ),
)

JOBACK_GROUPS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    text("segment", source_name="identifier (a plain string: the group name)"),
    num("molarweight", "g/mol", note="FeOS reads it as g/mol"),
    num("a", source_name="a"),
    num("b", source_name="b"),
    num("c", source_name="c"),
    num("d", source_name="d"),
    num("e", source_name="e"),
)

CHEMICAL_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    *identifier_columns("identifier", "identifier"),
    column(
        "segments",
        pa.list_(pa.string()),
        source_name="segments",
        unit="not applicable",
        note="segment names in source order; a name repeats for each instance of the segment",
    ),
    integer("bond_count", source_name="bonds (length of the array)"),
)

CHEMICAL_RECORD_BONDS = schema_of(
    RECORD_INDEX,
    position("bond_index", "position in the bonds array"),
    integer(
        "segment_index_1",
        source_name="bonds[i][0]",
        unit="not applicable",
        nullable=False,
    ),
    integer(
        "segment_index_2",
        source_name="bonds[i][1]",
        unit="not applicable",
        nullable=False,
    ),
)

SMARTS_RECORDS = schema_of(
    MODEL_DIRECTORY,
    RECORD_INDEX,
    text("group", source_name="group", nullable=False),
    text("smarts", source_name="smarts", nullable=False),
    integer("max", source_name="max", unit="not applicable"),
)

SCHEMAS = {
    "dippr_records": DIPPR_RECORDS,
    "joback_groups": JOBACK_GROUPS,
    "chemical_records": CHEMICAL_RECORDS,
    "chemical_record_bonds": CHEMICAL_RECORD_BONDS,
    "smarts_records": SMARTS_RECORDS,
}


def _records(tree: Path, artifact: str) -> list[tuple[int, str, JsonValue]]:
    return [
        (index, locator(artifact, f"/{index}"), record)
        for index, record in enumerate(load_array(tree, artifact))
    ]


def read_dippr(tree: Path, artifact: str, sink: Sink) -> None:
    directory = model_directory(artifact)
    for index, where, record in _records(tree, artifact):
        fields = Fields(record, where)
        row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": where,
            "model_directory": directory,
            "record_index": index,
            **identifier_row("identifier", fields.take("identifier"), f"{where}/identifier"),
        }
        rest = fields.keys()
        if len(rest) != 1 or not rest[0].startswith("DIPPR"):
            raise StagingError(f"{where}: expected one DIPPR<number> key, found {rest}")
        row["equation"] = rest[0]
        row["coefficients"] = fields.number_list(rest[0])
        fields.done()
        sink.add("dippr_records", row)


def read_joback(tree: Path, artifact: str, sink: Sink) -> None:
    directory = model_directory(artifact)
    for index, where, record in _records(tree, artifact):
        fields = Fields(record, where)
        segment = fields.string("identifier")
        if segment is None:
            raise StagingError(f"{where}/identifier: expected a string")
        row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": where,
            "model_directory": directory,
            "record_index": index,
            "segment": segment,
        }
        for name in ("molarweight", "a", "b", "c", "d", "e"):
            row[name] = fields.number(name)
        fields.done()
        sink.add("joback_groups", row)


def read_chemicals(tree: Path, artifact: str, sink: Sink) -> None:
    directory = model_directory(artifact)
    for index, where, record in _records(tree, artifact):
        fields = Fields(record, where)
        row: dict[str, JsonValue] = {
            "_artifact": artifact,
            "_locator": where,
            "model_directory": directory,
            "record_index": index,
            **identifier_row("identifier", fields.take("identifier"), f"{where}/identifier"),
        }
        segments = fields.array("segments")
        if segments is None or not all(isinstance(item, str) for item in segments):
            raise StagingError(f"{where}/segments: expected an array of strings")
        row["segments"] = list(segments)
        bonds = fields.array("bonds")
        row["bond_count"] = None if bonds is None else len(bonds)
        for bond_index, bond in enumerate(bonds or []):
            bond_where = f"{where}/bonds/{bond_index}"
            if (
                not isinstance(bond, list)
                or len(bond) != 2
                or not all(isinstance(item, int) and not isinstance(item, bool) for item in bond)
            ):
                raise StagingError(f"{bond_where}: expected a pair of segment indexes")
            sink.add(
                "chemical_record_bonds",
                {
                    "_artifact": artifact,
                    "_locator": bond_where,
                    "record_index": index,
                    "bond_index": bond_index,
                    "segment_index_1": bond[0],
                    "segment_index_2": bond[1],
                },
            )
        fields.done()
        sink.add("chemical_records", row)


def read_smarts(tree: Path, artifact: str, sink: Sink) -> None:
    directory = model_directory(artifact)
    for index, where, record in _records(tree, artifact):
        fields = Fields(record, where)
        group, smarts = fields.string("group"), fields.string("smarts")
        if group is None or smarts is None:
            raise StagingError(f"{where}: a SMARTS record needs group and smarts")
        sink.add(
            "smarts_records",
            {
                "_artifact": artifact,
                "_locator": where,
                "model_directory": directory,
                "record_index": index,
                "group": group,
                "smarts": smarts,
                "max": fields.integer("max"),
            },
        )
        fields.done()

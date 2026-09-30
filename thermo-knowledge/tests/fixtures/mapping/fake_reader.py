# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A fixture reader for the mapping tests: three JSON files as tables and a note it skips."""

from __future__ import annotations

import json
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging.schema import FLOAT64, INT64, STRING, column, table_schema
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES = {
    "species": table_schema(
        column("name", STRING, source_name="name", nullable=False),
        column("cas", STRING, source_name="cas"),
        column("inchikey", STRING, source_name="inchikey"),
        column("inchi", STRING, source_name="inchi"),
        column("placeholder", STRING, source_name="placeholder"),
        column("aliases", pa.list_(STRING), source_name="aliases"),
    ),
    "curves": table_schema(
        column("species", STRING, source_name="species", nullable=False),
        column("curve", STRING, source_name="curve", nullable=False),
        column("T_r", FLOAT64, source_name="T_r"),
        column("p_bar", FLOAT64, source_name="p_bar", unit="bar"),
    ),
    "coefficients": table_schema(
        column("species", STRING, source_name="species", nullable=False),
        column("curve", STRING, source_name="curve", nullable=False),
        column("pos", INT64, source_name="pos"),
        column("n", FLOAT64, source_name="n"),
        column("t", FLOAT64, source_name="t"),
    ),
    "notes": table_schema(column("text", STRING, source_name="line")),
}


def read(tree: Path, writer: Writer) -> None:
    for table in ("species", "curves", "coefficients"):
        artifact = f"data/{table}.json"
        records = json.loads((tree / artifact).read_text())
        names = [field.name for field in TABLES[table] if not field.name.startswith("_")]
        writer.rows(
            table,
            (
                {"_artifact": artifact, "_locator": f"{artifact}#/{index}"}
                | {name: record.get(name) for name in names}
                for index, record in enumerate(records)
            ),
        )
    lines = (tree / "notes.txt").read_text().splitlines()
    writer.rows(
        "notes",
        (
            {"_artifact": "notes.txt", "_locator": f"notes.txt#L{number}", "text": line}
            for number, line in enumerate(lines, 1)
        ),
    )

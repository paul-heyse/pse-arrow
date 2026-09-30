# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A fixture reader for the framework tests: JSON, CSV, text and a binary file it skips."""

from __future__ import annotations

import csv
import json
from pathlib import Path

from thermo_knowledge.staging.schema import (
    BOOL,
    FLOAT64,
    INT64,
    STRING,
    column,
    table_schema,
)
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES = {
    "alpha": table_schema(
        column("id", STRING, source_name="id", nullable=False),
        column("name", STRING, source_name="name"),
        column("flag", BOOL, source_name="flag"),
        column("count", INT64, source_name="count"),
    ),
    "alpha_values": table_schema(
        column("id", STRING, source_name="id", nullable=False),
        column("row_index", INT64, source_name="position in values", unit="not applicable"),
        column("values", FLOAT64, source_name="values[]", unit="m", note="integers as float64"),
    ),
    "beta": table_schema(
        column("key", STRING, source_name="key"),
        column("amount", FLOAT64, source_name="amount", unit="kg or mol per the unit column"),
        column("unit", STRING, source_name="unit"),
    ),
    "notes": table_schema(column("text", STRING, source_name="first line")),
}


def read(tree: Path, writer: Writer) -> None:
    alpha = "data/alpha.json"
    records = json.loads((tree / alpha).read_text())
    writer.rows(
        "alpha",
        (
            {
                "_artifact": alpha,
                "_locator": f"{alpha}#/{index}",
                "id": record["id"],
                "name": record["name"],
                "flag": record["flag"],
                "count": record.get("count"),
            }
            for index, record in enumerate(records)
        ),
    )
    writer.rows(
        "alpha_values",
        (
            {
                "_artifact": alpha,
                "_locator": f"{alpha}#/{index}/values[{row}]",
                "id": record["id"],
                "row_index": row,
                "values": value,
            }
            for index, record in enumerate(records)
            for row, value in enumerate(record["values"])
        ),
    )
    beta = "data/beta.csv"
    with (tree / beta).open(newline="") as handle:
        table = list(csv.DictReader(handle))
    writer.rows(
        "beta",
        (
            {
                "_artifact": beta,
                "_locator": f"{beta}#L{number}",
                "key": row["key"],
                "amount": float(row["amount"]) if row["amount"] else None,
                "unit": row["unit"],
            }
            for number, row in enumerate(table, start=2)
        ),
    )
    notes = "notes.txt"
    first = (tree / notes).read_text().splitlines()[0]
    writer.rows("notes", [{"_artifact": notes, "_locator": f"{notes}#L1", "text": first}])
    writer.partly_read(notes, "only the first line is read")
    writer.skipped("blob/raw.bin", "opaque binary blob, not a table")

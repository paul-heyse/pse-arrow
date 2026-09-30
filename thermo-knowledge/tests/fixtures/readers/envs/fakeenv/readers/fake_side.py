# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A fake side reader for the protocol test. It imports nothing from thermo_knowledge: it reads
the tiny fixture tree and writes Parquet files and tables.json as the protocol specifies."""

import argparse
import json
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

READER_VERSION = "1"


def field(name, dtype, source_name, unit="not stated", nullable=True):
    return pa.field(
        name, dtype, nullable=nullable, metadata={"source_name": source_name, "unit": unit}
    )


SCHEMA = pa.schema(
    [
        field(
            "_artifact",
            pa.string(),
            "file path relative to the acquired tree",
            "not applicable",
            False,
        ),
        field("_locator", pa.string(), "<artifact>#<position>", "not applicable", False),
        field("id", pa.string(), "id", nullable=False),
        field("values", pa.list_(pa.float64()), "values", "m"),
    ]
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree", required=True)
    parser.add_argument("--out", required=True)
    arguments = parser.parse_args()
    tree, out = Path(arguments.tree), Path(arguments.out)
    artifact = "data/alpha.json"
    records = json.loads((tree / artifact).read_text())
    table = pa.table(
        {
            "_artifact": [artifact] * len(records),
            "_locator": [f"{artifact}#/{i}" for i in range(len(records))],
            "id": [r["id"] for r in records],
            "values": [[float(v) for v in r["values"]] for r in records],
        },
        schema=SCHEMA,
    )
    pq.write_table(table, out / "gamma.parquet")
    described = {
        "protocol": 1,
        "reader_version": READER_VERSION,
        "tables": {
            "gamma": {
                "file": "gamma.parquet",
                "schema": [
                    {
                        "name": f.name,
                        "type": "list<float64>" if pa.types.is_list(f.type) else "string",
                        "nullable": f.nullable,
                        "metadata": {k.decode(): v.decode() for k, v in f.metadata.items()},
                    }
                    for f in SCHEMA
                ],
            }
        },
        "files": {
            "data/beta.csv": {"status": "skipped", "reason": "fake reader ignores CSV"},
            "notes.txt": {"status": "partly_read", "reason": "fake reader ignores text"},
            "blob/raw.bin": {"status": "skipped", "reason": "binary"},
        },
    }
    (out / "tables.json").write_text(json.dumps(described))


if __name__ == "__main__":
    main()

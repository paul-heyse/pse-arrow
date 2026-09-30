# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Reproduce the published C1-C20 PC-SAFT bank from its frozen FeOS artifact."""

from __future__ import annotations

import hashlib
import json
from decimal import Decimal
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

SOURCE_SHA256 = "f4b4018c7f02341b937c086cbb38626b72c1f3f3d9677a26b463112e16dfd404"
NAMES = (
    "methane",
    "ethane",
    "propane",
    "butane",
    "pentane",
    "hexane",
    "heptane",
    "octane",
    "nonane",
    "decane",
    "undecane",
    "dodecane",
    "tridecane",
    "tetradecane",
    "pentadecane",
    "hexadecane",
    "heptadecane",
    "octadecane",
    "nonadecane",
    "eicosane",
)


def main() -> None:
    bank = (
        Path(__file__).resolve().parents[1]
        / "packages/reference/data/gross-sadowski-2001/data"
    )
    content = (bank / "gross2001.json").read_bytes()
    if hashlib.sha256(content).hexdigest() != SOURCE_SHA256:
        raise ValueError(
            "the frozen FeOS 0.10.1 Gross-Sadowski artifact checksum differs"
        )
    rows = json.loads(content, parse_float=Decimal)
    by_name = {row["identifier"]["name"]: row for row in rows}
    selected = [by_name[name] for name in NAMES]
    schema = pa.schema(
        [
            pa.field("subject", pa.string(), nullable=False),
            pa.field("m", pa.float64(), nullable=False),
            pa.field("sigma", pa.float64(), nullable=False),
            pa.field("epsilon", pa.float64(), nullable=False),
        ]
    )
    columns = [
        pa.array([row["identifier"]["cas"] for row in selected], type=pa.string()),
        pa.array([float(row["m"]) for row in selected], type=pa.float64()),
        pa.array(
            [float(Decimal(row["sigma"]) * Decimal("1e-10")) for row in selected],
            type=pa.float64(),
        ),
        pa.array([float(row["epsilon_k"]) for row in selected], type=pa.float64()),
    ]
    pq.write_table(
        pa.Table.from_arrays(columns, schema=schema),
        bank / "parameters.parquet",
        compression="zstd",
        version="2.6",
    )


if __name__ == "__main__":
    main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Plan 23 paired feeds and complete independent native outcomes; no failure filtering."""

from __future__ import annotations

import json
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import scipy
from scipy.stats import qmc

ROOT = Path(__file__).resolve().parents[1]
FEEDS = ROOT / "packages/reference/campaign/data/flash-feeds.parquet"
COUNT = 200


def generate_feeds() -> None:
    """Freeze the declared sampler once, before admission and independent of outcomes."""
    sampler = qmc.LatinHypercube(
        d=3, scramble=True, strength=1, optimization=None, rng=23
    )
    points = qmc.scale(
        sampler.random(COUNT), [350.0, 80000.0, 0.05], [400.0, 120000.0, 0.95]
    )
    schema = pa.schema(
        [
            pa.field("index", pa.int64()),
            pa.field("temperature", pa.float64(), metadata={b"unit": b"K"}),
            pa.field("pressure", pa.float64(), metadata={b"unit": b"Pa"}),
            pa.field("benzene", pa.float64(), metadata={b"unit": b"1"}),
        ],
        metadata={
            b"generator": json.dumps(
                {
                    "scipy": scipy.__version__,
                    "sampler": "LatinHypercube",
                    "rng": 23,
                    "scramble": True,
                    "strength": 1,
                    "optimization": None,
                    "count": COUNT,
                }
            ).encode()
        },
    )
    FEEDS.parent.mkdir(parents=True, exist_ok=True)
    pq.write_table(
        pa.Table.from_arrays(
            [
                pa.array(range(COUNT), type=pa.int64()),
                *(pa.array(points[:, i]) for i in range(3)),
            ],
            schema=schema,
        ),
        FEEDS,
    )


if __name__ == "__main__":
    generate_feeds()

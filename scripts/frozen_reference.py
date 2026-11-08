# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only comparison and explicitly selected replacement of reference goldens."""

from __future__ import annotations

import argparse
import json
import math
from typing import TYPE_CHECKING

import pyarrow.parquet as pq

if TYPE_CHECKING:
    from pathlib import Path

    import pyarrow as pa


def regenerate_selected() -> bool:
    """Default to comparison and require an explicit selection to replace a golden."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--regenerate", action="store_true", help="replace the retained golden"
    )
    return bool(parser.parse_args().regenerate)


def table_reference(
    path: Path,
    actual: pa.Table,
    regenerate: bool,
    allowances: dict[str, tuple[float, float]],
    *,
    check_metadata: bool = True,
    compression: str = "zstd",
) -> None:
    """Compare typed case coordinates exactly and give only results numeric slack."""
    if regenerate:
        pq.write_table(actual, path, compression=compression, version="2.6")
        return
    frozen = pq.read_table(path)
    if not actual.schema.equals(frozen.schema, check_metadata=check_metadata):
        raise ValueError(f"stale reference schema or provenance in {path}")
    if actual.num_rows != frozen.num_rows:
        raise ValueError(f"stale reference population in {path}")
    for name in actual.column_names:
        for row, (observed, expected) in enumerate(
            zip(actual[name].to_pylist(), frozen[name].to_pylist(), strict=True)
        ):
            if observed is None or expected is None:
                same = False
            elif name in allowances:
                relative, absolute = allowances[name]
                same = (
                    math.isfinite(observed)
                    and math.isfinite(expected)
                    and math.isclose(
                        observed, expected, rel_tol=relative, abs_tol=absolute
                    )
                )
            else:
                same = observed == expected and (
                    not isinstance(observed, float) or math.isfinite(observed)
                )
            if not same:
                raise ValueError(
                    f"stale reference {path} at row {row}, {name}: frozen {expected}, observed {observed}"
                )


def json_reference(path: Path, actual: dict[str, object], regenerate: bool) -> None:
    """Retain exact JSON reference fields unless the caller selects regeneration."""
    if regenerate:
        path.write_text(json.dumps(actual, indent=2, allow_nan=False) + "\n")
    elif json.loads(path.read_text()) != actual:
        raise ValueError(f"stale JSON reference {path}")

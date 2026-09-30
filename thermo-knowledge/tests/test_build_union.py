# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The columnar union of a canonical table across files (pipeline section 3, step 2)."""

from __future__ import annotations

import time
import uuid
from pathlib import Path

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from thermo_knowledge.build.union import (
    Contribution,
    UnionConflictError,
    equal,
    union_table,
)

SCHEMA = pa.schema(
    [
        pa.field("id", pa.uuid(), nullable=False),
        pa.field("label", pa.string()),
        pa.field("value", pa.float64()),
        pa.field("axis", pa.list_(pa.float64())),
        pa.field("hash", pa.binary(32)),
        pa.field("when", pa.timestamp("us", tz="UTC")),
    ]
)
NAMESPACE = uuid.UUID("12345678-1234-5678-1234-567812345678")


def ident(n: int) -> uuid.UUID:
    return uuid.uuid5(NAMESPACE, str(n))


def row(n: int, **over: object) -> dict[str, object]:
    base: dict[str, object] = {
        "id": ident(n),
        "label": f"label {n}",
        "value": float(n),
        "axis": [float(n), 2.0],
        "hash": bytes([n % 256]) * 32,
        "when": None,
    }
    return {**base, **over}


def write(path: Path, rows: list[dict[str, object]]) -> Path:
    data = pa.table({name: [r[name] for r in rows] for name in SCHEMA.names}, schema=SCHEMA)
    pq.write_table(data, path)
    return path


def union(tmp_path: Path, *sources: list[dict[str, object]], keys: tuple[str, ...] = ("id",)):  # type: ignore[no-untyped-def]
    tmp_path.mkdir(parents=True, exist_ok=True)
    contributions = [
        Contribution(f"s{index}", write(tmp_path / f"s{index}.parquet", rows))
        for index, rows in enumerate(sources)
    ]
    return union_table("t.t", keys, contributions, tmp_path / "work"), contributions


def test_distinct_keys_load_from_the_original_files(tmp_path: Path) -> None:
    result, contributions = union(tmp_path, [row(1), row(2)], [row(3)])
    assert (result.rows, result.merged) == (3, 0)
    assert result.parts == tuple(c.path for c in contributions)
    assert not (tmp_path / "work").exists()


def test_identical_rows_are_one_row_whichever_source_comes_first(tmp_path: Path) -> None:
    result, _ = union(tmp_path, [row(1), row(2)], [row(2), row(3)], [row(2)])
    assert (result.rows, result.merged) == (3, 2)
    (part,) = result.parts
    merged = pq.read_table(part)
    assert merged.schema.equals(SCHEMA, check_metadata=False)
    assert sorted(merged.column("label").to_pylist()) == ["label 1", "label 2", "label 3"]
    assert merged.column("id").to_pylist().count(ident(2)) == 1


def test_identical_rows_within_one_source_are_merged(tmp_path: Path) -> None:
    result, _ = union(tmp_path, [row(1), row(1)])
    assert (result.rows, result.merged) == (1, 1)


def test_nulls_and_nan_compare_equal(tmp_path: Path) -> None:
    nulls = row(1, label=None, value=None, axis=None, hash=None)
    nan = row(2, value=float("nan"), axis=[float("nan"), 1.0])
    result, _ = union(tmp_path, [nulls, nan], [dict(nulls), dict(nan)])
    assert (result.rows, result.merged) == (2, 2)


@pytest.mark.parametrize(
    ("column", "value"),
    [
        ("label", "different"),
        ("value", 99.0),
        ("value", None),
        ("axis", [1.0, 2.0, 3.0]),
        ("axis", [1.0, 5.0]),
        ("axis", []),
        ("hash", b"\x07" * 32),
    ],
)
def test_different_content_refuses_naming_the_table_key_sources_and_attribute(
    tmp_path: Path, column: str, value: object
) -> None:
    with pytest.raises(UnionConflictError) as refused:
        union(tmp_path, [row(1), row(2)], [row(2), row(1, **{column: value})])
    (conflict,) = refused.value.conflicts
    assert conflict.table == "t.t"
    assert conflict.key == {"id": str(ident(1))}
    assert conflict.sources == ("s0", "s1")
    assert set(conflict.differences) == {column}
    assert set(conflict.differences[column]) == {"s0", "s1"}
    message = str(refused.value)
    assert "t.t" in message and str(ident(1)) in message and column in message


def test_a_conflict_in_a_run_of_three_names_every_source_and_only_the_differing_attributes(
    tmp_path: Path,
) -> None:
    with pytest.raises(UnionConflictError) as refused:
        union(tmp_path, [row(1)], [row(1)], [row(1, label="x", value=5.0)])
    (conflict,) = refused.value.conflicts
    assert conflict.sources == ("s0", "s1", "s2")
    assert set(conflict.differences) == {"label", "value"}


def test_a_composite_key_identifies_a_row(tmp_path: Path) -> None:
    first = [row(1, label="a"), row(1, label="b")]
    # same id, different `label` key column: distinct rows under a two-column key
    result, _ = union(tmp_path, first, [row(1, label="a")], keys=("id", "label"))
    assert (result.rows, result.merged) == (2, 1)
    with pytest.raises(UnionConflictError) as refused:
        union(tmp_path / "again", first, [row(1, label="a", value=0.5)], keys=("id", "label"))
    assert refused.value.conflicts[0].key == {"id": str(ident(1)), "label": "a"}


def test_the_count_of_conflicts_is_complete_and_the_report_is_bounded(tmp_path: Path) -> None:
    left = [row(n) for n in range(40)]
    right = [row(n, value=-1.0) for n in range(40)]
    with pytest.raises(UnionConflictError) as refused:
        union(tmp_path, left, right)
    assert refused.value.total == 40
    assert len(refused.value.conflicts) == 5
    assert "and 35 more" in str(refused.value)


def test_empty_files_contribute_nothing(tmp_path: Path) -> None:
    result, _ = union(tmp_path, [], [row(1)], [])
    assert (result.rows, result.merged) == (1, 0)
    assert len(result.parts) == 1
    empty, _ = union(tmp_path / "e", [], [])
    assert (empty.rows, empty.merged, empty.parts) == (0, 0, ())


def test_a_dry_run_writes_nothing(tmp_path: Path) -> None:
    contributions = [
        Contribution("a", write(tmp_path / "a.parquet", [row(1), row(2)])),
        Contribution("b", write(tmp_path / "b.parquet", [row(2)])),
    ]
    result = union_table("t.t", ("id",), contributions, None)
    assert (result.rows, result.merged, result.parts) == (2, 1, ())


def test_element_wise_equality_of_nested_columns() -> None:
    left = pa.array([[1.0, 2.0], None, [], [1.0], [float("nan")], [3.0]], pa.list_(pa.float64()))
    right = pa.array([[1.0, 2.0], None, [], [2.0], [float("nan")], None], pa.list_(pa.float64()))
    assert equal(left, right).to_pylist() == [True, True, True, False, True, False]
    structs = pa.array(
        [{"a": 1, "b": "x"}, {"a": 2, "b": None}],
        pa.struct([("a", pa.int64()), ("b", pa.string())]),
    )
    other = pa.array([{"a": 1, "b": "x"}, {"a": 2, "b": "y"}], structs.type)
    assert equal(structs, other).to_pylist() == [True, False]
    uuids = pa.array([ident(1), ident(2)], pa.uuid())
    assert equal(uuids, pa.array([ident(1), ident(3)], pa.uuid())).to_pylist() == [True, False]


def test_a_large_union_is_columnar(tmp_path: Path) -> None:
    """A million rows with half of them duplicated across two files merge and compare without
    building a Python object per row."""
    count = 1_000_000
    raw = np.zeros((count, 16), dtype=np.uint8)
    raw[:, 8:] = np.arange(count, dtype=">u8").view(np.uint8).reshape(count, 8)
    ids = pa.FixedSizeBinaryArray.from_buffers(pa.binary(16), count, [None, pa.py_buffer(raw)])
    values = pa.array(np.arange(count, dtype=np.float64))
    schema = pa.schema(
        [pa.field("id", pa.binary(16), nullable=False), pa.field("value", pa.float64())]
    )
    first = pa.table({"id": ids, "value": values}, schema=schema)
    second = first.slice(count // 2)
    pq.write_table(first, tmp_path / "first.parquet")
    pq.write_table(second, tmp_path / "second.parquet")
    started = time.perf_counter()
    result = union_table(
        "t.big",
        ("id",),
        [
            Contribution("a", tmp_path / "first.parquet"),
            Contribution("b", tmp_path / "second.parquet"),
        ],
        tmp_path / "work",
    )
    elapsed = time.perf_counter() - started
    assert (result.rows, result.merged) == (count, count // 2)
    assert pq.ParquetFile(result.parts[0]).metadata.num_rows == count
    assert elapsed < 30, f"{elapsed:.1f}s for a million rows: the union is not columnar"
    changed = second.set_column(
        1, "value", pa.array(np.arange(count // 2, count, dtype=np.float64) + 0.5)
    )
    pq.write_table(changed, tmp_path / "second.parquet")
    with pytest.raises(UnionConflictError) as refused:
        union_table(
            "t.big",
            ("id",),
            [
                Contribution("a", tmp_path / "first.parquet"),
                Contribution("b", tmp_path / "second.parquet"),
            ],
            None,
        )
    assert refused.value.total == count // 2

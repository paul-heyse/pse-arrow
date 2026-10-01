# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The union of one canonical table across every file that emits it (pipeline section 3, step 2).

Several stages emit the same row: a carrier, a licence or a convention set that two sources
cite, an entity a source and the resolution result both write. Rows with the same key and
identical content are one row; rows with the same key and different content refuse the build,
naming the table, the key, the contributing sources and the attributes that differ.

Everything is columnar. The files of a table are read with `pyarrow` and concatenated, and the
key columns are sorted once, so every run of rows with equal keys is contiguous. A row is then a
duplicate of its predecessor when its key equals the predecessor's, and a conflict when, in
addition, any non-key column differs from the predecessor's: equality is transitive, so the
comparison against the previous row decides a whole run. Content is compared only for the
duplicate rows, column by column, with Arrow kernels (nested lists element-wise): no row is ever
turned into a Python object except the few a refusal reports. A table whose keys are all
distinct is loaded from its original files untouched; a table with identical duplicates is
written once, without them, to the work directory.

The table being merged is held in memory as Arrow buffers, one table at a time; the union of a
table larger than memory would need a spill-to-disk variant of this step.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import numpy as np
import pyarrow as pa
import pyarrow.compute as pc
import pyarrow.parquet as pq

from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.store import CanonicalError

MAX_REPORTED = 5
"""Conflicting keys a refusal names; the count of all of them is always given."""
_SHOWN = 60
"""Characters of a value shown in a refusal."""


@dataclass(frozen=True)
class Contribution:
    """One file that emits rows of a table, and the stage that wrote it (a source's manifest
    id, or `_resolution`)."""

    source: str
    path: Path


@dataclass(frozen=True)
class Conflict:
    """One key whose rows differ: the attributes that differ, with each source's value."""

    table: str
    key: dict[str, str]
    sources: tuple[str, ...]
    differences: dict[str, dict[str, str]]

    def describe(self) -> str:
        key = ", ".join(f"{column}={value}" for column, value in self.key.items())
        lines = [f"{self.table} [{key}] contributed by {', '.join(self.sources)}; differs in:"]
        for attribute, values in self.differences.items():
            shown = "; ".join(f"{source}: {value}" for source, value in values.items())
            lines.append(f"    {attribute}: {shown}")
        return "\n".join(lines)


class UnionConflictError(CanonicalError):
    """Rows with the same key and different content: the stages disagree about a fact, or a
    mapping is wrong about identity. Nothing has been written."""

    def __init__(self, conflicts: Sequence[Conflict], total: int) -> None:
        self.conflicts = tuple(conflicts)
        self.total = total
        table = conflicts[0].table if conflicts else "?"
        lines = [f"{table}: {total} identifier(s) with different content in different rows:"]
        lines.extend(conflict.describe() for conflict in conflicts)
        if total > len(conflicts):
            lines.append(f"... and {total - len(conflicts)} more")
        super().__init__("\n".join(lines))


@dataclass(frozen=True)
class TableUnion:
    """What a table contributes to the build: its distinct rows, the identical duplicates
    that were collapsed, and the files that carry exactly those rows (none for a dry run that
    wrote nothing, or for an empty table)."""

    table: str
    rows: int
    merged: int
    parts: tuple[Path, ...]


def _storage(column: pa.ChunkedArray | pa.Array) -> pa.Array:
    """One contiguous array; an extension type (uuid) as its storage."""
    array = column.combine_chunks() if isinstance(column, pa.ChunkedArray) else column
    return array.storage if isinstance(array, pa.ExtensionArray) else array


def _and(left: pa.Array, right: pa.Array) -> pa.Array:
    return pc.and_(left, right)


def _or(left: pa.Array, right: pa.Array) -> pa.Array:
    return pc.or_(left, right)


def _null_as_false(mask: pa.Array) -> pa.Array:
    """`mask` with its nulls read as false."""
    return mask.fill_null(False)


def equal(left: pa.Array, right: pa.Array) -> pa.Array:
    """Element-wise equality as a boolean array with no nulls: two nulls are equal, a null and
    a value are not, two NaN are equal, lists and structs compare by their elements."""
    if isinstance(left, pa.ExtensionArray) or isinstance(right, pa.ExtensionArray):
        return equal(_storage(left), _storage(right))
    kind = left.type
    if pa.types.is_list(kind) or pa.types.is_large_list(kind):
        return _equal_lists(left, right)
    if pa.types.is_struct(kind):
        assert isinstance(left, pa.StructArray) and isinstance(right, pa.StructArray)
        both_valid = _and(pc.is_valid(left), pc.is_valid(right))
        fields = both_valid
        for index in range(kind.num_fields):
            fields = _and(fields, equal(left.field(index), right.field(index)))
        return _or(fields, _and(pc.is_null(left), pc.is_null(right)))
    same = _null_as_false(pc.equal(left, right))
    if pa.types.is_floating(kind):
        both_nan = _and(_null_as_false(pc.is_nan(left)), _null_as_false(pc.is_nan(right)))
        same = _or(same, both_nan)
    return _or(same, _and(pc.is_null(left), pc.is_null(right)))


def _equal_lists(left: pa.Array, right: pa.Array) -> pa.Array:
    both_null = _and(pc.is_null(left), pc.is_null(right))
    both_valid = _and(pc.is_valid(left), pc.is_valid(right))
    same_length = _null_as_false(
        pc.equal(pc.list_value_length(left), pc.list_value_length(right))
    )
    candidates = _and(both_valid, same_length)
    result = pa.array(np.zeros(len(left), dtype=bool))
    chosen_left, chosen_right = pc.filter(left, candidates), pc.filter(right, candidates)
    if len(chosen_left):
        elements = equal(pc.list_flatten(chosen_left), pc.list_flatten(chosen_right))
        parents = pc.list_parent_indices(chosen_left)
        bad = pc.unique(pc.filter(parents, pc.invert(elements))).cast(pa.int64())
        rows = pa.array(np.arange(len(chosen_left), dtype=np.int64))
        result = pc.replace_with_mask(result, candidates, pc.invert(pc.is_in(rows, bad)))
    return _or(result, both_null)


def _python_equal(left: object, right: object) -> bool:
    return left == right or (left != left and right != right)


def _shown(value: object) -> str:
    text = repr(value)
    return text if len(text) <= _SHOWN else text[: _SHOWN - 3] + "..."


def _conflict(
    table: str,
    data: pa.Table,
    keys: Sequence[str],
    names: Sequence[str],
    order: np.ndarray,
    same: np.ndarray,
    origin: np.ndarray,
    sources: Sequence[str],
    position: int,
) -> Conflict:
    start, end = position, position
    while same[start]:
        start -= 1
    while end + 1 < len(same) and same[end + 1]:
        end += 1
    rows = [int(row) for row in order[start : end + 1]]
    involved: list[str] = []
    for row in rows:
        source = sources[int(origin[row])]
        if source not in involved:
            involved.append(source)
    differences: dict[str, dict[str, str]] = {}
    for name in names:
        values = [data.column(name)[row].as_py() for row in rows]
        if any(not _python_equal(values[0], value) for value in values[1:]):
            differences[name] = {
                sources[int(origin[row])]: _shown(value) for row, value in zip(rows, values)
            }
    key = {column: str(data.column(column)[rows[0]].as_py()) for column in keys}
    return Conflict(table, key, tuple(involved), differences)


def union_table(
    table: str,
    keys: Sequence[str],
    contributions: Sequence[Contribution],
    work: Path | None,
) -> TableUnion:
    """The union of `contributions` (files of the canonical table `table`, all of the same
    schema, identified by the columns `keys`).

    Raises `UnionConflictError` for rows with one key and different content. Identical rows are
    written once to `work` (when there are any, and `work` is given); otherwise the original
    files are the parts to load.
    """
    tables = [pq.read_table(contribution.path) for contribution in contributions]
    sizes = [data.num_rows for data in tables]
    total = sum(sizes)
    if total == 0:
        return TableUnion(table, 0, 0, ())
    originals = tuple(c.path for c, rows in zip(contributions, sizes) if rows)
    data = pa.concat_tables(tables)
    del tables
    origin = np.repeat(np.arange(len(sizes)), sizes)

    key_arrays = {column: _storage(data.column(column)) for column in keys}
    ascending: list[tuple[str, Literal["ascending"]]] = [(column, "ascending") for column in keys]
    order_array = pc.sort_indices(pa.table(key_arrays), sort_keys=ascending)
    same = np.zeros(total, dtype=bool)
    if total > 1:
        equal_to_previous: pa.Array | None = None
        for array in key_arrays.values():
            ordered = pc.take(array, order_array)
            step = _null_as_false(pc.equal(ordered.slice(1), ordered.slice(0, total - 1)))
            equal_to_previous = step if equal_to_previous is None else _and(equal_to_previous, step)
        assert equal_to_previous is not None
        same[1:] = equal_to_previous.to_numpy(zero_copy_only=False)
    duplicates = np.flatnonzero(same)
    if duplicates.size == 0:
        return TableUnion(table, total, 0, originals)

    order = order_array.to_numpy()
    current = pa.array(order[duplicates], type=pa.uint64())
    previous = pa.array(order[duplicates - 1], type=pa.uint64())
    names = [name for name in data.column_names if name not in keys]
    differs = np.zeros(duplicates.size, dtype=bool)
    for name in names:
        column = data.column(name)
        agree = equal(
            pc.take(column, current).combine_chunks(), pc.take(column, previous).combine_chunks()
        )
        differs |= ~agree.to_numpy(zero_copy_only=False)
    if differs.any():
        reported = np.flatnonzero(differs)[:MAX_REPORTED]
        sources = [contribution.source for contribution in contributions]
        raise UnionConflictError(
            [
                _conflict(
                    table, data, keys, names, order, same, origin, sources, int(duplicates[i])
                )
                for i in reported
            ],
            int(differs.sum()),
        )

    kept = np.sort(order[~same])
    if work is None:
        return TableUnion(table, int(kept.size), total - int(kept.size), ())
    work.mkdir(parents=True, exist_ok=True)
    path = work / store.file_name(table)
    pq.write_table(data.take(pa.array(kept)), path, compression=store.COMPRESSION)
    return TableUnion(table, int(kept.size), total - int(kept.size), (path,))

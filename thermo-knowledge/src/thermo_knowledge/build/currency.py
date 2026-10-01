# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Whether a stored qualification run speaks for the values a build holds (pipeline section 5.4).

A run evaluated stored values and compared them with a library. It is current for a build only
while the records it read are, row for row, the ones the build holds: an identifier does not change
with a value, so a mapping fix that changes a number under the same identifiers must retire the
run. The run's manifest therefore records a content hash of every record it read, and the build
recomputes the same hashes from the canonical Parquet it is about to load and leaves out a run
whose hashes differ.

`read_records` is the one function that decides which records a run reads and hashes them, for
both sides. It is given the rows through a `RowSource`: `DatabaseRows` (the run, which reads the
database) and `ParquetRows` (the build, which reads the canonical files), each returning the rows
of a table with a value in a column, restricted to the table's canonical columns. A record's hash
(`record_hash`) is computed over those canonical column values alone, so a column PostgreSQL
computes plays no part and the two sides agree.

The records read, starting from the parameter sets a run evaluated and every set of the
parameterizations its sub-form choices read from: each parameter set, its slot-group row and the
rows of its families, the sets it nests and the sets its reference slots name (followed to a
fixed point), the parameterizations the sets belong to with their convention sets (the convention
facts an evaluation reads) and their sub-form choices, and the validity of every set and every such
parameterization: its regions, their clauses and its coverage rows.
"""

from __future__ import annotations

import hashlib
import json
import uuid
from collections.abc import Collection, Mapping, Sequence
from datetime import datetime
from pathlib import Path
from typing import Protocol

import psycopg
import pyarrow as pa
import pyarrow.compute as pac
import pyarrow.parquet as pq
from psycopg import sql

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.build.keys import table_keys
from thermo_knowledge.canonical.schemas import canonical_schemas, table_name
from thermo_knowledge.declaration import model as m

_HELD_SHAPES = ("nested_set", "set_reference")
_SHOWN = 5

type Row = dict[str, object]


class RowSource(Protocol):
    """Where a read of the records gets its rows."""

    def rows(self, table: str, column: str, values: Collection[uuid.UUID]) -> list[Row]:
        """The rows of canonical table `table` whose `column` holds one of `values`, each as its
        canonical columns by name; none for a table that holds nothing."""
        ...


def _uuid(value: object) -> uuid.UUID:
    """A key column's value as the UUID it holds: both row sources return keys as UUIDs."""
    if not isinstance(value, uuid.UUID):
        raise TypeError(f"a key column holds {value!r}, not a UUID")
    return value


def _plain(value: object) -> object:
    """A value as the JSON the hash is taken over: the same for a Parquet and a database value."""
    if isinstance(value, uuid.UUID):
        return str(value)
    if isinstance(value, bytes | bytearray | memoryview):
        return bytes(value).hex()
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, list | tuple):
        return [_plain(item) for item in value]
    if isinstance(value, dict):
        return {str(name): _plain(item) for name, item in sorted(value.items())}
    return value


def record_hash(table: str, row: Mapping[str, object]) -> str:
    """The content hash of one canonical row: SHA-256 over the table and every canonical column
    with its value, in column-name order."""
    text = json.dumps(
        [table, [[name, _plain(row[name])] for name in sorted(row)]],
        separators=(",", ":"),
        allow_nan=False,
    )
    return hashlib.sha256(text.encode()).hexdigest()


def read_digest(records: Mapping[str, str]) -> str:
    """One digest of the hashes of all the records read, for a reuse key."""
    text = json.dumps(sorted(records.items()), separators=(",", ":"))
    return hashlib.sha256(text.encode()).hexdigest()


class Layout:
    """The canonical columns and key columns of every table of one declaration."""

    def __init__(self, decl: m.Declaration) -> None:
        self.decl = decl
        self.columns = {name: tuple(schema.names) for name, schema in canonical_schemas(decl).items()}
        self.keys = table_keys(decl)
        self.groups = tuple(decl.slot_groups)

    def key(self, table: str, row: Mapping[str, object]) -> str:
        """`table[key values]`: how a record is named in a manifest and in a refusal."""
        return f"{table}[{', '.join(str(_plain(row[name])) for name in self.keys[table])}]"


class DatabaseRows:
    """The rows of the database a run reads."""

    def __init__(self, conn: psycopg.Connection, layout: Layout) -> None:
        self._conn = conn
        self._layout = layout

    def rows(self, table: str, column: str, values: Collection[uuid.UUID]) -> list[Row]:
        columns = self._layout.columns[table]
        schema, _, name = table.partition(".")
        query = sql.SQL("SELECT {columns} FROM {table} WHERE {column} = ANY(%s)").format(
            columns=sql.SQL(", ").join(sql.Identifier(c) for c in columns),
            table=sql.Identifier(schema, name),
            column=sql.Identifier(column),
        )
        return [
            dict(zip(columns, found, strict=True))
            for found in self._conn.execute(query, (list(values),)).fetchall()
        ]


class ParquetRows:
    """The rows of the canonical Parquet a build is about to load: the files of each table, one
    or several (the resolution result and every source may each emit rows of a table)."""

    def __init__(self, files: Mapping[str, Sequence[Path]]) -> None:
        self._files = files
        self._tables: dict[str, pa.Table | None] = {}

    def _table(self, table: str) -> pa.Table | None:
        if table not in self._tables:
            paths = self._files.get(table, ())
            self._tables[table] = (
                pa.concat_tables([pq.read_table(path) for path in paths]) if paths else None
            )
        return self._tables[table]

    def rows(self, table: str, column: str, values: Collection[uuid.UUID]) -> list[Row]:
        data = self._table(table)
        if data is None or not values:
            return []
        held = data.column(column).combine_chunks()
        storage = held.storage if isinstance(held, pa.ExtensionArray) else held
        wanted = pa.array([value.bytes for value in values], type=storage.type)
        return data.filter(pac.is_in(storage, value_set=wanted)).to_pylist()


def read_records(
    layout: Layout,
    source: RowSource,
    *,
    sets: Collection[uuid.UUID],
    parameterizations: Collection[uuid.UUID] = (),
) -> dict[str, str]:
    """The content hash of every record a run read, by record name (`table[key]`): the records
    reached from `sets` and from every set of `parameterizations` (see the module text)."""
    ps = pc.PARAMETER_SET
    records: dict[str, str] = {}
    known: set[uuid.UUID] = set()
    belonging: set[uuid.UUID] = set(parameterizations)
    convention_sets: set[uuid.UUID] = set()

    def add(table: str, rows: Sequence[Row]) -> None:
        for row in rows:
            records[layout.key(table, row)] = record_hash(table, row)

    seeds = source.rows(ps.table, ps.parameterization, parameterizations)
    frontier: set[uuid.UUID] = {*sets, *(_uuid(row["id"]) for row in seeds)}
    while frontier:
        known |= frontier
        found = source.rows(ps.table, "id", frontier)
        add(ps.table, found)
        belonging |= {_uuid(row[ps.parameterization]) for row in found}
        reached: set[uuid.UUID] = {
            _uuid(row["id"]) for row in source.rows(ps.table, ps.parent, frontier)
        }
        for group in layout.groups:
            held = [slot.name for slot in group.slots if slot.shape in _HELD_SHAPES]
            table = table_name(m.PARAM_SCHEMA, group.id)
            rows = source.rows(table, "id", frontier)
            add(table, rows)
            reached |= {
                _uuid(row[name]) for row in rows for name in held if row.get(name) is not None
            }
            for family in group.families:
                family_held = [s.name for s in family.slots if s.shape in _HELD_SHAPES]
                family_table = table_name(m.PARAM_SCHEMA, family.id)
                family_rows = source.rows(family_table, "set_id", frontier)
                add(family_table, family_rows)
                reached |= {
                    _uuid(row[name])
                    for row in family_rows
                    for name in family_held
                    if row.get(name) is not None
                }
        frontier = reached - known
    if belonging:
        pz = pc.PARAMETERIZATION
        rows = source.rows(pz.table, "id", belonging)
        add(pz.table, rows)
        convention_sets = {
            _uuid(row[pz.convention_set])
            for row in rows
            if row.get(pz.convention_set) is not None
        }
        choice = pc.SUBJECT_SUBFORM_CHOICE
        add(choice.table, source.rows(choice.table, choice.parameterization, belonging))
    if convention_sets:
        add(pc.CONVENTION_SET.table, source.rows(pc.CONVENTION_SET.table, "id", convention_sets))
    holders = known | belonging
    if holders:
        region, clause, coverage = pc.VALIDITY_REGION, pc.REGION_CLAUSE, pc.VALIDITY_COVERAGE
        regions = source.rows(region.table, region.record, holders)
        add(region.table, regions)
        if regions:
            add(
                clause.table,
                source.rows(clause.table, clause.region, [_uuid(row["id"]) for row in regions]),
            )
        add(coverage.table, source.rows(coverage.table, coverage.record, holders))
    return records


def differences(recorded: Mapping[str, str], current: Mapping[str, str]) -> str | None:
    """Why `current` is not what a run recorded, or `None` when it is: how many records changed,
    are gone and are new, with the first of each."""
    changed = sorted(name for name in recorded if name in current and recorded[name] != current[name])
    gone = sorted(name for name in recorded if name not in current)
    new = sorted(name for name in current if name not in recorded)
    if not (changed or gone or new):
        return None

    def listing(names: list[str]) -> str:
        shown = ", ".join(names[:_SHOWN])
        return f"{shown}{f' and {len(names) - _SHOWN} more' if len(names) > _SHOWN else ''}"

    parts = []
    if changed:
        parts.append(f"{len(changed)} record(s) it read changed ({listing(changed)})")
    if gone:
        parts.append(f"{len(gone)} record(s) it read are not in this build ({listing(gone)})")
    if new:
        parts.append(f"{len(new)} record(s) it would read are new in this build ({listing(new)})")
    return "; ".join(parts)

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A mapping's read access to staged Parquet: tables by name, rows with their provenance.

A mapping reads the staged tables, never the database. `StagedTables` yields each table's rows
as `SourceRow`s carrying the `_artifact` and `_locator` of the row, and classifies rows into
the dispositions `mapping.toml` declares for the table and its partitions.
"""

from __future__ import annotations

import re
from collections.abc import Callable, Iterator, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace

import pyarrow as pa
import pyarrow.parquet as pq

from thermo_knowledge.mapping import claims
from thermo_knowledge.mapping.spec import Partition, TableRule
from thermo_knowledge.staging.manifest import StagedManifest
from thermo_knowledge.staging.schema import ARTIFACT, LOCATOR

ELEMENT = re.compile(r"(.+)\[(\d+)\]")


class MappingError(Exception):
    """The mapping cannot run: its rules and the staged tables disagree."""


@dataclass(frozen=True)
class SourceRow:
    """One source-faithful row: its table, file, locator and values by column name."""

    table: str
    artifact: str
    locator: str
    values: Mapping[str, object]

    def __getitem__(self, column: str) -> object:
        """The value of `column`, or of one element of a list column (`coefficients[3]`, from
        zero); an element a list does not have is absent (`None`)."""
        found = ELEMENT.fullmatch(column)
        if found is None or column in self.values:
            return self.values[column]
        items = self.values[found.group(1)]
        position = int(found.group(2))
        if items is None:
            return None
        if not isinstance(items, list):
            raise MappingError(f"{self.table}.{found.group(1)} holds no list, it has no elements")
        return items[position] if position < len(items) else None

    def get(self, column: str) -> object | None:
        return self[column] if column in self.values or ELEMENT.fullmatch(column) else None

    @property
    def fields(self) -> SimpleNamespace:
        """The columns as attributes (`row.fields.section`), for a mapping that reads a row it
        does not map."""
        return SimpleNamespace(**self.values)


@dataclass(frozen=True)
class RowDisposition:
    """What the mapping does with a row: its disposition and the partition (if any) that decided
    it."""

    disposition: str
    partition: str | None
    reason: str | None
    wave: int | None
    loss: str | None
    origin_role: str | None


class StagedTables:
    """The tables of one staged directory."""

    def __init__(
        self, directory: Path, manifest: StagedManifest, schemas: Mapping[str, pa.Schema]
    ) -> None:
        self.directory = directory
        self.manifest = manifest
        self.schemas = dict(schemas)
        self._cache: dict[tuple[str, tuple[str, ...]], list[SourceRow]] = {}

    @property
    def names(self) -> tuple[str, ...]:
        return tuple(self.manifest.tables)

    def row_count(self, table: str) -> int:
        return self.manifest.tables[table].rows

    def rows(self, table: str, columns: Sequence[str] | None = None) -> list[SourceRow]:
        """Every row of `table` with `columns` (all when `None`)."""
        names = tuple(columns) if columns is not None else tuple(self.schemas[table].names)
        key = (table, names)
        cached = self._cache.get(key)
        if cached is None:
            wanted = list(dict.fromkeys([ARTIFACT, LOCATOR, *names]))
            data = pq.read_table(self.directory / self.manifest.tables[table].file, columns=wanted)
            cached = [
                SourceRow(
                    table,
                    row[ARTIFACT],
                    row[LOCATOR],
                    {name: row[name] for name in names},
                )
                for row in data.to_pylist()
            ]
            self._cache[key] = cached
        return cached


def _matches(where: Mapping[str, object], values: Mapping[str, object]) -> bool:
    for column, wanted in where.items():
        if column not in values:  # a column of a join the row has no related row in
            return False
        found = values[column]
        if isinstance(wanted, dict):
            excluded = wanted["not"]
            if found in (excluded if isinstance(excluded, list) else [excluded]):
                return False
        elif isinstance(wanted, list):
            if found not in wanted:
                return False
        elif found != wanted or isinstance(found, bool) != isinstance(wanted, bool):
            return False
    return True


def table_disposition(rule: TableRule) -> RowDisposition:
    return RowDisposition(
        rule.disposition, None, rule.reason, rule.wave, rule.loss, rule.origin_role
    )


def _partition_disposition(rule: TableRule, partition: Partition) -> RowDisposition:
    return RowDisposition(
        partition.disposition,
        partition.name,
        partition.reason,
        partition.wave,
        partition.loss,
        partition.origin_role or rule.origin_role,
    )


class Classifier:
    """The disposition of every row of the tables a mapping declares."""

    def __init__(self, tables: StagedTables, rules: Mapping[str, TableRule]) -> None:
        self._tables = tables
        self._rules = rules
        self._cache: dict[str, dict[str, RowDisposition]] = {}

    def classify(self, table: str) -> dict[str, RowDisposition]:
        """Locator to disposition for every row of `table`."""
        found = self._cache.get(table)
        if found is not None:
            return found
        rule = self._rules[table]
        default = table_disposition(rule)
        result: dict[str, RowDisposition] = {}
        if not rule.partitions:
            for row in self._tables.rows(table, []):
                result[row.locator] = default
        else:
            wanted = {column for p in rule.partitions for column in p.where}
            columns = sorted(column for column in wanted if "." not in column)
            joins = sorted({column.partition(".")[0] for column in wanted if "." in column})
            related = {name: self._joined(table, rule, name) for name in joins}
            for row in self._tables.rows(
                table, columns + [c for n in joins for c in rule.joins[n].on]
            ):
                values = dict(row.values)
                for name in joins:
                    values.update(related[name](row))
                hits = [p for p in rule.partitions if _matches(p.where, values)]
                if len(hits) > 1:
                    raise MappingError(
                        f"table {table}: the row {row.locator} matches the partitions "
                        f"{', '.join(p.name for p in hits)}; partitions are disjoint"
                    )
                result[row.locator] = _partition_disposition(rule, hits[0]) if hits else default
        self._cache[table] = result
        return result

    def _joined(
        self, table: str, rule: TableRule, name: str
    ) -> Callable[[SourceRow], dict[str, object]]:
        """The columns of the row of join `name` related to a row of `table`, keyed `name.column`
        (none when the row has no related row)."""
        join = rule.joins[name]
        index: dict[tuple[object, ...], list[SourceRow]] = {}
        for found in self._tables.rows(join.table):
            index.setdefault(tuple(found[c] for c in join.on.values()), []).append(found)

        def lookup(row: SourceRow) -> dict[str, object]:
            matches = index.get(tuple(row[c] for c in join.on), [])
            if len(matches) > 1:
                raise MappingError(
                    f"table {table}: the row {row.locator} is related to {len(matches)} rows of "
                    f"`{join.table}` by join `{name}`, needs at most one"
                )
            if not matches:
                return {}
            return {f"{name}.{column}": value for column, value in matches[0].values.items()}

        return lookup

    def counts(self, table: str) -> dict[tuple[str, str | None], int]:
        """Rows per (disposition, partition) without reading a table that has no partitions."""
        rule = self._rules[table]
        if not rule.partitions:
            return {(rule.disposition, None): self._tables.row_count(table)}
        tally: dict[tuple[str, str | None], int] = {}
        for item in self.classify(table).values():
            key = (item.disposition, item.partition)
            tally[key] = tally.get(key, 0) + 1
        return tally

    def mapped(self, table: str, partition: str | None = None) -> Iterator[SourceRow]:
        """The rows of `table` whose disposition is `mapped` (those of one partition when named),
        in table order."""
        dispositions = self.classify(table)
        if partition is not None and all(
            p.name != partition for p in self._rules[table].partitions
        ):
            raise MappingError(f"table {table} has no partition `{partition}`")
        for row in self._tables.rows(table):
            item = dispositions[row.locator]
            if item.disposition == claims.MAPPED and (
                partition is None or item.partition == partition
            ):
                yield row

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Everything a build will do, worked out before the database is touched.

`plan_build` generates the DDL, the reified rows and the declared entities, refuses inputs that
were written against another declaration or whose files are not the canonical tables, and takes
the union of every table (refusing rows that conflict). Nothing here opens a connection, so a
dry run is `plan_build` without a work directory.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

from thermo_knowledge.build import record
from thermo_knowledge.build.inputs import SourceInput, check_current, check_schemas
from thermo_knowledge.build.keys import table_keys
from thermo_knowledge.build.union import Contribution, TableUnion, union_table
from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.generate import SCHEMA_PATH, generate, schema_fingerprint
from thermo_knowledge.generate.entity_rows import TableRows
from thermo_knowledge.generate.reified import insert_batches, serialise
from thermo_knowledge.schema_build import read_physical


@dataclass(frozen=True)
class BuildPlan:
    fingerprint: str
    ddl: str
    physical: str
    batches: tuple[TableRows, ...]
    """The reified `meta` rows, then the declared entities."""
    inputs: tuple[SourceInput, ...]
    unions: tuple[TableUnion, ...]

    def parts(self) -> list[tuple[str, Path]]:
        """The files to load, as (table, file), tables in name order."""
        return [(union.table, path) for union in self.unions for path in union.parts]

    def counts(self) -> dict[str, int]:
        """Rows per table the finished database will hold (declared entities, the union and the
        build record), by `schema.table`."""
        counts: dict[str, int] = {}
        for batch in self.batches:
            name = f"{batch.schema}.{batch.table}"
            counts[name] = counts.get(name, 0) + len(batch.rows)
        for union in self.unions:
            counts[union.table] = counts.get(union.table, 0) + union.rows
        built = sum(1 for item in self.inputs if item.carrier is not None)
        counts[f"{record.SCHEMA}.{record.TABLE}"] = built
        return dict(sorted(counts.items()))

    def merged(self) -> dict[str, int]:
        """Identical duplicate rows collapsed, per table that had any."""
        return {union.table: union.merged for union in self.unions if union.merged}


def plan_build(
    decl: Declaration,
    inputs: Sequence[SourceInput] = (),
    *,
    tree: Path | None = None,
    work: Path | None = None,
) -> BuildPlan:
    """Plan a build of `decl` from `inputs`.

    Raises `CanonicalError` for an input written against another declaration fingerprint or
    with a file that is not the canonical table, and `UnionConflictError` for rows that share a
    key and differ. Tables with identical duplicates are written (without them) to `work`; with
    no `work` nothing is written and those tables have no parts to load.
    """
    ddl = generate(decl)[SCHEMA_PATH]
    physical = read_physical(tree)
    batches = insert_batches(decl)
    fingerprint = schema_fingerprint(ddl, physical, serialise(batches))
    check_current(inputs, fingerprint)
    check_schemas(decl, inputs)
    keys = table_keys(decl)
    grouped: dict[str, list[Contribution]] = {}
    for item in inputs:
        for table, path in sorted(item.files.items()):
            grouped.setdefault(table, []).append(Contribution(item.source_id, path))
    unions = tuple(
        union_table(table, keys[table], grouped[table], work) for table in sorted(grouped)
    )
    return BuildPlan(
        fingerprint=fingerprint,
        ddl=ddl.decode("utf-8"),
        physical=physical.decode("utf-8"),
        batches=tuple(batches),
        inputs=tuple(inputs),
        unions=unions,
    )

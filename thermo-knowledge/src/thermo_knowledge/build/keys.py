# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The identifying columns of every canonical table, read from the generator's plan.

Two rows of one table are the same row when their primary key values are equal: a kind's
`id` (computed from its declared identity), a relation's key columns (its `id` is computed from
them), a family row's (set, index) columns. The plan describes each table once; nothing here
restates it.
"""

from __future__ import annotations

import re

from thermo_knowledge.canonical.schemas import table_name
from thermo_knowledge.declaration import model as m
from thermo_knowledge.generate import ir
from thermo_knowledge.generate.plan import build_plan

_PRIMARY_KEY = re.compile(r"^PRIMARY KEY \((?P<columns>.*)\)$")
_QUOTED = re.compile(r'"((?:[^"]|"")*)"')


def primary_key_columns(table: ir.Table) -> tuple[str, ...]:
    """The columns of the table's primary key, in key order."""
    for constraint in table.constraints:
        found = _PRIMARY_KEY.match(constraint.body)
        if found is not None:
            return tuple(
                name.replace('""', '"') for name in _QUOTED.findall(found.group("columns"))
            )
    raise ValueError(f"{table.qualified} has no primary key")


def table_keys(decl: m.Declaration) -> dict[str, tuple[str, ...]]:
    """The key columns of every canonical table, keyed by `schema.table`."""
    return {
        table_name(table.schema, table.name): primary_key_columns(table)
        for table in build_plan(decl).tables
        if table.schema != m.META_SCHEMA
    }

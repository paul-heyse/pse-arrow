# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Everything a build inserts from a declaration, and its canonical serialisation.

The batches are what `tk build` copies into the database: the reified `meta` rows, then the
declared entities (and their `prov.record` rows). The serialisation of exactly those batches
is folded into the schema fingerprint, so the fingerprint changes whenever the declaration
does, not only when the DDL text does.
"""

from __future__ import annotations

import json
import uuid
from datetime import date, datetime

from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.generate.entity_rows import TableRows, entity_rows
from thermo_knowledge.generate.meta_rows import meta_rows
from thermo_knowledge.generate.meta_tables import META_TABLES


def insert_batches(decl: Declaration) -> list[TableRows]:
    """The non-empty batches a build inserts, in a fixed order: `meta` tables in their
    definition order, then the declared-entity tables by name."""
    rows = meta_rows(decl)
    batches = [
        TableRows("meta", name, tuple(c.name for c in table.columns), rows[name])
        for name, table in META_TABLES.items()
    ]
    batches.extend(entity_rows(decl))
    return [batch for batch in batches if batch.rows]


def _cell(value: object) -> object:
    if value is None or isinstance(value, bool | int | float | str):
        return value
    if isinstance(value, uuid.UUID):
        return str(value)
    if isinstance(value, datetime | date):
        return value.isoformat()
    if isinstance(value, bytes):
        return value.hex()
    if isinstance(value, list | tuple):
        return [_cell(item) for item in value]
    raise TypeError(f"cannot serialise {type(value).__name__} in a reified row")


def serialise(batches: list[TableRows]) -> bytes:
    """The canonical bytes of `batches`: compact JSON, batches and rows in the order given."""
    document = [
        [
            batch.schema,
            batch.table,
            list(batch.columns),
            [[_cell(v) for v in row] for row in batch.rows],
        ]
        for batch in batches
    ]
    return json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode("utf-8")

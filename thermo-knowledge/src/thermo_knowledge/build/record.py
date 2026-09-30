# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What a build records about itself: `meta.build_source`, one row per source built.

The declaration has no table for the per-source build record (pipeline section 3, step 4), so
the build owns one in schema `meta`: created by every build after the generated DDL, never part
of the declaration and never part of the schema fingerprint. A row holds the source's manifest
id, its resolved pin, the lock's tree hash, the reuse key of the mapping output that was loaded
and the time of the build. The fingerprint itself is recorded as a comment on schema `tk`.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from datetime import datetime

import psycopg
from psycopg import sql

from thermo_knowledge import db
from thermo_knowledge.build.inputs import SourceInput
from thermo_knowledge.generate.entity_rows import TableRows
from thermo_knowledge.generate.sqltext import ident, literal, qname
from thermo_knowledge.schema_build import recorded_in

SCHEMA = "meta"
TABLE = "build_source"

TABLE_COMMENT = (
    "One row per source built into this database: what `tk build` loaded. Owned by the build, "
    "not by the declaration."
)
COLUMNS: tuple[tuple[str, str, str], ...] = (
    ("manifest_id", "text", "The source's manifest id."),
    ("resolved_pin", "text", "The resolved pin the source was acquired at."),
    ("tree_hash", '"meta"."hash"', "The lock's tree hash of the acquired source."),
    ("reuse_key", "text", "The reuse key of the mapping output that was loaded."),
    ("built_at", "timestamptz", "When the build that loaded the source ran."),
)


@dataclass(frozen=True)
class BuiltSource:
    """One `meta.build_source` row."""

    manifest_id: str
    resolved_pin: str
    tree_hash: str
    reuse_key: str
    built_at: datetime


def create_statements() -> list[str]:
    """The statements that create `meta.build_source` and its comments, one per entry."""
    target = qname(SCHEMA, TABLE)
    lines = [f"    {ident(name)} {sql_type} NOT NULL" for name, sql_type, _ in COLUMNS]
    lines.append(f"    CONSTRAINT {ident(TABLE + '__pk')} PRIMARY KEY ({ident(COLUMNS[0][0])})")
    statements = [f"CREATE TABLE {target} (\n" + ",\n".join(lines) + "\n)"]
    statements.append(f"COMMENT ON TABLE {target} IS {literal(TABLE_COMMENT)}")
    statements.extend(
        f"COMMENT ON COLUMN {target}.{ident(name)} IS {literal(doc)}" for name, _, doc in COLUMNS
    )
    return statements


def rows(inputs: Sequence[SourceInput], built_at: datetime) -> TableRows:
    """The `meta.build_source` rows of the sources in `inputs` (the resolution result is not a
    source and has none)."""
    batch = TableRows(SCHEMA, TABLE, tuple(name for name, _, _ in COLUMNS))
    for item in inputs:
        carrier = item.carrier
        if carrier is None:
            continue
        batch.rows.append(
            (
                item.source_id,
                carrier.pin,
                bytes.fromhex(carrier.tree_hash),
                item.manifest.reuse_key,
                built_at,
            )
        )
    return batch


@dataclass(frozen=True)
class BuildState:
    """What a database records about how it was built."""

    fingerprint: str | None
    sources: tuple[BuiltSource, ...]


def read_state(url: str) -> BuildState:
    """The recorded schema fingerprint (comment on schema `tk`) and the sources built."""
    with db.connect(url) as conn:
        return BuildState(recorded_in(conn), _sources(conn))


def _sources(conn: psycopg.Connection) -> tuple[BuiltSource, ...]:
    (present,) = conn.execute(  # type: ignore[misc]
        "SELECT to_regclass(%s) IS NOT NULL", (f"{SCHEMA}.{TABLE}",)
    ).fetchone()
    if not present:
        return ()
    found = conn.execute(
        sql.SQL(
            "SELECT manifest_id, resolved_pin, encode(tree_hash, 'hex'), reuse_key, built_at "
            "FROM {} ORDER BY manifest_id"
        ).format(sql.Identifier(SCHEMA, TABLE))
    ).fetchall()
    return tuple(BuiltSource(*row) for row in found)

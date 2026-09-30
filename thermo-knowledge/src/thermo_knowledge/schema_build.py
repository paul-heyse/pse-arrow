# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declaration's physical layer and the schema fingerprint a database records.

`sql/physical.sql` holds the hand-written access paths a build applies after the generated DDL;
the fingerprint of a build is recorded as a comment on schema `tk` and compared here with the
one the declaration produces. The build itself is `thermo_knowledge.build`.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import psycopg

from thermo_knowledge import config, db
from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.generate.fingerprint import declaration_fingerprint, recorded_fingerprint

PHYSICAL_PATH = "sql/physical.sql"
"""Hand-written access paths, relative to the tree."""


@dataclass(frozen=True)
class FingerprintComparison:
    """The fingerprint the database records against the one the declaration produces."""

    expected: str
    recorded: str | None

    @property
    def matches(self) -> bool:
        return self.recorded == self.expected

    @property
    def message(self) -> str:
        if self.matches:
            return "the database matches the declaration"
        if self.recorded is None:
            return "the database records no schema fingerprint: it was not built from a declaration"
        return (
            f"the database was built from another declaration (recorded {self.recorded[:12]}, "
            f"current {self.expected[:12]}); rebuild it with `tk build`"
        )


def read_physical(tree: Path | None = None) -> bytes:
    """`sql/physical.sql` of the tree, or nothing when the file does not exist."""
    path = (tree if tree is not None else config.TREE_DIR) / PHYSICAL_PATH
    return path.read_bytes() if path.is_file() else b""


def recorded_in(conn: psycopg.Connection) -> str | None:
    """The fingerprint recorded as the comment on schema `tk` of the connected database."""
    row = conn.execute(
        "SELECT obj_description(oid, 'pg_namespace') FROM pg_namespace WHERE nspname = %s",
        ("tk",),
    ).fetchone()
    return recorded_fingerprint(row[0] if row else None)


def compare_fingerprint(
    url: str, decl: Declaration, *, tree: Path | None = None
) -> FingerprintComparison:
    """Compare the fingerprint recorded on schema `tk` of the database with the declaration's."""
    expected = declaration_fingerprint(decl, read_physical(tree))
    with db.connect(url) as conn:
        recorded = recorded_in(conn)
    return FingerprintComparison(expected=expected, recorded=recorded)

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers shared by the hard-case tests (plan 24, packet TK2, hard cases): a fixture written
through the canonical writer into a disposable database, the origin of a record, and the list of
checks of `tk verify` that fail. Contains no tests."""

from __future__ import annotations

import uuid
from collections.abc import Callable
from pathlib import Path

import psycopg
from build_support import fingerprint, inputs_of, write_source
from mapping_support import origin, real_declaration

from thermo_knowledge import config
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.provenance import Origin
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.testing import TestDatabase
from thermo_knowledge.verify.checks import VERIFY_DIR, load_checks
from thermo_knowledge.verify.run import run_checks

CHECKS = {c.target: c for c in load_checks(config.TREE_DIR / VERIFY_DIR)[0]}


def at(tag: str, role: str = "published") -> list[Origin]:
    """The origin of a record: a row of `a.json`, in the given role."""
    return [origin(f"a.json#/{tag}", role)]


def entity_id(decl: Declaration, kind: str, name: str) -> uuid.UUID:
    return next(e.id for e in decl.entities if e.kind == kind and e.name == name)


def observable_id(decl: Declaration, name: str) -> uuid.UUID:
    return entity_id(decl, "observable", name)


def build(
    tmp_path: Path,
    fill: Callable[[CanonicalWriter], object],
    decl: Declaration | None = None,
) -> TestDatabase:
    """A database built from the records `fill` writes through a canonical writer (the caller
    removes it, as `TestDatabase` says)."""
    decl = decl or real_declaration()
    write_source(tmp_path, "src", fill, decl=decl, declaration=fingerprint(decl))
    database = TestDatabase.create()
    try:
        build_database(database.url, decl, inputs_of(tmp_path))
    except BaseException:
        database.remove()
        raise
    return database


def failing(conn: psycopg.Connection) -> dict[str, int]:
    """Every check of `tk verify` that finds violations, with how many (none: an empty dict).
    An error in a check is a failure of the test."""
    results = run_checks(conn, list(CHECKS.values()))
    assert [(r.check.target, r.error) for r in results if r.error] == []
    return {r.check.target: r.violations for r in results if r.violations}

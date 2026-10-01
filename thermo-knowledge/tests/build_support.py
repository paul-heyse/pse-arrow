# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the build and verify tests: canonical source directories written through the
canonical writer, and a checksum of a built database. Contains no tests."""

from __future__ import annotations

import shutil
import uuid
from collections.abc import Callable, Sequence
from pathlib import Path

import psycopg

from mapping_support import carrier as make_carrier
from mapping_support import origin, real_declaration, write_identity
from thermo_knowledge.build import SourceInput, discover
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.store import CanonicalManifest
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import Declaration
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.generate.plan import GENERATED_SCHEMAS
from thermo_knowledge.schema_build import read_physical

ARTIFACTS = ("a.json", "b.json")


def fingerprint(decl: Declaration | None = None) -> str:
    """The fingerprint a manifest records for `decl` (default: the real declaration) and the
    tree's `sql/physical.sql`."""
    return declaration_fingerprint(decl or real_declaration(), read_physical())


def writer_for(source_id: str, decl: Declaration | None = None) -> CanonicalWriter:
    """A canonical writer whose only carrier is the source `source_id`."""
    from mapping_support import writer

    return writer(decl or real_declaration(), make_carrier(source_id, *ARTIFACTS))


def emit_species(
    w: CanonicalWriter, source_id: str, key: str, label: str | None = None
) -> uuid.UUID:
    """One species of the carrier `source_id`, its origin row `a.json#/<key>`."""
    return w.kind(
        "species",
        {"canonical_key": key, "label": label or key},
        origins=[origin(f"a.json#/{key}", carrier_id=source_id)],
    )


def write_source(
    canonical: Path,
    source_id: str,
    fill: Callable[[CanonicalWriter], object],
    *,
    decl: Declaration | None = None,
    declaration: str | None = None,
    drop: Sequence[str] = (),
) -> Path:
    """Write `<canonical>/<source_id>/` as `tk map` leaves it: the records `fill` emits through
    a canonical writer, a manifest of phase `records`, and the phase-1 output (with the
    carrier) beside them. `declaration` overrides the fingerprint the manifest records; `drop`
    omits tables from the written records."""
    decl = decl or real_declaration()
    shutil.rmtree(canonical / source_id, ignore_errors=True)
    w = writer_for(source_id, decl)
    fill(w)
    identity = write_identity(canonical, source_id, [])
    tables = {name: table for name, table in w.tables().items() if name not in drop}
    records = store.write_tables(canonical / source_id, tables)
    store.write_manifest(
        canonical / source_id,
        CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id=source_id,
            phase="records",
            reuse_key=f"mapping-{source_id}",
            inputs={
                "declaration": declaration or fingerprint(decl),
                "identity": store.manifest_hash(identity),
            },
            tables=records,
        ),
    )
    return canonical / source_id


def write_resolution(
    canonical: Path,
    fill: Callable[[CanonicalWriter], object],
    *,
    decl: Declaration | None = None,
    declaration: str | None = None,
) -> Path:
    """Write `<canonical>/_resolution/` as `tk resolve` leaves it, from `fill`."""
    decl = decl or real_declaration()
    shutil.rmtree(canonical / "_resolution", ignore_errors=True)
    w = writer_for("resolver", decl)
    fill(w)
    records = store.write_tables(canonical / "_resolution", w.tables())
    store.write_manifest(
        canonical / "_resolution",
        CanonicalManifest(
            schema=store.MANIFEST_SCHEMA,
            source_id="_resolution",
            phase="resolution",
            reuse_key="resolution",
            inputs={"declaration": declaration or fingerprint(decl)},
            tables=records,
        ),
    )
    return canonical / "_resolution"


def inputs_of(canonical: Path, *only: str) -> list[SourceInput]:
    return discover(canonical, only)


def count(url: str, table: str) -> int:
    with psycopg.connect(url) as conn:
        found = conn.execute(f"SELECT count(*) FROM {table}").fetchone()  # noqa: S608
    assert found is not None
    return int(found[0])


def snapshot(
    url: str, *, ignore: Sequence[str] = ("meta.build_source",)
) -> dict[str, tuple[int, str]]:
    """Row count and content checksum of every table in the canonical schemas: two equal
    snapshots are the same database content."""
    result: dict[str, tuple[int, str]] = {}
    with psycopg.connect(url) as conn:
        tables = conn.execute(
            "SELECT schemaname, tablename FROM pg_tables WHERE schemaname = ANY(%s) ORDER BY 1, 2",
            (list(GENERATED_SCHEMAS),),
        ).fetchall()
        for schema, table in tables:
            name = f"{schema}.{table}"
            if name in ignore:
                continue
            row = conn.execute(
                f"SELECT count(*), md5(coalesce(string_agg(t::text, '' ORDER BY t::text), '')) "
                f'FROM "{schema}"."{table}" t'  # noqa: S608
            ).fetchone()
            assert row is not None
            result[name] = (int(row[0]), str(row[1]))
    return result

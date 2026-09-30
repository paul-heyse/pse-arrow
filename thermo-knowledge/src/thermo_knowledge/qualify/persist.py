# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where a qualification run is kept (pipeline section 5.3).

```
<canonical>/_qualification/<case>/qual.qualification_run.parquet
                                  qual.run_parameter_set.parquet   (when sets were evaluated)
                                  prov.source.parquet, prov.software_release.parquet
                                                                   (when the library's version was known)
                                  manifest.json                    phase "qualification", with the
                                                                   content hash of every record read
                                  report.json                      the run's report
```

The Parquet is written through the canonical writer and survives a rebuild of the database,
which drops schema `qual`: `tk build` loads every current output (`build.inputs`). `tk qualify`
makes a run visible in the live database at once by running the build's load for these tables in
one transaction, replacing the rows of that case only.
"""

from __future__ import annotations

import json
import shutil
import uuid
from dataclasses import dataclass
from pathlib import Path

import adbc_driver_manager
import adbc_driver_postgresql.dbapi as adbc
import pyarrow as pa
import pyarrow.parquet as pq
from psycopg import sql

from thermo_knowledge import config, reuse
from thermo_knowledge.build.database import DatabaseRefusedError, ingest_file
from thermo_knowledge.build.inputs import QUALIFICATION_DIR, QUALIFICATION_PHASE
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.store import CanonicalError, CanonicalManifest, ReadRecords
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import model as m

REPORT_NAME = "report.json"
FORMAT = 4
"""Bumped when what a run records, or how, changes."""
RUN_TABLE = pc.QUALIFICATION_RUN.table
SET_TABLE = pc.RUN_PARAMETER_SET.table
SOURCE_TABLES = (pc.SOURCE.table, pc.SOFTWARE_RELEASE.table)


@dataclass(frozen=True)
class RunRecord:
    """What the canonical rows of one run say."""

    key: str
    form: str
    expression_hash: str
    """The hex evaluation hash of the compared output (`expression.canonical.evaluation_hash`)."""
    basis: str
    library: str
    version: str | None
    """The library version the harness reported; `None` for a run blocked before it was known,
    which then has no `reference`."""
    outcome: str
    points: int
    relative_tolerance: float
    absolute_tolerance: tuple[float, str] | None
    observable: str | None
    worst_relative_deviation: float | None
    blocked_reason: str | None
    """The `blocked_reason` member of a blocked run; `None` for any other outcome."""
    note: str | None
    sets: tuple[uuid.UUID, ...]
    validity_kind: str | None = None
    """The validity region kind the case counted its points against, when it names one."""
    inside_points: int | None = None
    outside_points: int | None = None
    undetermined_points: int | None = None
    """How many grid points were inside, outside and undetermined with respect to it; absent for
    a run that was blocked before it evaluated."""


def output_dir(canonical: Path, case: str) -> Path:
    return canonical / QUALIFICATION_DIR / case


def release_key(library: str, version: str) -> str:
    """The `source.key` of the release of `library` at `version`."""
    return f"{library}@{version}"


def tables(decl: m.Declaration, run: RunRecord) -> dict[str, pa.Table]:
    """The canonical rows of `run`, by table."""
    writer = CanonicalWriter(decl)
    record, release, link = pc.QUALIFICATION_RUN, pc.SOFTWARE_RELEASE, pc.RUN_PARAMETER_SET
    values: dict[str, object] = {
        record.key: run.key,
        record.form: run.form,
        record.expression_hash: run.expression_hash,
        record.basis: run.basis,
        record.outcome: run.outcome,
        record.points: run.points,
        record.relative_tolerance: run.relative_tolerance,
        record.blocked_reason: run.blocked_reason,
        record.note: run.note,
        record.worst_relative_deviation: run.worst_relative_deviation,
        record.validity_kind: run.validity_kind,
        record.inside_points: run.inside_points,
        record.outside_points: run.outside_points,
        record.undetermined_points: run.undetermined_points,
    }
    if run.version is not None:
        values[record.reference] = writer.kind(
            release.declared,
            {
                pc.SOURCE.key: release_key(run.library, run.version),
                pc.SOURCE.title: f"{run.library} {run.version}",
                release.version: run.version,
            },
        )
    if run.observable is not None:
        entity = decl.observable_entity(run.observable)
        assert entity is not None, f"`{run.observable}` is not a declared observable"
        values[record.observable] = entity.id
    if run.absolute_tolerance is not None:
        value, unit = run.absolute_tolerance
        values[record.absolute_tolerance] = Quantity(value, unit)
    identifier = writer.kind(record.declared, values)
    for parameter_set in run.sets:
        writer.relation(
            link.declared, {link.run: identifier, link.parameter_set: parameter_set}
        )
    return writer.tables()


def write_output(
    canonical: Path,
    case: str,
    decl: m.Declaration,
    run: RunRecord,
    key: reuse.Key,
    report: dict[str, object],
    read: ReadRecords | None,
) -> Path:
    """Write the output directory of `case` (replacing an earlier one) and return it. `read` is
    what the run read, which `tk build` checks against the values it holds."""
    destination = output_dir(canonical, case)
    work = store.new_work_directory(destination.parent)
    try:
        records = store.write_tables(work, tables(decl, run))
        store.write_manifest(
            work,
            CanonicalManifest(
                schema=store.MANIFEST_SCHEMA,
                source_id=case,
                phase=QUALIFICATION_PHASE,
                reuse_key=key.digest,
                inputs=key.inputs,
                tables=records,
                summary={"points": run.points, "sets": len(run.sets)},
                read=read,
            ),
        )
        (work / REPORT_NAME).write_text(
            json.dumps(report, indent=2, sort_keys=True, allow_nan=False) + "\n", encoding="utf-8"
        )
        store.install_directory(work, destination)
    except BaseException:
        shutil.rmtree(work, ignore_errors=True)
        raise
    return destination


@dataclass(frozen=True)
class Stored:
    manifest: CanonicalManifest
    report: dict[str, object]
    directory: Path


def read_output(canonical: Path, case: str) -> Stored | None:
    """The output of `case`, verified against its manifest, or `None` when there is none."""
    directory = output_dir(canonical, case)
    if not (directory / store.MANIFEST_NAME).is_file():
        return None
    manifest = store.read_manifest(directory)
    store.verify_directory(directory, manifest)
    try:
        report = json.loads((directory / REPORT_NAME).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CanonicalError(f"{directory / REPORT_NAME}: {error}") from error
    return Stored(manifest, report, directory)


def _uuids(path: Path, column: str) -> list[uuid.UUID]:
    return pq.read_table(path, columns=[column]).column(column).to_pylist()


def load_live(url: str, directory: Path, manifest: CanonicalManifest, prefix: str) -> None:
    """Load an output into the database `url` names in one transaction: the rows of the runs
    whose key starts with `prefix` (this case's) are replaced, and a release row already there
    is kept."""
    config.refuse_production(config.database_name(url))
    files = {name: directory / record.file for name, record in manifest.tables.items()}
    with adbc.connect(url) as connection:
        try:
            with connection.cursor() as cursor:
                cursor.execute("SET CONSTRAINTS ALL DEFERRED")
                run_table = sql.Identifier(*RUN_TABLE.split("."))
                set_table = sql.Identifier(*SET_TABLE.split("."))
                owned = sql.SQL("SELECT id FROM {table} WHERE starts_with({key}, {prefix})").format(
                    table=run_table,
                    key=sql.Identifier(pc.QUALIFICATION_RUN.key),
                    prefix=sql.Literal(prefix),
                )
                cursor.execute(
                    sql.SQL("DELETE FROM {table} WHERE {run} IN ({owned})")
                    .format(
                        table=set_table,
                        run=sql.Identifier(pc.RUN_PARAMETER_SET.run),
                        owned=owned,
                    )
                    .as_string()
                )
                cursor.execute(
                    sql.SQL("DELETE FROM {table} WHERE id IN ({owned})")
                    .format(table=run_table, owned=owned)
                    .as_string()
                )
                for table in SOURCE_TABLES:
                    path = files.get(table)
                    if path is None:
                        continue
                    ids = _uuids(path, "id")
                    cursor.execute(
                        sql.SQL("SELECT id FROM {} WHERE id IN ({})")
                        .format(
                            sql.Identifier(*table.split(".")),
                            sql.SQL(", ").join(sql.Literal(str(i)) for i in ids),
                        )
                        .as_string()
                    )
                    present = {row[0] for row in cursor.fetchall()}
                    if present:
                        continue  # the release, written by another case or an earlier run
                    ingest_file(cursor, table, path)
                for table in (RUN_TABLE, SET_TABLE):
                    if table in files:
                        ingest_file(cursor, table, files[table])
            connection.commit()
        except adbc_driver_manager.Error as error:
            connection.rollback()
            raise DatabaseRefusedError(
                f"the database refused the qualification rows: {error}"
            ) from error
        except BaseException:
            connection.rollback()
            raise

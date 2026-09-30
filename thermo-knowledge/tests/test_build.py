# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk build`: one transaction that replaces the canonical schemas from the declaration and every
source's canonical Parquet (pipeline section 3), against disposable databases."""

from __future__ import annotations

from collections.abc import Callable, Iterator
from pathlib import Path

import psycopg
import pytest
from build_support import (
    count,
    emit_species,
    fingerprint,
    inputs_of,
    snapshot,
    write_resolution,
    write_source,
)
from mapping_support import origin, real_declaration
from typer.testing import CliRunner

from thermo_knowledge import config
from thermo_knowledge.build import UnionConflictError, build_database, dry_run, read_state
from thermo_knowledge.build.database import DatabaseRefusedError, ForeignDependentsError
from thermo_knowledge.canonical.store import CanonicalError
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow
from thermo_knowledge.cli import app
from thermo_knowledge.schema_build import compare_fingerprint
from thermo_knowledge.testing import TestDatabase

runner = CliRunner()
SATURATION = "vapor_pressure_exp_series_tau.pure"


@pytest.fixture
def database() -> Iterator[TestDatabase]:
    with TestDatabase() as db:
        yield db


@pytest.fixture
def canonical(tmp_path: Path) -> Path:
    return tmp_path / "canonical"


def species_of(*keys: str, carrier: str) -> Callable[[CanonicalWriter], None]:
    def fill(w: CanonicalWriter) -> None:
        for key in keys:
            emit_species(w, carrier, key)

    return fill


def test_a_build_with_no_canonical_parquet_is_the_empty_schema(
    database: TestDatabase, canonical: Path
) -> None:
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert result.sources == () and result.merged == {}
    assert result.tables["tk.element"] == 118
    assert result.tables["meta.build_source"] == 0
    assert count(database.url, "meta.kind") > 0 and count(database.url, "tk.species") == 0
    state = read_state(database.url)
    assert state.fingerprint == result.fingerprint == fingerprint()
    assert state.sources == ()
    assert compare_fingerprint(database.url, real_declaration()).matches


def test_sources_with_identical_content_give_one_row_and_keep_both_origins(
    database: TestDatabase, canonical: Path
) -> None:
    write_source(canonical, "alpha", species_of("K0", "K1", carrier="alpha"))
    write_source(canonical, "beta", species_of("K0", "K2", carrier="beta"))
    # Both carriers cite the same licence, and each emits the same species K0: identical rows.
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert [s.manifest_id for s in result.sources] == ["alpha", "beta"]
    assert count(database.url, "tk.species") == 3
    assert result.merged["tk.species"] == 1
    assert result.merged["tk.material_entity"] == 1
    assert result.merged["prov.licence"] == 1
    assert result.merged["prov.record"] == 1
    assert count(database.url, "prov.licence") == 1
    with psycopg.connect(database.url) as conn:
        origins = conn.execute(
            "SELECT count(*), count(DISTINCT o.import_record), array_agg(DISTINCT c.manifest_id ORDER BY c.manifest_id) "
            "FROM tk.material_entity m JOIN prov.record_origin o ON o.record = m.id "
            "JOIN prov.import_record i ON i.id = o.import_record "
            "JOIN prov.artifact a ON a.id = i.artifact JOIN prov.carrier c ON c.id = a.carrier "
            "WHERE m.canonical_key = 'K0'"
        ).fetchone()
        assert origins == (2, 2, ["alpha", "beta"])
        rows = conn.execute(
            "SELECT manifest_id, resolved_pin, encode(tree_hash, 'hex'), reuse_key "
            "FROM meta.build_source ORDER BY 1"
        ).fetchall()
    assert rows == [
        ("alpha", "0123456789ab", "ab" * 32, "mapping-alpha"),
        ("beta", "0123456789ab", "ab" * 32, "mapping-beta"),
    ]
    assert [s.manifest_id for s in read_state(database.url).sources] == ["alpha", "beta"]


def test_conflicting_content_refuses_and_leaves_the_built_database_unchanged(
    database: TestDatabase, canonical: Path, tmp_path: Path
) -> None:
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    build_database(database.url, real_declaration(), inputs_of(canonical))
    before, state = snapshot(database.url), read_state(database.url)
    assert before["tk.species"][0] == 1

    def conflicting(w: CanonicalWriter) -> None:
        emit_species(w, "beta", "K0", label="another name for K0")
        emit_species(w, "beta", "K9")

    write_source(canonical, "beta", conflicting)
    with pytest.raises(UnionConflictError) as refused:
        build_database(database.url, real_declaration(), inputs_of(canonical))
    message = str(refused.value)
    assert "tk.material_entity" in message and "canonical_key" not in message.split("differs in")[1]
    assert "alpha, beta" in message and "label" in message
    assert "another name for K0" in message
    conflict = refused.value.conflicts[0]
    assert conflict.table == "tk.material_entity" and set(conflict.differences) == {"label"}
    assert conflict.sources == ("alpha", "beta") and refused.value.total == 1
    assert snapshot(database.url) == before
    assert read_state(database.url) == state


def test_a_foreign_key_violation_at_commit_rolls_everything_back(
    database: TestDatabase, canonical: Path
) -> None:
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    build_database(database.url, real_declaration(), inputs_of(canonical))
    before = snapshot(database.url)
    write_source(canonical, "alpha", species_of("K0", "K1", carrier="alpha"), drop=("prov.record",))
    with pytest.raises(DatabaseRefusedError, match="foreign key"):
        build_database(database.url, real_declaration(), inputs_of(canonical))
    assert snapshot(database.url) == before


def test_a_stale_declaration_fingerprint_refuses_naming_the_source(
    database: TestDatabase, canonical: Path
) -> None:
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    write_source(canonical, "beta", species_of("K1", carrier="beta"), declaration="0" * 64)
    write_resolution(canonical, species_of("R0", carrier="resolver"), declaration="1" * 64)
    with pytest.raises(CanonicalError) as refused:
        build_database(database.url, real_declaration(), inputs_of(canonical))
    message = str(refused.value)
    assert "run `tk map beta`" in message and "alpha" not in message
    assert "the resolution result was made against another declaration" in message
    assert "tk resolve" in message
    with psycopg.connect(database.url) as conn:  # nothing was built
        assert conn.execute("SELECT to_regclass('tk.species')").fetchone() == (None,)


def test_the_same_inputs_rebuild_to_the_same_database(
    database: TestDatabase, canonical: Path
) -> None:
    write_resolution(canonical, species_of("R0", carrier="resolver"))
    write_source(canonical, "alpha", species_of("K0", "K1", carrier="alpha"))
    write_source(canonical, "beta", species_of("K0", carrier="beta"))
    first = build_database(database.url, real_declaration(), inputs_of(canonical))
    first_snapshot = snapshot(database.url)
    second = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert first.tables == second.tables and first.fingerprint == second.fingerprint
    assert snapshot(database.url) == first_snapshot
    # a fresh database gets the same identifiers and rows
    with TestDatabase() as other:
        build_database(other.url, real_declaration(), inputs_of(canonical))
        assert snapshot(other.url) == first_snapshot


def test_the_resolution_result_and_a_source_load_together(
    database: TestDatabase, canonical: Path
) -> None:
    write_resolution(canonical, species_of("R0", carrier="resolver"))
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert result.tables["tk.species"] == 2
    assert [s.manifest_id for s in result.sources] == ["alpha"]  # the resolution is no source


def test_src_schemas_survive_a_build_and_a_dependent_view_refuses_it(
    database: TestDatabase, canonical: Path
) -> None:
    with psycopg.connect(database.url, autocommit=True) as conn:
        conn.execute("CREATE SCHEMA src_fake")
        conn.execute("CREATE TABLE src_fake.rows (locator text PRIMARY KEY, name text)")
        conn.execute("INSERT INTO src_fake.rows VALUES ('a#/0', 'ethanol'), ('a#/1', 'water')")
        conn.execute("COMMENT ON TABLE src_fake.rows IS 'source-faithful rows'")
        conn.execute("CREATE TABLE public.note (x int)")
        conn.execute("INSERT INTO public.note VALUES (7)")

    def outside() -> tuple[object, ...]:
        with psycopg.connect(database.url) as conn:
            return (
                conn.execute("SELECT * FROM src_fake.rows ORDER BY 1").fetchall(),
                conn.execute("SELECT * FROM public.note").fetchall(),
                conn.execute("SELECT obj_description('src_fake.rows'::regclass)").fetchone(),
                conn.execute(
                    "SELECT array_agg(nspname ORDER BY nspname) FROM pg_namespace "
                    "WHERE nspname LIKE 'src\\_%' OR nspname = 'public'"
                ).fetchone(),
            )

    before = outside()
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    build_database(database.url, real_declaration(), inputs_of(canonical))
    build_database(database.url, real_declaration(), inputs_of(canonical))
    assert outside() == before
    assert count(database.url, "tk.species") == 1

    # an object in another schema that depends on a canonical table is never dropped with it
    with psycopg.connect(database.url, autocommit=True) as conn:
        conn.execute("CREATE VIEW src_fake.species_view AS SELECT id FROM tk.species")
    canonical_before = snapshot(database.url)
    with pytest.raises(ForeignDependentsError, match="src_fake.species_view|species_view"):
        build_database(database.url, real_declaration(), inputs_of(canonical))
    assert snapshot(database.url) == canonical_before
    with psycopg.connect(database.url) as conn:
        assert conn.execute("SELECT count(*) FROM src_fake.species_view").fetchone() == (1,)


def test_the_production_database_is_never_built() -> None:
    with pytest.raises(config.ConfigError, match="operational store"):
        build_database("postgres:///pse?host=/var/run/postgresql", real_declaration())


def test_a_file_with_another_schema_or_table_is_refused_before_anything_is_written(
    database: TestDatabase, canonical: Path
) -> None:
    import pyarrow as pa
    import pyarrow.parquet as pq

    source = write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    path = source / "prov.record.parquet"
    table = pq.read_table(path)
    as_text = pa.array([str(u) for u in table.column("id").to_pylist()])
    pq.write_table(table.set_column(0, "id", as_text), path)
    from thermo_knowledge.canonical import store

    manifest = store.read_manifest(source)
    record = manifest.tables["prov.record"]
    from thermo_knowledge.staging import schema as staging_schema
    from thermo_knowledge.staging.writer import sha256_file

    manifest.tables["prov.record"] = type(record)(
        file=record.file,
        rows=record.rows,
        schema_fingerprint=staging_schema.fingerprint(pq.ParquetFile(path).schema_arrow),
        content_hash=sha256_file(path),
    )
    store.write_manifest(source, manifest)
    with pytest.raises(CanonicalError, match="not the canonical schema"):
        build_database(database.url, real_declaration(), inputs_of(canonical))
    with psycopg.connect(database.url) as conn:
        assert conn.execute("SELECT to_regclass('prov.record')").fetchone() == (None,)


def test_a_manifest_the_files_contradict_is_refused(canonical: Path) -> None:
    source = write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    (source / "tk.species.parquet").write_bytes(b"not parquet")
    with pytest.raises(CanonicalError, match="differs from the manifest"):
        inputs_of(canonical)


def test_sources_are_selected_by_id(canonical: Path) -> None:
    write_resolution(canonical, species_of("R0", carrier="resolver"))
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    write_source(canonical, "beta", species_of("K1", carrier="beta"))
    assert [i.source_id for i in inputs_of(canonical)] == ["_resolution", "alpha", "beta"]
    assert [i.source_id for i in inputs_of(canonical, "beta")] == ["_resolution", "beta"]
    with pytest.raises(CanonicalError, match="no canonical records for gamma"):
        inputs_of(canonical, "gamma")


# -- the combined relation and family tables -----------------------------------------------


def emit_saturation(w: CanonicalWriter, source_id: str, key: str) -> None:
    param = w.kind(
        "parameterization",
        {"key": "k", "revision": "r", "title": "t", "coherence": "independent_records"},
        origins=[origin("a.json#/0", "fitted", carrier_id=source_id)],
    )
    species = emit_species(w, source_id, key)
    w.parameter_set(
        parameterization=param,
        slot_group=SATURATION,
        subjects=[species],
        slots={"T_r": Quantity(400.0, "K"), "p_r": Quantity(1e6, "Pa")},
        families={"term": [FamilyRow({"k": k}, {"n": float(k), "t": 1.0}) for k in (1, 2, 3)]},
        origins=[origin(f"a.json#/{key}", "fitted", carrier_id=source_id)],
    )


def test_parameter_sets_of_two_sources_share_their_parameterization(
    database: TestDatabase, canonical: Path
) -> None:
    write_source(canonical, "alpha", lambda w: emit_saturation(w, "alpha", "K0"))
    write_source(canonical, "beta", lambda w: emit_saturation(w, "beta", "K1"))
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    # one parameterization, cited by both sources with identical content
    assert count(database.url, "tk.parameterization") == 1
    assert result.merged["tk.parameterization"] == 1
    assert count(database.url, "tk.parameter_set") == 2
    assert count(database.url, "param.vapor_pressure_exp_series_tau__pure__term") == 6


# -- the CLI ---------------------------------------------------------------------------


@pytest.fixture
def cli(monkeypatch: pytest.MonkeyPatch, database: TestDatabase, canonical: Path) -> Path:
    monkeypatch.setenv(config.STORE_ENV, str(canonical.parent))
    monkeypatch.setenv(config.DATABASE_URL_ENV, database.url)
    return canonical


def test_cli_builds_selected_sources_and_status_reports_them(
    cli: Path, database: TestDatabase
) -> None:
    write_source(cli, "alpha", species_of("K0", carrier="alpha"))
    write_source(cli, "beta", species_of("K1", carrier="beta"))
    built = runner.invoke(app, ["build", "--sources", "alpha"])
    assert built.exit_code == 0, built.output
    assert "built 1 source(s)" in built.output and "alpha" in built.output
    assert count(database.url, "tk.species") == 1
    status = runner.invoke(app, ["db", "status"])
    assert status.exit_code == 0, status.output
    assert f"fingerprint: {fingerprint()}" in status.output.replace(
        "fingerprint:  ", "fingerprint: "
    )
    assert "matches the database" in status.output
    assert "alpha  pin 0123456789ab" in status.output and "beta" not in status.output
    everything = runner.invoke(app, ["build", "--sources", "alpha,beta"])
    assert everything.exit_code == 0, everything.output
    assert count(database.url, "tk.species") == 2
    unknown = runner.invoke(app, ["build", "--sources", "gamma"])
    assert unknown.exit_code == 1 and "no canonical records for gamma" in unknown.output
    assert count(database.url, "tk.species") == 2


def test_cli_dry_run_reports_rows_and_touches_nothing(cli: Path, database: TestDatabase) -> None:
    write_source(cli, "alpha", species_of("K0", "K1", carrier="alpha"))
    write_source(cli, "beta", species_of("K0", carrier="beta"))
    build_database(database.url, real_declaration(), inputs_of(cli, "alpha"))
    before, state = snapshot(database.url), read_state(database.url)
    dry = runner.invoke(app, ["build", "--dry-run"])
    assert dry.exit_code == 0, dry.output
    assert "no database was touched" in dry.output
    lines = {line.split()[0]: line for line in dry.output.splitlines() if line.startswith("tk.")}
    assert lines["tk.species"].split()[1] == "2"
    assert "1 identical duplicates merged" in lines["tk.species"]
    assert snapshot(database.url) == before and read_state(database.url) == state
    # the same result without any reachable database
    assert dry_run(real_declaration(), inputs_of(cli)).tables["tk.species"] == 2


def test_cli_dry_run_needs_no_database_and_refuses_a_conflict(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv(config.STORE_ENV, str(tmp_path))
    monkeypatch.setenv(config.DATABASE_URL_ENV, "postgres:///never_created?host=/nonexistent")
    canonical = tmp_path / "canonical"
    write_source(canonical, "alpha", species_of("K0", carrier="alpha"))
    ok = runner.invoke(app, ["build", "--dry-run"])
    assert ok.exit_code == 0, ok.output

    def conflicting(w: CanonicalWriter) -> None:
        emit_species(w, "beta", "K0", label="other")

    write_source(canonical, "beta", conflicting)
    refused = runner.invoke(app, ["build", "--dry-run"])
    assert refused.exit_code == 1
    assert "tk.material_entity" in refused.output and "alpha, beta" in refused.output


def test_cli_refuses_a_stale_source_and_a_foreign_failure_exits_one(
    cli: Path, database: TestDatabase
) -> None:
    write_source(cli, "alpha", species_of("K0", carrier="alpha"), declaration="f" * 64)
    stale = runner.invoke(app, ["build"])
    assert stale.exit_code == 1 and "run `tk map alpha`" in stale.output
    write_source(cli, "gamma", species_of("K1", carrier="gamma"), drop=("prov.record",))
    (cli / "alpha").rename(cli / ".retired-alpha")
    failed = runner.invoke(app, ["build"])
    assert failed.exit_code == 1 and "foreign key" in failed.output
    with psycopg.connect(database.url) as conn:
        assert conn.execute("SELECT to_regclass('tk.species')").fetchone() == (None,)

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The committed model: it loads clean, its DDL applies, its rows insert, its fingerprint checks."""

from __future__ import annotations

from pathlib import Path

import psycopg
import pytest
from typer.testing import CliRunner

from thermo_knowledge import config
from thermo_knowledge.cli import app
from thermo_knowledge.declaration import Declaration, load_declaration
from thermo_knowledge.generate import compare_tree, generate
from thermo_knowledge.generate.plan import GENERATED_SCHEMAS, build_plan
from thermo_knowledge.generate.reified import insert_batches
from thermo_knowledge.build import build_database
from thermo_knowledge.schema_build import compare_fingerprint
from thermo_knowledge.testing import TestDatabase

runner = CliRunner()


@pytest.fixture(scope="module")
def model() -> Declaration:
    result = load_declaration()
    assert result.diagnostics == (), "\n".join(map(str, result.diagnostics))
    return result.require()


def test_the_committed_ddl_is_the_generation_of_the_committed_model(model: Declaration) -> None:
    assert compare_tree(generate(model), config.TREE_DIR) == []


def test_the_real_models_ddl_applies_and_its_rows_insert(model: Declaration) -> None:
    plan = build_plan(model)
    tables_per_schema: dict[str, int] = {}
    for table in plan.tables:
        tables_per_schema[table.schema] = tables_per_schema.get(table.schema, 0) + 1
    # a build also creates `meta.build_source`, its own record of the sources it loaded
    tables_per_schema["meta"] = tables_per_schema.get("meta", 0) + 1
    batches = insert_batches(model)
    with TestDatabase() as database:
        result = build_database(database.url, model)
        assert compare_fingerprint(database.url, model).matches
        with psycopg.connect(database.url) as conn:
            built = dict(
                conn.execute(
                    "SELECT schemaname, count(*) FROM pg_tables WHERE schemaname = ANY(%s) "
                    "GROUP BY 1",
                    (list(GENERATED_SCHEMAS),),
                ).fetchall()
            )
            assert built == tables_per_schema
            for batch in batches:
                (count,) = conn.execute(  # type: ignore[misc]
                    f'SELECT count(*) FROM "{batch.schema}"."{batch.table}"'  # noqa: S608
                ).fetchone()
                assert count == len(batch.rows), f"{batch.schema}.{batch.table}"
            (records,) = conn.execute("SELECT count(*) FROM prov.record").fetchone()  # type: ignore[misc]
            owned = [e for e in model.entities if model.kinds[e.kind].provenance.mode == "own"]
            assert records == len(owned)
    assert result.meta_rows == sum(len(b.rows) for b in batches if b.schema == "meta")
    assert result.entity_rows == sum(len(b.rows) for b in batches if b.schema != "meta")


def test_the_real_model_declares_the_elements(model: Declaration) -> None:
    assert sum(1 for e in model.entities if e.kind == "element") == 118


def test_cli_builds_the_real_model_into_a_disposable_database(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """With no canonical Parquet in the store, `tk build` is the empty schema; building again
    replaces it."""
    monkeypatch.setenv(config.STORE_ENV, str(tmp_path))
    with TestDatabase() as database:
        monkeypatch.setenv(config.DATABASE_URL_ENV, database.url)
        result = runner.invoke(app, ["build"])
        assert result.exit_code == 0, result.output
        again = runner.invoke(app, ["build"])
        assert again.exit_code == 0, again.output
        assert compare_fingerprint(database.url, load_declaration().require()).matches

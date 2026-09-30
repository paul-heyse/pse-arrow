# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Smoke tests for the TK0a scaffold: command surface, configuration, database helpers."""

from __future__ import annotations

from pathlib import Path

import psycopg
import pytest
import typer.main
from typer.testing import CliRunner

from thermo_knowledge import config, db
from thermo_knowledge.cli import STAGES, app
from thermo_knowledge.testing import NAME_PREFIX, TestDatabase

runner = CliRunner()

STAGE_NAMES = [
    "acquire",
    "read",
    "load-src",
    "resolve",
    "map",
    "build",
    "verify",
    "qualify",
    "project",
    "generate",
]


@pytest.fixture(autouse=True)
def _clean_environment(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv(config.STORE_ENV, raising=False)
    monkeypatch.delenv(config.DATABASE_URL_ENV, raising=False)
    monkeypatch.delenv(config.KEEP_FAILED_DATABASES_ENV, raising=False)


def database_exists(name: str) -> bool:
    with db.connect_admin() as conn:
        return db.database_exists(conn, name)


# -- command surface ---------------------------------------------------------------------


def test_cli_lists_every_subcommand() -> None:
    assert list(STAGES) == STAGE_NAMES
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    registered = set(typer.main.get_command(app).commands)  # type: ignore[attr-defined]
    assert registered == {*STAGE_NAMES, "db"}
    for name in [*STAGE_NAMES, "db"]:
        assert name in result.output
    group = runner.invoke(app, ["db", "--help"])
    assert group.exit_code == 0
    for name in ("url", "bootstrap", "status"):
        assert name in group.output


@pytest.mark.parametrize(
    "stage",
    [
        name
        for name in STAGE_NAMES
        if name
        not in (
            "acquire",
            "read",
            "load-src",
            "resolve",
            "map",
            "build",
            "verify",
            "qualify",
            "generate",
        )
    ],
)
def test_stage_stub_fails_with_not_implemented(stage: str) -> None:
    result = runner.invoke(app, [stage])
    assert result.exit_code != 0
    assert "not implemented in this packet" in result.output
    assert f"tk {stage}" in result.output


def test_stage_stub_accepts_later_options() -> None:
    result = runner.invoke(app, ["project", "--check", "some-source"])
    assert result.exit_code == 1
    assert "not implemented in this packet" in result.output


# -- configuration -----------------------------------------------------------------------


def test_tree_and_default_store_resolve_under_the_repository_root() -> None:
    assert config.TREE_DIR.name == config.TREE_NAME
    assert config.TREE_DIR.parent == config.REPO_ROOT
    assert (config.REPO_ROOT / ".git").exists()
    assert (config.TREE_DIR / "pyproject.toml").is_file()
    assert config.store_root() == config.TREE_DIR / ".store"
    assert config.raw_dir() == config.store_root() / "raw"
    assert config.staged_dir() == config.store_root() / "staged"
    assert config.canonical_dir() == config.store_root() / "canonical"
    assert not config.store_root().exists() or config.store_root().is_dir()


def test_store_override(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.setenv(config.STORE_ENV, str(tmp_path))
    assert config.store_root() == tmp_path
    assert config.canonical_dir() == tmp_path / "canonical"
    monkeypatch.setenv(config.STORE_ENV, "elsewhere/store")
    assert config.store_root() == config.REPO_ROOT / "elsewhere" / "store"


def test_database_url_default_and_override(monkeypatch: pytest.MonkeyPatch) -> None:
    assert config.database_url() == config.DEFAULT_DATABASE_URL
    assert config.database_name(config.DEFAULT_DATABASE_URL) == "pse_thermo"
    monkeypatch.setenv(config.DATABASE_URL_ENV, "postgres:///other?host=/tmp")
    assert config.database_url() == "postgres:///other?host=/tmp"
    result = runner.invoke(app, ["db", "url"])
    assert result.exit_code == 0
    assert result.output.strip() == "postgres:///other?host=/tmp"


def test_with_database_keeps_the_server() -> None:
    url = config.with_database(config.DEFAULT_DATABASE_URL, "thing")
    assert config.database_name(url) == "thing"
    assert url == "postgres:///thing?host=/var/run/postgresql"
    assert config.with_database("postgresql://u@h:5433/a?dbname=b&sslmode=disable", "c") == (
        "postgresql://u@h:5433/c?sslmode=disable"
    )


@pytest.mark.parametrize(
    "url", ["postgres:///pse", "postgresql://localhost/pse", "postgres://localhost/?dbname=pse"]
)
def test_production_database_is_refused(url: str, monkeypatch: pytest.MonkeyPatch) -> None:
    with pytest.raises(config.ConfigError, match="operational store"):
        config.refuse_production(config.database_name(url))
    with pytest.raises(config.ConfigError):
        db.connect(url)
    with pytest.raises(config.ConfigError):
        db.drop_database(url)
    with pytest.raises(config.ConfigError):
        db.create_database(url)
    with pytest.raises(config.ConfigError):
        config.with_database(config.DEFAULT_DATABASE_URL, "pse")
    monkeypatch.setenv(config.DATABASE_URL_ENV, url)
    with pytest.raises(config.ConfigError):
        config.database_url()
    for command in (["db", "url"], ["db", "bootstrap"], ["db", "status"]):
        result = runner.invoke(app, command)
        assert result.exit_code == 1
        assert "operational store" in result.output


def test_url_without_a_database_is_refused() -> None:
    with pytest.raises(config.ConfigError):
        config.database_name("postgres:///?host=/tmp")
    with pytest.raises(config.ConfigError):
        config.database_name("mysql:///x")


# -- database helpers --------------------------------------------------------------------


def test_test_database_accepts_a_query_and_is_dropped() -> None:
    database = TestDatabase.create()
    try:
        assert database.name.startswith(NAME_PREFIX)
        assert database.name != "pse_thermo"
        assert database_exists(database.name)
        with db.connect(database.url) as conn:
            assert conn.execute("SELECT 1").fetchone() == (1,)
            assert db.server_info(database.url).database == database.name
    finally:
        dropped = database.remove()
    assert dropped
    assert not database_exists(database.name)


def test_test_database_context_manager_drops_on_exit() -> None:
    with TestDatabase() as database:
        name = database.name
        assert database_exists(name)
    assert not database_exists(name)


def test_failed_test_database_is_kept_only_when_asked(monkeypatch: pytest.MonkeyPatch) -> None:
    database = TestDatabase.create()
    try:
        assert database.remove(failed=True)  # no request to keep: dropped
    finally:
        database.remove()
    assert not database_exists(database.name)

    monkeypatch.setenv(config.KEEP_FAILED_DATABASES_ENV, "1")
    kept = TestDatabase.create()
    try:
        assert not kept.remove(failed=True)
        assert database_exists(kept.name)
    finally:
        monkeypatch.delenv(config.KEEP_FAILED_DATABASES_ENV)
        assert kept.remove(failed=True)
    assert not database_exists(kept.name)


def test_fixture_database_is_usable(test_database: TestDatabase) -> None:
    with db.connect(test_database.url) as conn:
        assert conn.execute("SELECT current_database()").fetchone() == (test_database.name,)


def test_db_bootstrap_is_idempotent_and_status_reports(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    database = TestDatabase()  # a minted name; not created yet
    monkeypatch.setenv(config.DATABASE_URL_ENV, database.url)
    try:
        first = runner.invoke(app, ["db", "bootstrap"])
        assert first.exit_code == 0, first.output
        assert f"created: {database.name}" in first.output
        second = runner.invoke(app, ["db", "bootstrap"])
        assert second.exit_code == 0, second.output
        assert f"exists: {database.name}" in second.output
        status = runner.invoke(app, ["db", "status"])
        assert status.exit_code == 0, status.output
        assert f"database: {database.name}" in status.output
        assert "PostgreSQL" in status.output
        assert "fingerprint: (none recorded)" in status.output
        assert "the database records none" in status.output
        assert "sources:     (none built)" in status.output
        info = db.server_info()
        with db.connect() as conn:
            owner = conn.execute(
                "SELECT pg_get_userbyid(datdba) FROM pg_database WHERE datname = current_database()"
            ).fetchone()
        assert owner == (info.user,)
    finally:
        db.drop_database(database.url)
    assert not database_exists(database.name)


def test_unreachable_server_is_reported(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(config.DATABASE_URL_ENV, "postgres:///x?host=/nonexistent-socket-dir")
    result = runner.invoke(app, ["db", "status"])
    assert result.exit_code == 1
    assert "cannot reach the database" in result.output
    with pytest.raises(psycopg.OperationalError):
        db.connect().close()

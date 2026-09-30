# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`tk load-src` into a disposable database: tables, types, keys, comments and refusals."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
import typer
from readers_support import PIN, Workspace, build_workspace, module_resolver, tiny_module

from thermo_knowledge import config, db
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import schema as schema_module
from thermo_knowledge.staging.command import execute_load
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import sha256_file
from thermo_knowledge.testing import TestDatabase


@pytest.fixture
def staged(tmp_path: Path) -> Workspace:
    workspace = build_workspace(tmp_path)
    ctx = workspace.context(module_resolver(tiny_module()))
    stage.read_source(ctx, workspace.manifest(), workspace.entries())
    return workspace


def query(url: str, statement: str, *parameters: object) -> list[tuple[object, ...]]:
    with db.connect(url) as conn:
        return conn.execute(statement, parameters or None).fetchall()


def test_load_creates_typed_tables_with_keys_comments_and_manifest(
    staged: Workspace, test_database: TestDatabase
) -> None:
    url = test_database.url
    outcome = load.load_staged(url, staged.staged_pin, "src_tiny")
    assert outcome.schema == "src_tiny"
    assert outcome.tables == {"alpha": 2, "alpha_values": 3, "beta": 2, "notes": 1}

    for table, expected in outcome.tables.items():
        assert query(url, f"SELECT count(*) FROM src_tiny.{table}") == [(expected,)]
    tables = query(
        url,
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'src_tiny' "
        "ORDER BY table_name",
    )
    assert tables == [("_manifest",), ("alpha",), ("alpha_values",), ("beta",), ("notes",)]

    types = dict(
        query(
            url,
            "SELECT column_name, data_type FROM information_schema.columns "
            "WHERE table_schema = 'src_tiny' AND table_name = 'alpha'",
        )
    )
    assert types == {
        "_artifact": "text",
        "_locator": "text",
        "id": "text",
        "name": "text",
        "flag": "boolean",
        "count": "bigint",
    }
    assert (
        dict(
            query(
                url,
                "SELECT column_name, data_type FROM information_schema.columns "
                "WHERE table_schema = 'src_tiny' AND table_name = 'beta'",
            )
        )["amount"]
        == "double precision"
    )
    assert query(url, 'SELECT "values" FROM src_tiny.alpha_values ORDER BY _locator') == [
        (1.0,),
        (2.5,),
        (3.0,),
    ]
    assert query(url, "SELECT count, flag FROM src_tiny.alpha ORDER BY _locator") == [
        (3, True),
        (None, False),
    ]
    assert query(url, "SELECT amount FROM src_tiny.beta ORDER BY _locator") == [(1.5,), (None,)]

    keys = query(
        url,
        "SELECT c.relname, a.attname FROM pg_index i "
        "JOIN pg_class c ON c.oid = i.indrelid "
        "JOIN pg_namespace n ON n.oid = c.relnamespace "
        "JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = ANY (i.indkey) "
        "WHERE n.nspname = 'src_tiny' AND i.indisprimary ORDER BY c.relname",
    )
    assert keys == [
        ("alpha", "_locator"),
        ("alpha_values", "_locator"),
        ("beta", "_locator"),
        ("notes", "_locator"),
    ]
    not_null = query(
        url,
        "SELECT attname FROM pg_attribute WHERE attrelid = 'src_tiny.alpha'::regclass "
        "AND attnum > 0 AND attnotnull ORDER BY attnum",
    )
    assert not_null == [("_artifact",), ("_locator",), ("id",)]

    comment = query(
        url,
        "SELECT col_description('src_tiny.alpha_values'::regclass, "
        "(SELECT attnum FROM pg_attribute WHERE attrelid = 'src_tiny.alpha_values'::regclass "
        "AND attname = 'values'))",
    )
    assert comment == [("source_name: values[]; unit: m; note: integers as float64",)]
    locator_comment = query(
        url,
        "SELECT col_description('src_tiny.beta'::regclass, 2)",
    )
    assert "source_name:" in str(locator_comment[0][0])

    row = query(
        url,
        "SELECT source_id, pin, reader, reader_version, reuse_key, loaded_at IS NOT NULL, "
        "manifest FROM src_tiny._manifest",
    )
    manifest = staged_manifest.read(staged.staged_pin)
    assert len(row) == 1
    assert row[0][:6] == ("tiny", PIN, "tiny", "1", manifest.reuse_key, True)
    assert row[0][6]["tables"]["alpha"]["rows"] == 2  # type: ignore[index]


def test_load_array_types_and_reload_replaces_the_schema(
    tmp_path: Path, test_database: TestDatabase
) -> None:
    schema = schema_module.table_schema(
        schema_module.column("s", pa.list_(pa.string())),
        schema_module.column("i", pa.list_(pa.int32())),
        schema_module.column("f", pa.list_(pa.float32())),
        schema_module.column("n", pa.int16()),
    )
    directory = make_staged(
        tmp_path,
        {"arrays": schema},
        {
            "arrays": pa.table(
                {
                    "_artifact": ["a", "a"],
                    "_locator": ["a#1", "a#2"],
                    "s": [["x", "y"], None],
                    "i": [[1, 2], []],
                    "f": [[0.5], None],
                    "n": [7, None],
                },
                schema=schema,
            )
        },
    )
    url = test_database.url
    load.load_staged(url, directory, "src_arrays")
    assert query(url, "SELECT s, i, f, n FROM src_arrays.arrays ORDER BY _locator") == [
        (["x", "y"], [1, 2], [0.5], 7),
        (None, [], None, None),
    ]
    udt = dict(
        query(
            url,
            "SELECT column_name, udt_name FROM information_schema.columns "
            "WHERE table_schema = 'src_arrays'",
        )
    )
    assert (udt["s"], udt["i"], udt["f"], udt["n"]) == ("_text", "_int4", "_float4", "int2")

    with db.connect(url, autocommit=True) as conn:
        conn.execute("INSERT INTO src_arrays.arrays (_artifact, _locator) VALUES ('a', 'a#3')")
    assert query(url, "SELECT count(*) FROM src_arrays.arrays") == [(3,)]
    load.load_staged(url, directory, "src_arrays")
    assert query(url, "SELECT count(*) FROM src_arrays.arrays") == [(2,)]
    assert query(url, "SELECT count(*) FROM src_arrays._manifest") == [(1,)]


def make_staged(root: Path, schemas: dict[str, pa.Schema], tables: dict[str, pa.Table]) -> Path:
    """A staged directory written by hand, with a manifest that matches its files."""
    directory = root / "handmade"
    directory.mkdir()
    records = {}
    for name, table in tables.items():
        path = directory / f"{name}.parquet"
        pq.write_table(table, path)
        records[name] = staged_manifest.TableRecord(
            file=path.name,
            rows=table.num_rows,
            schema_fingerprint=schema_module.fingerprint(table.schema),
            content_hash=sha256_file(path),
        )
    staged_manifest.write(
        directory,
        staged_manifest.StagedManifest(
            schema=staged_manifest.MANIFEST_SCHEMA,
            source_id="handmade",
            pin="p" * 12,
            reader=staged_manifest.ReaderRecord("r", "1", "core", "0" * 64),
            reuse_key="k" * 64,
            tree_hash="t" * 64,
            tables=records,
            payload=[],
        ),
    )
    return directory


@pytest.mark.parametrize(
    "schema_name",
    [
        "public",
        "pse",
        "tk",
        "src",
        "src_",
        "src_Upper",
        "src-x",
        "src_x; DROP SCHEMA public",
        "pg_catalog",
        "meta",
    ],
)
def test_load_refuses_a_schema_that_is_not_src(
    staged: Workspace, test_database: TestDatabase, schema_name: str
) -> None:
    url = test_database.url
    with db.connect(url, autocommit=True) as conn:
        conn.execute("CREATE TABLE public.keep (x integer)")
    with pytest.raises(StagingError, match="only manages schemas named src_"):
        load.load_staged(url, staged.staged_pin, schema_name)
    assert query(url, "SELECT count(*) FROM public.keep") == [(0,)]
    assert query(url, "SELECT count(*) FROM pg_namespace WHERE nspname LIKE 'src%'") == [(0,)]


def test_load_refuses_the_production_database(staged: Workspace) -> None:
    with pytest.raises(config.ConfigError, match="operational store"):
        load.load_staged("postgres:///pse?host=/var/run/postgresql", staged.staged_pin, "src_tiny")


def corrupt(directory: Path, how: str) -> None:
    parquet = directory / "beta.parquet"
    manifest_path = directory / "manifest.json"
    match how:
        case "bytes":
            parquet.write_bytes(parquet.read_bytes() + b"\0")
        case "missing":
            parquet.unlink()
        case "rows":
            data = json.loads(manifest_path.read_text())
            data["tables"]["beta"]["rows"] = 99
            manifest_path.write_text(json.dumps(data))
        case "fingerprint":
            data = json.loads(manifest_path.read_text())
            data["tables"]["beta"]["schema_fingerprint"] = "0" * 64
            manifest_path.write_text(json.dumps(data))
        case "extra":
            (directory / "stray.parquet").write_bytes(parquet.read_bytes())
        case "manifest":
            manifest_path.write_text("{not json")
        case "absent":
            manifest_path.unlink()


@pytest.mark.parametrize(
    ("how", "message"),
    [
        ("bytes", "differs from the manifest's content hash"),
        ("missing", "beta.parquet is missing"),
        ("rows", "the manifest records 99"),
        ("fingerprint", "content hash|schema the manifest does not record"),
        ("extra", "stray.parquet: a Parquet file the manifest does not list"),
        ("manifest", "invalid"),
        ("absent", "missing"),
    ],
)
def test_load_refuses_staged_data_that_does_not_match_its_manifest(
    staged: Workspace, test_database: TestDatabase, how: str, message: str
) -> None:
    corrupt(staged.staged_pin, how)
    with pytest.raises(StagingError, match=message):
        load.load_staged(test_database.url, staged.staged_pin, "src_tiny")
    assert query(
        test_database.url, "SELECT count(*) FROM pg_namespace WHERE nspname = 'src_tiny'"
    ) == [(0,)]


def test_load_refuses_a_column_type_a_staging_table_cannot_hold(
    tmp_path: Path, test_database: TestDatabase
) -> None:
    schema = schema_module.table_schema(schema_module.column("when", pa.timestamp("us")))
    table = pa.table(
        {"_artifact": ["a"], "_locator": ["a#1"], "when": pa.array([1], pa.timestamp("us"))},
        schema=schema,
    )
    directory = make_staged(tmp_path, {}, {"stamps": table})
    with pytest.raises(StagingError, match="not supported"):
        load.load_staged(test_database.url, directory, "src_stamps")
    with pytest.raises(StagingError, match="not supported"):
        load.sql_type(pa.decimal128(10, 2))
    with pytest.raises(StagingError, match="not supported"):
        load.sql_type(pa.large_string())
    assert load.sql_type(pa.list_(pa.bool_())) == "boolean[]"


def test_execute_load_through_the_command(
    staged: Workspace, test_database: TestDatabase, capsys: pytest.CaptureFixture[str]
) -> None:
    ctx = staged.context(module_resolver(tiny_module()))
    execute_load(ids=[], sources=staged.sources, lock=staged.lock, ctx=ctx, url=test_database.url)
    assert "loaded" in capsys.readouterr().out
    assert query(test_database.url, "SELECT count(*) FROM src_tiny.alpha") == [(2,)]
    # the same again replaces the schema
    execute_load(
        ids=["tiny"], sources=staged.sources, lock=staged.lock, ctx=ctx, url=test_database.url
    )
    assert query(test_database.url, "SELECT count(*) FROM src_tiny._manifest") == [(1,)]


def test_execute_load_refuses_stale_and_unstaged_data(
    tmp_path: Path,
    staged: Workspace,
    test_database: TestDatabase,
    capsys: pytest.CaptureFixture[str],
) -> None:
    edited = tmp_path / "edited_reader.py"
    edited.write_text(Path(tiny_module().__file__).read_text() + "\n# edit\n")  # type: ignore[arg-type]
    ctx = staged.context(module_resolver(tiny_module(edited)))
    with pytest.raises(typer.Exit) as stale:
        execute_load(
            ids=["tiny"], sources=staged.sources, lock=staged.lock, ctx=ctx, url=test_database.url
        )
    assert stale.value.exit_code == 1
    assert "stale" in capsys.readouterr().err
    execute_load(ids=[], sources=staged.sources, lock=staged.lock, ctx=ctx, url=test_database.url)
    assert "nothing to load" in capsys.readouterr().out

    shutil.rmtree(staged.staged_pin)
    with pytest.raises(typer.Exit):
        execute_load(
            ids=["tiny"], sources=staged.sources, lock=staged.lock, ctx=ctx, url=test_database.url
        )
    assert "not staged" in capsys.readouterr().err
    assert query(
        test_database.url, "SELECT count(*) FROM pg_namespace WHERE nspname = 'src_tiny'"
    ) == [(0,)]

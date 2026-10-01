# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The read stage: writer contract, manifest, reuse, refusals and the command."""

from __future__ import annotations

import hashlib
import sys
from collections.abc import Callable
from pathlib import Path
from types import ModuleType

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
import typer
from typer.testing import CliRunner

from readers_support import (
    PIN,
    TINY_READER,
    Workspace,
    build_workspace,
    module_resolver,
    tiny_module,
)
from thermo_knowledge import config
from thermo_knowledge.acquire.lock import write_lock
from thermo_knowledge.cli import app
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import payload, reader, schema, stage
from thermo_knowledge.staging.command import execute_read
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageState
from thermo_knowledge.staging.writer import Writer

runner = CliRunner()


@pytest.fixture
def workspace(tmp_path: Path) -> Workspace:
    return build_workspace(tmp_path)


def counted(module: ModuleType) -> tuple[ModuleType, list[int]]:
    """The module with its `read` wrapped to count calls."""
    calls: list[int] = []
    original = module.read

    def read(tree: Path, writer: Writer) -> None:
        calls.append(1)
        original(tree, writer)

    module.read = read
    return module, calls


def variant(module: ModuleType, body: Callable[[Path, Writer], None]) -> ModuleType:
    """The fixture reader's module with `read` replaced."""
    module.read = body
    return module


def read_once(
    workspace: Workspace, module: ModuleType, *, force: bool = False
) -> stage.ReadOutcome:
    ctx = workspace.context(module_resolver(module))
    return stage.read_source(ctx, workspace.manifest(), workspace.entries(), force=force)


# -- globs -------------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("pattern", "path", "expected"),
    [
        ("dev/fluids/*.json", "dev/fluids/Water.json", True),
        ("dev/fluids/*.json", "dev/fluids/Water.json_disabled", False),
        ("dev/fluids/*.json", "dev/fluids/sub/Water.json", False),
        ("dev/mixtures/**", "dev/mixtures/a.py", True),
        ("dev/mixtures/**", "dev/mixtures/x/y/z.txt", True),
        ("dev/mixtures/**", "dev/mixtures/.hidden", True),
        ("dev/mixtures/**", "dev/mixturesX/a.py", False),
        ("a/**/*.csv", "a/b.csv", True),
        ("a/**/*.csv", "a/x/y/b.csv", True),
        ("a/**/*.csv", "a/x/y/b.txt", False),
        ("notes.txt", "notes.txt", True),
        ("notes.tx?", "notes.txt", True),
    ],
)
def test_payload_globs(pattern: str, path: str, expected: bool) -> None:
    assert payload.matches(pattern, path) is expected


def test_payload_files_apply_exclude(workspace: Workspace) -> None:
    files = payload.payload_files(workspace.tree, ["data/**", "notes.txt"], ["data/beta.*"])
    assert files == ["data/alpha.json", "notes.txt"]


# -- a read, its manifest and Parquet ---------------------------------------------------


def test_read_writes_parquet_and_manifest(workspace: Workspace) -> None:
    module = tiny_module()
    outcome = read_once(workspace, module)
    assert outcome.status == "read"
    directory = workspace.staged_pin
    assert sorted(path.name for path in directory.iterdir()) == [
        "alpha.parquet",
        "alpha_values.parquet",
        "beta.parquet",
        "manifest.json",
        "notes.parquet",
    ]
    manifest = staged_manifest.read(directory)
    entry = workspace.entries()["tiny"]
    assert (manifest.source_id, manifest.pin, manifest.tree_hash) == ("tiny", PIN, entry.tree_hash)
    assert manifest.reader.name == "tiny"
    assert manifest.reader.version == "1"
    assert manifest.reader.environment == "core"
    assert {name: record.rows for name, record in manifest.tables.items()} == {
        "alpha": 2,
        "alpha_values": 3,
        "beta": 2,
        "notes": 1,
    }
    for name, record in manifest.tables.items():
        data = (directory / record.file).read_bytes()
        assert record.content_hash == hashlib.sha256(data).hexdigest()
        assert record.schema_fingerprint == schema.fingerprint(module.TABLES[name])
        assert pq.ParquetFile(directory / record.file).metadata.num_rows == record.rows
    statuses = {record.path: (record.status, record.reason) for record in manifest.payload}
    assert statuses == {
        "blob/raw.bin": ("skipped", "opaque binary blob, not a table"),
        "data/alpha.json": ("read", None),
        "data/beta.csv": ("read", None),
        "notes.txt": ("partly_read", "only the first line is read"),
    }


def test_parquet_keeps_field_metadata_and_types(workspace: Workspace) -> None:
    read_once(workspace, tiny_module())
    table = pq.read_table(workspace.staged_pin / "alpha_values.parquet")
    values = table.schema.field("values")
    assert values.type == pa.float64()
    assert values.metadata[b"unit"] == b"m"
    assert values.metadata[b"source_name"] == b"values[]"
    assert values.metadata[b"note"] == b"integers as float64"
    assert table.column("values").to_pylist() == [1.0, 2.5, 3.0]
    assert table.column("_locator").to_pylist() == [
        "data/alpha.json#/0/values[0]",
        "data/alpha.json#/0/values[1]",
        "data/alpha.json#/0/values[2]",
    ]
    alpha = pq.read_table(workspace.staged_pin / "alpha.parquet")
    assert alpha.column("count").to_pylist() == [3, None]  # absent is null, never zero
    beta = pq.read_table(workspace.staged_pin / "beta.parquet")
    assert beta.column("amount").to_pylist() == [1.5, None]


# -- reuse --------------------------------------------------------------------------------


def test_current_stage_is_reused_and_force_reads_again(workspace: Workspace) -> None:
    module, calls = counted(tiny_module())
    assert read_once(workspace, module).status == "read"
    assert len(calls) == 1
    again = read_once(workspace, module)
    assert again.status == "current"
    assert len(calls) == 1
    forced = read_once(workspace, module, force=True)
    assert forced.status == "read"
    assert len(calls) == 2
    manifest = staged_manifest.read(workspace.staged_pin)
    assert manifest.tables["alpha"].rows == 2
    leftovers = [p.name for p in workspace.staged_pin.parent.iterdir()]
    assert leftovers == [PIN]


def test_reuse_key_follows_its_inputs(workspace: Workspace, tmp_path: Path) -> None:
    module = tiny_module()
    ctx = workspace.context(module_resolver(module))
    manifest, entry = workspace.manifest(), workspace.entries()["tiny"]
    resolved = module_resolver(module)(manifest)
    assert resolved is not None
    assert stage.stage_state(ctx, manifest, entry, resolved)[0] is StageState.NOT_STAGED
    read_once(workspace, module)
    assert stage.stage_state(ctx, manifest, entry, resolved)[0] is StageState.CURRENT

    # a changed reader source file
    edited = tmp_path / "edited_reader.py"
    edited.write_text(TINY_READER.read_text() + "\n# a comment changes the hash\n")
    changed = tiny_module(edited)
    changed_resolved = module_resolver(changed)(manifest)
    assert changed_resolved is not None
    assert stage.stage_state(ctx, manifest, entry, changed_resolved)[0] is StageState.STALE

    # a changed reader version
    bumped = tmp_path / "bumped_reader.py"
    bumped.write_text(
        TINY_READER.read_text().replace('READER_VERSION = "1"', 'READER_VERSION = "2"')
    )
    bumped_resolved = module_resolver(tiny_module(bumped))(manifest)
    assert bumped_resolved is not None
    state, why = stage.stage_state(ctx, manifest, entry, bumped_resolved)
    assert state is StageState.STALE
    assert why

    # a changed source tree (the lock's tree hash)
    moved = entry.__class__(**{**_fields(entry), "tree_hash": "f" * 64})
    assert stage.stage_state(ctx, manifest, moved, resolved)[0] is StageState.STALE

    # a stale stage is read again without --force
    assert read_once(workspace, changed).status == "read"
    assert stage.stage_state(ctx, manifest, entry, changed_resolved)[0] is StageState.CURRENT


def _fields(entry: object) -> dict[str, object]:
    import msgspec

    return msgspec.structs.asdict(entry)  # type: ignore[arg-type]


# -- refusals -----------------------------------------------------------------------------


def test_refuses_a_source_not_in_the_lock(workspace: Workspace) -> None:
    write_lock(workspace.lock, {})
    with pytest.raises(StagingError, match="not in sources.lock"):
        read_once(workspace, tiny_module())
    assert not workspace.staged.exists()


def test_refuses_a_store_that_does_not_verify(workspace: Workspace) -> None:
    (workspace.tree / "data" / "beta.csv").write_text("key,amount,unit\nk1,9,kg\n")
    with pytest.raises(StagingError, match=r"(?s)does not verify.*changed file: data/beta.csv"):
        read_once(workspace, tiny_module())
    (workspace.tree / "extra.txt").write_text("x")
    with pytest.raises(StagingError, match="extra file: extra.txt"):
        read_once(workspace, tiny_module())
    assert not list((workspace.staged / "tiny").glob("*")) if workspace.staged.exists() else True


def test_refuses_a_source_without_reader(tmp_path: Path) -> None:
    workspace = build_workspace(tmp_path, reader_name="absent")
    ctx = workspace.context(lambda manifest: None)
    with pytest.raises(StagingError, match="no reader core:absent exists"):
        stage.read_source(ctx, workspace.manifest(), workspace.entries())


def test_refuses_a_table_without_declared_schema(workspace: Workspace) -> None:
    def body(tree: Path, writer: Writer) -> None:
        writer.rows(
            "undeclared",
            [{"_artifact": "notes.txt", "_locator": "notes.txt#L1"}],
        )

    with pytest.raises(StagingError, match="emitted without a declared schema"):
        read_once(workspace, variant(tiny_module(), body))
    assert not workspace.staged_pin.exists()


@pytest.mark.parametrize(
    ("row", "message"),
    [
        ({"text": 7}, "does not match the declared type"),
        ({"text": "x", "extra": 1}, "does not declare: extra"),
        ({"text": None, "_artifact": None}, "non-nullable column _artifact"),
        ({"text": "x", "_artifact": "not/a/payload/file"}, "not a payload file"),
        ({"text": "x", "_locator": "wrong#L1"}, "must start with"),
    ],
)
def test_refuses_a_batch_that_does_not_match_its_schema(
    workspace: Workspace, row: dict[str, object], message: str
) -> None:
    def body(tree: Path, writer: Writer) -> None:
        record = {"_artifact": "notes.txt", "_locator": "notes.txt#L1", **row}
        writer.rows("notes", [record])

    with pytest.raises(StagingError, match=message):
        read_once(workspace, variant(tiny_module(), body))


def test_refuses_booleans_and_strings_in_numeric_columns(workspace: Workspace) -> None:
    def bool_in_int(tree: Path, writer: Writer) -> None:
        writer.rows(
            "alpha",
            [
                {
                    "_artifact": "data/alpha.json",
                    "_locator": "data/alpha.json#/0",
                    "id": "a",
                    "count": True,
                }
            ],
        )

    with pytest.raises(StagingError, match="column count"):
        read_once(workspace, variant(tiny_module(), bool_in_int))

    def text_in_float(tree: Path, writer: Writer) -> None:
        writer.rows(
            "beta",
            [{"_artifact": "data/beta.csv", "_locator": "data/beta.csv#L2", "amount": "1.5"}],
        )

    with pytest.raises(StagingError, match="column amount"):
        read_once(workspace, variant(tiny_module(), text_in_float))


def test_refuses_an_arrow_batch_with_another_schema(workspace: Workspace) -> None:
    def body(tree: Path, writer: Writer) -> None:
        wrong = pa.table({"_artifact": ["notes.txt"], "_locator": ["notes.txt#L1"], "text": [1]})
        writer.batch("notes", wrong)

    with pytest.raises(StagingError, match="does not match the declared schema"):
        read_once(workspace, variant(tiny_module(), body))


def test_accepts_an_arrow_batch_with_the_declared_schema(workspace: Workspace) -> None:
    module = tiny_module()
    original = module.read

    def body(tree: Path, writer: Writer) -> None:
        original(tree, writer)
        declared = module.TABLES["alpha"]
        batch = pa.RecordBatch.from_pylist(
            [
                {
                    "_artifact": "data/alpha.json",
                    "_locator": "data/alpha.json#/extra",
                    "id": "a3",
                    "name": None,
                    "flag": None,
                    "count": 1,
                }
            ],
            schema=declared,
        )
        assert writer.batch("alpha", batch) == 1

    read_once(workspace, variant(module, body))
    table = pq.read_table(workspace.staged_pin / "alpha.parquet")
    assert table.num_rows == 3
    assert staged_manifest.read(workspace.staged_pin).tables["alpha"].rows == 3


def test_refuses_duplicate_locators(workspace: Workspace) -> None:
    def body(tree: Path, writer: Writer) -> None:
        row = {"_artifact": "notes.txt", "_locator": "notes.txt#L1", "text": "a"}
        writer.rows("notes", [row])
        writer.rows("notes", [row])

    with pytest.raises(StagingError, match="duplicate _locator 'notes.txt#L1'"):
        read_once(workspace, variant(tiny_module(), body))


def test_refuses_an_unaccounted_payload_file(workspace: Workspace) -> None:
    module = tiny_module()
    original = module.read

    def body(tree: Path, writer: Writer) -> None:
        original(tree, writer)
        writer._skipped.pop("blob/raw.bin")  # the reader forgets to list the blob

    with pytest.raises(
        StagingError, match=r"(?s)1 payload file\(s\) are unaccounted.*blob/raw.bin"
    ):
        read_once(workspace, variant(module, body))
    assert not workspace.staged_pin.exists()
    assert list((workspace.staged / "tiny").iterdir()) == []


def test_refuses_skipped_files_with_rows_reasons_and_unknown_files(workspace: Workspace) -> None:
    def cited_and_skipped(tree: Path, writer: Writer) -> None:
        writer.rows("notes", [{"_artifact": "notes.txt", "_locator": "notes.txt#L1"}])
        writer.skipped("notes.txt", "no longer read")
        writer.skipped("blob/raw.bin", "binary")
        writer.opened("data/alpha.json")
        writer.opened("data/beta.csv")

    with pytest.raises(StagingError, match="listed as skipped but rows cite it"):
        read_once(workspace, variant(tiny_module(), cited_and_skipped))

    def no_reason(tree: Path, writer: Writer) -> None:
        writer.skipped("blob/raw.bin", "  ")

    with pytest.raises(StagingError, match="needs a reason"):
        read_once(workspace, variant(tiny_module(), no_reason))

    def unknown(tree: Path, writer: Writer) -> None:
        writer.skipped("elsewhere.txt", "not in the payload")

    with pytest.raises(StagingError, match="not a payload file"):
        read_once(workspace, variant(tiny_module(), unknown))


def test_a_file_opened_without_rows_is_read(workspace: Workspace) -> None:
    def body(tree: Path, writer: Writer) -> None:
        for path in writer.payload_files:
            writer.opened(path)

    read_once(workspace, variant(tiny_module(), body))
    manifest = staged_manifest.read(workspace.staged_pin)
    assert {record.status for record in manifest.payload} == {"read"}
    assert {record.rows for record in manifest.tables.values()} == {0}


def test_refuses_a_payload_that_matches_nothing(tmp_path: Path) -> None:
    workspace = build_workspace(tmp_path, include=["nothing/**"])
    with pytest.raises(StagingError, match="match no file"):
        read_once(workspace, tiny_module())


def test_a_failed_read_leaves_the_previous_staging(workspace: Workspace) -> None:
    read_once(workspace, tiny_module())
    before = (workspace.staged_pin / "manifest.json").read_bytes()

    def body(tree: Path, writer: Writer) -> None:
        raise RuntimeError("the reader failed")

    with pytest.raises(RuntimeError, match="the reader failed"):
        read_once(workspace, variant(tiny_module(), body), force=True)
    assert (workspace.staged_pin / "manifest.json").read_bytes() == before
    assert [p.name for p in workspace.staged_pin.parent.iterdir()] == [PIN]


# -- declared schemas ---------------------------------------------------------------------


def test_declared_schemas_are_checked() -> None:
    good = schema.table_schema(schema.column("x", schema.FLOAT64))
    schema.validate_schema("good", good)

    with pytest.raises(StagingError, match="_locator is not declared|required column _locator"):
        schema.validate_schema("t", pa.schema([schema.PROVENANCE_FIELDS[0]]))
    undocumented = pa.schema([*schema.PROVENANCE_FIELDS, pa.field("x", pa.float64())])
    with pytest.raises(StagingError, match="metadata 'source_name' is missing"):
        schema.validate_schema("t", undocumented)
    no_unit = pa.schema(
        [*schema.PROVENANCE_FIELDS, pa.field("x", pa.float64(), metadata={"source_name": "x"})]
    )
    with pytest.raises(StagingError, match="metadata 'unit' is missing"):
        schema.validate_schema("t", no_unit)
    with pytest.raises(StagingError, match="not supported"):
        schema.validate_schema("t", schema.table_schema(schema.column("x", pa.timestamp("us"))))
    with pytest.raises(StagingError, match="not supported"):
        schema.validate_schema(
            "t", schema.table_schema(schema.column("x", pa.list_(pa.list_(pa.int64()))))
        )
    with pytest.raises(StagingError, match="duplicate column"):
        schema.validate_schema(
            "t", schema.table_schema(schema.column("x", pa.int64()), schema.column("x", pa.int64()))
        )
    with pytest.raises(StagingError, match="PostgreSQL system column"):
        schema.validate_schema("t", schema.table_schema(schema.column("xmin", pa.float64())))
    with pytest.raises(StagingError, match="table name"):
        schema.validate_schema("Bad-Name", good)
    with pytest.raises(StagingError, match="table name"):
        schema.validate_schema("_manifest", good)


def test_a_reader_module_needs_a_version(tmp_path: Path) -> None:
    module = ModuleType("versionless")
    module.__file__ = str(TINY_READER)
    with pytest.raises(StagingError, match="READER_VERSION"):
        reader.ResolvedReader.from_module("versionless", module)


def test_schema_fingerprint_follows_documentation() -> None:
    first = schema.table_schema(schema.column("x", pa.float64(), unit="m"))
    second = schema.table_schema(schema.column("x", pa.float64(), unit="ft"))
    assert schema.fingerprint(first) != schema.fingerprint(second)
    assert schema.fingerprint(first) == schema.fingerprint(
        schema.table_schema(schema.column("x", pa.float64(), unit="m"))
    )


# -- the command --------------------------------------------------------------------------


def test_execute_read_lists_reads_and_reuses(
    workspace: Workspace, capsys: pytest.CaptureFixture[str]
) -> None:
    module = tiny_module()
    ctx = workspace.context(module_resolver(module))

    def run(**kwargs: object) -> str:
        arguments = {
            "ids": [],
            "force": False,
            "list_": False,
            "sources": workspace.sources,
            "lock": workspace.lock,
            "ctx": ctx,
            **kwargs,
        }
        execute_read(**arguments)  # type: ignore[arg-type]
        return capsys.readouterr().out

    assert "not staged" in run(list_=True)
    assert "core:tiny" in run(list_=True)
    first = run()
    assert first.startswith("read ")
    assert "4 tables, 8 rows, 4 payload files" in first
    assert "current" in run(list_=True)
    assert run().startswith("current ")
    assert run(force=True).startswith("read ")
    assert run(ids=["tiny"]).startswith("current ")

    with pytest.raises(typer.Exit) as unknown:
        run(ids=["nope"])
    assert unknown.value.exit_code == 2
    with pytest.raises(typer.Exit) as contradictory:
        run(list_=True, force=True)
    assert contradictory.value.exit_code == 2


def test_execute_read_reports_a_refusal_and_continues(
    workspace: Workspace, capsys: pytest.CaptureFixture[str]
) -> None:
    (workspace.tree / "data" / "beta.csv").write_text("changed\n")
    ctx = workspace.context(module_resolver(tiny_module()))
    with pytest.raises(typer.Exit) as failed:
        execute_read(
            ids=["tiny"],
            force=False,
            list_=False,
            sources=workspace.sources,
            lock=workspace.lock,
            ctx=ctx,
        )
    assert failed.value.exit_code == 1
    captured = capsys.readouterr()
    assert "refused" in captured.err
    assert "does not verify" in captured.err


def test_no_ids_skips_sources_without_a_reader(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    workspace = build_workspace(tmp_path, reader_name="absent")
    ctx = workspace.context(lambda manifest: None)
    execute_read(
        ids=[], force=False, list_=False, sources=workspace.sources, lock=workspace.lock, ctx=ctx
    )
    assert "nothing to read" in capsys.readouterr().out
    execute_read(
        ids=[], force=False, list_=True, sources=workspace.sources, lock=workspace.lock, ctx=ctx
    )
    assert "no reader" in capsys.readouterr().out


def test_the_tk_command_reads_through_the_default_resolver(
    workspace: Workspace, monkeypatch: pytest.MonkeyPatch
) -> None:
    module = tiny_module(TINY_READER)
    module.__name__ = "thermo_knowledge.readers.tiny"
    monkeypatch.setitem(sys.modules, "thermo_knowledge.readers.tiny", module)
    monkeypatch.setenv(config.STORE_ENV, str(workspace.root / "store"))
    base = ["--sources", str(workspace.sources), "--lock", str(workspace.lock)]

    listing = runner.invoke(app, ["read", "--list", *base])
    assert listing.exit_code == 0, listing.output
    assert "core:tiny" in listing.output and "not staged" in listing.output
    result = runner.invoke(app, ["read", "tiny", *base])
    assert result.exit_code == 0, result.output
    assert (workspace.staged_pin / "manifest.json").is_file()
    assert "current" in runner.invoke(app, ["read", "--list", *base]).output
    assert "current" in runner.invoke(app, ["read", *base]).output
    assert runner.invoke(app, ["read", "tiny", "--force", *base]).output.startswith("read ")
    assert runner.invoke(app, ["read", "unknown", *base]).exit_code == 2

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The side-environment reader protocol, exercised with a fake side reader that the core
interpreter runs through the same subprocess protocol as `envs/tk-env.sh run`."""

from __future__ import annotations

import shutil
from pathlib import Path

import pyarrow.parquet as pq
import pytest
from readers_support import (
    FAKE_ENVS,
    PIN,
    Workspace,
    build_workspace,
    fake_side_command,
    side_resolver,
)

from thermo_knowledge import config
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import reader, side, stage
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.reader import ResolvedReader

SCRIPT = FAKE_ENVS / "fakeenv" / "readers" / "fake_side.py"


@pytest.fixture
def workspace(tmp_path: Path) -> Workspace:
    return build_workspace(tmp_path, reader_name="fake_side", environment="fakeenv")


def side_context(workspace: Workspace, envs: Path = FAKE_ENVS) -> stage.StageContext:
    def resolve(manifest):  # type: ignore[no-untyped-def]
        return reader.resolve(manifest, package="thermo_knowledge.readers", envs=envs)

    return workspace.context(resolve, side_command=fake_side_command)


def copy_envs(tmp_path: Path, edit) -> Path:  # type: ignore[no-untyped-def]
    envs = tmp_path / "envs"
    shutil.copytree(FAKE_ENVS, envs, dirs_exist_ok=True)
    script = envs / "fakeenv" / "readers" / "fake_side.py"
    script.write_text(edit(script.read_text()))
    return envs


def test_the_tk_env_invocation() -> None:
    tree, out = Path("/t/tree"), Path("/t/out")
    command = side.tk_env_command("thermotools", Path("/envs/thermotools/readers/x.py"), tree, out)
    assert command == [
        "bash",
        str(config.TREE_DIR / "envs" / "tk-env.sh"),
        "run",
        "thermotools",
        "python",
        "/envs/thermotools/readers/x.py",
        "--tree",
        "/t/tree",
        "--out",
        "/t/out",
    ]


def test_a_side_reader_is_found_by_its_script_and_version(workspace: Workspace) -> None:
    resolved = side_resolver()(workspace.manifest())
    assert resolved is not None
    assert resolved.side
    assert (resolved.environment, resolved.name, resolved.version) == ("fakeenv", "fake_side", "1")
    assert resolved.source_files == (SCRIPT,)


def test_a_side_reader_is_validated_and_written_like_an_in_process_reader(
    workspace: Workspace,
) -> None:
    ctx = side_context(workspace)
    outcome = stage.read_source(ctx, workspace.manifest(), workspace.entries())
    assert outcome.status == "read"
    manifest = staged_manifest.read(workspace.staged_pin)
    assert manifest.reader.environment == "fakeenv"
    assert manifest.reader.version == "1"
    assert manifest.pin == PIN
    assert manifest.tables["gamma"].rows == 2
    statuses = {record.path: (record.status, record.reason) for record in manifest.payload}
    assert statuses == {
        "blob/raw.bin": ("skipped", "binary"),
        "data/alpha.json": ("read", None),
        "data/beta.csv": ("skipped", "fake reader ignores CSV"),
        "notes.txt": ("partly_read", "fake reader ignores text"),
    }
    table = pq.read_table(workspace.staged_pin / "gamma.parquet")
    assert table.column("values").to_pylist() == [[1.0, 2.5, 3.0], []]
    assert table.schema.field("values").metadata[b"unit"] == b"m"
    assert [p.name for p in workspace.staged_pin.iterdir() if p.suffix == ".parquet"] == [
        "gamma.parquet"
    ]
    assert stage.read_source(ctx, workspace.manifest(), workspace.entries()).status == "current"
    resolved = ctx.resolver(workspace.manifest())
    assert isinstance(resolved, ResolvedReader)
    state, _ = stage.stage_state(ctx, workspace.manifest(), workspace.entries()["tiny"], resolved)
    assert state is stage.StageState.CURRENT


def test_a_changed_side_script_makes_the_stage_stale(workspace: Workspace, tmp_path: Path) -> None:
    stage.read_source(side_context(workspace), workspace.manifest(), workspace.entries())
    envs = copy_envs(tmp_path, lambda text: text + "\n# changed\n")
    ctx = side_context(workspace, envs)
    resolved = ctx.resolver(workspace.manifest())
    assert resolved is not None
    state, _ = stage.stage_state(ctx, workspace.manifest(), workspace.entries()["tiny"], resolved)
    assert state is stage.StageState.STALE


def read_with(workspace: Workspace, tmp_path: Path, edit) -> None:  # type: ignore[no-untyped-def]
    envs = copy_envs(tmp_path, edit)
    stage.read_source(side_context(workspace, envs), workspace.manifest(), workspace.entries())


def test_a_failing_side_reader_is_refused(workspace: Workspace, tmp_path: Path) -> None:
    with pytest.raises(StagingError, match="exited with status 3"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace(
                "def main() -> None:", "def main():\n    raise SystemExit(3)\n\n\ndef unused():"
            ),
        )
    assert not workspace.staged_pin.exists()


def test_a_side_reader_version_must_match_its_declaration(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match="differs from the script's READER_VERSION"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace('"reader_version": READER_VERSION', '"reader_version": "9"'),
        )


def test_a_side_reader_without_a_version_literal_is_refused(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match="READER_VERSION"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace('READER_VERSION = "1"', "READER_VERSION = str(1)"),
        )


def test_a_side_file_that_differs_from_its_declaration_is_refused(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match="does not have the schema tables.json declares"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace('"type": "list<float64>" if', '"type": "list<int64>" if'),
        )


def test_a_side_reader_must_account_for_every_payload_file(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match=r"(?s)unaccounted.*data/beta.csv"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace(
                '"data/beta.csv": {"status": "skipped", "reason": "fake reader ignores CSV"},', ""
            ),
        )


def test_a_side_reader_undeclared_and_duplicate_rows_are_refused(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match="duplicate _locator"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace(
                '[f"{artifact}#/{i}" for i in range(len(records))]',
                '[f"{artifact}#/0" for i in range(len(records))]',
            ),
        )
    with pytest.raises(StagingError, match="does not declare"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace(
                'pq.write_table(table, out / "gamma.parquet")',
                'pq.write_table(table, out / "gamma.parquet")\n    pq.write_table(table, out / "stray.parquet")',
            ),
        )


def test_a_side_reader_that_writes_no_description_is_refused(
    workspace: Workspace, tmp_path: Path
) -> None:
    with pytest.raises(StagingError, match="wrote no tables.json"):
        read_with(
            workspace,
            tmp_path,
            lambda text: text.replace(
                '(out / "tables.json").write_text(json.dumps(described))', "pass"
            ),
        )

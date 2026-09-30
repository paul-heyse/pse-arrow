# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The one reuse-key builder (`thermo_knowledge.reuse`): every stage's key covers the framework
files it executes, the versions of the libraries it declares and, for a harness or a side
reader, the script and the environment lock; a change to one of them marks the dependent stage
stale, and a change to nothing it declares does not."""

from __future__ import annotations

import importlib
import shutil
from pathlib import Path

import pytest
from mapping_support import fake_environment, real_declaration
from qualify_support import case_text
from readers_support import Workspace, build_workspace, module_resolver, tiny_module
from test_qualify_run import Env, env  # noqa: F401  (the `env` fixture)
from test_staging_side import copy_envs, side_context

from thermo_knowledge import reuse
from thermo_knowledge.canonical.environment import Environment
from thermo_knowledge.mapping import runner
from thermo_knowledge.resolve.command import resolve_all
from thermo_knowledge.staging import reader as staging_reader
from thermo_knowledge.staging import stage
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageState

PACKAGE = Path(reuse.__file__).resolve().parent
INSTALLED = reuse.installed_version


@pytest.fixture
def copied_package(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """A copy of the framework's source that the digests are taken from."""
    root = tmp_path / "thermo_knowledge"
    shutil.copytree(PACKAGE, root, ignore=shutil.ignore_patterns("__pycache__"))
    monkeypatch.setattr(reuse, "PACKAGE_ROOT", root)
    return root


def versions(monkeypatch: pytest.MonkeyPatch, **changed: str) -> None:
    """Make the installed version of each named distribution `changed[name]`, and of no other
    what it really is (the accessor is the only way the keys read versions); none restores."""
    monkeypatch.setattr(
        reuse, "installed_version", lambda name: changed.get(name) or INSTALLED(name)
    )


# -- the declarations ---------------------------------------------------------------------------


def test_every_declared_path_exists_and_each_stage_lists_its_own_package() -> None:
    own = {"read": "staging", "map": "mapping", "resolve": "resolve", "qualify": "qualify"}
    assert set(reuse.STAGES) == set(own)
    for stage_name, declared in reuse.STAGES.items():
        assert own[stage_name] in declared.sources
        for entry in declared.sources:
            assert (PACKAGE / entry).exists(), entry
        assert reuse.framework_digest(stage_name)
        assert all(reuse.installed_version(library) for library in declared.libraries)


def test_the_declared_libraries_are_the_result_affecting_ones_of_each_stage() -> None:
    libraries = {name: set(d.libraries) for name, d in reuse.STAGES.items()}
    assert "pyarrow" in libraries["read"]
    assert {"pint", "pyarrow"} <= libraries["map"]
    assert "rdkit" in libraries["resolve"]
    assert {"sympy", "numpy", "scipy", "pint"} <= libraries["qualify"]


def test_a_declared_path_that_does_not_exist_is_an_error_never_an_empty_digest(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    broken = reuse.StageDeclaration(("nowhere.py",), ())
    monkeypatch.setattr(reuse, "STAGES", {"read": broken})
    with pytest.raises(reuse.ReuseError, match="`nowhere.py` does not exist"):
        reuse.framework_digest("read")
    with pytest.raises(reuse.ReuseError, match="not a stage that keeps a reuse key"):
        reuse.stage_key("build", {})


def test_a_library_that_is_not_installed_is_an_error() -> None:
    with pytest.raises(reuse.ReuseError, match="`no-such-distribution`.*not installed"):
        reuse.installed_version("no-such-distribution")


def test_a_key_records_what_it_covers() -> None:
    key = reuse.stage_key("map", {"staged": "s", "format": 1})
    assert key.inputs["staged"] == "s" and key.inputs["format"] == "1"
    assert key.inputs["framework"] == reuse.framework_digest("map")
    assert {name for name in key.inputs if name.startswith("library:")} == {
        "library:pint",
        "library:pyarrow",
    }
    assert reuse.stage_key("map", {"staged": "s", "format": 1}) == key
    assert reuse.stage_key("map", {"staged": "t", "format": 1}).digest != key.digest
    assert reuse.stage_key("resolve", {"staged": "s", "format": 1}).digest != key.digest


# -- what changes a key and what does not --------------------------------------------------------


def test_a_provider_version_changes_the_key_of_the_stages_that_declare_it(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    before = {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}
    versions(monkeypatch, pint="0.0.0")
    after = {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}
    changed = {name for name in before if before[name] != after[name]}
    assert changed == {name for name, d in reuse.STAGES.items() if "pint" in d.libraries}
    assert changed == {"map", "qualify"}
    versions(monkeypatch, rdkit="0.0.0")
    assert {
        name for name in reuse.STAGES if reuse.stage_key(name, {}).digest != before[name]
    } == {"resolve"}


def test_a_framework_file_changes_the_key_of_the_stages_that_execute_it(
    copied_package: Path,
) -> None:
    before = {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}
    assert before == {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}, (
        "a copy of the same files gives the same keys"
    )
    with (copied_package / "canonical" / "writer.py").open("a") as handle:
        handle.write("\n# the writer changed\n")
    after = {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}
    assert {n for n in before if before[n] != after[n]} == {"map", "resolve", "qualify"}, (
        "the writer is executed by every stage after reading, and not by reading"
    )
    with (copied_package / "staging" / "writer.py").open("a") as handle:
        handle.write("\n# the staging writer changed\n")
    assert reuse.stage_key("read", {}).digest != before["read"]


def test_a_file_no_stage_declares_changes_no_key(copied_package: Path) -> None:
    before = {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}
    (copied_package / "cli.py").write_text((copied_package / "cli.py").read_text() + "\n# x\n")
    (copied_package / "verify" / "checks.py").write_text(
        (copied_package / "verify" / "checks.py").read_text() + "\n# x\n"
    )
    (copied_package / "survey_index").mkdir(exist_ok=True)
    (copied_package / "survey_index" / "extra.py").write_text("X = 1\n")
    (copied_package / "canonical" / "__pycache__").mkdir()
    (copied_package / "canonical" / "__pycache__" / "x.py").write_text("ignored")
    assert before == {name: reuse.stage_key(name, {}).digest for name in reuse.STAGES}


def test_the_content_of_a_named_file_and_not_its_timestamp_enters_the_key(tmp_path: Path) -> None:
    script = tmp_path / "harness.py"
    script.write_text("a = 1\n")
    first = reuse.stage_key("qualify", {}, files={"harness": [script]}).digest
    script.touch()
    assert reuse.stage_key("qualify", {}, files={"harness": [script]}).digest == first
    script.write_text("a = 2\n")
    assert reuse.stage_key("qualify", {}, files={"harness": [script]}).digest != first
    with pytest.raises(reuse.ReuseError, match="do not exist"):
        reuse.stage_key("qualify", {}, files={"harness": [tmp_path / "missing.py"]})


def test_the_lock_of_an_environment_is_found_where_the_environment_keeps_it(
    tmp_path: Path,
) -> None:
    assert reuse.environment_locks("core", tmp_path) == (reuse.core_lock(),)
    assert reuse.core_lock().name == "uv.lock" and reuse.core_lock().is_file()
    side = tmp_path / "envs" / "geochem"
    side.mkdir(parents=True)
    with pytest.raises(reuse.ReuseError, match="has no lock file"):
        reuse.environment_locks("geochem", tmp_path)
    (side / "environment.lock.txt").write_text("a\n")
    (side / "pip.lock.txt").write_text("b\n")
    (side / "environment.yml").write_text("not a lock\n")
    assert [p.name for p in reuse.environment_locks("geochem", tmp_path)] == [
        "environment.lock.txt",
        "pip.lock.txt",
    ]


# -- the stages ---------------------------------------------------------------------------------


def test_reading_is_stale_after_a_change_the_stage_declares_and_not_otherwise(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, copied_package: Path
) -> None:
    workspace = build_workspace(tmp_path / "work")
    module = tiny_module()
    ctx = workspace.context(module_resolver(module))
    manifest, entry = workspace.manifest(), workspace.entries()["tiny"]
    assert stage.read_source(ctx, manifest, workspace.entries()).status == "read"
    resolved = module_resolver(module)(manifest)
    assert resolved is not None

    def state() -> StageState:
        return stage.stage_state(ctx, manifest, entry, resolved)[0]

    assert state() is StageState.CURRENT, "a copy of the same framework files is current"
    versions(monkeypatch, pint="0.0.0", rdkit="0.0.0", sympy="0.0.0")
    assert state() is StageState.CURRENT, "reading declares none of these libraries"
    (copied_package / "qualify" / "run.py").write_text(
        (copied_package / "qualify" / "run.py").read_text() + "\n# x\n"
    )
    assert state() is StageState.CURRENT, "reading does not execute the qualify package"
    versions(monkeypatch, pyarrow="0.0.0")
    assert state() is StageState.STALE, "the installed version of pyarrow is part of the key"
    versions(monkeypatch)
    assert state() is StageState.CURRENT
    with (copied_package / "staging" / "manifest.py").open("a") as handle:
        handle.write("\n# x\n")
    assert state() is StageState.STALE, "a framework file the stage executes is part of the key"
    assert stage.read_source(ctx, manifest, workspace.entries()).status == "read"
    assert state() is StageState.CURRENT


def test_a_change_to_code_the_chemicals_and_thermo_readers_share_marks_both_stale(
    copied_package: Path,
) -> None:
    """The shared table rules and the shared Law and Bell declarations live in the staging
    package, which the read key covers for every reader, so neither reader's key can miss them."""
    readers = {
        name: staging_reader.ResolvedReader.from_module(name, importlib.import_module(f"{package}.{name}"))
        for name, package in (
            ("chemicals", "thermo_knowledge.readers"),
            ("thermo", "thermo_knowledge.readers"),
        )
    }

    def keys() -> dict[str, str]:
        return {
            name: staging_reader.reuse_key(source_id=name, tree_identity="t", reader=reader)[0]
            for name, reader in readers.items()
        }

    before = keys()
    assert before == keys() and before["chemicals"] != before["thermo"]
    for module in ("tabular.py", "shared_specs.py"):
        assert (PACKAGE / "staging" / module).is_file()
        assert not (PACKAGE / "readers" / "chemicals" / module).exists()
        previous = keys()
        with (copied_package / "staging" / module).open("a") as handle:
            handle.write("\n# a shared reader module changed\n")
        changed = keys()
        assert changed["chemicals"] != previous["chemicals"], module
        assert changed["thermo"] != previous["thermo"], module
    with (copied_package / "qualify" / "run.py").open("a") as handle:
        handle.write("\n# not executed by reading\n")
    assert keys() == changed


def test_a_side_reader_is_stale_when_the_lock_of_its_environment_changes(
    tmp_path: Path,
) -> None:
    workspace: Workspace = build_workspace(tmp_path / "work", reader_name="fake_side", environment="fakeenv")
    envs = copy_envs(tmp_path, lambda text: text)
    ctx = side_context(workspace, envs)
    manifest = workspace.manifest()
    assert stage.read_source(ctx, manifest, workspace.entries()).status == "read"
    resolved = ctx.resolver(manifest)
    assert resolved is not None
    assert stage.stage_state(ctx, manifest, workspace.entries()["tiny"], resolved)[0] is (
        StageState.CURRENT
    )
    lock = envs / "fakeenv" / "environment.lock.txt"
    lock.write_text(lock.read_text() + "a package moved\n")
    assert stage.stage_state(ctx, manifest, workspace.entries()["tiny"], resolved)[0] is (
        StageState.STALE
    )
    lock.unlink()
    with pytest.raises(StagingError, match="has no lock file"):
        stage.stage_state(ctx, manifest, workspace.entries()["tiny"], resolved)


def test_mapping_and_resolution_follow_their_declared_libraries_and_files(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, copied_package: Path
) -> None:
    mapping_env, _ = fake_environment(tmp_path / "work")
    decl = real_declaration()
    assert runner.run_identity(mapping_env, "fake").status == "mapped"
    assert resolve_all(mapping_env, decl=decl).status == "resolved"
    assert runner.run_records(mapping_env, "fake").status == "mapped"

    def statuses() -> tuple[str, str, str]:
        return (
            runner.run_identity(mapping_env, "fake").status,
            resolve_all(mapping_env, decl=decl).status,
            runner.run_records(mapping_env, "fake").status,
        )

    assert statuses() == ("current", "current", "current")
    versions(monkeypatch, sympy="0.0.0", numpy="0.0.0", scipy="0.0.0")
    assert statuses() == ("current", "current", "current"), "no mapping stage declares these"
    # rdkit computes the structure keys of resolution: resolution is stale, and with it its
    # result, whose hash the records of a mapping record as an input
    versions(monkeypatch, rdkit="0.0.0")
    assert runner.run_identity(mapping_env, "fake").status == "current"
    assert resolve_all(mapping_env, decl=decl).status == "resolved"
    assert runner.run_records(mapping_env, "fake").status == "mapped"
    assert statuses() == ("current", "current", "current")
    versions(monkeypatch)
    assert resolve_all(mapping_env, decl=decl).status == "resolved"  # rdkit is back
    assert runner.run_records(mapping_env, "fake").status == "mapped"
    # pint converts the units of a mapping: its phases are stale, and so is what reads them
    versions(monkeypatch, pint="0.0.0")
    assert runner.run_identity(mapping_env, "fake").status == "mapped"
    assert resolve_all(mapping_env, decl=decl).status == "resolved", (
        "its input, the phase-1 output, was made again"
    )
    assert runner.run_records(mapping_env, "fake").status == "mapped"
    versions(monkeypatch)
    # a change to a framework file every stage after reading executes
    assert runner.run_identity(mapping_env, "fake").status == "mapped"
    resolve_all(mapping_env, decl=decl)
    runner.run_records(mapping_env, "fake")
    assert statuses() == ("current", "current", "current")
    with (copied_package / "canonical" / "writer.py").open("a") as handle:
        handle.write("\n# the writer changed\n")
    assert runner.run_identity(mapping_env, "fake").status == "mapped"
    assert resolve_all(mapping_env, decl=decl).status == "resolved"
    assert isinstance(mapping_env, Environment)


def test_a_qualification_run_is_stale_after_an_oracle_script_a_provider_or_a_lock_changes(
    env: Env,  # noqa: F811
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    text = case_text()
    assert env.run(text).status == "ran"
    assert env.run(text).status == "current"
    # the oracle script
    script = env.tree / "oracles" / "fake.py"
    original = script.read_text()
    script.write_text(original + "\n# the harness was fixed\n")
    assert env.run(text).status == "ran"
    assert env.run(text).status == "current"
    # a library the evaluation uses, and one it does not
    versions(monkeypatch, rdkit="0.0.0")
    assert env.run(text).status == "current"
    versions(monkeypatch, sympy="0.0.0")
    assert env.run(text).status == "ran"
    assert env.run(text).status == "current"
    versions(monkeypatch)
    assert env.run(text).status == "ran", "the run was made with another version of sympy"
    # the lock of the environment the harness runs in
    lock = tmp_path / "uv.lock"
    lock.write_text("one\n")
    monkeypatch.setattr(reuse, "core_lock", lambda: lock)
    assert env.run(text).status == "ran"
    assert env.run(text).status == "current"
    lock.write_text("two\n")
    assert env.run(text).status == "ran"
    assert env.run(text).status == "current"

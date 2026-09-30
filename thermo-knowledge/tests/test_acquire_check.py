# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The lock, `tk acquire --check` and `--list`, exercised through the `tk` command."""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

import httpx
import pytest
from test_acquire_support import (
    HttpFixture,
    Route,
    independent_tree_hash,
    manifest_text,
    md5,
    sha256,
    write_manifest,
)
from typer.testing import CliRunner

from thermo_knowledge import config
from thermo_knowledge.cli import app

runner = CliRunner()

ONE = b"first file\n"
TWO = b"second file\n"


@dataclass
class Workspace:
    tmp_path: Path
    server: HttpFixture

    @property
    def sources(self) -> Path:
        return self.tmp_path / "sources"

    @property
    def lock(self) -> Path:
        return self.tmp_path / "sources.lock"

    @property
    def raw(self) -> Path:
        return self.tmp_path / "store" / "raw"

    def declare_file(self, source_id: str, path: str, data: bytes, tier: str = "A") -> None:
        body = (
            f'kind = "file"\nfiles = [{{ url = "{self.server.url(path)}", name = "f.txt", '
            f'sha256 = "{sha256(data)}" }}]'
        )
        write_manifest(self.sources, source_id, body, tier)

    def cli(self, *arguments: str):
        return runner.invoke(
            app,
            ["acquire", *arguments, "--sources", str(self.sources), "--lock", str(self.lock)],
        )

    def tree(self, source_id: str, pin: str) -> Path:
        return self.raw / source_id / pin / "tree"


@pytest.fixture
def workspace(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv(config.STORE_ENV, str(tmp_path / "store"))
    monkeypatch.setenv("NO_PROXY", "127.0.0.1")
    monkeypatch.delenv("HTTP_PROXY", raising=False)
    monkeypatch.delenv("http_proxy", raising=False)
    with HttpFixture() as server:
        server.routes["/one.txt"] = Route(ONE)
        server.routes["/two.txt"] = Route(TWO)
        yield Workspace(tmp_path, server)


def acquire_both(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    workspace.declare_file("beta", "/two.txt", TWO)
    result = workspace.cli()
    assert result.exit_code == 0, result.output


# -- the lock -----------------------------------------------------------------------------


def test_the_lock_is_json_with_sorted_keys_and_a_trailing_newline(workspace: Workspace) -> None:
    acquire_both(workspace)
    text = workspace.lock.read_text()
    assert text.endswith("}\n")
    assert not text.endswith("\n\n")
    data = json.loads(text)
    assert text == json.dumps(data, sort_keys=True, indent=2) + "\n"
    assert list(data["sources"]) == ["alpha", "beta"]
    entry = data["sources"]["alpha"]
    assert set(entry) == {
        "kind",
        "pin",
        "retrieved",
        "file_count",
        "total_bytes",
        "tree_hash",
        "resolved",
    }
    assert entry["total_bytes"] == len(ONE)
    assert entry["retrieved"].endswith("Z")


def test_the_lock_tree_hash_matches_an_independent_computation(workspace: Workspace) -> None:
    acquire_both(workspace)
    for source_id, data in (("alpha", ONE), ("beta", TWO)):
        entry = json.loads(workspace.lock.read_text())["sources"][source_id]
        assert entry["pin"] == sha256(data)[:12]
        assert entry["tree_hash"] == independent_tree_hash(workspace.tree(source_id, entry["pin"]))
    # the hash is over `<sha256>  <path>\n` lines: one file here
    from hashlib import sha256 as digest

    expected = digest(f"{sha256(ONE)}  f.txt\n".encode()).hexdigest()
    assert json.loads(workspace.lock.read_text())["sources"]["alpha"]["tree_hash"] == expected


def test_only_the_entries_of_acquired_sources_change(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    assert workspace.cli("alpha").exit_code == 0
    first = json.loads(workspace.lock.read_text())
    workspace.declare_file("beta", "/two.txt", TWO)
    assert workspace.cli("beta").exit_code == 0
    second = json.loads(workspace.lock.read_text())
    assert second["sources"]["alpha"] == first["sources"]["alpha"]
    assert set(second["sources"]) == {"alpha", "beta"}
    # naming one source leaves every other entry alone
    workspace.lock.write_text(
        workspace.lock.read_text().replace(
            second["sources"]["beta"]["retrieved"], "2000-01-01T00:00:00Z"
        )
    )
    assert workspace.cli("alpha").exit_code == 0
    assert json.loads(workspace.lock.read_text())["sources"]["beta"]["retrieved"] == (
        "2000-01-01T00:00:00Z"
    )


def test_a_deleted_lock_is_restored_from_the_store_without_fetching(workspace: Workspace) -> None:
    acquire_both(workspace)
    before = workspace.lock.read_text()
    workspace.lock.unlink()
    requests = len(workspace.server.log)
    result = workspace.cli()
    assert result.exit_code == 0, result.output
    assert "verified" in result.output
    assert len(workspace.server.log) == requests
    assert workspace.lock.read_text() == before


# -- --check ------------------------------------------------------------------------------


def test_check_passes_after_acquisition(workspace: Workspace) -> None:
    acquire_both(workspace)
    result = workspace.cli("--check")
    assert result.exit_code == 0, result.output
    assert "ok    alpha" in result.output
    assert "ok    beta" in result.output


def pin_of(workspace: Workspace, source_id: str) -> str:
    return json.loads(workspace.lock.read_text())["sources"][source_id]["pin"]


def test_check_reports_a_changed_file_by_path(workspace: Workspace) -> None:
    acquire_both(workspace)
    (workspace.tree("alpha", pin_of(workspace, "alpha")) / "f.txt").write_bytes(b"tampered\n")
    result = workspace.cli("--check")
    assert result.exit_code == 1
    assert "FAIL  alpha" in result.output
    assert "changed file: f.txt" in result.output
    assert "tree hash" in result.output
    assert "ok    beta" in result.output


def test_check_reports_a_missing_file_by_path(workspace: Workspace) -> None:
    acquire_both(workspace)
    (workspace.tree("alpha", pin_of(workspace, "alpha")) / "f.txt").unlink()
    result = workspace.cli("--check")
    assert result.exit_code == 1
    assert "missing file: f.txt" in result.output


def test_check_reports_an_extra_file_by_path(workspace: Workspace) -> None:
    acquire_both(workspace)
    tree = workspace.tree("alpha", pin_of(workspace, "alpha"))
    (tree / "sub").mkdir()
    (tree / "sub" / "stray.txt").write_text("extra")
    result = workspace.cli("--check")
    assert result.exit_code == 1
    assert "extra file: sub/stray.txt" in result.output


def test_check_reports_a_lock_that_disagrees_with_the_store(workspace: Workspace) -> None:
    acquire_both(workspace)
    data = json.loads(workspace.lock.read_text())
    data["sources"]["alpha"]["tree_hash"] = "0" * 64
    workspace.lock.write_text(json.dumps(data, sort_keys=True, indent=2) + "\n")
    result = workspace.cli("--check", "alpha")
    assert result.exit_code == 1
    assert "the lock records " + "0" * 64 in result.output


def test_check_reports_a_source_that_was_never_acquired(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    result = workspace.cli("--check")
    assert result.exit_code == 1
    assert "absent" in result.output


def test_check_reports_a_declared_pin_the_lock_does_not_record(workspace: Workspace) -> None:
    acquire_both(workspace)
    workspace.declare_file("alpha", "/two.txt", TWO)  # the declaration moved to another pin
    result = workspace.cli("--check", "alpha")
    assert result.exit_code == 1
    assert "manifest declares" in result.output


def test_a_not_acquired_source_is_not_a_check_failure(workspace: Workspace) -> None:
    write_manifest(workspace.sources, "ddbst", 'kind = "none"\nreason = "terms of use"')
    assert workspace.cli().exit_code == 0
    result = workspace.cli("--check")
    assert result.exit_code == 0, result.output
    assert "not acquired: terms of use" in result.output


def test_check_and_list_are_offline(workspace: Workspace, monkeypatch: pytest.MonkeyPatch) -> None:
    acquire_both(workspace)
    requests = len(workspace.server.log)

    def forbidden(*args: object, **kwargs: object) -> None:
        raise AssertionError("the network or git must not be touched")

    monkeypatch.setattr(httpx, "Client", forbidden)
    monkeypatch.setattr(subprocess, "run", forbidden)
    assert workspace.cli("--check").exit_code == 0
    assert workspace.cli("--list").exit_code == 0
    assert len(workspace.server.log) == requests


# -- --list -------------------------------------------------------------------------------


def test_list_shows_each_state(workspace: Workspace) -> None:
    acquire_both(workspace)
    workspace.declare_file("alpha", "/one.txt", ONE, tier="B")
    workspace.declare_file("beta", "/one.txt", ONE)  # declaration moved: lock records another pin
    workspace.declare_file("gamma", "/two.txt", TWO)  # never acquired
    write_manifest(workspace.sources, "delta", 'kind = "none"\nreason = "terms of use"')
    result = workspace.cli("--list")
    assert result.exit_code == 0, result.output
    rows = {line.split()[0]: line for line in result.output.splitlines()}
    assert list(rows) == ["alpha", "beta", "delta", "gamma"]
    assert rows["alpha"].split()[1:5] == ["B", "file", sha256(ONE)[:12], "present"]
    assert rows["beta"].split()[1:3] == ["A", "file"]
    assert " mismatched" in rows["beta"]
    assert sha256(TWO)[:12] in rows["beta"]  # the lock's pin is named in the explanation
    assert rows["gamma"].split()[1:5] == ["A", "file", sha256(TWO)[:12], "absent"]
    assert rows["delta"].split()[1:5] == ["A", "none", "-", "not"]
    assert "not acquired" in rows["delta"]


def test_list_reports_a_store_that_lost_its_directory_as_mismatched(workspace: Workspace) -> None:
    acquire_both(workspace)
    import shutil

    shutil.rmtree(workspace.raw / "alpha")
    result = workspace.cli("--list", "alpha")
    assert " mismatched" in result.output


def test_list_of_pages_and_git_shows_no_declared_pin_for_pages(workspace: Workspace) -> None:
    write_manifest(
        workspace.sources,
        "web",
        'kind = "pages"\nenumerator = "url_list"\nmin_interval_seconds = 1\nurls = ["http://x/y"]',
    )
    write_manifest(
        workspace.sources,
        "repo",
        f'kind = "git"\nurl = "file:///nowhere"\ncommit = "{"ab" * 20}"',
    )
    rows = {line.split()[0]: line.split() for line in workspace.cli("--list").output.splitlines()}
    assert rows["web"][1:5] == ["A", "pages", "-", "absent"]
    assert rows["repo"][1:5] == ["A", "git", "ab" * 6, "absent"]


# -- acquire: pins, refusals, exit codes --------------------------------------------------


def test_a_new_pin_is_a_new_directory_and_the_old_one_is_kept(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    assert workspace.cli().exit_code == 0
    old = workspace.raw / "alpha" / sha256(ONE)[:12]
    workspace.declare_file("alpha", "/two.txt", TWO)
    result = workspace.cli()
    assert result.exit_code == 0, result.output
    assert old.is_dir()
    assert (workspace.raw / "alpha" / sha256(TWO)[:12]).is_dir()
    assert pin_of(workspace, "alpha") == sha256(TWO)[:12]


def test_a_tampered_store_is_refused_rather_than_repaired(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    assert workspace.cli().exit_code == 0
    tree = workspace.tree("alpha", sha256(ONE)[:12])
    (tree / "f.txt").write_bytes(b"tampered\n")
    lock_before = workspace.lock.read_bytes()
    requests = len(workspace.server.log)
    result = workspace.cli()
    assert result.exit_code == 1
    assert "changed file: f.txt" in result.output
    assert (tree / "f.txt").read_bytes() == b"tampered\n"
    assert workspace.lock.read_bytes() == lock_before
    assert len(workspace.server.log) == requests


def test_a_lock_that_disagrees_with_the_store_is_refused_on_acquire(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    assert workspace.cli().exit_code == 0
    data = json.loads(workspace.lock.read_text())
    data["sources"]["alpha"]["tree_hash"] = "0" * 64
    workspace.lock.write_text(json.dumps(data, sort_keys=True, indent=2) + "\n")
    result = workspace.cli()
    assert result.exit_code == 1
    assert "the lock records " + "0" * 64 in result.output


def test_a_stray_partial_directory_is_ignored_and_replaced(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    partial = workspace.raw / "alpha" / ".partial"
    (partial / "tree").mkdir(parents=True)
    (partial / "tree" / "leftover").write_text("from an interrupted run")
    assert " absent" in workspace.cli("--list").output
    assert workspace.cli().exit_code == 0
    assert not partial.exists()
    assert not (workspace.tree("alpha", sha256(ONE)[:12]) / "leftover").exists()


def test_one_failing_source_does_not_stop_the_others(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    body = f'kind = "file"\nfiles = [{{ url = "{workspace.server.url("/two.txt")}", name = "f.txt", md5 = "{"0" * 32}" }}]'
    write_manifest(workspace.sources, "beta", body)
    workspace.declare_file("gamma", "/two.txt", TWO)
    result = workspace.cli()
    assert result.exit_code == 1
    assert "refused" in result.output
    assert set(json.loads(workspace.lock.read_text())["sources"]) == {"alpha", "gamma"}
    assert md5(TWO) in result.output


def test_unknown_ids_and_conflicting_options_are_usage_errors(workspace: Workspace) -> None:
    workspace.declare_file("alpha", "/one.txt", ONE)
    unknown = workspace.cli("nosuch")
    assert unknown.exit_code == 2
    assert "nosuch" in unknown.output
    assert "alpha" in unknown.output
    assert workspace.cli("--check", "--list").exit_code == 2
    assert workspace.cli("--list", "nosuch").exit_code == 2


def test_manifest_errors_are_reported_with_the_file_and_fail(workspace: Workspace) -> None:
    workspace.sources.mkdir()
    (workspace.sources / "broken.toml").write_text(
        manifest_text("broken", 'kind = "none"\nreason = "r"\nbogus = 1')
    )
    for arguments in ((), ("--check",), ("--list",)):
        result = workspace.cli(*arguments)
        assert result.exit_code == 1
        assert "broken.toml" in result.output
        assert "acquire.bogus" in result.output


def test_a_missing_sources_directory_fails(workspace: Workspace) -> None:
    result = workspace.cli("--list")
    assert result.exit_code == 1
    assert "sources directory does not exist" in result.output


def test_the_default_sources_directory_is_the_trees(monkeypatch: pytest.MonkeyPatch) -> None:
    from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir

    assert default_sources_dir() == config.TREE_DIR / "sources"
    assert default_lock_path() == config.TREE_DIR / "sources.lock"

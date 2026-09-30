# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `git`: exactly the declared commit, no history, tags verified, submodules, sparse."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from test_acquire_support import (
    GitFixture,
    independent_tree_hash,
    make_context,
    make_git_fixture,
    run_acquire,
    tree_files,
    write_manifest,
)

from thermo_knowledge.acquire import git as git_kind
from thermo_knowledge.acquire.store import ACQUISITION_NAME


@pytest.fixture
def origin(tmp_path: Path) -> GitFixture:
    return make_git_fixture(tmp_path / "fixture")


def git_manifest(origin: GitFixture, commit: str, extra: str = "") -> str:
    return f'kind = "git"\nurl = "{origin.url}"\ncommit = "{commit}"\n{extra}'


def setup(tmp_path: Path, origin: GitFixture, commit: str, extra: str = ""):
    sources = tmp_path / "sources"
    write_manifest(sources, "gitsrc", git_manifest(origin, commit, extra))
    return make_context(tmp_path), sources, tmp_path / "sources.lock"


def stored_tree(ctx, commit: str) -> Path:
    return ctx.raw_dir / "gitsrc" / commit[:12] / "tree"


def test_stored_tree_is_the_files_of_the_declared_commit(
    tmp_path: Path, origin: GitFixture
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    assert result.status == "acquired"
    tree = stored_tree(ctx, origin.second)
    assert tree_files(tree) == origin.files_at(origin.second)
    assert not list(tree.rglob(".git"))
    # the first and third commits differ from the second: this is exactly the declared one
    assert tree_files(tree) != origin.files_at(origin.third)
    assert tree_files(tree) != origin.files_at(origin.first)


def test_bytes_are_untouched_by_the_repositorys_own_eol_attributes(
    tmp_path: Path, origin: GitFixture
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second)
    run_acquire(ctx, sources, lock)
    assert (stored_tree(ctx, origin.second) / "data" / "a.txt").read_bytes() == (
        b"alpha\nbeta\ngamma\n"
    )


def test_acquisition_record_and_lock_entry(tmp_path: Path, origin: GitFixture) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second, 'tag = "v0.2"\n')
    run_acquire(ctx, sources, lock)
    pin_dir = ctx.raw_dir / "gitsrc" / origin.second[:12]
    record = json.loads((pin_dir / ACQUISITION_NAME).read_text())
    assert record["pin"] == origin.second[:12]
    assert record["resolved"] == origin.second
    assert record["tool_versions"]["git"]
    assert record["details"]["tag"] == "v0.2"
    assert {f["path"] for f in record["files"]} == set(origin.files_at(origin.second))
    entry = json.loads(lock.read_text())["sources"]["gitsrc"]
    assert entry["kind"] == "git"
    assert entry["pin"] == origin.second[:12]
    assert entry["resolved"] == origin.second
    assert entry["file_count"] == len(origin.files_at(origin.second))
    assert entry["tree_hash"] == independent_tree_hash(pin_dir / "tree")


@pytest.mark.parametrize(("tag", "commit_index"), [("v0.1", 0), ("v0.2", 1)])
def test_lightweight_and_annotated_tags_that_name_the_commit_are_accepted(
    tmp_path: Path, origin: GitFixture, tag: str, commit_index: int
) -> None:
    commit = origin.commits[commit_index]
    ctx, sources, lock = setup(tmp_path, origin, commit, f'tag = "{tag}"\n')
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message


@pytest.mark.parametrize("tag", ["v0.1", "v0.2"])
def test_a_tag_that_names_a_different_commit_is_refused(
    tmp_path: Path, origin: GitFixture, tag: str
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.third, f'tag = "{tag}"\n')
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert result.status == "refused"
    assert f"tag '{tag}'" in result.message
    assert origin.third in result.message
    assert origin.url in result.message
    assert not (ctx.raw_dir / "gitsrc").exists()
    assert not lock.exists()


def test_a_missing_tag_is_refused(tmp_path: Path, origin: GitFixture) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second, 'tag = "v9"\n')
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "does not exist" in result.message


def test_declared_submodule_is_fetched_at_its_recorded_commit(
    tmp_path: Path, origin: GitFixture
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second, 'submodules = ["vendor/sub"]\n')
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    tree = stored_tree(ctx, origin.second)
    assert (tree / "vendor" / "sub" / "lib.txt").read_text() == "library\n"
    assert not list(tree.rglob(".git"))
    record = json.loads(
        (ctx.raw_dir / "gitsrc" / origin.second[:12] / ACQUISITION_NAME).read_text()
    )
    assert origin.sub_commit in record["details"]["submodule vendor/sub"]


def test_undeclared_submodule_is_not_fetched(tmp_path: Path, origin: GitFixture) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second)
    run_acquire(ctx, sources, lock)
    assert not (stored_tree(ctx, origin.second) / "vendor" / "sub" / "lib.txt").exists()


def test_a_declared_path_that_is_not_a_submodule_is_refused(
    tmp_path: Path, origin: GitFixture
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second, 'submodules = ["data"]\n')
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "not a submodule" in result.message
    assert not (ctx.raw_dir / "gitsrc").exists()


def test_sparse_keeps_the_listed_paths_and_licence_files(
    tmp_path: Path, origin: GitFixture
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second, 'sparse = ["data"]\n')
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    kept = tree_files(stored_tree(ctx, origin.second))
    assert set(kept) == {"data/a.txt", "data/b.json", "LICENSE", "NOTICE.md"}
    full = origin.files_at(origin.second)
    assert all(kept[name] == full[name] for name in ("data/b.json", "LICENSE", "NOTICE.md"))


def test_sparse_accepts_a_single_file_and_a_glob(tmp_path: Path, origin: GitFixture) -> None:
    ctx, sources, lock = setup(
        tmp_path, origin, origin.second, 'sparse = ["README.md", "data/*.json"]\n'
    )
    run_acquire(ctx, sources, lock)
    assert set(tree_files(stored_tree(ctx, origin.second))) == {
        "README.md",
        "data/b.json",
        "LICENSE",
        "NOTICE.md",
    }


def test_rerunning_does_nothing(
    tmp_path: Path, origin: GitFixture, monkeypatch: pytest.MonkeyPatch
) -> None:
    ctx, sources, lock = setup(tmp_path, origin, origin.second)
    run_acquire(ctx, sources, lock)
    before_lock = lock.read_bytes()
    before_tree = tree_files(stored_tree(ctx, origin.second))

    def forbidden(*args: object, **kwargs: object) -> None:
        raise AssertionError("git must not be run for a source already in the store")

    monkeypatch.setattr(git_kind, "run_git", forbidden)
    [result] = run_acquire(ctx, sources, lock)
    assert result.status == "unchanged"
    assert lock.read_bytes() == before_lock
    assert tree_files(stored_tree(ctx, origin.second)) == before_tree


def test_an_unreachable_commit_leaves_nothing_behind(tmp_path: Path, origin: GitFixture) -> None:
    ctx, sources, lock = setup(tmp_path, origin, "0" * 40)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "fetch" in result.message
    assert not (ctx.raw_dir / "gitsrc").exists()
    assert not lock.exists()


def test_an_unreachable_url_is_reported_with_the_source(tmp_path: Path) -> None:
    sources = tmp_path / "sources"
    write_manifest(
        sources,
        "gitsrc",
        f'kind = "git"\nurl = "{(tmp_path / "nowhere").as_uri()}"\ncommit = "{"1" * 40}"',
    )
    [result] = run_acquire(make_context(tmp_path), sources, tmp_path / "sources.lock")
    assert not result.ok
    assert result.message.startswith("gitsrc:")


@pytest.mark.parametrize(
    ("base", "relative", "expected"),
    [
        ("https://h.example/a/b.git", "../c.git", "https://h.example/a/c.git"),
        ("https://h.example/a/b.git", "./c.git", "https://h.example/a/b.git/c.git"),
        ("https://h.example/a/b.git", "https://other/x.git", "https://other/x.git"),
    ],
)
def test_relative_submodule_urls_resolve_against_the_superproject(
    base: str, relative: str, expected: str
) -> None:
    assert git_kind.resolve_submodule_url(base, relative) == expected


def test_sparse_patterns_cover_licence_files_in_any_case() -> None:
    patterns = git_kind.sparse_patterns(["data", "/x/y"])
    assert patterns[:2] == ["/data", "/x/y"]
    assert any(p.startswith("/[Ll][Ii][Cc][Ee][Nn][Ss][Ee]") for p in patterns)

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `local` and kind `none`: nothing is copied; the lock records the verified revision."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from test_acquire_support import (
    commit_all,
    git,
    make_context,
    run_acquire,
    write,
    write_manifest,
)

from thermo_knowledge.acquire.manifest import load_sources
from thermo_knowledge.acquire.runner import check_sources


class Checkout:
    def __init__(self, tmp_path: Path) -> None:
        self.repo_root = tmp_path / "repo"
        self.path = self.repo_root / "external" / "lib"
        self.path.mkdir(parents=True)
        git(self.path, "init", "--quiet")
        write(self.path, "a.txt", "one\n")
        self.first = commit_all(self.path, "first")
        git(self.path, "tag", "v1")
        write(self.path, "a.txt", "two\n")
        self.second = commit_all(self.path, "second")
        self.tmp_path = tmp_path
        self.sources = tmp_path / "sources"
        self.lock = tmp_path / "sources.lock"
        self.ctx = make_context(tmp_path, repo_root=self.repo_root)

    def declare(self, selector: str, path: str = "external/lib") -> None:
        write_manifest(self.sources, "lib", f'kind = "local"\npath = "{path}"\n{selector}')

    def run(self):
        [result] = run_acquire(self.ctx, self.sources, self.lock)
        return result


@pytest.fixture
def checkout(tmp_path: Path) -> Checkout:
    return Checkout(tmp_path)


def test_a_clean_checkout_at_the_declared_commit_verifies(checkout: Checkout) -> None:
    checkout.declare(f'commit = "{checkout.second}"')
    result = checkout.run()
    assert result.ok, result.message
    assert result.status == "verified"
    entry = json.loads(checkout.lock.read_text())["sources"]["lib"]
    assert entry["kind"] == "local"
    assert entry["pin"] == checkout.second[:12]
    assert entry["resolved"] == checkout.second
    assert entry["path"] == "external/lib"
    assert entry["file_count"] == 0
    assert entry["tree_hash"] is None
    assert not checkout.ctx.raw_dir.exists()  # nothing was copied or stored


def test_a_tag_selects_the_declared_revision(checkout: Checkout) -> None:
    git(checkout.path, "checkout", "--quiet", "v1")
    checkout.declare('tag = "v1"')
    result = checkout.run()
    assert result.ok, result.message
    assert json.loads(checkout.lock.read_text())["sources"]["lib"]["resolved"] == checkout.first


def test_a_different_revision_is_refused(checkout: Checkout) -> None:
    checkout.declare(f'commit = "{checkout.first}"')
    result = checkout.run()
    assert not result.ok
    assert checkout.second in result.message
    assert checkout.first in result.message
    assert not checkout.lock.exists()


def test_a_tag_naming_another_revision_is_refused(checkout: Checkout) -> None:
    checkout.declare('tag = "v1"')
    result = checkout.run()
    assert not result.ok
    assert "'v1'" in result.message


@pytest.mark.parametrize("change", ["modify", "untracked", "staged"])
def test_a_dirty_working_tree_is_refused(checkout: Checkout, change: str) -> None:
    checkout.declare(f'commit = "{checkout.second}"')
    if change == "modify":
        write(checkout.path, "a.txt", "changed\n")
    elif change == "untracked":
        write(checkout.path, "new.txt", "new\n")
    else:
        write(checkout.path, "b.txt", "staged\n")
        git(checkout.path, "add", "b.txt")
    result = checkout.run()
    assert not result.ok
    assert "local changes" in result.message
    assert not checkout.lock.exists()


def test_a_directory_that_is_not_its_own_checkout_is_refused(checkout: Checkout) -> None:
    (checkout.path / "inner").mkdir()
    checkout.declare(f'commit = "{checkout.second}"', path="external/lib/inner")
    result = checkout.run()
    assert not result.ok
    assert "not the top level" in result.message


def test_a_missing_checkout_is_refused(checkout: Checkout) -> None:
    checkout.declare(f'commit = "{checkout.second}"', path="external/absent")
    result = checkout.run()
    assert not result.ok
    assert "does not exist" in result.message


def test_rerunning_keeps_the_lock_entry(checkout: Checkout) -> None:
    checkout.declare(f'commit = "{checkout.second}"')
    checkout.run()
    before = checkout.lock.read_bytes()
    result = checkout.run()
    assert result.status == "unchanged"
    assert checkout.lock.read_bytes() == before


def test_check_reverifies_the_checkout_offline(checkout: Checkout) -> None:
    checkout.declare(f'commit = "{checkout.second}"')
    checkout.run()
    manifests = list(load_sources(checkout.sources).values())
    from thermo_knowledge.acquire.lock import read_lock

    entries = read_lock(checkout.lock)
    assert all(r.ok for r in check_sources(checkout.ctx, manifests, entries))
    write(checkout.path, "a.txt", "changed\n")
    [result] = check_sources(checkout.ctx, manifests, entries)
    assert not result.ok
    assert "local changes" in "\n".join(result.problems)


def test_a_none_source_is_recorded_with_its_reason(tmp_path: Path) -> None:
    sources = tmp_path / "sources"
    write_manifest(
        sources, "ddbst", 'kind = "none"\nreason = "terms of use; permission route open"'
    )
    ctx = make_context(tmp_path)
    lock = tmp_path / "sources.lock"
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok
    assert result.status == "not acquired"
    entry = json.loads(lock.read_text())["sources"]["ddbst"]
    assert entry["kind"] == "none"
    assert entry["reason"] == "terms of use; permission route open"
    assert entry["pin"] is None
    assert entry["tree_hash"] is None
    assert not ctx.raw_dir.exists()
    before = lock.read_bytes()
    [again] = run_acquire(ctx, sources, lock)
    assert again.status == "unchanged"
    assert lock.read_bytes() == before

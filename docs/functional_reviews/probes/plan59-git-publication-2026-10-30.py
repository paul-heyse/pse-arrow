#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""D09 research probe: real Git publication/index races in disposable repositories.

This is a reproducible investigation, not the product commit implementation. Run with
``uv run --no-sync python docs/functional_reviews/probes/plan59-git-publication-2026-10-30.py``.
Only temporary repositories are modified. Git merge-tree supplies content/rename semantics.
"""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from collections.abc import Callable
from dataclasses import dataclass


class Refused(Exception):
    """No ref/index publication occurred."""


@dataclass(frozen=True)
class Proposal:
    head: str
    branch: str
    commit: str
    observed: dict[str, bytes | None]


@dataclass(frozen=True)
class Outcome:
    state: str
    commit: str
    reconciled_tree: str


class Repository:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.environment = dict(os.environ)
        self.environment.update(
            GIT_CONFIG_NOSYSTEM="1",
            GIT_CONFIG_GLOBAL=os.devnull,
            GIT_AUTHOR_NAME="D09 probe",
            GIT_AUTHOR_EMAIL="probe@example.invalid",
            GIT_COMMITTER_NAME="D09 probe",
            GIT_COMMITTER_EMAIL="probe@example.invalid",
        )
        self.git("init", "-q", "--initial-branch=main")
        self.write("a", "a0\n" + "middle\n" * 10 + "z0\n")
        self.write("b", "b0\n")
        self.write("c", "c0\n")
        self.git("add", ".")
        self.git("commit", "-qm", "base")

    def git(
        self,
        *args: str,
        index: Path | None = None,
        input_bytes: bytes | None = None,
        check: bool = True,
    ) -> subprocess.CompletedProcess[bytes]:
        environment = self.environment.copy()
        if index is not None:
            environment["GIT_INDEX_FILE"] = str(index)
        return subprocess.run(
            ["git", *args],
            cwd=self.root,
            env=environment,
            input=input_bytes,
            capture_output=True,
            check=check,
        )

    def oid(self, *args: str, index: Path | None = None) -> str:
        return self.git(*args, index=index).stdout.decode().strip()

    def write(self, name: str, value: str) -> None:
        (self.root / name).write_text(value)

    def index_path(self) -> Path:
        return self.root / self.oid("rev-parse", "--git-path", "index")

    def head(self) -> str:
        return self.oid("rev-parse", "HEAD")

    def staged(self, path: str) -> bytes:
        return self.git("show", f":{path}").stdout

    def content(self, path: str) -> bytes | None:
        file = self.root / path
        return file.read_bytes() if file.exists() else None

    def prepare(
        self,
        selected: list[str],
        formatter: Callable[[bytes], bytes] = lambda value: value,
    ) -> Proposal:
        head = self.head()
        branch_result = self.git("symbolic-ref", "-q", "HEAD", check=False)
        if branch_result.returncode:
            raise Refused("this probe qualifies attached branches only")
        branch = branch_result.stdout.decode().strip()
        observed = {path: self.content(path) for path in selected}
        with tempfile.TemporaryDirectory(prefix="d09-prepare-") as temporary:
            index = Path(temporary) / "index"
            self.git("read-tree", head, index=index)
            for path, content in observed.items():
                if content is None:
                    self.git("update-index", "--force-remove", "--", path, index=index)
                else:
                    blob = (
                        self.git("hash-object", "-w", "--stdin", input_bytes=formatter(content))
                        .stdout.decode().strip()
                    )
                    self.git("update-index", "--add", "--cacheinfo", f"100644,{blob},{path}", index=index)
            tree = self.oid("write-tree", index=index)
            commit = self.git("commit-tree", tree, "-p", head, input_bytes=b"selected change\n").stdout.decode().strip()
        return Proposal(head, branch, commit, observed)

    def publish(
        self,
        proposal: Proposal,
        *,
        before_lock: Callable[[], None] = lambda: None,
        after_lock: Callable[[], None] = lambda: None,
        fail_after_ref: bool = False,
    ) -> Outcome:
        index = self.index_path()
        snapshot = index.read_bytes()
        with tempfile.TemporaryDirectory(prefix="d09-reconcile-") as temporary:
            copied = Path(temporary) / "index"
            copied.write_bytes(snapshot)
            staged_tree = self.oid("write-tree", index=copied)
            staged_commit = self.git("commit-tree", staged_tree, "-p", proposal.head, input_bytes=b"private staged intent\n").stdout.decode().strip()
            merged = self.git("merge-tree", "--write-tree", proposal.commit, staged_commit, check=False)
            if merged.returncode:
                raise Refused("staged intent conflicts or native merge unavailable")
            reconciled_tree = merged.stdout.decode().splitlines()[0]
            self.git("read-tree", reconciled_tree, index=copied)
            prepared_index = copied.read_bytes()
        before_lock()
        lock = index.with_name(index.name + ".lock")
        try:
            fd = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except FileExistsError as error:
            raise Refused("index writer active") from error
        try:
            with os.fdopen(fd, "wb") as stream:
                stream.write(prepared_index)
                stream.flush()
                os.fsync(stream.fileno())
            if index.read_bytes() != snapshot:
                raise Refused("index changed during private preparation")
            if any(self.content(path) != value for path, value in proposal.observed.items()):
                raise Refused("selected worktree content changed")
            if self.git("symbolic-ref", "-q", "HEAD", check=False).stdout.decode().strip() != proposal.branch:
                raise Refused("selected branch changed")
            after_lock()
            published = self.git("update-ref", proposal.branch, proposal.commit, proposal.head, check=False)
            if published.returncode:
                raise Refused("HEAD moved")
            if fail_after_ref:
                return Outcome("committed-with-unreconciled-index", proposal.commit, reconciled_tree)
            os.replace(lock, index)
            return Outcome("committed", proposal.commit, reconciled_tree)
        finally:
            lock.unlink(missing_ok=True)


class PublicationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="plan59-git-")
        self.addCleanup(self.directory.cleanup)
        self.repo = Repository(Path(self.directory.name))

    def test_unrelated_staged_entry_is_preserved(self) -> None:
        r = self.repo
        r.write("b", "staged b\n")
        r.git("add", "b")
        r.write("a", "selected a\n")
        proposal = r.prepare(["a"])
        before = r.staged("b")
        outcome = r.publish(proposal)
        self.assertEqual(outcome.state, "committed")
        self.assertEqual(r.staged("b"), before)
        self.assertEqual(r.git("show", "HEAD:b").stdout, b"b0\n")

    def test_same_path_partial_staged_intent_is_transplanted(self) -> None:
        r = self.repo
        original = r.content("a")
        assert original is not None
        r.write("a", original.decode().replace("a0", "committed first"))
        proposal = r.prepare(["a"])
        r.write("a", original.decode().replace("z0", "staged last"))
        r.git("add", "a")
        r.write("a", original.decode().replace("a0", "committed first"))
        r.publish(proposal)
        expected = original.replace(b"a0", b"committed first").replace(b"z0", b"staged last")
        self.assertEqual(r.staged("a"), expected)
        self.assertEqual(r.git("show", "HEAD:a").stdout, original.replace(b"a0", b"committed first"))

    def test_conflicting_same_path_stage_refuses_without_changes(self) -> None:
        r = self.repo
        r.write("b", "selected\n")
        proposal = r.prepare(["b"])
        r.write("b", "different staged\n")
        r.git("add", "b")
        r.write("b", "selected\n")
        head, index, work = r.head(), r.index_path().read_bytes(), r.content("b")
        with self.assertRaises(Refused):
            r.publish(proposal)
        self.assertEqual((r.head(), r.index_path().read_bytes(), r.content("b")), (head, index, work))

    def test_stage_during_private_reconciliation_refuses(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        def stage() -> None:
            r.write("c", "new concurrent stage\n")
            r.git("add", "c")
        with self.assertRaises(Refused):
            r.publish(proposal, before_lock=stage)
        self.assertEqual(r.head(), proposal.head)
        self.assertEqual(r.staged("c"), b"new concurrent stage\n")

    def test_head_movement_under_index_lock_refuses(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        other = r.git("commit-tree", r.oid("rev-parse", "HEAD^{tree}"), "-p", proposal.head, input_bytes=b"concurrent head\n").stdout.decode().strip()
        index = r.index_path().read_bytes()
        with self.assertRaises(Refused):
            r.publish(proposal, after_lock=lambda: r.git("update-ref", "HEAD", other, proposal.head))
        self.assertEqual(r.head(), other)
        self.assertEqual(r.index_path().read_bytes(), index)

    def test_worktree_edit_refuses_and_survives(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        r.write("a", "concurrent working edit\n")
        index = r.index_path().read_bytes()
        with self.assertRaises(Refused):
            r.publish(proposal)
        self.assertEqual(r.head(), proposal.head)
        self.assertEqual(r.index_path().read_bytes(), index)
        self.assertEqual(r.content("a"), b"concurrent working edit\n")

    def test_branch_switch_refuses_and_preserves_other_branch(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        r.git("checkout", "-qb", "other")
        index = r.index_path().read_bytes()
        with self.assertRaises(Refused):
            r.publish(proposal)
        self.assertEqual(r.oid("symbolic-ref", "HEAD"), "refs/heads/other")
        self.assertEqual(r.head(), proposal.head)
        self.assertEqual(r.index_path().read_bytes(), index)

    def test_git_stage_respects_publication_index_lock(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        def stage() -> None:
            r.write("c", "working c\n")
            result = r.git("add", "c", check=False)
            self.assertNotEqual(result.returncode, 0)
        r.publish(proposal, after_lock=stage)
        self.assertEqual(r.staged("c"), b"c0\n")
        self.assertEqual(r.content("c"), b"working c\n")

    def test_rename_delete_staged_intent_survives(self) -> None:
        r = self.repo
        r.write("a", "selected\n")
        proposal = r.prepare(["a"])
        r.git("mv", "b", "renamed")
        r.git("rm", "c")
        r.publish(proposal)
        self.assertEqual(r.staged("renamed"), b"b0\n")
        self.assertEqual(r.git("ls-files", "b", "c").stdout, b"")
        self.assertEqual(r.git("show", "HEAD:b").stdout, b"b0\n")

    def test_formatter_failure_and_private_format_preserve_worktree(self) -> None:
        r = self.repo
        r.write("b", "unformatted\n")
        head, index = r.head(), r.index_path().read_bytes()
        def failed(_: bytes) -> bytes:
            raise ValueError("formatter failed")
        with self.assertRaises(ValueError):
            r.prepare(["b"], formatter=failed)
        self.assertEqual((r.head(), r.index_path().read_bytes()), (head, index))
        proposal = r.prepare(["b"], formatter=lambda value: value.upper())
        r.publish(proposal)
        self.assertEqual(r.git("show", "HEAD:b").stdout, b"UNFORMATTED\n")
        self.assertEqual(r.content("b"), b"unformatted\n")

    def test_post_publication_failure_is_not_retried(self) -> None:
        r = self.repo
        r.write("b", "selected\n")
        proposal = r.prepare(["b"])
        index = r.index_path().read_bytes()
        result = r.publish(proposal, fail_after_ref=True)
        self.assertEqual(result.state, "committed-with-unreconciled-index")
        self.assertEqual(r.head(), proposal.commit)
        self.assertEqual(r.index_path().read_bytes(), index)
        self.assertFalse(r.index_path().with_suffix(".lock").exists())
        self.assertEqual(r.git("rev-list", "--count", "HEAD").stdout.strip(), b"2")


if __name__ == "__main__":
    unittest.main(verbosity=2)

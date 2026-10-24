# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Activity membership, resumable coverage, provenance and selected reads."""

# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import datetime as dt
import json
import tempfile
import unittest
from pathlib import Path

from scripts import session_retrospective as history

NOW = dt.datetime(2026, 10, 23, 20, tzinfo=dt.UTC)
STAMP = NOW.isoformat()
OLD = "2026-09-01T10:00:00+00:00"


def meta(session: str, cwd: str = "/checkout", **extra: object) -> dict:
    return {
        "type": "session_meta",
        "timestamp": OLD,
        "payload": {"id": session, "cwd": cwd, **extra},
    }


def record(kind: str, **payload: object) -> dict:
    return {
        "type": "response_item",
        "timestamp": STAMP,
        "payload": {"type": kind, **payload},
    }


def write(path: Path, *items: dict) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(item) + "\n" for item in items))
    return path


class RetrospectiveTests(unittest.TestCase):
    def test_activity_across_old_dates_archive_explicit_worktree_and_parent(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "sessions"
            write(
                root / "2026/09/01/old.jsonl",
                meta("parent"),
                record(
                    "function_call",
                    call_id="c1",
                    name="exec_command",
                    arguments="SECRET",
                ),
            )
            write(
                root / "2026/01/01/child.jsonl",
                meta(
                    "child",
                    "/foreign",
                    source={
                        "subagent": {"thread_spawn": {"parent_thread_id": "parent"}}
                    },
                ),
                record("function_call", call_id="c2", name="child_tool"),
            )
            write(
                root / "2026/09/01/worktree.jsonl",
                meta("worktree", "/selected-tree"),
                record("function_call", call_id="c3", name="worktree_tool"),
            )
            write(
                root / "foreign.jsonl",
                meta("foreign", "/checkout-similar"),
                record("function_call", call_id="c4", name="foreign_tool"),
            )
            write(
                root.parent / "archived_sessions/ancient.jsonl",
                meta("archive"),
                record("function_call", call_id="c5", name="archived_tool"),
            )
            result = history.inspect(
                root,
                Path("/checkout"),
                related_checkouts=(Path("/selected-tree"),),
                now=NOW,
                query="child_tool",
            )
            self.assertEqual(
                result["tool_calls"],
                {
                    "exec_command": 1,
                    "child_tool": 1,
                    "worktree_tool": 1,
                    "archived_tool": 1,
                },
            )
            self.assertEqual(
                {source["session_id"] for source in result["sources"]},
                {"parent", "child", "worktree", "archive"},
            )
            self.assertTrue(result["discovery"]["complete"])
            self.assertTrue(result["read"]["complete"])
            self.assertFalse(result["truncated"])
            self.assertEqual(result["recall_matches"][0]["originating_session_id"], "child")
            self.assertNotIn("SECRET", json.dumps(result))

    def test_call_role_dedup_preserves_attempts_outputs_and_inherited_provenance(
        self,
    ) -> None:
        failure = '{"exit_code": 101, "output": "could not compile SECRET"}'
        call = record(
            "function_call", call_id="call-1", name="exec_command", arguments="SECRET"
        )
        output = record("function_call_output", call_id="call-1", output=failure)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write(
                root / "parent.jsonl",
                meta("parent"),
                call,
                output,
                record("function_call", call_id="call-2", name="exec_command"),
                record("function_call_output", call_id="call-2", output=failure),
                record(
                    "function_call_output",
                    call_id="call-1",
                    output="different poll SECRET",
                ),
            )
            write(
                root / "child.jsonl",
                meta("child", parent_thread_id="parent"),
                call,
                output,
            )
            result = history.recall(
                root, Path("/checkout"), "could not compile", now=NOW
            )
            self.assertEqual(result["tool_calls"], {"exec_command": 2})
            self.assertEqual(
                result["observed_record_classes"],
                {"observed_exit_101": 2, "compile_failure": 2},
            )
            self.assertEqual(result["inherited_duplicates_removed"], 2)
            self.assertEqual(len(result["recall_matches"]), 2)
            inherited = [
                match
                for match in result["recall_matches"]
                if match["inherited_sources"]
            ]
            self.assertEqual(len(inherited), 1)
            self.assertEqual(inherited[0]["originating_session_id"], "parent")
            self.assertNotIn("SECRET", json.dumps(result))

    def test_identical_records_without_call_identity_remain_distinct_observations(
        self,
    ) -> None:
        output = record("function_call_output", output="Permission denied")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write(root / "one.jsonl", meta("one"), output, output)
            write(root / "two.jsonl", meta("two"), output)
            result = history.inspect(root, Path("/checkout"), now=NOW)
            self.assertEqual(result["observed_record_classes"], {"permission": 3})
            self.assertEqual(result["inherited_duplicates_removed"], 0)

    def test_interrupted_discovery_and_read_resume_without_secret_buffers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write(
                root / "2020/01/01/old.jsonl",
                meta("old"),
                *[
                    record(
                        "function_call",
                        call_id=str(i),
                        name="exec_command",
                        arguments="SECRET",
                    )
                    for i in range(7)
                ],
            )
            cursor = None
            pages = 0
            first = None
            while True:
                result = history.inspect(
                    root,
                    Path("/checkout"),
                    now=NOW,
                    continuation=cursor,
                    max_entries=1,
                    max_files=1,
                    max_records=2,
                    max_record_bytes=1024,
                    max_bytes=4096,
                )
                first = first or result
                pages += 1
                self.assertLess(pages, 30)
                self.assertLessEqual(result["read"]["records_this_page"], 2)
                self.assertNotIn("SECRET", json.dumps(result))
                cursor = result["continuation"]
                if cursor is None:
                    break
            self.assertTrue(first["truncated"])
            self.assertFalse(first["discovery"]["complete"])
            self.assertEqual(result["tool_calls"], {"exec_command": 7})
            self.assertFalse(result["truncated"])

    def test_refresh_reuses_unchanged_reads_and_invalidates_changes_deletions(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = write(
                root / "one.jsonl",
                meta("one"),
                record("function_call", call_id="old", name="old_tool"),
            )
            first = history.inspect(root, Path("/checkout"), now=NOW)
            unchanged = history.inspect(
                root, Path("/checkout"), now=NOW, continuation=first["checkpoint"]
            )
            self.assertEqual(unchanged["read"]["records_this_page"], 0)
            write(
                path,
                meta("one"),
                record("function_call", call_id="new", name="new_tool"),
            )
            changed = history.inspect(
                root, Path("/checkout"), now=NOW, continuation=unchanged["checkpoint"]
            )
            self.assertEqual(changed["tool_calls"], {"new_tool": 1})
            path.unlink()
            deleted = history.inspect(
                root, Path("/checkout"), now=NOW, continuation=changed["checkpoint"]
            )
            self.assertEqual(deleted["sources"], [])
            self.assertEqual(deleted["tool_calls"], {})

    def test_oversized_record_does_not_hide_later_activity_and_is_a_coverage_gap(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write(
                root / "one.jsonl",
                meta("one"),
                record("message", content="SECRET" * 1000),
                record("function_call", call_id="after", name="after_tool"),
            )
            result = history.inspect(
                root, Path("/checkout"), now=NOW, max_record_bytes=512, max_bytes=16384
            )
            self.assertEqual(result["tool_calls"], {"after_tool": 1})
            self.assertEqual(result["oversized_or_partial_records"], 1)
            self.assertTrue(result["truncated"])
            self.assertTrue(result["read"]["complete"])
            self.assertIn("oversized_or_partial_record", result["discovery"]["gaps"])
            self.assertNotIn("SECRET", json.dumps(result))

    def test_selected_rationale_has_exact_attribution_and_explicit_excerpt(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = write(
                root / "2020/01/01/rationale.jsonl",
                meta("rationale-session"),
                record(
                    "message",
                    role="assistant",
                    content=[
                        {
                            "type": "output_text",
                            "text": "We retain force validation because trust-boundary buffers can be malformed. SECRET",
                        }
                    ],
                ),
            )
            result = history.recall(
                root, Path("/checkout"), "because trust-boundary", now=NOW
            )
            self.assertEqual(len(result["recall_matches"]), 1)
            selected = result["recall_matches"][0]
            self.assertEqual(selected["session_id"], "rationale-session")
            self.assertEqual(selected["timestamp"], STAMP)
            self.assertNotIn("SECRET", json.dumps(result))
            excerpt = history.read_excerpt(root, selected)
            self.assertIn("because trust-boundary", excerpt["text"])
            self.assertEqual(excerpt["path"], str(path))
            self.assertIn("historical evidence", excerpt["authority"])
            path.write_text(path.read_text() + "\n")
            with self.assertRaisesRegex(ValueError, "changed"):
                history.read_excerpt(root, selected)

    def test_missing_roots_partial_records_and_symlinks_are_not_complete_negatives(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = write(root / "partial.jsonl", meta("one"))
            with path.open("ab") as stream:
                stream.write(b'{"timestamp":"partial SECRET')
            (root / "link").symlink_to(root / "partial.jsonl")
            result = history.inspect(root, Path("/checkout"), now=NOW)
            self.assertTrue(result["truncated"])
            self.assertIn("symlink_refused", result["discovery"]["gaps"])
            self.assertIn("oversized_or_partial_record", result["discovery"]["gaps"])
            missing = history.inspect(root / "missing", Path("/checkout"), now=NOW)
            self.assertTrue(missing["truncated"])
            self.assertIn("unavailable_directory", missing["discovery"]["gaps"])

    def test_mid_scan_change_invalidates_observations_until_refresh(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = write(root / "one.jsonl", meta("one"), record("function_call", call_id="before", name="before_tool"))
            first = history.inspect(root, Path("/checkout"), now=NOW, max_records=1)
            self.assertIsNotNone(first["continuation"])
            write(path, meta("one"), record("function_call", call_id="after", name="after_tool"))
            changed = history.inspect(root, Path("/checkout"), now=NOW, continuation=first["continuation"])
            self.assertTrue(changed["truncated"])
            self.assertIn("source_changed_during_scan", changed["discovery"]["gaps"])
            self.assertEqual(changed["tool_calls"], {})
            refreshed = history.inspect(root, Path("/checkout"), now=NOW, continuation=changed["checkpoint"])
            self.assertEqual(refreshed["tool_calls"], {"after_tool": 1})
            self.assertFalse(refreshed["truncated"])

    def test_workspace_metadata_selects_explicit_root_without_cwd_match(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "sessions"
            write(root / "one.jsonl", meta("one", "/other", runtime_workspace_roots=["/checkout"]), record("function_call", call_id="one", name="workspace_tool"))
            result = history.inspect(root, Path("/checkout"), now=NOW)
            self.assertEqual(result["tool_calls"], {"workspace_tool": 1})
            self.assertFalse(result["truncated"])

    def test_continuation_cannot_change_selection_and_excerpt_cannot_escape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write(
                root / "one.jsonl", meta("one"), record("message", content="rationale")
            )
            result = history.recall(root, Path("/checkout"), "rationale", now=NOW)
            with self.assertRaisesRegex(ValueError, "another selection"):
                history.inspect(
                    root, Path("/foreign"), now=NOW, continuation=result["checkpoint"]
                )
            selected = dict(result["recall_matches"][0], relative="../outside.jsonl")
            with self.assertRaisesRegex(ValueError, "escapes"):
                history.read_excerpt(root, selected)


if __name__ == "__main__":
    unittest.main()

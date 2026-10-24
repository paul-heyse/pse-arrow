# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Derived task and design projections preserve current ownership and uncertainty."""

# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import contextlib
import hashlib
import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable
from unittest.mock import patch

from scripts import workspace_context, workspace_sources, workspace_tasks


class WorkspaceTasksTests(unittest.TestCase):
    def test_task_uses_native_state_and_fresh_bytes_not_historical_commit_authority(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs/plans").mkdir(parents=True)
            (root / "docs/lifecycle.toml").write_text("version = 1\n")
            plan = "docs/plans/task.md"
            owner = "docs/plans/coordinator.md"
            (root / plan).write_text("# Task\nNext unresolved consumer\n")
            (root / owner).write_text("# Current coordinator\n")
            (root / "source.rs").write_text("fn current() {}\n")
            row = {
                "document": owner,
                "native_id": "A01",
                "state": "open",
                "status_owner": owner,
            }
            original = workspace_sources.source

            def selected(relative: str, *, root: Path = workspace_sources.ROOT, revision: str | None = None, max_bytes: int = 4 << 20, expected_digest: str | None = None) -> dict:
                result = original(relative, root=root, revision=revision, max_bytes=max_bytes, expected_digest=expected_digest)
                if relative == plan:
                    result["owner"] = owner
                return result

            with (
                patch.object(workspace_sources, "source", side_effect=selected),
                patch.object(
                    workspace_sources,
                    "relationships",
                    return_value={"items": [], "coverage": "dynamic unresolved"},
                ),
                patch.object(workspace_sources, "git", return_value=""),
                patch.object(
                    workspace_tasks.document_lifecycle, "aggregate", return_value=[row]
                ),
                patch.object(
                    workspace_tasks.document_metadata, "load_policy", return_value={}
                ),
                patch.object(
                    workspace_tasks.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [], 0, "oldcommit\t2026-10-01\tPreviously completed\n", ""
                    ),
                ),
            ):
                result = workspace_tasks.task_packet(
                    plan + "#A01", ("source.rs",), root=root
                )
            self.assertEqual(result["native_scope"][0]["state"], "open")
            self.assertEqual(
                result["git_rationale"]["items"][0]["subject"], "Previously completed"
            )
            self.assertIn("not current authorization", result["git_rationale"]["basis"])
            self.assertEqual(
                result["sources"][0]["digest"],
                hashlib.sha256((root / "source.rs").read_bytes()).hexdigest(),
            )
            self.assertNotIn("text", result["sources"][0])
            self.assertTrue(result["population_complete"])
            self.assertEqual(result["omissions"], [])
            self.assertIn("dynamic consumers unresolved", result["unknowns"])
            self.assertIn(
                "database and session history not required or opened",
                result["exclusions"],
            )

    def test_requested_missing_source_is_a_population_gap_not_an_exclusion(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(
                workspace_tasks.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ):
                result = workspace_tasks.task_packet(None, ("missing.rs",), root=root)
            self.assertFalse(result["population_complete"])
            self.assertTrue(
                any(
                    "selected source unavailable" in item
                    for item in result["omissions"]
                )
            )
            self.assertIn("dynamic consumers unresolved", result["unknowns"])
            self.assertEqual(result["sources"][0]["availability"], "unavailable")

    def test_native_rows_refuse_drift_and_capture_external_status_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs/plans").mkdir(parents=True)
            (root / "docs/lifecycle.toml").write_text("version = 1\n")
            plan = "docs/plans/task.md"
            external = "docs/plans/external.md"
            (root / plan).write_text("# Old plan\n")
            (root / external).write_text("# Canonical status owner\n")

            def aggregate(_selected_root: Path, _policy: dict, *, documents: set[str] | None = None, read_source: Callable[[str], bytes] | None = None) -> list[dict]:
                del documents
                if read_source is None:
                    self.fail("Expected the original selected source reader")
                read_source(external)
                (root / plan).write_text("# Changed after native parsing\n")
                return [{"document": plan, "native_id": "A01", "state": "open"}]

            with (
                patch.object(
                    workspace_tasks.document_lifecycle,
                    "aggregate",
                    side_effect=aggregate,
                ),
                patch.object(
                    workspace_tasks.document_metadata, "load_policy", return_value={}
                ),
            ):
                result = workspace_tasks.task_packet(plan + "#A01", (), root=root)
            self.assertEqual(result["native_scope"], [])
            self.assertTrue(
                any("stale rows refused" in item for item in result["omissions"])
            )
            self.assertIn(external, [fact["path"] for fact in result["sources"]])
            self.assertFalse(result["population_complete"])

    def test_alternatives_bind_distinct_sources_and_follow_renamed_optional_target_dependencies(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, extra in [
                ("owner", ""),
                (
                    "consumer",
                    '\n[target.\'cfg(unix)\'.dependencies]\nrenamed = { package = "owner", path = "../owner", optional = true }\n',
                ),
            ]:
                path = root / "crates" / name
                path.mkdir(parents=True)
                (path / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.0.0"\n' + extra
                )
                (path / "lib.rs").write_text("pub fn current() {}\n")
            question = {
                "question": "Where should coordination live?",
                "alternatives": [
                    {
                        "name": "native owner",
                        "proposal": "Extend native owner",
                        "paths": ["crates/owner/lib.rs"],
                        "removals": ["old host coordinator"],
                        "unknowns": ["string-selected callers"],
                    },
                    {
                        "name": "host consumer",
                        "proposal": "Extend explicit host boundary",
                        "paths": ["crates/consumer/lib.rs"],
                        "removals": [],
                        "unknowns": ["cross-language dispatch"],
                    },
                ],
            }
            with patch.object(
                workspace_tasks,
                "task_packet",
                side_effect=lambda _task, paths, **_kwargs: {"selected": paths},
            ):
                result = workspace_tasks.alternatives(question, root=root)
            self.assertEqual(
                result["alternatives"][0]["package_reach"], ["consumer", "owner"]
            )
            self.assertEqual(result["alternatives"][1]["package_reach"], ["consumer"])
            self.assertEqual(
                result["alternatives"][0]["unknowns"], ["string-selected callers"]
            )
            self.assertIn(
                "not implemented", result["alternatives"][0]["proposal_basis"]
            )
            self.assertIn("no architectural score", result["selection"])
            question["alternatives"][0]["paths"] = ["Cargo.toml"]
            with patch.object(workspace_tasks, "task_packet", return_value={}):
                broad = workspace_tasks.alternatives(question, root=root)
            self.assertEqual(
                broad["alternatives"][0]["package_reach"], ["consumer", "owner"]
            )
            with self.assertRaises(ValueError):
                workspace_tasks.alternatives(
                    {**question, "alternatives": question["alternatives"][:1]},
                    root=root,
                )

    def test_explicit_context_operations_do_not_dump_unrequested_recipe_catalogue(
        self,
    ) -> None:
        with (
            patch.object(
                workspace_context.subprocess,
                "check_output",
                side_effect=AssertionError("unrequested subprocess"),
            ),
            patch.object(
                workspace_context.workspace_tasks,
                "task_packet",
                return_value={"sources": [], "omissions": ["dynamic consumer unknown"]},
            ),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            self.assertEqual(workspace_context.main(["--path", "source.rs"]), 0)
        result = json.loads(output.getvalue())
        self.assertEqual(result["items"], [])
        self.assertIn("dynamic consumer unknown", result["task"]["omissions"])

    def test_deleted_only_selection_retains_relationships_git_delta_and_history(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(
                    workspace_tasks.workspace_sources,
                    "relationships",
                    return_value={"items": []},
                ) as links,
                patch.object(
                    workspace_tasks.workspace_sources,
                    "git",
                    return_value="D missing.rs",
                ),
                patch.object(
                    workspace_tasks.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [], 0, "prior\t2026-10-01\tEarlier implementation\n", ""
                    ),
                ) as git,
            ):
                result = workspace_tasks.task_packet(None, ("missing.rs",), root=root)
            self.assertEqual(result["sources"][0]["availability"], "unavailable")
            self.assertEqual(result["dirty_delta"], ["D missing.rs"])
            self.assertEqual(links.call_args.kwargs["exact_paths"], ("missing.rs",))
            self.assertIn(":(literal)missing.rs", git.call_args.args[0])

    def test_line_history_requires_selected_source_and_bounded_span(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "selected.rs").write_text("fn current() {}\n")
            with self.assertRaises(ValueError):
                workspace_tasks.line_history("1,100:selected.rs", root=root)
            with self.assertRaises(ValueError):
                workspace_tasks.line_history("1,2:../outside.rs", root=root)
            with patch.object(
                workspace_tasks.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, "oldcommit\told rationale\n", ""
                ),
            ) as run:
                result = workspace_tasks.line_history("1,1:selected.rs", root=root)
            self.assertTrue(result["available"])
            self.assertIn("-L", run.call_args.args[0])


if __name__ == "__main__":
    unittest.main()

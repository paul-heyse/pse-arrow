# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentic original-owner fixtures exercise detached evidence population meaning."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import evidence_projection as projection
from scripts import (
    evidence_store,
    resource_store,
    service_borrowers,
    test_resources,
    test_run,
)


class EvidenceProjectionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.registry = Path(temporary.name) / "registry"
        selected = patch.object(
            service_borrowers, "registry", return_value=self.registry
        )
        selected.start()
        self.addCleanup(selected.stop)
        self.path = test_resources.invocation(
            "python", [{"name": "suite::one"}, {"name": "suite::two"}]
        )

    def project(self) -> projection.EvidenceSnapshot:
        return projection.invocation_snapshot(str(self.path), registry=self.registry)

    def finish(
        self,
        rows: list[dict[str, str]],
        *,
        code: int = 1,
        errors: list[str] | None = None,
    ) -> None:
        test_run.finish(
            self.path,
            {"exit_code": code, "results": rows, "report_errors": errors or []},
            "runner",
        )

    def test_complete_failure_is_complete_population_not_command_failure(self) -> None:
        self.finish(
            [
                {"nodeid": "suite::one", "status": "failure"},
                {"nodeid": "suite::two", "status": "passed"},
            ]
        )
        result = self.project()
        self.assertEqual(result["coverage"], "complete")
        self.assertEqual(
            [row["status"] for row in result["facts"] if row["kind"] != "context"],
            ["complete_failure", "failure", "pass"],
        )
        self.assertEqual(
            result["expected_members"], [row["member"] for row in result["facts"]]
        )
        self.assertEqual(result, self.project())

    def test_interrupted_command_keeps_missing_subject_explicit(self) -> None:
        self.finish(
            [{"nodeid": "suite::one", "status": "passed"}],
            code=130,
            errors=["interrupted report"],
        )
        result = self.project()
        self.assertEqual(result["coverage"], "partial")
        self.assertEqual(
            [row["status"] for row in result["facts"] if row["kind"] != "context"],
            ["command_failure", "pass", "unreported"],
        )

    def test_open_invocation_remains_open_and_unreported(self) -> None:
        result = self.project()
        self.assertEqual(result["coverage"], "partial")
        self.assertEqual(
            [row["status"] for row in result["facts"] if row["kind"] != "context"],
            ["open", "unreported", "unreported"],
        )

    def test_skip_reuse_transfer_are_explicit_original_row_meanings(self) -> None:
        for status in ("skipped", "reused", "transferred", "not_run", "error"):
            with self.subTest(status=status):
                self.path = test_resources.invocation(
                    "python", [{"name": "suite::one"}]
                )
                self.finish([{"nodeid": "suite::one", "status": status}])
                self.assertEqual(self.project()["facts"][1]["status"], status)

    def test_snapshot_changes_when_original_state_changes(self) -> None:
        original = self.project()
        self.finish(
            [{"nodeid": "suite::one", "status": "passed"}],
            code=7,
            errors=["report absent"],
        )
        current = self.project()
        self.assertEqual(original["source_id"], current["source_id"])
        self.assertEqual(original["authority"], current["authority"])
        self.assertNotEqual(original["snapshot_digest"], current["snapshot_digest"])

    def test_no_initialization_repair_borrow_or_source_mutation(self) -> None:
        original = {
            path: path.read_bytes()
            for path in self.registry.rglob("*")
            if path.is_file()
        }
        with (
            patch.object(
                resource_store.FileStore,
                "ensure",
                side_effect=AssertionError("initialization"),
            ),
            patch.object(
                resource_store.FileStore,
                "freeze",
                side_effect=AssertionError("terminal mutation"),
            ),
            patch.object(
                test_resources, "borrow", side_effect=AssertionError("payload custody")
            ),
        ):
            self.project()
        self.assertEqual(
            original,
            {
                path: path.read_bytes()
                for path in self.registry.rglob("*")
                if path.is_file()
            },
        )

    def test_rust_identity_uses_binary_id_without_invented_crate_mapping(self) -> None:
        self.path = test_resources.invocation(
            "rust", [{"binary_id": "owner-binary", "name": "test_name"}]
        )
        result = self.project()
        self.assertEqual(result["facts"][1]["test_id"], "owner-binary::test_name")
        self.assertNotIn("crate_name", result["facts"][1])

    def test_duplicate_subject_and_tampered_frozen_authority_refuse(self) -> None:
        self.path = test_resources.invocation(
            "python", [{"name": "same"}, {"name": "same"}]
        )
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            self.project()
        self.path = test_resources.invocation("python", [])
        self.finish([])
        path = self.registry / "invocation-state" / self.path.name
        state = resource_store.read(path)
        detail_path = (
            self.registry / "invocation-terminals" / (state["terminal_ref"] + ".json")
        )
        detail = resource_store.read(detail_path)
        detail["terminal"]["complete"] = False
        resource_store.publish(detail_path, detail)
        with self.assertRaisesRegex(ValueError, "Invalid frozen"):
            self.project()

    def test_supplied_only_snapshot_cli_needs_no_state_server_or_extension(
        self,
    ) -> None:
        output = io.StringIO()
        with (
            contextlib.redirect_stdout(output),
            patch.object(
                evidence_store.surreal_server,
                "tooling_activity",
                side_effect=AssertionError("service"),
            ),
            patch.object(
                evidence_store, "run_native", side_effect=AssertionError("native tool")
            ),
        ):
            self.assertEqual(
                evidence_store.main(
                    ["snapshot", str(self.path), "--registry", str(self.registry)]
                ),
                0,
            )
        self.assertEqual(json.loads(output.getvalue()), self.project())


if __name__ == "__main__":
    unittest.main()

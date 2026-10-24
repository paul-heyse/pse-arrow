# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped test adapters retain native selection, handles and terminal ownership."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from pydantic import ValidationError

from scripts import test_invocation, test_resources, test_run
from scripts.capabilities import rust_tests as subject


def inventory() -> dict:
    return {"rust-suites": {"selected": {
        "binary-id": "pse-ids::lib", "binary-path": "/selected/test",
        "testcases": {"control": {"filter-match": {"status": "matches"}}},
    }}}


class ScopedRustSelectionTests(unittest.TestCase):
    def test_native_unit_and_named_integration_selection_share_normalizer(self) -> None:
        with patch.dict(os.environ, {"PSE_NEXTEST_ACTION": "run --no-fail-fast"}):
            request = subject.RustTestSelection(
                package="pse-ids", expression="control", features=["pse-ids/force-validate"],
                default_features=False, profile="local", cargo_profile="test",
            )
            command, normalized = request.prepared()
            self.assertEqual(normalized, test_invocation.rust(command[3:]))
            self.assertEqual(normalized.values("--filter-expr"), ("test(control)",))
            self.assertEqual(normalized.values("--package"), ("pse-ids",))
            self.assertEqual(normalized.nextest_profile, "local")
            self.assertIn("--lib", command)
            self.assertIn("--locked", command)
            integration, normalized = subject.RustTestSelection(
                package="pse-ids", kind="integration", target="public-api", expression="test(control) | test(other)",
            ).prepared()
            self.assertEqual(normalized.values("--test"), ("public-api",))
            self.assertEqual(normalized.values("--filter-expr"), ("test(control) | test(other)",))
            self.assertNotIn("--lib", integration)

    def test_unknown_options_and_inherited_listing_refuse_before_output(self) -> None:
        for values in (
            {"package": "--workspace", "expression": "control"},
            {"package": "pse-ids", "kind": "integration"},
            {"package": "pse-ids", "expression": "control", "target": "wrong"},
            {"package": "pse-ids", "expression": "control", "features": ["--jobs=1"]},
            {"package": "pse-ids", "expression": "control", "extra": ["--unknown"]},
        ):
            with self.subTest(values=values), self.assertRaises(ValidationError):
                subject.RustTestSelection.model_validate(values)
        for action in ("list", "run --unknown-scope-option", "run --no-run"):
            with self.subTest(action=action), patch.dict(os.environ, {"PSE_NEXTEST_ACTION": action}), patch.object(subject.validation, "fresh_output", side_effect=AssertionError("effects before normalization")), self.assertRaises(ValueError):
                subject.execute(Path.cwd(), subject.RustTestRequest(package="pse-ids", expression="control", output="build/new"))

    def test_description_does_not_claim_compiled_catalogue(self) -> None:
        with patch.dict(os.environ, {"PSE_NEXTEST_ACTION": "run"}):
            result = subject.describe(subject.RustTestSelection(package="pse-ids", expression="control"))
        self.assertIsNone(result["catalogue"])
        self.assertFalse(result["compiled_or_enumerated"])
        self.assertTrue(result["package_scope"])


class InvocationCallbackTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def test_actual_allocated_invocation_callback_precedes_terminal_execution(self) -> None:
        seen = []

        def execute(path: Path, *_args: object, **_options: object) -> int:
            self.assertEqual(seen, [path])
            self.assertEqual(test_resources.read_invocation(path)["selected"][0]["name"], "control")
            return 7

        with (
            patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}),
            patch.object(test_resources.host, "inherit", return_value=None),
            patch.object(subject.test_run, "execute_selected", side_effect=execute),
            patch("scripts.native_tests.prepared_rust_inventory", return_value=inventory()),
        ):
            self.assertEqual(test_run.run_rust(
                ["cargo", "nextest", "run", "-p", "pse-ids"], env={},
                inventory=inventory(), effects="native-local", on_invocation=seen.append,
            ), 7)
        self.assertEqual(len(seen), 1)

    def test_borrowed_invocation_callback_returns_original_handle_without_new_owner(self) -> None:
        with patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}), patch.object(test_resources.host, "inherit", return_value=None):
            selected = test_run.rust_selected(inventory())
            inherited = test_resources.invocation("rust", selected)
            seen = []
            with patch("scripts.native_tests.prepared_rust_inventory", return_value=inventory()), patch.object(test_run.subprocess, "call", return_value=9), patch.object(test_run, "execute_selected", side_effect=AssertionError("borrowed terminal reconciled twice")), patch.object(test_resources, "invocation", side_effect=AssertionError("new owner fabricated")):
                self.assertEqual(test_run.run_rust(
                    ["cargo", "nextest", "run", "-p", "pse-ids"],
                    env={test_resources.MARKER: str(inherited)}, inventory=inventory(),
                    effects="native-local", on_invocation=seen.append,
                ), 9)
            self.assertEqual(seen, [inherited])

    def test_canonical_observer_publishes_actual_handle_before_owned_terminal(self) -> None:
        selection = self.root / "selected.json"
        selection.write_text(json.dumps(inventory()))
        seen = []

        def execute(path: Path, *_args: object, **_options: object) -> int:
            self.assertEqual(seen, [path])
            self.assertEqual(test_resources.read_invocation(path)["selected"], test_run.rust_selected(inventory()))
            return 4

        environment = {
            "PSE_NATIVE_PROVENANCE": str(self.root / "native.json"),
            "PSE_NATIVE_SELECTION": str(selection), "RUST_MIN_STACK": "8388608",
            "PSE_SURREAL_STATE": str(self.root / "state"),
        }
        with patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}), patch.object(test_resources.host, "inherit", return_value=None), patch("scripts.native_tests.ordinary_rust_capture", return_value=([], self.root / "worker")), patch("scripts.native_tests.rust_fixture_bindings", return_value={}), patch.object(test_run, "execute_selected", side_effect=execute):
            self.assertEqual(test_run.observer_rust(["cargo", "nextest", "run"], environment, on_invocation=seen.append), 4)
        self.assertEqual(len(seen), 1)

    def test_scoped_adapter_returns_native_catalogue_and_separate_terminal(self) -> None:
        output = self.root / "output"
        invocation = self.root / "native-invocation.json"
        terminal = {"status": "failed", "exit_code": 1, "process": {"owned_drain": "complete"}, "report_errors": []}

        def fresh(_root: Path, path: Path, _label: str) -> Path:
            path.mkdir()
            return path

        def run(command: list[str], **options: object) -> int:
            self.assertIn("--lib", command)
            self.assertTrue(options["package_scope"])
            self.assertEqual(options["effects"], "native-local")
            callback = options["on_invocation"]
            if not callable(callback):
                raise TypeError("native invocation callback must be callable")
            callback(invocation)
            invocation.with_suffix(".terminal.json").write_text(json.dumps(terminal))
            return 1

        with patch.dict(os.environ, {"PSE_NEXTEST_ACTION": "run"}), patch.object(subject.arrow_validation, "rust_test_environment", return_value={}), patch.object(subject.validation, "fresh_output", side_effect=fresh), patch.object(subject.test_run, "run_rust", side_effect=run), patch.object(subject.test_resources, "read_invocation", return_value={"selected": test_run.rust_selected(inventory())}):
            result = subject.execute(self.root, subject.RustTestRequest(package="pse-ids", expression="control", execution_effects="native-local", output=str(output)))
        self.assertEqual(result["exit_code"], 1)
        self.assertEqual(result["invocation"], str(invocation))
        self.assertEqual(result["invocation_ownership"], "owned")
        self.assertEqual(result["native_terminal"], terminal)
        self.assertEqual(json.loads(Path(result["invocation_reference"]).read_text())["invocation"], str(invocation))
        self.assertEqual(result["selected_count"], 1)
        self.assertTrue(result["terminal_observed"])

    def test_scoped_borrowed_handle_never_reuses_prior_terminal_as_new_evidence(self) -> None:
        output = self.root / "output"
        invocation = self.root / "borrowed.json"
        invocation.with_suffix(".terminal.json").write_text('{"status":"passed"}')

        def fresh(_root: Path, path: Path, _label: str) -> Path:
            path.mkdir()
            return path

        def run(_command: list[str], **options: object) -> int:
            callback = options["on_invocation"]
            if not callable(callback):
                raise TypeError("native invocation callback must be callable")
            callback(invocation)
            return 0

        with patch.dict(os.environ, {"PSE_NEXTEST_ACTION": "run"}), patch.object(subject.arrow_validation, "rust_test_environment", return_value={test_resources.MARKER: str(invocation)}), patch.object(subject.validation, "fresh_output", side_effect=fresh), patch.object(subject.test_run, "run_rust", side_effect=run), patch.object(subject.test_resources, "read_invocation", return_value={"selected": []}):
            result = subject.execute(self.root, subject.RustTestRequest(package="pse-ids", expression="control", output=str(output)))
        self.assertEqual(result["invocation"], str(invocation))
        self.assertEqual(result["invocation_ownership"], "borrowed")
        self.assertIsNone(result["terminal"])
        self.assertFalse(result["terminal_observed"])


if __name__ == "__main__":
    unittest.main()

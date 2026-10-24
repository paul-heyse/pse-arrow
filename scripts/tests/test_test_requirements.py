# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Catalogue correspondence and before-effects admission controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import test_requirements as requirements
from scripts import test_resources as resources


def inventory(*names: str) -> dict:
    return {"rust-suites": {"runtime": {
        "binary-id": "pse-runtime::lib", "kind": "lib",
        "testcases": {name: {"filter-match": {"status": "matches"}} for name in names},
    }}}


class RequirementTests(unittest.TestCase):
    def test_context_peer_declares_the_context_it_actually_prepares(self) -> None:
        owner, name = requirements.RUST_PEERS["context-overlap"]
        demand = requirements.rust(owner + "::lib", name)
        self.assertEqual(demand.effects, ("canonical", "native"))
        self.assertIn("context-overlap", demand.roles)

    def test_undeclared_native_fixture_demand_refuses_before_preparation(self) -> None:
        declarations = {"actual::selected": requirements.Requirement(fixtures=("native-fixture",)).document()}
        with self.assertRaisesRegex(ValueError, "Undeclared native demand"):
            requirements.fixture_guard(declarations, "native-fixture", "native")
        with self.assertRaisesRegex(ValueError, "no actual selected consumer"):
            requirements.fixture_guard(declarations, "renamed-fixture", "native")

    def test_lost_required_role_fails_despite_nonempty_selection(self) -> None:
        selected = inventory("workflow::diagnostics::tests::local")
        with self.assertRaisesRegex(ValueError, "Required selected role missing"):
            requirements.rust_catalog(selected, required_roles=("PSE_NATIVE_LANE_EXECUTABLE",), validate_declarations=False)

    def test_stale_exception_checks_discovered_owner_not_only_selected_matches(self) -> None:
        names = [name for owner, name in requirements.RUST_FIXTURES.values() if owner == "pse-runtime"]
        complete = inventory(*names, "other")
        for name in names:
            complete["rust-suites"]["runtime"]["testcases"][name]["filter-match"]["status"] = "mismatch"
        self.assertEqual(len(requirements.rust_catalog(complete)), 1)
        del complete["rust-suites"]["runtime"]["testcases"][names[0]]
        with self.assertRaisesRegex(ValueError, "Unmatched fixture declaration"):
            requirements.rust_catalog(complete)
        # That declaration does not apply to a separate integration binary.
        complete["rust-suites"]["runtime"]["kind"] = "test"
        self.assertEqual(len(requirements.rust_catalog(complete)), 1)

    def test_unavailable_feature_does_not_make_a_declaration_stale(self) -> None:
        self.assertTrue(requirements.rust_catalog(inventory("local"), active_features={"pse-runtime": []}))

    def test_new_canonical_fixture_in_local_scope_refuses_before_service_or_path_access(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            path = resources.invocation("python", [{"name": "new.py::test_effect"}])
            with patch.dict(os.environ, {resources.MARKER: str(path)}), patch("scripts.surreal_server.config_for") as service:
                with self.assertRaisesRegex(resources.ResourceError, "Undeclared canonical demand"):
                    resources.register({"test": "new.py::test_effect", "state": "/does-not-exist", "database": "canonical_test_" + "a" * 32})
                service.assert_not_called()

    def test_actual_fixture_closure_and_owned_peer_declarations_admit_consistently(self) -> None:
        base = requirements.python(("request", "canonical_substrate"))
        self.assertIn("canonical", base.effects)
        with self.assertRaisesRegex(ValueError, "Effect-free"):
            requirements.python(("canonical_substrate",), no_allocation=True)
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            path = resources.invocation("python", [])
            resources.publish_python_requirements(path, {"peer": requirements.Requirement(("canonical",), roles=("peer",)).document()})
            with patch.dict(os.environ, {resources.MARKER: str(path), "PSE_TEST_REQUIRED_ROLES": "peer"}):
                resources.publish_python_collection(path, ["peer"])
                resources.require_effect({"test": "peer"})
                with self.assertRaisesRegex(resources.ResourceError, "not uniquely collected"):
                    resources.require_effect({"test": "renamed-peer"})


if __name__ == "__main__":
    unittest.main()

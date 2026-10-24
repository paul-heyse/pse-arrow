# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Catalogue correspondence and before-effects admission controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
from types import SimpleNamespace

import tomlkit
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts import test_requirements as requirements
from scripts import test_resources as resources


def inventory(*names: str) -> dict:
    return {
        "rust-suites": {
            "runtime": {
                "binary-id": "pse-runtime::lib",
                "kind": "lib",
                "testcases": {
                    name: {"filter-match": {"status": "matches"}} for name in names
                },
            }
        }
    }


class RequirementTests(unittest.TestCase):
    def test_namespace_local_default_and_known_pure_native_contracts(self) -> None:
        for binary, name in (
            ("pse-operations", "new_module::new_case"),
            ("pse-runtime", "workflow::new_module::new_case"),
            ("pse-backend-native", "kinsol::build_tests::native_profile_identity_requires_provider_and_source_evidence"),
            ("pse-backend-native", "ipopt::contract_tests::status_scope_and_finite_infinity_are_not_conflated"),
        ):
            with self.subTest(binary=binary, name=name):
                demand = requirements.rust(binary, name)
                self.assertEqual(demand, requirements.LOCAL)
                self.assertEqual(requirements.scheduling(demand.document()), (None, 1))
                with self.assertRaisesRegex(ValueError, "Undeclared canonical demand"):
                    requirements.guard({name: demand.document()}, name, "canonical")
        native = requirements.rust("pse-backend-native", "kinsol::native_build_tests::admitted_native_profile_has_concrete_adapter_identity")
        self.assertEqual(requirements.scheduling(native.document()), ("scientific", 2))

    def test_isolated_database_assertions_do_not_claim_whole_service_quiet(self) -> None:
        for binary, name in (
            ("pse-operations", "testing::canonical_server_unit::unowned_connections_preserve_selected_fixture_database"),
            ("pse-operations", "canonical::canonical_server_unit::initialization_sixteen_independent_databases_complete_in_parallel"),
            ("pse-operations", "canonical::canonical_server_unit::initialization_cancellation_cannot_establish_success"),
            ("pse-operations", "canonical_execution::canonical_execution_server_unit::canonical_execution_expiry_cancellation_and_truthful_recovery"),
            ("pse-operations", "canonical_selection::canonical_server_unit::blocked_product_roundtrip_shared_roots_and_bounded_frontier_cleanup"),
            ("pse-runtime", "workflow::study_tests::canonical_study_cancellation_after_summary_close_fences_success"),
        ):
            with self.subTest(binary=binary, name=name):
                demand = requirements.rust(binary, name)
                self.assertIn("canonical", demand.effects)
                self.assertIsNone(demand.exclusion)
                expected = ("scientific", 2) if "native" in demand.effects else ("store", 1)
                self.assertEqual(requirements.scheduling(demand.document()), expected)
        owner, name = requirements.RUST_PEERS["external-restart"]
        self.assertEqual(requirements.rust(owner, name).exclusion, "service-generation-maintenance")

    def test_new_python_database_scope_does_not_imply_native_and_collected_owners_agree(self) -> None:
        local = requirements.python(("canonical_substrate",), owner="new_namespace.py")
        self.assertEqual(local.effects, ("canonical",))
        self.assertIsNone(requirements.pytest_group(local.document()))
        item = SimpleNamespace(nodeid="python/pse/tests/test_projects.py::new_durable_case", fixturenames=("canonical_substrate",), get_closest_marker=lambda _name: None)
        native = requirements.python_item(item)
        self.assertEqual(native.effects, ("canonical", "native"))
        self.assertEqual(requirements.pytest_group(native.document()), "scientific-owner")
        item.fixturenames = ("no_canonical_allocation",)
        self.assertEqual(requirements.python_item(item), requirements.LOCAL)

    def test_catalogue_scheduling_binds_exact_binary_names_and_preserves_timeouts(self) -> None:
        catalogue = {"rust-suites": {"lib": {"binary-id": "pse-operations", "testcases": {"testing::canonical_server_unit::unowned_connections_preserve_selected_fixture_database": {"filter-match": {"status": "matches"}}}}}}
        selected = requirements.rust_catalog(catalogue, validate_declarations=False)
        document = tomlkit.parse('[profile.local]\nslow-timeout="60s"\n[[profile.local.overrides]]\nfilter="all()"\nslow-timeout="120s"\n')
        requirements.nextest_configuration(document, "local", catalogue, selected)
        overrides = document["profile"]["local"]["overrides"]
        self.assertIn("binary_id(/^pse\\-operations$/)", overrides[0]["filter"])
        self.assertIn("unowned_connections_preserve_selected_fixture_database", overrides[0]["filter"])
        self.assertEqual(overrides[0]["threads-required"], 1)
        self.assertEqual(overrides[-1]["slow-timeout"], "120s")
        self.assertEqual(document["profile"]["local"]["slow-timeout"], "60s")
        with self.assertRaisesRegex(ValueError, "differ from selected catalogue"):
            requirements.nextest_configuration(document, "local", catalogue, {})

    def test_unsupported_service_quiet_claim_refuses(self) -> None:
        with self.assertRaisesRegex(ValueError, "Unknown test exclusion"):
            requirements.Requirement(exclusion="service-quiet")

    def test_undeclared_durable_runtime_refuses_before_constructor_and_local_runtime_remains_explicit(self) -> None:
        from scripts.tests.test_fixture_settlement import fixture_type

        fixture_class = fixture_type()
        fixture = fixture_class("state", "database", "resource")
        native_owner = fixture_class.runtime.__globals__["pse"]
        with (
            patch.object(native_owner, "Runtime", return_value=Mock()) as constructor,
            patch.object(resources, "require_effect", side_effect=resources.ResourceError("Undeclared native demand")) as demand,
        ):
            with self.assertRaisesRegex(resources.ResourceError, "Undeclared native"):
                fixture.runtime(Mock())
            constructor.assert_not_called()
            demand.assert_called_once()
            demand.reset_mock()
            fixture.runtime(Mock(), ephemeral=True)
            demand.assert_not_called()
            self.assertTrue(constructor.call_args.kwargs["ephemeral"])

    def test_context_peer_declares_the_context_it_actually_prepares(self) -> None:
        owner, name = requirements.RUST_PEERS["context-overlap"]
        demand = requirements.rust(owner + "::lib", name)
        self.assertEqual(demand.effects, ("canonical", "native"))
        self.assertIn("context-overlap", demand.roles)

    def test_undeclared_native_fixture_demand_refuses_before_preparation(self) -> None:
        declarations = {
            "actual::selected": requirements.Requirement(
                fixtures=("native-fixture",)
            ).document()
        }
        with self.assertRaisesRegex(ValueError, "Undeclared native demand"):
            requirements.fixture_guard(declarations, "native-fixture", "native")
        with self.assertRaisesRegex(ValueError, "no actual selected consumer"):
            requirements.fixture_guard(declarations, "renamed-fixture", "native")

    def test_lost_required_role_fails_despite_nonempty_selection(self) -> None:
        selected = inventory("workflow::diagnostics::tests::local")
        with self.assertRaisesRegex(ValueError, "Required selected role missing"):
            requirements.rust_catalog(
                selected,
                required_roles=("PSE_NATIVE_LANE_EXECUTABLE",),
                validate_declarations=False,
            )

    def test_stale_exception_checks_discovered_owner_not_only_selected_matches(
        self,
    ) -> None:
        names = [
            name
            for owner, name in requirements.RUST_FIXTURES.values()
            if owner == "pse-runtime"
        ]
        complete = inventory(*names, "other")
        for name in names:
            complete["rust-suites"]["runtime"]["testcases"][name]["filter-match"][
                "status"
            ] = "mismatch"
        self.assertEqual(len(requirements.rust_catalog(complete)), 1)
        del complete["rust-suites"]["runtime"]["testcases"][names[0]]
        with self.assertRaisesRegex(ValueError, "Unmatched fixture declaration"):
            requirements.rust_catalog(complete)
        # That declaration does not apply to a separate integration binary.
        complete["rust-suites"]["runtime"]["kind"] = "test"
        self.assertEqual(len(requirements.rust_catalog(complete)), 1)

    def test_unavailable_feature_does_not_make_a_declaration_stale(self) -> None:
        self.assertTrue(
            requirements.rust_catalog(
                inventory("local"), active_features={"pse-runtime": []}
            )
        )

    def test_new_canonical_fixture_in_local_scope_refuses_before_service_or_path_access(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")},
            ),
        ):
            path = resources.invocation("python", [{"name": "new.py::test_effect"}])
            with (
                patch.dict(os.environ, {resources.MARKER: str(path)}),
                patch("scripts.surreal_server.config_for") as service,
            ):
                with self.assertRaisesRegex(
                    resources.ResourceError, "Undeclared canonical demand"
                ):
                    resources.register(
                        {
                            "test": "new.py::test_effect",
                            "state": "/does-not-exist",
                            "database": "canonical_test_" + "a" * 32,
                        }
                    )
                service.assert_not_called()

    def test_actual_fixture_closure_and_owned_peer_declarations_admit_consistently(
        self,
    ) -> None:
        base = requirements.python(("request", "canonical_substrate"))
        self.assertIn("canonical", base.effects)
        with self.assertRaisesRegex(ValueError, "Effect-free"):
            requirements.python(("canonical_substrate",), no_allocation=True)
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")},
            ),
        ):
            path = resources.invocation("python", [])
            resources.publish_python_requirements(
                path,
                {
                    "peer": requirements.Requirement(
                        ("canonical",), roles=("peer",)
                    ).document()
                },
            )
            with patch.dict(
                os.environ,
                {resources.MARKER: str(path), "PSE_TEST_REQUIRED_ROLES": "peer"},
            ):
                resources.publish_python_collection(path, ["peer"])
                resources.require_effect({"test": "peer"})
                with self.assertRaisesRegex(
                    resources.ResourceError, "not uniquely collected"
                ):
                    resources.require_effect({"test": "renamed-peer"})


if __name__ == "__main__":
    unittest.main()

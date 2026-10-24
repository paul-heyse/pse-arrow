# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentic setup failures preserve assurance and assertion-reach distinctions."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import regression_experiments, run_inspection, test_resources, test_run


class SetupObservationTests(unittest.TestCase):
    def test_peer_join_uses_same_registry_frozen_identity_and_required_role(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            parent = test_resources.invocation("python", [{"name": "peer.py::test_restart"}])
            name = "canonical_execution::canonical_execution_server_unit::retained_canonical_store_recovers_a_new_generation_after_external_server_restart"
            child = test_resources.invocation("rust", [{"binary_id": "pse-operations::lib", "name": name, "executable": "/controlled/peer"}])
            with patch.dict(os.environ, {test_resources.MARKER: str(parent)}):
                with self.assertRaisesRegex(test_resources.ResourceError, "frozen invocation"):
                    test_resources.observe_peer({"test": "peer.py::test_restart"}, child, role="external-restart")
                test_resources.finalize(child, {"pse-operations::lib::" + name: "pass"}, complete=True, terminal_owner="runner", terminal={"assurance": "pass"})
                test_resources.observe_peer({"test": "peer.py::test_restart"}, child, role="external-restart")
                image = test_resources.storage().invocation_snapshot(parent.stem)
                actual = image["peer_invocations"]["peer.py::test_restart"]
                self.assertEqual(actual["invocation"], child.stem)
                self.assertEqual(actual["terminal_identity"], test_resources.storage().invocation_snapshot(child.stem)["terminal_identity"])
                with self.assertRaisesRegex(test_resources.ResourceError, "required role"):
                    test_resources.observe_peer({"test": "peer.py::test_restart"}, child, role="operator-query")

    def test_empty_catalog_is_known_not_run_and_not_uncollected(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            path = test_run.empty_selection({"rust-suites": {}})
            result = run_inspection.product_result(str(path))
            self.assertEqual(result["selected_count"], 0)
            self.assertEqual(result["selection_state"], "empty")
            self.assertEqual(result["refusal"], "empty-selection")
            self.assertEqual(result["assurance"], "not-run")
            self.assertIsNone(result["workload_exit"])

    def test_typed_observation_is_selected_open_and_validated(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            path = test_resources.invocation("python", [{"name": "real.py::test_case"}])
            payload = {"test": "real.py::test_case", "kind": "infrastructure", "stage": "canonical-connect", "code": "runtime::infrastructure", "assertion_reach": "not-reached"}
            with patch.dict(os.environ, {test_resources.MARKER: str(path)}):
                for mutation in ({"code": "inferred-from-text"}, {"test": "outside"}, {"assertion_reach": "assumed"}):
                    with self.subTest(mutation=mutation), self.assertRaises(test_resources.ResourceError):
                        test_resources.observe_setup_failure({**payload, **mutation})
                test_resources.observe_setup_failure(payload)
                image = test_resources.storage().invocation_snapshot(path.stem)
                self.assertEqual(image["setup_failures"][payload["test"]]["code"], "runtime::infrastructure")
                test_resources.finalize(path, {payload["test"]: "failure"}, complete=True, terminal_owner="runner")
                with self.assertRaisesRegex(test_resources.ResourceError, "Frozen"):
                    test_resources.observe_setup_failure(payload)

    def test_composer_preserves_passed_negative_control_and_marks_failing_setup(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            for status, reach in (("passed", "unknown"), ("failure", "unknown"), ("failure", "not-reached")):
                with self.subTest(status=status, reach=reach):
                    path = test_resources.invocation("python", [{"name": "real.py::test_case"}])
                    with patch.dict(os.environ, {test_resources.MARKER: str(path)}):
                        test_resources.observe_setup_failure({"test": "real.py::test_case", "kind": "infrastructure", "stage": "canonical-connect", "code": "runtime::infrastructure", "assertion_reach": reach})
                    check = {"status": "passed" if status == "passed" else "failed", "exit_code": 0 if status == "passed" else 1, "report_errors": [], "results": [{"nodeid": "real.py::test_case", "class": "real", "name": "test_case", "status": status}], "artifacts": {}, "process": {}}
                    test_run.finish(path, check, "runner", deferred=[])
                    result = run_inspection.product_result(str(path))
                    self.assertEqual(result["assurance"], "pass" if status == "passed" else "failure")
                    if status != "passed":
                        self.assertEqual(result["report_rows"][0]["failure_kind"], "infrastructure")
                        self.assertEqual(result["report_rows"][0]["assertion_reach"], reach)
                    else:
                        self.assertNotIn("failure_kind", result["report_rows"][0])

    def test_setup_failure_is_inconclusive_for_regression_even_with_panic_exit(self) -> None:
        result = {"phase": "frozen", "terminal": {"complete": True}, "selected_count": 1, "selected_unreported_count": 0, "process": {}, "workload_exit": 100, "report_rows": [{"class": regression_experiments.BINARY, "name": regression_experiments.TEST_NAME, "status": "failure"}], "predicate_execution": {"failure_type": "test failure with exit code 101"}}
        self.assertEqual(regression_experiments.classify_product(result), 1)
        self.assertEqual(regression_experiments.classify_product({**result, "setup_failures": {"selected": {"kind": "infrastructure"}}}), 128)


if __name__ == "__main__":
    unittest.main()

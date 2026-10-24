# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Setup selection uses the common requirement and outcome owners."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import run_inspection, setup_report, test_requirements, test_resources


class SetupReportTests(unittest.TestCase):
    def test_nonempty_setup_suite_cannot_hide_a_missing_required_role(self) -> None:
        suite = unittest.defaultTestLoader.loadTestsFromName(
            "scripts.tests.test_test_invocation.InvocationTests.test_opaque_arguments_cannot_change_profiles_or_compilation"
        )
        with patch.dict(os.environ, {"PSE_TEST_REQUIRED_ROLES": "missing-peer"}), patch.object(setup_report.test_run, "execute_selected") as launch:
            with self.assertRaisesRegex(ValueError, "Required selected role missing"):
                setup_report.execute(suite, Path("unused"))
        launch.assert_not_called()

    def test_actual_loader_selection_and_class_requirements_feed_managed_composer(self) -> None:
        class Case(unittest.TestCase):
            test_requirements = test_requirements.Requirement(roles=("setup-control",))

            def test_control(self) -> None:
                pass

        suite = unittest.defaultTestLoader.loadTestsFromTestCase(Case)
        selected = setup_report.selection(suite)
        identity = selected[0]["class"] + "::" + selected[0]["name"]
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")}):
            def execute(path, command, environment, report, *, selected):
                declaration = test_resources.read_invocation(path)
                self.assertEqual(declaration["kind"], "unittest")
                self.assertEqual(declaration["selected"], [{"name": identity}])
                state = test_resources.storage().invocation_snapshot(path.stem)
                self.assertEqual(state["requirements"][identity]["roles"], ["setup-control"])
                self.assertEqual(environment[test_resources.MARKER], str(path))
                self.assertIn("--child", command)
                return 1

            with patch.object(setup_report.test_run, "execute_selected", side_effect=execute) as composed:
                self.assertEqual(setup_report.execute(suite, Path(directory)), 1)
            composed.assert_called_once()


if __name__ == "__main__":
    unittest.main()

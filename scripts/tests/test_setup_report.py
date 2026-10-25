# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Setup selection uses the common requirement and outcome owners."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import setup_report, test_requirements, test_resources

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class SetupReportTests(unittest.TestCase):
    def test_main_discovers_sync_and_async_cases_once_with_requirements(self) -> None:
        loader = unittest.TestLoader()
        root = Path(setup_report.__file__).resolve().parents[1]
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(sys, "argv", ["setup-report", directory]),
            patch.object(setup_report.unittest, "TestLoader", return_value=loader),
            patch.object(loader, "discover", wraps=loader.discover) as discover,
            patch.object(setup_report, "execute", return_value=0) as execute,
        ):
            self.assertEqual(setup_report.main(), 0)
        discover.assert_called_once_with(
            str(root / "scripts/tests"), pattern="test_*.py", top_level_dir=str(root)
        )
        suite = execute.call_args.args[0]
        self.assertIsInstance(suite, unittest.TestSuite)
        identities = [
            item["class"] + "::" + item["name"]
            for item in setup_report.selection(suite)
        ]
        synchronous = (
            "scripts.tests.test_setup_report.SetupReportTests::"
            "test_nonempty_setup_suite_cannot_hide_a_missing_required_role"
        )
        asynchronous = (
            "scripts.tests.test_capability_telemetry.StdioTelemetryTests::"
            "test_real_source_only_stdio_reveal_read_and_redaction_without_backend_or_native"
        )
        self.assertEqual(identities.count(synchronous), 1)
        self.assertEqual(identities.count(asynchronous), 1)
        demands = setup_report.requirements(suite)
        self.assertEqual(set(demands), set(identities))
        self.assertEqual(demands[asynchronous], test_requirements.LOCAL.document())

    def test_nonempty_setup_suite_cannot_hide_a_missing_required_role(self) -> None:
        suite = unittest.defaultTestLoader.loadTestsFromName(
            "scripts.tests.test_test_invocation.InvocationTests.test_opaque_arguments_cannot_change_profiles_or_compilation"
        )
        with (
            patch.dict(os.environ, {"PSE_TEST_REQUIRED_ROLES": "missing-peer"}),
            patch.object(setup_report.test_run, "execute_selected") as launch,
            self.assertRaisesRegex(ValueError, "Required selected role missing"),
        ):
            setup_report.execute(suite, Path("unused"))
        launch.assert_not_called()

    def test_actual_loader_selection_and_class_requirements_feed_managed_composer(
        self,
    ) -> None:
        class Case(unittest.TestCase):
            test_requirements = test_requirements.Requirement(roles=("setup-control",))

            def test_control(self) -> None:
                pass

        suite = unittest.defaultTestLoader.loadTestsFromTestCase(Case)
        selected = setup_report.selection(suite)
        identity = selected[0]["class"] + "::" + selected[0]["name"]
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {"PSE_TEST_RESOURCE_STATE": str(Path(directory) / "registry")},
            ),
        ):

            def execute(
                path: Path,
                command: Sequence[str],
                environment: Mapping[str, str],
                _report: Path,
                *,
                selected: list[dict[str, str]],
            ) -> int:
                declaration = test_resources.read_invocation(path)
                self.assertEqual(declaration["kind"], "unittest")
                self.assertEqual(declaration["selected"], [{"name": identity}])
                self.assertEqual(
                    [case["class"] + "::" + case["name"] for case in selected],
                    [identity],
                )
                state = test_resources.storage().invocation_snapshot(path.stem)
                self.assertEqual(
                    state["requirements"][identity]["roles"], ["setup-control"]
                )
                self.assertEqual(environment[test_resources.MARKER], str(path))
                self.assertIn("--child", command)
                return 1

            with patch.object(
                setup_report.test_run, "execute_selected", side_effect=execute
            ) as composed:
                self.assertEqual(setup_report.execute(suite, Path(directory)), 1)
            composed.assert_called_once()


if __name__ == "__main__":
    unittest.main()

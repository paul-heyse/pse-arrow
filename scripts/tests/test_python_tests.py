# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Controls for the ordinary and managed py-test process boundaries."""
# ruff: noqa: PT009, PT027 -- stdlib controls exercise command dispatch

from __future__ import annotations

import os
import signal
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import python_tests, surreal_server


class PythonProcessRoutingTests(unittest.TestCase):
    def test_ordinary_defaults_keep_sixteen_workers_and_original_categories(
        self,
    ) -> None:
        with (
            patch.object(sys, "argv", ["python_tests"]),
            patch.object(python_tests.subprocess, "call", return_value=0) as run,
        ):
            self.assertEqual(python_tests.main(), 0)
        command = run.call_args.args[0]
        self.assertIn(
            ["-n", "16"],
            [command[index : index + 2] for index in range(len(command) - 1)],
        )
        self.assertEqual(
            command[-2:], ["-m", "(unit or component) and not managed_primary"]
        )
        self.assertEqual(run.call_args.kwargs["cwd"], python_tests.ROOT)

    def test_explicit_integration_keeps_selection_inside_ordinary_partition(
        self,
    ) -> None:
        extra = [
            "-m",
            "integration",
            "python/pse/tests/test_studies.py",
            "--junitxml=chosen.xml",
            "-k",
            "flash",
        ]
        with (
            patch.object(sys, "argv", ["python_tests", *extra]),
            patch.object(python_tests.subprocess, "call", return_value=7) as run,
        ):
            self.assertEqual(python_tests.main(), 7)
        command = run.call_args.args[0]
        self.assertEqual(command[-2:], ["-m", "(integration) and not managed_primary"])
        for argument in (extra[2], extra[3], "-k", "flash"):
            self.assertIn(argument, command)

    def test_managed_parent_reexecutes_before_pytest_with_exact_extras(self) -> None:
        extra = [
            "--managed-primary-route",
            "-m",
            "integration",
            "-k",
            "flash",
            "--junitxml=chosen.xml",
        ]
        with (
            patch.object(sys, "argv", ["python_tests", *extra]),
            patch.dict(os.environ, {"PSE_SURREAL_STATE": "/owned/state"}),
            patch.object(surreal_server, "observer", return_value=19) as placed,
            patch.object(python_tests.subprocess, "call") as run,
        ):
            self.assertEqual(python_tests.main(), 19)
        state, command = placed.call_args.args
        self.assertEqual(state, Path("/owned/state"))
        self.assertEqual(
            command,
            [
                sys.executable,
                "-m",
                "scripts.python_tests",
                *extra,
                "--managed-primary-child",
            ],
        )
        run.assert_not_called()

    def test_managed_child_forces_one_observer_without_forcing_a_report(self) -> None:
        extra = [
            "--managed-primary-route",
            "--managed-primary-child",
            "--markexpr=integration",
            "-n",
            "16",
            "python/pse/tests/test_studies.py",
        ]
        with (
            patch.object(sys, "argv", ["python_tests", *extra]),
            patch.object(surreal_server, "observer") as placed,
            patch.object(python_tests.subprocess, "call", return_value=0) as run,
        ):
            self.assertEqual(python_tests.main(), 0)
        command = run.call_args.args[0]
        self.assertEqual(
            command[-4:], ["-m", "(integration) and managed_primary", "-n", "0"]
        )
        self.assertIn(extra[-1], command)
        self.assertFalse(any(argument.startswith("--junitxml") for argument in command))
        self.assertNotIn("--managed-primary-child", command)
        placed.assert_not_called()

    def test_managed_child_requires_the_declared_route(self) -> None:
        with (
            patch.object(sys, "argv", ["python_tests", "--managed-primary-child"]),
            patch.object(python_tests.subprocess, "call") as run,
            self.assertRaises(ValueError),
        ):
            python_tests.main()
        run.assert_not_called()

    def test_managed_route_requires_selected_state(self) -> None:
        with (
            patch.object(sys, "argv", ["python_tests", "--managed-primary-route"]),
            patch.dict(os.environ, {}, clear=True),
            patch.object(surreal_server, "observer") as placed,
            self.assertRaises(ValueError),
        ):
            python_tests.main()
        placed.assert_not_called()

    def test_child_signal_status_keeps_shell_semantics(self) -> None:
        with (
            patch.object(sys, "argv", ["python_tests"]),
            patch.object(python_tests.subprocess, "call", return_value=-signal.SIGTERM),
        ):
            self.assertEqual(python_tests.main(), 128 + signal.SIGTERM)

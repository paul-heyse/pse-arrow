# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Direct selector entry preserves native selection without PYTHONPATH repair."""

# ruff: noqa: PT009

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import select

ROOT = Path(__file__).resolve().parents[2]


class SelectorTests(unittest.TestCase):
    def test_integration_run_uses_existing_runner_and_preserves_target_selection(
        self,
    ) -> None:
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_OPERATION": "owned",
                    "PSE_NEXTEST_ACTION": "run --no-fail-fast",
                },
            ),
            patch.object(select, "resolve") as resolution,
            patch.object(select, "run_rust", return_value=7) as execution,
        ):
            self.assertEqual(
                select.main(
                    [
                        "--features",
                        "pse-runtime/native-solvers",
                        "integration",
                        "pse-runtime",
                        "worker",
                        "-E",
                        "test(caller_exit)",
                    ]
                ),
                7,
            )
        resolution.assert_not_called()
        execution.assert_called_once_with(
            [
                "cargo",
                "nextest",
                "run",
                "--no-fail-fast",
                "-p",
                "pse-runtime",
                "--test",
                "worker",
                "--locked",
                "--features",
                "pse-runtime/native-solvers",
                "-E",
                "test(caller_exit)",
            ],
            effects="canonical",
            package_scope=False,
        )

    def test_integration_listing_never_enters_execution_runner(self) -> None:
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_OPERATION": "owned",
                    "PSE_NEXTEST_ACTION": "list --message-format json",
                },
            ),
            patch.object(select, "resolve") as resolution,
            patch.object(select, "run_rust") as execution,
            patch.object(select.subprocess, "call", return_value=0) as listing,
        ):
            resolution.return_value.command = [
                "cargo",
                "nextest",
                "list",
                "--message-format",
                "json",
                "-p",
                "pse-runtime",
                "--test",
                "worker",
            ]
            self.assertEqual(select.main(["integration", "pse-runtime", "worker"]), 0)
        execution.assert_not_called()
        listing.assert_called_once_with(resolution.return_value.command)

    def test_running_observer_owns_the_single_cargo_resolution(self) -> None:
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_OPERATION": "owned",
                    "PSE_NEXTEST_ACTION": "run --no-fail-fast",
                },
            ),
            patch.object(select, "resolve") as resolution,
            patch.object(select, "run_rust", return_value=7) as execution,
        ):
            self.assertEqual(
                select.main(
                    [
                        "unit",
                        "pse-structural",
                        "a::b",
                        "-p",
                        "pse-relations",
                        "--execution-effects",
                        "native-local",
                    ]
                ),
                7,
            )
        resolution.assert_not_called()
        execution.assert_called_once_with(
            select.unit("pse-structural", "a::b", ["-p", "pse-relations"]),
            effects="native-local",
            package_scope=True,
        )

    def test_direct_and_module_selection_match(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory)
            witness = scratch / "cargo"
            witness.write_text(
                f"#!{sys.executable}\n"
                "import json, os, pathlib, sys\n"
                "args = sys.argv[1:]\n"
                "if args[0] == 'metadata':\n"
                "    print(json.dumps({'packages': [\n"
                "        {'id': 'pse-structural', 'name': 'pse-structural', 'features': {}},\n"
                "        {'id': 'pse-relations', 'name': 'pse-relations', 'features': {'force-validate': []}},\n"
                "        {'id': 'arrow-data', 'name': 'arrow-data', 'features': {'force_validate': []}}]}))\n"
                "elif '--unit-graph' in args:\n"
                "    def unit(name, deps, features):\n"
                "        return {'pkg_id': name, 'target': {'name': name, 'kind': ['lib'], 'src_path': '/' + name}, 'dependencies': [{'index': i} for i in deps], 'features': features, 'platform': None, 'mode': 'test'}\n"
                "    enabled = any('pse-relations/force-validate' in item for item in args)\n"
                "    print(json.dumps({'version': 1, 'roots': [0, 1], 'units': [unit('pse-structural', [], []), unit('pse-relations', [2], []), unit('arrow-data', [], ['force_validate'] if enabled else [])]}))\n"
                "else:\n"
                "    pathlib.Path(os.environ['SELECT_WITNESS']).write_text(json.dumps(args))\n"
            )
            witness.chmod(0o700)
            output = scratch / "argv.json"
            env = dict(os.environ)
            env.pop("PYTHONPATH", None)
            env.update(
                PATH=f"{scratch}:{env['PATH']}",
                PSE_NEXTEST_ACTION="list --message-format json",
                SELECT_WITNESS=str(output),
            )
            arguments = ["unit", "pse-structural", "a::b", "-p", "pse-relations"]
            seen = []
            for entry in [[str(ROOT / "scripts/select.py")], ["-m", "scripts.select"]]:
                subprocess.run(
                    [sys.executable, *entry, *arguments],
                    cwd=ROOT,
                    env=env,
                    check=True,
                    capture_output=True,
                )
                seen.append(json.loads(output.read_text()))
            self.assertEqual(seen[0], seen[1])
            self.assertIn("pse-relations/force-validate", seen[0])
            self.assertIn(
                "(test(a::b)) & (package(pse-relations) | package(pse-structural))",
                seen[0],
            )


if __name__ == "__main__":
    unittest.main()

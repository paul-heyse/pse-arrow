# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pytest owns default versus explicit native Python test selection."""
# ruff: noqa: PT009 -- stdlib controls exercise the installed pytest CLI

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_tests


class PythonSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "pytest.ini").write_text(
            "[pytest]\ntestpaths = python/pse/tests python/pse/parity\n"
        )
        for directory, name in (
            ("tests", "first"),
            ("tests", "second"),
            ("parity", "parity"),
        ):
            target = self.root / f"python/pse/{directory}/test_{name}.py"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"def test_{name}(): pass\n")

    def selected(self, *extra: str) -> str:
        environment = dict(os.environ)
        environment.pop("PYTEST_ADDOPTS", None)
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                *native_tests.PYTHON_DEFAULT_SELECTION,
                "--collect-only",
                "-q",
                *extra,
            ],
            cwd=self.root,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def test_option_values_keep_product_defaults_without_parity(self) -> None:
        selected = self.selected(
            "--log-file",
            "collection.log",
            "--assert",
            "plain",
            "--junitprefix",
            "selection",
        )
        self.assertIn("test_first.py::test_first", selected)
        self.assertIn("test_second.py::test_second", selected)
        self.assertNotIn("test_parity.py", selected)

    def test_explicit_file_directory_and_node_remain_exact(self) -> None:
        for target in (
            "python/pse/tests/test_first.py",
            "python/pse/tests/test_first.py::test_first",
            "python/pse/parity",
        ):
            with self.subTest(target=target):
                selected = self.selected("--ignore", "absent.py", target)
                self.assertNotIn("test_second.py", selected)
                self.assertIn("1 test collected", selected)

    def test_sixteen_workers_overlap_and_controller_inventory_preserves_every_case(
        self,
    ) -> None:
        """An actual parallel run retains collection and terminal identities."""
        (self.root / "conftest.py").write_bytes(
            (native_tests.ROOT / "conftest.py").read_bytes()
        )
        (self.root / "pytest.ini").write_text(
            "[pytest]\nmarkers = unit: fixture control\n"
        )
        arrivals = self.root / "arrivals"
        arrivals.mkdir()
        (self.root / "test_parallel.py").write_text(
            "import os, time\nfrom pathlib import Path\nimport pytest\n"
            "@pytest.mark.unit\n@pytest.mark.parametrize('index', range(16))\n"
            "def test_worker(index):\n"
            "    arrivals = Path(__file__).parent / 'arrivals'\n"
            "    (arrivals / str(index)).write_text(str(os.getpid()))\n"
            "    deadline = time.monotonic() + 30\n"
            "    while len(list(arrivals.iterdir())) != 16:\n"
            "        assert time.monotonic() < deadline, 'sixteen workers did not overlap'\n"
            "        time.sleep(0.01)\n"
        )
        selected = self.root / "selected.txt"
        report = self.root / "parallel.xml"
        environment = dict(os.environ)
        environment.pop("PYTEST_ADDOPTS", None)
        environment["PSE_TEST_ENUMERATION"] = str(selected)
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                "test_parallel.py",
                "-n",
                "16",
                "--dist=worksteal",
                "--max-worker-restart=0",
                f"--junitxml={report}",
                "-q",
            ],
            cwd=self.root,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
            timeout=90,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        identities = selected.read_text().splitlines()
        self.assertEqual(len(identities), 16)
        self.assertEqual(len(set(identities)), 16)
        self.assertEqual(len({path.read_text() for path in arrivals.iterdir()}), 16)
        # The same exact identifiers travel through worker reports into JUnit.
        import xml.etree.ElementTree as ET  # noqa: PLC0415 -- only this control reads XML

        properties = ET.parse(report).findall(".//property[@name='nodeid']")
        self.assertEqual({item.attrib["value"] for item in properties}, set(identities))


class ArtifactObservationTests(unittest.TestCase):
    def test_control_observations_follow_actual_import_and_worker_replacement(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            worker = root / "worker"
            worker.write_bytes(b"actual worker")
            python = root / "_native.so"
            python.write_bytes(b"actual imported extension")
            output = root / "observed.json"
            environment = {
                "PSE_WORKER_BINARY": str(worker),
                "PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS": str(output),
            }
            with patch.object(
                native_tests, "python_native_binary", return_value=python
            ):
                native_tests.observe_deployed_artifacts(environment)
                first = json.loads(output.read_text())
                worker.write_bytes(b"same-path replacement")
                native_tests.observe_deployed_artifacts(environment)
                second = json.loads(output.read_text())
            self.assertNotEqual(first["worker"]["sha256"], second["worker"]["sha256"])
            self.assertEqual(first["python"], second["python"])


class NativeIgnoredSelectionTests(unittest.TestCase):
    def test_exact_ignored_selection_is_forwarded_to_both_list_and_run(self) -> None:
        name = "math::portable::canonical_deployment_tests::canonical_deployment_actual_receipts_enforce_selected_role"
        extra = ["--profile", "local", "--run-ignored", "all", "-E", f"test(={name})"]
        inventory = {
            "rust-suites": {
                "runtime": {
                    "binary-id": "pse-runtime",
                    "binary-path": "/actual/runtime-control",
                    "testcases": {
                        name: {"ignored": True, "filter-match": {"status": "matches"}}
                    },
                }
            }
        }
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_PROVENANCE": str(Path(directory) / "native.json"),
                    "PSE_NATIVE_SELECTION": str(Path(directory) / "selected.json"),
                },
            ),
            patch.object(
                sys,
                "argv",
                ["native_tests.py", "rust", *extra, "--success-output", "final"],
            ),
            patch.object(
                subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, json.dumps(inventory)),
            ) as listed,
            patch.object(subprocess, "call", return_value=0) as executed,
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(native_tests, "native_provenance", return_value={}),
        ):
            self.assertEqual(native_tests.main(), 0)
        self.assertEqual(
            listed.call_args.args[0],
            native_tests.rust_command("list", ["--message-format", "json", *extra]),
        )
        self.assertEqual(
            executed.call_args.args[0],
            native_tests.rust_command(
                "run", ["--no-fail-fast", *extra, "--success-output", "final"]
            ),
        )


if __name__ == "__main__":
    unittest.main()

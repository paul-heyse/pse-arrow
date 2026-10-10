# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pytest owns default versus explicit native Python test selection."""
# ruff: noqa: PT009, PT027 -- stdlib controls exercise the installed pytest CLI

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_tests, surreal_server, test_run


class ManagedRustSelectionTests(unittest.TestCase):
    def test_explicit_package_does_not_build_the_whole_workspace(self) -> None:
        for arguments in (
            ["-p", "pse-runtime"],
            ["--package=pse-runtime"],
            ["-ppse-runtime"],
        ):
            command = native_tests.rust_command("list", list(arguments), managed=True)
            self.assertNotIn("--workspace", command)
            self.assertEqual(command[-len(arguments) :], arguments)
        self.assertIn("--workspace", native_tests.rust_command("list", []))
        managed = native_tests.rust_command("list", [], managed=True)
        self.assertNotIn("--workspace", managed)
        self.assertIn("pse-runtime", managed)
        self.assertIn("pse-runtime/canonical-tests,pse-runtime/native-solvers", managed)
        self.assertNotIn(
            "xtask/canonical-tests", managed[managed.index("--features") + 1]
        )
        self.assertEqual(
            native_tests.rust_command("list", ["--workspace"], managed=True).count(
                "--workspace"
            ),
            1,
        )
        # A test-binary argument cannot change Cargo package selection.
        self.assertIn(
            "--workspace", native_tests.rust_command("run", ["--", "-p", "test-input"])
        )

    def test_native_opt_ins_follow_selected_package_declarations(self) -> None:
        pure = native_tests.rust_command("list", ["-p", "pse-quantity"])
        self.assertNotIn("--features", pure)
        operations = native_tests.rust_command("list", ["-p", "pse-operations"])
        self.assertEqual(
            operations[operations.index("--features") + 1],
            "pse-operations/canonical-tests",
        )
        runtime = native_tests.rust_command(
            "list", ["-p", "pse-runtime", "--features", "pse-runtime/solver-kinsol"]
        )
        self.assertNotIn("--workspace", runtime)
        self.assertIn("pse-runtime/solver-kinsol", runtime)
        self.assertNotIn("xtask/canonical-tests", " ".join(runtime))

    def setUp(self) -> None:
        composition = patch.object(
            native_tests,
            "correctness_command",
            side_effect=lambda command, _environment: command,
        )
        composition.start()
        self.addCleanup(composition.stop)
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.worker = self.root / "worker"
        self.worker.write_bytes(b"prebuilt actual worker")
        self.worker.chmod(0o700)
        self.binary = self.root / "runtime-unit"
        self.binary.write_bytes(b"prebuilt actual unit binary")
        self.provenance = self.root / "native.json"
        self.selected = self.root / "native-selected.json"
        self.inventory = json.dumps(
            {
                "rust-suites": {
                    "runtime": {
                        "binary-id": "pse-runtime::lib",
                        "binary-name": "pse_runtime",
                        "binary-path": str(self.binary),
                        "testcases": {
                            "managed_primary_control": {
                                "filter-match": {"status": "matches"}
                            },
                        },
                    }
                }
            }
        )

    def parent_inputs(
        self, profile: dict, binaries: list[str], *, environment: dict
    ) -> dict:
        self.assertEqual(environment["PSE_WORKER_BINARY"], str(self.worker))
        files = {}
        for name in binaries:
            with Path(name).open("rb") as stream:
                files[name] = hashlib.file_digest(stream, "sha256").hexdigest()
        return {"profile": profile, "files": files}

    def capture(self, extra: list[str]) -> tuple[list[str], Path, list[list[str]]]:
        responses = [
            subprocess.CompletedProcess([], 0, '{"rust-binaries": {}}'),
            subprocess.CompletedProcess([], 0, '{"workspace_root": "/checkout"}'),
            subprocess.CompletedProcess([], 0, self.inventory),
        ]
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                native_tests, "worker_binary", return_value=self.worker
            ) as worker,
            patch.object(
                native_tests.subprocess, "run", side_effect=responses
            ) as captured,
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(
                native_tests, "native_provenance", side_effect=self.parent_inputs
            ),
        ):
            command, actual = native_tests.managed_rust_capture(
                extra, self.provenance, self.selected
            )
        worker.assert_called_once()
        return command, actual, [call.args[0] for call in captured.call_args_list]

    def test_composed_validation_features_reach_metadata_and_provenance(self) -> None:
        def compose(command: list[str], environment: dict[str, str]) -> list[str]:
            self.assertEqual(environment["PSE_WORKER_BINARY"], str(self.worker))
            return [*command, "--features", "pse-runtime/force-validate"]

        with patch.object(native_tests, "correctness_command", side_effect=compose):
            _, _, commands = self.capture(["-p", "pse-runtime"])
        build, metadata, _ = commands
        self.assertIn("pse-runtime/force-validate", build)
        selected = metadata[metadata.index("--features") + 1].split(",")
        self.assertIn("pse-runtime/force-validate", selected)
        self.assertNotIn("pse-relations/force-validate", selected)
        self.assertEqual(
            json.loads(self.provenance.read_text())["profile"]["features"], selected
        )

    def test_managed_parent_builds_and_enumerates_before_observer_entry(self) -> None:
        execution, worker, _ = self.capture(["--profile", "local"])
        with (
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "rust",
                    "--managed-primary-route",
                    "--profile",
                    "local",
                ],
            ),
            patch.dict(
                os.environ,
                {
                    "PSE_SURREAL_STATE": "/owned/state",
                    "PSE_NATIVE_PROVENANCE": str(self.provenance),
                },
            ),
            patch.object(
                native_tests, "managed_rust_capture", return_value=(execution, worker)
            ) as capture,
            patch.object(
                surreal_server, "reference_state", side_effect=lambda state: state
            ),
            patch.object(surreal_server, "observer", return_value=13) as route,
        ):
            self.assertEqual(native_tests.main(), 13)
        capture.assert_called_once()
        state, command = route.call_args.args
        self.assertEqual(state, Path("/owned/state"))
        self.assertEqual(
            command[:5],
            [
                sys.executable,
                "-m",
                "scripts.native_tests",
                "rust",
                "--managed-primary-child",
            ],
        )
        self.assertIn(f"--managed-primary-worker={worker}", command)
        self.assertEqual(command[6:], execution)

    def test_parent_metadata_retains_build_profile_and_exact_execution_selection(
        self,
    ) -> None:
        extra = [
            "-p",
            "pse-runtime",
            "--lib",
            "--cargo-profile=producer",
            "--profile",
            "local",
            "-E",
            "test(chosen)",
            "--partition",
            "hash:1/2",
            "--test-threads",
            "16",
            "--failure-output",
            "immediate",
        ]
        execution, _, commands = self.capture(extra)
        build, graph, inventory = commands
        self.assertIn("--list-type", build)
        self.assertIn("binaries-only", build)
        for value in ("pse-runtime", "--lib", "--cargo-profile=producer"):
            self.assertIn(value, build)
        self.assertEqual(graph[:2], ["cargo", "metadata"])
        self.assertIn("pse-runtime/canonical-tests,pse-runtime/native-solvers", graph)
        self.assertIn("--binaries-metadata", inventory)
        self.assertIn("--cargo-metadata", inventory)
        self.assertNotIn("--features", inventory)
        for value in ("--workspace", "--lib", "--cargo-profile=producer", "-p"):
            self.assertNotIn(value, execution)
        for value in (
            "local",
            "test(managed_primary_) and ((test(chosen)))",
            "hash:1/2",
            "--test-threads",
            "16",
            "--failure-output",
            "immediate",
        ):
            self.assertIn(value, execution)
        self.assertNotIn("--test-threads", inventory)
        self.assertNotIn("--failure-output", inventory)
        native = json.loads(self.provenance.read_text())
        self.assertEqual(native["profile"]["cargo_profile"], "producer")
        self.assertEqual(
            native_tests.validation_receipts.native_selection(
                self.selected.read_text()
            ),
            [{"class": "pse-runtime::lib", "name": "managed_primary_control"}],
        )

    def test_managed_child_only_runs_retained_metadata_and_prebuilt_worker(
        self,
    ) -> None:
        execution, worker, _ = self.capture(
            ["--profile", "local", "--success-output", "immediate"]
        )
        with (
            patch.dict(os.environ, {"PSE_NATIVE_PROVENANCE": str(self.provenance)}),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "rust",
                    "--managed-primary-child",
                    f"--managed-primary-worker={worker}",
                    *execution,
                ],
            ),
            patch.object(native_tests.subprocess, "run") as build,
            patch.object(native_tests, "worker_binary") as worker_build,
            patch.object(test_run, "run_rust", return_value=7) as run,
        ):
            self.assertEqual(native_tests.main(), 7)
        build.assert_not_called()
        worker_build.assert_not_called()
        command = run.call_args.args[0]
        self.assertEqual(command[:4], ["cargo", "nextest", "run", "--no-fail-fast"])
        self.assertIn("--binaries-metadata", command)
        self.assertIn("--cargo-metadata", command)
        self.assertIn("--success-output", command)
        self.assertNotIn("--workspace", command)
        self.assertNotIn("--features", command)
        self.assertEqual(run.call_args.kwargs["env"]["PSE_WORKER_BINARY"], str(worker))

    def test_changed_parent_artifact_refuses_before_observer_execution(self) -> None:
        execution, worker, _ = self.capture([])
        Path(execution[1]).write_text('{"changed": true}')
        with (
            patch.object(native_tests.subprocess, "call") as run,
            self.assertRaises(ValueError),
        ):
            native_tests.managed_rust_run(
                [f"--managed-primary-worker={worker}", *execution], self.provenance
            )
        run.assert_not_called()

    def test_managed_child_without_parent_metadata_cannot_fall_back_to_build(
        self,
    ) -> None:
        with (
            patch.dict(os.environ, {"PSE_NATIVE_PROVENANCE": str(self.provenance)}),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "rust",
                    "--managed-primary-child",
                    f"--managed-primary-worker={self.worker}",
                ],
            ),
            patch.object(native_tests.subprocess, "run") as build,
            self.assertRaises(ValueError),
        ):
            native_tests.main()
        build.assert_not_called()

    def test_explicit_worker_path_is_reused_without_building_or_overriding(
        self,
    ) -> None:
        with patch.object(native_tests.subprocess, "run") as build:
            actual = native_tests.worker_binary(
                ["--cargo-profile", "producer"], {"PSE_WORKER_BINARY": str(self.worker)}
            )
        self.assertEqual(actual, self.worker)
        build.assert_not_called()

    def test_managed_filters_precede_libtest_separator_and_cargo_options_stay_in_metadata(
        self,
    ) -> None:
        execution, _, commands = self.capture(
            [
                "-Fpse-runtime/native-solvers",
                "--manifest-path",
                "/checkout/Cargo.toml",
                "--config",
                "build.jobs=16",
                "--",
                "chosen",
                "--exact",
            ]
        )
        self.assertLess(
            execution.index("test(managed_primary_)"), execution.index("--")
        )
        self.assertEqual(execution[-3:], ["--", "chosen", "--exact"])
        self.assertIn("--manifest-path", commands[1])
        self.assertIn("--config", commands[1])
        self.assertIn(
            "pse-runtime/native-solvers",
            commands[1][commands[1].index("--features") + 1],
        )
        self.assertNotIn("--manifest-path", execution)

    def test_managed_route_refuses_another_metadata_owner(self) -> None:
        with self.assertRaises(ValueError):
            native_tests.managed_rust_arguments(["--binaries-metadata=other.json"])

    def test_caller_filterset_union_is_intersected_with_managed_partition(self) -> None:
        scoped = native_tests.managed_rust_selection(
            [
                "-Etest(first)",
                "--filterset=test(second)",
                "--filter-expr",
                "test(third)",
                "--ignore-default-filter",
                "--",
                "chosen",
                "--exact",
            ]
        )
        self.assertEqual(
            scoped,
            [
                "--ignore-default-filter",
                "-E",
                "test(managed_primary_) and ((test(first)) or (test(second)) or (test(third)))",
                "--",
                "chosen",
                "--exact",
            ],
        )
        self.assertEqual(scoped.count("-E"), 1)


class PythonSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "pytest.ini").write_text(
            "[pytest]\ntestpaths = python/pse/tests python/pse/parity\n"
            "markers =\n    unit: fixture control\n    managed_primary: managed observer control\n"
        )
        for directory, name in (
            ("tests", "first"),
            ("tests", "second"),
            ("parity", "parity"),
        ):
            target = self.root / f"python/pse/{directory}/test_{name}.py"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(
                f"import pytest\n@pytest.mark.unit\ndef test_{name}(): pass\n"
            )
        (self.root / "python/pse/tests/test_managed.py").write_text(
            "import pytest\n@pytest.mark.unit\n@pytest.mark.managed_primary\n"
            "def test_managed(): pass\n"
        )

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

    def test_declared_python_budget_partitions_keep_selection_disjoint(self) -> None:
        environment = dict(os.environ)
        environment.pop("PYTEST_ADDOPTS", None)
        selected: dict[bool, str] = {}
        for managed in (False, True):
            command = native_tests.python_command(
                ["--collect-only", "-q", "-n", "0", "-m", "unit"], managed=managed
            )
            result = subprocess.run(
                command,
                cwd=self.root,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            selected[managed] = result.stdout
        self.assertIn("test_first.py::test_first", selected[False])
        self.assertIn("test_second.py::test_second", selected[False])
        self.assertNotIn("test_managed.py::test_managed", selected[False])
        self.assertIn("test_managed.py::test_managed", selected[True])
        self.assertNotIn("test_first.py::test_first", selected[True])
        self.assertNotIn("test_second.py::test_second", selected[True])

    def test_ordinary_default_keeps_four_workers_and_managed_observer_is_single(
        self,
    ) -> None:
        ordinary = native_tests.python_command([])
        self.assertEqual(ordinary[ordinary.index("-n") + 1], "4")
        self.assertEqual(
            ordinary[-2:],
            ["-m", "(unit or component or integration) and not managed_primary"],
        )
        managed = native_tests.python_command(
            ["-n", "16", "--markexpr=integration"], managed=True
        )
        self.assertEqual(
            managed[-4:], ["-m", "(integration) and managed_primary", "-n", "0"]
        )

    def test_four_workers_overlap_and_controller_inventory_preserves_every_case(
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
        environment["PYTHONPATH"] = str(native_tests.ROOT)
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

        properties = ET.parse(report).findall(".//property[@name='nodeid']")  # noqa: S314 -- XML generated by this control
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


class ManagedPythonRouteTests(unittest.TestCase):
    def test_parent_reexecutes_before_loading_extension_and_preserves_receipt_arguments(
        self,
    ) -> None:
        arguments = [
            "--managed-primary-route",
            "--terminal-owner=assessment",
            "--junitxml=/owned/managed/native-python.xml",
            "python/pse/tests/test_studies.py",
        ]
        with (
            patch.object(sys, "argv", ["native_tests.py", "python", *arguments]),
            patch.dict(os.environ, {"PSE_SURREAL_STATE": "/owned/state"}),
            patch.object(
                surreal_server, "reference_state", side_effect=lambda state: state
            ),
            patch.object(surreal_server, "observer", return_value=17) as placed,
            patch.object(
                native_tests, "worker_binary", return_value=Path("/owned/worker")
            ) as built,
            patch.object(
                native_tests,
                "python_native_binary",
                side_effect=AssertionError("parent loaded extension"),
            ),
        ):
            self.assertEqual(native_tests.main(), 17)
        built.assert_called_once_with([], os.environ)
        placed.assert_called_once_with(
            Path("/owned/state"),
            [
                sys.executable,
                "-m",
                "scripts.native_tests",
                "python",
                *arguments,
                "--managed-primary-child",
            ],
            profile="reference",
        )

    def test_managed_child_captures_original_native_and_collection_receipts(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "native-python.xml"
            selected = Path(directory) / "managed-selected.txt"
            binary = Path(directory) / "_native.so"
            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "native_tests.py",
                        "python",
                        "--managed-primary-route",
                        "--managed-primary-child",
                        "--terminal-owner=assessment",
                        f"--junitxml={report}",
                    ],
                ),
                patch.dict(
                    os.environ,
                    {
                        "PSE_NATIVE_PROVENANCE": str(Path(directory) / "native.json"),
                        "PSE_TEST_ENUMERATION": str(selected),
                    },
                    clear=True,
                ),
                patch.object(native_tests, "python_native_binary", return_value=binary),
                patch.object(native_tests, "observe_deployed_artifacts"),
                patch.object(
                    native_tests,
                    "native_provenance",
                    return_value={"files": {str(binary): "actual-digest"}},
                ),
                patch.object(subprocess, "call", return_value=0) as run,
                patch.object(surreal_server, "observer") as placed,
            ):
                self.assertEqual(native_tests.main(), 0)
            placed.assert_not_called()
            command = run.call_args.args[0]
            self.assertEqual(command[-2:], ["-n", "0"])
            self.assertIn(f"--junitxml={report}", command)
            self.assertNotIn("--managed-primary-child", command)
            self.assertNotIn("--managed-primary-route", command)
            environment = run.call_args.kwargs["env"]
            self.assertEqual(environment["PSE_TEST_ENUMERATION"], str(selected))
            self.assertEqual(environment["PSE_NATIVE_EXPECTED_BINARY"], str(binary))
            self.assertEqual(environment["PSE_NATIVE_EXPECTED_SHA256"], "actual-digest")
            self.assertEqual(
                json.loads((Path(directory) / "native.json").read_text())["files"],
                {str(binary): "actual-digest"},
            )

    def test_child_sentinel_cannot_select_an_undeclared_route(self) -> None:
        with (
            patch.object(
                sys, "argv", ["native_tests.py", "python", "--managed-primary-child"]
            ),
            self.assertRaisesRegex(ValueError, "declared route"),
        ):
            native_tests.main()


class NativeIgnoredSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        composition = patch.object(
            native_tests,
            "correctness_command",
            side_effect=lambda command, _environment: command,
        )
        composition.start()
        self.addCleanup(composition.stop)

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
                clear=True,
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
            patch.object(test_run, "run_rust", return_value=0) as executed,
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

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pytest owns default versus explicit native Python test selection."""
# ruff: noqa: PT009, PT027 -- stdlib controls exercise the installed pytest CLI

from __future__ import annotations

import hashlib
import json
import os
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import (
    native_tests,
    scientific_inputs,
    surreal_server,
    test_requirements,
    test_run,
)
from scripts.cargo_tools import selected_identity as authenticated_tool_identity

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence


def scientific_capture_fixture(destination: Path) -> Path:
    """Stand in for Cargo's immutable output, keeping real role validation active."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        json.dumps(
            {
                "schema": "scientific-inputs-v1",
                "files": {},
                "families": {},
                "directories": {},
                "package_headers": [],
            }
        )
    )
    return destination.resolve()


@patch(
    "scripts.cargo_tools.selected_identity",
    new=lambda name: {
        "name": name,
        "version": "0.9.146",
        "path": "/owned/cargo-nextest",
        "sha256": "selected",
    },
)
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
            "correctness_resolution",
            side_effect=lambda command, _environment: self.resolution(command),
        )
        composition.start()
        self.addCleanup(composition.stop)
        source = patch.object(
            native_tests, "rust_source_identity", return_value={"source": "current"}
        )
        source.start()
        self.addCleanup(source.stop)
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.worker = self.root / "worker"
        self.worker.write_bytes(b"prebuilt actual worker")
        self.worker.chmod(0o700)
        self.binary = self.root / "runtime-unit"
        self.binary.write_bytes(b"prebuilt actual unit binary")
        self.binary.chmod(0o700)
        self.scientific: Path | None = None

        def capture_scientific(destination: Path, _environment: dict) -> Path:
            self.scientific = scientific_capture_fixture(destination)
            return self.scientific

        capture_input = patch.object(
            scientific_inputs, "capture", side_effect=capture_scientific
        )
        self.scientific_capture = capture_input.start()
        self.addCleanup(capture_input.stop)
        self.provenance = self.root / "native.json"
        self.selected = self.root / "native-selected.json"
        self.inventory = json.dumps(
            {
                "rust-suites": {
                    "runtime": {
                        "binary-id": "pse-runtime::lib",
                        "binary-name": "pse_runtime",
                        "package-id": "pse-runtime",
                        "kind": "lib",
                        "build-platform": "target",
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

    def resolution(
        self, command: list[str]
    ) -> native_tests.arrow_validation.Resolution:
        graph = native_tests.arrow_validation.Graph(
            1,
            [
                native_tests.arrow_validation.Unit(
                    pkg_id="pse-runtime",
                    target=native_tests.arrow_validation.Target(
                        "pse_runtime", ["lib"], str(self.root / "lib.rs")
                    ),
                    features=[],
                    dependencies=[],
                    platform=None,
                    mode="test",
                )
            ],
            [0],
        )
        return native_tests.arrow_validation.Resolution(
            command,
            ["pse-runtime"],
            graph,
            b'{"workspace_root": "/checkout", "packages": [{"id": "pse-runtime", "name": "pse-runtime"}]}',
            native_tests.arrow_validation.msgspec.json.encode(graph),
        )

    def parent_inputs(
        self, profile: dict, binaries: list[str], *, environment: dict
    ) -> dict:
        self.assertEqual(environment["PSE_WORKER_BINARY"], str(self.worker))
        files = {}
        capsule = environment.get(scientific_inputs.ROLE)
        for name in [*binaries, *([capsule] if capsule else [])]:
            with Path(name).open("rb") as stream:
                files[name] = hashlib.file_digest(stream, "sha256").hexdigest()
        return {"profile": profile, "files": files}

    def capture(
        self, extra: list[str], *, package_scope: bool = False
    ) -> tuple[list[str], Path, list[list[str]]]:
        responses = [
            subprocess.CompletedProcess(
                [],
                0,
                json.dumps(
                    {
                        "rust-binaries": {
                            "pse-runtime::lib": {
                                key: value
                                for key, value in json.loads(self.inventory)[
                                    "rust-suites"
                                ]["runtime"].items()
                                if key != "testcases"
                            }
                        }
                    }
                ),
            ),
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
            if package_scope:
                command, actual = native_tests.ordinary_rust_capture(
                    ["cargo", "nextest", "run", *extra],
                    self.provenance,
                    self.selected,
                    {},
                    package_scope=True,
                )
            else:
                command, actual = native_tests.managed_rust_capture(
                    extra, self.provenance, self.selected
                )
        worker.assert_called_once()
        self.assertIsNotNone(actual)
        assert actual is not None
        return command, actual, [call.args[0] for call in captured.call_args_list]

    def test_ordinary_pure_selection_does_not_prepare_a_native_worker(self) -> None:
        inventory = json.loads(self.inventory)
        suite = inventory["rust-suites"]["runtime"]
        suite["testcases"] = {
            "workflow::result_inspection::tests::field_descriptor_is_the_original_arrow_type_and_metadata": {
                "filter-match": {"status": "matches"}
            }
        }
        responses = [
            subprocess.CompletedProcess([], 0, '{"rust-binaries": {}}'),
            subprocess.CompletedProcess([], 0, json.dumps(inventory)),
        ]
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_tests, "worker_binary") as worker,
            patch.object(native_tests.subprocess, "run", side_effect=responses),
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {str(self.binary): "unit-artifact"}},
            ) as provenance,
        ):
            execution, actual = native_tests.ordinary_rust_capture(
                ["cargo", "nextest", "run", "-p", "pse-runtime", "--lib"],
                self.provenance,
                self.selected,
                {},
            )
        worker.assert_not_called()
        self.assertIsNone(actual)
        self.assertEqual(provenance.call_args.args[1], [str(self.binary)])
        self.assertNotIn(
            "PSE_WORKER_BINARY", provenance.call_args.kwargs["environment"]
        )
        self.assertIn("--binaries-metadata", execution)
        self.assertEqual(
            json.loads(self.selected.read_text()),
            inventory,
            "worker omission must retain the exact selected inventory",
        )

    def test_workerless_canonical_child_authenticates_retained_inputs_and_demand(
        self,
    ) -> None:
        inventory = json.loads(self.inventory)
        suite = inventory["rust-suites"]["runtime"]
        suite["testcases"] = {
            "workflow::result_inspection::canonical_result_inspection_server_unit::stored_result": {
                "filter-match": {"status": "matches"}
            }
        }
        responses = [
            subprocess.CompletedProcess(
                [],
                0,
                json.dumps(
                    {
                        "rust-binaries": {
                            "pse-runtime::lib": {
                                key: value
                                for key, value in suite.items()
                                if key != "testcases"
                            }
                        }
                    }
                ),
            ),
            subprocess.CompletedProcess([], 0, json.dumps(inventory)),
        ]
        stale = {"PSE_WORKER_BINARY": "/stale/unrelated-worker"}
        with (
            patch.dict(os.environ, stale, clear=True),
            patch.object(native_tests, "worker_binary") as built,
            patch.object(native_tests.subprocess, "run", side_effect=responses),
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={
                    "files": {
                        str(self.binary): hashlib.sha256(
                            self.binary.read_bytes()
                        ).hexdigest()
                    }
                },
            ),
        ):
            execution, worker = native_tests.ordinary_rust_capture(
                ["cargo", "nextest", "run", "-p", "pse-runtime", "--lib"],
                self.provenance,
                self.selected,
                stale,
            )
        built.assert_not_called()
        self.assertIsNone(worker)
        native = json.loads(self.provenance.read_text())
        environment = native_tests.arrow_validation.rust_test_environment(
            {
                "PSE_NATIVE_PROVENANCE": str(self.provenance),
                **stale,
            }
        )
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(test_run, "execute_selected", return_value=7) as run,
            patch.object(
                test_run.resources,
                "invocation",
                return_value=self.root / "invocation.json",
            ),
            patch.object(
                test_run,
                "rust_report_command",
                side_effect=lambda _path, command, _env: (
                    command,
                    self.root / "report.xml",
                ),
            ),
            patch.object(
                native_tests,
                "prepared_rust_inventory",
                wraps=native_tests.prepared_rust_inventory,
            ) as verified,
            patch.object(
                native_tests, "observer_execution", return_value={}
            ) as observe,
        ):
            self.assertEqual(
                native_tests.managed_rust_run(
                    execution, self.provenance, require_worker=False
                ),
                7,
            )
            self.assertNotIn("PSE_WORKER_BINARY", run.call_args.args[2])
            self.assertEqual(
                verified.call_count, 1, "one full verifier at the execution boundary"
            )
            self.assertNotIn("PSE_WORKER_BINARY", observe.call_args.args[0])
            observe.reset_mock()
            run.reset_mock()
            for arguments in (
                execution,
                ["--managed-primary-worker", *execution],
                ["--managed-primary-worker=", *execution],
                [
                    f"--managed-primary-worker={self.worker}",
                    f"--managed-primary-worker={self.worker}",
                    *execution,
                ],
            ):
                with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                    native_tests.managed_rust_run(arguments, self.provenance)
            original_binary = self.binary.read_bytes()
            self.binary.write_bytes(original_binary + b"changed")
            with self.assertRaisesRegex(ValueError, "artifacts changed"):
                native_tests.managed_rust_run(
                    execution, self.provenance, require_worker=False
                )
            self.binary.write_bytes(original_binary)
            with (
                patch.object(
                    native_tests,
                    "rust_source_identity",
                    return_value={"source": "changed"},
                ),
                self.assertRaisesRegex(ValueError, "source or configuration changed"),
            ):
                native_tests.managed_rust_run(
                    execution, self.provenance, require_worker=False
                )
            observe.reset_mock()
            # A separately captured native-demand selection cannot use this exception.
            suite["testcases"] = {
                "workflow::durable_tests::native": {
                    "filter-match": {"status": "matches"}
                }
            }
            self.selected.write_text(json.dumps(inventory))
            native["files"][str(self.selected)] = hashlib.sha256(
                self.selected.read_bytes()
            ).hexdigest()
            self.provenance.write_text(json.dumps(native))
            with self.assertRaisesRegex(ValueError, "canonical-only selected demand"):
                native_tests.managed_rust_run(
                    execution, self.provenance, require_worker=False
                )
            observe.assert_not_called()
            run.assert_not_called()

    def test_unit_scope_uses_actual_validated_roots_and_one_prepared_inventory(
        self,
    ) -> None:
        def resolve(
            command: list[str], *, run: Callable[[Sequence[str]], bytes]
        ) -> native_tests.arrow_validation.Resolution:
            self.assertTrue(callable(run))
            resolved = self.resolution(
                [*command, "--features", "pse-runtime/force-validate"]
            )
            resolved.packages = ["pse-relations", "pse-runtime"]
            return resolved

        with patch.object(
            native_tests.arrow_validation, "resolve", side_effect=resolve
        ) as resolution:
            execution, _, commands = self.capture(
                [
                    "-p",
                    "pse-runtime",
                    "-p",
                    "pse-relations",
                    "--lib",
                    "-E",
                    "test(chosen)",
                    "--partition",
                    "hash:1/2",
                ],
                package_scope=True,
            )
        resolution.assert_called_once()
        self.assertEqual(len(commands), 2)
        build, inventory = commands
        self.assertIn("pse-runtime/force-validate", build)
        self.assertEqual(
            json.loads(Path(execution[3]).read_bytes())["workspace_root"], "/checkout"
        )
        scoped = "(test(chosen)) & (package(pse-relations) | package(pse-runtime))"
        self.assertIn(scoped, inventory)
        self.assertIn(scoped, execution)
        self.assertIn("--binaries-metadata", execution)
        self.assertIn("--cargo-metadata", execution)
        self.assertIn("hash:1/2", execution)

    def test_composed_validation_features_reach_metadata_and_provenance(self) -> None:
        def compose(
            command: list[str], environment: dict[str, str]
        ) -> native_tests.arrow_validation.Resolution:
            self.assertNotIn("PSE_WORKER_BINARY", environment)
            return self.resolution(
                [*command, "--features", "pse-runtime/force-validate"]
            )

        with patch.object(native_tests, "correctness_resolution", side_effect=compose):
            _, _, commands = self.capture(["-p", "pse-runtime"])
        build, _ = commands
        self.assertIn("pse-runtime/force-validate", build)
        selected = json.loads(self.provenance.read_text())["profile"]["features"]
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
        build, inventory = commands
        self.assertIn("--list-type", build)
        self.assertIn("binaries-only", build)
        for value in ("pse-runtime", "--lib", "--cargo-profile=producer"):
            self.assertIn(value, build)
        self.assertIn("pse-runtime/canonical-tests,pse-runtime/native-solvers", build)
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
            patch.object(
                native_tests, "observer_execution", return_value={}
            ) as observed,
        ):
            self.assertEqual(native_tests.main(), 7)
        build.assert_not_called()
        worker_build.assert_not_called()
        self.assertEqual(observed.call_args.args[0]["PSE_WORKER_BINARY"], str(worker))
        command = run.call_args.args[0]
        self.assertEqual(command[:4], ["cargo", "nextest", "run", "--no-fail-fast"])
        self.assertIn("--binaries-metadata", command)
        self.assertIn("--cargo-metadata", command)
        self.assertIn("--success-output", command)
        self.assertNotIn("--workspace", command)
        self.assertNotIn("--features", command)
        self.assertEqual(run.call_args.kwargs["env"]["PSE_WORKER_BINARY"], str(worker))
        self.assertEqual(run.call_args.kwargs["inventory"], json.loads(self.inventory))

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

    def prepared_environment(self) -> dict[str, str]:
        return native_tests.arrow_validation.rust_test_environment(
            {
                "PSE_WORKER_BINARY": str(self.worker),
                "PSE_NATIVE_PROVENANCE": str(self.provenance),
                **(
                    {scientific_inputs.ROLE: str(self.scientific)}
                    if self.scientific
                    else {}
                ),
            }
        )

    def test_prepared_environment_tracks_explicit_inputs_across_observer_admission(
        self,
    ) -> None:
        parent = {
            "BUILD_INPUT": "fixed",
            "PSE_HOST_ALLOCATION": "compile-owner",
            "PSE_RESOURCE_CLASS": "compile",
            "PSE_ADMISSION_DEADLINE": "100",
            "PSE_NATIVE_SETUP_PYTHON": "/parent/setup-python",
            "CARGO_BUILD_JOBS": "4",
            "PSE_CARGO_DEFAULT_JOBS": "4",
            "NEXTEST_TEST_THREADS": "4",
            "PSE_NEXTEST_DEFAULT_THREADS": "4",
        }
        observer = {
            **parent,
            "PSE_HOST_ALLOCATION": "observer-owner",
            "PSE_RESOURCE_CLASS": "functional",
            "PSE_ADMISSION_DEADLINE": "200",
            "CARGO_BUILD_JOBS": "2",
            "PSE_CARGO_DEFAULT_JOBS": "2",
            "NEXTEST_TEST_THREADS": "2",
            "PSE_NEXTEST_DEFAULT_THREADS": "2",
        }
        observer.pop("PSE_NATIVE_SETUP_PYTHON")
        identity = native_tests.rust_environment_identity
        self.assertEqual(identity(parent), identity(observer))
        self.assertNotEqual(
            identity(parent), identity({**observer, "CARGO_BUILD_JOBS": "8"})
        )
        self.assertNotEqual(
            identity(parent), identity({**observer, "NEXTEST_TEST_THREADS": "8"})
        )
        self.assertNotEqual(
            identity(parent), identity({**observer, "BUILD_INPUT": "changed"})
        )
        self.assertNotEqual(
            identity(parent), identity({**observer, "NEW_INPUT": "changed"})
        )

    def test_prepared_selection_allows_actual_junit_composition_only(self) -> None:
        for profile in ("default", "local", "flaky-hunt"):
            original = [
                "--profile",
                profile,
                "-E",
                "test(control)",
                "--",
                "--nocapture",
            ]
            composed, _ = test_run.rust_report_command(
                self.root / f"{profile}.json", ["cargo", "nextest", "run", *original]
            )
            self.assertEqual(
                native_tests.rust_selection_identity(original),
                native_tests.rust_selection_identity(composed[3:]),
            )
            self.assertLess(composed.index("--config-file"), composed.index("--"))
            config = Path(composed[composed.index("--config-file") + 1])
            config.write_text(
                config.read_text().replace("retries = 0", "retries = 1", 1)
            )
            self.assertNotEqual(
                native_tests.rust_selection_identity(original),
                native_tests.rust_selection_identity(composed[3:]),
            )

    def test_prepared_inventory_reuses_catalog_and_preserves_exit_and_signal(
        self,
    ) -> None:
        execution, _, _ = self.capture(["--profile", "local"])
        inventory = json.loads(self.inventory)
        for status in (7, -signal.SIGTERM):
            with (
                self.subTest(status=status),
                patch.object(
                    test_run.subprocess,
                    "run",
                    side_effect=AssertionError("duplicate inventory"),
                ),
                patch.object(
                    test_run.resources,
                    "invocation",
                    return_value=self.root / "invocation.json",
                ),
                patch.object(
                    test_run,
                    "rust_report_command",
                    side_effect=lambda _path, command, _environment=None: (
                        command,
                        self.root / "report.xml",
                    ),
                ),
                patch.object(test_run, "execute_selected", return_value=status) as run,
            ):
                self.assertEqual(
                    test_run.run_rust(
                        ["cargo", "nextest", "run", *execution],
                        env=self.prepared_environment(),
                        inventory=inventory,
                    ),
                    status,
                )
            self.assertEqual(
                run.call_args.kwargs["selected"],
                [{"class": "pse-runtime::lib", "name": "managed_primary_control"}],
            )

    def test_prepared_inventory_rejects_changed_source_environment_selection_and_catalog(
        self,
    ) -> None:
        execution, _, _ = self.capture(["--profile", "local", "-E", "test(control)"])
        environment = self.prepared_environment()
        retained = native_tests.prepared_rust_inventory(execution, environment)
        self.assertEqual(retained, json.loads(self.inventory))
        for option, value in (
            ("--profile", "ci"),
            ("--cargo-profile", "release"),
            ("--features", "other/feature"),
            ("--target", "other-target"),
            ("-E", "test(other)"),
        ):
            with (
                self.subTest(option=option),
                self.assertRaisesRegex(
                    ValueError, "selection differs|effective selection"
                ),
            ):
                native_tests.prepared_rust_inventory(
                    [*execution, option, value], environment
                )
        with self.assertRaisesRegex(ValueError, "environment changed"):
            native_tests.prepared_rust_inventory(
                execution, {**environment, "PREVIOUSLY_UNKNOWN_BUILD_INPUT": "changed"}
            )
        with (
            patch.object(
                native_tests, "rust_source_identity", return_value={"source": "changed"}
            ),
            self.assertRaisesRegex(ValueError, "source or configuration changed"),
        ):
            native_tests.prepared_rust_inventory(execution, environment)
        changed = json.loads(self.inventory)
        changed["rust-suites"]["runtime"]["testcases"]["other"] = {
            "filter-match": {"status": "matches"}
        }
        with self.assertRaisesRegex(ValueError, "inventory differs"):
            native_tests.prepared_rust_inventory(
                execution, environment, inventory=changed
            )

    def test_prepared_inventory_rejects_changed_binary_metadata_and_unknown_identity(
        self,
    ) -> None:
        execution, _, _ = self.capture([])
        environment = self.prepared_environment()
        for path in (
            self.binary,
            self.selected,
            Path(execution[1]),
            Path(execution[3]),
        ):
            original = path.read_bytes()
            try:
                path.write_bytes(original + b"changed")
                with (
                    self.subTest(path=path),
                    self.assertRaisesRegex(ValueError, "artifacts changed"),
                ):
                    native_tests.prepared_rust_inventory(execution, environment)
            finally:
                path.write_bytes(original)
        original = json.loads(self.provenance.read_text())
        original["rust_invocation"]["unit_graph"] = {}
        self.provenance.write_text(json.dumps(original))
        with self.assertRaisesRegex(ValueError, "Unknown Rust preparation"):
            native_tests.prepared_rust_inventory(execution, environment)

    def test_prepared_inventory_binds_nextest_and_external_cargo_configuration(
        self,
    ) -> None:
        config = self.root / "nextest.toml"
        config.write_text((native_tests.ROOT / ".config/nextest.toml").read_text())
        cargo = self.root / "cargo.toml"
        cargo.write_text("[build]\njobs = 2\n")
        execution, _, _ = self.capture(
            ["--config-file", str(config), "--config", str(cargo)]
        )
        environment = self.prepared_environment()
        native_tests.prepared_rust_inventory(execution, environment)
        config.write_text(
            config.read_text().replace("fail-fast = false", "fail-fast = true")
        )
        with self.assertRaisesRegex(ValueError, "configuration changed"):
            native_tests.prepared_rust_inventory(execution, environment)
        config.write_text((native_tests.ROOT / ".config/nextest.toml").read_text())
        cargo.write_text("[build]\njobs = 3\n")
        with self.assertRaisesRegex(ValueError, "Cargo configuration changed"):
            native_tests.prepared_rust_inventory(execution, environment)

    def test_real_nextest_executable_bytes_and_version_invalidate_prepared_reuse(
        self,
    ) -> None:
        executable = self.root / "cargo-nextest"
        executable.write_text("#!/bin/sh\necho 'cargo-nextest 0.9.146'\n")
        executable.chmod(0o700)
        execution, _, _ = self.capture([])
        with (
            patch(
                "scripts.cargo_tools.selected_identity", new=authenticated_tool_identity
            ),
            patch("scripts.cargo_tools.shutil.which", return_value=str(executable)),
        ):
            native = json.loads(self.provenance.read_text())
            native["rust_invocation"]["nextest"] = authenticated_tool_identity(
                "cargo-nextest"
            )
            self.provenance.write_text(json.dumps(native))
            environment = self.prepared_environment()
            native_tests.prepared_rust_inventory(execution, environment)
            executable.write_text(
                "#!/bin/sh\n# replaced executable\necho 'cargo-nextest 0.9.146'\n"
            )
            with self.assertRaisesRegex(ValueError, "tool identity changed"):
                native_tests.prepared_rust_inventory(execution, environment)
            executable.write_text("#!/bin/sh\necho 'cargo-nextest 0.9.145'\n")
            with self.assertRaisesRegex(ValueError, "version differs"):
                native_tests.prepared_rust_inventory(execution, environment)

    def test_prepared_inventory_refuses_unsupported_machine_metadata_shapes(
        self,
    ) -> None:
        execution, _, _ = self.capture([])
        original = json.loads(self.provenance.read_text())
        for path, value in (
            (Path(execution[1]), {"rust-binaries": []}),
            (Path(execution[3]), {"packages": {}}),
        ):
            previous = path.read_bytes()
            try:
                path.write_text(json.dumps(value))
                native = json.loads(json.dumps(original))
                native["files"][str(path)] = hashlib.sha256(
                    path.read_bytes()
                ).hexdigest()
                self.provenance.write_text(json.dumps(native))
                with self.assertRaisesRegex(TypeError, "metadata shape"):
                    native_tests.prepared_rust_inventory(
                        execution, self.prepared_environment()
                    )
            finally:
                path.write_bytes(previous)
                self.provenance.write_text(json.dumps(original))

    def test_prepared_inventory_requires_exact_validated_target_association(
        self,
    ) -> None:
        execution, _, _ = self.capture([])
        native = json.loads(self.provenance.read_text())
        native["rust_invocation"]["unit_graph"]["units"][0]["target"]["name"] = (
            "foreign_target"
        )
        self.provenance.write_text(json.dumps(native))
        with self.assertRaisesRegex(ValueError, "outside validated Cargo roots"):
            native_tests.prepared_rust_inventory(execution, self.prepared_environment())

    def test_observer_capture_binds_actual_scope_and_transformed_environment(
        self,
    ) -> None:
        for profile in ("reference", "exclusive-observer"):
            resources = surreal_server.execution_resources(profile)
            execution = resources["execution"]
            assert isinstance(execution, dict)
            environment = {
                "PSE_TEST_EXECUTION_PROFILE": profile,
                "PSE_SURREAL_STATE": str(self.root),
                "PSE_WORKER_BINARY": str(self.worker),
            }
            with (
                patch.object(
                    surreal_server, "config_for", return_value={"resources": resources}
                ),
                patch.object(
                    surreal_server, "execution_resources", return_value=resources
                ),
                patch.object(
                    surreal_server,
                    "effective_limits",
                    return_value=(
                        execution["observer_memory_bytes"],
                        execution["cpu_threads"],
                    ),
                ) as limits,
                patch.object(
                    surreal_server,
                    "placement_observation",
                    return_value={"physical_cpus": [0, 1]},
                ) as placement,
                patch.object(
                    surreal_server, "execution_slice", return_value="admitted.slice"
                ),
                patch.object(
                    surreal_server,
                    "group_for_slice",
                    return_value=Path("/sys/fs/cgroup/admitted.slice"),
                ),
                patch.object(
                    surreal_server, "role_affinity_ready", return_value=True
                ) as affinity,
                patch.object(
                    Path,
                    "read_text",
                    return_value="0::/admitted.slice/observer.scope\n",
                ),
                patch.object(
                    native_tests.validation,
                    "relevant_environment",
                    return_value=environment,
                ),
            ):
                observed = native_tests.observer_execution(environment)
                self.assertEqual(observed["profile"], profile)
                self.assertEqual(observed["environment"], environment)
                self.assertNotIn("PSE_MEMORY_MAX", observed["environment"])
                self.assertEqual(observed["resources"], resources)
                self.assertEqual(
                    observed["memory_max"], execution["observer_memory_bytes"]
                )
                for memory, cpu in (
                    (1 << 30, execution["cpu_threads"]),
                    (execution["observer_memory_bytes"], 1),
                ):
                    limits.return_value = memory, cpu
                    with (
                        self.subTest(profile=profile, memory=memory, cpu=cpu),
                        self.assertRaisesRegex(ValueError, "admitted allocation"),
                    ):
                        native_tests.observer_execution(environment)
                limits.return_value = (
                    execution["observer_memory_bytes"],
                    execution["cpu_threads"],
                )
                affinity.return_value = False
                with self.assertRaisesRegex(ValueError, "admitted allocation"):
                    native_tests.observer_execution(environment)
                affinity.return_value = True
                for invalid in (
                    None,
                    {},
                    {"physical_cpus": "0,1"},
                    {"physical_cpus": [0, "1"]},
                    {"physical_cpus": [False]},
                ):
                    placement.return_value = invalid
                    affinity.reset_mock()
                    with (
                        self.subTest(placement=invalid),
                        self.assertRaisesRegex(ValueError, "admitted allocation"),
                    ):
                        native_tests.observer_execution(environment)
                    affinity.assert_not_called()

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

    def test_canonical_initializer_uses_actual_artifact_and_selected_profile(
        self,
    ) -> None:
        environment = {"CALLER_INPUT": "preserved"}
        original = environment.copy()
        response = subprocess.CompletedProcess(
            [],
            0,
            json.dumps(
                {
                    "reason": "compiler-artifact",
                    "target": {"name": "xtask"},
                    "executable": str(self.worker),
                }
            ),
        )
        with patch.object(
            native_tests.subprocess, "run", return_value=response
        ) as build:
            actual = native_tests.canonical_tools_binary(
                ["--cargo-profile", "producer"], environment
            )
        self.assertEqual(actual, self.worker)
        self.assertFalse((self.root / "target/debug/xtask").exists())
        command = build.call_args.args[0]
        self.assertEqual(command[command.index("--bin") + 1], "xtask")
        self.assertEqual(command[command.index("--features") + 1], "canonical-tools")
        self.assertEqual(command[command.index("--profile") + 1], "producer")
        self.assertIn("--locked", command)
        self.assertEqual(
            command[command.index("--message-format") + 1], "json-render-diagnostics"
        )
        self.assertEqual(environment, original)

    def test_explicit_canonical_initializer_never_falls_back_to_building(self) -> None:
        with patch.object(native_tests.subprocess, "run") as build:
            self.assertEqual(
                native_tests.canonical_tools_binary(
                    [], {"PSE_CANONICAL_TOOLS": str(self.worker)}
                ),
                self.worker,
            )
            for path in (self.root / "missing-custom-initializer", self.binary):
                if path.exists():
                    path.chmod(0o600)
                with (
                    self.subTest(path=path),
                    self.assertRaisesRegex(ValueError, "PSE_CANONICAL_TOOLS"),
                ):
                    native_tests.canonical_tools_binary(
                        [], {"PSE_CANONICAL_TOOLS": str(path)}
                    )
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
        self.assertIn("--manifest-path", commands[0])
        self.assertIn("--config", commands[0])
        self.assertIn(
            "pse-runtime/native-solvers",
            commands[0][commands[0].index("--features") + 1],
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

    def select_fixture_consumer(self, variable: str, *, selected: bool = True) -> None:
        inventory = json.loads(self.inventory)
        inventory["rust-suites"]["runtime"]["testcases"][
            test_requirements.RUST_FIXTURES[variable][1]
        ] = {"filter-match": {"status": "matches" if selected else "mismatch"}}
        self.inventory = json.dumps(inventory)

    def test_native_lane_build_uses_selected_profile_and_actual_cargo_artifact(
        self,
    ) -> None:
        response = subprocess.CompletedProcess(
            [],
            0,
            json.dumps(
                {
                    "reason": "compiler-artifact",
                    "target": {"name": "pse-native-lane"},
                    "executable": str(self.worker),
                }
            ),
        )
        with patch.object(
            native_tests.subprocess, "run", return_value=response
        ) as build:
            actual = native_tests.native_lane_binary(
                ["--cargo-profile", "producer"], {}
            )
        self.assertEqual(actual, self.worker)
        command = build.call_args.args[0]
        self.assertEqual(command[command.index("--bin") + 1], "pse-native-lane")
        self.assertEqual(command[command.index("--features") + 1], "native-solvers")
        self.assertEqual(command[command.index("--profile") + 1], "producer")
        self.assertIn("--locked", command)

    def test_selected_native_lane_is_source_bound_and_refuses_changed_artifact_or_role(
        self,
    ) -> None:
        variable = "PSE_NATIVE_LANE_EXECUTABLE"
        self.select_fixture_consumer(variable)
        with patch.object(
            native_tests, "native_lane_binary", return_value=self.worker
        ) as build:
            execution, _, _ = self.capture(["--cargo-profile", "producer"])
        self.assertEqual(build.call_args.args[0], ["--cargo-profile", "producer"])
        environment = self.prepared_environment()
        bindings = native_tests.rust_fixture_bindings(self.provenance, environment)
        self.assertEqual(
            bindings,
            {variable: str(self.worker)},
        )
        environment.update(bindings)
        native_tests.prepared_rust_inventory(execution, environment)
        with self.assertRaisesRegex(ValueError, "binding differs"):
            native_tests.rust_fixture_bindings(
                self.provenance, {variable: str(self.binary)}
            )
        original = self.worker.read_bytes()
        self.worker.write_bytes(original + b"changed")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            native_tests.rust_fixture_bindings(self.provenance, environment)
        self.worker.write_bytes(original)
        native = json.loads(self.provenance.read_text())
        native["rust_invocation"]["environment"][variable] = "different-role"
        self.provenance.write_text(json.dumps(native))
        with self.assertRaisesRegex(ValueError, "role differs"):
            native_tests.rust_fixture_bindings(self.provenance, environment)

    def select_scientific_consumer(self) -> None:
        inventory = json.loads(self.inventory)
        inventory["rust-suites"]["runtime"]["testcases"][
            "workflow::modeling::forms_tests::authored_gdp"
        ] = {"filter-match": {"status": "matches"}}
        self.inventory = json.dumps(inventory)

    def test_unselected_native_fixtures_keep_only_selected_scientific_input(
        self,
    ) -> None:
        self.select_scientific_consumer()
        self.select_fixture_consumer("PSE_NATIVE_LANE_EXECUTABLE", selected=False)
        self.select_fixture_consumer("PSE_PRODUCER_FIXTURE_RECEIPT", selected=False)
        with (
            patch.object(native_tests, "native_lane_binary") as lane,
            patch.object(native_tests.subprocess, "run") as capture,
        ):
            self.assertEqual(
                native_tests.prepare_rust_fixtures(
                    json.loads(self.inventory), [], self.root / "receipt.json", {}
                ),
                {scientific_inputs.ROLE: str(self.scientific)},
            )
        lane.assert_not_called()
        capture.assert_not_called()
        self.scientific_capture.assert_called_once()

    def test_selected_portable_consumer_runs_real_recipe_or_preserves_explicit_receipt(
        self,
    ) -> None:
        variable = "PSE_PRODUCER_FIXTURE_RECEIPT"
        self.select_fixture_consumer(variable)
        receipt = self.root / "actual-tool-receipt.json"

        def capture(
            command: list[str], **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            self.assertEqual(command, ["just", "producer-fixture", str(receipt)])
            receipt.write_text("actual tool output")
            return subprocess.CompletedProcess(command, 0)

        environment = {}
        with patch.object(native_tests.subprocess, "run", side_effect=capture) as run:
            bindings = native_tests.prepare_rust_fixtures(
                json.loads(self.inventory), [], receipt, environment
            )
        self.assertEqual(
            bindings,
            {variable: str(receipt)},
        )
        self.assertEqual(environment, bindings)
        self.assertEqual(run.call_count, 1)
        with patch.object(native_tests.subprocess, "run") as run:
            self.assertEqual(
                native_tests.prepare_rust_fixtures(
                    json.loads(self.inventory), [], self.root / "unused", environment
                ),
                bindings,
            )
        run.assert_not_called()

    def test_native_local_fixture_binding_reaches_actual_execution_environment(
        self,
    ) -> None:
        variable = "PSE_NATIVE_LANE_EXECUTABLE"
        self.select_fixture_consumer(variable)
        with patch.object(native_tests, "native_lane_binary", return_value=self.worker):
            execution, _, _ = self.capture([])
        environment = self.prepared_environment()
        with (
            patch.object(
                native_tests, "ordinary_rust_capture", return_value=(execution, None)
            ),
            patch.object(test_run.validation_receipts, "verify_native_profile"),
            patch.object(test_run, "run_rust", return_value=7) as run,
        ):
            self.assertEqual(
                test_run.observer_rust(
                    ["cargo", "nextest", "run"], environment, effects="native-local"
                ),
                7,
            )
        self.assertEqual(run.call_args.kwargs["env"][variable], str(self.worker))
        self.assertEqual(run.call_args.kwargs["effects"], "native-local")

    def test_canonical_fixture_binding_is_scoped_through_observer_and_restored(
        self,
    ) -> None:
        variable = "PSE_NATIVE_LANE_EXECUTABLE"
        self.select_fixture_consumer(variable)
        with patch.object(native_tests, "native_lane_binary", return_value=self.worker):
            execution, _, _ = self.capture([])
        environment = {
            **self.prepared_environment(),
            "PSE_SURREAL_STATE": str(self.root),
        }
        report = self.root / "report.xml"

        def observer(*_args: object, **_kwargs: object) -> int:
            self.assertEqual(os.environ[variable], str(self.worker))
            return 7

        def execute(*_args: object, **kwargs: object) -> int:
            launch = kwargs["launch"]
            assert callable(launch)
            return launch({})

        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                native_tests,
                "ordinary_rust_capture",
                return_value=(execution, self.worker),
            ),
            patch.object(
                test_run.resources,
                "invocation",
                return_value=self.root / "invocation.json",
            ),
            patch.object(
                test_run,
                "rust_report_command",
                return_value=(["cargo", "nextest", "run", *execution], report),
            ),
            patch.object(test_run, "execute_selected", side_effect=execute),
            patch.object(surreal_server, "observer", side_effect=observer),
        ):
            self.assertEqual(
                test_run.observer_rust(["cargo", "nextest", "run"], environment), 7
            )
            self.assertNotIn(variable, os.environ)

    def test_selected_receipt_is_hashed_by_real_provenance_and_checked_at_forwarding(
        self,
    ) -> None:
        self.select_scientific_consumer()
        variable = "PSE_PRODUCER_FIXTURE_RECEIPT"
        self.select_fixture_consumer(variable)
        receipt = self.root / "producer-fixture.json"
        receipt.write_bytes(b"immutable fixture capture bytes")
        suite = json.loads(self.inventory)["rust-suites"]["runtime"]
        responses = [
            subprocess.CompletedProcess(
                [],
                0,
                json.dumps(
                    {
                        "rust-binaries": {
                            suite["binary-id"]: {
                                key: value
                                for key, value in suite.items()
                                if key != "testcases"
                            }
                        }
                    }
                ),
            ),
            subprocess.CompletedProcess([], 0, self.inventory),
        ]
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_tests, "worker_binary", return_value=self.worker),
            patch.object(native_tests.subprocess, "run", side_effect=responses),
            patch.object(
                native_tests.subprocess,
                "check_output",
                return_value="observed tool metadata",
            ),
            patch.object(native_tests, "observe_deployed_artifacts"),
        ):
            execution, _ = native_tests.ordinary_rust_capture(
                ["cargo", "nextest", "run"],
                self.provenance,
                self.selected,
                {variable: str(receipt)},
            )
        native = json.loads(self.provenance.read_text())
        self.assertEqual(
            native["files"][str(receipt)],
            hashlib.sha256(receipt.read_bytes()).hexdigest(),
        )
        environment = {**self.prepared_environment(), variable: str(receipt)}
        self.assertEqual(
            native_tests.rust_fixture_bindings(self.provenance, environment),
            {variable: str(receipt), scientific_inputs.ROLE: str(self.scientific)},
        )
        assert self.scientific is not None
        self.assertEqual(
            native["files"][str(self.scientific)],
            hashlib.sha256(self.scientific.read_bytes()).hexdigest(),
        )
        native_tests.prepared_rust_inventory(execution, environment)
        original_capsule = self.scientific.read_bytes()
        self.scientific.write_bytes(original_capsule + b"changed")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            native_tests.rust_fixture_bindings(self.provenance, environment)
        self.scientific.write_bytes(original_capsule)
        receipt.write_bytes(b"same path changed after capture")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            native_tests.rust_fixture_bindings(self.provenance, environment)


class PythonSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        width = patch.object(native_tests.pse_env, "default_test_width", return_value=4)
        width.start()
        self.addCleanup(width.stop)
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

    def test_ordinary_default_uses_admitted_width_and_managed_observer_is_single(
        self,
    ) -> None:
        with patch.object(native_tests.pse_env, "default_test_width", return_value=3):
            ordinary = native_tests.python_command([])
        self.assertEqual(ordinary[ordinary.index("-n") + 1], "3")
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

    def test_explicit_workers_overlap_and_controller_inventory_preserves_every_case(
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
                "--basetemp",
                str(self.root / "pytest-temp"),
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
            observation={},
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
                patch.object(
                    native_tests.python_extension,
                    "capture",
                    return_value={
                        "binary": str(binary),
                        "sha256": "actual-digest",
                        "features": ["force-validate", "native-solvers"],
                    },
                ),
                patch.object(native_tests, "observe_deployed_artifacts"),
                patch.object(
                    scientific_inputs,
                    "capture",
                    side_effect=lambda _destination, _environment: (
                        scientific_capture_fixture(Path(directory) / "scientific.json")
                    ),
                ) as scientific,
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
            scientific.assert_called_once()
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


@patch(
    "scripts.cargo_tools.selected_identity",
    new=lambda name: {
        "name": name,
        "version": "0.9.146",
        "path": "/owned/cargo-nextest",
        "sha256": "selected",
    },
)
class NativeIgnoredSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        composition = patch.object(
            native_tests,
            "correctness_resolution",
            side_effect=lambda command, _environment: (
                native_tests.arrow_validation.Resolution(
                    command,
                    ["pse-runtime"],
                    native_tests.arrow_validation.Graph(1, [], []),
                    b'{"workspace_root": "/checkout", "packages": []}',
                    b'{"version": 1, "units": [], "roots": []}',
                )
            ),
        )
        composition.start()
        self.addCleanup(composition.stop)
        source = patch.object(
            native_tests, "rust_source_identity", return_value={"source": "current"}
        )
        source.start()
        self.addCleanup(source.stop)

    def test_owned_native_route_defers_validation_to_its_preparing_observer(
        self,
    ) -> None:
        extra = ["-p", "pse-runtime", "--run-ignored", "all", "-E", "test(=chosen)"]
        with (
            patch.dict(
                os.environ, {"PSE_NATIVE_OPERATION": "/exact/operation"}, clear=True
            ),
            patch.object(sys, "argv", ["native_tests.py", "rust", *extra]),
            patch.object(
                native_tests,
                "correctness_command",
                side_effect=AssertionError("duplicate resolution"),
            ),
            patch.object(test_run, "run_rust", return_value=7) as executed,
        ):
            self.assertEqual(native_tests.main(), 7)
        executed.assert_called_once_with(
            native_tests.rust_command("run", ["--no-fail-fast", *extra])
        )

    def test_standalone_ignored_selection_reuses_one_prepared_inventory(self) -> None:
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
        responses = [
            subprocess.CompletedProcess([], 0, '{"rust-binaries": {}}'),
            subprocess.CompletedProcess([], 0, json.dumps(inventory)),
        ]
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
            patch.object(subprocess, "run", side_effect=responses) as listed,
            patch.object(
                scientific_inputs,
                "capture",
                side_effect=lambda destination, _environment: (
                    scientific_capture_fixture(destination)
                ),
            ) as scientific,
            patch.object(test_run, "run_rust", return_value=0) as executed,
            patch.object(
                native_tests,
                "worker_binary",
                return_value=Path(directory) / "actual-worker",
            ),
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {}},
            ),
            patch.object(
                native_tests,
                "correctness_resolution",
                wraps=native_tests.correctness_resolution,
            ) as resolved,
        ):
            self.assertEqual(native_tests.main(), 0)
        resolved.assert_called_once()
        scientific.assert_not_called()
        self.assertEqual(
            listed.call_count,
            2,
            "one build and one inventory reuse resolution metadata",
        )
        selected = listed.call_args_list[1].args[0]
        self.assertIn("--binaries-metadata", selected)
        self.assertIn("--cargo-metadata", selected)
        self.assertIn("--run-ignored", selected)
        self.assertIn(f"test(={name})", selected)
        self.assertNotIn("--success-output", selected)
        run = executed.call_args.args[0]
        self.assertEqual(run[:3], ["cargo", "nextest", "run"])
        self.assertIn("--binaries-metadata", run)
        self.assertIn("--cargo-metadata", run)
        self.assertIn("--run-ignored", run)
        self.assertIn(f"test(={name})", run)
        self.assertIn("--success-output", run)
        self.assertEqual(executed.call_args.kwargs["inventory"], inventory)


if __name__ == "__main__":
    unittest.main()

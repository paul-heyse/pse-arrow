# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded controls for prebuilt observer routing and terminal ownership."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_tests, python_tests, surreal_server, test_resources, test_run


class Plan30RoutingTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.worker = self.root / "worker"
        self.worker.write_bytes(b"retained receiver")
        self.worker.chmod(0o700)
        self.binary = self.root / "unit-binary"
        self.binary.write_bytes(b"retained tests")
        self.provenance = self.root / "native.json"
        self.selection = self.root / "selected.json"
        self.invocation = self.root / "invocation.json"
        self.inventory = {
            "rust-suites": {
                "unit": {
                    "binary-id": "pse-runtime::lib",
                    "binary-name": "pse_runtime",
                    "binary-path": str(self.binary),
                    "testcases": {
                        "chosen": {"filter-match": {"status": "matches"}},
                    },
                }
            }
        }

    def capture(self, extra: list[str]) -> tuple[list[str], list[list[str]]]:
        results = [
            subprocess.CompletedProcess([], 0, '{"rust-binaries": {}}'),
            subprocess.CompletedProcess([], 0, '{"workspace_root": "/checkout"}'),
            subprocess.CompletedProcess([], 0, json.dumps(self.inventory)),
        ]

        def provenance(
            profile: dict[str, object],
            binaries: list[str],
            *,
            environment: dict[str, str],
        ) -> dict[str, object]:
            self.assertEqual(environment["PSE_WORKER_BINARY"], str(self.worker))
            return {
                "profile": profile,
                "files": {
                    name: hashlib.sha256(Path(name).read_bytes()).hexdigest()
                    for name in binaries
                },
            }

        with (
            patch.object(native_tests, "worker_binary", return_value=self.worker),
            patch.object(native_tests.subprocess, "run", side_effect=results) as run,
            patch.object(native_tests, "observe_deployed_artifacts"),
            patch.object(native_tests, "native_provenance", side_effect=provenance),
        ):
            execution, worker = native_tests.ordinary_rust_capture(
                ["cargo", "nextest", "run", *extra],
                self.provenance,
                self.selection,
                {},
            )
        self.assertEqual(worker, self.worker)
        return execution, [call.args[0] for call in run.call_args_list]

    def declaration(self, owner: str = "assessment") -> None:
        self.invocation.write_text(
            json.dumps(
                {
                    "version": 1,
                    "nonce": "a" * 32,
                    "kind": "rust",
                    "selected": test_run.rust_selected(self.inventory),
                    "terminal_owner": owner,
                }
            )
        )
        self.invocation.chmod(0o600)

    def test_ordinary_metadata_never_adds_managed_features(self) -> None:
        execution, metadata = native_tests.managed_rust_arguments(
            [
                "-p",
                "pse-operations",
                "--lib",
                "-F",
                "pse-relations/force-validate",
                "-E",
                "test(chosen)",
            ],
            include_managed_features=False,
        )
        self.assertEqual(metadata, ["--features", "pse-relations/force-validate"])
        self.assertEqual(execution, ["-E", "test(chosen)"])
        _, metadata = native_tests.managed_rust_arguments(
            ["--lib"], include_managed_features=False
        )
        self.assertEqual(metadata, [])

    def test_prebuild_preserves_profile_features_and_selection_before_reuse(
        self,
    ) -> None:
        extra = [
            "-p",
            "pse-runtime",
            "--lib",
            "--cargo-profile=producer",
            "--features",
            "pse-relations/force-validate,caller-feature",
            "--profile",
            "local",
            "-E",
            "test(chosen)",
            "--partition",
            "hash:1/2",
            "--success-output",
            "final",
            "--run-ignored",
            "all",
            "--no-fail-fast",
        ]
        execution, commands = self.capture(extra)
        build, metadata, inventory = commands
        self.assertIn("--cargo-profile=producer", build)
        self.assertIn("pse-runtime", build)
        self.assertIn("caller-feature", metadata[metadata.index("--features") + 1])
        self.assertNotIn(native_tests.MANAGED_FEATURES, metadata)
        self.assertIn("--binaries-metadata", inventory)
        self.assertIn("--cargo-metadata", inventory)
        self.assertNotIn("--success-output", inventory)
        self.assertIn("--run-ignored", inventory)
        self.assertIn("hash:1/2", execution)
        self.assertIn("--success-output", execution)
        self.assertNotIn("--features", execution)
        self.assertNotIn("--cargo-profile=producer", execution)
        native = json.loads(self.provenance.read_text())
        self.assertEqual(
            native["profile"]["features"],
            ["pse-relations/force-validate", "caller-feature"],
        )
        self.assertEqual(native["profile"]["cargo_profile"], "producer")

    def test_observer_receives_selected_invocation_prebuilt_metadata_and_worker(
        self,
    ) -> None:
        execution, _ = self.capture(
            [
                "--features",
                "pse-relations/force-validate",
                "-E",
                "test(chosen)",
                "--no-fail-fast",
            ]
        )
        self.declaration()
        environment = {
            "PSE_NATIVE_OPERATION": "/owned/operation",
            "PSE_SURREAL_STATE": "/owned/state",
            "PSE_NATIVE_PROVENANCE": str(self.provenance),
            "PSE_NATIVE_SELECTION": str(self.selection),
            test_resources.MARKER: str(self.invocation),
        }
        order: list[str] = []

        def prepare(*_args: object) -> tuple[list[str], Path]:
            order.append("prepare")
            return execution, self.worker

        def observer(state: Path, command: list[str], profile: str) -> int:
            order.append("observer")
            self.assertEqual(state, Path("/owned/state"))
            self.assertEqual(profile, "exclusive-observer")
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], str(self.worker))
            self.assertEqual(os.environ[test_resources.MARKER], str(self.invocation))
            self.assertEqual(os.environ["PSE_TEST_EXECUTION_PROFILE"], profile)
            self.assertEqual(command[3], "--observer-child")
            self.assertIn("--binaries-metadata", command)
            self.assertIn("--cargo-metadata", command)
            return 19

        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(native_tests, "ordinary_rust_capture", side_effect=prepare),
            patch.object(surreal_server, "observer", side_effect=observer),
        ):
            self.assertEqual(
                test_run.run_rust(
                    ["cargo", "nextest", "run", "--features", "caller-feature"],
                    env=environment,
                ),
                19,
            )
            self.assertNotIn(test_resources.MARKER, os.environ)
        self.assertEqual(order, ["prepare", "observer"])
        self.assertEqual(
            test_resources.read_invocation(self.invocation)["terminal_owner"],
            "assessment",
        )

    def test_observer_child_verifies_artifacts_without_building(self) -> None:
        execution, _ = self.capture(
            [
                "--features",
                "pse-relations/force-validate",
                "-E",
                "test(chosen)",
                "--no-fail-fast",
            ]
        )
        with (
            patch.dict(
                os.environ,
                {
                    test_run.RUST_OBSERVER: "1",
                    "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
                    "PSE_NATIVE_PROVENANCE": str(self.provenance),
                    "PSE_WORKER_BINARY": str(self.worker),
                },
                clear=True,
            ),
            patch.object(
                native_tests,
                "worker_binary",
                side_effect=AssertionError("child built receiver"),
            ),
            patch.object(
                native_tests.subprocess,
                "run",
                side_effect=AssertionError("child built or queried metadata"),
            ),
            patch.object(test_run, "run_rust", return_value=7) as run,
        ):
            self.assertEqual(test_run.observer_child(execution), 7)
        self.assertIn("--binaries-metadata", run.call_args.args[0])
        self.assertEqual(run.call_args.args[0].count("--no-fail-fast"), 1)
        self.assertEqual(
            run.call_args.kwargs["env"]["PSE_WORKER_BINARY"], str(self.worker)
        )

    def test_assessment_finalization_stays_with_inherited_owner(self) -> None:
        self.declaration()
        with (
            patch.object(test_run.subprocess, "call", return_value=7) as call,
            patch.object(test_run, "finish") as finish,
        ):
            self.assertEqual(
                test_run.run_rust(
                    ["cargo", "nextest", "run"],
                    env={test_resources.MARKER: str(self.invocation)},
                    inventory=self.inventory,
                ),
                7,
            )
        call.assert_called_once()
        finish.assert_not_called()

    def test_nonnative_inventory_preserves_selection_without_run_only_options(
        self,
    ) -> None:
        self.declaration()
        command = [
            "cargo",
            "nextest",
            "run",
            "--features",
            "pse-relations/force-validate",
            "-p",
            "pse-operations",
            "--lib",
            "-E",
            "test(chosen)",
            "--run-ignored",
            "all",
            "--partition",
            "hash:1/2",
            "--success-output",
            "final",
            "--test-threads=2",
            "--no-fail-fast",
        ]
        with (
            patch.object(
                test_run.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, json.dumps(self.inventory)
                ),
            ) as listing,
            patch.object(test_run.subprocess, "call", return_value=0) as execution,
        ):
            self.assertEqual(
                test_run.run_rust(
                    command, env={test_resources.MARKER: str(self.invocation)}
                ),
                0,
            )
        inventory = listing.call_args.args[0]
        self.assertNotIn("--success-output", inventory)
        self.assertNotIn("final", inventory)
        self.assertNotIn("--test-threads=2", inventory)
        self.assertNotIn("--no-fail-fast", inventory)
        self.assertIn("pse-relations/force-validate", inventory)
        self.assertIn("hash:1/2", inventory)
        self.assertIn("--run-ignored", inventory)
        self.assertIn("test(chosen)", inventory)
        self.assertEqual(execution.call_args.args[0], command)

    def test_standalone_observer_finalizes_existing_runner_invocation(self) -> None:
        self.declaration("runner")
        with (
            patch.object(test_run.subprocess, "call", return_value=0),
            patch.object(test_run.validation, "compose_terminal"),
            patch.object(test_run, "finish", return_value=True) as finish,
        ):
            self.assertEqual(
                test_run.run_rust(
                    ["cargo", "nextest", "run"],
                    env={
                        test_resources.MARKER: str(self.invocation),
                        "PSE_RUST_TERMINAL_OWNER": "runner",
                    },
                    inventory=self.inventory,
                ),
                0,
            )
        self.assertEqual(finish.call_args.args[0], self.invocation)
        self.assertEqual(finish.call_args.args[2], "runner")

    def test_pure_python_unit_selection_bypasses_native_observer(self) -> None:
        with (
            patch.object(sys, "argv", ["python_tests", "--unit-only", "-k", "chosen"]),
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_OPERATION": "/owned/operation",
                    "PSE_SURREAL_STATE": "/owned/state",
                },
                clear=True,
            ),
            patch.object(
                surreal_server,
                "observer",
                side_effect=AssertionError("unit entered scientific observer"),
            ),
            patch.object(python_tests, "run_python", return_value=0) as run,
        ):
            self.assertEqual(python_tests.main(), 0)
        self.assertEqual(
            run.call_args.args[0][-2:], ["-m", "(unit) and not managed_primary"]
        )

    def test_native_process_groups_preserve_parallel_pure_tests(self) -> None:
        config = tomllib.loads((native_tests.ROOT / ".config/nextest.toml").read_text())
        self.assertEqual(config["test-groups"]["scientific"]["max-threads"], 2)
        self.assertNotIn("solver", config["test-groups"])
        self.assertNotIn("engineering", config["test-groups"])
        scientific = [
            item
            for item in config["profile"]["default"]["overrides"]
            if item.get("test-group") == "scientific"
        ]
        self.assertEqual(len(scientific), 2)
        self.assertTrue(all(item["threads-required"] == 2 for item in scientific))
        self.assertIn("binary(worker)", scientific[0]["filter"])
        self.assertIn("managed_primary_", scientific[0]["filter"])
        self.assertGreater(config["profile"]["default"]["test-threads"], 2)

    def test_managed_rust_binds_captured_worker_before_reference_admission(
        self,
    ) -> None:
        order = []

        def reference(state: Path) -> Path:
            order.append("reference")
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], str(self.worker))
            self.assertEqual(os.environ["PSE_NATIVE_PROVENANCE"], str(self.provenance))
            return state / "reference"

        def observer(_state: Path, command: list[str], *, profile: str) -> int:
            order.append("observer")
            self.assertEqual(profile, "reference")
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], str(self.worker))
            self.assertIn("--managed-primary-worker=" + str(self.worker), command)
            return 23

        with (
            patch.dict(
                os.environ,
                {
                    "PSE_SURREAL_STATE": "/owned/state",
                    "PSE_NATIVE_PROVENANCE": str(self.provenance),
                    "PSE_WORKER_BINARY": "ambient",
                },
                clear=True,
            ),
            patch.object(
                sys, "argv", ["native_tests", "rust", "--managed-primary-route"]
            ),
            patch.object(
                native_tests,
                "managed_rust_capture",
                return_value=(["-E", "test(chosen)"], self.worker),
            ),
            patch.object(surreal_server, "reference_state", side_effect=reference),
            patch.object(surreal_server, "observer", side_effect=observer),
        ):
            self.assertEqual(native_tests.main(), 23)
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], "ambient")
            self.assertEqual(os.environ["PSE_NATIVE_PROVENANCE"], str(self.provenance))
        self.assertEqual(order, ["reference", "observer"])

    def test_rust_stack_default_override_and_nested_execution_are_recorded(
        self,
    ) -> None:
        default = native_tests.rust_test_environment({})
        self.assertEqual(default["RUST_MIN_STACK"], str(16 << 20))
        self.assertEqual(
            native_tests.validation.relevant_environment(default)["RUST_MIN_STACK"],
            str(16 << 20),
        )
        explicit = native_tests.rust_test_environment({"RUST_MIN_STACK": str(24 << 20)})
        self.assertEqual(explicit["RUST_MIN_STACK"], str(24 << 20))
        self.assertEqual(native_tests.rust_test_environment(explicit), explicit)
        self.declaration()
        with patch.object(test_run.subprocess, "call", return_value=0) as execution:
            self.assertEqual(
                test_run.run_rust(
                    ["cargo", "nextest", "run"],
                    env={test_resources.MARKER: str(self.invocation), **explicit},
                    inventory=self.inventory,
                ),
                0,
            )
        self.assertEqual(
            execution.call_args.kwargs["env"]["RUST_MIN_STACK"], str(24 << 20)
        )
        for value in ("", "0", "-1", "16M", " 16777216", str(1 << 64)):
            with (
                self.subTest(value=value),
                self.assertRaisesRegex(ValueError, "positive finite"),
            ):
                native_tests.rust_test_environment({"RUST_MIN_STACK": value})

    def test_ordinary_native_launchers_supply_completion_once(self) -> None:
        for native_operation in (False, True):
            for caller_completion in ([], ["--no-fail-fast"]):
                with self.subTest(
                    native_operation=native_operation,
                    caller_completion=caller_completion,
                ):
                    environment = {"PSE_NATIVE_PROVENANCE": str(self.provenance)}
                    if native_operation:
                        environment["PSE_NATIVE_OPERATION"] = "/owned/operation"
                    with (
                        patch.dict(os.environ, environment, clear=True),
                        patch.object(
                            sys,
                            "argv",
                            [
                                "native_tests",
                                "rust",
                                "-p",
                                "pse-runtime",
                                "--lib",
                                *caller_completion,
                            ],
                        ),
                        patch.object(
                            native_tests.subprocess,
                            "run",
                            return_value=subprocess.CompletedProcess(
                                [], 0, json.dumps(self.inventory)
                            ),
                        ),
                        patch.object(native_tests, "observe_deployed_artifacts"),
                        patch.object(
                            native_tests,
                            "native_provenance",
                            return_value={"files": {}},
                        ),
                        patch.object(test_run, "run_rust", return_value=17) as run,
                    ):
                        self.assertEqual(native_tests.main(), 17)
                    command = run.call_args.args[0]
                    self.assertEqual(command.count("--no-fail-fast"), 1)
                    self.assertNotIn("--workspace", command)

    def test_explicit_package_selection_supersedes_only_default_workspace(self) -> None:
        for selection in (
            ["-p", "pse-runtime"],
            ["--package=pse-runtime"],
            ["--package", "pse-runtime"],
        ):
            self.assertNotIn(
                "--workspace", native_tests.rust_command("list", selection)
            )
        self.assertEqual(
            native_tests.rust_command(
                "list", ["--workspace", "--exclude", "pse-tests-conformance"]
            ).count("--workspace"),
            1,
        )
        self.assertIn(
            "--workspace",
            native_tests.rust_command("list", ["--exclude", "pse-tests-conformance"]),
        )
        self.assertIn(
            "--workspace", native_tests.rust_command("run", ["--", "-p", "test-input"])
        )

    def test_both_managed_python_launchers_bind_worker_before_admission_and_restore(
        self,
    ) -> None:
        for module, argv in (
            (native_tests, ["native_tests", "python", "--managed-primary-route"]),
            (python_tests, ["python_tests", "--managed-primary-route"]),
        ):
            for ambient in (None, "ambient"):
                with self.subTest(module=module.__name__, ambient=ambient):
                    environment = {"PSE_SURREAL_STATE": "/owned/state"}
                    if ambient is not None:
                        environment["PSE_WORKER_BINARY"] = ambient

                    def reference(_state: Path) -> Path:
                        self.assertEqual(
                            os.environ["PSE_WORKER_BINARY"], str(self.worker)
                        )
                        raise ValueError("admission failure")

                    with (
                        patch.dict(os.environ, environment, clear=True),
                        patch.object(sys, "argv", argv),
                        patch.object(
                            native_tests, "worker_binary", return_value=self.worker
                        ) as select,
                        patch.object(
                            surreal_server, "reference_state", side_effect=reference
                        ),
                        patch.object(surreal_server, "observer") as observer,
                        self.assertRaisesRegex(ValueError, "admission failure"),
                    ):
                        try:
                            module.main()
                        finally:
                            self.assertEqual(
                                os.environ.get("PSE_WORKER_BINARY"), ambient
                            )
                    select.assert_called_once()
                    observer.assert_not_called()


if __name__ == "__main__":
    unittest.main()

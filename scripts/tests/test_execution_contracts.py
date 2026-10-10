# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned effects, terminal ownership and selected functional claim controls."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import nullcontext
from dataclasses import asdict, replace
from pathlib import Path
from unittest.mock import patch

from scripts import (
    case_measure,
    native_tests,
    surreal_server,
    test_resources,
    validation,
    validation_receipts,
)
from scripts.validation_scope import FUNCTIONAL_SCOPES, Gate, comprehensive, native_gate

ROOT = Path(__file__).resolve().parents[2]


class NativePythonImportIdentity(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        self.package = self.output / "foreign/pse"
        self.package.mkdir(parents=True)
        (self.package / "__init__.py").write_text("from . import _native\n")
        (self.package / "_native.py").write_text("__version__ = 'compatible'\n")
        fixture_package = self.package / "tests"
        fixture_package.mkdir()
        (fixture_package / "__init__.py").write_text("")
        (fixture_package / "canonical_fixture.py").write_text(
            "class CanonicalFixture: pass\n"
        )
        self.environment = {
            **os.environ,
            "PYTHONPATH": os.pathsep.join((str(self.package.parent), str(ROOT))),
            "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
        }

    def test_foreign_python_import_is_refused_before_provenance_or_tests(self) -> None:
        report = self.output / "native-python.xml"
        with (
            patch.dict(os.environ, self.environment, clear=True),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "python",
                    "--functional-observer-child",
                    f"--junitxml={report}",
                ],
            ),
            patch.object(native_tests, "native_provenance") as provenance,
            patch.object(native_tests.subprocess, "call") as child,
            self.assertRaisesRegex(ValueError, "different checkout"),
        ):
            native_tests.main()
        provenance.assert_not_called()
        child.assert_not_called()

    def test_resolved_matching_import_is_the_single_recorded_binary(self) -> None:
        checkout = self.output / "checkout"
        expected = checkout / "python/pse"
        expected.mkdir(parents=True)
        (expected / "__init__.py").write_text("from . import _native\n")
        (expected / "_native.py").write_text("__version__ = 'compatible'\n")
        # A stale candidate must not enter provenance merely because it is on disk.
        (expected / "_native.stale.so").write_bytes(b"not imported")
        environment = {**self.environment, "PYTHONPATH": str(expected.parent)}
        with patch.object(native_tests, "ROOT", checkout):
            self.assertEqual(
                native_tests.python_native_binary(environment), expected / "_native.py"
            )

    def test_running_pytest_refuses_a_changed_import_before_tests(self) -> None:
        (self.package / "_build.py").write_text(
            "CacheSettings = EngineSettings = object\ndef build_info(): return None\n"
        )
        contracts = self.package / "contracts"
        contracts.mkdir()
        (contracts / "__init__.py").write_text("")
        (contracts / "extension_types.py").write_text("EXTENSION_NAMES = ()\n")
        tests = self.output / "tests"
        tests.mkdir()
        (tests / "conftest.py").write_bytes(
            (ROOT / "python/pse/tests/conftest.py").read_bytes()
        )
        (tests / "test_probe.py").write_text(
            "from pathlib import Path\n"
            "def test_must_not_execute(): Path('executed').write_text('wrong import')\n"
        )
        result = subprocess.run(
            [sys.executable, "-m", "pytest", "tests", "-q", "-p", "no:cacheprovider"],
            cwd=self.output,
            env={
                **self.environment,
                "PSE_NATIVE_EXPECTED_BINARY": str(ROOT / "python/pse/_native.so"),
            },
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 4, result.stdout + result.stderr)
        self.assertIn("differs from the binary recorded", result.stderr)
        self.assertFalse((self.output / "executed").exists())

    def test_running_pytest_refuses_replacement_at_the_same_path(self) -> None:
        (self.package / "_build.py").write_text(
            "CacheSettings = EngineSettings = object\ndef build_info(): return None\n"
        )
        contracts = self.package / "contracts"
        contracts.mkdir()
        (contracts / "__init__.py").write_text("")
        (contracts / "extension_types.py").write_text("EXTENSION_NAMES = ()\n")
        tests = self.output / "tests"
        tests.mkdir()
        (tests / "conftest.py").write_bytes(
            (ROOT / "python/pse/tests/conftest.py").read_bytes()
        )
        (tests / "test_probe.py").write_text(
            "def test_must_not_execute(): raise AssertionError('executed')\n"
        )
        result = subprocess.run(
            [sys.executable, "-m", "pytest", "tests", "-q", "-p", "no:cacheprovider"],
            cwd=self.output,
            env={
                **self.environment,
                "PSE_NATIVE_EXPECTED_BINARY": str(self.package / "_native.py"),
                "PSE_NATIVE_EXPECTED_SHA256": "previous-artifact",
            },
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 4, result.stdout + result.stderr)
        self.assertIn("changed after wrapper provenance capture", result.stderr)


class ExecutionContracts(unittest.TestCase):
    def setUp(self) -> None:
        composition = patch.object(
            native_tests,
            "correctness_command",
            side_effect=lambda command, _environment: command,
        )
        composition.start()
        self.addCleanup(composition.stop)
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        registry = patch.object(
            test_resources, "registry", return_value=self.output / "registry"
        )
        registry.start()
        self.addCleanup(registry.stop)
        self.rust_invocation = self.output / "rust-invocation.json"
        self.rust_invocation.write_text(
            json.dumps(
                {
                    "version": 1,
                    "nonce": "a" * 32,
                    "kind": "rust",
                    "selected": [],
                    "terminal_owner": "assessment",
                }
            )
        )
        self.rust_invocation.chmod(0o600)
        binary = ROOT / "python/pse/_native.so"
        self.native_binary = self.enterContext(
            patch.object(native_tests, "python_native_binary", return_value=binary)
        )

    def test_native_provenance_refuses_changed_deployment_inputs(self) -> None:
        binary = self.output / "worker"
        binary.write_bytes(b"observed executable")
        names = (
            "PSE_PRODUCER_RECEIPT",
            "PSE_WORKER_PRODUCER_RECEIPT",
            "PSE_PYTHON_PRODUCER_RECEIPT",
            "PSE_PYTHON_DEPLOYMENT_ATTESTATION",
        )
        inputs = {name: self.output / f"{name}.json" for name in names}
        for path in inputs.values():
            path.write_bytes(b"observed input")
        environment = {
            **{name: str(path) for name, path in inputs.items()},
            "OMP_NUM_THREADS": "1",
            "OPENBLAS_NUM_THREADS": "1",
            "MKL_NUM_THREADS": "1",
        }
        with patch.object(
            native_tests.subprocess, "check_output", return_value="observed tool output"
        ):
            native = native_tests.native_provenance(
                {}, [str(binary)], environment=environment
            )
        validation_receipts.verify_native(native)
        for name, path in inputs.items():
            with self.subTest(name=name):
                path.write_bytes(b"replacement input")
                with self.assertRaisesRegex(
                    ValueError, "binary or linked library changed"
                ):
                    validation_receipts.verify_native(native)
                path.write_bytes(b"observed input")

    def test_native_wrapper_lists_once_then_runs_with_exact_filter(self) -> None:
        provenance = self.output / "native.json"
        inventory = {
            "rust-suites": {
                "suite": {
                    "binary-id": "suite",
                    "binary-path": "/test/binary",
                    "testcases": {"case": {"filter-match": {"status": "matches"}}},
                }
            }
        }
        selection = "test(=exact); literal $(must-not-expand)"
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_PROVENANCE": str(provenance),
                    "PSE_TEST_INVOCATION": str(self.rust_invocation),
                },
                clear=True,
            ),
            patch.object(sys, "argv", ["native_tests", "rust", "-E", selection]),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {str(ROOT / "python/pse/_native.so"): "digest"}},
            ),
            patch.object(
                native_tests.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, stdout=json.dumps(inventory)
                ),
            ) as listing,
            patch.object(native_tests.subprocess, "call", return_value=0) as run,
        ):
            self.assertEqual(native_tests.main(), 0)
        self.assertEqual(listing.call_count, 1)
        self.assertEqual(listing.call_args.args[0][-2:], ["-E", selection])
        self.assertEqual(run.call_args.args[0][-2:], ["-E", selection])
        self.assertEqual(
            json.loads((self.output / "native-selected.json").read_text()), inventory
        )
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_PROVENANCE": str(provenance),
                    "PSE_TEST_INVOCATION": str(self.rust_invocation),
                },
                clear=True,
            ),
            patch.object(sys, "argv", ["native_tests", "rust"]),
            patch.object(
                native_tests.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 7, stdout="failed collection"
                ),
            ),
            patch.object(native_tests.subprocess, "call") as run,
        ):
            self.assertEqual(native_tests.main(), 7)
            run.assert_not_called()
        self.assertEqual(
            (self.output / "native-selected.json").read_text(), "failed collection"
        )

    def test_selected_worker_uses_actual_built_executable_in_requested_profile(
        self,
    ) -> None:
        binary = self.output / "artifacts/worker"
        binary.parent.mkdir()
        binary.write_bytes(b"child executable")
        binary.chmod(0o700)
        inventory = {
            "rust-suites": {
                "pse-runtime::worker": {
                    "binary-id": "pse-runtime::worker",
                    "binary-name": "worker",
                    "binary-path": "/test/worker-journey",
                    "testcases": {"runs_case": {"filter-match": {"status": "matches"}}},
                }
            }
        }
        artifact = {
            "reason": "compiler-artifact",
            "target": {"name": "pse-worker"},
            "executable": str(binary),
        }
        with (
            patch.dict(
                os.environ,
                {
                    "PSE_NATIVE_PROVENANCE": str(self.output / "native.json"),
                    "PSE_TEST_INVOCATION": str(self.rust_invocation),
                },
                clear=True,
            ),
            patch.object(
                sys, "argv", ["native_tests", "rust", "--cargo-profile", "producer"]
            ),
            patch.object(
                native_tests.subprocess,
                "run",
                side_effect=[
                    subprocess.CompletedProcess([], 0, stdout=json.dumps(inventory)),
                    subprocess.CompletedProcess([], 0, stdout=json.dumps(artifact)),
                ],
            ) as build,
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {str(ROOT / "python/pse/_native.so"): "digest"}},
            ) as provenance,
            patch.object(native_tests.subprocess, "call", return_value=0) as run,
        ):
            self.assertEqual(native_tests.main(), 0)
        command = build.call_args.args[0]
        self.assertEqual(command[command.index("--profile") + 1], "producer")
        self.assertEqual(run.call_args.kwargs["env"]["PSE_WORKER_BINARY"], str(binary))
        self.assertIn(str(binary), provenance.call_args.args[1])

    def test_missing_supplied_worker_refuses_before_test_execution(self) -> None:
        with (
            patch.object(native_tests.subprocess, "run") as build,
            self.assertRaisesRegex(ValueError, "executable file"),
        ):
            native_tests.worker_binary(
                [], {"PSE_WORKER_BINARY": str(self.output / "missing")}
            )
        build.assert_not_called()

    def test_failed_worker_build_does_not_substitute_a_stale_default_binary(
        self,
    ) -> None:
        with (
            patch.object(
                native_tests.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 17, stdout="build failed\n"
                ),
            ),
            self.assertRaisesRegex(ValueError, "build failed with exit 17"),
        ):
            native_tests.worker_binary(["--release"], {})

    def test_nested_python_returns_exit_without_parsing_terminal_report(self) -> None:
        report = self.output / "nested.xml"
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "python",
                    "--functional-observer-child",
                    f"--junitxml={report}",
                    "--terminal-owner=assessment",
                ],
            ),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {str(ROOT / "python/pse/_native.so"): "digest"}},
            ),
            patch.object(native_tests.subprocess, "call", return_value=9),
            patch.object(validation, "compose_terminal") as compose,
        ):
            self.assertEqual(native_tests.main(), 9)
            compose.assert_not_called()

    def test_standalone_python_composes_once_and_preserves_nonzero_exit(self) -> None:
        report = self.output / "standalone.xml"

        def run(_command: list[str], *, cwd: Path, env: dict[str, str]) -> int:
            self.assertEqual(cwd, native_tests.ROOT)
            self.assertEqual(
                env["PSE_NATIVE_EXPECTED_BINARY"], str(ROOT / "python/pse/_native.so")
            )
            report.write_text(
                '<testsuite><testcase name="one"><properties><property name="nodeid" value="one"/></properties></testcase></testsuite>'
            )
            Path(env["PSE_TEST_ENUMERATION"]).write_text("one\n")
            return 7

        native = {
            "schema": "native-profile-v1",
            "files": {str(ROOT / "python/pse/_native.so"): "digest"},
        }
        # A synchronous mocked child can finish inside the filesystem timestamp
        # tick. Give this successful report a deterministic invocation boundary;
        # freshness rejection is exercised independently below.
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "python",
                    "--functional-observer-child",
                    f"--junitxml={report}",
                ],
            ),
            patch.object(native_tests.time, "time", return_value=1),
            patch.object(
                native_tests, "native_provenance", return_value=native
            ) as provenance,
            patch.object(native_tests.subprocess, "call", side_effect=run) as child,
            patch.object(
                validation, "collect_report", wraps=validation.collect_report
            ) as collect,
        ):
            self.assertEqual(native_tests.main(), 7)
            self.assertEqual(collect.call_count, 1)
        receipt = json.loads(report.with_suffix(".terminal.json").read_text())
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(receipt["results"][0]["status"], "passed")
        self.assertEqual(receipt["native"], native)
        self.assertEqual(receipt["command"], child.call_args.args[0])
        self.assertEqual(receipt["mode"], "python-native")
        self.assertEqual(
            json.loads(report.with_suffix(".native.json").read_text()), native
        )
        provenance.assert_called_once()
        self.assertEqual(
            provenance.call_args.args[1], [str(ROOT / "python/pse/_native.so")]
        )

    def test_standalone_python_rejects_stale_report_and_missing_native_identity(
        self,
    ) -> None:
        report = self.output / "stale.xml"
        report.write_text('<testsuite><testcase name="one"/></testsuite>')
        os.utime(report, (1, 1))

        def run(_command: list[str], *, cwd: Path, env: dict[str, str]) -> int:
            self.assertEqual(cwd, native_tests.ROOT)
            Path(env["PSE_TEST_ENUMERATION"]).write_text("one\n")
            return 0

        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "python",
                    "--functional-observer-child",
                    f"--junitxml={report}",
                ],
            ),
            patch.object(
                native_tests,
                "native_provenance",
                return_value={"files": {str(ROOT / "python/pse/_native.so"): "digest"}},
            ),
            patch.object(native_tests.subprocess, "call", side_effect=run),
        ):
            self.assertEqual(native_tests.main(), 1)
        receipt = json.loads(report.with_suffix(".terminal.json").read_text())
        self.assertTrue(
            any("missing current report" in error for error in receipt["report_errors"])
        )
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                sys,
                "argv",
                [
                    "native_tests",
                    "python",
                    "--functional-observer-child",
                    f"--junitxml={report}",
                ],
            ),
            patch.object(
                native_tests,
                "native_provenance",
                side_effect=ValueError("no actual native binaries"),
            ),
            patch.object(native_tests.subprocess, "call") as child,
            self.assertRaisesRegex(ValueError, "no actual native binaries"),
        ):
            native_tests.main()
        child.assert_not_called()

    def test_pure_pytest_run_has_no_git_attribution_or_native_setup(self) -> None:
        (self.output / "conftest.py").write_bytes((ROOT / "conftest.py").read_bytes())
        (self.output / "test_owned.py").write_text(
            'import pathlib, pytest\n@pytest.mark.unit\ndef test_owned(tmp_path):\n    (tmp_path / "output").write_text("owned")\n    pathlib.Path("concurrent.txt").write_text("authorized unrelated edit")\n'
        )
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                "-q",
                "-p",
                "no:cacheprovider",
                "-p",
                "xdist.plugin",
            ],
            cwd=self.output,
            capture_output=True,
            text=True,
            check=False,
            env={
                **os.environ,
                "PYTHONPATH": str(ROOT),
                "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
            },
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(
            (self.output / "concurrent.txt").read_text(), "authorized unrelated edit"
        )
        for markers in ("", "@pytest.mark.unit\n@pytest.mark.component\n"):
            (self.output / "test_owned.py").write_text(
                "import pytest\n" + markers + "def test_bad(): pass\n"
            )
            failed = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "-q",
                    "-p",
                    "no:cacheprovider",
                    "-p",
                    "xdist.plugin",
                ],
                cwd=self.output,
                capture_output=True,
                text=True,
                check=False,
                env={
                    **os.environ,
                    "PYTHONPATH": str(ROOT),
                    "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
                },
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("exactly one", failed.stderr)

    def claim(self, gate: Gate | None = None) -> dict:
        gate = gate or FUNCTIONAL_SCOPES["preparation"]
        binary = self.output / "test-binary"
        binary.write_bytes(b"native binary")
        native = {
            "schema": "native-profile-v1",
            "captured": 2,
            "files": {str(binary): validation_receipts.digest(binary)},
            "links": {str(binary): ""},
            "toolchain": "pinned",
            "threads": dict.fromkeys(
                ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"), "1"
            ),
            "environment": {},
            "profile": {
                "cargo_profile": "dev",
                "features": native_tests.FEATURES.split(","),
            },
        }
        declaration = json.loads(json.dumps(asdict(gate)))
        selected = [{"class": "owner", "name": "behavior"}]
        check = {
            "gate": gate.name,
            "invocation": declaration,
            "mode": gate.mode,
            "profile": gate.profile,
            "exit_code": 0,
            "started": 1,
            "status": "passed",
            "selected": selected,
            "results": [{**selected[0], "status": "passed"}],
            "report_errors": [],
            "changed_source": [],
            "artifacts": {},
            "evidence_kind": "executed",
            "native": native,
            "inputs": validation.input_identity(
                gate.input_scope, {"Cargo.lock": "original"}, {}
            ),
        }
        return {
            "version": 5,
            "input_coverage": True,
            "complete": True,
            "provenance_errors": [],
            "baseline_failures": 0,
            "scope": [declaration],
            "checks": [check],
            "environment": {},
        }

    def require(
        self,
        receipt: dict,
        *,
        scopes: list[str] | None = None,
        snapshot: dict[str, str] | None = None,
        environment: dict[str, str] | None = None,
    ) -> dict:
        validation.write_json(self.output / "checks.json", receipt)
        with (
            patch.object(
                validation, "relevant_environment", return_value=environment or {}
            ),
            patch.object(
                case_measure,
                "functional_features",
                side_effect=lambda gate: set(
                    (
                        native_tests.MANAGED_FEATURES
                        if "--managed-primary-route" in gate.args
                        else native_tests.FEATURES
                    ).split(",")
                ),
            ),
            patch.object(case_measure, "verify_functional_execution"),
        ):
            return case_measure.require_functional(
                self.output,
                self.output,
                [{"id": "chosen", "functional_scopes": scopes or ["preparation"]}],
                snapshot=snapshot
                or {"Cargo.lock": "original", "docs/plans/prose.md": "changed"},
            )

    def composite_claim(self, gate: Gate) -> tuple[dict, dict[str, str]]:
        receipt = self.claim(gate)
        parent = validation.relevant_environment({})
        path = self.output / "producer-fixture.json"
        path.write_text('{"identity": "qualified-finite-fixture"}')
        environment = validation.relevant_environment(
            {"PSE_PRODUCER_FIXTURE_RECEIPT": str(path)}
        )
        receipt["environment"] = parent
        native_check = receipt["checks"][0]
        native_check["inputs"] = validation.input_identity(
            gate.input_scope, {"Cargo.lock": "original"}, parent
        )
        native_check["native"]["environment"] = environment
        native_check["native"]["files"][str(path)] = validation_receipts.digest(path)
        if gate.name == "managed-native":
            native_check["native"]["profile"]["features"] = (
                native_tests.MANAGED_FEATURES.split(",")
            )
        fixture = next(
            candidate
            for candidate in comprehensive()
            if candidate.name == "producer-fixture"
        )
        fixture_receipt = self.claim(fixture)
        fixture_check = fixture_receipt["checks"][0]
        fixture_check.pop("native")
        fixture_check.pop("selected")
        fixture_check["results"] = []  # The producing command owns terminal success.
        fixture_check["inputs"] = validation.input_identity(
            fixture.input_scope,
            {"Cargo.lock": "original", "docs/plans/prose.md": "changed"},
            parent,
        )
        fixture_check["artifacts"] = {path.name: validation_receipts.digest(path)}
        receipt["scope"].extend(fixture_receipt["scope"])
        receipt["checks"].append(fixture_check)
        return receipt, environment

    def test_selected_measurement_accepts_exact_or_explicit_covering_native_claim(
        self,
    ) -> None:
        assembled = next(gate for gate in comprehensive() if gate.name == "native-test")
        producer_assembled = next(
            gate for gate in comprehensive("producer") if gate.name == "native-test"
        )
        for gate in (
            FUNCTIONAL_SCOPES["preparation"],
            native_gate(),
            assembled,
            producer_assembled,
        ):
            receipt, environment = (
                self.composite_claim(gate)
                if gate.dependencies
                else (self.claim(gate), {})
            )
            consumed = self.require(receipt, environment=environment)
            self.assertEqual(
                consumed["prerequisites"]["preparation"]["gate"], gate.name
            )
        arbitrary = replace(
            native_gate(), args=("--profile", "local", "-E", "package(pse-runtime)")
        )
        with self.assertRaisesRegex(ValueError, "exact functional invocation"):
            self.require(self.claim(arbitrary))
        with self.assertRaisesRegex(ValueError, "exact functional invocation"):
            self.require(
                self.claim(replace(assembled, dependencies=("unreviewed-setup",)))
            )

    def test_managed_measurement_requires_its_positive_managed_native_owner(
        self,
    ) -> None:
        ordinary = next(gate for gate in comprehensive() if gate.name == "native-test")
        with self.assertRaisesRegex(ValueError, "exact functional invocation"):
            self.require(self.claim(ordinary), scopes=["managed-primary"])
        for profile in ("dev", "producer"):
            gate = next(
                gate for gate in comprehensive(profile) if gate.name == "managed-native"
            )
            receipt, environment = self.composite_claim(gate)
            consumed = self.require(
                receipt, scopes=["managed-primary"], environment=environment
            )
            self.assertEqual(
                consumed["prerequisites"]["managed-primary"]["gate"], "managed-native"
            )
            for mutation in ("failed", "missing-terminal"):
                failed = json.loads(json.dumps(receipt))
                check = failed["checks"][0]
                if mutation == "failed":
                    check["status"], check["exit_code"] = "failed", 1
                else:
                    check["results"] = []
                with (
                    self.subTest(profile=profile, mutation=mutation),
                    self.assertRaisesRegex(
                        ValueError, "incomplete functional prerequisite"
                    ),
                ):
                    self.require(
                        failed, scopes=["managed-primary"], environment=environment
                    )

    def test_assessment_fixture_composes_only_its_owned_environment_injection(
        self,
    ) -> None:
        for name, scopes in (
            ("native-test", ["preparation"]),
            ("managed-native", ["managed-primary"]),
        ):
            gate = next(
                candidate for candidate in comprehensive() if candidate.name == name
            )
            receipt, environment = self.composite_claim(gate)
            self.require(receipt, scopes=scopes, environment=environment)
            # Preserve other selectors, including their captured bytes.
            selector = {
                "resolved": "/native/provider",
                "sha256": "exact",
                "admission": None,
            }
            for captured in (receipt["environment"], environment):
                configuration = json.loads(captured["EFFECTIVE_NATIVE_CONFIGURATION"])
                configuration["IPOPT_DIR"] = selector
                captured["EFFECTIVE_NATIVE_CONFIGURATION"] = json.dumps(
                    configuration, sort_keys=True
                )
            for check in receipt["checks"]:
                scope = check["invocation"]["input_scope"]
                check["inputs"] = validation.input_identity(
                    scope,
                    {"Cargo.lock": "original", "docs/plans/prose.md": "changed"},
                    receipt["environment"],
                )
            self.require(receipt, scopes=scopes, environment=environment)

    def test_assessment_fixture_refuses_tampering_and_unowned_context_changes(
        self,
    ) -> None:
        gate = next(
            candidate
            for candidate in comprehensive()
            if candidate.name == "native-test"
        )
        for mutation in (
            "failed-producer",
            "skipped-producer",
            "changed-source",
            "changed-inputs",
            "transferred-producer",
            "wrong-producer-declaration",
            "missing-artifact",
            "changed-artifact",
            "wrong-native-digest",
            "wrong-fixture-path",
            "wrong-current-fixture",
            "wrong-configuration-digest",
            "other-selector",
            "other-environment",
            "missing-parent-configuration",
            "changed-origin",
            "stale-artifact",
            "unexecuted-producer",
        ):
            receipt, environment = self.composite_claim(gate)
            producer = receipt["checks"][1]
            native = receipt["checks"][0]["native"]
            if mutation == "failed-producer":
                producer.update(status="failed", exit_code=1)
            elif mutation == "skipped-producer":
                producer["results"] = [
                    {"class": "producer", "name": "capture", "status": "skipped"}
                ]
            elif mutation == "changed-source":
                producer["changed_source"] = ["scripts/producer.py"]
            elif mutation == "changed-inputs":
                producer["inputs"]["files"]["Cargo.lock"] = "different"
            elif mutation == "transferred-producer":
                producer["applicability_transfers"] = [{"reason": "different context"}]
            elif mutation == "wrong-producer-declaration":
                receipt["scope"][1]["args"] = ["another-fixture.json"]
            elif mutation == "missing-artifact":
                producer["artifacts"] = {}
            elif mutation == "changed-artifact":
                (self.output / "producer-fixture.json").write_text("changed")
            elif mutation == "wrong-native-digest":
                native["files"][str(self.output / "producer-fixture.json")] = "wrong"
            elif mutation == "wrong-fixture-path":
                native["environment"] = {
                    **environment,
                    "PSE_PRODUCER_FIXTURE_RECEIPT": "/other/producer-fixture.json",
                }
            elif mutation == "wrong-current-fixture":
                environment["PSE_PRODUCER_FIXTURE_RECEIPT"] = (
                    "/other/producer-fixture.json"
                )
            elif mutation in {"wrong-configuration-digest", "other-selector"}:
                configuration = json.loads(
                    environment["EFFECTIVE_NATIVE_CONFIGURATION"]
                )
                if mutation == "wrong-configuration-digest":
                    configuration["PSE_PRODUCER_FIXTURE_RECEIPT"]["sha256"] = "wrong"
                else:
                    configuration["IPOPT_DIR"] = {"resolved": "/another/provider"}
                environment["EFFECTIVE_NATIVE_CONFIGURATION"] = json.dumps(
                    configuration, sort_keys=True
                )
            elif mutation == "other-environment":
                environment["PSE_MEMORY_MAX"] = "4G"
            elif mutation == "missing-parent-configuration":
                receipt["environment"].pop("EFFECTIVE_NATIVE_CONFIGURATION")
                for check in receipt["checks"]:
                    check["inputs"]["environment"].pop("EFFECTIVE_NATIVE_CONFIGURATION")
            elif mutation == "changed-origin":
                producer.update(origin=str(self.output), origin_digest="wrong")
            elif mutation == "stale-artifact":
                producer["started"] = (
                    self.output / "producer-fixture.json"
                ).stat().st_mtime + 1
            elif mutation == "unexecuted-producer":
                producer["selected"] = [{"class": "producer", "name": "missing"}]
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.require(receipt, environment=environment)

        receipt, environment = self.composite_claim(gate)
        standalone = native_gate()
        receipt["scope"][0] = json.loads(json.dumps(asdict(standalone)))
        receipt["checks"][0]["invocation"] = receipt["scope"][0]
        with self.assertRaisesRegex(ValueError, "invocation environment changed"):
            self.require(receipt, environment=environment)

    def test_assessment_fixture_reuse_authenticates_its_original_report(self) -> None:
        gate = next(
            candidate
            for candidate in comprehensive()
            if candidate.name == "native-test"
        )
        receipt, environment = self.composite_claim(gate)
        origin = self.output / "original"
        origin.mkdir()
        path = origin / "producer-fixture.json"
        path.write_bytes((self.output / path.name).read_bytes())
        environment = validation.relevant_environment(
            {"PSE_PRODUCER_FIXTURE_RECEIPT": str(path)}
        )
        receipt["checks"][0]["native"]["environment"] = environment
        receipt["checks"][0]["native"]["files"][str(path)] = validation_receipts.digest(
            path
        )
        validation.write_json(origin / "checks.json", receipt)
        producer = receipt["checks"][1]
        producer.update(
            evidence_kind="unchanged-input-reuse",
            origin=str(origin),
            origin_digest=validation_receipts.digest(origin / "checks.json"),
        )
        self.require(receipt, environment=environment)
        (origin / "checks.json").write_text("changed original")
        with self.assertRaisesRegex(
            ValueError, "changed assessment producer-fixture origin"
        ):
            self.require(receipt, environment=environment)

    def test_standalone_managed_prerequisite_retains_existing_route(self) -> None:
        gate = FUNCTIONAL_SCOPES["managed-primary"]
        assembled = next(
            gate for gate in comprehensive() if gate.name == "managed-native"
        )
        self.assertEqual(replace(assembled, dependencies=()), gate)
        self.assertEqual(gate.recipe, "native-test")
        self.assertEqual(gate.profile, "local")
        self.assertEqual(gate.mode, "native-force-validate")
        selection = native_tests.managed_rust_selection(list(gate.args[:-1]))
        self.assertEqual(
            selection,
            [
                "--profile",
                "local",
                "--ignore-default-filter",
                "-E",
                "test(managed_primary_)",
            ],
        )
        receipt = self.claim(gate)
        receipt["checks"][0]["native"]["profile"]["features"] = (
            native_tests.MANAGED_FEATURES.split(",")
        )
        self.assertEqual(
            self.require(receipt, scopes=["managed-primary"])["prerequisites"][
                "managed-primary"
            ]["gate"],
            "managed-native",
        )

    def test_measurement_requires_matching_parent_and_native_state_and_allocation(
        self,
    ) -> None:
        for managed, state, memory in (
            (False, "/state/wide", "80G"),
            (False, "/state/reference", "160G"),
            (True, "/state/reference", "160G"),
        ):
            scopes = ["managed-primary"] if managed else ["preparation"]
            gate = FUNCTIONAL_SCOPES[scopes[0]]
            environment = {
                "PSE_SURREAL_STATE": state,
                "PSE_MEMORY_MAX": memory,
                "PSE_WORKER_BINARY": "/qualified/worker",
                "LD_LIBRARY_PATH": "/qualified/native/lib",
            }
            receipt = self.claim(gate)
            check = receipt["checks"][0]
            receipt["environment"] = environment
            check["native"]["environment"] = dict(environment)
            if managed:
                check["native"]["profile"]["features"] = (
                    native_tests.MANAGED_FEATURES.split(",")
                )
            check["inputs"] = validation.input_identity(
                gate.input_scope, {"Cargo.lock": "original"}, environment
            )
            with self.subTest(managed=managed, memory=memory):
                self.require(receipt, scopes=scopes, environment=environment)
            for owner in ("parent", "native"):
                for key, value in (
                    ("PSE_SURREAL_STATE", "/state/another"),
                    ("PSE_MEMORY_MAX", "4G"),
                    ("PSE_WORKER_BINARY", "/another/worker"),
                    ("LD_LIBRARY_PATH", "/another/native/lib"),
                ):
                    altered = json.loads(json.dumps(receipt))
                    if owner == "parent":
                        altered["environment"][key] = value
                        altered["checks"][0]["inputs"] = validation.input_identity(
                            gate.input_scope,
                            {"Cargo.lock": "original"},
                            altered["environment"],
                        )
                    else:
                        altered["checks"][0]["native"]["environment"][key] = value
                    with (
                        self.subTest(managed=managed, owner=owner, key=key),
                        self.assertRaisesRegex(ValueError, "environment changed"),
                    ):
                        self.require(altered, scopes=scopes, environment=environment)

    def test_measurement_rejects_unfinished_stale_or_transferred_prerequisite(
        self,
    ) -> None:
        for mutation in (
            "unfinished",
            "provenance",
            "skipped",
            "missing-selected-terminal",
            "stale-native",
            "changed-environment",
            "transfer",
            "transfer-history",
            "duplicate-gate",
            "duplicate-check",
            "changed-artifact",
        ):
            receipt = self.claim()
            check = receipt["checks"][0]
            if mutation == "unfinished":
                receipt["complete"] = False
            elif mutation == "provenance":
                receipt["provenance_errors"] = ["capture unavailable"]
            elif mutation == "skipped":
                check["results"][0]["status"] = "skipped"
            elif mutation == "missing-selected-terminal":
                check["selected"].append({"class": "owner", "name": "unexecuted"})
            elif mutation == "stale-native":
                check["native"]["captured"] = 0
            elif mutation == "changed-environment":
                check["changed_environment"] = ["PSE_MEMORY_MAX"]
            elif mutation == "transfer":
                check["evidence_kind"] = "reviewed-transfer"
            elif mutation == "transfer-history":
                check["applicability_transfers"] = [{"reason": "old transfer"}]
            elif mutation == "duplicate-gate":
                receipt["scope"].append(receipt["scope"][0])
            elif mutation == "duplicate-check":
                receipt["checks"].append(check)
            elif mutation == "changed-artifact":
                artifact = self.output / "selection.json"
                artifact.write_text("original")
                check["artifacts"][artifact.name] = validation_receipts.digest(artifact)
                artifact.write_text("changed")
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.require(receipt)

    def test_functional_observer_context_retains_parent_cap_and_actual_child_scope(
        self,
    ) -> None:
        for managed in (False, True):
            environment = {
                "PSE_SURREAL_STATE": str(self.output),
                "PSE_MEMORY_MAX": "160G" if managed else "80G",
                "PSE_WORKER_BINARY": "/qualified/worker",
                "LD_LIBRARY_PATH": "/qualified/native/lib",
            }
            profile = "reference" if managed else "exclusive-observer"
            resources = surreal_server.execution_resources(profile)
            allocation = resources["execution"]
            assert isinstance(allocation, dict)
            child = native_tests.rust_test_environment(environment)
            child.pop("PSE_MEMORY_MAX")
            native = {
                "captured": 1,
                "execution": {
                    "captured": 2,
                    "environment": child,
                    "profile": profile,
                    "resources": resources,
                    "state_resources": resources,
                    "control_group": "/admitted/native-observer.scope",
                    "memory_max": allocation["observer_memory_bytes"],
                    "cpu_threads": allocation["cpu_threads"],
                },
            }
            with patch.object(
                surreal_server, "config_for", return_value={"resources": resources}
            ):
                case_measure.verify_functional_execution(
                    native, managed=managed, environment=environment
                )
                for mutation in (
                    "parent-cap-in-child",
                    "state",
                    "worker",
                    "native-path",
                    "profile",
                    "resources",
                    "state-resources",
                    "memory",
                    "cpu",
                    "stale",
                ):
                    altered = json.loads(json.dumps(native))
                    execution = altered["execution"]
                    if mutation == "parent-cap-in-child":
                        execution["environment"]["PSE_MEMORY_MAX"] = environment[
                            "PSE_MEMORY_MAX"
                        ]
                    elif mutation == "state":
                        execution["environment"]["PSE_SURREAL_STATE"] = "/another/state"
                    elif mutation == "worker":
                        execution["environment"]["PSE_WORKER_BINARY"] = (
                            "/another/worker"
                        )
                    elif mutation == "native-path":
                        execution["environment"]["LD_LIBRARY_PATH"] = (
                            "/another/native/lib"
                        )
                    elif mutation == "profile":
                        execution["profile"] = "timing"
                    elif mutation == "resources":
                        execution["resources"] = {}
                    elif mutation == "state-resources":
                        execution["state_resources"] = {}
                    elif mutation == "memory":
                        execution["memory_max"] = 1 << 30
                    elif mutation == "cpu":
                        execution["cpu_threads"] = 1
                    elif mutation == "stale":
                        execution["captured"] = 0
                    with (
                        self.subTest(managed=managed, mutation=mutation),
                        self.assertRaises(ValueError),
                    ):
                        case_measure.verify_functional_execution(
                            altered, managed=managed, environment=environment
                        )

    def test_functional_feature_identity_uses_current_native_composition(self) -> None:
        for scope in ("preparation", "managed-primary"):
            gate = FUNCTIONAL_SCOPES[scope]
            managed = scope == "managed-primary"
            with patch.object(
                native_tests,
                "correctness_command",
                side_effect=lambda command, _environment: [
                    *command,
                    "--features",
                    "pse-runtime/force-validate",
                ],
            ) as compose:
                features = case_measure.functional_features(gate)
            self.assertEqual(
                features,
                set(native_tests.native_root_features([], managed=managed).split(","))
                | {"pse-runtime/force-validate"},
            )
            command = compose.call_args.args[0]
            self.assertEqual(command[:3], ["cargo", "nextest", "list"])
            self.assertIn("--profile", command)
            self.assertNotIn("--managed-primary-route", command)
            if managed:
                self.assertIn("--ignore-default-filter", command)
                self.assertEqual(command[-2:], ["-E", "test(managed_primary_)"])

    def test_managed_fixture_drain_refuses_another_database_before_signalling(
        self,
    ) -> None:
        marker = {"pid": os.getpid(), "canonical_database": "another-fixture"}
        with (
            patch.object(surreal_server, "state_lock", return_value=nullcontext()),
            patch.object(surreal_server, "config_for", return_value={}),
            patch.object(
                surreal_server,
                "primary_observation",
                return_value={"ActiveState": "active", "ControlGroup": "/primary"},
            ),
            patch.object(surreal_server, "read_json", return_value=marker),
            patch.object(case_measure.os, "pidfd_open") as opened,
            patch.object(case_measure.signal, "pidfd_send_signal") as signalled,
            self.assertRaisesRegex(ValueError, "not associated with this fixture"),
        ):
            case_measure.stop_managed_primary(self.output, "owned-fixture")
        opened.assert_not_called()
        signalled.assert_not_called()

    def test_managed_fixture_drain_refuses_a_changed_receiver_before_signalling(
        self,
    ) -> None:
        marker = {
            "pid": os.getpid(),
            "canonical_database": "owned-fixture",
            "nonce": "original",
        }
        changed = {**marker, "nonce": "replacement"}
        with (
            patch.object(surreal_server, "state_lock", return_value=nullcontext()),
            patch.object(surreal_server, "config_for", return_value={}),
            patch.object(
                surreal_server,
                "primary_observation",
                return_value={"ActiveState": "active", "ControlGroup": "/primary"},
            ),
            patch.object(surreal_server, "primary_ready", return_value=True),
            patch.object(surreal_server, "read_json", side_effect=[marker, changed]),
            patch.object(case_measure.os, "pidfd_open", return_value=123) as opened,
            patch.object(case_measure.os, "close") as closed,
            patch.object(case_measure.signal, "pidfd_send_signal") as signalled,
            self.assertRaisesRegex(ValueError, "association changed before drain"),
        ):
            case_measure.stop_managed_primary(self.output, "owned-fixture")
        opened.assert_called_once_with(os.getpid())
        closed.assert_called_once_with(123)
        signalled.assert_not_called()

    def test_selected_measurement_rejects_wrong_workload_mode_or_incomplete_claim(
        self,
    ) -> None:
        for mutation in (
            "missing-terminal",
            "failed",
            "wrong-mode",
            "wrong-profile",
            "wrong-version",
            "changed-provider",
        ):
            receipt = self.claim()
            check = receipt["checks"][0]
            if mutation == "missing-terminal":
                check["results"] = []
            if mutation == "failed":
                check["exit_code"] = 1
            if mutation == "wrong-mode":
                check["mode"] = "default"
            if mutation == "wrong-profile":
                check["native"]["profile"]["cargo_profile"] = "release"
            if mutation == "wrong-version":
                receipt["version"] = 4
            if mutation == "changed-provider":
                Path(next(iter(check["native"]["files"]))).write_bytes(b"replacement")
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.require(receipt)
        with self.assertRaises(ValueError):
            self.require(self.claim(), scopes=["process"])
        with self.assertRaisesRegex(ValueError, "inputs changed"):
            self.require(self.claim(), snapshot={"Cargo.lock": "changed"})

    def seal_reuse_origin(self, receipt: dict, label: str) -> Path:
        owner = self.output / "report-owner"
        origin = owner / "build" / label
        origin.mkdir(parents=True)
        (origin / "fixture-source.json").write_text('{"scope":"native-claim-control"}')
        receipt = {
            "complete": True,
            "required_checks_covered": True,
            "source_unchanged": True,
            "provenance_errors": [],
            **receipt,
        }
        validation.write_json(origin / "checks.json", receipt)
        resource = test_resources.register_report(origin, root=owner)
        test_resources.finish_report(
            resource,
            receipt,
            roles={"checks.json": "receipt", "fixture-source.json": "provenance"},
            required_provenance=["fixture-source.json"],
        )
        return origin

    def test_scope_definition_change_and_uncovered_receipt_refuse_reuse(self) -> None:
        receipt = self.claim()
        receipt["source_files"] = {"Cargo.lock": "original"}
        origin = self.seal_reuse_origin(receipt, "original")
        gate = FUNCTIONAL_SCOPES["preparation"].name
        with (
            patch(
                "scripts.validation_scope.INPUT_SCOPE_VERSION",
                receipt["checks"][0]["inputs"]["version"] + 1,
            ),
            self.assertRaisesRegex(ValueError, "identical inputs"),
        ):
            validation_receipts.reuse_checks(
                origin,
                receipt["source_files"],
                {},
                receipt["scope"],
                {gate},
                set(),
                None,
            )
        receipt["input_coverage"] = False
        origin = self.seal_reuse_origin(receipt, "uncovered")
        with self.assertRaisesRegex(ValueError, "trustworthy input coverage"):
            validation_receipts.reuse_checks(
                origin,
                receipt["source_files"],
                {},
                receipt["scope"],
                {gate},
                set(),
                None,
            )

    def test_python_environment_controls_are_captured_and_refuse_changed_reuse(
        self,
    ) -> None:
        receipt = self.claim()
        snapshot = {"Cargo.lock": "original"}
        environment = validation.relevant_environment(
            {"PYTHONPATH": "original-path", "PYTHONHASHSEED": "1"}
        )
        self.assertEqual(environment["PYTHONPATH"], "original-path")
        self.assertEqual(environment["PYTHONHASHSEED"], "1")
        check = receipt["checks"][0]
        check["inputs"] = validation.input_identity(
            "rust-product", snapshot, environment
        )
        origin = self.seal_reuse_origin(receipt, "original")
        gate = check["gate"]
        retained = validation_receipts.reuse_checks(
            origin, snapshot, environment, receipt["scope"], {gate}, set(), None
        )
        self.assertEqual(retained[0]["evidence_kind"], "unchanged-input-reuse")
        for key in ("PYTHONPATH", "PYTHONHASHSEED"):
            changed_environment = validation.relevant_environment(
                {"PYTHONPATH": "original-path", "PYTHONHASHSEED": "1", key: "changed"}
            )
            with (
                self.subTest(control=key),
                self.assertRaisesRegex(ValueError, "identical inputs and environment"),
            ):
                validation_receipts.reuse_checks(
                    origin,
                    snapshot,
                    changed_environment,
                    receipt["scope"],
                    {gate},
                    set(),
                    None,
                )

    def test_reviewed_transfer_keeps_observation_inputs_and_new_applicability(
        self,
    ) -> None:
        receipt = self.claim()
        receipt["source_files"] = {"Cargo.lock": "original"}
        origin = self.seal_reuse_origin(receipt, "original")
        gate = FUNCTIONAL_SCOPES["preparation"].name
        retained = validation_receipts.reuse_checks(
            origin,
            {"Cargo.lock": "changed"},
            {},
            receipt["scope"],
            set(),
            {gate},
            "reviewed lock change",
        )
        self.assertEqual(
            retained[0]["observed_inputs"]["files"]["Cargo.lock"], "original"
        )
        self.assertEqual(retained[0]["inputs"]["files"]["Cargo.lock"], "changed")
        self.assertEqual(retained[0]["changed_inputs"], ["Cargo.lock"])
        self.assertEqual(retained[0]["evidence_kind"], "reviewed-transfer")
        parent = self.seal_reuse_origin({**receipt, "checks": retained}, "transferred")
        reused = validation_receipts.reuse_checks(
            parent, {"Cargo.lock": "changed"}, {}, receipt["scope"], {gate}, set(), None
        )[0]
        self.assertEqual(reused["evidence_kind"], "unchanged-input-reuse")
        self.assertEqual(reused["transfer_reason"], "reviewed lock change")
        self.assertEqual(reused["changed_inputs"], ["Cargo.lock"])
        self.assertEqual(reused["observed_inputs"]["files"]["Cargo.lock"], "original")
        self.assertEqual(
            reused["applicability_transfers"], retained[0]["applicability_transfers"]
        )
        self.assertEqual(
            reused["applicability_transfers"][0]["to_inputs"], reused["inputs"]
        )

    def test_measurement_and_product_identity_include_consumed_benchmarks(
        self,
    ) -> None:
        before = {
            "Cargo.lock": "original",
            "benches/src/native_process.rs": "old",
            ".config/process-cases.json": "old",
            "docs/plans/prose.md": "old",
        }
        after = {
            **before,
            "benches/src/native_process.rs": "new",
            "docs/plans/prose.md": "new",
        }
        self.assertNotEqual(
            case_measure.measurement_inputs(before),
            case_measure.measurement_inputs(after),
        )
        self.assertNotEqual(
            validation.input_identity("rust-product", before, {}),
            validation.input_identity("rust-product", after, {}),
        )


if __name__ == "__main__":
    unittest.main()

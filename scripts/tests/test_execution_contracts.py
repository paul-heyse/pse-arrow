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
from dataclasses import asdict, replace
from pathlib import Path
from unittest.mock import patch

from scripts import case_measure, native_tests, validation, validation_receipts
from scripts.validation_scope import FUNCTIONAL_SCOPES, Gate, native_gate

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
        self.environment = {
            **os.environ,
            "PYTHONPATH": str(self.package.parent),
            "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
        }

    def test_foreign_python_import_is_refused_before_provenance_or_tests(self) -> None:
        report = self.output / "native-python.xml"
        with (
            patch.dict(os.environ, self.environment, clear=True),
            patch.object(
                sys, "argv", ["native_tests", "python", f"--junitxml={report}"]
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
            "CacheSettings = EngineSettings = OperationalStore = "
            "_TestOperationalStore = object\n"
            "def build_info(): return None\n"
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


class ExecutionContracts(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        binary = ROOT / "python/pse/_native.so"
        self.native_binary = self.enterContext(
            patch.object(native_tests, "python_native_binary", return_value=binary)
        )

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
                os.environ, {"PSE_NATIVE_PROVENANCE": str(provenance)}, clear=True
            ),
            patch.object(sys, "argv", ["native_tests", "rust", "-E", selection]),
            patch.object(native_tests, "native_provenance", return_value={}),
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
                os.environ, {"PSE_NATIVE_PROVENANCE": str(provenance)}, clear=True
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
                    f"--junitxml={report}",
                    "--terminal-owner=assessment",
                ],
            ),
            patch.object(native_tests, "native_provenance", return_value={}),
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

        native = {"schema": "native-profile-v1", "files": {"actual-library": "digest"}}
        # A synchronous mocked child can finish inside the filesystem timestamp
        # tick. Give this successful report a deterministic invocation boundary;
        # freshness rejection is exercised independently below.
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                sys, "argv", ["native_tests", "python", f"--junitxml={report}"]
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
                sys, "argv", ["native_tests", "python", f"--junitxml={report}"]
            ),
            patch.object(native_tests, "native_provenance", return_value={}),
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
                sys, "argv", ["native_tests", "python", f"--junitxml={report}"]
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
            [sys.executable, "-m", "pytest", "-q", "-p", "no:cacheprovider"],
            cwd=self.output,
            capture_output=True,
            text=True,
            check=False,
            env={**os.environ, "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"},
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
                [sys.executable, "-m", "pytest", "-q", "-p", "no:cacheprovider"],
                cwd=self.output,
                capture_output=True,
                text=True,
                check=False,
                env={**os.environ, "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"},
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("exactly one", failed.stderr)

    def claim(self, gate: Gate | None = None) -> dict:
        gate = gate or FUNCTIONAL_SCOPES["preparation"]
        binary = self.output / "test-binary"
        binary.write_bytes(b"native binary")
        native = {
            "schema": "native-profile-v1",
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
    ) -> dict:
        validation.write_json(self.output / "checks.json", receipt)
        with patch.object(validation, "relevant_environment", return_value={}):
            return case_measure.require_functional(
                self.output,
                self.output,
                [{"id": "chosen", "functional_scopes": scopes or ["preparation"]}],
                snapshot=snapshot
                or {"Cargo.lock": "original", "docs/plans/prose.md": "changed"},
            )

    def test_selected_measurement_accepts_exact_or_explicit_covering_native_claim(
        self,
    ) -> None:
        for gate in (FUNCTIONAL_SCOPES["preparation"], native_gate()):
            consumed = self.require(self.claim(gate))
            self.assertEqual(
                consumed["prerequisites"]["preparation"]["gate"], gate.name
            )
        arbitrary = replace(
            native_gate(), args=("--profile", "ci", "-E", "package(pse-runtime)")
        )
        with self.assertRaisesRegex(ValueError, "exact functional invocation"):
            self.require(self.claim(arbitrary))

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

    def test_scope_definition_change_and_uncovered_receipt_refuse_reuse(self) -> None:
        receipt = self.claim()
        receipt["source_files"] = {"Cargo.lock": "original"}
        validation.write_json(self.output / "checks.json", receipt)
        gate = FUNCTIONAL_SCOPES["preparation"].name
        with (
            patch("scripts.validation_scope.INPUT_SCOPE_VERSION", 2),
            self.assertRaisesRegex(ValueError, "identical inputs"),
        ):
            validation_receipts.reuse_checks(
                self.output,
                receipt["source_files"],
                {},
                receipt["scope"],
                {gate},
                set(),
                None,
            )
        receipt["input_coverage"] = False
        validation.write_json(self.output / "checks.json", receipt)
        with self.assertRaisesRegex(ValueError, "trustworthy input coverage"):
            validation_receipts.reuse_checks(
                self.output,
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
        validation.write_json(self.output / "checks.json", receipt)
        gate = check["gate"]
        retained = validation_receipts.reuse_checks(
            self.output, snapshot, environment, receipt["scope"], {gate}, set(), None
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
                    self.output,
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
        validation.write_json(self.output / "checks.json", receipt)
        gate = FUNCTIONAL_SCOPES["preparation"].name
        retained = validation_receipts.reuse_checks(
            self.output,
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
        parent = self.output / "transferred"
        parent.mkdir()
        validation.write_json(parent / "checks.json", {**receipt, "checks": retained})
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

    def test_measurement_identity_includes_declared_cases_without_invalidating_product_claim(
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
        self.assertEqual(
            validation.input_identity("rust-product", before, {}),
            validation.input_identity("rust-product", after, {}),
        )


if __name__ == "__main__":
    unittest.main()

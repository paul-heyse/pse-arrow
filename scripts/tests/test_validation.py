# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Failure collection must work without successful product builds or tests."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

from scripts import case_measure, native_tests, validation, validation_receipts
from scripts.validation_scope import GROUPS, Gate, comprehensive, expand


class ValidationTests(unittest.TestCase):
    def test_native_recipe_preserves_exact_nextest_filter(self) -> None:
        source = (Path(__file__).resolve().parents[2] / "justfile").read_text()
        match = re.search(
            r"(?m)^\[positional-arguments\]\ntest[^\n]*:\n(?:[ \t]+[^\n]*\n)+",
            source,
        )
        self.assertIsNotNone(match)
        if match is None:
            raise AssertionError("missing positional test recipe")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            recipe = match.group().replace(
                "cargo nextest", '"{{ py }}" capture.py nextest'
            )
            (root / "justfile").write_text(
                f'py := "{sys.executable}"\n'
                'nextest_action := "list --message-format json"\n'
                'validate := "--features pse-relations/force-validate"\n' + recipe
            )
            (root / "scripts").mkdir()
            (root / "capture.py").write_text(
                "import json, pathlib, sys\n"
                "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
            )
            selection = "test(=a) or test(=b); $(touch injected)"
            subprocess.run(
                ["just", "test", "--profile", "ci", "-E", selection],
                cwd=root,
                check=True,
                capture_output=True,
            )
            arguments = json.loads((root / "arguments.json").read_text())
            self.assertEqual(arguments[-4:], ["--profile", "ci", "-E", selection])
            self.assertIn("pse-relations/force-validate", arguments)
            self.assertFalse((root / "injected").exists())

    def test_assessment_recipe_preserves_reason_and_output_arguments(self) -> None:
        source = (Path(__file__).resolve().parents[2] / "justfile").read_text()
        match = re.search(
            r"(?m)^\[positional-arguments\]\nassessment[^\n]*:\n(?:[ \t]+[^\n]*\n)+",
            source,
        )
        self.assertIsNotNone(match)
        if match is None:
            raise AssertionError("missing positional assessment recipe")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "justfile").write_text(
                'py := "' + sys.executable + '"\n' + match.group()
            )
            (root / "scripts").mkdir()
            (root / "scripts/validation.py").write_text(
                "import json, pathlib, sys\n"
                "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
            )
            reason = 'literal "quotes"; $HOME `pwd` $(touch injected)'
            subprocess.run(
                ["just", "assessment", "output with spaces", "--change-reason", reason],
                cwd=root,
                check=True,
                capture_output=True,
            )
            self.assertEqual(
                json.loads((root / "arguments.json").read_text()),
                ["--output", "output with spaces", "--change-reason", reason],
            )
            self.assertFalse((root / "injected").exists())

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "evidence"
        self.output.mkdir()

    def test_source_inventory_tracks_additions_deletions_modes_and_symlinks(
        self,
    ) -> None:
        source = self.root / "source.rs"
        source.write_text("fn original() {}")
        names = b"source.rs\0added.rs\0alias\0"
        with patch.object(validation, "git", return_value=names):
            before = validation.sources(self.root)
            source.write_text("fn changed() {}")
            (self.root / "added.rs").write_text("new")
            (self.root / "alias").symlink_to("source.rs")
            after = validation.sources(self.root)
            self.assertEqual(
                validation_receipts.changed(before, after),
                ["added.rs", "alias", "source.rs"],
            )
            source.chmod(0o755)
            executable = validation.sources(self.root)
            self.assertNotEqual(executable["source.rs"], after["source.rs"])
            source.unlink()
            (self.root / "alias").unlink()
            (self.root / "alias").symlink_to("added.rs")
            removed = validation.sources(self.root)
            self.assertNotEqual(removed["source.rs"], executable["source.rs"])
            self.assertNotEqual(removed["alias"], executable["alias"])

    def test_evidence_requires_ignored_contained_paths_and_rejects_symlink_escape(
        self,
    ) -> None:
        with (
            patch.object(
                validation.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1),
            ),
            self.assertRaisesRegex(ValueError, "gitignored"),
        ):
            validation.fresh_output(self.root, self.root / "source-evidence")
        (self.root / "escape").symlink_to(self.root.parent, target_is_directory=True)
        with self.assertRaises(ValueError):
            validation.fresh_output(
                self.root, self.root / "escape/uncontained-evidence"
            )

    def test_failed_process_does_not_prevent_later_checks(self) -> None:
        real_execute = validation.execute

        def execute(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            program = (
                "raise SystemExit(7)"
                if name == "first"
                else "print('later check executed')"
            )
            return real_execute(
                root, output, name, [sys.executable, "-c", program], env
            )

        with patch.object(validation, "execute", side_effect=execute):
            code = validation.run_gates(
                self.root, self.output, [Gate("first"), Gate("last")], capture=False
            )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(code, 1)
        self.assertTrue(receipt["complete"])
        self.assertEqual([check["exit_code"] for check in receipt["checks"]], [7, 0])
        self.assertIn("later check executed", (self.output / "last.log").read_text())

    def test_spawn_failure_is_persistent_and_not_a_success(self) -> None:
        result = validation.execute(
            self.root, self.output, "missing", [str(self.root / "absent")], {}
        )
        self.assertIsNone(result["exit_code"])
        self.assertEqual(result["status"], "failed")
        self.assertTrue(result["spawn_error"])
        self.assertTrue((self.output / "missing.log").read_text())

    def test_malformed_report_does_not_prevent_later_checks(self) -> None:
        real_execute = validation.execute

        def execute(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            result = real_execute(
                root, output, name, [sys.executable, "-c", "pass"], env
            )
            (output / "bad.xml").write_text("<malformed")
            return result

        with patch.object(validation, "execute", side_effect=execute):
            code = validation.run_gates(
                self.root,
                self.output,
                [Gate("first", report="{output}/bad.xml"), Gate("last")],
                capture=False,
            )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(code, 1)
        self.assertEqual(len(receipt["checks"]), 2)
        self.assertTrue(receipt["checks"][0]["report_errors"])
        self.assertEqual(receipt["checks"][1]["status"], "passed")

    def test_stale_report_cannot_be_attributed_to_current_gate(self) -> None:
        report = self.output / "old.xml"
        report.write_text('<testsuite><testcase name="old" /></testsuite>')
        os.utime(report, (1, 1))
        cases, errors = validation.collect_report(
            report, self.output, "check", time.time()
        )
        self.assertEqual(cases, [])
        self.assertTrue(errors)

    def test_native_report_retains_failures_errors_and_skips(self) -> None:
        report = self.output / "native.xml"
        report.write_text(
            '<testsuite><testcase name="pass"/><testcase name="bad"><failure message="cause">details</failure></testcase><testcase name="setup"><error>broken</error></testcase><testcase name="skip"><skipped>reason</skipped></testcase></testsuite>'
        )
        cases, errors = validation.collect_report(report, self.output, "copied", 0)
        self.assertEqual(errors, [])
        self.assertEqual(
            [case["status"] for case in cases],
            ["passed", "failure", "error", "skipped"],
        )
        self.assertIn("details", cases[1]["details"])
        self.assertEqual(report.read_bytes(), (self.output / "copied.xml").read_bytes())

    def test_output_refuses_overwrite(self) -> None:
        with patch.object(validation.subprocess, "run") as run:
            run.return_value.returncode = 0
            with self.assertRaises(FileExistsError):
                validation.fresh_output(self.root, self.output)

    def test_native_report_configuration_preserves_timeouts_and_retries(self) -> None:
        source = Path(__file__).resolve().parents[2] / ".config/nextest.toml"
        (self.root / ".config").mkdir()
        (self.root / ".config/nextest.toml").write_bytes(source.read_bytes())
        copied = validation.native_report_config(self.root, self.output, "native")
        original = tomllib.loads(source.read_text())
        actual = tomllib.loads(copied.read_text())
        original["profile"]["ci"]["junit"]["path"] = str(self.output / "native.xml")
        self.assertEqual(original, actual)

    def test_failed_setup_blocks_only_dependents_and_records_unattempted_work(
        self,
    ) -> None:
        real = validation.execute

        def fake(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command, env
            return real(
                root,
                output,
                name,
                [
                    sys.executable,
                    "-c",
                    "raise SystemExit(1)" if name == "setup" else "pass",
                ],
                {},
            )

        with (
            patch.dict(os.environ, {"PSE_TEST_ENUMERATION": "foreign-selected.txt"}),
            patch.object(validation, "execute", side_effect=fake),
        ):
            code = validation.run_gates(
                self.root,
                self.output,
                [
                    Gate("setup"),
                    Gate("dependent", dependencies=("setup",)),
                    Gate("independent"),
                ],
                capture=False,
            )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(code, 1)
        self.assertEqual(
            [c["status"] for c in receipt["checks"]], ["failed", "blocked", "passed"]
        )
        self.assertFalse((self.output / "dependent.log").exists())

    def test_unsupported_and_advisory_findings_do_not_mask_tool_failure(self) -> None:
        self.assertTrue(
            validation_receipts.qualified({"status": "unsupported", "role": "deferred"})
        )
        self.assertTrue(
            validation_receipts.qualified({"status": "findings", "role": "advisory"})
        )
        self.assertFalse(
            validation_receipts.qualified({"status": "failed", "role": "advisory"})
        )
        self.assertFalse(
            validation_receipts.qualified({"status": "unsupported", "role": "required"})
        )

    def test_declared_totals_and_duplicate_cases_cannot_hide_truncation(self) -> None:
        report = self.output / "counts.xml"
        report.write_text(
            '<testsuite tests="3"><testcase name="one"/><testcase name="one"/></testsuite>'
        )
        cases, errors = validation.collect_report(report, self.output, "counts", 0)
        self.assertEqual(len(cases), 2)
        self.assertEqual(len(errors), 2)

    def test_interrupted_selection_retains_unexecuted_test_identities(self) -> None:
        check = {
            "selected": [
                {"class": "crate", "name": "one"},
                {"class": "crate", "name": "two"},
            ],
            "results": [{"class": "crate", "name": "one", "status": "passed"}],
            "status": "interrupted",
            "report_errors": [],
        }
        validation_receipts.reconcile(check)
        self.assertEqual(check["status"], "interrupted")
        self.assertEqual(check["results"][1]["name"], "two")
        self.assertEqual(check["results"][1]["status"], "not_run")

    def test_nextest_selection_uses_native_filters(self) -> None:
        record = {
            "rust-suites": {
                "crate": {
                    "binary-id": "crate",
                    "testcases": {
                        "one": {"filter-match": {"status": "matches"}},
                        "two": {"filter-match": {"status": "mismatch"}},
                    },
                }
            }
        }
        self.assertEqual(
            validation_receipts.native_selection(
                "compiler output\n" + json.dumps(record)
            ),
            [{"class": "crate", "name": "one"}],
        )
        with self.assertRaises(ValueError):
            validation_receipts.native_selection("truncated")

    def test_tool_receipt_cannot_replace_failure_with_pass(self) -> None:
        validation.write_json(
            self.output / "audit-shear-tool.json",
            {
                "gate": "audit-shear",
                "status": "passed",
                "started": 2,
                "invocations": [{"exit_code": 0}],
            },
        )
        check = {
            "gate": "audit-shear",
            "role": "advisory",
            "exit_code": 2,
            "status": "failed",
            "started": 1,
            "report_errors": [],
            "artifacts": {},
        }
        validation_receipts.classify(check, self.output)
        self.assertEqual(check["status"], "failed")
        self.assertTrue(check["report_errors"])

    def test_interrupt_marks_later_gates_not_run(self) -> None:
        real = validation.execute

        def fake(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            result = real(root, output, name, [sys.executable, "-c", "pass"], env)
            return {**result, "exit_code": -2, "status": "interrupted"}

        with patch.object(validation, "execute", side_effect=fake):
            self.assertEqual(
                validation.run_gates(
                    self.root, self.output, [Gate("first"), Gate("last")], capture=False
                ),
                1,
            )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(
            [c["status"] for c in receipt["checks"]], ["interrupted", "not_run"]
        )
        self.assertFalse(receipt["complete"])

    def test_scope_runs_python_once_and_keeps_strict_checks(self) -> None:
        names = [g.name for g in comprehensive()]
        self.assertEqual(len(names), len(set(names)))
        self.assertTrue(
            {
                "test",
                "native-test",
                "native-python",
                "clippy-default",
                "clippy-no-default",
                "governance-tests",
            }
            <= set(names)
        )
        self.assertNotIn("register-check", names)
        self.assertNotIn("case-measure", names)
        self.assertEqual(
            [
                g
                for g in names
                if "python" in g and g != "native-python" and g.startswith("assessment")
            ],
            [],
        )
        for group in GROUPS:
            leaves = [g.name for g in expand((group,))]
            self.assertEqual(len(leaves), len(set(leaves)))

    def test_strict_policy_does_not_turn_findings_into_success(self) -> None:
        for group, expected in (("deps-report", True), ("policy", False)):
            for gate in expand((group,)):
                self.assertEqual(
                    validation_receipts.qualified(
                        {"status": "findings", "role": gate.role}
                    ),
                    expected,
                )
        combined = expand(("policy", "deps-report"))
        self.assertEqual(
            next(g.role for g in combined if g.name == "audit-advisories"), "required"
        )

    def test_reuse_and_transfer_preserve_original_observation(self) -> None:
        scope = [asdict(Gate("test"))]
        log = self.output / "test.log"
        log.write_text("original execution")
        check = {
            "gate": "test",
            "status": "passed",
            "evidence_kind": "executed",
            "artifacts": {log.name: validation_receipts.digest(log)},
        }
        validation.write_json(
            self.output / "checks.json",
            {
                "version": 4,
                "source_files": {"src": "old"},
                "environment": {},
                "checks": [check],
                "scope": scope,
            },
        )
        use = validation_receipts.reuse_checks
        self.assertEqual(
            use(self.output, {"src": "old"}, {}, scope, set(), set(), None), []
        )
        reused = use(self.output, {"src": "old"}, {}, scope, {"test"}, set(), None)[0]
        self.assertEqual(reused["evidence_kind"], "unchanged-input-reuse")
        with self.assertRaises(ValueError):
            use(self.output, {"src": "new"}, {}, scope, {"test"}, set(), None)
        with self.assertRaises(ValueError):
            use(self.output, {"src": "new"}, {}, scope, set(), {"test"}, "")
        transferred = use(
            self.output,
            {"src": "new"},
            {},
            scope,
            set(),
            {"test"},
            "Reviewed only documentation changes",
        )[0]
        self.assertEqual(transferred["evidence_kind"], "reviewed-transfer")
        self.assertEqual(transferred["changed_inputs"], ["src"])
        log.write_text("tampered")
        with self.assertRaises(ValueError):
            use(self.output, {"src": "new"}, {}, scope, set(), {"test"}, "reviewed")

    def test_native_identity_detects_changed_linked_library(self) -> None:
        library = self.output / "library.so"
        library.write_bytes(b"original")
        native = {
            "schema": "native-profile-v1",
            "files": {str(library): validation_receipts.digest(library)},
            "threads": dict.fromkeys(
                ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"), "1"
            ),
        }
        validation_receipts.verify_native(native)
        library.write_bytes(b"replacement")
        with self.assertRaises(ValueError):
            validation_receipts.verify_native(native)

    def test_csv_samples_use_units_and_iterations(self) -> None:
        source = self.output / "raw.csv"
        header = "sample_measured_value,unit,iteration_count\n"
        source.write_text(header + "100,ns,2\n300,ns,3\n")
        result = case_measure.samples(source)
        self.assertEqual(result["mean_nanoseconds"], 80)
        self.assertEqual(result["sample_count"], 2)
        for rows in (
            "nan,ns,2\n300,ns,3\n",
            "100,ms,2\n300,ns,3\n",
            "100,ns,0\n300,ns,3\n",
            "100,ns,2\n",
        ):
            source.write_text(header + rows)
            with self.assertRaises(ValueError):
                case_measure.samples(source)

    def test_native_command_preserves_filter_and_full_feature_graph(self) -> None:
        selection = "test(=a) or test(=b); $(touch injected)"
        command = native_tests.rust_command("list", ["-E", selection])
        self.assertEqual(command[-2:], ["-E", selection])
        self.assertIn("--workspace", command)
        self.assertIn(
            "pse-relations/force-validate", command[command.index("--features") + 1]
        )

    def test_native_python_refuses_empty_or_skipped_reports_despite_zero_exit(
        self,
    ) -> None:
        path = self.output / "native-python.xml"
        for case, expected in (
            ("", 1),
            ('<testcase name="a"><skipped/></testcase>', 1),
            ('<testcase name="a"/>', 0),
        ):

            def run(*_args: object, case: str = case, **_kwargs: object) -> int:
                path.write_text(f"<testsuite>{case}</testsuite>")
                return 0

            with (
                patch.object(
                    sys, "argv", ["native_tests", "python", f"--junitxml={path}"]
                ),
                patch.dict(os.environ, {}, clear=True),
                patch("subprocess.call", side_effect=run),
            ):
                self.assertEqual(native_tests.main(), expected)
        with (
            patch.object(sys, "argv", ["native_tests", "python"]),
            patch.dict(os.environ, {}, clear=True),
            patch("subprocess.call") as call,
        ):
            with self.assertRaises(ValueError):
                native_tests.main()
            call.assert_not_called()

    def test_empty_nextest_selection_fails(self) -> None:
        with self.assertRaisesRegex(ValueError, "no tests selected"):
            validation_receipts.native_selection(json.dumps({"rust-suites": {}}))


if __name__ == "__main__":
    unittest.main()

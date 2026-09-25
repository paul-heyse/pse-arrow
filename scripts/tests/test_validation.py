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
from pathlib import Path
from unittest.mock import patch

from scripts import validation, validation_cases, validation_receipts
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
            (root / "scripts/implementation_phase.py").write_text("")
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
        (self.root / "Cargo.toml").write_text(
            "[workspace.metadata.pse.execution]\nactive-plan = 14\n"
        )
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

    def test_continuation_refuses_cross_plan_cross_phase_and_changed_ancestors(
        self,
    ) -> None:
        parent = self.root / "parent"
        parent.mkdir()
        prior = {
            "version": 3,
            "plan": 14,
            "mode": "functional",
            "source_files": {},
            "checks": [],
            "parent": None,
        }
        receipt = parent / "checks.json"
        receipt.write_text(json.dumps(prior))
        validation_receipts.continuation(
            parent, {}, set(), None, plan=14, mode="functional"
        )
        for plan, mode in ((11, "functional"), (14, "performance")):
            with self.subTest(plan=plan, mode=mode), self.assertRaises(ValueError):
                validation_receipts.continuation(
                    parent, {}, set(), None, plan=plan, mode=mode
                )
        child = self.root / "child"
        child.mkdir()
        (child / "checks.json").write_text(
            json.dumps(
                {
                    **prior,
                    "parent": {
                        "path": str(parent),
                        "digest": validation_receipts.digest(receipt),
                    },
                }
            )
        )
        validation_receipts.continuation(
            child, {}, set(), None, plan=14, mode="functional"
        )
        receipt.write_text(json.dumps({**prior, "plan": 11}))
        with self.assertRaises(ValueError):
            validation_receipts.continuation(
                child, {}, set(), None, plan=14, mode="functional"
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

    def test_scope_has_no_duplicate_or_cross_environment_checks(self) -> None:
        names = [gate.name for gate in comprehensive()]
        self.assertEqual(len(names), len(set(names)))
        self.assertFalse(
            set(names)
            & {
                "wheels-check",
                "parity",
                "parity-container",
                "solver-rebuild-check",
                "msrv-check",
                "udeps",
                "gh-setup-check",
                "test-release",
                "doctest-release",
                "coverage",
                "features-combinations",
                "features-no-default",
                "clippy-default",
                "clippy-no-default",
                "governance-tests",
            }
        )
        self.assertTrue(
            {
                "test",
                "doctest",
                "plan14-native",
                "plan14-python",
                "assessment-python-unit",
                "assessment-python-component",
                "assessment-python-integration",
            }
            <= set(names)
        )
        for group in GROUPS:
            leaves = [gate.name for gate in expand((group,))]
            self.assertEqual(len(leaves), len(set(leaves)))
            self.assertFalse(set(leaves) & GROUPS.keys())
        self.assertFalse(
            {
                "native-solver-test",
                "native-compiler-solver-test",
                "engineering-inspection",
            }
            & set(names)
        )

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

    def test_continuation_authenticates_logs_and_explicit_source_changes(self) -> None:
        log = self.output / "one.log"
        log.write_text("actual passed invocation\n")
        check = {
            "gate": "one",
            "status": "passed",
            "artifacts": {"one.log": validation_receipts.digest(log)},
        }
        validation.write_json(
            self.output / "checks.json",
            {
                "version": 3,
                "plan": 11,
                "source_files": {"source": "old"},
                "checks": [check],
            },
        )
        with self.assertRaises(ValueError):
            validation_receipts.continuation(
                self.output, {"source": "new"}, set(), None, plan=11
            )
        parent, checks = validation_receipts.continuation(
            self.output, {"source": "old"}, set(), None, plan=11
        )
        self.assertEqual(parent["changed_source"], [])
        self.assertEqual(len(checks), 1)
        log.write_text("replaced\n")
        with self.assertRaises(ValueError):
            validation_receipts.continuation(
                self.output, {"source": "old"}, set(), None, plan=11
            )

    def test_observed_formatting_changes_require_explicit_impact_selection(
        self,
    ) -> None:
        log = self.output / "test.log"
        log.write_text("completed workspace tests")
        check = {
            "gate": "test",
            "status": "passed",
            "changed_source": ["src/lib.rs"],
            "artifacts": {log.name: validation_receipts.digest(log)},
        }
        validation.write_json(
            self.output / "checks.json",
            {
                "version": 3,
                "plan": 14,
                "mode": "functional",
                "source_files": {"src/lib.rs": "before-formatting"},
                "checks": [check],
            },
        )
        current = {"src/lib.rs": "formatted"}
        with self.assertRaises(ValueError):
            validation_receipts.continuation(self.output, current, set(), None, plan=14)
        link, retained = validation_receipts.continuation(
            self.output,
            current,
            {"plan14-native"},
            "reviewed formatting-only edits; exercise new native diagnostic",
            plan=14,
        )
        self.assertEqual(len(retained), 1)
        self.assertEqual(retained[0]["changed_source"], ["src/lib.rs"])
        self.assertEqual(retained[0]["origin"], str(self.output))
        child = self.root / "continued"
        child.mkdir()
        validation.write_json(
            child / "checks.json",
            {
                "version": 3,
                "plan": 14,
                "mode": "functional",
                "source_files": current,
                "checks": retained,
                "parent": link,
            },
        )
        _, verified = validation_receipts.continuation(
            child, current, set(), None, plan=14
        )
        self.assertEqual(verified, retained)
        _, retained = validation_receipts.continuation(
            self.output,
            current,
            {"test"},
            "functional change requires workspace rerun",
            plan=14,
        )
        self.assertEqual(retained, [])

    def test_retained_measurement_allows_only_document_changes(self) -> None:
        log = self.output / "measure.log"
        log.write_text("actual operation samples")
        sources = {
            "docs/plans/14-m22-execution.md": "old",
            "benches/benches/native_process.rs": "same",
        }
        check = {
            "gate": "plan14-measure",
            "status": "passed",
            "artifacts": {log.name: validation_receipts.digest(log)},
        }
        validation.write_json(
            self.output / "checks.json",
            {
                "version": 3,
                "plan": 14,
                "mode": "performance",
                "source_files": sources,
                "checks": [check],
            },
        )
        updated = {**sources, "docs/plans/14-m22-execution.md": "new"}
        _, retained = validation_receipts.continuation(
            self.output, updated, {"plan14-reviews"}, "final documentation", plan=14
        )
        self.assertEqual(len(retained), 1)
        self.assertEqual(retained[0]["origin"], str(self.output))
        for name in [
            "benches/benches/native_process.rs",
            "tests/fixtures/plan14/model.json",
            "scripts/plan14_measure.py",
            "docs/plans/14-acceptance-cases.toml",
            "Cargo.lock",
            ".cargo/config.toml",
        ]:
            with (
                self.subTest(name=name),
                self.assertRaisesRegex(ValueError, "executable inputs changed"),
            ):
                validation_receipts.continuation(
                    self.output,
                    {**updated, name: "changed"},
                    {"plan14-reviews"},
                    "claimed docs update",
                    plan=14,
                )
        _, retained = validation_receipts.continuation(
            self.output,
            {**updated, "Cargo.lock": "changed"},
            {"plan14-measure", "plan14-reviews"},
            "requalify changed executable",
            plan=14,
        )
        self.assertEqual(retained, [])

    def test_functional_scope_defers_benchmarks_and_includes_threaded_python(
        self,
    ) -> None:
        functional = {gate.name for gate in comprehensive()}
        self.assertEqual(
            {gate.name for gate in comprehensive("performance")},
            {"plan14-measure", "plan14-reviews"},
        )
        self.assertIn("assessment-python-integration", functional)
        self.assertFalse(any(name.startswith("bench-") for name in functional))

    def test_measurement_checkpoint_is_incomplete_and_resumes_without_remeasurement(
        self,
    ) -> None:
        real = validation.execute
        invoked = []

        def execute(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command, env
            invoked.append(name)
            return real(root, output, name, [sys.executable, "-c", "pass"], {})

        gates = [Gate("plan14-measure"), Gate("plan14-reviews")]
        with (
            patch.object(validation, "execute", side_effect=execute),
            patch.object(validation_receipts, "classify"),
            patch.object(
                validation.implementation_phase,
                "require_functional",
                return_value={"path": str(self.output)},
            ),
        ):
            code = validation.run_gates(
                self.root,
                self.output,
                gates,
                capture=False,
                phase="performance",
                stop_after="plan14-measure",
            )
            receipt = json.loads((self.output / "checks.json").read_text())
            self.assertEqual(code, 0)
            self.assertEqual(invoked, ["plan14-measure"])
            self.assertEqual(receipt["stopped_after"], "plan14-measure")
            self.assertFalse(receipt["complete"])
            self.assertFalse(receipt["required_checks_covered"])
            self.assertEqual(
                [c["status"] for c in receipt["checks"]], ["passed", "not_run"]
            )
            continuation = self.root / "continued"
            continuation.mkdir()
            code = validation.run_gates(
                self.root,
                continuation,
                gates,
                capture=False,
                phase="performance",
                resume_from=self.output,
            )
        self.assertEqual(code, 0)
        self.assertEqual(invoked, ["plan14-measure", "plan14-reviews"])
        receipt = json.loads((continuation / "checks.json").read_text())
        self.assertTrue(receipt["complete"])
        self.assertTrue(receipt["required_checks_covered"])
        self.assertIsNone(receipt["stopped_after"])

    def test_measurement_checkpoint_cannot_skip_functional_work(self) -> None:
        with self.assertRaisesRegex(ValueError, "performance measurement"):
            validation.run_gates(
                self.root,
                self.output,
                [Gate("plan14-measure")],
                capture=False,
                stop_after="plan14-measure",
            )

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

    def test_case_coverage_requires_current_binary_and_all_parameters(self) -> None:
        case = {
            "id": "m20.example",
            "binary": "new",
            "tests": ["check[first]", "check[second]"],
            "mode": "force-validate",
            "profile": "rust-native",
            "runner": "nextest",
            "source": "test.rs",
        }
        declaration = {
            "plan": 14,
            "cases": [case],
            "acceptance": [
                {"id": f"Q{i:02}", "cases": [case["id"]]} for i in range(1, 18)
            ],
        }
        result = {"class": "new", "name": "check[first]", "status": "passed"}
        check = {
            "gate": "test",
            "mode": "force-validate",
            "profile": "rust-native",
            "status": "passed",
            "results": [result, {**result, "name": "check[second]"}],
        }
        self.assertTrue(
            validation_cases.coverage(declaration, [check], "functional")["complete"]
        )
        for change in (
            {"mode": "production"},
            {"profile": "other"},
            {"results": [result]},
            {"results": [result, result]},
            {"results": [{**result, "status": "skipped"}]},
        ):
            with self.subTest(change=change):
                self.assertFalse(
                    validation_cases.coverage(
                        declaration, [{**check, **change}], "functional"
                    )["complete"]
                )
        declaration["acceptance"] = []
        report = validation_cases.coverage(declaration, [check], "functional")
        self.assertEqual(
            report["missing_obligations"], [f"Q{i:02}" for i in range(1, 18)]
        )
        self.assertFalse(report["complete"])

    def test_aggregate_obligations_use_current_phase_and_actual_leaf_gates(
        self,
    ) -> None:
        declaration = {
            "plan": 14,
            "cases": [],
            "acceptance": [
                {"id": "Q18", "phase": "performance", "cases": ["m22.cost"]}
            ],
        }
        report = validation_cases.coverage(declaration, [], "functional")
        self.assertNotIn("Q18", report["obligations"])
        self.assertNotIn("Q18", report["missing_obligations"])
        self.assertFalse(
            validation_cases.coverage(declaration, [], "performance")["complete"]
        )


if __name__ == "__main__":
    unittest.main()

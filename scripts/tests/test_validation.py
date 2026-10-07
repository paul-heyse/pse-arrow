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
from scripts.validation_scope import (
    FUNCTIONAL_SCOPES,
    GROUPS,
    INPUT_SCOPES,
    Gate,
    comprehensive,
    expand,
    native_gate,
)


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
            # Argument forwarding is the scope; native lifecycle is exercised by
            # its own operation controls rather than invoking setup in this fixture.
            (root / "scripts/native_exec.sh").write_text('exec "$@"\n')
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

    def test_publication_inputs_include_root_pages_and_library_collections(
        self,
    ) -> None:
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        page = self.root / "docs/README.md"
        library = self.root / "docs/capability-maps/new-map.md"
        library.parent.mkdir(parents=True)
        page.write_text("old page")
        subprocess.run(["git", "-C", str(self.root), "add", "docs"], check=True)
        before = validation.sources(self.root)
        page.write_text("new page")
        library.write_text("new consumed collection")
        after = validation.sources(self.root)
        self.assertEqual(
            validation_receipts.changed(before, after),
            ["docs/README.md", "docs/capability-maps/new-map.md"],
        )
        self.assertNotEqual(
            validation.input_identity("documentation", before, {}),
            validation.input_identity("documentation", after, {}),
        )
        subprocess.run(["git", "-C", str(self.root), "add", "docs"], check=True)
        page.unlink()
        library.unlink()
        deleted = validation.sources(self.root)
        self.assertEqual(
            validation_receipts.changed(after, deleted),
            ["docs/README.md", "docs/capability-maps/new-map.md"],
        )

    def test_declared_inputs_and_agent_policies_are_covered_without_local_credentials(
        self,
    ) -> None:
        for scope, prefixes in INPUT_SCOPES.items():
            for prefix in prefixes:
                with self.subTest(scope=scope, prefix=prefix):
                    self.assertTrue(
                        any(
                            prefix == captured or prefix.startswith(captured + "/")
                            for captured in validation.SOURCE_PATHS
                        )
                    )
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / ".gitignore").write_text(
            ".claude/settings.local.json\n.codex/auth.json\n"
        )
        policies = (
            ".claude/settings.json",
            ".codex/hooks.json",
            ".agents/roles/executor.md",
        )
        credentials = (".claude/settings.local.json", ".codex/auth.json")
        for name in (*policies, *credentials):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("original")
        before = validation.sources(self.root)
        self.assertTrue(all(name in before for name in policies))
        self.assertTrue(all(name not in before for name in credentials))
        for name in policies:
            (self.root / name).write_text("changed policy")
        after = validation.sources(self.root)
        self.assertEqual(validation_receipts.changed(before, after), sorted(policies))
        self.assertNotEqual(
            validation.input_identity("tooling", before, {}),
            validation.input_identity("tooling", after, {}),
        )
        (self.root / ".claude/settings.local.json").write_text("changed credential")
        self.assertEqual(after, validation.sources(self.root))

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
        for profile in ("local", "ci"):
            with self.subTest(profile=profile):
                copied = validation.native_report_config(
                    self.root, self.output, f"native-{profile}", profile
                )
                original = tomllib.loads(source.read_text())
                actual = tomllib.loads(copied.read_text())
                original["profile"][profile]["junit"]["path"] = str(
                    self.output / f"native-{profile}.xml"
                )
                # Isolation may change only the selected report destination;
                # production task limits and inherited runner settings stay owned
                # by the source configuration, including the local override.
                self.assertEqual(original, actual)

    def test_selected_local_and_manual_ci_reports_are_isolated_and_collected(
        self,
    ) -> None:
        source = Path(__file__).resolve().parents[2] / ".config/nextest.toml"
        (self.root / ".config").mkdir()
        (self.root / ".config/nextest.toml").write_bytes(source.read_bytes())
        commands: dict[str, list[str]] = {}
        real_execute = validation.execute

        def execute(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            commands[name] = command
            result = real_execute(
                root, output, name, [sys.executable, "-c", "pass"], env
            )
            (output / f"{name}.xml").write_text(
                f'<testsuite><testcase classname="fixture" name="{name}"/></testsuite>'
            )
            return result

        gates = [
            Gate(
                f"selected-{profile}",
                ("--profile", profile),
                f"{{target}}/nextest/{profile}/junit.xml",
                recipe="test",
                profile=profile,
            )
            for profile in ("local", "ci")
        ]
        with patch.object(validation, "execute", side_effect=execute):
            code = validation.run_gates(self.root, self.output, gates, capture=False)
        self.assertEqual(code, 0)
        checks = json.loads((self.output / "checks.json").read_text())["checks"]
        for gate, check in zip(gates, checks, strict=True):
            command = commands[gate.name]
            self.assertEqual(command[command.index("--profile") + 1], gate.profile)
            config = Path(command[command.index("--config-file") + 1])
            selected = tomllib.loads(config.read_text())["profile"][gate.profile]
            self.assertEqual(
                selected["junit"]["path"], str(self.output / f"{gate.name}.xml")
            )
            self.assertEqual(check["profile"], gate.profile)
            self.assertEqual(check["results"][0]["name"], gate.name)
            self.assertEqual(check["report_errors"], [])

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
                "feature-absence",
                "native-test",
                "native-python",
                "clippy-default",
                "clippy-no-default",
            }
            <= set(names)
        )
        self.assertNotIn("test", names)
        self.assertNotIn("governance-tests", names)
        self.assertFalse(
            next(g for g in comprehensive() if g.name == "native-test").enumerate_native
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

    def test_python_producer_profile_keeps_native_dev_graph(self) -> None:
        gates = {gate.name: gate for gate in comprehensive("producer")}
        self.assertEqual(gates["py-sync-native"].args, ("producer",))
        self.assertEqual(gates["py-sync-native"].profile, "producer")
        self.assertEqual(
            gates["native-python"].dependencies,
            ("py-sync-native", "python-deployment-association"),
        )
        self.assertEqual(
            gates["native-test"].args,
            next(g for g in comprehensive() if g.name == "native-test").args,
        )
        self.assertEqual(
            gates["native-test"].profile,
            next(g for g in comprehensive() if g.name == "native-test").profile,
        )

    def test_producer_deployment_is_ordered_after_install_before_consumers(
        self,
    ) -> None:
        gates = comprehensive("producer")
        positions = {gate.name: index for index, gate in enumerate(gates)}
        for prerequisite, consumer in (
            ("py-sync-native", "producer-deployment"),
            ("producer-deployment", "python-deployment-association"),
            ("python-deployment-association", "native-test"),
            ("python-deployment-association", "native-python"),
        ):
            self.assertLess(positions[prerequisite], positions[consumer])
            self.assertIn(prerequisite, gates[positions[consumer]].dependencies)
        self.assertNotIn("producer-deployment", {gate.name for gate in comprehensive()})

    def test_deployment_step_cannot_pass_without_current_artifacts(self) -> None:
        real = validation.execute

        def child(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            return real(root, output, name, [sys.executable, "-c", "pass"], env)

        with patch.object(validation, "execute", side_effect=child):
            self.assertEqual(
                validation.run_gates(
                    self.root,
                    self.output,
                    [
                        Gate("producer-deployment"),
                        Gate("consumer", dependencies=("producer-deployment",)),
                    ],
                    capture=False,
                ),
                1,
            )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(
            [item["status"] for item in receipt["checks"]], ["failed", "blocked"]
        )
        self.assertIn("without runtime.json", receipt["checks"][0]["report_errors"][0])

    def test_deployment_prerequisites_require_fresh_execution(self) -> None:
        for name in ("producer-deployment", "python-deployment-association"):
            for mode in ("reuse", "transfer"):
                with (
                    self.subTest(name=name, mode=mode),
                    self.assertRaisesRegex(ValueError, "require fresh execution"),
                ):
                    validation.run_gates(
                        self.root,
                        self.output,
                        [Gate(name)],
                        capture=False,
                        reuse_from=self.root / "prior",
                        reuse=(name,) if mode == "reuse" else (),
                        transfer=(name,) if mode == "transfer" else (),
                    )

    def test_review_context_contents_enter_deployment_input_identity(self) -> None:
        context = self.root / "review.json"
        key = "PSE_RUNTIME_PRODUCER_DECLARATIONS"
        context.write_text('{"native_abi":"first"}')
        before = validation.relevant_environment({key: str(context)})
        context.write_text('{"native_abi":"changed"}')
        after = validation.relevant_environment({key: str(context)})
        self.assertNotEqual(
            validation.input_identity("deployment-capture", {}, before),
            validation.input_identity("deployment-capture", {}, after),
        )
        source = {"scripts/producer_deployment.py": "changed"}
        self.assertEqual(
            validation.input_identity("deployment-capture", source, {})["files"], source
        )

    def test_reused_fixture_consumer_uses_verified_origin(self) -> None:
        origin = self.root / "prior"
        observed = {}
        real = validation.execute

        def child(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            observed.update(env)
            return real(root, output, name, [sys.executable, "-c", "pass"], env)

        retained = {
            "gate": "producer-fixture",
            "origin": str(origin),
            "status": "passed",
            "exit_code": 0,
            "elapsed_seconds": 0,
            "log": "fixture.log",
            "results": [],
            "report_errors": [],
        }
        with (
            patch.object(validation_receipts, "digest", return_value="hash"),
            patch.object(validation_receipts, "reuse_checks", return_value=[retained]),
            patch.object(validation, "execute", side_effect=child),
        ):
            validation.run_gates(
                self.root,
                self.output,
                [
                    Gate("producer-fixture"),
                    Gate("consumer", dependencies=("producer-fixture",)),
                ],
                capture=False,
                reuse_from=origin,
                reuse=("producer-fixture",),
            )
        self.assertEqual(
            observed["PSE_PRODUCER_FIXTURE_RECEIPT"],
            str(origin / "producer-fixture.json"),
        )

    def test_association_observation_is_output_until_python_emits_header(self) -> None:
        real = validation.execute
        observed = {}

        def child(
            root: Path, output: Path, name: str, command: list[str], env: dict[str, str]
        ) -> dict:
            del command
            observed.update(env)
            return real(root, output, name, [sys.executable, "-c", "pass"], env)

        gates = [
            Gate("producer-deployment"),
            Gate("python-deployment-association", recipe="native-python"),
        ]
        with patch.object(validation, "execute", side_effect=child):
            self.assertEqual(
                validation.run_gates(self.root, self.output, gates, capture=False), 1
            )
        self.assertNotIn("PSE_PYTHON_DEPLOYMENT_ATTESTATION", observed)
        self.assertTrue(
            observed["PSE_PYTHON_DEPLOYMENT_OBSERVATION_OUTPUT"].endswith(
                "python-attestation.json"
            )
        )
        receipt = json.loads((self.output / "checks.json").read_text())
        self.assertEqual(receipt["checks"][1]["status"], "failed")

    def test_native_fixture_precedes_every_fixture_receipt_consumer(self) -> None:
        gates = comprehensive("producer")
        positions = {gate.name: index for index, gate in enumerate(gates)}
        for consumer in ("feature-absence", "native-test"):
            gate = gates[positions[consumer]]
            self.assertLess(positions["producer-fixture"], positions[consumer])
            self.assertIn("producer-fixture", gate.dependencies)
            self.assertEqual(gate.profile, "local")
            self.assertEqual(gate.args[:2], ("--profile", "local"))
            self.assertEqual(gate.report, "{target}/nextest/local/junit.xml")

    def test_assessment_routes_each_deployment_receipt_to_its_consumer(self) -> None:
        observed: dict[str, str] = {}
        execute = validation.execute

        def child(
            root: Path,
            output: Path,
            name: str,
            _command: list[str],
            environment: dict[str, str],
        ) -> dict:
            observed[name] = environment["PSE_PRODUCER_RECEIPT"]
            return execute(
                root, output, name, [sys.executable, "-c", "pass"], environment
            )

        with (
            patch.dict(
                os.environ,
                {
                    "PSE_WORKER_PRODUCER_RECEIPT": "/deployment/worker.json",
                    "PSE_PYTHON_PRODUCER_RECEIPT": "/deployment/python.json",
                },
            ),
            patch.object(validation, "execute", side_effect=child),
        ):
            validation.run_gates(
                self.root,
                self.output,
                [
                    Gate("worker", recipe="native-test"),
                    Gate("python", recipe="native-python"),
                ],
                capture=False,
            )
        self.assertEqual(
            observed,
            {"worker": "/deployment/worker.json", "python": "/deployment/python.json"},
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
            "invocation": scope[0],
            "status": "passed",
            "evidence_kind": "executed",
            "artifacts": {log.name: validation_receipts.digest(log)},
            "inputs": validation.input_identity("unknown", {"src": "old"}, {}),
        }
        validation.write_json(
            self.output / "checks.json",
            {
                "version": 5,
                "source_files": {"src": "old"},
                "input_coverage": True,
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

    def test_scoped_inputs_exclude_prose_but_keep_relevant_file_identity(self) -> None:
        before = {
            "crates/pse-runtime/src/lib.rs": "a",
            "Cargo.lock": "a",
            "docs/plans/plan.md": "a",
        }
        original = validation.input_identity(
            "rust-product", before, {"IPOPT_DIR": "a", "SCCACHE_DIR": "cache-a"}
        )
        prose = validation.input_identity(
            "rust-product",
            {**before, "docs/plans/plan.md": "b"},
            {"IPOPT_DIR": "a", "SCCACHE_DIR": "cache-b"},
        )
        self.assertEqual(original, prose)
        for name in (
            "Cargo.lock",
            ".config/sccache.toml",
            "crates/new/data.txt",
            "crates/pse-runtime/src/lib.rs",
        ):
            after = validation.input_identity(
                "rust-product", {**before, name: "b"}, {"IPOPT_DIR": "a"}
            )
            self.assertNotEqual(original, after)
        del before["Cargo.lock"]
        self.assertNotEqual(
            original,
            validation.input_identity("rust-product", before, {"IPOPT_DIR": "a"}),
        )
        self.assertNotEqual(
            original,
            validation.input_identity("rust-product", before, {"IPOPT_DIR": "b"}),
        )
        # Gate routing is selected by these inputs even when the generic receipt
        # variable is absent; changing either deployment must invalidate reuse.
        for key in ("PSE_WORKER_PRODUCER_RECEIPT", "PSE_PYTHON_PRODUCER_RECEIPT"):
            for scope in ("rust-product", "python-product"):
                self.assertNotEqual(
                    validation.input_identity(scope, before, {key: "first.json"}),
                    validation.input_identity(scope, before, {key: "second.json"}),
                )
        self.assertEqual(
            validation.input_identity("unknown", before, {})["files"], before
        )

    def test_inventory_is_captured_twice_and_only_relevant_drift_fails(self) -> None:
        real = validation.execute
        for path, expected in (("docs/plans/concurrent.md", 0), ("Cargo.lock", 1)):
            output = self.output / path.split("/")[-1]
            output.mkdir()
            before = {"Cargo.lock": "a", "docs/plans/concurrent.md": "a"}
            after = {**before, path: "b"}

            def execute(
                root: Path,
                output: Path,
                name: str,
                _command: list[str],
                env: dict[str, str],
            ) -> dict:
                return real(root, output, name, [sys.executable, "-c", "pass"], env)

            with (
                patch.object(
                    validation,
                    "provenance",
                    return_value=(before, self.root / "target", []),
                ),
                patch.object(validation, "sources", return_value=after) as inventory,
                patch.object(validation, "execute", side_effect=execute),
            ):
                result = validation.run_gates(
                    self.root, output, [Gate("pure", input_scope="rust-product")]
                )
            self.assertEqual(
                inventory.call_count, 1
            )  # provenance owns the initial capture
            self.assertEqual(result, expected)
            receipt = json.loads((output / "checks.json").read_text())
            self.assertEqual(receipt["contextual_changes"], [path])
            self.assertEqual(
                receipt["checks"][0]["status"], "passed" if expected == 0 else "failed"
            )

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

    def test_case_selection_preserves_default_and_refuses_unknown_identity(
        self,
    ) -> None:
        declarations = (
            {"workloads": [{"id": "process"}, {"id": "durable"}]},
            {"workloads": [{"id": "preparation"}]},
        )
        self.assertEqual(len(case_measure.selected_workloads(declarations, [])), 3)
        self.assertEqual(
            case_measure.selected_workloads(declarations, ["durable", "durable"]),
            [{"id": "durable"}],
        )
        with self.assertRaisesRegex(ValueError, "unknown measurement cases"):
            case_measure.selected_workloads(declarations, ["absent"])
        with self.assertRaisesRegex(ValueError, "identities must be unique"):
            case_measure.selected_workloads((declarations[0], declarations[0]), [])

    def test_k4_smoke_selectors_build_and_dispatch_only_the_preparation_target(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        process = json.loads((root / ".config/process-cases.json").read_text())
        preparation = json.loads((root / ".config/preparation-cases.json").read_text())
        identities = ["k4-demand-orders-selected-output", "k4-worker-permit-lifetime"]
        selected = case_measure.selected_workloads((process, preparation), identities)
        self.assertEqual({w["id"] for w in selected}, set(identities))
        self.assertTrue(all(w["model"] == "scalar-k4" and w["smoke"] for w in selected))
        artifact = json.dumps(
            {
                "reason": "compiler-artifact",
                "target": {"name": "modeling_preparation"},
                "executable": "/bench/modeling_preparation",
            }
        )
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(
                case_measure.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, stdout=artifact),
            ) as run,
            patch.object(case_measure, "native_provenance", return_value={}),
        ):
            case_measure.smoke_campaign(
                Path(directory), "dev", process, preparation, set(identities)
            )
            calls = run.call_args_list
            self.assertEqual(len(calls), 3)
            build = calls[0].args[0]
            self.assertEqual(
                build[build.index("--bench") + 1 :], ["modeling_preparation"]
            )
            self.assertEqual(
                {call.kwargs["env"]["PSE_PREPARATION_CASE"] for call in calls[1:]},
                set(identities),
            )
            self.assertTrue(
                all(
                    call.args[0] == ["/bench/modeling_preparation", "--test"]
                    for call in calls[1:]
                )
            )
            receipt = json.loads((Path(directory) / "case-smoke.json").read_text())
            self.assertFalse(receipt["measured"])

    def test_case_build_failure_retains_json_and_surfaces_rendered_diagnostics(
        self,
    ) -> None:
        stdout = (
            json.dumps(
                {
                    "reason": "compiler-message",
                    "message": {"rendered": "error[E0599]: no method named keys\n"},
                }
            )
            + "\n"
        )
        stderr = "error: could not compile pse-benches\n"
        with patch.object(
            case_measure.subprocess,
            "run",
            return_value=subprocess.CompletedProcess(
                ["cargo"], 101, stdout=stdout, stderr=stderr
            ),
        ) as run:
            with self.assertRaisesRegex(RuntimeError, "error\\[E0599\\].*keys"):
                case_measure.build_benchmark(["cargo", "bench"], self.output, "smoke")
            self.assertFalse(run.call_args.kwargs["check"])
        self.assertEqual((self.output / "smoke-cargo.jsonl").read_text(), stdout)
        self.assertEqual((self.output / "smoke-cargo.stderr.log").read_text(), stderr)

    def test_native_command_preserves_filter_and_full_feature_graph(self) -> None:
        selection = "test(=a) or test(=b); $(touch injected)"
        gate = native_gate("selected-native", selection)
        command = native_tests.rust_command("list", list(gate.args))
        self.assertEqual(command[-len(gate.args) :], list(gate.args))
        self.assertEqual(command[-2:], ["-E", selection])
        self.assertTrue(
            all(gate.profile == "local" for gate in FUNCTIONAL_SCOPES.values())
        )
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
            (
                '<testcase name="a"><properties><property name="nodeid" value="node"/></properties></testcase>',
                0,
            ),
        ):

            def run(*_args: object, case: str = case, **_kwargs: object) -> int:
                path.write_text(f"<testsuite>{case}</testsuite>")
                path.with_name(path.stem + "-selected.txt").write_text("node\n")
                return 0

            with (
                patch.object(
                    sys, "argv", ["native_tests", "python", f"--junitxml={path}"]
                ),
                patch.dict(os.environ, {}, clear=True),
                patch.object(
                    native_tests,
                    "native_provenance",
                    return_value={"files": {"/owned/extension.so": "digest"}},
                ),
                patch.object(
                    native_tests,
                    "python_native_binary",
                    return_value=Path("/owned/extension.so"),
                ),
                patch.object(native_tests.time, "time", return_value=1),
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

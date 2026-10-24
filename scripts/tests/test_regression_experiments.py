# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real Git traversal with bounded fixtures; no compiler/performance qualification."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict
from unittest.mock import patch

from scripts import (
    case_measure,
    native_operation,
    native_tests,
    run_inspection,
    test_resources,
    test_run,
)
from scripts import regression_experiments as driver

if TYPE_CHECKING:
    from collections.abc import Mapping


class RegressionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / "main"
        self.repo.mkdir()
        self.git("init", "--quiet")
        self.git("config", "user.name", "Regression fixture")
        self.git("config", "user.email", "fixture@invalid")
        self.predicate = self.repo / driver.TEST_SOURCE
        self.predicate.parent.mkdir(parents=True)
        self.predicate.write_text("fixed semantic predicate\n")
        self.commits = []
        for index in range(9):
            self.predicate.write_text(
                "unsupported\n" if index == 4 else "fixed semantic predicate\n"
            )
            (self.repo / "value").write_text("bad" if index >= 6 else "good")
            (self.repo / "revision").write_text(str(index))
            self.git("add", ".")
            self.git("commit", "--quiet", "-m", f"revision {index}")
            self.commits.append(self.git("rev-parse", "HEAD"))
        (self.repo / "dirty").write_text("preserved current work")
        self.before = self.git("status", "--porcelain")
        self.head = self.git("rev-parse", "HEAD")

    def git(self, *args: str) -> str:
        return subprocess.check_output(
            ["git", *args], cwd=self.repo, text=True, stderr=subprocess.DEVNULL
        ).strip()

    def frozen(self, output: Path) -> Path:
        controller = output / "controller"
        (controller / "scripts").mkdir(parents=True)
        predicate = controller / driver.TEST_SOURCE
        predicate.parent.mkdir(parents=True)
        predicate.write_text("fixed semantic predicate\n")
        # Replace only the compiler adapter. The public CLI, real product composer,
        # frozen result reader and Git evaluator remain the production code.
        code = f"""
import os, sys
from pathlib import Path
sys.path.insert(0, {str(driver.ROOT)!r})
from scripts import regression_experiments as driver, test_resources, test_run

def bounded_trial(checkout, output):
    value = (checkout / 'value').read_text()
    if value == 'infrastructure':
        raise OSError('deliberate unavailable compiler')
    if value == 'wait':
        (Path(os.environ['FIXTURE_READY'])).write_text(str(os.getpid()))
        import time
        time.sleep(300)
    if value == 'mixed':
        value = 'good' if output.name == '0' else 'bad'
    path = test_resources.invocation('rust', [{{'binary_id': driver.BINARY, 'name': driver.TEST_NAME}}])
    test_run.finish(path, {{
        'exit_code': 0 if value == 'good' else 100,
        'results': [{{'class': driver.BINARY, 'name': driver.TEST_NAME, 'status': 'passed' if value == 'good' else 'failure'}}],
        'report_errors': [], 'owned_drain': 'complete',
    }}, 'runner')
    from scripts import run_inspection
    result = run_inspection.product_result(str(path))
    result['predicate_execution'] = {{'failure_type': None if value == 'good' else 'test failure with exit code 101'}}
    return result
driver.rust_trial = bounded_trial
raise SystemExit(driver.main())
"""
        (controller / "scripts/regression_experiments.py").write_text(code)
        return controller

    @staticmethod
    def fixture_run(command: list[str], cwd: Path, log: Path) -> tuple[int, dict]:
        observation: dict = {}
        with log.open("w") as stream:
            code = native_operation.run(
                command,
                dict(os.environ),
                cwd=cwd,
                observation=observation,
                stdout=stream.fileno(),
                stderr=subprocess.STDOUT,
            )
        # This bounded fixture has no compiler/service descendants. Actual runs
        # use run_child_scope's authenticated cgroup drain, not this fixture.
        observation["owned_drain"] = "complete"
        return code, observation

    def search(self, *, repetitions: int = 1) -> tuple[int, dict]:
        output = self.root / f"experiment-{time.time_ns()}"
        args = argparse.Namespace(
            repo=self.repo,
            good=self.commits[0],
            bad=self.commits[-1],
            output=output,
            repetitions=repetitions,
            cleanup=True,
        )
        with (
            patch.object(driver, "freeze_controller", side_effect=self.frozen),
            patch.object(driver, "run_owned", side_effect=self.fixture_run),
            patch.dict(
                os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}
            ),
        ):
            result = driver.bisect(args)
        self.assertEqual(self.git("status", "--porcelain"), self.before)
        self.assertEqual(self.git("rev-parse", "HEAD"), self.head)
        self.assertEqual((self.repo / "dirty").read_text(), "preserved current work")
        self.assertFalse((output / "checkout").exists())
        return result

    def test_real_history_visits_unsupported_revision_and_preserves_dirty_main(
        self,
    ) -> None:
        code, result = self.search()
        self.assertEqual(code, 0)
        self.assertEqual(result["status"], "narrowed")
        self.assertEqual(result["causal_lead"], self.commits[6])
        predicates = [
            json.loads(Path(path).read_text()) for path in result["predicate_results"]
        ]
        self.assertIn(125, [item["code"] for item in predicates])
        for predicate in predicates:
            for trial in predicate["trials"]:
                self.assertTrue(trial["result"]["invocation"])
                self.assertEqual(trial["subject"], driver.SUBJECT)

    def rewrite_revision(self, index: int, value: str) -> None:
        self.git("checkout", "--quiet", "--detach", self.commits[index])
        (self.repo / "value").write_text(value)
        self.git("add", "value")
        self.git("commit", "--quiet", "-m", value)
        self.commits[-1] = self.git("rev-parse", "HEAD")
        self.head = self.commits[-1]
        self.before = self.git("status", "--porcelain")

    def test_infrastructure_endpoint_aborts_instead_of_bad_or_skip(self) -> None:
        self.rewrite_revision(8, "infrastructure")
        code, result = self.search()
        self.assertEqual(code, 128)
        self.assertEqual(result["status"], "aborted")
        self.assertNotIn("causal_lead", result)

    def test_bounded_mixed_endpoint_is_inconclusive(self) -> None:
        self.rewrite_revision(8, "mixed")
        code, result = self.search(repetitions=2)
        self.assertEqual(code, 125)
        self.assertEqual(result["status"], "inconclusive")
        self.assertNotIn("causal_lead", result)

    def test_skipped_boundary_retains_ambiguous_set(self) -> None:
        # Boundary is exactly the unsupported revision, so no unique blame.
        self.git("checkout", "--quiet", "--detach", self.commits[4])
        (self.repo / "value").write_text("bad")
        self.git("add", ".")
        self.git("commit", "--quiet", "-m", "unknown first bad")
        unknown = self.git("rev-parse", "HEAD")
        self.predicate.write_text("fixed semantic predicate\n")
        self.git("add", ".")
        self.git("commit", "--quiet", "-m", "supported bad")
        self.commits[-1] = self.git("rev-parse", "HEAD")
        self.head = self.commits[-1]
        self.before = self.git("status", "--porcelain")
        code, result = self.search()
        self.assertEqual(code, 125)
        self.assertEqual(result["status"], "inconclusive")
        self.assertIn(unknown, result["possible_commits"])
        self.assertNotIn("causal_lead", result)

    def test_result_protocol_never_infers_bad_from_missing_error_or_wrong_subject(
        self,
    ) -> None:
        path = test_resources.invocation
        with patch.dict(
            os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}
        ):
            invocation = path(
                "rust", [{"binary_id": driver.BINARY, "name": driver.TEST_NAME}]
            )
            test_run.finish(
                invocation,
                {
                    "exit_code": 100,
                    "results": [
                        {
                            "class": driver.BINARY,
                            "name": driver.TEST_NAME,
                            "status": "error",
                        }
                    ],
                    "report_errors": [],
                },
                "runner",
            )
            result = run_inspection.product_result(str(invocation))
        self.assertEqual(driver.classify_product(result), 128)
        result["report_rows"][0]["status"] = "failure"
        result["predicate_execution"] = {
            "failure_type": "test failure with exit code 101"
        }
        self.assertEqual(driver.classify_product(result), 1)
        result["report_rows"][0]["name"] = "other_test"
        self.assertEqual(driver.classify_product(result), 128)

    def test_structured_failure_variants_refuse_crashes_timeouts_and_reruns(
        self,
    ) -> None:
        report = self.root / "report.xml"
        for variant in (
            "test failure with exit code 101",
            "test abort",
            "test timeout",
            "test failure with exit code 101 (leaked handles)",
        ):
            report.write_text(
                f'<testsuites><testsuite><testcase classname="{driver.BINARY}" name="{driver.TEST_NAME}"><failure type="{variant}" /></testcase></testsuite></testsuites>'
            )
            result = {"artifacts": {report.name: driver.digest(report)}}
            self.assertEqual(
                driver.predicate_execution(result, report)["failure_type"], variant
            )
        report.write_text(
            report.read_text().replace("</testcase>", "<rerunFailure /></testcase>")
        )
        with self.assertRaisesRegex(ValueError, "one attempt"):
            driver.predicate_execution(
                {"artifacts": {report.name: driver.digest(report)}}, report
            )
        with self.assertRaisesRegex(ValueError, "frozen product artifact"):
            driver.predicate_execution({"artifacts": {report.name: "wrong"}}, report)

    def test_real_rust_adapter_uses_selected_preparation_and_existing_terminal_owner(
        self,
    ) -> None:
        checkout = self.root / "candidate"
        (checkout / ".config").mkdir(parents=True)
        (checkout / ".config/nextest.toml").write_text("[profile.default]\n")
        output = self.root / "trial"
        output.mkdir()

        def capture(
            command: list[str],
            provenance: Path,
            selection: Path,
            environment: Mapping[str, str],
            **options: object,
        ) -> tuple[list[str], Path | None]:
            self.assertEqual(options, {"effects": "native-local"})
            self.assertEqual(native_tests.ROOT, checkout)
            self.assertIn("--locked", command)
            self.assertEqual(command[-1], f"test(={driver.TEST_NAME})")
            self.assertIn("RUST_MIN_STACK", environment)
            inventory = {
                "rust-suites": {
                    "selected": {
                        "binary-id": driver.BINARY,
                        "binary-path": "/bounded/fake-compiler-output",
                        "testcases": {
                            driver.TEST_NAME: {"filter-match": {"status": "matches"}}
                        },
                    }
                }
            }
            selection.write_text(json.dumps(inventory))
            provenance.write_text("{}")
            return ["--retries", "0"], None

        def execute(
            command: list[str],
            environment: Mapping[str, str],
            *,
            cwd: Path,
            observation: dict[str, object],
        ) -> int:
            self.assertEqual(cwd, checkout)
            self.assertEqual(environment[test_run.EXECUTION_EFFECTS], "native-local")
            config = Path(command[command.index("--config-file") + 1])
            report = config.with_suffix(".xml")
            report.write_text(
                f'<testsuites tests="1" failures="1" errors="0"><testsuite tests="1" failures="1"><testcase classname="{driver.BINARY}" name="{driver.TEST_NAME}"><failure type="test failure with exit code 101">bounded semantic failure</failure></testcase></testsuite></testsuites>'
            )
            observation.update(owned_drain="complete", child_status=100)
            return 100

        with (
            patch.object(native_tests, "ROOT", driver.ROOT),
            patch.object(test_run, "ROOT", driver.ROOT),
            patch.object(native_tests, "ordinary_rust_capture", side_effect=capture),
            patch.object(
                native_tests, "prepared_rust_inventory", return_value={}
            ) as prepared,
            patch.object(native_operation, "scope_owner", return_value=None),
            patch.object(native_operation, "run", side_effect=execute),
            patch.dict(
                os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}
            ),
        ):
            result = driver.rust_trial(checkout, output)
        prepared.assert_called_once()
        self.assertEqual(driver.classify_product(result), 1)
        self.assertEqual(result["kind"], "product-test")
        self.assertEqual(result["owned_drain"], "complete")
        self.assertEqual(result["subjects"][0]["identity"], driver.SUBJECT)

    def test_cleanup_rejects_replaced_checkout(self) -> None:
        checkout = self.root / "checkout"
        checkout.mkdir()
        stat = checkout.stat()
        checkout.rename(self.root / "original")
        checkout.symlink_to(self.repo, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "identity changed"):
            driver.remove_owned(checkout, (stat.st_dev, stat.st_ino))
        self.assertTrue(self.repo.is_dir())

    def test_execution_exception_cannot_reuse_an_earlier_drain_witness_for_cleanup(
        self,
    ) -> None:
        output = self.root / "interrupted"
        args = argparse.Namespace(
            repo=self.repo,
            good=self.commits[0],
            bad=self.commits[-1],
            output=output,
            repetitions=1,
            cleanup=True,
        )
        calls = 0

        def interrupted(command: list[str], cwd: Path, log: Path) -> tuple[int, dict]:
            nonlocal calls
            calls += 1
            if calls == 2:
                raise ValueError("owned drain unresolved")
            return self.fixture_run(command, cwd, log)

        with (
            patch.object(driver, "freeze_controller", side_effect=self.frozen),
            patch.object(driver, "run_owned", side_effect=interrupted),
        ):
            code, result = driver.bisect(args)
        self.assertEqual(code, 128)
        self.assertEqual(result["status"], "aborted")
        self.assertTrue((output / "checkout").is_dir())
        self.assertEqual(result["cleanup"], "retained: physical drain unconfirmed")
        self.assertNotIn("causal_lead", result)
        self.assertEqual(self.git("status", "--porcelain"), self.before)

    def test_cancellation_uses_existing_owner_to_forward_signal_and_reap_child(
        self,
    ) -> None:
        # Exercise actual existing foreground signal forwarding with a live child.
        ready = self.root / "ready"
        evidence = self.root / "cancel.json"
        script = self.root / "cancel.py"
        script.write_text(f"""
import json, os, sys
from pathlib import Path
sys.path.insert(0, {str(driver.ROOT)!r})
from scripts import native_operation
observation = {{}}
code = native_operation.run([sys.executable, '-c', "import os,time; from pathlib import Path; Path({str(ready)!r}).write_text(str(os.getpid())); time.sleep(300)"], dict(os.environ), observation=observation)
Path({str(evidence)!r}).write_text(json.dumps({{'code':code, 'process':observation}}))
""")
        child = subprocess.Popen([sys.executable, str(script)])
        self.addCleanup(lambda: child.poll() is None and child.kill())
        deadline = time.monotonic() + 10
        while not ready.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        self.assertTrue(ready.exists())
        child.send_signal(signal.SIGTERM)
        child.wait(timeout=15)
        result = json.loads(evidence.read_text())
        self.assertEqual(result["code"], 128 + signal.SIGTERM)
        self.assertEqual(result["process"]["forwarded_signal"], signal.SIGTERM)
        self.assertFalse(Path(f"/proc/{ready.read_text()}").exists())


class ComparisonPair(TypedDict):
    order: list[str]
    baseline: dict[str, str]
    candidate: dict[str, str]


class ComparisonTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def declaration(self, ratios: list[float], *, incompatible: bool = False) -> Path:
        pairs: list[ComparisonPair] = []
        for index, ratio in enumerate(ratios):
            order = (
                ["baseline", "candidate"]
                if index % 2 == 0
                else ["candidate", "baseline"]
            )
            references: dict[str, dict[str, str]] = {}
            for side, value in (("baseline", 10), ("candidate", 10 * ratio)):
                directory = self.root / f"{index}-{side}"
                (directory / "warm-0").mkdir(parents=True)
                sample_result = {
                    "exit_code": 0,
                    "wall_seconds": value,
                    "disk_free_before": 1000000 + index,
                }
                report = {
                    "builds": {"warm-0": sample_result},
                    "original_source_unchanged": True,
                    "cache_state": "new target; shared downloaded sources, native prerequisites and selected compiler cache",
                    "correctness_execution": False,
                    "cold_repetitions": 1,
                    "statistics": {},
                    "preparation": {"status": "passed"},
                }
                # Current build_measurements has no D04 observations: never fill
                # that gap with declarations or an invented producer field.
                (directory / "summary.json").write_text(json.dumps(report))
                command = {
                    "command": ["cargo", "build", "--locked"],
                    "environment": {
                        "CARGO_BUILD_JOBS": "2"
                        if incompatible and side == "candidate"
                        else "1",
                        "CARGO_TARGET_DIR": str(directory / "target"),
                    },
                }
                (directory / "warm-0/command.json").write_text(json.dumps(command))
                files = {
                    "rust-toolchain.toml": "pinned toolchain",
                    "crates/product.rs": side,
                    "tests/fixtures/input": "same input",
                }
                (directory / "warm-0/source-files.json").write_text(json.dumps(files))
                (directory / "warm-0/result.json").write_text(json.dumps(sample_result))
                references[side] = {
                    "path": f"{index}-{side}/summary.json",
                    "sample": "warm-0",
                }
            pairs.append(
                {
                    "order": order,
                    "baseline": references["baseline"],
                    "candidate": references["candidate"],
                }
            )
        path = self.root / "pairs.json"
        _value, conditions, baseline = driver.measurement(
            pairs[0]["baseline"], self.root
        )
        _value, _conditions, candidate = driver.measurement(
            pairs[0]["candidate"], self.root
        )
        path.write_text(
            json.dumps(
                {
                    "conditions": conditions,
                    "treatment": {
                        "dimension": "source",
                        "baseline": baseline["source_identity"],
                        "candidate": candidate["source_identity"],
                    },
                    "pairs": pairs,
                }
            )
        )
        return path

    def test_explicit_pairs_keep_membership_order_and_no_automatic_winner(self) -> None:
        result = driver.compare(self.declaration([0.8, 0.81, 0.79]), 0.1)
        self.assertEqual(result["status"], "inconclusive")
        self.assertEqual(result["reasons"], ["D04 pressure observations unavailable"])
        self.assertEqual(result["candidate_over_baseline_median"], 0.8)
        self.assertEqual(result["pairs"][1]["order"], ["candidate", "baseline"])
        self.assertEqual(
            result["pairs"][0]["baseline"]["population"], "one named build invocation"
        )
        self.assertNotIn("winner", result)

    def test_noisy_comparison_is_inconclusive(self) -> None:
        result = driver.compare(self.declaration([0.6, 1.5, 0.9]), 0.1)
        self.assertEqual(result["status"], "inconclusive")
        self.assertIn("paired ratios are noisy", result["reasons"])

    def test_incompatible_conditions_and_missing_pressure_are_inconclusive(
        self,
    ) -> None:
        result = driver.compare(
            self.declaration([0.8, 0.8, 0.8], incompatible=True), 0.1
        )
        self.assertEqual(result["status"], "inconclusive")
        self.assertIn(
            "actual measurement conditions are incompatible", result["reasons"]
        )
        self.assertIn("D04 pressure observations unavailable", result["reasons"])

    def test_reused_or_missing_membership_is_inconclusive(self) -> None:
        path = self.declaration([1, 1, 1])
        declaration = json.loads(path.read_text())
        declaration["pairs"][1]["baseline"] = declaration["pairs"][0]["baseline"]
        declaration["pairs"][2]["candidate"]["sample"] = "missing"
        path.write_text(json.dumps(declaration))
        result = driver.compare(path, 0.1)
        self.assertEqual(result["status"], "inconclusive")
        self.assertIn("sample membership is reused across pairs", result["reasons"])

    def test_declared_conditions_and_source_treatment_cannot_fill_or_replace_actual_facts(
        self,
    ) -> None:
        path = self.declaration([0.8, 0.81, 0.79])
        declaration = json.loads(path.read_text())
        declaration["conditions"]["input"] = "claimed identical"
        declaration["treatment"]["baseline"] = "invented source"
        path.write_text(json.dumps(declaration))
        result = driver.compare(path, 0.1)
        self.assertEqual(result["status"], "inconclusive")
        self.assertIn(
            "declared conditions differ from actual producer observations",
            result["reasons"],
        )
        self.assertIn(
            "source treatment differs from actual producer inventory", result["reasons"]
        )

    def test_case_population_does_not_join_criterion_to_harness_invocations(
        self,
    ) -> None:
        csv = self.root / "raw.csv"
        csv.write_text(
            "sample_measured_value,iteration_count,unit\n42,1,ns\n43,1,ns\n41,1,ns\n"
        )
        report = {
            "schema": "process-cost-v5",
            "measured": True,
            "profile": {"cargo_profile": "dev", "features": []},
            "source_observations_start": {"crates/product.rs": "source"},
            "environment_start": {},
            "environment_end": {},
            "source_changes_during_measurement": [],
            "environment_changes_during_measurement": [],
            "compilation_timed": False,
            "managed_receiver": None,
            "cases": [
                {
                    "id": "solve",
                    "workload": {"id": "solve"},
                    "setup": {},
                    "sampling": "flat measured samples",
                    "timed_scope": "solve",
                    "untimed_scope": "setup",
                    "threads": 1,
                    "native_threads": 1,
                    "criterion_statistics": case_measure.samples(csv),
                    "invocation_observations": {
                        "criterion_membership": {"status": "unavailable"}
                    },
                }
            ],
        }
        (self.root / "case-measure.json").write_text(json.dumps(report))
        value, _conditions, reference = driver.measurement(
            {"path": "case-measure.json", "sample": "solve"}, self.root
        )
        self.assertEqual(value, 42)
        self.assertEqual(reference["population"], "Criterion measured samples only")
        self.assertIn("unavailable", reference["criterion_invocation_join"])


if __name__ == "__main__":
    unittest.main()

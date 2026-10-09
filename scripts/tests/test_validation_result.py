# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent assessment checkpoints exercise read-only result retrieval."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

if TYPE_CHECKING:
    from typing import TextIO

from scripts import test_resources, validation, validation_result


class ValidationResultTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.run_directory = self.root / "run with spaces"
        self.run_directory.mkdir()
        self.record = {
            "version": 5,
            "mode": "local",
            "baseline_failures": 0,
            "complete": True,
            "required_checks_covered": True,
            "source_unchanged": True,
            "provenance_errors": [],
            "input_coverage": True,
            "evidence": "Tested",
            "parent": None,
            "scope": [{"name": "unit", "role": "required"}],
            "checks": [self.check("unit", "passed")],
        }
        (self.run_directory / "unit.log").write_text("old\nlast\n")
        self.write()

    @staticmethod
    def check(name: str, status: str, role: str = "required") -> dict:
        return {
            "gate": name,
            "status": status,
            "role": role,
            "log": f"{name}.log",
            "artifacts": {f"{name}.log": "recorded-sha256"},
            "evidence_kind": "executed",
            "exit_code": 0 if status == "passed" else 1,
            "report_errors": [],
            "results": [],
        }

    def write(self) -> None:
        (self.run_directory / "checks.json").write_text(json.dumps(self.record))

    def invoke(self, *args: str, run: Path | None = None) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = validation_result.main([str(run or self.run_directory), *args])
        return code, out.getvalue(), err.getvalue()

    def result(self, *args: str) -> dict:
        code, output, error = self.invoke(*args, "--json")
        self.assertEqual((code, error), (0, ""))
        return json.loads(output)

    def test_success_identifies_run_and_paths_without_default_success_tail(
        self,
    ) -> None:
        result = self.result()
        self.assertEqual(result["run_path"], str(self.run_directory))
        self.assertEqual(result["record_path"], str(self.run_directory / "checks.json"))
        self.assertEqual(result["assessment"]["complete"], True)
        self.assertEqual(result["assessment"]["required_checks_covered"], True)
        self.assertEqual(result["assessment"]["baseline_failures"], 0)
        check = result["checks"][0]
        self.assertEqual(check["record"]["status"], "passed")
        self.assertEqual(check["record"]["role"], "required")
        self.assertEqual(check["record"]["evidence_kind"], "executed")
        self.assertEqual(check["log"]["path"], str(self.run_directory / "unit.log"))
        self.assertEqual(
            check["artifacts"][0]["path"], str(self.run_directory / "unit.log")
        )
        self.assertEqual(check["log"]["tail"], [])
        code, human, _ = self.invoke()
        self.assertEqual(code, 0)
        for text in (
            str(self.run_directory),
            "complete: true",
            "required_checks_covered: true",
            "baseline_failures: 0",
            "unit: passed",
            "role=required",
            "evidence_kind=executed",
        ):
            self.assertIn(text, human)

    def test_failed_record_retains_errors_and_zero_reader_exit(self) -> None:
        self.record["checks"][0].update(
            status="failed",
            report_errors=["missing selected identity"],
            spawn_error="cannot start",
            exit_code=2,
        )
        self.record.update(
            required_checks_covered=False,
            source_unchanged=False,
            provenance_errors=["source drift"],
        )
        (self.run_directory / "unit.log").write_text(
            "\n".join(f"line {i}" for i in range(30))
        )
        self.write()
        result = self.result("--failures")
        check = result["checks"][0]
        self.assertEqual(check["log"]["tail"], [f"line {i}" for i in range(10, 30)])
        self.assertFalse(check["qualified"])
        self.assertFalse(result["assessment"]["source_unchanged"])
        self.assertEqual(
            check["record"]["report_errors"], ["missing selected identity"]
        )
        code, human, _ = self.invoke("--failures")
        self.assertEqual(code, 0)
        for text in (
            "unit: failed",
            "source_unchanged: false",
            "source drift",
            "missing selected identity",
            "cannot start",
            "line 29",
        ):
            self.assertIn(text, human)

    def test_failure_filter_preserves_advisory_and_deferred_qualification(self) -> None:
        states = [
            ("advice", "findings", "advisory"),
            ("later", "unsupported", "deferred"),
            ("broken", "failed", "advisory"),
            ("required", "unsupported", "required"),
        ]
        self.record["scope"] = [
            {"name": name, "role": role} for name, _, role in states
        ]
        self.record["checks"] = [self.check(*state) for state in states]
        self.write()
        result = self.result("--failures")
        self.assertEqual(
            [item["record"]["gate"] for item in result["checks"]],
            ["broken", "required"],
        )
        all_checks = self.result()["checks"]
        self.assertEqual(
            [item["record"]["status"] for item in all_checks],
            ["findings", "unsupported", "failed", "unsupported"],
        )
        self.assertEqual(
            [item["qualified"] for item in all_checks], [True, True, False, False]
        )

    def test_interruption_keeps_unattempted_scope_separate(self) -> None:
        self.record["scope"].append({"name": "docs", "role": "required"})
        self.record["checks"][0]["status"] = "interrupted"
        self.record["complete"] = False
        del self.record["required_checks_covered"]
        self.write()
        result = self.result("--failures")
        self.assertFalse(result["assessment"]["complete"])
        self.assertNotIn("required_checks_covered", result["assessment"])
        self.assertEqual(result["checks"][0]["record"]["status"], "interrupted")
        self.assertEqual(
            result["unattempted_scope"], [{"name": "docs", "role": "required"}]
        )
        selected = self.result("--gate", "docs")
        self.assertEqual(selected["checks"], [])
        self.assertEqual(
            selected["unattempted_scope"], [{"name": "docs", "role": "required"}]
        )
        _, human, _ = self.invoke("--gate", "docs")
        self.assertIn("required_checks_covered: not recorded", human)
        self.assertIn(
            "Unattempted scope (no recorded check):\n  docs; role=required", human
        )

    def test_reused_evidence_reads_original_logs_and_artifacts(self) -> None:
        origin = self.root / "original"
        origin.mkdir()
        (origin / "unit.log").write_text("original evidence\n")
        (origin / "report.xml").write_text("original artifact")
        self.record["checks"][0].update(
            origin=str(origin),
            origin_digest="original-checkpoint-sha256",
            evidence_kind="unchanged-input-reuse",
            artifacts={"report.xml": "report-sha256"},
        )
        self.write()
        result = self.result("--gate", "unit")
        check = result["checks"][0]
        self.assertEqual(check["log"]["path"], str(origin / "unit.log"))
        self.assertEqual(check["log"]["tail"], ["original evidence"])
        self.assertEqual(check["artifacts"][0]["path"], str(origin / "report.xml"))
        self.assertEqual(check["record"]["evidence_kind"], "unchanged-input-reuse")
        _, human, _ = self.invoke("--gate", "unit")
        self.assertIn("evidence_kind=unchanged-input-reuse", human)
        self.assertIn("original-checkpoint-sha256", human)

    def test_explicit_success_tail_and_zero_tail(self) -> None:
        self.assertEqual(
            self.result("--gate", "unit", "--tail", "1")["checks"][0]["log"]["tail"],
            ["last"],
        )
        self.assertEqual(
            self.result("--gate", "unit", "--tail", "0")["checks"][0]["log"]["tail"], []
        )

    def test_tail_is_bounded_to_final_64_kib_and_decodes_invalid_bytes(self) -> None:
        (self.run_directory / "unit.log").write_bytes(
            b"outside-bound\n" + b"x" * (64 * 1024) + b"\nlast\xff\n"
        )
        tail = self.result("--gate", "unit", "--tail", "999999")["checks"][0]["log"][
            "tail"
        ]
        self.assertNotIn("outside-bound", tail)
        self.assertEqual(tail[-1], "last\ufffd")
        self.assertLessEqual(sum(len(line) for line in tail), 64 * 1024)

    def test_missing_log_and_artifact_are_explicit_for_json_and_human(self) -> None:
        (self.run_directory / "unit.log").unlink()
        self.record["checks"][0]["artifacts"]["report.xml"] = "missing-sha256"
        self.write()
        check = self.result()["checks"][0]
        self.assertEqual(check["log"]["status"], "missing")
        self.assertIn("error", check["log"])
        self.assertTrue(all(item["status"] == "missing" for item in check["artifacts"]))
        code, human, _ = self.invoke()
        self.assertEqual(code, 0)
        self.assertIn(f"log: {self.run_directory / 'unit.log'} (missing)", human)
        self.assertIn(f"artifact: {self.run_directory / 'report.xml'} (missing)", human)

    def test_latest_is_resolved_once_even_if_link_changes_during_record_read(
        self,
    ) -> None:
        latest = self.root / "latest-ready"
        latest.symlink_to(self.run_directory.name)
        other = self.root / "other"
        other.mkdir()
        original_load = json.load
        calls = []

        def move_latest(stream: TextIO) -> object:
            calls.append(stream.name)
            latest.unlink()
            latest.symlink_to(other.name)
            return original_load(stream)

        with patch.object(validation_result.json, "load", side_effect=move_latest):
            code, output, error = self.invoke("--gate", "unit", "--json", run=latest)
        self.assertEqual((code, error), (0, ""))
        self.assertEqual(calls, [str(self.run_directory / "checks.json")])
        result = json.loads(output)
        self.assertEqual(result["run_path"], str(self.run_directory))
        self.assertEqual(result["checks"][0]["log"]["tail"], ["old", "last"])

    def test_read_has_no_runner_cleanup_or_checkpoint_effects(self) -> None:
        before = {
            path: (path.read_bytes(), path.stat().st_mtime_ns)
            for path in self.run_directory.iterdir()
        }
        with (
            patch.object(validation, "execute", side_effect=AssertionError("executed")),
            patch.object(
                validation, "checkpoint", side_effect=AssertionError("checkpoint")
            ),
            patch.object(
                test_resources, "reclaim_reports", side_effect=AssertionError("cleanup")
            ),
            patch.object(
                test_resources,
                "register_report",
                side_effect=AssertionError("registered"),
            ),
            patch.object(
                test_resources,
                "reference_report",
                side_effect=AssertionError("referenced"),
            ),
        ):
            self.result("--gate", "unit")
        after = {
            path: (path.read_bytes(), path.stat().st_mtime_ns)
            for path in self.run_directory.iterdir()
        }
        self.assertEqual(after, before)

    def test_missing_and_malformed_records_return_one(self) -> None:
        checkpoint = self.run_directory / "checks.json"
        checkpoint.unlink()
        self.assertEqual(self.invoke()[0], 1)
        for bad in (
            "{",
            "[]",
            '{"version": 4}',
            json.dumps({**self.record, "checks": [None]}),
            json.dumps({**self.record, "complete": "yes"}),
            json.dumps({**self.record, "scope": [{"name": "unit"}, {"name": "unit"}]}),
        ):
            with self.subTest(record=bad):
                checkpoint.write_text(bad)
                code, output, _ = self.invoke("--json")
                self.assertEqual(code, 1)
                self.assertIn("error", json.loads(output))

    def test_unknown_gate_and_invalid_usage_return_two(self) -> None:
        for args in (
            ("--gate", "unknown"),
            ("--tail", "-1"),
            ("--tail", "bad"),
            ("--unknown",),
        ):
            with (
                self.subTest(args=args),
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as raised,
            ):
                validation_result.main([str(self.run_directory), *args])
            self.assertEqual(raised.exception.code, 2)


if __name__ == "__main__":
    unittest.main()

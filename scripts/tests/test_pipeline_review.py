# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Catalog routing and publication preserve original outcomes and observation values."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import csv
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import pipeline_review


class PipelineReviewTests(unittest.TestCase):
    def test_catalog_preserves_explicit_existing_selection_and_target(self) -> None:
        selected = pipeline_review.catalog(["k4-scalar-preparation", "cold-small-1"])
        self.assertEqual(
            [target for target, _ in selected],
            ["modeling_preparation", "native_process"],
        )
        self.assertEqual(
            [workload["id"] for _, workload in selected],
            ["k4-scalar-preparation", "cold-small-1"],
        )
        for invalid in ([], ["unknown-case"], ["cold-small-1", "cold-small-1"]):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                pipeline_review.catalog(invalid)

    def test_prebuilt_map_and_literal_path_overrides(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            binary = directory / "literal binary"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(0o700)
            source = directory / "binaries.json"
            source.write_text(json.dumps({"binaries": {"native_process": str(binary)}}))
            binaries = pipeline_review.binaries_from(
                source, [f"modeling_preparation={binary}"]
            )
            self.assertEqual(
                binaries, {"native_process": binary, "modeling_preparation": binary}
            )
            with self.assertRaises(ValueError):
                pipeline_review.binaries_from(None, ["native_process=missing-binary"])

    def test_execute_routes_real_spec_and_keeps_scientific_failure(self) -> None:
        workload = pipeline_review.catalog(["cold-small-1"])[0][1]
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "case"
            with (
                patch.object(
                    pipeline_review.case_measure,
                    "managed_fixture_environment",
                    return_value={"ACTUAL_MANAGED": "selected"},
                ),
                patch.object(
                    pipeline_review.case_measure,
                    "process_command",
                    return_value=["literal-binary", "--test"],
                ) as command,
                patch.object(
                    pipeline_review.case_audit,
                    "run_case",
                    return_value={
                        "status": "failed",
                        "exit_code": 7,
                        "fixture_cleanup_failures": 1,
                    },
                ) as audit,
            ):
                record = pipeline_review.execute_case(
                    "/bin/true",
                    workload,
                    directory,
                    recording_mode="ordinary",
                    env={"CALLER": "value"},
                    label="paired ordinary",
                )
            self.assertEqual(command.call_args.args[2], workload)
            self.assertEqual(audit.call_args.args[0], ["literal-binary", "--test"])
            environment = audit.call_args.kwargs["env"]
            self.assertEqual(json.loads(environment["PSE_PROCESS_COST_SPEC"]), workload)
            self.assertEqual(environment["PSE_PROCESS_COST_OUTPUT"], str(directory))
            self.assertEqual(environment["PSE_PREPARATION_OUTPUT"], str(directory))
            self.assertEqual(environment["ACTUAL_MANAGED"], "selected")
            self.assertEqual(audit.call_args.kwargs["recording_mode"], "ordinary")
            self.assertEqual(record["result"]["exit_code"], 7)
            self.assertEqual(
                json.loads((directory / "case-run.json").read_text()), record
            )

    def test_publish_aggregates_exact_original_values_without_summing_or_overwriting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            case = root / "runs/case-one"
            case.mkdir(parents=True)
            (case / "case-run.json").write_text(
                json.dumps(
                    {
                        "case": "cold-small-1",
                        "result": {
                            "status": "completed",
                            "fixture_cleanup_failures": 1,
                        },
                    }
                )
            )
            (case / "cold-small-1-memory.json").write_text(
                json.dumps({"pool_peak_bytes": 12, "native_work": None})
            )
            observation = case / "cold-small-1-observations.jsonl"
            original = b'{"seconds":0.001,"work":{"iterations":7}}\n{"seconds":0.002,"work":null}\n'
            observation.write_bytes(original)
            raw = case / "criterion/samples.csv"
            raw.parent.mkdir()
            raw.write_bytes(b"original criterion sample bytes\n")
            supplement = root / "supplement.json"
            supplement.write_text(
                json.dumps({"actual": {"factorizations": 3}, "unavailable": None})
            )
            output = root / "publication"
            index = pipeline_review.publish(
                output,
                runs=[root / "runs"],
                supplements=[supplement],
                label="retained <review>",
            )
            self.assertEqual(index["cases"][0]["status"], "failed")
            self.assertEqual(index["cases"][0]["result"]["status"], "completed")
            self.assertEqual(index["errors"], [])
            self.assertEqual(observation.read_bytes(), original)
            self.assertEqual(raw.read_bytes(), b"original criterion sample bytes\n")
            with (output / "observations.csv").open() as source:
                rows = list(csv.DictReader(source))
            timings = [row for row in rows if row["pointer"] == "/seconds"]
            self.assertEqual([row["value_json"] for row in timings], ["0.001", "0.002"])
            self.assertEqual([row["record_line"] for row in timings], ["1", "2"])
            self.assertTrue(
                any(
                    row["pointer"] == "/native_work" and row["value_json"] == "null"
                    for row in rows
                )
            )
            self.assertIn("criterion/samples.csv", (output / "index.md").read_text())
            self.assertIn(
                "retained &lt;review&gt;", (output / "index.html").read_text()
            )
            self.assertTrue((output / "index.json").is_file())

    def test_campaign_continues_after_one_case_execution_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "campaign"
            with (
                patch.object(
                    pipeline_review.case_audit,
                    "run_case",
                    side_effect=[
                        RuntimeError("original failure"),
                        {"status": "completed"},
                    ],
                ) as audit,
                patch.object(
                    pipeline_review.case_measure,
                    "managed_fixture_environment",
                    return_value={},
                ),
                patch.object(
                    pipeline_review.case_measure,
                    "process_command",
                    return_value=["/bin/true", "--test"],
                ),
            ):
                status = pipeline_review.main(
                    [
                        str(output),
                        "--binary",
                        "native_process=/bin/true",
                        "--case",
                        "cold-small-1",
                        "--case",
                        "warm-small-1",
                        "--recording-mode",
                        "ordinary",
                    ]
                )
            self.assertEqual(status, 1)
            self.assertEqual(audit.call_count, 2)
            index = json.loads((output / "index.json").read_text())
            self.assertEqual(len(index["cases"]), 2)
            first = next(
                case for case in index["cases"] if case["case"] == "cold-small-1"
            )
            self.assertIn("original failure", first["execution_error"])
            self.assertEqual(first["status"], "failed")


if __name__ == "__main__":
    unittest.main()

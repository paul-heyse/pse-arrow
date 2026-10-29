# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only paired measurement interpretation; no live execution or qualification."""

# ruff: noqa: PT009
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from typing import TypedDict

from scripts import regression_experiments as driver


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
                # Historical build reports have no D04 observations: never fill
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

    def test_frame_limit_is_a_recorded_cache_condition(self) -> None:
        self.declaration([0.8])
        directory = self.root / "0-baseline"
        path = directory / "warm-0/command.json"
        command = json.loads(path.read_text())
        command["environment"]["SCCACHE_MAX_FRAME_LENGTH"] = "134217728"
        path.write_text(json.dumps(command))
        _value, conditions, _association = driver.measurement(
            {"path": "0-baseline/summary.json", "sample": "warm-0"}, self.root
        )
        self.assertEqual(
            conditions["cache"]["environment"]["SCCACHE_MAX_FRAME_LENGTH"],
            "134217728",
        )
        self.assertNotIn("SCCACHE_MAX_FRAME_LENGTH", conditions["build"]["environment"])

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
                    "criterion_statistics": {
                        "sample_count": 3,
                        "mean_nanoseconds": 42.0,
                    },
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

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pinned mutation protocol keeps compile scope and failed baselines explicit."""
# ruff: noqa: PT009, PT027
from __future__ import annotations

import unittest
from unittest.mock import patch

from scripts import assertion_sensitivity as sensitivity


class SensitivityTests(unittest.TestCase):
    def test_both_phases_keep_validated_compile_flags_and_only_test_gets_filter(self):
        common, tests = sensitivity.split_arguments([
            "--locked", "-p", "actual", "--lib", "--features", "actual/force-validate",
            "--cargo-profile", "debugging", "-E", "test(exact)", "--test-threads", "16",
        ])
        self.assertEqual(common, ["--locked", "--lib", "--features", "actual/force-validate", "--cargo-profile", "debugging"])
        self.assertEqual(tests, ["-E", "test(exact)", "--test-threads", "16"])
        self.assertEqual(sensitivity.split_arguments(["-Factual/force-validate"])[0], ["-Factual/force-validate"])

    def test_copy_bypass_and_missing_values_are_refused(self):
        for args in (["--target-dir", "/original/target"], ["--manifest-path", "/original/Cargo.toml"], ["--archive-file", "original.zip"], ["--features"]):
            with self.assertRaises(ValueError):
                sensitivity.split_arguments(args)

    def test_actual_protocol_baseline_and_mutants_have_distinct_outcomes(self):
        baseline = {"scenario": "Baseline", "summary": "Success", "phase_results": [{"phase": "Test", "process_status": "Success"}]}
        for summary, expected in (("CaughtMutant", "caught"), ("MissedMutant", "missed"), ("Unviable", "unviable"), ("Timeout", "timeout")):
            rows, status = sensitivity.decode_outcomes({"outcomes": [baseline, {"scenario": {"Mutant": {"name": "actual"}}, "summary": summary, "phase_results": []}]})
            self.assertEqual(status, "passed")
            self.assertEqual(rows[1]["classification"], expected)
        rows, status = sensitivity.decode_outcomes({"outcomes": [{"scenario": "Baseline", "summary": "Failure", "phase_results": []}]})
        self.assertEqual(status, "setup-failed")
        self.assertEqual(rows[0]["classification"], "setup-failed")

    def test_unknown_protocol_or_no_challenge_is_not_positive_evidence(self):
        for value in ({}, {"outcomes": []}, {"outcomes": [{"scenario": "Baseline", "summary": "Future", "phase_results": []}]}, {"outcomes": [{"scenario": "Baseline", "summary": "Success", "phase_results": []}]}):
            with self.assertRaises(ValueError):
                sensitivity.decode_outcomes(value)

    def test_cli_default_uses_fresh_output_owner(self):
        with patch.object(sensitivity, "run", return_value=0) as selected:
            self.assertEqual(sensitivity.main(["crates/actual/src/lib.rs", "--", "-p", "actual"]), 0)
        self.assertIsNone(selected.call_args.args[1])


if __name__ == "__main__":
    unittest.main()

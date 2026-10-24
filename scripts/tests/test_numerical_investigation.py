# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent numerical-report interpretation controls."""

# ruff: noqa: PT009, PT027
import unittest
from decimal import Decimal

from scripts.numerical_investigation import investigate, reference


class NumericalInvestigationTests(unittest.TestCase):
    def test_decimal_exact_input_oracle_and_wrong_derivative_remain_visible(
        self,
    ) -> None:
        value, derivative = reference("analytic-cubic", 2.0)
        self.assertEqual((value, derivative), (Decimal(8), Decimal(12)))
        sample = {
            "case": "wrong-derivative",
            "point": 2.0,
            "coordinate_scale": 4.0,
            "row_scale": 1.0,
            "complete": True,
            "passed": False,
            "comparisons": [
                {"label": "jac_g[0,0]", "analytic": 96.0, "numeric": 48.0, "step": 1e-4}
            ],
        }
        result = investigate(
            {"schema": "numerical-investigation-v1", "samples": [sample]}
        )
        comparison = result["samples"][0]["comparisons"][0]
        self.assertEqual(comparison["sampling_error"], 0.0)
        self.assertEqual(comparison["derivative_error"], 48.0)
        self.assertFalse(result["samples"][0]["passed"])

    def test_refused_and_incomplete_samples_are_preserved(self) -> None:
        sample = {
            "case": "log-refused",
            "point": -0.9999,
            "coordinate_scale": 1.0,
            "row_scale": 1.0,
            "complete": False,
            "passed": False,
            "rejected_evaluations": 1,
            "failure": "outside domain",
            "comparisons": [],
        }
        result = investigate(
            {"schema": "numerical-investigation-v1", "samples": [sample]}
        )
        self.assertEqual(result["samples"][0]["failure"], "outside domain")
        self.assertEqual(result["samples"][0]["rejected_evaluations"], 1)
        self.assertGreater(
            Decimal(result["samples"][0]["reference"]["derivative"]), Decimal(1000)
        )
        self.assertEqual(len(result["plan14_reference"]["checks"]), 5)

    def test_empty_or_unknown_capture_is_not_success(self) -> None:
        for document in [
            {"schema": "invented", "samples": [{}]},
            {"schema": "numerical-investigation-v1", "samples": []},
        ]:
            with self.assertRaises(ValueError):
                investigate(document)

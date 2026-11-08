# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Controls for separating independent scientific agreement from native policy."""
# ruff: noqa: PT009, PT027 -- named reference recipes use standard-library unittest

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from tests.reference.oracles import covariance, legacy_exchange, sensitivity
else:
    from oracles import covariance, legacy_exchange, sensitivity


class LegacyAgreementTests(unittest.TestCase):
    """Exercise the independent comparison before any solver or candidate launch."""

    def test_oversized_response_metadata_is_refused_before_publication(self) -> None:
        """Reject oversized diagnostics and population before writing the response."""
        response = covariance.Comparison("candidate", "failed", ["x" * 1025])
        with TemporaryDirectory() as directory:
            path = Path(directory) / "response.json"
            with self.assertRaisesRegex(ValueError, "comparison exceeds"):
                legacy_exchange.write_response(path, response, 1, {}, [response])
            self.assertFalse(path.exists())
            with self.assertRaisesRegex(ValueError, "population outside"):
                legacy_exchange.response_limit(33)

    def test_covariance_rejects_nonfinite_and_inflated_native_allowance(self) -> None:
        """Preserve scientific agreement with an independently set error limit."""
        case = covariance.LineCase(
            "weighted-line-known-sigma-v1",
            [1.0, 2.0, 3.0, 4.0, 5.0],
            [2.1, 3.9, 6.2, 7.8, 10.1],
            0.5,
            ["a", "b"],
            "1",
            ["1", "1"],
            "inverse_information_known_sigma",
        )
        covariance.validate_case(case)
        matrix = covariance.Covariance(
            case.parameters,
            case.parameter_units,
            [1.0, 1.0],
            "physical",
            "row_major",
            case.covariance_convention,
            [0.275, -0.075, -0.075, 0.025],
        )
        independent = covariance.Reference(
            {"a": 0.05, "b": 1.99}, matrix, matrix.values
        )
        candidate = covariance.CandidateReport(
            True, independent.estimates.copy(), {"a": 0.001, "b": 0.001}, "1", matrix
        )
        item = covariance.Candidate("witness", candidate)
        request = covariance.Request(1, case, {}, [item])
        self.assertEqual(
            covariance.compare(item, request, independent).status, "passed"
        )
        candidate.estimate_allowances["a"] = 100.0
        for value in (0.06, float("inf"), float("nan")):
            candidate.estimates["a"] = value
            self.assertEqual(
                covariance.compare(item, request, independent).reasons,
                ["estimate differs at a"],
            )

    def test_sipopt_prediction_and_native_derivative_refuse_independently(self) -> None:
        """Challenge the oracle's own prediction and the native Jacobian separately."""
        case = sensitivity.SensitivityCase(
            "active-bound-quadratic-response",
            {"a": 1.0, "b": 2.0},
            {"a": 1.1, "b": 2.1},
            ["x", "y", "z"],
            "dimensionless Scalar",
            0.5,
            (0.0, 10.0),
            "x + y == a",
            "0.5*x*x + y*y - b*x + 0.5*z*z + a*z",
        )
        sensitivity.validate_case(case)
        nominal, prediction = (
            sensitivity.exact(case.nominal),
            sensitivity.exact(case.perturbed),
        )
        sensitivity.check_sipopt(case, nominal, prediction)
        for observation, other, name in (
            (nominal, prediction, "nominal"),
            (prediction, nominal, "prediction"),
        ):
            wrong = {**observation, "x": observation["x"] + 0.01}
            with self.assertRaisesRegex(ValueError, f"sIPOPT {name} differs"):
                if name == "nominal":
                    sensitivity.check_sipopt(case, wrong, other)
                else:
                    sensitivity.check_sipopt(case, other, wrong)
        quantities = {}
        derivatives = ((2 / 3, 1 / 3), (1 / 3, -1 / 3), (0.0, 0.0))
        for name, (a, b) in zip(case.variables, derivatives, strict=True):
            increment = prediction[name] - nominal[name]
            quantities[name] = sensitivity.QuantityReport(
                nominal[name],
                increment,
                nominal[name] + increment,
                0.001,
                1.0,
                {"a": a, "b": b},
            )
        item = sensitivity.Candidate(
            "witness",
            sensitivity.CandidateReport(
                True, case.unit, "physical", {"a": 1.0, "b": 1.0}, quantities
            ),
        )
        self.assertEqual(sensitivity.compare(item, case, prediction).status, "passed")
        quantities["x"].allowance = 100.0
        for wrong in (2 / 3 + 0.01, float("nan")):
            quantities["x"].derivative["a"] = wrong
            self.assertEqual(
                sensitivity.compare(item, case, prediction).reasons,
                ["physical derivative differs for x/a"],
            )

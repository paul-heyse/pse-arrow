# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Controls for separating independent scientific agreement from native policy."""
# ruff: noqa: PT009, PT027 -- named reference recipes use standard-library unittest

from __future__ import annotations

import unittest
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from tests.reference.oracles import sensitivity
else:
    from oracles import sensitivity


class ReferenceAgreementTests(unittest.TestCase):
    """Exercise the independent comparison before any solver or candidate launch."""

    def test_sipopt_predictions_are_independently_checked(self) -> None:
        """The oracle independently checks both performing solver observations."""
        case = sensitivity.SensitivityCase(
            "active-bound-quadratic-response",
            {"a": 1.0, "b": 2.0},
            {"a": 1.1, "b": 2.1},
            ["x", "y", "z"],
            "dimensionless Scalar",
            0.5,
            (0.0, 10.0),
            (1.0, 1.0),
            (0.5, 1.0, 1.0, 0.5, 1.0),
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

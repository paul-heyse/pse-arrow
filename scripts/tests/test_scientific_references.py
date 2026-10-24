# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Revealing controls for the independent-reference math and comparison boundary."""
# ruff: noqa: PT009, PT027 -- standard-library script controls

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

from scripts import cstr_controller_reference as cstr
from scripts import decay_fit_reference as decay
from scripts import scientific_reference as reference


class ScientificReferenceTests(unittest.TestCase):
    def test_frozen_comparison_refuses_wrong_value_and_nonfinite(self) -> None:
        self.assertTrue(reference.comparison(2.0123424, "2.012342")["passed"])
        self.assertFalse(reference.comparison(2.012343, "2.012342")["passed"])
        self.assertFalse(reference.comparison(float("nan"), "2.012342")["passed"])
        self.assertFalse(reference.comparison(1.0, "1e999")["passed"])
        self.assertFalse(reference.comparison(1.0, "2.0", float("inf"))["passed"])

    def test_frozen_lookup_refuses_missing_or_ambiguous_consumers(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = root / "fixture.pse"
            with patch.object(reference, "ROOT", root):
                fixture.write_text("expect a==1.0")
                self.assertEqual(reference.frozen_number("fixture.pse", r"a==(\d+\.\d+)"), "1.0")
                for contents in ("expect b==1.0", "expect a==1.0; expect a==2.0"):
                    fixture.write_text(contents)
                    with self.assertRaisesRegex(ValueError, "expected one frozen value"):
                        reference.frozen_number("fixture.pse", r"a==(\d+\.\d+)")

    def test_decay_analytic_jacobian_matches_residual_directions(self) -> None:
        # Exercise non-optimal points and both parameter directions, rather than
        # trusting least_squares' success with a possibly incorrect Jacobian.
        for point in (np.array([0.3, 0.02]), np.array([2.1, 0.6]), np.array([9.0, 4.0])):
            jacobian = decay.jacobian(point)
            for direction in (np.array([1.0, 0.0]), np.array([0.0, 1.0]), np.array([0.7, -0.2])):
                step = 1e-6
                finite_difference = (decay.residual(point + step * direction) - decay.residual(point - step * direction)) / (2 * step)
                np.testing.assert_allclose(jacobian @ direction, finite_difference, rtol=1e-7, atol=1e-9)

    def test_cstr_filter_carries_derivative_and_antiwindup_recovers_saturation(self) -> None:
        initial = np.array(cstr.INITIAL)
        output, _raw, _integral, filtered_rate = cstr.controller(initial)
        self.assertLess(abs(filtered_rate), 1e-12)
        heated = initial.copy()
        heated[1] += 1.0
        filtered_output, _raw, _integral, filtered_rate = cstr.controller(heated)
        self.assertGreater(filtered_rate, 0.0)
        self.assertLess(filtered_output, output)
        saturated = initial.copy()
        saturated[2] = 100.0
        limited, raw, integral_rate, _filtered = cstr.controller(saturated)
        self.assertGreater(raw, 1.0)
        self.assertLessEqual(limited, 1.0 + cstr.INPUTS["eps"])
        self.assertLess(integral_rate, 0.0)


if __name__ == "__main__":
    unittest.main()

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
import pyarrow as pa

from scripts import cstr_controller_reference as cstr
from scripts import decay_fit_reference as decay
from scripts import frozen_reference as frozen
from scripts import pfr_reference as pfr
from scripts import reference_attribution as attribution
from scripts import scientific_reference as reference


class ScientificReferenceTests(unittest.TestCase):
    def test_table_compare_keeps_the_golden_and_requires_exact_input_coordinates(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "retained.parquet"
            original = pa.table({"case": [0], "T": [358.0], "value": [1.0]})
            allowances = {"value": (0.0, 1e-8)}
            frozen.table_reference(path, original, True, allowances)
            retained = path.read_bytes()
            frozen.table_reference(path, original, False, allowances)
            altered = pa.table({"case": [0], "T": [358.00000000001], "value": [1.0]})
            with self.assertRaisesRegex(ValueError, "stale reference.*row 0, T"):
                frozen.table_reference(path, altered, False, allowances)
            self.assertEqual(path.read_bytes(), retained)

    def test_authored_reference_inputs_refuse_stale_reaction_parameters(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in (pfr.FIXTURE, pfr.SAPONIFICATION, pfr.CONSTANTS):
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((reference.ROOT / name).read_bytes())
            pfr.validate_inputs(root)
            source = root / pfr.SAPONIFICATION
            original = source.read_text()
            self.assertEqual(original.count("3132000{m^3/(mol*s)}"), 1)
            source.write_text(
                original.replace("3132000{m^3/(mol*s)}", "3132001{m^3/(mol*s)}")
            )
            with self.assertRaisesRegex(ValueError, "stale authored input.*3132001"):
                pfr.validate_inputs(root)
            self.assertEqual(
                (root / pfr.FIXTURE).read_bytes(),
                (reference.ROOT / pfr.FIXTURE).read_bytes(),
            )

    def test_phase_attribution_refuses_a_different_case_or_historical_source(
        self,
    ) -> None:
        original = (reference.ROOT / attribution.BT_FIXTURES).read_text()
        bank = reference.ROOT / attribution.PRESSURE_BANK
        attribution.check(reference.ROOT)
        wrong_temperature = original.replace(
            "fix root.T=358{K}", "fix root.T=359{K}", 1
        )
        with self.assertRaisesRegex(ValueError, "stale authored input.*nested_liquid"):
            attribution.check_btideal(wrong_temperature, bank)
        wrong_attribution = original.replace(
            "test nested_liquid oracle references.analytic_btideal_liquid_358",
            "test nested_liquid oracle references.idaes_test_btideal",
            1,
        )
        with self.assertRaisesRegex(
            ValueError, "nested_liquid: reference attribution differs"
        ):
            attribution.check_btideal(wrong_attribution, bank)

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
                self.assertEqual(
                    reference.frozen_number("fixture.pse", r"a==(\d+\.\d+)"), "1.0"
                )
                for contents in ("expect b==1.0", "expect a==1.0; expect a==2.0"):
                    fixture.write_text(contents)
                    with self.assertRaisesRegex(
                        ValueError, "expected one frozen value"
                    ):
                        reference.frozen_number("fixture.pse", r"a==(\d+\.\d+)")

    def test_decay_analytic_jacobian_matches_residual_directions(self) -> None:
        # Exercise non-optimal points and both parameter directions, rather than
        # trusting least_squares' success with a possibly incorrect Jacobian.
        for point in (
            np.array([0.3, 0.02]),
            np.array([2.1, 0.6]),
            np.array([9.0, 4.0]),
        ):
            jacobian = decay.jacobian(point)
            for direction in (
                np.array([1.0, 0.0]),
                np.array([0.0, 1.0]),
                np.array([0.7, -0.2]),
            ):
                step = 1e-6
                finite_difference = (
                    decay.residual(point + step * direction)
                    - decay.residual(point - step * direction)
                ) / (2 * step)
                np.testing.assert_allclose(
                    jacobian @ direction, finite_difference, rtol=1e-7, atol=1e-9
                )

    def test_cstr_filter_carries_derivative_and_antiwindup_recovers_saturation(
        self,
    ) -> None:
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

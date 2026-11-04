# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent original-equation controls for the selected exchanger references."""

from __future__ import annotations

import math
import unittest

from oracles.ordinary_exchanger import observations, solve_exchanger


class OrdinaryExchangerReferenceTests(unittest.TestCase):
    """Complete calculations still need the separate native differential comparison."""

    def test_reference_design_rating_and_arithmetic_keep_original_balances(
        self,
    ) -> None:
        for arithmetic, rating in ((False, False), (False, True), (True, False)):
            with self.subTest(arithmetic=arithmetic, rating=rating):
                output = observations(
                    solve_exchanger(arithmetic=arithmetic, rating=rating)
                )
                self.assertEqual(len(output), 25)
                for side, flow in (("hot", 5.0), ("cold", 1.0)):
                    self.assertAlmostEqual(
                        output[f"exchanger.{side}.inlet.flow"], flow, places=8
                    )
                    self.assertAlmostEqual(
                        output[f"exchanger.{side}.outlet.flow"], flow, places=8
                    )
                    difference = (
                        output[f"exchanger.{side}.outlet.energy_flow"]
                        - output[f"exchanger.{side}.inlet.energy_flow"]
                    )
                    self.assertAlmostEqual(
                        difference, output[f"ordinary {side} duty"], places=5
                    )
                self.assertAlmostEqual(
                    output["ordinary hot duty"] + output["ordinary cold duty"],
                    0.0,
                    places=5,
                )
                first = (
                    output["exchanger.hot.inlet.T"] - output["exchanger.cold.inlet.T"]
                )
                second = (
                    output["exchanger.hot.outlet.T"] - output["exchanger.cold.outlet.T"]
                )
                driving = (
                    (first + second) / 2
                    if arithmetic
                    else (first - second) / math.log(first / second)
                )
                self.assertAlmostEqual(
                    output["ordinary cold duty"],
                    100 * output["solved heat-transfer area"] * driving,
                    places=5,
                )
                if rating:
                    self.assertAlmostEqual(
                        output["solved heat-transfer area"], 1.0, places=10
                    )
                else:
                    self.assertAlmostEqual(
                        output["exchanger.cold.outlet.T"], 331.6111111111111, places=8
                    )


if __name__ == "__main__":
    unittest.main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent original-equation controls for the selected exchanger references."""

from __future__ import annotations

import math
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from oracles.ordinary_exchanger import observations, solve_exchanger
from oracles.reference_environment import ReferenceEnvironment, prepared_solver


class OrdinaryExchangerReferenceTests(unittest.TestCase):
    """Complete calculations still need the separate native differential comparison."""

    def test_prepared_ipopt_reaches_the_initializer_and_direct_solves(self) -> None:
        """The observed initializer and direct solver all bypass a PATH substitute."""
        environment = ReferenceEnvironment.from_process()
        prepared = prepared_solver("ipopt", "exchanger test")
        with tempfile.TemporaryDirectory() as temporary:
            substitute = Path(temporary) / "ipopt"
            marker = Path(temporary) / "used"
            substitute.write_text(f"#!/bin/sh\ntouch '{marker}'\nexit 99\n")
            substitute.chmod(0o700)
            with patch.dict(
                os.environ, {"PATH": f"{temporary}{os.pathsep}{os.environ['PATH']}"}
            ):
                solve_exchanger(environment=environment)
            self.assertFalse(marker.exists())
        for stage, count in (("initializer", "14"), ("design", "1"), ("costing", "1")):
            name = f"exchanger.{stage}"
            self.assertEqual(
                environment.evidence[f"{name}.executable"], str(prepared.path)
            )
            self.assertEqual(environment.evidence[f"{name}.sha256"], prepared.sha256)
            self.assertTrue(environment.evidence[f"{name}.release"])
            self.assertEqual(environment.evidence[f"{name}.solves"], count)

    def test_reference_design_rating_and_arithmetic_keep_original_balances(
        self,
    ) -> None:
        """Reference design rating and arithmetic keep original balances."""
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

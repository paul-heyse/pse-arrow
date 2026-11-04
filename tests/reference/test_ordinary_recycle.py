# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent original-equation and failed-stage controls for ordinary BT."""

from __future__ import annotations

import json
import unittest

from pyomo.environ import Constraint, Var, value
from pyomo.opt import TerminationCondition

from oracles.ordinary_recycle import (
    build_recycle,
    observations,
    overheated_failure,
    overheated_observations,
    solve_recycle,
)


class OrdinaryRecycleReferenceTests(unittest.TestCase):
    """These calculations do not claim a native differential comparison."""

    def test_actual_tear_then_original_preserve_components_energy_and_bounds(
        self,
    ) -> None:
        solution = solve_recycle()
        output = observations(solution)
        self.assertEqual(len(output), 14)
        self.assertEqual(
            solution.tear_result.solver.termination_condition,
            TerminationCondition.optimal,
        )
        self.assertEqual(
            solution.original_result.solver.termination_condition,
            TerminationCondition.optimal,
        )
        fs = solution.model.fs
        self.assertEqual(set(fs.recycled[0].phase_list), {"Liq"})
        self.assertEqual(set(fs.splitter.mixed_state[0].phase_list), {"Liq"})
        for species in ("benzene", "toluene"):
            self.assertAlmostEqual(output[f"ordinary feed {species}"], 0.5, places=12)
            self.assertAlmostEqual(
                output[f"ordinary feed {species}"],
                output[f"ordinary vapor {species}"]
                + output[f"ordinary purge {species}"],
                places=8,
            )
        self.assertAlmostEqual(
            output["ordinary feed flow"],
            output["vapor product"] + output["ordinary purge flow"],
            places=8,
        )
        self.assertAlmostEqual(
            output["ordinary feed energy"] + output["ordinary heater duty"],
            output["ordinary vapor energy"] + output["ordinary purge energy"],
            places=5,
        )
        self.assertAlmostEqual(
            output["liquid recycle"], 3 * output["ordinary purge flow"], places=8
        )
        self.assertGreater(output["vapor product"], 0.0)
        self.assertGreater(output["ordinary purge flow"], 0.0)
        self.assertAlmostEqual(
            value(fs.heater.control_volume.properties_out[0].temperature),
            368.0,
            places=10,
        )
        self.assertAlmostEqual(value(fs.flash.heat_duty[0]), 0.0, places=12)
        self.assertTrue(fs.closed_recycle.active)
        self.assertFalse(fs.tear_connection.active)
        for accepted in (solution.tear, solution.model):
            for row in accepted.component_data_objects(Constraint, active=True):
                body = float(value(row.body))
                if row.has_lb():
                    self.assertGreaterEqual(
                        body, float(value(row.lower)) - 1e-5, row.name
                    )
                if row.has_ub():
                    self.assertLessEqual(body, float(value(row.upper)) + 1e-5, row.name)
            for coordinate in accepted.component_data_objects(Var):
                actual = float(value(coordinate))
                if coordinate.has_lb():
                    self.assertGreaterEqual(
                        actual, float(value(coordinate.lb)) - 1e-7, coordinate.name
                    )
                if coordinate.has_ub():
                    self.assertLessEqual(
                        actual, float(value(coordinate.ub)) + 1e-7, coordinate.name
                    )
        print("ORDINARY_RECYCLE_REFERENCE=" + json.dumps(output, sort_keys=True))

    def test_actual_overheated_stage_fails_without_changing_the_original(self) -> None:
        failure = overheated_failure()
        self.assertEqual(
            failure.result.solver.termination_condition, TerminationCondition.infeasible
        )
        output = overheated_observations(failure)
        self.assertEqual(output["source temperature"], 500.0)
        self.assertEqual(output["source caloric temperature"], 368.0)
        self.assertEqual(output["receiver upper bound"], 450.0)
        self.assertLessEqual(output["receiver temperature"], 450.0 + 1e-8)
        self.assertLessEqual(output["temperature connection residual"], -49.999)
        self.assertGreater(abs(output["energy transport residual"]), 1.0)
        for species in ("benzene", "toluene"):
            self.assertEqual(output[f"source {species}"], 0.005)
        self.assertEqual(failure.original_before, failure.original_after)
        self.assertTrue(failure.original.fs.closed_recycle.active)
        self.assertFalse(failure.original.fs.tear_connection.active)
        self.assertFalse(failure.stage.fs.closed_recycle.active)
        self.assertTrue(failure.stage.fs.tear_connection.active)
        untouched = build_recycle()
        initial = {
            coordinate.name: float(value(coordinate))
            for coordinate in untouched.component_data_objects(Var)
        }
        actual = {
            coordinate.name: float(value(coordinate))
            for coordinate in failure.original.component_data_objects(Var)
        }
        self.assertEqual(
            actual,
            initial,
            "a rejected stage cannot commit any primal coordinate to the original",
        )
        print(
            "ORDINARY_OVERHEATED_REFERENCE="
            + json.dumps(
                {
                    "stage": "overheated_tear",
                    "solver_status": str(failure.result.solver.status),
                    "termination": str(failure.result.solver.termination_condition),
                    "message": str(failure.result.solver.message),
                    "original_committed": False,
                    **output,
                },
                sort_keys=True,
            )
        )


if __name__ == "__main__":
    unittest.main()

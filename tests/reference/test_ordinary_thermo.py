# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Decisive input/datum controls before complete ordinary process comparisons."""

from __future__ import annotations

import math
import unittest

from idaes.models.properties.modular_properties.base.generic_property import (
    GenericParameterBlock,
)
from idaes.models.properties.modular_properties.pure import Perrys, RPP5
from oracles.ordinary_thermo import SPECIES, Wagner25, configuration, load_inputs
from pyomo.environ import ConcreteModel, units, value


class OrdinaryThermodynamicReferenceTests(unittest.TestCase):
    """The oracle uses the selected publication values and explicit datum."""

    def test_source_population_and_distinct_pressure_critical_coordinates(self) -> None:
        sources = load_inputs()
        self.assertEqual(set(sources), set(SPECIES))
        self.assertEqual(
            sources["benzene"].liquid_cp_mol, (129.44, -0.1695, 0.00064781, 0.0, 0.0)
        )
        self.assertEqual(sources["benzene"].vaporization_298, 33830.0)
        self.assertEqual(sources["toluene"].vaporization_298, 38010.0)
        for source in sources.values():
            self.assertNotEqual(source.wagner[4], source.critical_temperature)
            self.assertNotEqual(source.wagner[5], source.critical_pressure)

    def test_actual_idaes_methods_preserve_gas_five_terms_and_liquid_datum(
        self,
    ) -> None:
        model = ConcreteModel()
        model.properties = GenericParameterBlock(**configuration("ideal"))
        self.assertEqual(value(model.properties.pressure_ref), 100000.0)
        self.assertEqual(value(model.properties.temperature_ref), 298.15)
        model.states = model.properties.build_state_block([0], defined_state=True)
        state = model.states[0]
        for name, source in load_inputs().items():
            component = model.properties.get_component(name)
            gas_origin = value(
                RPP5.enth_mol_ig_comp.return_expression(
                    state, component, 298.15 * units.K
                )
            )
            liquid_origin = value(
                Perrys.enth_mol_liq_comp.return_expression(
                    state, component, 298.15 * units.K
                )
            )
            self.assertEqual(gas_origin, 0.0)
            self.assertAlmostEqual(liquid_origin, -source.vaporization_298, places=8)
            for temperature in (300.0, 368.0, 425.0):
                cp = (
                    sum(c * temperature**i for i, c in enumerate(source.gas_cp_kmol))
                    / 1000
                )
                enthalpy = (
                    sum(
                        c * (temperature ** (i + 1) - 298.15 ** (i + 1)) / (i + 1)
                        for i, c in enumerate(source.gas_cp_kmol)
                    )
                    / 1000
                )
                entropy = (
                    source.gas_cp_kmol[0] * math.log(temperature / 298.15)
                    + sum(
                        c * (temperature**i - 298.15**i) / i
                        for i, c in enumerate(source.gas_cp_kmol)
                        if i
                    )
                ) / 1000
                self.assertAlmostEqual(
                    value(
                        RPP5.cp_mol_ig_comp.return_expression(
                            state, component, temperature * units.K
                        )
                    ),
                    cp,
                    places=9,
                )
                self.assertAlmostEqual(
                    value(
                        RPP5.enth_mol_ig_comp.return_expression(
                            state, component, temperature * units.K
                        )
                    ),
                    enthalpy,
                    places=8,
                )
                self.assertAlmostEqual(
                    value(
                        RPP5.entr_mol_ig_comp.return_expression(
                            state, component, temperature * units.K
                        )
                    ),
                    entropy,
                    places=9,
                )
                derivative = value(
                    Wagner25.return_expression(
                        state, component, temperature * units.K, dT=True
                    )
                )
                delta = 0.001
                finite_difference = (
                    value(
                        Wagner25.return_expression(
                            state, component, (temperature + delta) * units.K
                        )
                    )
                    - value(
                        Wagner25.return_expression(
                            state, component, (temperature - delta) * units.K
                        )
                    )
                ) / (2 * delta)
                self.assertAlmostEqual(derivative / finite_difference, 1.0, places=8)

    def test_single_phase_and_pr_are_actual_distinct_parameter_blocks(self) -> None:
        for kind in ("liquid", "pr"):
            model = ConcreteModel()
            model.properties = GenericParameterBlock(**configuration(kind))
            model.states = model.properties.build_state_block([0], defined_state=True)
            self.assertEqual(
                set(model.properties.phase_list),
                {"Liq"} if kind == "liquid" else {"Liq", "Vap"},
            )
            if kind == "pr":
                state = model.states[0]
                self.assertAlmostEqual(value(state.eps_1_Vap_Liq), 0.01)
                self.assertAlmostEqual(value(state.eps_2_Vap_Liq), 0.0005)


if __name__ == "__main__":
    unittest.main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES realization of the three selected ordinary PR exchangers.

These are reference calculations, without a candidate import or verdict. The
native/reference output mapping and complete comparison are separate consumers.
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING

import idaes.logger as idaeslog
from idaes.core import FlowsheetBlock, UnitModelCostingBlock
from idaes.core.util.model_statistics import degrees_of_freedom
from idaes.models.costing.SSLW import (
    HXMaterial,
    HXTubeLength,
    HXType,
    SSLWCosting,
    SSLWCostingData,
)
from idaes.models.properties.modular_properties.base.generic_property import (
    GenericParameterBlock,
)
from idaes.models.unit_models.heat_exchanger import (
    HeatExchanger,
    HeatExchangerFlowPattern,
    delta_temperature_amtd_callback,
    delta_temperature_lmtd_callback,
)
from pyomo.core.expr.numeric_expr import NumericValue
from pyomo.environ import (
    ConcreteModel,
    assert_optimal_termination,
    units,
    value,
)

from .ordinary_thermo import SPECIES, SpeciesInput, configuration
from .reference_environment import ReferenceEnvironment

if TYPE_CHECKING:
    from collections.abc import Mapping


def build_exchanger(
    *,
    arithmetic: bool = False,
    rating: bool = False,
    inputs: Mapping[str, SpeciesInput] | None = None,
) -> ConcreteModel:
    """Build the original fixed specification, without initialization or a solve."""
    if arithmetic and rating:
        raise ValueError("the selected rating case uses the logarithmic closure")
    model = ConcreteModel()
    model.fs = FlowsheetBlock(dynamic=False)
    model.fs.properties = GenericParameterBlock(**configuration("pr", inputs=inputs))
    model.fs.exchanger = HeatExchanger(
        hot_side={
            "property_package": model.fs.properties,
            "has_phase_equilibrium": True,
            "has_pressure_change": True,
        },
        cold_side={
            "property_package": model.fs.properties,
            "has_phase_equilibrium": True,
            "has_pressure_change": True,
        },
        flow_pattern=HeatExchangerFlowPattern.cocurrent,
        delta_temperature_callback=delta_temperature_amtd_callback
        if arithmetic
        else delta_temperature_lmtd_callback,
    )
    exchanger = model.fs.exchanger
    for side, flow, temperature in (
        (exchanger.hot_side, 5.0, 365.0),
        (exchanger.cold_side, 1.0, 300.0),
    ):
        inlet = side.properties_in[0]
        inlet.flow_mol.fix(flow)
        inlet.temperature.fix(temperature)
        inlet.pressure.fix(101325.0)
        for component in SPECIES:
            inlet.mole_frac_comp[component].fix(0.5)
        side.deltaP.fix(0.0)
        for state in (inlet, side.properties_out[0]):
            state.temperature_bubble["Vap", "Liq"].set_value(365.0)
            state.temperature_dew["Vap", "Liq"].set_value(370.0)
            state.eps_1_Vap_Liq.set_value(0.01)
            state.eps_2_Vap_Liq.set_value(0.0005)
    exchanger.overall_heat_transfer_coefficient.fix(100.0)
    exchanger.area.set_value(1.0)
    exchanger.hot_side.properties_out[0].temperature.set_value(360.0)
    if rating:
        exchanger.area.fix(1.0)
        exchanger.cold_side.properties_out[0].temperature.set_value(330.0)
    else:
        exchanger.cold_side.properties_out[0].temperature.fix(331.6111111111111)
    exchanger.delta_temperature_in[0].set_value(65.0)
    exchanger.delta_temperature_out[0].set_value(30.0)
    if degrees_of_freedom(model) != 0:
        raise RuntimeError("the selected independent exchanger is not square")
    return model


def fixed_inputs(model: ConcreteModel, *, rating: bool) -> dict[str, NumericValue]:
    """Select authored fixed-header roles from an untouched reference model."""
    exchanger = model.fs.exchanger
    result: dict[str, NumericValue] = {}
    for name, side in (("hot", exchanger.hot_side), ("cold", exchanger.cold_side)):
        state = side.properties_in[0]
        prefix = f"exchanger.{name}"
        result.update(
            {
                f"{prefix}.inlet.flow": state.flow_mol,
                f"{prefix}.inlet.T": state.temperature,
                f"{prefix}.inlet.pressure": state.pressure,
                f"{prefix}.delta_pressure": side.deltaP[0],
                **{
                    f"{prefix}.inlet.z[{species}]": state.mole_frac_comp[species]
                    for species in SPECIES
                },
            }
        )
    result["exchanger.coefficient"] = exchanger.overall_heat_transfer_coefficient[0]
    result["exchanger.area" if rating else "exchanger.cold.outlet.T"] = (
        exchanger.area if rating else exchanger.cold_side.properties_out[0].temperature
    )
    if len(result) != 14 or not all(
        coordinate.is_fixed() for coordinate in result.values()
    ):
        raise RuntimeError(
            "the original independent exchanger fixed population differs"
        )
    return result


def solve_exchanger(
    *,
    arithmetic: bool = False,
    rating: bool = False,
    environment: ReferenceEnvironment | None = None,
    inputs: Mapping[str, SpeciesInput] | None = None,
) -> ConcreteModel:
    """Solve the original design, then optionally its area-fixed rating question."""
    if arithmetic and rating:
        raise ValueError("the selected rating case uses the logarithmic closure")
    if environment is None:
        environment = ReferenceEnvironment.from_process()
    model = build_exchanger(arithmetic=arithmetic, inputs=inputs)
    exchanger = model.fs.exchanger
    options = {"tol": 1e-10, "max_iter": 1000, "max_cpu_time": 300}

    def solve(stage: str) -> None:
        solver = environment.solver("ipopt", stage)
        solver.options.update(options)
        assert_optimal_termination(solver.solve(model))
        environment.require_solves(stage, 1)

    # The unit initializer's side-by-side duty construction expects a rating
    # specification. Use that only to obtain numerical starts, then restore and
    # solve the actual original design before any reference output is readable.
    exchanger.cold_side.properties_out[0].temperature.unfix()
    exchanger.area.fix(1.0)
    alias = "pse_reference_exchanger_initializer"
    with environment.registration("ipopt", "exchanger.initializer", alias=alias):
        # IDAES 2.13.0 forwards this name to both control volumes and on to
        # their inlet and outlet state initializers. Every shell command issued
        # by the resulting solver instances is checked by the same witness.
        exchanger.initialize(solver=alias, optarg=options, outlvl=idaeslog.ERROR)
    environment.require_solves("exchanger.initializer", 14)
    exchanger.area.unfix()
    exchanger.cold_side.properties_out[0].temperature.fix(331.6111111111111)
    solve("exchanger.design")
    if rating:
        # Transfer only the accepted same-model primal state into the unchanged
        # rating specification; no source result supplies a target answer.
        exchanger.area.fix(1.0)
        exchanger.cold_side.properties_out[0].temperature.unfix()
        if degrees_of_freedom(model) != 0:
            raise RuntimeError("the independent rating specification is not square")
        solve("exchanger.rating")
    model.fs.costing = SSLWCosting()
    exchanger.costing = UnitModelCostingBlock(
        flowsheet_costing_block=model.fs.costing,
        costing_method=SSLWCostingData.cost_heat_exchanger,
        costing_method_arguments={
            "hx_type": HXType.Utube,
            "material_type": HXMaterial.StainlessSteelStainlessSteel,
            "tube_length": HXTubeLength.TwelveFoot,
        },
    )
    exchanger.costing.number_of_units.fix(1)
    exchanger.costing.hx_oversize.set_value(1.1)
    model.fs.costing.cost_process()
    model.fs.costing.initialize()
    solve("exchanger.costing")
    return model


def observations(model: ConcreteModel) -> dict[str, float]:
    """Read the entire agreed exchanger scalar population from the solved model."""
    exchanger = model.fs.exchanger
    result: dict[str, float] = {}
    for side_name, side in (("hot", exchanger.hot_side), ("cold", exchanger.cold_side)):
        for position, state in (
            ("inlet", side.properties_in[0]),
            ("outlet", side.properties_out[0]),
        ):
            prefix = f"exchanger.{side_name}.{position}"
            flow = float(value(state.flow_mol))
            enthalpy = float(value(state.enth_mol))
            result.update(
                {
                    f"{prefix}.flow": flow,
                    f"{prefix}.T": float(value(state.temperature)),
                    f"{prefix}.h": enthalpy,
                    f"{prefix}.energy_flow": flow * enthalpy,
                }
            )
    cost = exchanger.costing
    currency = units.USD_CE500
    result.update(
        {
            "ordinary hot duty": float(value(exchanger.hot_side.heat[0])),
            "ordinary cold duty": float(value(exchanger.cold_side.heat[0])),
            "ordinary heat transfer coefficient": float(
                value(exchanger.overall_heat_transfer_coefficient[0])
            ),
            "solved heat-transfer area": float(value(exchanger.area)),
            "area in square feet": float(
                value(units.convert(exchanger.area, to_units=units.ft**2))
            ),
            "base purchase cost": float(
                value(units.convert(cost.base_cost_per_unit, to_units=currency))
            ),
            "ordinary material factor": float(value(cost.material_factor)),
            "ordinary pressure factor": float(value(cost.pressure_factor)),
            "capital cost at CEPCI 500": float(
                value(units.convert(cost.capital_cost, to_units=currency))
            ),
        }
    )
    if len(result) != 25 or not all(
        math.isfinite(number) for number in result.values()
    ):
        raise RuntimeError(
            "independent exchanger output population is incomplete or nonfinite"
        )
    return result

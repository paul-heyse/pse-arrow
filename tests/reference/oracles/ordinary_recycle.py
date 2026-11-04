# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES realization of the ordinary ideal BT recycle and tear.

The liquid recycle has its own liquid-only state owner. The mixer equations
express conservation between those distinct property owners; a two-phase mixer
inlet would represent a different process. Candidate code and values are absent.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

from idaes.core import FlowsheetBlock, MaterialBalanceType
from idaes.core.util.model_statistics import degrees_of_freedom
from idaes.models.properties.modular_properties.base.generic_property import (
    GenericParameterBlock,
    GenericStateBlockData,
)
from idaes.models.unit_models import Flash, Heater, Separator
from idaes.models.unit_models.separator import SplittingType
from pyomo.core.base.block import BlockData
from pyomo.environ import (
    Block,
    ConcreteModel,
    Constraint,
    Expression,
    Param,
    SolverFactory,
    Var,
    assert_optimal_termination,
    units,
    value,
)
from pyomo.opt import SolverResults, TerminationCondition

from .ordinary_thermo import SPECIES, configuration


@dataclass(frozen=True)
class RecycleSolution:
    """Accepted tear and separately solved unchanged original specification."""

    model: ConcreteModel
    tear: ConcreteModel
    tear_result: SolverResults
    original_result: SolverResults


@dataclass(frozen=True)
class OverheatedFailure:
    """Actual failed stage, retained separately from the untouched original."""

    original: ConcreteModel
    stage: ConcreteModel
    result: SolverResults
    original_before: tuple[float, ...]
    original_after: tuple[float, ...]


def _connect(
    connection: BlockData,
    source: GenericStateBlockData,
    target: GenericStateBlockData,
) -> None:
    connection.flow = Constraint(expr=target.flow_mol == source.flow_mol)
    connection.temperature = Constraint(expr=target.temperature == source.temperature)
    connection.pressure = Constraint(expr=target.pressure == source.pressure)
    connection.composition = Constraint(
        list(SPECIES),
        rule=lambda _block, species: (
            target.mole_frac_comp[species] == source.mole_frac_comp[species]
        ),
    )


def _states(model: ConcreteModel) -> tuple[GenericStateBlockData, ...]:
    fs = model.fs
    return (
        fs.fresh[0],
        fs.recycled[0],
        fs.mixed[0],
        fs.tear_caloric[0],
        fs.heater.control_volume.properties_in[0],
        fs.heater.control_volume.properties_out[0],
        fs.flash.control_volume.properties_in[0],
        fs.flash.control_volume.properties_out[0],
        fs.splitter.mixed_state[0],
        fs.splitter.purge_state[0],
        fs.splitter.recycle_state[0],
    )


def _energy(state: GenericStateBlockData) -> float:
    return float(value(state.flow_mol * state.enth_mol))


def original_specification(model: ConcreteModel) -> tuple[float, ...]:
    """Read selected specifications and unchanged receiver bounds, not outputs."""
    fs = model.fs
    return tuple(
        float(value(item))
        for item in (
            fs.fresh[0].flow_mol,
            fs.fresh[0].temperature,
            fs.fresh[0].pressure,
            *(fs.fresh[0].mole_frac_comp[j] for j in SPECIES),
            fs.heater.control_volume.properties_out[0].temperature,
            fs.heater.deltaP[0],
            fs.flash.heat_duty[0],
            fs.flash.deltaP[0],
            fs.splitter.split_fraction[0, "purge"],
            fs.recycled[0].temperature.lb,
            fs.recycled[0].temperature.ub,
        )
    )


def build_recycle() -> ConcreteModel:
    """Build the original closed process, with its inactive explicit tear stage."""
    model = ConcreteModel()
    model.fs = FlowsheetBlock(dynamic=False)
    fs = model.fs
    fs.ideal = GenericParameterBlock(**configuration("ideal"))
    fs.liquid = GenericParameterBlock(**configuration("liquid"))
    fs.fresh = fs.ideal.build_state_block([0], defined_state=True)
    fs.recycled = fs.liquid.build_state_block([0], defined_state=True)
    fs.mixed = fs.ideal.build_state_block([0], defined_state=False)
    fs.tear_caloric = fs.liquid.build_state_block([0], defined_state=True)
    for state, flow in ((fs.fresh[0], 1.0), (fs.tear_caloric[0], 0.01)):
        state.flow_mol.fix(flow)
        state.temperature.fix(368.0)
        state.pressure.fix(101325.0)
        for species in SPECIES:
            state.mole_frac_comp[species].fix(0.5)
    fs.mixing_components = Constraint(
        list(SPECIES),
        rule=lambda _fs, j: (
            fs.mixed[0].flow_mol * fs.mixed[0].mole_frac_comp[j]
            == fs.fresh[0].flow_mol * fs.fresh[0].mole_frac_comp[j]
            + fs.recycled[0].flow_mol * fs.recycled[0].mole_frac_comp[j]
        ),
    )
    fs.mixing_energy = Constraint(
        expr=fs.mixed[0].flow_mol * fs.mixed[0].enth_mol
        == fs.fresh[0].flow_mol * fs.fresh[0].enth_mol
        + fs.recycled[0].flow_mol * fs.recycled[0].enth_mol
    )
    fs.mixing_pressure = Constraint(expr=fs.mixed[0].pressure == fs.fresh[0].pressure)
    fs.heater = Heater(
        property_package=fs.ideal,
        has_phase_equilibrium=True,
        has_pressure_change=True,
        material_balance_type=MaterialBalanceType.componentTotal,
    )
    fs.heater.control_volume.properties_out[0].temperature.fix(368.0)
    fs.heater.deltaP.fix(0.0)
    fs.flash = Flash(
        property_package=fs.ideal,
        has_heat_transfer=True,
        has_pressure_change=True,
        material_balance_type=MaterialBalanceType.componentTotal,
    )
    fs.flash.heat_duty.fix(0.0)
    fs.flash.deltaP.fix(0.0)
    fs.splitter = Separator(
        property_package=fs.liquid,
        outlet_list=["purge", "recycle"],
        split_basis=SplittingType.totalFlow,
        material_balance_type=MaterialBalanceType.componentTotal,
    )
    fs.splitter.split_fraction[0, "purge"].fix(0.25)
    fs.mixed_connection = Block()
    _connect(
        fs.mixed_connection, fs.mixed[0], fs.heater.control_volume.properties_in[0]
    )
    fs.heated_connection = Block()
    _connect(
        fs.heated_connection,
        fs.heater.control_volume.properties_out[0],
        fs.flash.control_volume.properties_in[0],
    )
    flashed = fs.flash.control_volume.properties_out[0]
    liquid = fs.splitter.mixed_state[0]
    fs.liquid_flow = Constraint(expr=liquid.flow_mol == flashed.flow_mol_phase["Liq"])
    fs.liquid_temperature = Constraint(expr=liquid.temperature == flashed.temperature)
    fs.liquid_pressure = Constraint(expr=liquid.pressure == flashed.pressure)
    fs.liquid_composition = Constraint(
        list(SPECIES),
        rule=lambda _fs, j: (
            liquid.mole_frac_comp[j] == flashed.mole_frac_phase_comp["Liq", j]
        ),
    )
    fs.closed_recycle = Block()
    _connect(fs.closed_recycle, fs.splitter.recycle_state[0], fs.recycled[0])
    fs.tear_temperature = Param(initialize=368.0, mutable=True, units=units.K)
    fs.tear_energy = Expression(
        expr=fs.tear_caloric[0].flow_mol * fs.tear_caloric[0].enth_mol
    )
    fs.tear_connection = Block()
    _connect(fs.tear_connection, fs.tear_caloric[0], fs.recycled[0])
    fs.tear_connection.temperature.set_value(
        fs.recycled[0].temperature == fs.tear_temperature
    )
    # F,z,T,P already determine the same liquid energy. Keep the independent
    # transport requirement in acceptance rather than duplicating its NLP row.
    fs.tear_connection.deactivate()
    for state in _states(model):
        if "Vap" in state.phase_list:
            state.temperature_bubble["Vap", "Liq"].set_value(365.0)
            state.temperature_dew["Vap", "Liq"].set_value(370.0)
            state.eps_1_Vap_Liq.set_value(0.01)
            state.eps_2_Vap_Liq.set_value(0.0005)
    if degrees_of_freedom(model) != 0:
        raise RuntimeError("the independent original recycle is not square")
    return model


def _solve(model: ConcreteModel) -> SolverResults:
    solver = SolverFactory("ipopt")
    if not solver.available(exception_flag=True):
        raise RuntimeError("the selected independent recycle requires Ipopt")
    solver.options.update(
        {
            "tol": 1e-10,
            "max_iter": 1000,
            "max_cpu_time": 120,
            "bound_relax_factor": 0.0,
        }
    )
    return solver.solve(model, load_solutions=False)


def _tear_model(original: ConcreteModel, temperature: float) -> ConcreteModel:
    stage = original.clone()
    stage.fs.closed_recycle.deactivate()
    stage.fs.tear_connection.activate()
    stage.fs.tear_temperature.set_value(temperature)
    # The source caloric owner remains at368K even for the500K port coordinate.
    if degrees_of_freedom(stage) != 0:
        raise RuntimeError("the independent tear stage is not square")
    return stage


def solve_recycle() -> RecycleSolution:
    """Solve the actual tear, then transfer only its primal starts to the original."""
    original = build_recycle()
    before = original_specification(original)
    tear = _tear_model(original, 368.0)
    tear_result = _solve(tear)
    assert_optimal_termination(tear_result)
    tear.solutions.load_from(tear_result)
    source, receiver = tear.fs.tear_caloric[0], tear.fs.recycled[0]
    for species in SPECIES:
        residual = value(
            receiver.flow_mol * receiver.mole_frac_comp[species]
            - source.flow_mol * source.mole_frac_comp[species]
        )
        if abs(residual) > 1e-9:
            raise RuntimeError("accepted tear violates original component transport")
    if (
        abs(value(receiver.temperature - tear.fs.tear_temperature)) > 1e-8
        or abs(value(receiver.pressure - source.pressure)) > 1e-8
    ):
        raise RuntimeError(
            "accepted tear violates original temperature/pressure coordinates"
        )
    if (
        not float(value(receiver.temperature.lb))
        <= value(receiver.temperature)
        <= float(value(receiver.temperature.ub))
    ):
        raise RuntimeError("accepted tear violates the original receiver bounds")
    if abs(_energy(tear.fs.recycled[0]) - float(value(tear.fs.tear_energy))) > 1e-6:
        raise RuntimeError(
            "accepted tear violates the original liquid energy transport"
        )
    for coordinate in tear.component_data_objects(Var):
        target = original.find_component(coordinate.name)
        if not target.fixed:
            target.set_value(value(coordinate))
    if original_specification(original) != before:
        raise RuntimeError("tear transfer changed the original recycle specification")
    result = _solve(original)
    assert_optimal_termination(result)
    original.solutions.load_from(result)
    if original_specification(original) != before:
        raise RuntimeError("original solve changed the recycle specification")
    return RecycleSolution(original, tear, tear_result, result)


def observations(solution: RecycleSolution) -> dict[str, float]:
    """Read all fourteen required scalar reports from the accepted original."""
    fs = solution.model.fs
    fresh = fs.fresh[0]
    flashed = fs.flash.control_volume.properties_out[0]
    purge = fs.splitter.purge_state[0]
    output = {
        "ordinary feed flow": float(value(fresh.flow_mol)),
        "vapor product": float(value(flashed.flow_mol_phase["Vap"])),
        "ordinary purge flow": float(value(purge.flow_mol)),
        "liquid recycle": float(value(fs.recycled[0].flow_mol)),
        "ordinary feed energy": _energy(fresh),
        "ordinary heater duty": float(value(fs.heater.heat_duty[0])),
        "ordinary vapor energy": float(
            value(flashed.flow_mol_phase["Vap"] * flashed.enth_mol_phase["Vap"])
        ),
        "ordinary purge energy": _energy(purge),
    }
    for species in SPECIES:
        output[f"ordinary feed {species}"] = float(
            value(fresh.flow_mol * fresh.mole_frac_comp[species])
        )
        output[f"ordinary vapor {species}"] = float(
            value(
                flashed.flow_mol_phase["Vap"]
                * flashed.mole_frac_phase_comp["Vap", species]
            )
        )
        output[f"ordinary purge {species}"] = float(
            value(purge.flow_mol * purge.mole_frac_comp[species])
        )
    if len(output) != 14 or not all(math.isfinite(item) for item in output.values()):
        raise RuntimeError(
            "independent recycle scalar population is incomplete or nonfinite"
        )
    return output


def overheated_failure() -> OverheatedFailure:
    """Execute the500K-port stage without committing any candidate to the original."""
    original = build_recycle()
    before = original_specification(original)
    stage = _tear_model(original, 500.0)
    result = _solve(stage)
    if result.solver.termination_condition != TerminationCondition.infeasible:
        raise RuntimeError(
            f"overheated stage did not reach its intended numerical failure: {result.solver}"
        )
    if len(result.solution) == 0:
        raise RuntimeError(
            "the failed overheated stage retained no numerical candidate"
        )
    stage.solutions.load_from(result)
    after = original_specification(original)
    if after != before:
        raise RuntimeError("failed stage changed the original recycle specification")
    return OverheatedFailure(original, stage, result, before, after)


def overheated_observations(failure: OverheatedFailure) -> dict[str, float]:
    """Retain the actual failed receiver and both original transport residuals."""
    fs = failure.stage.fs
    receiver = fs.recycled[0]
    return {
        "source temperature": float(value(fs.tear_temperature)),
        "receiver temperature": float(value(receiver.temperature)),
        "receiver upper bound": float(value(receiver.temperature.ub)),
        "temperature connection residual": float(
            value(receiver.temperature - fs.tear_temperature)
        ),
        "source energy": float(value(fs.tear_energy)),
        "source caloric temperature": float(value(fs.tear_caloric[0].temperature)),
        "receiver energy": _energy(receiver),
        "energy transport residual": _energy(receiver) - float(value(fs.tear_energy)),
        **{
            f"source {species}": float(
                value(
                    fs.tear_caloric[0].flow_mol
                    * fs.tear_caloric[0].mole_frac_comp[species]
                )
            )
            for species in SPECIES
        },
    }

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Consumed component surfaces of the dynamically constructed IDAES models.

These protocols describe Python component access, not scientific equations or
wire declarations. The actual IDAES constructors still build every component.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

from pyomo.environ import ConcreteModel

if TYPE_CHECKING:
    from collections.abc import Collection, Mapping

    from pyomo.core.base.constraint import ConstraintData
    from pyomo.core.base.param import ParamData
    from pyomo.core.base.var import IndexedVar, VarData
    from pyomo.core.expr.numeric_expr import NumericValue

    from .ordinary_thermo import WagnerComponent


class State(Protocol):
    """State coordinates and property expressions consumed by ordinary references."""

    flow_mol: VarData
    temperature: VarData
    pressure: VarData
    mole_frac_comp: Mapping[str, VarData]
    mole_frac_phase_comp: Mapping[tuple[str, str], VarData]
    flow_mol_phase: Mapping[str, NumericValue]
    enth_mol: NumericValue
    enth_mol_phase: Mapping[str, NumericValue]
    temperature_bubble: Mapping[tuple[str, str], VarData]
    temperature_dew: Mapping[tuple[str, str], VarData]
    eps_1_Vap_Liq: VarData
    eps_2_Vap_Liq: VarData
    phase_list: Collection[str]

    def is_property_constructed(self, name: str) -> bool:
        """Ask the actual IDAES block whether a lazy property exists."""
        ...


class States(Protocol):
    """State blocks indexed by the flowsheet time."""

    def __getitem__(self, index: int) -> State:
        """Read the selected original state block."""
        ...


class Parameters(Protocol):
    """The dynamic parameter block's selected construction and datum surface."""

    pressure_ref: NumericValue
    temperature_ref: NumericValue
    phase_list: Collection[str]

    def build_state_block(self, indices: list[int], *, defined_state: bool) -> States:
        """Construct actual IDAES states using this parameter owner."""
        ...

    def get_component(self, name: str) -> WagnerComponent:
        """Select the component with its installed pure-property parameters."""
        ...


class ControlVolume(Protocol):
    """Inlet/outlet state owners and extensive transfer variables."""

    properties_in: States
    properties_out: States
    deltaP: IndexedVar
    heat: IndexedVar


class Costing(Protocol):
    """The dynamically installed SSLW costing expressions consumed here."""

    number_of_units: VarData
    hx_oversize: ParamData
    base_cost_per_unit: NumericValue
    material_factor: NumericValue
    pressure_factor: NumericValue
    capital_cost: NumericValue

    def cost_process(self) -> None:
        """Build the actual aggregate flowsheet costing expressions."""
        ...

    def initialize(self) -> None:
        """Initialize the actual library costing variables."""
        ...


class Exchanger(Protocol):
    """The public heat-exchanger components selected by the ordinary cases."""

    hot_side: ControlVolume
    cold_side: ControlVolume
    overall_heat_transfer_coefficient: IndexedVar
    area: VarData
    delta_temperature_in: IndexedVar
    delta_temperature_out: IndexedVar
    costing: Costing

    def initialize(self, **settings: object) -> None:
        """Initialize the original unit using the witnessed solver alias."""
        ...


class Heater(Protocol):
    """Heater or flash component roles consumed by the recycle reference."""

    control_volume: ControlVolume
    deltaP: IndexedVar
    heat_duty: IndexedVar


class Splitter(Protocol):
    """Liquid splitter state owners and original purge specification."""

    mixed_state: States
    purge_state: States
    recycle_state: States
    split_fraction: IndexedVar


class Connection(Protocol):
    """An actual Pyomo block carrying the explicit connection equations."""

    flow: ConstraintData
    temperature: ConstraintData
    pressure: ConstraintData
    composition: object
    active: bool

    def activate(self) -> None:
        """Activate the original connection block."""
        ...

    def deactivate(self) -> None:
        """Deactivate the original connection block."""
        ...


class Flowsheet(Protocol):
    """Authored component graph built by the ordinary IDAES reference constructors."""

    properties: Parameters
    exchanger: Exchanger
    costing: Costing
    ideal: Parameters
    liquid: Parameters
    fresh: States
    recycled: States
    mixed: States
    tear_caloric: States
    mixing_components: object
    mixing_energy: object
    mixing_pressure: object
    heater: Heater
    flash: Heater
    splitter: Splitter
    mixed_connection: Connection
    heated_connection: Connection
    liquid_flow: object
    liquid_temperature: object
    liquid_pressure: object
    liquid_composition: object
    closed_recycle: Connection
    tear_temperature: ParamData
    tear_energy: NumericValue
    tear_connection: Connection


class OrdinaryModel(ConcreteModel):
    """Static component view over a real ConcreteModel; never constructed directly."""

    fs: Flowsheet
    properties: Parameters
    states: States

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Freeze one independently evaluated IDAES SRK vapor into declared Parquet schemas."""

from collections.abc import Collection, Mapping
from copy import deepcopy
from math import log
from pathlib import Path
from typing import Protocol, cast

import idaes
import pyarrow as pa
import pyarrow.parquet as pq
from idaes.models.properties.modular_properties.base import generic_property
from idaes.models.properties.modular_properties.eos.ceos import CubicType
from idaes.models.properties.modular_properties.examples.BT_PR import configuration
from pyomo.core.base.var import IndexedVar, VarData
from pyomo.core.expr.numeric_expr import NumericValue
from pyomo.environ import ConcreteModel, value

from scripts.frozen_reference import regenerate_selected, table_reference

CAS = {"benzene": "71-43-2", "toluene": "108-88-3"}


class SrkState(Protocol):
    """The exact dynamically generated state members consumed by this oracle."""

    flow_mol: VarData
    temperature: VarData
    pressure: VarData
    component_list: Collection[str]
    phase_component_set: Collection[tuple[str, str]]
    mole_frac_comp: IndexedVar
    mole_frac_phase_comp: IndexedVar
    dens_mol_phase: Mapping[str, NumericValue]
    compress_fact_phase: Mapping[str, NumericValue]
    fug_coeff_phase_comp: Mapping[tuple[str, str], NumericValue]


class SrkStates(Protocol):
    """Original indexed state block retained on the Pyomo model."""

    def __getitem__(self, index: int) -> SrkState: ...


class SrkParameters(Protocol):
    """The selected parameter block's state construction boundary."""

    def build_state_block(
        self, indices: list[int], *, defined_state: bool
    ) -> SrkStates: ...


class SrkFactory(Protocol):
    """IDAES registers this constructor through declare_process_block_class."""

    def __call__(self, **settings: object) -> SrkParameters: ...


def main() -> None:
    regenerate = regenerate_selected()
    if idaes.__version__ != "2.13.0":
        raise RuntimeError("the declared oracle requires IDAES 2.13.0")
    settings = deepcopy(configuration)
    for phase in settings["phases"].values():
        phase["equation_of_state_options"]["type"] = CubicType.SRK
    settings["parameter_data"]["SRK_kappa"] = settings["parameter_data"].pop("PR_kappa")
    model = ConcreteModel()
    parameter_block = cast(
        "SrkFactory", vars(generic_property)["GenericParameterBlock"]
    )
    parameters = parameter_block(**settings)
    model.parameters = parameters
    states = parameters.build_state_block([0], defined_state=True)
    model.states = states
    state = states[0]
    state.flow_mol.set_value(1)
    state.temperature.set_value(450)
    state.pressure.set_value(100000)
    for species in state.component_list:
        state.mole_frac_comp[species].set_value(0.5)
    for phase, species in state.phase_component_set:
        state.mole_frac_phase_comp[phase, species].set_value(0.5)
    metadata = {
        b"oracle_release": b"idaes-pse==2.13.0",
        b"generator": b"scripts/idaes_srk_reference.py",
    }
    state_schema = pa.schema(
        [
            pa.field("case", pa.int64()),
            *[pa.field(name, pa.float64()) for name in ("T", "rho", "pressure", "Z")],
        ],
        metadata=metadata,
    )
    component_schema = pa.schema(
        [
            pa.field("j", pa.string()),
            pa.field("fraction", pa.float64()),
            pa.field("ln_phi", pa.float64()),
        ],
        metadata=metadata,
    )
    destination = (
        Path(__file__).resolve().parents[1]
        / "packages/reference/data/oracles/idaes-2.13/data"
    )
    critical = {
        row["subject"]: (row["tc"], row["pc"], row["omega"])
        for row in pq.read_table(destination / "bt_critical.parquet").to_pylist()
    }
    for species, cas in CAS.items():
        parameters = settings["components"][species]["parameter_data"]
        actual = (
            parameters["temperature_crit"][0],
            parameters["pressure_crit"][0],
            parameters["omega"],
        )
        if critical.get(cas) != actual:
            raise ValueError(
                f"stale SRK critical input for {species}: {critical.get(cas)} != {actual}"
            )
    states = [
        {
            "case": 0,
            "T": value(state.temperature),
            "rho": value(state.dens_mol_phase["Vap"]),
            "pressure": value(state.pressure),
            "Z": value(state.compress_fact_phase["Vap"]),
        }
    ]
    components = [
        {
            "j": cas,
            "fraction": 0.5,
            "ln_phi": log(
                cast("float", value(state.fug_coeff_phase_comp["Vap", species]))
            ),
        }
        for species, cas in CAS.items()
    ]
    table_reference(
        destination / "srk_state.parquet",
        pa.Table.from_pylist(states, schema=state_schema),
        regenerate,
        {"rho": (2e-12, 1e-10), "Z": (2e-12, 1e-12)},
    )
    table_reference(
        destination / "srk_components.parquet",
        pa.Table.from_pylist(components, schema=component_schema),
        regenerate,
        {"ln_phi": (2e-12, 1e-12)},
    )


if __name__ == "__main__":
    main()

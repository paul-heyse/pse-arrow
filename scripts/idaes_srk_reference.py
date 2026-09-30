# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Freeze one independently evaluated IDAES SRK vapor into declared Parquet schemas."""

from copy import deepcopy
from math import log
from pathlib import Path

import idaes
import pyarrow as pa
import pyarrow.parquet as pq
from idaes.models.properties.modular_properties.base.generic_property import (
    GenericParameterBlock,
)
from idaes.models.properties.modular_properties.eos.ceos import CubicType
from idaes.models.properties.modular_properties.examples.BT_PR import configuration
from pyomo.environ import ConcreteModel, value


def main() -> None:
    if idaes.__version__ != "2.13.0":
        raise RuntimeError("the declared oracle requires IDAES 2.13.0")
    settings = deepcopy(configuration)
    for phase in settings["phases"].values():
        phase["equation_of_state_options"]["type"] = CubicType.SRK
    settings["parameter_data"]["SRK_kappa"] = settings["parameter_data"].pop("PR_kappa")
    model = ConcreteModel()
    model.parameters = GenericParameterBlock(**settings)
    model.states = model.parameters.build_state_block([0], defined_state=True)
    state = model.states[0]
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
            "ln_phi": log(value(state.fug_coeff_phase_comp["Vap", species])),
        }
        for species, cas in (("benzene", "71-43-2"), ("toluene", "108-88-3"))
    ]
    pq.write_table(
        pa.Table.from_pylist(states, schema=state_schema),
        destination / "srk_state.parquet",
        compression="zstd",
    )
    pq.write_table(
        pa.Table.from_pylist(components, schema=component_schema),
        destination / "srk_components.parquet",
        compression="zstd",
    )


if __name__ == "__main__":
    main()

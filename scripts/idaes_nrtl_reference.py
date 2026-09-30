# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Freeze the declared NRTL binary case through independent IDAES equalities."""

import re
from math import log
from pathlib import Path

import idaes
import pyarrow as pa
import pyarrow.parquet as pq
from idaes.models.properties.activity_coeff_models.BTX_activity_coeff_VLE import (
    BTXParameterBlock,
)
from pyomo.environ import ConcreteModel, value
from pyomo.util.calc_var_value import calculate_variable_from_constraint


def main() -> None:
    if idaes.__version__ != "2.13.0":
        raise RuntimeError("the declared oracle requires IDAES 2.13.0")
    root = Path(__file__).resolve().parents[1]
    destination = root / "packages/reference/data/oracles/idaes-2.13"
    source = (destination / "models/nrtl.pse").read_text()
    match = re.search(
        r'@id\("([0-9a-f]{32})"\) entity provenance.release_artifact defaults', source
    )
    if match is None:
        raise RuntimeError("the NRTL oracle source declaration is absent")
    source_id = bytes.fromhex(match.group(1))
    model = ConcreteModel()
    model.parameters = BTXParameterBlock(
        valid_phase=("Liq", "Vap"), activity_coeff_model="NRTL"
    )
    model.states = model.parameters.build_state_block([0], defined_state=True)
    state = model.states[0]
    cas = {"benzene": "71-43-2", "toluene": "108-88-3"}
    rows = []
    for i in model.parameters.component_list:
        for j in model.parameters.component_list:
            # The off-diagonal entries retain IDAES defaults. Pure-component NRTL
            # uses zero self-interaction; the upstream constructor initializes all tau to 1.
            if i == j:
                model.parameters.tau[i, j].set_value(0)
            rows.append(
                {
                    "s": source_id,
                    "i": cas[i],
                    "j": cas[j],
                    "tau": value(model.parameters.tau[i, j]),
                    "alpha": value(model.parameters.alpha[i, j]),
                }
            )
    observations = []
    for x in (0.2, 0.5, 0.8):
        for species, fraction in (("benzene", x), ("toluene", 1 - x)):
            state.mole_frac_phase_comp["Liq", species].set_value(fraction)
        # Solve each declared equality in dependency order. This uses no pse
        # expressions or copied activity-coefficient implementation.
        for key, constraint in state.eq_Gij_coeff.items():
            calculate_variable_from_constraint(
                state.Gij_coeff[key], constraint, eps=1e-13
            )
        for species in model.parameters.component_list:
            for variable, constraint in (
                (state.A[species], state.eq_A[species]),
                (state.B[species], state.eq_B[species]),
                (state.activity_coeff_comp[species], state.eq_activity_coeff[species]),
            ):
                calculate_variable_from_constraint(variable, constraint, eps=1e-13)
            observations.append(
                {
                    "x": x,
                    "j": cas[species],
                    "value": log(value(state.activity_coeff_comp[species])),
                }
            )
    metadata = {
        b"oracle_release": b"idaes-pse==2.13.0",
        b"generator": b"scripts/idaes_nrtl_reference.py",
        b"conditions": b"off-diagonal defaults; diagonal tau=0; x=0.2,0.5,0.8",
    }
    parameter_schema = pa.schema(
        [
            pa.field("s", pa.binary(16)),
            pa.field("i", pa.string()),
            pa.field("j", pa.string()),
            pa.field("tau", pa.float64()),
            pa.field("alpha", pa.float64()),
        ],
        metadata=metadata,
    )
    observation_schema = pa.schema(
        [
            pa.field("x", pa.float64()),
            pa.field("j", pa.string()),
            pa.field("value", pa.float64()),
        ],
        metadata=metadata,
    )
    pq.write_table(
        pa.Table.from_pylist(rows, schema=parameter_schema),
        destination / "data/nrtl_parameters.parquet",
    )
    pq.write_table(
        pa.Table.from_pylist(observations, schema=observation_schema),
        destination / "data/nrtl_activities.parquet",
    )


if __name__ == "__main__":
    main()

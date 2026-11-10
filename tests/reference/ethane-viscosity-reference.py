# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Evaluate the pinned independent Chapman-Enskog pure ethane expression."""

import json
import sys
from math import isfinite
from pathlib import Path
from types import SimpleNamespace

# The isolated interpreter admits only this explicit reference-tool directory.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from idaes.core.base.property_meta import PropertyClassMetadata
from idaes.models.properties.modular_properties.pure.ChapmanEnskog import (
    ChapmanEnskogLennardJones,
    collision_integral_neufeld_callback,
)

from oracles.pyomo_support import units, value
from oracles.reference_environment import ReferenceEnvironment


def main() -> None:
    """Record the released three-point gas selection in Pa s without a solver."""
    environment = ReferenceEnvironment.from_process(
        ("idaes-pse", "pyomo", "pint"), role="PSE_ETHANE_REFERENCE_PYTHON"
    )
    metadata = PropertyClassMetadata()
    parameters = SimpleNamespace(
        get_metadata=lambda: metadata,
        lennard_jones_sigma=4.443 * units.angstrom,
        lennard_jones_epsilon_reduced=215.7 * units.K,
        mw=0.030070 * units.kg / units.mol,
        viscosity_collision_integral_callback=collision_integral_neufeld_callback,
    )
    state = SimpleNamespace(params=parameters)
    observations = []
    for temperature in (320.15, 390.15, 520.15):
        expression = ChapmanEnskogLennardJones.visc_d_phase_comp.return_expression(
            state, parameters, "Vap", temperature * units.K
        )
        viscosity = float(value(units.convert(expression, to_units=units.Pa * units.s)))
        if not isfinite(viscosity) or viscosity <= 0:
            raise ValueError(f"invalid pure-vapor viscosity at {temperature} K")
        observations.append(
            {"temperature_kelvin": temperature, "dynamic_viscosity_pa_s": viscosity}
        )
    print(
        json.dumps({"environment": environment.evidence, "observations": observations})
    )


if __name__ == "__main__":
    main()

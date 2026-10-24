# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent adiabatic liquid PFR reduction; python -m scripts.pfr_reference.

Equal reactant feeds and 1:1 stoichiometry leave one concentration c. Constant
solvent calorics integrate the adiabatic energy balance algebraically:
T = Tin - dHr (cin-c)/(rho cp). The material equation is dc/dV=-k(T)c²/q.
This is the continuum reference, not the 20-cell backward-difference oracle.
No product or IDAES implementation is imported.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
from scipy.integrate import solve_ivp

from scripts.scientific_reference import NUMBER, comparison, frozen_number, publish

if TYPE_CHECKING:
    from numpy.typing import NDArray

INPUTS = {
    "pre_exponential_m3_per_mol_s": 3132000.0,
    "activation_energy_J_per_mol": 43000.0,
    "gas_constant_J_per_mol_K": 8.31446261815324,
    "reaction_heat_J_per_mol": -49000.0,
    "solvent_density_mol_per_m3": 55388.0,
    "solvent_heat_capacity_J_per_mol_K": 75.327,
    "inlet_temperature_K": 303.15,
    "inlet_reactant_mol_per_m3": 100.0,
    "inlet_each_product_mol_per_m3": 1e-8,
    "volume_flow_m3_per_s": 1.0,
    "length_m": 0.5,
    "area_m2": 0.1,
    "pressure_Pa": 101325.0,
    "heat_duty_W": 0.0,
    "pressure_change_Pa": 0.0,
}
FIXTURE = "packages/reference/seed-data/models/unit-fixtures.pse"


def rate_constant(temperature: float) -> float:
    return float(
        INPUTS["pre_exponential_m3_per_mol_s"]
        * np.exp(
            -INPUTS["activation_energy_J_per_mol"]
            / (INPUTS["gas_constant_J_per_mol_K"] * temperature)
        )
    )


def temperature(concentration: float) -> float:
    return INPUTS["inlet_temperature_K"] - INPUTS["reaction_heat_J_per_mol"] * (
        INPUTS["inlet_reactant_mol_per_m3"] - concentration
    ) / (
        INPUTS["solvent_density_mol_per_m3"]
        * INPUTS["solvent_heat_capacity_J_per_mol_K"]
    )


def rhs(_volume: float, state: NDArray[np.float64]) -> list[float]:
    c = float(state[0])
    return [-rate_constant(temperature(c)) * c * c / INPUTS["volume_flow_m3_per_s"]]


def main() -> int:
    frozen = {
        "temperature_K": frozen_number(
            FIXTURE,
            rf'@id\("466634400fec49858a6472af5013d335"\) expect root.outlet_temperature==({NUMBER})',
        ),
        "concentration_mol_per_m3": frozen_number(
            FIXTURE,
            rf'@id\("bf9270f9860c4a559219c99e1e6f949b"\) expect root.outlet_concentration\[chem.ethyl_acetate\]==({NUMBER})',
        ),
    }
    runs: dict[str, object] = {}
    checks: dict[str, bool] = {}
    outputs: list[list[float]] = []
    for method in ("DOP853", "Radau"):
        solution = solve_ivp(
            rhs,
            (0.0, INPUTS["length_m"] * INPUTS["area_m2"]),
            [INPUTS["inlet_reactant_mol_per_m3"]],
            method=method,
            rtol=1e-12,
            atol=1e-12,
        )
        checks[f"{method}.success"] = bool(solution.success)
        c = float(solution.y[0, -1])
        values = {"concentration_mol_per_m3": c, "temperature_K": temperature(c)}
        comparisons = {
            key: comparison(
                value, frozen[key], 1e-7 if key == "concentration_mol_per_m3" else 5e-10
            )
            for key, value in values.items()
        }
        checks.update(
            {
                f"{method}.{key}": bool(value["passed"])
                for key, value in comparisons.items()
            }
        )
        outputs.append(list(values.values()))
        runs[method] = {
            "success": bool(solution.success),
            "message": solution.message,
            "nfev": solution.nfev,
            "njev": solution.njev,
            "outputs": values,
            "comparisons": comparisons,
        }
    checks["independent_methods_agree"] = bool(
        np.max(np.abs(np.array(outputs[0]) - outputs[1])) <= 1e-9
    )
    return publish(
        "pfr_reference",
        {
            "inputs_SI": INPUTS,
            "equations": [
                "k(T)=A exp(-Ea/(R T))",
                "T=Tin-dHr*(cin-c)/(rho*cp)",
                "dc/dV=-k(T)*c*c/q",
            ],
            "method": {
                "algorithms": ["DOP853", "Radau"],
                "rtol": 1e-12,
                "atol": 1e-12,
                "frozen_concentration_numerical_allowance_mol_per_m3": 1e-7,
                "frozen_temperature_numerical_allowance_K": 5e-10,
                "cross_method_absolute_allowance": 1e-9,
            },
            "consumer": f"{FIXTURE}::pfr_radau",
            "runs": runs,
        },
        checks,
        [
            FIXTURE,
            "packages/reference/seed-data/models/saponification.pse",
            "packages/reference/domain/models/constants.pse",
            "packages/reference/process/sources.md",
        ],
    )


if __name__ == "__main__":
    raise SystemExit(main())

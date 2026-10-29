# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent four-state CSTR/PID; python -m scripts.cstr_controller_reference.

Conservation of equal reactants reduces the material inventory to c. Incompressible
constant solvent calorics give V rho cp T' = q rho cp (Tin-T)+Q-V dHr k(T)c².
PID uses normalized measurement T-300, derivative on measurement, a first-order
filter, smooth limits and back-calculation. All four states are integrated together.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
from scipy.integrate import solve_ivp

from scripts.pfr_reference import INPUTS as CHEMISTRY
from scripts.pfr_reference import rate_constant
from scripts.scientific_reference import NUMBER, comparison, frozen_number, publish

if TYPE_CHECKING:
    from numpy.typing import NDArray

INPUTS = {
    **{
        key: value
        for key, value in CHEMISTRY.items()
        if key not in ("length_m", "area_m2", "heat_duty_W")
    },
    "volume_flow_m3_per_s": 0.001,
    "volume_m3": 0.0015,
    "heat_gain_W": 10000.0,
    "measurement_origin_K": 300.0,
    "measurement_scale_K": 1.0,
    "setpoint": 5.0,
    "Kp": 1.0,
    "Ti_s": 5.0,
    "Td_s": 0.1,
    "Tf_s": 0.2,
    "Taw_s": 0.5,
    "bias": 0.0,
    "lower": 0.0,
    "upper": 1.0,
    "eps": 1e-6,
}
# Retain the supplied initial state, including its printed precision. It is an
# input to this independent initial-value problem, not solved by production code.
INITIAL = [20.316088036339764, 304.0858373453369, 0.0, 0.40858373453369]
TIMES = [0.0, 1.0, 5.0, 20.0]
FIXTURE = "packages/reference/campaign/models/cstr-dynamics.pse"


def controller(state: NDArray[np.float64]) -> tuple[float, float, float, float]:
    _c, temperature, integral, filtered = state
    measurement = (temperature - INPUTS["measurement_origin_K"]) / INPUTS[
        "measurement_scale_K"
    ]
    error = INPUTS["setpoint"] - measurement
    filter_rate = (INPUTS["Td_s"] * measurement - filtered) / INPUTS["Tf_s"]
    raw = INPUTS["bias"] + INPUTS["Kp"] * (
        error + integral / INPUTS["Ti_s"] - filter_rate
    )
    # Algebraic smooth max/min, independently expressed from the stated contract.
    lower = 0.5 * (
        raw + INPUTS["lower"] + np.hypot(raw - INPUTS["lower"], INPUTS["eps"])
    )
    output = 0.5 * (
        lower + INPUTS["upper"] - np.hypot(lower - INPUTS["upper"], INPUTS["eps"])
    )
    integral_rate = error + (output - raw) * INPUTS["Ti_s"] / (
        INPUTS["Kp"] * INPUTS["Taw_s"]
    )
    return float(output), float(raw), float(integral_rate), float(filter_rate)


def rhs(_time: float, state: NDArray[np.float64]) -> list[float]:
    c, temperature, _integral, _filtered = state
    output, _raw, integral_rate, filter_rate = controller(state)
    reaction = rate_constant(float(temperature)) * c * c
    q, volume = INPUTS["volume_flow_m3_per_s"], INPUTS["volume_m3"]
    calorics = (
        INPUTS["solvent_density_mol_per_m3"]
        * INPUTS["solvent_heat_capacity_J_per_mol_K"]
    )
    return [
        float(q / volume * (INPUTS["inlet_reactant_mol_per_m3"] - c) - reaction),
        float(
            q / volume * (INPUTS["inlet_temperature_K"] - temperature)
            + (
                INPUTS["heat_gain_W"] * output / volume
                - INPUTS["reaction_heat_J_per_mol"] * reaction
            )
            / calorics
        ),
        integral_rate,
        filter_rate,
    ]


def main() -> int:
    checks: dict[str, bool] = {}
    runs: dict[str, object] = {}
    outputs: list[NDArray[np.float64]] = []
    for method in ("DOP853", "Radau"):
        solution = solve_ivp(
            rhs,
            (TIMES[0], TIMES[-1]),
            INITIAL,
            method=method,
            t_eval=TIMES,
            rtol=1e-11,
            atol=1e-12,
        )
        checks[f"{method}.success"] = solution.success
        comparisons: dict[str, object] = {}
        for index, field in enumerate(("reactant", "temperature")):
            for point, time in enumerate(TIMES):
                literal = frozen_number(
                    FIXTURE, rf"i!={time:g}\{{s\}} or abs\({field}\[i\]-({NUMBER})\{{"
                )
                result = comparison(float(solution.y[index, point]), literal, 1e-8)
                key = f"{field}[{time:g}s]"
                comparisons[key] = result
                checks[f"{method}.{key}"] = bool(result["passed"])
        outputs.append(solution.y)
        runs[method] = {
            "success": solution.success,
            "message": solution.message,
            "nfev": solution.nfev,
            "njev": solution.njev,
            "state_order": [
                "reactant_mol_per_m3",
                "temperature_K",
                "integral_s",
                "filter_s",
            ],
            "states_at_times": solution.y.T.tolist(),
            "controller_output_at_times": [
                controller(state)[0] for state in solution.y.T
            ],
            "comparisons": comparisons,
        }
    checks["independent_methods_agree"] = bool(
        np.max(np.abs(outputs[0] - outputs[1])) <= 1e-8
    )
    return publish(
        "cstr_controller_reference",
        {
            "inputs_SI": INPUTS,
            "initial_state": INITIAL,
            "times_s": TIMES,
            "equations": [
                "c'=(q/V)*(cin-c)-k(T)*c^2",
                "T'=(q/V)*(Tin-T)+(Q/V-dHr*k(T)*c^2)/(rho*cp)",
                "measurement=(T-300)/1; error=5-measurement",
                "F'=(Td*measurement-F)/Tf",
                "raw=bias+Kp*(error+I/Ti-F')",
                "output=smooth_min(smooth_max(raw,lower,eps),upper,eps)",
                "I'=error+(output-raw)*Ti/(Kp*Taw); Q=heat_gain*output",
            ],
            "method": {
                "algorithms": ["DOP853", "Radau"],
                "rtol": 1e-11,
                "atol": 1e-12,
                "frozen_value_numerical_allowance": 1e-8,
                "cross_method_absolute_allowance": 1e-8,
            },
            "consumer": f"{FIXTURE}::ControlledCSTR (integrated and simultaneous samples only)",
            "runs": runs,
        },
        checks,
        [
            FIXTURE,
            "scripts/pfr_reference.py",
            "packages/reference/process/models/control.pse",
            "packages/reference/physical/models/math.pse",
            "packages/reference/seed-data/models/saponification.pse",
            "packages/reference/domain/models/constants.pse",
        ],
    )


if __name__ == "__main__":
    raise SystemExit(main())

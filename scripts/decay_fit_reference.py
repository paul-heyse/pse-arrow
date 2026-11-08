# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent bounded decay regression; python -m scripts.decay_fit_reference.

Minimize sum((a exp(-k t)-y)^2), with analytic residual Jacobian columns
exp(-k t) and -a t exp(-k t). Multiple explicit seeds test local consistency;
agreement does not establish a global optimum or interval certificate.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
from scipy.optimize import least_squares

from scripts.scientific_reference import (
    NUMBER,
    ROOT,
    comparison,
    frozen_number,
    publish,
    require_input,
    selected_record,
)

if TYPE_CHECKING:
    from pathlib import Path

    from numpy.typing import NDArray

TIMES = np.array([0.0, 1.0, 2.0, 3.0, 4.0])
OBSERVATIONS = np.array([2.02, 1.19, 0.75, 0.43, 0.28])
BOUNDS = ([0.1, 0.01], [10.0, 5.0])
SEEDS = [(1.0, 1.0), (0.1, 0.01), (0.1, 5.0), (10.0, 0.01), (10.0, 5.0), (2.0, 0.5)]
FIXTURE = "packages/reference/seed-data/models/regression-fixtures.pse"


def validate_inputs(root: Path = ROOT) -> None:
    """Compare observations, sample times and parameter bounds with the authored fit."""
    authored = (root / FIXTURE).read_text()
    selected_record(
        authored,
        r"test first_order_decay_certified oracle references\.decay_fit_reference fixture ([^\n]+)",
        FIXTURE,
    )
    sse = selected_record(authored, r"let sse:Scalar=([^\n]+);", FIXTURE)
    # The authored expression repeats each residual to square it.
    expected_terms = []
    for time, y in zip(TIMES, OBSERVATIONS, strict=True):
        model = (
            "a" if time == 0 else "a*exp(-k)" if time == 1 else f"a*exp(-{time:g}*k)"
        )
        term = f"({y:g}-{model})"
        expected_terms.append(f"{term}*{term}")
    if sse != "+".join(expected_terms):
        raise ValueError(
            f"stale authored observations or sample coordinates in {FIXTURE}"
        )
    for name, lower, upper in zip(("a", "k"), *BOUNDS, strict=True):
        bounds = selected_record(
            authored, rf"annotation bounds {name}\(([^\n]+)\);", FIXTURE
        )
        require_input(bounds, "", ",", lower, FIXTURE)
        require_input(bounds + ")", ",", ")", upper, FIXTURE)


def residual(parameters: NDArray[np.float64]) -> NDArray[np.float64]:
    a, k = parameters
    return a * np.exp(-k * TIMES) - OBSERVATIONS


def jacobian(parameters: NDArray[np.float64]) -> NDArray[np.float64]:
    a, k = parameters
    exponential = np.exp(-k * TIMES)
    return np.column_stack((exponential, -a * TIMES * exponential))


def main() -> int:
    validate_inputs()
    checks: dict[str, bool] = {}
    runs: list[dict[str, object]] = []
    parameters: list[NDArray[np.float64]] = []
    frozen = {
        field: frozen_number(FIXTURE, rf"expect root\.{field}==({NUMBER})")
        for field in ("a", "k")
    }
    for index, seed in enumerate(SEEDS):
        result = least_squares(
            residual,
            seed,
            jac=jacobian,
            bounds=BOUNDS,
            method="trf",
            ftol=1e-14,
            xtol=1e-14,
            gtol=1e-14,
            max_nfev=10000,
            x_scale=1.0,
            loss="linear",
        )
        checks[f"seed{index}.success"] = result.success
        comparisons = {
            field: comparison(float(value), frozen[field])
            for field, value in zip(("a", "k"), result.x, strict=True)
        }
        checks.update(
            {
                f"seed{index}.{field}": bool(check["passed"])
                for field, check in comparisons.items()
            }
        )
        gradient = 2 * jacobian(result.x).T @ residual(result.x)
        checks[f"seed{index}.interior_stationarity"] = bool(
            np.all(result.x > BOUNDS[0])
            and np.all(result.x < BOUNDS[1])
            and np.max(np.abs(gradient)) <= 1e-8
        )
        parameters.append(result.x)
        runs.append(
            {
                "seed": list(seed),
                "success": result.success,
                "status": result.status,
                "message": result.message,
                "nfev": result.nfev,
                "njev": result.njev,
                "a": float(result.x[0]),
                "k": float(result.x[1]),
                "sse": float(2 * result.cost),
                "residuals": result.fun.tolist(),
                "sse_gradient": gradient.tolist(),
                "optimality": result.optimality,
                "comparisons": comparisons,
            }
        )
    checks["multistart_agreement"] = bool(np.max(np.ptp(parameters, axis=0)) <= 1e-8)
    return publish(
        "decay_fit_reference",
        {
            "inputs": {
                "times": TIMES.tolist(),
                "observations": OBSERVATIONS.tolist(),
                "parameter_order": ["a", "k"],
                "bounds": BOUNDS,
                "seeds": SEEDS,
            },
            "equations": [
                "residual=a*exp(-k*t)-y",
                "J[:,a]=exp(-k*t)",
                "J[:,k]=-a*t*exp(-k*t)",
                "objective=sum(residual^2)",
            ],
            "method": {
                "algorithm": "scipy.optimize.least_squares / trf",
                "jacobian": "analytic",
                "ftol": 1e-14,
                "xtol": 1e-14,
                "gtol": 1e-14,
                "max_nfev": 10000,
                "x_scale": 1.0,
                "loss": "linear",
                "interior_sse_gradient_allowance": 1e-8,
                "multistart_absolute_allowance": 1e-8,
            },
            "consumer": f"{FIXTURE}::first_order_decay_certified (rounded a, k only)",
            "runs": runs,
        },
        checks,
        [FIXTURE],
    )


if __name__ == "__main__":
    raise SystemExit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent Peng-Robinson tangent-plane reference for a two-phase feed (teqp).

teqp's canonical Peng-Robinson uses the exact constants Omega_a = 0.4572355... and
Omega_b = 0.0777960...; the authored model uses the conventional rounded 0.45724 and
0.07780. A closed-form Peng-Robinson written here is first checked against teqp at the
exact constants, then evaluated at the authored ones; both minima are recorded.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import scipy
import teqp
from scipy.optimize import brentq

if TYPE_CHECKING:
    from collections.abc import Callable

    from numpy.typing import NDArray

# The authored benzene/toluene dataset `eos_data.bt_critical` (IDAES 2.13 BT_PR values,
# attributed there to RPP4) with every binary interaction zero (`eos_data.bt_interactions`).
CRITICAL_TEMPERATURE = np.array([562.2, 591.8])
CRITICAL_PRESSURE = np.array([4890000.0, 4100000.0])
ACENTRIC_FACTOR = np.array([0.212, 0.263])
TEMPERATURE, PRESSURE = 368.0, 101325.0
FEED = np.array([0.5, 0.5])
VAPOR, LIQUID = (1.0, 200.0), (5000.0, 9800.0)
GAS_CONSTANT = 8.31446261815324
EXACT = (0.45723552892138218938, 0.077796073903888455972)
AUTHORED = (0.45724, 0.07780)


class PengRobinson:
    """Closed-form Peng-Robinson with zero interactions and the given constants."""

    def __init__(self, omega: tuple[float, float]) -> None:
        kappa = 0.37464 + 1.54226 * ACENTRIC_FACTOR - 0.26992 * ACENTRIC_FACTOR**2
        alpha = (1 + kappa * (1 - np.sqrt(TEMPERATURE / CRITICAL_TEMPERATURE))) ** 2
        rtc = GAS_CONSTANT * CRITICAL_TEMPERATURE
        self.a = omega[0] * rtc**2 / CRITICAL_PRESSURE * alpha
        self.b = omega[1] * rtc / CRITICAL_PRESSURE

    def pressure(self, x: NDArray[np.float64], rho: float) -> float:
        a = float(np.sqrt(np.outer(self.a, self.a)) @ x @ x)
        b = float(self.b @ x)
        rt = GAS_CONSTANT * TEMPERATURE
        return rt * rho / (1 - b * rho) - a * rho**2 / (
            1 + 2 * b * rho - (b * rho) ** 2
        )

    def ln_phi(self, x: NDArray[np.float64], rho: float) -> NDArray[np.float64]:
        rt = GAS_CONSTANT * TEMPERATURE
        root = np.sqrt(np.outer(self.a, self.a))
        a, b = float(root @ x @ x), float(self.b @ x)
        big_a, big_b = a * PRESSURE / rt**2, b * PRESSURE / rt
        z = PRESSURE / (rho * rt)
        s = np.sqrt(2.0)
        return (
            self.b / b * (z - 1)
            - np.log(z - big_b)
            - big_a
            / (2 * s * big_b)
            * (2 * (root @ x) / a - self.b / b)
            * np.log((z + (1 + s) * big_b) / (z + (1 - s) * big_b))
        )


@dataclass(frozen=True)
class Minimum:
    """The stationary liquid-root trial phase against the feed's vapor root."""

    reference_density: float
    trial: list[float]
    trial_density: float
    tpd: float
    stationarity: float


def minimum(
    pressure: Callable[[NDArray[np.float64], float], float],
    ln_phi: Callable[[NDArray[np.float64], float], NDArray[np.float64]],
) -> Minimum:
    """The stationary liquid-root trial phase against the feed's vapor root."""

    def density(x: NDArray[np.float64], branch: tuple[float, float]) -> float:
        return brentq(
            lambda r: pressure(x, r) - PRESSURE, *branch, xtol=1e-14, rtol=1e-15
        )

    reference = ln_phi(FEED, density(FEED, VAPOR))

    def terms(benzene: float) -> tuple[NDArray[np.float64], NDArray[np.float64], float]:
        x = np.array([benzene, 1 - benzene])
        rho = density(x, LIQUID)
        return x, np.log(x) + ln_phi(x, rho) - np.log(FEED) - reference, rho

    # A stationary point of the distance has equal tangent-plane terms for every component;
    # it is solved for directly, bracketed around the smallest distance on a grid.
    grid = np.linspace(0.05, 0.6, 1101)
    best = int(np.argmin([float(np.sum(t[0] * t[1])) for t in map(terms, grid)]))

    def difference(benzene: float) -> float:
        _, d, _ = terms(benzene)
        return float(d[0] - d[1])

    benzene = brentq(difference, grid[best - 1], grid[best + 1], xtol=1e-15, rtol=1e-15)
    trial, d, rho = terms(benzene)
    value = float(np.sum(trial * d))
    stationarity = float(np.max(np.abs(d - value)))
    if value >= 0 or stationarity > 1e-10:
        raise ValueError("no stationary liquid trial phase with a negative distance")
    return Minimum(
        reference_density=density(FEED, VAPOR),
        trial=trial.tolist(),
        trial_density=rho,
        tpd=value,
        stationarity=stationarity,
    )


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    model = teqp.canonical_PR(
        CRITICAL_TEMPERATURE.tolist(),
        CRITICAL_PRESSURE.tolist(),
        ACENTRIC_FACTOR.tolist(),
    )

    def teqp_pressure(x: NDArray[np.float64], rho: float) -> float:
        gas = model.get_R(x)
        return rho * gas * TEMPERATURE * (1 + model.get_Ar01(TEMPERATURE, rho, x))

    def teqp_ln_phi(x: NDArray[np.float64], rho: float) -> NDArray[np.float64]:
        return np.log(np.array(model.get_fugacity_coefficients(TEMPERATURE, rho * x)))

    reference = minimum(teqp_pressure, teqp_ln_phi)
    exact = PengRobinson(EXACT)
    closed = minimum(exact.pressure, exact.ln_phi)
    deviation = max(
        abs(closed.tpd - reference.tpd), abs(closed.trial[0] - reference.trial[0])
    )
    if deviation > 1e-10:
        raise ValueError(f"closed-form Peng-Robinson differs from teqp by {deviation}")
    authored = PengRobinson(AUTHORED)
    destination = root / "tests/fixtures/pr-stability-reference.json"
    destination.write_text(
        json.dumps(
            {
                "schema": "pr-stability-reference-v1",
                "generator": {
                    "library": "teqp",
                    "version": teqp.__version__,
                    "model": "canonical_PR",
                    "scipy": scipy.__version__,
                },
                "components": ["benzene", "toluene"],
                "critical_temperature": CRITICAL_TEMPERATURE.tolist(),
                "critical_pressure": CRITICAL_PRESSURE.tolist(),
                "acentric_factor": ACENTRIC_FACTOR.tolist(),
                "binary_interactions": "zero",
                "gas_constant": GAS_CONSTANT,
                "temperature": TEMPERATURE,
                "pressure": PRESSURE,
                "feed": FEED.tolist(),
                "teqp": asdict(reference),
                "closed_form_deviation_from_teqp": deviation,
                "authored_constants": {"omega_a": AUTHORED[0], "omega_b": AUTHORED[1]},
                "authored": asdict(minimum(authored.pressure, authored.ln_phi)),
                "scope": "the feed's vapor root against liquid-root trial phases: the stationary minimum of the reduced tangent-plane distance; negative, so the feed is unstable. `teqp` is teqp's canonical Peng-Robinson with the exact constants; `authored` is the closed-form Peng-Robinson above, checked against teqp at the exact constants and evaluated at the authored rounded constants",
            },
            indent=2,
            allow_nan=False,
        )
        + "\n"
    )
    print(
        f"Wrote the Peng-Robinson stability reference to {destination.relative_to(root)}"
    )


if __name__ == "__main__":
    main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Freeze independent PC-SAFT and analytic caloric references; no product imports."""

from __future__ import annotations

import hashlib
import json
from decimal import Decimal, localcontext
from pathlib import Path


def caloric(coefficients: list[float], temperature: float) -> tuple[float, float]:
    """Integrate the declared DIPPR100 Cp polynomial and Cp/T, J/(kmol K) -> SI."""
    with localcontext() as context:
        context.prec = 60
        t, t0 = Decimal(str(temperature)), Decimal("298.15")
        c = [Decimal(str(value)) / 1000 for value in coefficients]
        h = sum(a * (t ** (i + 1) - t0 ** (i + 1)) / (i + 1) for i, a in enumerate(c))
        s = c[0] * (t / t0).ln() + sum(
            a * (t**i - t0**i) / i for i, a in enumerate(c) if i
        )
        return float(h), float(s)


def flash_reference(model):
    """Independent isothermal ternary flash; teqp chemical potentials, SciPy iteration."""
    import numpy as np
    import scipy
    from scipy.optimize import least_squares

    temperature, pressure = 280.0, 2e6
    feed = np.array([0.2, 0.3, 0.5])

    def fractions(a):
        v = np.exp(np.array([*a, 0.0]))
        return v / sum(v)

    def residual(a):
        liquid, vapor = np.exp(a[:2])
        x, y, beta = fractions(a[2:4]), fractions(a[4:6]), a[6]
        rt = model.get_R(x) * temperature
        mu = (
            model.get_chempotVLE_autodiff(temperature, liquid * x)
            - model.get_chempotVLE_autodiff(temperature, vapor * y)
        ) / rt
        pl = liquid * rt * (1 + model.get_Ar01(temperature, liquid, x))
        pv = vapor * rt * (1 + model.get_Ar01(temperature, vapor, y))
        return np.r_[
            (pl - pressure) / pressure,
            (pv - pressure) / pressure,
            mu,
            ((1 - beta) * x + beta * y - feed)[:2],
        ]

    start = np.r_[
        np.log([12500.0, 1000.0]),
        np.log(np.array([0.05, 0.2]) / 0.75),
        np.log(np.array([0.6, 0.3]) / 0.1),
        0.4,
    ]
    result = least_squares(
        residual,
        start,
        bounds=(
            np.r_[np.log([6000.0, 1.0]), [-10.0] * 4, 0.0],
            np.r_[np.log([25000.0, 5999.0]), [10.0] * 4, 1.0],
        ),
        max_nfev=1000,
        gtol=1e-13,
        ftol=1e-13,
        xtol=1e-13,
    )
    error = max(abs(result.fun))
    if not result.success or not np.isfinite(error) or error > 1e-10:
        raise ValueError("independent flash did not satisfy every physical equation")
    return dict(
        temperature=temperature,
        pressure=pressure,
        feed=feed.tolist(),
        liquid_density=float(np.exp(result.x[0])),
        vapor_density=float(np.exp(result.x[1])),
        liquid=fractions(result.x[2:4]).tolist(),
        vapor=fractions(result.x[4:6]).tolist(),
        beta=float(result.x[6]),
        maximum_scaled_residual=float(error),
        scipy=scipy.__version__,
        scope="two homogeneous phases satisfying chemical potential, pressure and material balances; no global stability certificate",
    )


def main() -> None:
    import numpy as np
    import teqp

    root = Path(__file__).resolve().parents[1]
    data = root / "crates/pse-kernels/data"
    pc_path = data / "pcsaft-light-hydrocarbons.json"
    ig_path = data / "ideal-gas-light-hydrocarbons.json"
    pc, ig = json.loads(pc_path.read_text()), json.loads(ig_path.read_text())
    coeffs = []
    for row in pc:
        coefficient = teqp.SAFTCoeffs()
        coefficient.m = row["m"]
        coefficient.sigma_Angstrom = row["sigma"]
        coefficient.epsilon_over_k = row["epsilon_k"]
        coeffs.append(coefficient)
    model = teqp.PCSAFTEOS(coeffs, np.zeros((3, 3)))
    cases = []
    # Finite homogeneous states, not implicit claims of global phase stability.
    for temperature, density, composition in [
        (300.0, 10.0, [0.2, 0.3, 0.5]),
        (350.0, 34.565566349336066, [0.2, 0.3, 0.5]),
        (400.0, 1000.0, [0.6, 0.25, 0.15]),
        (300.0, 14000.0, [0.1, 0.2, 0.7]),
        (298.15, 10.0, [0.2, 0.3, 0.5]),
    ]:
        x = np.array(composition)
        gas = model.get_R(x)
        ar01 = model.get_Ar01(temperature, density, x)
        ar10 = model.get_Ar10(temperature, density, x)
        h0 = sum(
            z * caloric(row["DIPPR100"], temperature)[0]
            for z, row in zip(composition, ig, strict=True)
        )
        cases.append(
            {
                "input": [temperature, density, *composition[:2]],
                "pressure": density * gas * temperature * (1 + ar01),
                "enthalpy": h0 + gas * temperature * (ar10 + ar01),
                "ln_phi": np.log(
                    model.get_fugacity_coefficients(temperature, density * x)
                ).tolist(),
                "ideal_enthalpy": h0,
                "gas_constant": gas,
            }
        )
    with localcontext() as context:
        context.prec = 100
        difficult = [
            {"x": x, "log1px": str((1 + Decimal.from_float(x)).ln())}
            for x in [-0.9999, 1e-8, 0.0001, 1.0, 10000.0]
        ]
    (root / "tests/fixtures/plan14/real-algebra-reference.json").write_text(
        json.dumps(
            {
                "oracle": "Python Decimal with exact binary64 inputs",
                "precision": 100,
                "cases": difficult,
            },
            indent=2,
        )
        + "\n"
    )
    result = {
        "schema": "plan14-thermodynamic-reference-v1",
        "generator": {
            "library": "teqp",
            "version": teqp.__version__,
            "python": "3.12",
            "caloric_precision": 60,
        },
        "source": "https://teqp.readthedocs.io/en/latest/models/PCSAFT.html",
        "parameters": {
            str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (pc_path, ig_path)
        },
        "conventions": {
            "components": ["methane", "ethane", "propane"],
            "binary_interactions": "zero",
            "temperature": "K",
            "density": "mol/m^3",
            "pressure": "Pa",
            "enthalpy": "J/mol",
            "caloric_reference": "DIPPR100 integral from 298.15 K; no formation enthalpy",
            "phase": "homogeneous explicit density; no global stability claim",
        },
        "tolerances": {
            "relative": 2e-8,
            "pressure_absolute": 1e-3,
            "enthalpy_absolute": 1e-5,
            "ln_phi_absolute": 1e-7,
        },
        "cases": cases,
        "flash": flash_reference(model),
    }
    destination = root / "tests/fixtures/plan14/thermo-reference.json"
    destination.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n")
    print(
        f"Wrote {len(cases)} independent reference states to {destination.relative_to(root)}"
    )


if __name__ == "__main__":
    main()

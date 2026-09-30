# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Freeze independent PC-SAFT and analytic caloric references; no product imports."""

from __future__ import annotations

import hashlib
import json
from decimal import Decimal, localcontext
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, TypedDict

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq
import scipy
import teqp
from scipy.optimize import least_squares

if TYPE_CHECKING:
    from numpy.typing import NDArray


class FlashModel(Protocol):
    """The independent reference calls, without a product provider dependency."""

    def get_R(self, composition: NDArray[np.float64]) -> float: ...  # noqa: N802 -- upstream spelling

    def get_chempotVLE_autodiff(  # noqa: N802 -- upstream spelling
        self, temperature: float, densities: NDArray[np.float64]
    ) -> NDArray[np.float64]: ...

    def get_Ar01(  # noqa: N802 -- upstream spelling
        self, temperature: float, density: float, composition: NDArray[np.float64]
    ) -> float: ...


def caloric(coefficients: list[float], temperature: float) -> tuple[float, float]:
    """Integrate the declared DIPPR100 Cp polynomial and Cp/T, SI coefficient magnitudes."""
    with localcontext() as context:
        context.prec = 60
        t, t0 = Decimal(str(temperature)), Decimal("298.15")
        c = [Decimal(str(value)) for value in coefficients]
        h = sum(a * (t ** (i + 1) - t0 ** (i + 1)) / (i + 1) for i, a in enumerate(c))
        s = c[0] * (t / t0).ln() + sum(
            a * (t**i - t0**i) / i for i, a in enumerate(c) if i
        )
        return float(h), float(s)


def flash_reference(model: FlashModel) -> dict:
    """Independent isothermal ternary flash; teqp chemical potentials, SciPy iteration."""
    temperature, pressure = 280.0, 2e6
    feed = np.array([0.2, 0.3, 0.5])

    def fractions(a: NDArray[np.float64]) -> NDArray[np.float64]:
        v = np.exp(np.array([*a, 0.0]))
        return v / sum(v)

    def residual(a: NDArray[np.float64]) -> NDArray[np.float64]:
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
    return {
        "temperature": temperature,
        "pressure": pressure,
        "feed": feed.tolist(),
        "liquid_density": float(np.exp(result.x[0])),
        "vapor_density": float(np.exp(result.x[1])),
        "liquid": fractions(result.x[2:4]).tolist(),
        "vapor": fractions(result.x[4:6]).tolist(),
        "beta": float(result.x[6]),
        "maximum_scaled_residual": float(error),
        "scipy": scipy.__version__,
        "scope": "two homogeneous phases satisfying chemical potential, pressure and material balances; no global stability certificate",
    }


class ReferenceCase(TypedDict):
    """One independent homogeneous reference state."""

    input: list[float]
    pressure: float
    enthalpy: float
    ln_phi: list[float]
    ideal_enthalpy: float
    gas_constant: float


def main() -> None:

    root = Path(__file__).resolve().parents[1]
    pc_path = (
        root / "packages/reference/data/gross-sadowski-2001/data/parameters.parquet"
    )
    ig_path = root / "packages/reference/data/poling2000/data/vessel_caloric.parquet"
    subjects = ["74-82-8", "74-84-0", "74-98-6"]
    pc_rows = {row["subject"]: row for row in pq.read_table(pc_path).to_pylist()}
    ig_rows = {row["subject"]: row for row in pq.read_table(ig_path).to_pylist()}
    pc = [pc_rows[subject] for subject in subjects]
    ig = [ig_rows[subject] for subject in subjects]
    coeffs = []
    for row in pc:
        coefficient = teqp.SAFTCoeffs()
        coefficient.m = row["m"]
        coefficient.sigma_Angstrom = row["sigma"] * 1e10
        coefficient.epsilon_over_k = row["epsilon"]
        coeffs.append(coefficient)
    model = teqp.PCSAFTEOS(coeffs, np.zeros((3, 3)))
    cases: list[ReferenceCase] = []
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
            z * caloric([row[f"c{i}"] for i in range(1, 6)], temperature)[0]
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
            {"x": x, "log1px": str((1 + Decimal(x)).ln())}
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
    destination = root / "packages/reference/data/oracles/teqp-0.23.1/data"
    write_reference(
        destination / "states.parquet",
        [
            {
                "case": i,
                "T": case["input"][0],
                "rho": case["input"][1],
                "pressure": case["pressure"],
                "enthalpy": case["enthalpy"],
                "ideal_enthalpy": case["ideal_enthalpy"],
                "h_residual": case["enthalpy"] - case["ideal_enthalpy"],
            }
            for i, case in enumerate(cases)
        ],
        {"case": pa.int64()},
        result["generator"],
    )
    write_reference(
        destination / "components.parquet",
        [
            {"case": i, "subject": subjects[j], "ln_phi": value}
            for i, case in enumerate(cases)
            for j, value in enumerate(case["ln_phi"])
        ],
        {"case": pa.int64(), "subject": pa.string()},
        result["generator"],
    )
    flash = result["flash"]
    write_reference(
        destination / "flash.parquet",
        [
            {
                "case": 0,
                "T": flash["temperature"],
                "pressure": flash["pressure"],
                "liquid_density": flash["liquid_density"],
                "vapor_density": flash["vapor_density"],
                "beta": flash["beta"],
                "maximum_scaled_residual": flash["maximum_scaled_residual"],
            }
        ],
        {"case": pa.int64()},
        result["generator"],
    )
    write_reference(
        destination / "flash_components.parquet",
        [
            {
                "subject": subject,
                "feed": flash["feed"][j],
                "liquid": flash["liquid"][j],
                "vapor": flash["vapor"][j],
            }
            for j, subject in enumerate(subjects)
        ],
        {"subject": pa.string()},
        result["generator"],
    )
    print(
        f"Wrote {len(cases)} independent states and one flash to {destination.relative_to(root)}"
    )


def write_reference(
    path: Path, rows: list[dict], types: dict[str, pa.DataType], metadata: dict
) -> None:
    """Explicit schemas: identifiers and integer keys, otherwise SI Float64 observations."""
    schema = pa.schema(
        [
            pa.field(name, types.get(name, pa.float64()), nullable=False)
            for name in rows[0]
        ],
        metadata={b"oracle": json.dumps(metadata).encode()},
    )
    pq.write_table(
        pa.Table.from_pylist(rows, schema=schema),
        path,
        compression="zstd",
        version="2.6",
    )


if __name__ == "__main__":
    main()

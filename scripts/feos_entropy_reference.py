# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent homogeneous entropy from teqp residuals and Decimal Cp/T integrals."""

import hashlib
import json
import sys
from decimal import Decimal, localcontext
from pathlib import Path

import numpy as np
import teqp

from scripts.plan14_reference import caloric


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    data = root / "crates/pse-kernels/data"
    paths = [
        data / "pcsaft-light-hydrocarbons.json",
        data / "ideal-gas-light-hydrocarbons.json",
    ]
    pc, ig = (json.loads(p.read_text()) for p in paths)
    cases = []
    for indices, temperature, density, fractions in [
        ([0], 298.15, 1e-4, [1.0]),
        ([1, 0], 350.0, 30.0, [0.3, 0.7]),
        ([0, 1, 2], 300.0, 10.0, [0.2, 0.3, 0.5]),
        ([0, 1, 2], 350.0, 34.565566349336066, [0.2, 0.3, 0.5]),
        ([0, 1, 2], 400.0, 1000.0, [0.6, 0.25, 0.15]),
    ]:
        coefficients = []
        for index in indices:
            row = pc[index]
            coefficient = teqp.SAFTCoeffs()
            coefficient.m = row["m"]
            coefficient.sigma_Angstrom = row["sigma"]
            coefficient.epsilon_over_k = row["epsilon_k"]
            coefficients.append(coefficient)
        model = teqp.PCSAFTEOS(coefficients, np.zeros((len(indices), len(indices))))
        x = np.array(fractions)
        gas = model.get_R(x)
        ar00 = model.get_Ar00(temperature, density, x)
        ar10 = model.get_Ar10(temperature, density, x)
        with localcontext() as context:
            context.prec = 60
            r = Decimal(str(gas))
            mixing = -r * sum(Decimal(str(z)) * Decimal(str(z)).ln() for z in fractions)
            pressure = (
                -r
                * (
                    Decimal(str(density))
                    * r
                    * Decimal(str(temperature))
                    / Decimal(100000)
                ).ln()
            )
            thermal = sum(
                z * caloric(ig[i]["DIPPR100"], temperature)[1]
                for z, i in zip(fractions, indices, strict=True)
            )
            ideal = thermal + float(mixing + pressure)
        cases.append(
            {
                "cas": [pc[i]["identifier"]["cas"] for i in indices],
                "input": [temperature, density, *fractions[:-1]],
                "entropy": ideal + gas * (ar10 - ar00),
                "ideal_entropy": ideal,
                "residual_entropy": gas * (ar10 - ar00),
            }
        )
    with localcontext() as context:
        context.prec = 60
        kb = Decimal("1.380649e-23")
        na = Decimal("6.02214076e23")
        pressure_unit = kb / Decimal("1e-30")
        offset = kb * na * (pressure_unit / Decimal(100000)).ln()
    output = {
        "generator": {
            "library": "teqp",
            "version": teqp.__version__,
            "python": sys.version.split()[0],
            "caloric_precision": 60,
        },
        "reference": "Pure ideal gas S=0 at 298.15 K and 100000 Pa; mixture includes ideal mixing; no third-law claim",
        "residual_formula": "R*(Ar10-Ar00) at identical T, molar density and composition",
        "feos_reference_pressure_pa": str(pressure_unit),
        "feos_raw_minus_reference_j_per_mol_k": str(offset),
        "parameters": {
            str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in paths
        },
        "cases": cases,
    }
    path = root / "tests/fixtures/thermo-entropy-reference.json"
    path.write_text(json.dumps(output, indent=2, allow_nan=False) + "\n")
    print(
        f"Wrote {len(cases)} independent homogeneous entropy states; derived raw offset {offset}"
    )


if __name__ == "__main__":
    main()

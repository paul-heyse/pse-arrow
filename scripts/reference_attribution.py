# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Check independent phase and mixer derivations against their authored claims."""

from __future__ import annotations

import math
from decimal import Decimal
from typing import TYPE_CHECKING

import pyarrow.parquet as pq

from scripts.pfr_reference import FIXTURE as UNIT_FIXTURES
from scripts.scientific_reference import ROOT, require_input, selected_record

if TYPE_CHECKING:
    from pathlib import Path

BT_FIXTURES = "packages/reference/seed-data/models/bt-ideal.pse"
PRESSURE_BANK = (
    "packages/reference/data/oracles/idaes-2.13/data/rpp4_pressure_oracles.parquet"
)


def _case(source: str, name: str, attribution: str) -> tuple[str, str]:
    record = selected_record(
        source, rf"\btest {name} oracle ([^\n]+\n[\s\S]+?)\n\}}", name
    )
    header, body = record.split("\n", maxsplit=1)
    expected = f"references.{attribution} fixture "
    if not header.startswith(expected):
        raise ValueError(f"{name}: reference attribution differs from {attribution}")
    return header.removeprefix(expected), body


def check_btideal(source: str, bank: Path) -> None:
    """Determine the Raoult-law endpoint from the bound experiment coordinates."""
    species = ("benzene", "toluene")
    bank_rows = pq.read_table(
        bank,
        filters=[
            ("subject", "in", ["71-43-2", "108-88-3"]),
            ("phase_type", "=", "liquidPhase"),
        ],
    ).to_pylist()
    if len(bank_rows) != 2 or {row["subject"] for row in bank_rows} != {
        "71-43-2",
        "108-88-3",
    }:
        raise ValueError("BTIdeal saturation bank differs")
    pressure = 101325.0
    for label, temperature, expected_beta in (
        ("liquid", 358.0, 0),
        ("vapor", 378.0, 1),
    ):
        ratios = []
        for row in bank_rows:
            if (
                not row["minimum_temperature"]
                <= temperature
                <= row["maximum_temperature"]
            ):
                raise ValueError(
                    f"BTIdeal saturation input outside its range at {temperature:g} K"
                )
            reduced = temperature / row["tc"]
            theta = 1 - reduced
            log_ratio = (
                sum(
                    row[name] * theta**power
                    for name, power in (("a", 1), ("b", 1.5), ("c", 3), ("d", 6))
                )
                / reduced
            )
            ratios.append(row["pc"] * math.exp(log_ratio) / pressure)
        # The Rachford-Rice residual decreases over 0 <= beta <= 1.
        # A negative liquid endpoint or a positive vapor endpoint fixes the phase.
        liquid_residual = sum((k - 1) / 2 for k in ratios)
        vapor_residual = sum((1 - 1 / k) / 2 for k in ratios)
        actual_beta = 0 if liquid_residual < 0 else 1 if vapor_residual > 0 else None
        if actual_beta != expected_beta:
            raise ValueError(f"BTIdeal {temperature:g} K analytic phase bound differs")
        for name in (
            f"nested_{label}",
            *(
                f"complementarity_{label}_{mode}"
                for mode in ("smooth", "disjunctive", "penalty")
            ),
        ):
            header, body = _case(
                source, name, f"analytic_btideal_{label}_{temperature:g}"
            )
            for coordinate, unit, amount in (
                ("T", "K", temperature),
                ("pressure", "Pa", pressure),
                ("flow", "mol/s", 1.0),
                *((f"z[chem.{member}]", "1", 0.5) for member in species),
            ):
                require_input(
                    header, f"fix root.{coordinate}=", f"{{{unit}}};", amount, name
                )
            require_input(body, "expect root.beta==", " tolerance", expected_beta, name)
            for member in species if name.startswith("nested_") else species[:1]:
                closure = (
                    f"expect (1-root.beta)*root.x[chem.{member}]+root.beta*root.y[chem.{member}]"
                    f"==root.z[chem.{member}] tolerance numerical_policy.mole_fraction;"
                )
                if closure not in body:
                    raise ValueError(f"{name}: {member} closure expectation differs")

    # The historical observation remains attached to its actual experiment.
    for name in (
        "flash_368",
        "ftpx_oracle",
        "initialized_ftpx",
        "nested_two_phase",
        *(
            f"complementarity_two_phase_{mode}"
            for mode in ("smooth", "disjunctive", "penalty")
        ),
    ):
        header, _ = _case(source, name, "idaes_test_btideal")
        require_input(header, "fix root.flow=", "{mol/s};", 1.0, name)
        require_input(header, "fix root.T=", "{K};", 368.0, name)
        require_input(header, "fix root.pressure=", "{Pa};", pressure, name)
        for member in species:
            require_input(header, f"fix root.z[chem.{member}]=", "{1};", 0.5, name)


def check_mixer(source: str) -> None:
    """Bind the outlet energy and pressure rules to the unequal-pressure experiment."""
    name = "mixer_unequal_pressure_minimum"
    header, body = _case(source, name, "analytic_mixer_energy_minimum_pressure")
    if (
        "inlet_pkg=saponification.Saponification(defined_state=true,selected={chem.water}),outlet_pkg=saponification.Saponification(defined_state=false,selected={chem.water})"
        not in body
    ):
        raise ValueError(f"{name}: common water-only caloric model differs")
    inlets = (
        ("first", Decimal("0.001"), Decimal(320), Decimal(101325), Decimal(55388)),
        ("second", Decimal("0.001"), Decimal(300), Decimal(202650), Decimal(55388)),
    )
    for port, volume, temperature, pressure, water in inlets:
        for coordinate, unit, amount in (
            ("volume_flow", "m^3/s", volume),
            ("T", "K", temperature),
            ("pressure", "Pa", pressure),
            ("concentration[chem.water]", "mol/m^3", water),
        ):
            require_input(
                header,
                f"fix root.inlet[{port}].{coordinate}=",
                f"{{{unit}}};",
                float(amount),
                name,
            )
    molar_flow = sum(volume * water for _, volume, _, _, water in inlets)
    # The common positive heat capacity and caloric datum cancel in the energy balance.
    temperature = (
        sum(volume * water * temp for _, volume, temp, _, water in inlets) / molar_flow
    )
    for coordinate, unit, amount in (
        ("T", "K", temperature),
        ("pressure", "Pa", min(pressure for _, _, _, pressure, _ in inlets)),
        ("flow", "mol/s", molar_flow),
        ("component_flow[chem.water]", "mol/s", molar_flow),
    ):
        require_input(
            body,
            f"expect root.outlet.{coordinate}==",
            f"{{{unit}}} tolerance",
            float(amount),
            name,
        )


def check(root: Path = ROOT) -> None:
    """Check the selected attribution and original published inputs without writes."""
    check_btideal((root / BT_FIXTURES).read_text(), root / PRESSURE_BANK)
    check_mixer((root / UNIT_FIXTURES).read_text())


if __name__ == "__main__":
    check()

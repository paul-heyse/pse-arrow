# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Smoke test of the thermotools side environment: one real call per library, JSON report.

Run it with `just tk-env-run thermotools python envs/thermotools/smoke.py`. Standalone: it
imports nothing from thermo_knowledge.
"""

import json
import platform
from importlib.metadata import version

import numpy as np
import pyarrow as pa


def main() -> None:
    report: dict[str, object] = {
        "environment": "thermotools",
        "python": platform.python_version(),
        "versions": {
            name: version(name)
            for name in ("numpy", "pyarrow", "scipy", "thermopack", "pykingas", "surfpack")
        },
    }
    calls: dict[str, object] = {}

    # thermopack: SRK cubic for methane, bubble pressure of a methane/ethane mixture.
    from thermopack.cubic import SoaveRedlichKwong

    srk = SoaveRedlichKwong("C1,C2")
    pressure, _ = srk.bubble_pressure(150.0, np.array([0.5, 0.5]))
    calls["thermopack_srk_bubble_pressure_pa"] = float(pressure)
    assert 1.0e4 < pressure < 1.0e7

    # pykingas: MieKinGas for pure methane (database parameters), particle mass and pure-gas viscosity.
    from pykingas.MieKinGas import MieKinGas

    kinetic_gas = MieKinGas("C1")
    calls["pykingas_particle_mass_kg"] = float(np.atleast_1d(kinetic_gas.mole_weights)[0])
    # A single-component model is treated as a binary, so the composition has two entries.
    molar_volume = 8.314462618 * 300.0 / 1.0e5  # ideal gas at 300 K and 1 bar, m3/mol
    viscosity = kinetic_gas.viscosity(300.0, molar_volume, [0.5, 0.5])
    calls["pykingas_viscosity_pa_s_300k_1bar"] = float(viscosity)
    assert 5.0e-6 < viscosity < 2.0e-5

    # surfpack: import the PC-SAFT functional class.
    from surfpack.pcsaft import PC_SAFT

    calls["surfpack_functional_class"] = f"{PC_SAFT.__module__}.{PC_SAFT.__name__}"

    # pyarrow: a real round trip, the exchange format with the core pipeline.
    table = pa.table({"bubble_pressure_pa": [calls["thermopack_srk_bubble_pressure_pa"]]})
    calls["pyarrow_rows"] = pa.ipc.open_stream(_ipc_bytes(table)).read_all().num_rows

    report["calls"] = calls
    print(json.dumps(report, indent=2, sort_keys=True))


def _ipc_bytes(table: pa.Table) -> bytes:
    sink = pa.BufferOutputStream()
    with pa.ipc.new_stream(sink, table.schema) as writer:
        writer.write_table(table)
    return sink.getvalue().to_pybytes()


if __name__ == "__main__":
    main()

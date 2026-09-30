# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Smoke test of the geochem side environment: one real call per library, JSON report.

Run it with `just tk-env-run geochem python envs/geochem/smoke.py`. Standalone: it imports
nothing from thermo_knowledge. The ThermoFun database and the PHREEQC database file are read
from the raw store (read-only).
"""

import json
import os
import platform
import sys
import tempfile
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

TREE = Path(__file__).resolve().parents[2]
RAW = TREE / ".store" / "raw"


def raw_file(pattern: str) -> Path:
    matches = sorted(RAW.glob(pattern))
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one raw-store file for {pattern}, found {len(matches)}")
    return matches[0]


def main() -> None:
    import chemicalfun
    import pyarrow as pa
    import reaktoro
    import thermofun

    # Every library must come from this environment, not from a user site directory.
    for module in (reaktoro, thermofun, chemicalfun, pa):
        assert str(Path(module.__file__).resolve()).startswith(sys.prefix), module.__file__

    versions = {name: version(name) for name in ("reaktoro", "thermofun", "chemicalfun")}
    versions["pyarrow"] = pa.__version__
    calls: dict[str, object] = {}

    # Reaktoro: an embedded PHREEQC database.
    database = reaktoro.PhreeqcDatabase("phreeqc.dat")
    calls["reaktoro_phreeqc_dat_species"] = len(database.species())
    calls["reaktoro_phreeqc_dat_elements"] = len(database.elements())
    assert calls["reaktoro_phreeqc_dat_species"] > 100

    # ThermoFun: a database JSON from the raw store.
    thermofun_json = raw_file("thermofun/*/tree/Resources/databases/mines16-thermofun.json")
    tf_database = thermofun.Database(str(thermofun_json))
    calls["thermofun_mines16_substances"] = tf_database.numberOfSubstances()
    calls["thermofun_mines16_reactions"] = tf_database.numberOfReactions()
    assert calls["thermofun_mines16_substances"] > 100

    # ChemicalFun: the elements occurring in a list of formulas.
    elements = chemicalfun.elementsInFormulas(["H2O", "CaCO3"])
    calls["chemicalfun_elements"] = len(elements)
    assert len(elements) == 4

    # GEMS3K: the library is a C++ shared object without a Python binding; check it is found.
    gems3k = _prefix_library()
    calls["gems3k_shared_object"] = gems3k
    assert gems3k

    # phreeqcrm (optional, installed with pip): load a database and find the components.
    try:
        versions["phreeqcrm"] = version("phreeqcrm")
    except PackageNotFoundError:
        calls["phreeqcrm"] = "not installed"
    else:
        import phreeqcrm

        model = phreeqcrm.PhreeqcRM(1, 1)
        phreeqc_dat = raw_file("phreeqc/*/tree/database/phreeqc.dat")
        assert model.LoadDatabase(str(phreeqc_dat)) == phreeqcrm.IRM_OK
        assert model.RunString(True, False, False, "SOLUTION 1\n pH 7\n") == phreeqcrm.IRM_OK
        calls["phreeqcrm_components"] = model.FindComponents()

    # pyarrow: a real table construction, the exchange format with the core pipeline.
    calls["pyarrow_rows"] = pa.table({"n": [calls["reaktoro_phreeqc_dat_species"]]}).num_rows

    print(
        json.dumps(
            {
                "environment": "geochem",
                "python": platform.python_version(),
                "versions": versions,
                "calls": calls,
            },
            indent=2,
            sort_keys=True,
        )
    )


def _prefix_library() -> str:
    candidates = sorted(Path(sys.prefix, "lib").glob("libGEMS3K*.so*"))
    return candidates[0].name if candidates else ""


if __name__ == "__main__":
    # The libraries write log files into the working directory; keep it out of the tree.
    with tempfile.TemporaryDirectory() as scratch:
        os.chdir(scratch)
        main()

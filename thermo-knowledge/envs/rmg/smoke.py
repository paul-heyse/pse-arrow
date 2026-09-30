# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Smoke test of the rmg side environment: RMG-Py's own loader on the raw-store database.

Run it with `just tk-env-run rmg python envs/rmg/smoke.py`. Standalone: it imports
nothing from thermo_knowledge. Only three thermo libraries are loaded, through RMG's own
`ThermoDatabase.load_libraries`; kinetics are not loaded. The database is read from the raw
store (read-only).
"""

import json
import logging
import platform
import sys
from pathlib import Path

TREE = Path(__file__).resolve().parents[2]
LIBRARIES = ("primaryThermoLibrary", "C3", "GRI-Mech3.0")


def conda_version(package: str) -> str:
    """Version of a conda package from the prefix's conda-meta (RMG ships no Python metadata)."""
    records = sorted(Path(sys.prefix, "conda-meta").glob(f"{package}-[0-9]*.json"))
    assert len(records) == 1, (package, records)
    return json.loads(records[0].read_text())["version"]


def main() -> None:
    import pyarrow as pa
    import rmgpy
    from rmgpy.data.thermo import ThermoDatabase

    # Every library must come from this environment, not from a user site directory.
    for module in (rmgpy, pa):
        assert str(Path(module.__file__).resolve()).startswith(sys.prefix), module.__file__

    matches = sorted(
        (TREE / ".store" / "raw" / "rmg_database").glob("*/tree/input/thermo/libraries")
    )
    if len(matches) != 1:
        raise SystemExit(f"expected exactly one raw-store RMG database, found {len(matches)}")

    logging.disable(logging.INFO)  # RMG logs every library it loads
    database = ThermoDatabase()
    database.load_libraries(str(matches[0]), libraries=list(LIBRARIES))
    entries = {label: len(library.entries) for label, library in database.libraries.items()}
    assert sorted(entries) == sorted(LIBRARIES) and all(entries.values())

    first = database.libraries[LIBRARIES[0]]
    sample = next(iter(first.entries.values()))
    table = pa.table({"library": list(entries), "entries": list(entries.values())})

    print(
        json.dumps(
            {
                "environment": "rmg",
                "python": platform.python_version(),
                "versions": {
                    "rmgpy": rmgpy.__version__,
                    "rmg": conda_version("rmg"),
                    "rmgdatabase": conda_version("rmgdatabase"),
                    "pyarrow": pa.__version__,
                },
                "calls": {
                    "thermo_libraries_loaded": len(database.libraries),
                    "thermo_library_entries": entries,
                    "sample_entry": f"{LIBRARIES[0]}:{sample.label}",
                    "pyarrow_rows": table.num_rows,
                },
            },
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()

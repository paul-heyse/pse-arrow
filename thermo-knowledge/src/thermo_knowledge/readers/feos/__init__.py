# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the FeOS source (`sources/feos.toml`, release 0.10.1).

Source-faithful: FeOS's own field names, values as written, one row per JSON record with its
JSON pointer as locator (`<file>#/<record>`; association sites and bonds add their own
segments). Nothing is converted, resolved or merged: the same compound appears in several
files and keeps one row in each. A JSON object is consumed key by key and a key no column
declares is an error, so nothing is silently dropped; an absent key is a null column.

| Payload | Tables |
|---|---|
| pure-component, segment and binary files of `pcsaft`, `epcsaft`, `saftvrmie`, `saftvrqmie` | `pure_records`, `segment_records`, `binary_records`, `binary_segment_records`, `association_sites`, `permittivity_records`, `permittivity_data_points` |
| `ideal_gas/poling2000.json`, `burkhardt2025.json` | `dippr_records` |
| `ideal_gas/joback1987.json` | `joback_groups` |
| `pcsaft/gc_substances.json` | `chemical_records`, `chemical_record_bonds` |
| `pcsaft/sauer2014_smarts.json` | `smarts_records` |
| `multiparameter/coolprop.json` | `multiparameter_fluids`, `multiparameter_terms`, `multiparameter_term_rows` |

The manifest's payload is the `parameters/**/*.json` files only; the README and BibTeX files of
the parameter directories, the legacy `parameters_old` tree and the Rust source are not in it.
Units are those the FeOS Rust parameter structs state in their comments; the JSON files state
none (see `records.py`).
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.feos import multiparameter, others, records
from thermo_knowledge.readers.feos.common import Sink, merge_schemas
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES: dict[str, pa.Schema] = merge_schemas(
    records.SCHEMAS,
    others.SCHEMAS,
    multiparameter.SCHEMAS,
)


def read(tree: Path, writer: Writer) -> None:
    """Decompose every payload file (all of them hold JSON arrays of records)."""
    sink = Sink(writer)
    for artifact in writer.payload_files:
        if artifact in others.DIPPR_FILES:
            others.read_dippr(tree, artifact, sink)
        elif artifact == others.JOBACK_FILE:
            others.read_joback(tree, artifact, sink)
        elif artifact == others.CHEMICAL_FILE:
            others.read_chemicals(tree, artifact, sink)
        elif artifact == others.SMARTS_FILE:
            others.read_smarts(tree, artifact, sink)
        elif artifact == multiparameter.MULTIPARAMETER_FILE:
            multiparameter.read_multiparameter(tree, artifact, sink)
        else:
            records.read_records(tree, artifact, sink)
        writer.opened(artifact)
    sink.flush()

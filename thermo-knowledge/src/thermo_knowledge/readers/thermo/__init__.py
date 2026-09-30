# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the `thermo` source (`sources/thermo.toml`, 0.6.1).

Source-faithful: each data file keeps its own keys, numbers and spellings; nothing is mapped,
converted, resolved or deduplicated. The payload is 52 files. Every one is decomposed into rows,
except `thermo/unifac.py`, which is Python source read only for its data constructs (status
`partly_read`).

| Payload | Module | Tables |
|---|---|---|
| `Interaction Parameters/**.json`, `Scalar Parameters/*.json` | `interaction` | `parameter_files`, `henry_pairs`, `nrtl_pairs`, `uniquac_pairs`, `wilson_pairs`, `pr_kij_pairs`, `eppr78_kij_pairs`, `scalar_*` (eight) |
| `Interaction Parameters/ChemSep/*.ipd` | `ipd` | `ipd_files`, `ipd_rows`, `ipd_lines` |
| `Misc/*.json` (fitted correlations) | `correlations` | `correlation_leaves`, `correlation_series` |
| `Phase Change/*interaction parameters*.tsv`, `DDBST UNIFAC assignments.tsv`, `unifac.py` | `unifac` | `unifac_interaction_parameters`, `ddbst_unifac_assignments`, `ddbst_unifac_assignment_pairs`, `unifac_subgroups`, `unifac_main_groups` |
| `Law/*` (eight inventories) and `Phase Change/Bell 2018 ...tsv` | `thermo_knowledge.readers.chemicals.shared_specs` | `law_*`, `phase_change_bell_2018` |

The inventory and Bell files are byte-identical to the ones in the chemicals payload, so both readers
use the same declarations and the same strict cell rules (`thermo_knowledge.readers.chemicals.tabular`).
That module and `shared_specs` are therefore inputs of this reader that the reuse key of a stage does
not see (it hashes this package only); bump `READER_VERSION` when either changes the rows they produce.
The manifest includes no `.zip`, so `Law/ECHA Tonnage Bands.csv.zip` is not in this payload and its
table is not declared here. Tables embedded in the other Python modules of the library are outside the
payload (the manifest includes only `thermo/unifac.py`).
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals.shared_specs import SHARED_TABLES
from thermo_knowledge.readers.chemicals.tabular import Delimited, Handler, read_delimited, schemas_of
from thermo_knowledge.readers.thermo import correlations, interaction, ipd, unifac
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"
DATA_ROOT = "thermo"

DELIMITED: tuple[Delimited, ...] = tuple(
    spec for spec in SHARED_TABLES if not any(file.endswith(".zip") for file in spec.files)
)

TABLES: dict[str, pa.Schema] = {
    **schemas_of(DELIMITED),
    **interaction.SCHEMAS,
    **ipd.SCHEMAS,
    **correlations.SCHEMAS,
    **unifac.SCHEMAS,
}
if len(TABLES) != (
    len(DELIMITED)
    + len(interaction.SCHEMAS)
    + len(ipd.SCHEMAS)
    + len(correlations.SCHEMAS)
    + len(unifac.SCHEMAS)
):
    raise StagingError("a table name is declared twice in the thermo reader")


def _handlers() -> dict[str, Handler]:
    handlers: dict[str, Handler] = {}
    for module in (interaction, ipd, correlations, unifac):
        for artifact, handler in module.HANDLERS.items():
            if artifact in handlers:
                raise StagingError(f"{artifact}: two rules for one file")
            handlers[artifact] = handler
    for spec in DELIMITED:
        for rel in spec.files:
            artifact = f"{DATA_ROOT}/{rel}"
            if artifact in handlers:
                raise StagingError(f"{artifact}: two rules for one file")

            def handler(tree: Path, artifact: str, writer: Writer, spec: Delimited = spec) -> None:
                read_delimited(tree, DATA_ROOT, spec, artifact.removeprefix(f"{DATA_ROOT}/"), writer)

            handlers[artifact] = handler
    return handlers


HANDLERS: dict[str, Handler] = _handlers()


def read(tree: Path, writer: Writer) -> None:
    """Stage every payload file: each has a rule, or the read fails naming it."""
    for artifact in writer.payload_files:
        handler = HANDLERS.get(artifact)
        if handler is None:
            raise StagingError(
                f"{artifact}: the thermo reader has no rule for this payload file; add one "
                "and bump READER_VERSION"
            )
        handler(tree, artifact, writer)

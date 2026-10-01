# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the `chemicals` source (`sources/chemicals.toml`, v1.5.2).

Source-faithful: each data file keeps its own headings, keys, numbers and spellings; nothing is
mapped, converted, resolved or deduplicated. The payload is 155 data files and every one is
decomposed into rows; none is skipped.

| Payload | Module | Tables |
|---|---|---|
| handbook and compilation tables (`.tsv`, `.csv`, `.csv.gz`), 120 files | `specs` | one table per file, named for its folder and file |
| `Misc/Element data.csv` | `specs` | `misc_element_data` |
| `Law/*` (nine inventories) and `Phase Change/Bell 2018 ...tsv` | `thermo_knowledge.staging.shared_specs` | `law_*`, `phase_change_bell_2018` |
| `Misc/element.txt` (Open Babel element table, `#` banner and tab-separated rows) | `elements_txt` | `misc_element_txt`, `misc_element_txt_lines` |
| `Identifiers/*` | `identifiers` | `identifiers_*` |
| JSON objects keyed by CAS (12 files) | `jsonfiles` | `heat_capacity_janaf_*`, `heat_capacity_perry_2_151_json`, `heat_capacity_psi4_*`, `heat_capacity_webbook_shomate`, `misc_vdi_saturation_*`, `safety_ontario_exposure_limits_json` |
| `Misc/ChemSep8.32.xml` | `chemsep_xml` | `chemsep_library`, `chemsep_scalars`, `chemsep_equations`, `chemsep_groups` |

Conventions, all stated in `tabular`: a blank cell is null (never zero); the literal `nan` of a
float column is NaN; text is kept exactly, trailing spaces included; a CAS written without hyphens
is an integer column and one written with hyphens is text (the files use both spellings, even for
the same kind of table). A file states no unit unless its heading does; the units live in the
library's docstrings and are therefore `not stated` here. The same column name means different
things in different tables (A, B, C, D in the vapor pressure tables are not one equation), so the
table name, and not the column name, identifies the correlation form.

Every file is checked against its declaration: headings, widths, and cell kinds. A file or key the
declaration does not cover raises an error naming it.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals import chemsep_xml, elements_txt, identifiers, jsonfiles
from thermo_knowledge.readers.chemicals.specs import CHEMICALS_TABLES
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.shared_specs import SHARED_TABLES
from thermo_knowledge.staging.tabular import Delimited, Handler, read_delimited, schemas_of
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "2"
DATA_ROOT = "chemicals"

DELIMITED: tuple[Delimited, ...] = (*CHEMICALS_TABLES, *SHARED_TABLES)

TABLES: dict[str, pa.Schema] = {
    **schemas_of(DELIMITED),
    **identifiers.SCHEMAS,
    **jsonfiles.SCHEMAS,
    **chemsep_xml.SCHEMAS,
    **elements_txt.SCHEMAS,
}
if len(TABLES) != len(DELIMITED) + len(identifiers.SCHEMAS) + len(jsonfiles.SCHEMAS) + len(
    chemsep_xml.SCHEMAS
) + len(elements_txt.SCHEMAS):
    raise StagingError("a table name is declared twice in the chemicals reader")


def _handlers() -> dict[str, Handler]:
    handlers: dict[str, Handler] = {
        **identifiers.HANDLERS,
        **jsonfiles.HANDLERS,
        **chemsep_xml.HANDLERS,
        **elements_txt.HANDLERS,
    }
    for spec in DELIMITED:
        for rel in spec.files:
            artifact = f"{DATA_ROOT}/{rel}"
            if artifact in handlers:
                raise StagingError(f"{artifact}: two rules for one file")

            def handler(tree: Path, artifact: str, writer: Writer, spec: Delimited = spec) -> None:
                read_delimited(
                    tree, DATA_ROOT, spec, artifact.removeprefix(f"{DATA_ROOT}/"), writer
                )

            handlers[artifact] = handler
    return handlers


HANDLERS: dict[str, Handler] = _handlers()


def read(tree: Path, writer: Writer) -> None:
    """Stage every payload file: each has a rule, or the read fails naming it."""
    for artifact in writer.payload_files:
        handler = HANDLERS.get(artifact)
        if handler is None:
            raise StagingError(
                f"{artifact}: the chemicals reader has no rule for this payload file; add one "
                "and bump READER_VERSION"
            )
        handler(tree, artifact, writer)

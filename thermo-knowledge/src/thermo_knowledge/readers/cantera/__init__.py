# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the Cantera source (`sources/cantera.toml`, tag v3.2.0).

Source-faithful: Cantera's own keys, structure and unit strings; no conversion, no mapping to
the domain model, nothing resolved or deduplicated. The YAML is read with the YAML 1.2 core
schema that Cantera's parser (yaml-cpp) uses (`yaml12`): `name: NO` is the species NO, not a
boolean. Every YAML node has a JSON-pointer locator (`data/gri30.yaml#/species/3/thermo`), so a
species or reaction that recurs in several files stays one row per occurrence, keyed by file and
position.

| Construct | Tables |
|---|---|
| file | `yaml_files` (description, generator, versions), `yaml_duplicate_keys`, `file_sections` (each top-level key with its shape and entry kinds) |
| `units` mappings | `units_entries`: one row per dimension, with the scope (file, section, entry) and its owner's pointer |
| elements | `elements` |
| species | `species`, `species_composition`, `species_thermo` (NASA-7, NASA-9, Shomate, constant-cp, piecewise-Gibbs: model, temperature ranges and constants), `thermo_pieces` (one row per coefficient array with its temperature range), `species_transport`, `species_critical_parameters`, `species_equation_of_state` |
| phases | `phases`, `phase_references` (the species, elements and reactions a phase takes, including the section and `file/section` forms) |
| reactions | `reactions` (equation, type and flags), `reaction_terms` (the equation's terms, where it follows the documented grammar) |
| electron collisions | `collisions` |
| anything else | `other_entries`, and `entry_parameters` for every key no column declares |
| Chemkin, CTI, CTML, CSV and text fixtures | `text_files`, `text_lines` (verbatim lines; status `partly_read`) |

`entry_parameters` holds, as one row per YAML leaf, every key that has no typed column or whose
value has another type than its column (for example `rate-constant`, `Troe`, `efficiencies`,
`orders`, `state`, `activity-data`, a coefficient written as `"1.2e10 cm^3/mol/s"`). Its `block`
is the source key the leaf belongs to and `path` the JSON pointer inside the owning row. A
quantity that the source writes either as a number or as a string with a unit has a number
column and a `_text` column.

Skipped with a reason: HDF5 snapshots (binary). See `SKIPPED` and `TEXT_PARTLY`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.cantera import text
from thermo_knowledge.readers.cantera.common import Sink
from thermo_knowledge.readers.cantera.tables import SCHEMAS
from thermo_knowledge.readers.cantera.yamlfile import FileReader, load_document
from thermo_knowledge.staging import payload
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES: dict[str, pa.Schema] = dict(SCHEMAS)

SKIPPED: tuple[tuple[str, str], ...] = (
    (
        "test/data/*.h5",
        "binary HDF5 snapshot of a Cantera SolutionArray (flame or reactor output); the core "
        "environment has no HDF5 reader, and the file holds results Cantera computed, not data",
    ),
)

_CHEMKIN = (
    "Chemkin-II input kept verbatim as text lines: the mechanism and thermo cards are fixed-format "
    "sections that Cantera's ck2yaml converter reads (and many files are invalid on purpose), "
    "so they are not decomposed into columns"
)
TEXT_PARTLY: tuple[tuple[str, str], ...] = (
    ("test/data/*.inp", _CHEMKIN),
    ("test/data/*.dat", _CHEMKIN + " (thermo cards, NASA-9 tables and transport tables)"),
    (
        "test/data/*.txt",
        "test input kept verbatim as text lines; not decomposed into columns",
    ),
    (
        "test/data/*.cti",
        "legacy CTI (Python-syntax) input kept verbatim as text lines; Cantera 3 no longer reads "
        "it, only its cti2yaml converter does, so it is not decomposed into columns",
    ),
    (
        "test/data/*.xml",
        "legacy CTML (XML) input kept verbatim as text lines; Cantera 3 no longer reads it, only "
        "its ctml2yaml converter does, so it is not decomposed into columns",
    ),
    (
        "test/data/*.csv",
        "headerless numeric regression table produced by Cantera itself; its columns are stated "
        "only by the test that reads it, so it is kept verbatim as text lines",
    ),
    (
        "data/example_data/*.md",
        "prose (the illustration-only disclaimer) kept verbatim as text lines",
    ),
)


def _first(rules: tuple[tuple[str, str], ...], path: str) -> str | None:
    for pattern, reason in rules:
        if payload.matches(pattern, path):
            return reason
    return None


def read(tree: Path, writer: Writer) -> None:
    """Read every payload file of the source: decompose a YAML file, keep any other text file
    verbatim as lines, or list it as skipped with a reason."""
    sink = Sink(writer)
    for artifact in writer.payload_files:
        if artifact.endswith(".yaml"):
            document, duplicates = load_document(tree, artifact)
            FileReader(artifact, sink).read(document, duplicates)
        elif (reason := _first(SKIPPED, artifact)) is not None:
            writer.skipped(artifact, reason)
        elif (reason := _first(TEXT_PARTLY, artifact)) is not None:
            text.read_text_lines(tree, artifact, sink)
            writer.partly_read(artifact, reason)
        else:
            raise StagingError(
                f"{artifact}: the Cantera reader has no rule for this payload file; add one and "
                "bump READER_VERSION"
            )
    sink.flush()

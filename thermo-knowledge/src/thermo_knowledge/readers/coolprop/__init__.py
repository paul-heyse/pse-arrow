# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the CoolProp source (`sources/coolprop.toml`, tag v8.0.0).

Source-faithful: CoolProp's own names, keys and units, no conversion, no renaming to the domain
model. Numbers are the JSON numbers (stored as float64 where a field mixes integer and float
spellings); an absent value is null. A JSON object is consumed key by key and a key no column
declares is an error, so nothing is silently dropped.

| Payload | Tables |
|---|---|
| `dev/fluids/*.json` | `fluids`, `eos_entries`, `states`, `alphar_terms`, `alphar_term_rows`, `alpha0_terms`, `alpha0_term_rows`, `ancillary_equations`, `ancillary_equation_rows`, `melting_lines`, `melting_line_parts`, `melting_line_part_rows`, `surface_tension`, `surface_tension_rows`, `critical_region_splines`, `critical_region_spline_rows`, `superancillaries`, `superancillary_check_points`, `superancillary_expansions`, `transport_objects`, `transport_scalars`, `transport_rows` |
| `dev/mixtures/` JSON | `binary_pairs` (also `old_BIP.json`), `departure_functions`, `departure_function_rows`, `predefined_mixtures`, `predefined_mixture_components` |
| `dev/cubics/all_cubic_fluids.json` | `cubic_fluids`, `cubic_alpha0_terms`, `cubic_alpha0_term_rows` |
| `dev/pcsaft/*.json` | `pcsaft_fluids`, `pcsaft_binary_pairs` |
| `dev/incompressible_liquids/json/*.json` | `incompressible_fluids`, `incompressible_functions`, `incompressible_coefficients` |
| text tables, grids, schemas | `text_files`, `text_lines` (verbatim lines; status `partly_read`) |

Term objects are split into a scalar table (`*_terms`: one row per term object, with the
fluid, EOS index, term index and type) and a long table (`*_term_rows`: one row per coefficient
row, one nullable column per coefficient name; parallel arrays become rows). `array_lengths`
keeps each array's length, so the arrays can be rebuilt exactly.

Skipped with a reason: code (`*.py`), a notebook, Pascal sources, Excel workbooks and housekeeping
files; see `SKIPPED`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.coolprop import cubics, fluids, incompressible, mixtures, text
from thermo_knowledge.readers.coolprop.common import Sink, merge_schemas
from thermo_knowledge.staging import payload
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES: dict[str, pa.Schema] = merge_schemas(
    fluids.SCHEMAS,
    mixtures.SCHEMAS,
    cubics.SCHEMAS,
    incompressible.SCHEMAS,
    text.SCHEMAS,
)

CODE = "Python script: code, not data; it records how CoolProp's JSON was produced"
SKIPPED: tuple[tuple[str, str], ...] = (
    *(
        (
            f"dev/incompressible_liquids/CPIncomp/{name}.py",
            "Python class infrastructure (base classes, data holders, JSON writer): code, not data",
        )
        for name in ("BaseObjects", "DataObjects", "WriterObjects")
    ),
    (
        "dev/incompressible_liquids/CPIncomp/__init__.py",
        "Python package initialiser: code, not data",
    ),
    (
        "dev/incompressible_liquids/CPIncomp/*.py",
        "embedded Python classes: vendor and literature grids sit in numpy array literals "
        "interleaved with unit-conversion statements and fit-type choices; decomposing them needs "
        "a Python source parser and a convention for the conversions, which is out of proportion "
        "for a source-faithful reader (the fitted results are read from json/*.json)",
    ),
    (
        "dev/incompressible_liquids/CPIncomp/data/SecCool/*.pas",
        "Object Pascal source with coefficient arrays and an evaluator: a program, not a data "
        "format",
    ),
    (
        "dev/incompressible_liquids/CPIncomp/data/SecCool/*.xlsx",
        "binary Excel workbooks (the two are byte-identical); the core environment has no "
        "workbook reader and their tables are out of proportion to decompose here",
    ),
    ("dev/incompressible_liquids/*.py", CODE + " (fitting, testing and transport helpers)"),
    ("dev/incompressible_liquids/*.ipynb", "Jupyter notebook of derivation code, not data"),
    ("dev/incompressible_liquids/.gitignore", "repository housekeeping; no data"),
    ("dev/incompressible_liquids/CPIncomp/data/.gitignore", "repository housekeeping; no data"),
    ("dev/mixtures/*.py", CODE + " (table conversion and maintenance)"),
    ("dev/cubics/*.py", CODE + " (generates the cubic listing from the Helmholtz library)"),
)

TEXT_PARTLY = (
    (
        "dev/mixtures/*.txt",
        "transcription of a published table kept verbatim as text lines; the files state no "
        "column headings (the converter scripts do), so the rows are not decomposed into columns",
    ),
    (
        "dev/mixtures/*.tsv",
        "tab-separated table with comment lines kept verbatim as text lines; not decomposed "
        "into columns",
    ),
    (
        "dev/mixtures/*_schema.json",
        "JSON Schema document kept verbatim as text lines; its property declarations are not "
        "decomposed",
    ),
    (
        "dev/cubics/*_schema.json",
        "JSON Schema document kept verbatim as text lines; its property declarations are not "
        "decomposed",
    ),
    (
        "dev/pcsaft/*_schema.json",
        "JSON Schema document (with units stated per field) kept verbatim as text lines; its "
        "property declarations are not decomposed",
    ),
    (
        "dev/incompressible_liquids/CPIncomp/data/**/*.txt",
        "numeric property grid kept verbatim as text lines; grids are not decomposed into columns "
        "(their units are not stated in the files)",
    ),
    (
        "dev/incompressible_liquids/CPIncomp/data/**/*.csv",
        "SecCool table with free-text header lines kept verbatim as text lines (files that are "
        "not UTF-8 are decoded as latin-1); not decomposed into columns",
    ),
)


def _first(rules: tuple[tuple[str, str], ...], path: str) -> str | None:
    for pattern, reason in rules:
        if payload.matches(pattern, path):
            return reason
    return None


def read(tree: Path, writer: Writer) -> None:
    """Read every payload file of the source: decompose it, keep it verbatim as lines, or list
    it as skipped with a reason."""
    sink = Sink(writer)
    for artifact in writer.payload_files:
        if payload.matches("dev/fluids/*.json", artifact):
            fluids.read_fluid(tree, artifact, sink)
        elif artifact in mixtures.BINARY_PAIR_FILES:
            mixtures.read_binary_pairs(tree, artifact, sink)
        elif artifact == mixtures.DEPARTURE_FILE:
            mixtures.read_departure_functions(tree, artifact, sink)
        elif artifact == mixtures.PREDEFINED_FILE:
            mixtures.read_predefined_mixtures(tree, artifact, sink)
        elif artifact == cubics.CUBIC_FILE:
            cubics.read_cubic_fluids(tree, artifact, sink)
        elif artifact == cubics.PCSAFT_FILE:
            cubics.read_pcsaft_fluids(tree, artifact, sink)
        elif artifact == cubics.PCSAFT_PAIR_FILE:
            cubics.read_pcsaft_pairs(tree, artifact, sink)
        elif payload.matches("dev/incompressible_liquids/json/*.json", artifact):
            incompressible.read_incompressible(tree, artifact, sink)
        elif (reason := _first(SKIPPED, artifact)) is not None:
            writer.skipped(artifact, reason)
        elif (reason := _first(TEXT_PARTLY, artifact)) is not None:
            text.read_text_lines(tree, artifact, sink)
            writer.partly_read(artifact, reason)
        else:
            raise StagingError(
                f"{artifact}: the CoolProp reader has no rule for this payload file; add one and "
                "bump READER_VERSION"
            )
    sink.flush()

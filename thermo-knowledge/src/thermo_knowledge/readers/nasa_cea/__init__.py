# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the NASA CEA source (`sources/nasa_cea.toml`, tag v3.3.4): `data/thermo.inp` and
`data/trans.inp`, fixed-column Fortran text.

Source-faithful: every field is sliced at the columns of its format (NASA TP-2002-211556,
Appendix A for thermo.inp; see `thermo.py` and `trans.py`) and read with Fortran formatted-read
rules (`fortran.py`): the exponent letters D and E and a space in place of an exponent sign are
read, and a blank numeric field is null, not zero. Nothing is mapped, converted or resolved;
each table keeps the raw text of the lines it was read from. The files state no units. Both
files use CRLF line endings; the raw lines are kept without them.

| File | Tables |
|---|---|
| `data/thermo.inp` | `frame_lines` (comment block, the word thermo, the temperature grid, the END sentinels), `species_records` (name, notes, interval count, reference-date code, phase flag, molecular weight, heat of formation), `species_formula_pairs` (the element-count pairs), `thermo_intervals` (bounds, exponents, H(298.15)-H(0) and the nine coefficients of each interval) |
| `data/trans.inp` | `frame_lines` (title and end lines), `trans_entries` (names, interval counts, reference), `trans_intervals` (bounds and four coefficients per viscosity or conductivity interval) |

A line is located as `<file>#L<line>`; the pairs of one record add `/<slot>`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.nasa_cea.fortran import Line
from thermo_knowledge.readers.nasa_cea.rows import Rows
from thermo_knowledge.readers.nasa_cea.tables import TABLES as _TABLES
from thermo_knowledge.readers.nasa_cea.thermo import read_thermo
from thermo_knowledge.readers.nasa_cea.trans import read_trans
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

TABLES: dict[str, pa.Schema] = dict(_TABLES)

THERMO = "data/thermo.inp"
TRANS = "data/trans.inp"


def read_lines(tree: Path, artifact: str) -> list[Line]:
    """The lines of `artifact` without their line endings (CRLF or LF)."""
    try:
        content = (tree / artifact).read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as text: {error}") from error
    parts = content.split("\n")
    if parts and parts[-1] == "":
        parts.pop()
    return [
        Line(artifact, number, text.removesuffix("\r")) for number, text in enumerate(parts, 1)
    ]


def read(tree: Path, writer: Writer) -> None:
    """Read every payload file of the source."""
    rows = Rows(writer)
    for artifact in writer.payload_files:
        if artifact == THERMO:
            read_thermo(read_lines(tree, artifact), rows)
        elif artifact == TRANS:
            read_trans(read_lines(tree, artifact), rows)
        else:
            raise StagingError(
                f"{artifact}: the NASA CEA reader has no rule for this payload file; add one "
                "and bump READER_VERSION"
            )
    rows.flush()

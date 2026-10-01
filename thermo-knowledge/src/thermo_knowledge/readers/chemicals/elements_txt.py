# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The Open Babel element table (`Misc/element.txt`).

The file is a `#` banner (the Open Babel and Blue Obelisk provenance, the GPL notice and a
description of every column), a `#` legend line naming the columns, one commented-out row
(`#0 Xx ... Dummy`) and then one tab-separated row per element, atomic number 1 to 118.

| Table | Content |
|---|---|
| `misc_element_txt` | one row per element, with the legend's own columns (`Num`, `Symb`, `ARENeg`, ... `Name`) |
| `misc_element_txt_lines` | every line that is not an element row (banner, legend, the commented-out dummy row, blank lines), as written |

A row is checked against the legend: the legend must be the declared one and a row must have its
width, and a cell that is not a value of its kind is an error. The banner states the unit of some
columns (Angstrom, amu, eV) and states `0.0 if unknown` (or `1.6`, `2.0`, `6`) for the
electronegativities, radii and valence; those values stay as written, the file does not mark a
missing value any other way. `_locator` is `<artifact>#L<line>`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging import tabular
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.tabular import integer, number, text
from thermo_knowledge.staging.writer import Writer

FILE = "chemicals/Misc/element.txt"
_LEGEND = "#Num\tSymb\tARENeg\tRCov\tRBO\tRVdW\tMaxBnd\tMass\tElNeg.\tIonization\tElAffinity\tRed\tGreen\tBlue\tName"

_UNKNOWN = "the banner states {default} when unknown; kept as written"
_ELEMENTS = (
    integer("Num", "Num", note="atomic number"),
    text("Symb", "Symb", note="elemental symbol"),
    number(
        "ARENeg",
        "ARENeg",
        note="Allred and Rochow electronegativity; " + _UNKNOWN.format(default="0.0"),
    ),
    number(
        "RCov",
        "RCov",
        unit="Angstrom",
        note="covalent radius; " + _UNKNOWN.format(default="1.6"),
    ),
    number(
        "RBO",
        "RBO",
        note="'bond order' radius; the banner says it is ignored and kept for compatibility",
    ),
    number(
        "RVdW",
        "RVdW",
        unit="Angstrom",
        note="van der Waals radius; " + _UNKNOWN.format(default="2.0"),
    ),
    integer("MaxBnd", "MaxBnd", note="maximum bond valence; " + _UNKNOWN.format(default="6")),
    number("Mass", "Mass", unit="amu", note="IUPAC recommended atomic mass"),
    number("ElNeg", "ElNeg.", note="Pauling electronegativity; " + _UNKNOWN.format(default="0.0")),
    number(
        "Ionization",
        "Ionization",
        unit="eV",
        note="ionization potential; " + _UNKNOWN.format(default="0.0"),
    ),
    number(
        "ElAffinity",
        "ElAffinity",
        unit="eV",
        note="electron affinity; " + _UNKNOWN.format(default="0.0"),
    ),
    number("Red", "Red", note="default visualization colour, red component"),
    number("Green", "Green", note="default visualization colour, green component"),
    number("Blue", "Blue", note="default visualization colour, blue component"),
    text("Name", "Name", note="element name in English"),
)
_LINES = (
    integer("line", "line number", note="one-based"),
    text("text", "the line as written"),
)

SCHEMAS: dict[str, pa.Schema] = {
    "misc_element_txt": tabular.schema(*_ELEMENTS),
    "misc_element_txt_lines": tabular.schema(*_LINES),
}


def read_elements(tree: Path, artifact: str, writer: Writer) -> None:
    lines = tabular.split_lines(tabular.read_text(tree, artifact))
    rows: list[dict[str, object]] = []
    other: list[dict[str, object]] = []
    legend_seen = False
    for number_, line in enumerate(lines, start=1):
        place = tabular.where(artifact, number_)
        if line.startswith("#") or line.strip() == "":
            if line.startswith("#Num"):
                if line != _LEGEND:
                    raise StagingError(
                        f"{place}: the legend {line!r} differs from the declared one"
                    )
                legend_seen = True
            other.append({"_artifact": artifact, "_locator": place, "line": number_, "text": line})
            continue
        if not legend_seen:
            raise StagingError(f"{place}: an element row before the legend line")
        cells = line.split("\t")
        if len(cells) != len(_ELEMENTS):
            raise StagingError(f"{place}: {len(cells)} cells, the legend names {len(_ELEMENTS)}")
        row: dict[str, object] = {"_artifact": artifact, "_locator": place}
        for spec, cell in zip(_ELEMENTS, cells, strict=True):
            row[spec.name] = tabular.value_of(spec.kind, cell, place, spec.name)
        rows.append(row)
    if not legend_seen:
        raise StagingError(f"{artifact}: no legend line (#Num ...) found")
    writer.rows("misc_element_txt", rows)
    writer.rows("misc_element_txt_lines", other)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {FILE: read_elements}

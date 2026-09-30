# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The ChemSep interaction-parameter files in IPD text form (`ChemSep/nrtl.ipd`, `pr.ipd`).

An IPD file is a free-text banner, an `[IPD]` marker, optional `Comment=` and `Units=` lines, a
legend comment of the form `# ID/CASN ID/CASN <value names> Name/Name Comments`, and then one row per
pair: two identifiers, as many numbers as the legend names, and a free-text remainder.

| Table | Content |
|---|---|
| `ipd_files` | one row per file: `Comment=`, `Units=` (null when absent), the legend line and the value names it lists |
| `ipd_rows` | one row per pair: `ID1` and `ID2` (the identifier tokens as written, CAS numbers in these files), `values` (the numbers, in legend order; leading-zero-less spellings such as `.187e-1` are read as decimal numbers) and `remainder` (the text after the numbers: `Name/Name` and the comment, whose split is ambiguous because names contain spaces) |
| `ipd_lines` | every line that is not a data row (banner, marker, comment lines, legend, blank lines) |

The files use CRLF line ends; the carriage return is dropped. `_locator` is `<artifact>#L<line>`.
The parameters' units are whatever `Units=` says (`nrtl.ipd` states cal/mol; `pr.ipd` states none,
the PR k12 being dimensionless in the library); the legend names the values (A12, A21, alpha12 or k12).
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging import tabular
from thermo_knowledge.staging.tabular import integer, numbers, text, texts
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

FILES = (
    "thermo/Interaction Parameters/ChemSep/nrtl.ipd",
    "thermo/Interaction Parameters/ChemSep/pr.ipd",
)
_LEGEND_START = ("ID/CASN", "ID/CASN")
_LEGEND_END = "Name/Name"

_FILES = (
    text("Comment", "Comment= line", note="null when the file has none"),
    text("Units", "Units= line", note="null when the file has none"),
    text("legend", "the # ID/CASN ... comment line"),
    texts("value_names", "legend, between the two ID/CASN words and Name/Name"),
)
_ROWS = (
    text("ID1", "first identifier"),
    text("ID2", "second identifier"),
    numbers("values", "the numbers named by the legend", note="in legend order"),
    text("remainder", "Name/Name Comments", note="the text after the numbers, as written"),
)
_LINES = (
    integer("line", "line number", note="one-based"),
    text("text", "line without its carriage return"),
)

SCHEMAS: dict[str, pa.Schema] = {
    "ipd_files": tabular.schema(*_FILES),
    "ipd_rows": tabular.schema(*_ROWS),
    "ipd_lines": tabular.schema(*_LINES),
}


def read_ipd(tree: Path, artifact: str, writer: Writer) -> None:
    content = tabular.read_text(tree, artifact)
    lines = [line.removesuffix("\r") for line in tabular.split_lines(content)]
    comment = units = legend = None
    value_names: list[str] = []
    in_data = False
    rows: list[dict[str, object]] = []
    other: list[dict[str, object]] = []
    for number, line in enumerate(lines, start=1):
        place = tabular.where(artifact, number)
        if not in_data:
            if line.startswith("Comment="):
                comment = line.removeprefix("Comment=")
            elif line.startswith("Units="):
                units = line.removeprefix("Units=")
            elif line.startswith("#") and "ID/CASN" in line:
                legend = line
                tokens = line.removeprefix("#").split()
                if tuple(tokens[:2]) != _LEGEND_START or _LEGEND_END not in tokens:
                    raise StagingError(f"{place}: an unrecognised legend {line!r}")
                value_names = tokens[2 : tokens.index(_LEGEND_END)]
                in_data = True
            other.append({"_artifact": artifact, "_locator": place, "line": number, "text": line})
            continue
        if line.strip() == "":
            other.append({"_artifact": artifact, "_locator": place, "line": number, "text": line})
            continue
        tokens = line.split(None, 2 + len(value_names))
        if len(tokens) < 2 + len(value_names):
            raise StagingError(f"{place}: {len(tokens)} fields, the legend needs {2 + len(value_names)}")
        values = [
            tabular.value_of("f", token, place, name)
            for token, name in zip(tokens[2 : 2 + len(value_names)], value_names, strict=True)
        ]
        if any(value is None for value in values):
            raise StagingError(f"{place}: a blank value")
        remainder = tokens[2 + len(value_names)] if len(tokens) > 2 + len(value_names) else None
        rows.append(
            {
                "_artifact": artifact,
                "_locator": place,
                "ID1": tokens[0],
                "ID2": tokens[1],
                "values": values,
                "remainder": remainder,
            }
        )
    if legend is None:
        raise StagingError(f"{artifact}: no legend line (# ID/CASN ...) found")
    writer.rows(
        "ipd_files",
        [
            {
                "_artifact": artifact,
                "_locator": f"{artifact}#L1-L{len(lines)}",
                "Comment": comment,
                "Units": units,
                "legend": legend,
                "value_names": value_names,
            }
        ],
    )
    writer.rows("ipd_rows", rows)
    writer.rows("ipd_lines", other)
    writer.opened(artifact)


HANDLERS: dict[str, tabular.Handler] = {path: read_ipd for path in FILES}

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reader of the NIST-JANAF tables (`sources/janaf.toml`): one plain-text, tab-separated file
per substance and phase set (`<element>-<nnn>.txt`).

A table file is

1. a title line of two tab-separated fields, `<substance name> (<formula>)` and
   `<JANAF formula>(<phase designator>)`,
2. the header line `T(K)<TAB>Cp<TAB>S<TAB>-[G-H(Tr)]/T<TAB>H-H(Tr)<TAB>delta-f H<TAB>delta-f G<TAB>log Kf`,
3. one line per temperature and a few separator lines.

The staged tables:

- `tables`: one row per file with both title fields as written, the parts the title's grammar
  separates exactly (the last parenthesised group of each field; null when a field does not end
  in one), the header line and line counts.
- `rows`: one row per line after the header, `raw_line` being the line exactly. `row_kind` is
  `data`, `plus` (a line holding only `+`, with or without a tab), `empty` (tabs only) or `blank`
  (no content). For a data row each of the eight columns has its cell text exactly as written
  (`<column>_text`: null when the line ends before the cell, empty when the cell is empty) and, where
  the cell is a plain number, its value (`<column>`); `INFINITE`, blank cells, marker text and any
  other text leave the value null, so a missing value is never a zero.

Phase changes are two rows at one temperature: the first carries, where the formation enthalpy
would stand, a marker text naming the change (`marker`), the second the text `TRANSITION` (or a
fugacity or pressure statement), also in `marker`.

Two irregularities of the files are recognised exactly. Where the three formation cells are one
space-separated field of three numbers, the three values are assigned to `delta_f_h`,
`delta_f_g` and `log_kf` and `fused_cells` says so; the field stays in `delta_f_h_text`. Where two
numbers run together in one cell (a digit string with two decimal points: a fixed-width column
overflowed), the cell boundaries of the line no longer match the columns, so no positional value
is assigned and `parse_note` says why; the line is in `raw_line`. The units are not in the files
except the header's `T(K)`.
"""

from __future__ import annotations

import re
from pathlib import Path

import pyarrow as pa

from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import (
    BOOL,
    FLOAT64,
    INT64,
    NOT_STATED,
    STRING,
    column,
    table_schema,
)
from thermo_knowledge.staging.writer import Writer

READER_VERSION = "1"

NOT_APPLICABLE = "not applicable"
HEADER = "T(K)\tCp\tS\t-[G-H(Tr)]/T\tH-H(Tr)\tdelta-f H\tdelta-f G\tlog Kf"
COLUMNS: tuple[tuple[str, str, str], ...] = (
    ("t", "T(K)", "K"),
    ("cp", "Cp", NOT_STATED),
    ("s", "S", NOT_STATED),
    ("g_function", "-[G-H(Tr)]/T", NOT_STATED),
    ("h_increment", "H-H(Tr)", NOT_STATED),
    ("delta_f_h", "delta-f H", NOT_STATED),
    ("delta_f_g", "delta-f G", NOT_STATED),
    ("log_kf", "log Kf", NOT_STATED),
)
FORMATION = ("delta_f_h", "delta_f_g", "log_kf")
MARKER_AT = 5
NUMBER = re.compile(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
FUSED_TRIPLE = re.compile(r"^\s*(\S+)\s+(\S+)\s+(\S+)\s*$")
RUN_TOGETHER = re.compile(r"\d*\.\d+\.\d")
FILE_NAME = re.compile(r"^[A-Za-z]{1,2}-\d{3}\.txt$")


def _text(
    name: str,
    source_name: str,
    *,
    nullable: bool = True,
    unit: str = NOT_APPLICABLE,
    note: str | None = None,
) -> pa.Field:
    return column(name, STRING, source_name=source_name, unit=unit, note=note, nullable=nullable)


def _int(name: str, source_name: str, *, nullable: bool = False) -> pa.Field:
    return column(name, INT64, source_name=source_name, unit=NOT_APPLICABLE, nullable=nullable)


TABLES_SCHEMA = table_schema(
    _text("code", "file name without .txt (element symbol, hyphen, sequence number)", nullable=False),
    _text("title_field_1", "title line, first tab-separated field", nullable=False, note="verbatim"),
    _text("title_field_2", "title line, second tab-separated field", nullable=False, note="verbatim"),
    _text(
        "substance_name",
        "title field 1 before its last parenthesised group",
        note="null when the field does not end in a parenthesised group",
    ),
    _text(
        "name_formula",
        "the last parenthesised group of title field 1",
        note="the formula the title writes in the name, parentheses inside it balanced; null as above",
    ),
    _text(
        "janaf_formula",
        "title field 2 before its last parenthesised group",
        note="the JANAF formula with atom counts; null when the field does not end in a group",
    ),
    _text(
        "phase_designator",
        "the last parenthesised group of title field 2",
        note="for example g, cr, l, cr,l, ref, l,g, fl; null as above",
    ),
    _text("header_line", "line 2", nullable=False, note="verbatim; the reader requires the standard header"),
    _int("line_count", "number of lines in the file"),
    _int("data_row_count", "lines after the header of kind data"),
    _int("plus_count", "lines after the header of kind plus"),
    _int("empty_count", "lines after the header of kind empty"),
    _int("blank_count", "lines after the header of kind blank"),
    column(
        "ends_with_newline",
        BOOL,
        source_name="the file ends with a line feed",
        unit=NOT_APPLICABLE,
        nullable=False,
    ),
)


def _row_columns() -> list[pa.Field]:
    fields: list[pa.Field] = []
    for name, heading, unit in COLUMNS:
        fields.append(
            column(
                name,
                FLOAT64,
                source_name=heading,
                unit=unit,
                note="set only where the cell is a plain number; INFINITE, blank and marker cells leave it null",
            )
        )
        fields.append(
            _text(
                f"{name}_text",
                f"{heading} (the cell as written)",
                unit=unit if name == "t" else NOT_STATED,
                note="null when the line ends before the cell; empty when the cell is empty",
            )
        )
    return fields


ROWS = table_schema(
    _int("line_number", "line of the file (1-based)"),
    _int("row_index", "position among the lines after the header (0-based)"),
    _text("row_kind", "data, plus, empty or blank", nullable=False),
    _text("raw_line", "the line exactly, without its line feed", nullable=False),
    _int("cell_count", "number of tab-separated cells", nullable=True),
    *_row_columns(),
    _text(
        "marker",
        "the cell where delta-f H stands when it is text",
        note="a phase-change name, TRANSITION, or a fugacity or pressure statement",
    ),
    _text(
        "fused_cells",
        "columns whose values come from one space-separated field",
        note="comma-separated column names; null when the row has none",
    ),
    _text(
        "parse_note",
        "why positional values were not assigned",
        note="set when digits of two cells run together; null otherwise",
    ),
)

TABLES: dict[str, pa.Schema] = {"tables": TABLES_SCHEMA, "rows": ROWS}


def _located(artifact: str, line: int, message: str) -> StagingError:
    return StagingError(f"{artifact}#L{line}: {message}")


def _last_group(field: str, separator: str) -> tuple[str, str] | None:
    """`(before, inside)` of the last parenthesised group of `field`, which must end the field
    and be preceded by `separator`; parentheses inside the group are balanced."""
    if not field.endswith(")"):
        return None
    depth = 0
    for position in range(len(field) - 1, -1, -1):
        if field[position] == ")":
            depth += 1
        elif field[position] == "(":
            depth -= 1
            if depth == 0:
                before = field[:position]
                if separator and not before.endswith(separator):
                    return None
                return before[: len(before) - len(separator)], field[position + 1 : -1]
    return None


def _title_parts(field_1: str, field_2: str) -> dict[str, str | None]:
    first = _last_group(field_1, " ")
    second = _last_group(field_2, "")
    return {
        "substance_name": first[0] if first else None,
        "name_formula": first[1] if first else None,
        "janaf_formula": second[0] if second else None,
        "phase_designator": second[1] if second else None,
    }


def _row(artifact: str, line_number: int, row_index: int, line: str) -> dict[str, object]:
    row: dict[str, object] = {
        "_artifact": artifact,
        "_locator": f"{artifact}#L{line_number}",
        "line_number": line_number,
        "row_index": row_index,
        "raw_line": line,
    }
    if "\r" in line:
        raise _located(artifact, line_number, "a carriage return; the files end lines with a line feed only")
    if line.strip() == "" and "\t" not in line:
        row["row_kind"] = "blank"
        return row
    cells = line.split("\t")
    row["cell_count"] = len(cells)
    if line.strip(" \t") == "+":
        row["row_kind"] = "plus"
        return row
    if all(cell.strip() == "" for cell in cells):
        row["row_kind"] = "empty"
        return row
    row["row_kind"] = "data"
    if any(cell.strip() for cell in cells[len(COLUMNS) :]):
        raise _located(
            artifact, line_number, "a non-empty cell beyond the eighth column; extend the reader"
        )
    if any(RUN_TOGETHER.search(cell) for cell in cells):
        row["parse_note"] = (
            "a cell holds two numbers run together (a digit string with two decimal points), so "
            "the cell boundaries no longer match the columns; no positional value is assigned"
        )
        return row
    for position, (name, _, _) in enumerate(COLUMNS):
        if position < len(cells):
            cell = cells[position]
            row[f"{name}_text"] = cell
            if NUMBER.match(cell):
                row[name] = float(cell)
    formation = cells[MARKER_AT] if MARKER_AT < len(cells) else None
    if formation is not None and formation.strip() and not NUMBER.match(formation):
        triple = FUSED_TRIPLE.match(formation)
        if triple and all(NUMBER.match(part) for part in triple.groups()):
            for name, part in zip(FORMATION, triple.groups(), strict=True):
                row[name] = float(part)
            row["fused_cells"] = ",".join(FORMATION)
        elif formation != "INFINITE":
            row["marker"] = formation
    return row


def read_table_file(tree: Path, artifact: str) -> tuple[dict[str, object], list[dict[str, object]]]:
    """The `tables` row and the `rows` rows of one file."""
    if not FILE_NAME.match(artifact):
        raise StagingError(f"{artifact}: not a JANAF table file name (<element>-<nnn>.txt)")
    try:
        text = (tree / artifact).read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as UTF-8 text: {error}") from error
    ends = text.endswith("\n")
    lines = text.split("\n")
    if ends:
        lines.pop()
    if len(lines) < 2:
        raise _located(artifact, 1, "a table file needs a title line and a header line")
    title = lines[0].split("\t")
    if len(title) != 2:
        raise _located(artifact, 1, f"the title line has {len(title)} tab-separated fields, not two")
    if lines[1] != HEADER:
        raise _located(artifact, 2, f"the header line is not the standard header: {lines[1]!r}")
    rows = [_row(artifact, number, number - 3, line) for number, line in enumerate(lines[2:], 3)]
    kinds = [row["row_kind"] for row in rows]
    summary: dict[str, object] = {
        "_artifact": artifact,
        "_locator": f"{artifact}#L1",
        "code": artifact[: -len(".txt")],
        "title_field_1": title[0],
        "title_field_2": title[1],
        **_title_parts(title[0], title[1]),
        "header_line": lines[1],
        "line_count": len(lines),
        "data_row_count": kinds.count("data"),
        "plus_count": kinds.count("plus"),
        "empty_count": kinds.count("empty"),
        "blank_count": kinds.count("blank"),
        "ends_with_newline": ends,
    }
    return summary, rows


def read(tree: Path, writer: Writer) -> None:
    """Decompose every table file of the payload."""
    summaries: list[dict[str, object]] = []
    pending: list[dict[str, object]] = []
    for artifact in writer.payload_files:
        summary, rows = read_table_file(tree, artifact)
        summaries.append(summary)
        pending.extend(rows)
        if len(pending) >= 50_000:
            writer.rows("rows", pending)
            pending = []
    if pending:
        writer.rows("rows", pending)
    writer.rows("tables", summaries)

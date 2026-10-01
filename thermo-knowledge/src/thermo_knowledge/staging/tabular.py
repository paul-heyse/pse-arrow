# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Delimited text tables and typed JSON values, shared by the `chemicals` and `thermo` readers.

A table is declared once as a compact column list, `heading[|column][:kind[:unit]]` items joined
by `;`:

| Part | Meaning |
|---|---|
| `heading` | the heading the file states (checked against line 1, character for character) |
| `column` | the staged column name; without it the heading is cleaned (runs of characters other than letters and digits become `_`, a leading digit gets `x_`, a repeated name gets `_2`, `_3`) |
| `kind` | `s` text (default), `f` float64, `i` int64, `c` CAS Registry Number written without hyphens (int64), `b` the literals True and False |
| `unit` | the unit the heading states; a heading that states none leaves `not stated` |

Reading is strict and keeps the source's spelling: a cell is split on tabs (quotes are ordinary
characters unless the table is declared `quoted`), a row whose width differs from the heading is an
error, and a cell that is not a value of its kind is an error that names the file, the line and the
column. A blank cell is null in every kind (an all-space cell too, for the numeric kinds), so a
missing value is never zero; the literal `nan` in a float column is NaN, which stays distinct from
null. Text is kept exactly, trailing spaces included. `_locator` is `<artifact>#L<line>`, the
physical line of the decompressed text (a gzip file or the one member of a zip archive).
"""

from __future__ import annotations

import csv
import gzip
import io
import json
import math
import re
import zipfile
from collections.abc import Callable, Iterator, Mapping, Sequence
from dataclasses import dataclass
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

NOT_APPLICABLE = "not applicable"

_TYPES: dict[str, pa.DataType] = {
    "s": STRING,
    "f": FLOAT64,
    "i": INT64,
    "c": INT64,
    "b": BOOL,
    "S": pa.list_(STRING),
    "F": pa.list_(FLOAT64),
    "N": pa.list_(INT64),
}
_NOTES: dict[str, str] = {
    "s": "a blank cell is null; other text is kept exactly, spaces included",
    "f": "a blank or all-space cell is null; the literal nan is NaN; other cells are decimal numbers",
    "i": "a blank cell is null; other cells are integers",
    "c": (
        "CAS Registry Number written without hyphens as an integer (the last digit is the check "
        "digit); a blank cell is null"
    ),
    "b": "the literal True or False; a blank cell is null",
}
_NUMBER = re.compile(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$")
_INTEGER = re.compile(r"^[+-]?\d+$")
_NON_NAME = re.compile(r"[^A-Za-z0-9]+")


@dataclass(frozen=True)
class Col:
    """One documented column of a staged table."""

    name: str
    kind: str
    source_name: str
    unit: str = NOT_STATED
    note: str | None = None
    heading: str | None = None
    """The heading the file states for this column, or `None` for a file without headings."""

    def field(self) -> pa.Field:
        return column(
            self.name,
            _TYPES[self.kind],
            source_name=self.source_name,
            unit=self.unit,
            note=self.note if self.note is not None else _NOTES.get(self.kind),
        )


def text(
    name: str, source: str | None = None, *, unit: str = NOT_STATED, note: str | None = None
) -> Col:
    return Col(name, "s", source or name, unit, note)


def number(
    name: str, source: str | None = None, *, unit: str = NOT_STATED, note: str | None = None
) -> Col:
    return Col(name, "f", source or name, unit, note)


def integer(
    name: str, source: str | None = None, *, unit: str = NOT_STATED, note: str | None = None
) -> Col:
    return Col(name, "i", source or name, unit, note)


def flag(name: str, source: str | None = None, *, note: str | None = None) -> Col:
    return Col(name, "b", source or name, NOT_STATED, note)


def texts(name: str, source: str | None = None, *, note: str | None = None) -> Col:
    return Col(name, "S", source or name, NOT_STATED, note)


def numbers(
    name: str, source: str | None = None, *, unit: str = NOT_STATED, note: str | None = None
) -> Col:
    return Col(name, "F", source or name, unit, note)


def index(name: str, source: str) -> Col:
    return Col(name, "i", source, NOT_APPLICABLE, "zero-based position")


def schema(*columns: Col) -> pa.Schema:
    return table_schema(*(c.field() for c in columns))


def clean_name(heading: str, position: int) -> str:
    """The staged column name of a heading that the declaration does not name."""
    cleaned = _NON_NAME.sub("_", heading).strip("_")
    if not cleaned:
        return f"column_{position + 1}"
    return f"x_{cleaned}" if cleaned[0].isdigit() else cleaned


def parse_columns(spec: str, *, headings: bool = True) -> tuple[Col, ...]:
    """The columns of a compact declaration (see the module docstring)."""
    columns: list[Col] = []
    used: dict[str, int] = {}
    for position, item in enumerate(spec.split(";")):
        head, _, rest = item.partition(":")
        kind, _, unit = rest.partition(":")
        kind = kind or "s"
        if kind not in _TYPES:
            raise StagingError(f"column declaration {item!r}: unknown kind {kind!r}")
        heading, _, given = head.partition("|")
        name = given or clean_name(heading, position)
        seen = used.get(name, 0) + 1
        used[name] = seen
        note: str | None = None
        if seen > 1:
            name = f"{name}_{seen}"
            note = (
                f"the file repeats the heading {heading!r}; this is occurrence {seen}; "
                f"{_NOTES[kind]}"
            )
        if headings:
            shown = heading if heading else f"(empty heading of column {position + 1})"
            source = shown
        else:
            source = f"column {position + 1} (the file has no headings): {heading or name}"
        columns.append(
            Col(
                name,
                kind,
                source,
                unit or NOT_STATED,
                note,
                heading=heading if headings else None,
            )
        )
    return tuple(columns)


@dataclass(frozen=True)
class Delimited:
    """A tab-separated table declared once: the files that hold it and its columns."""

    table: str
    files: tuple[str, ...]
    columns: tuple[Col, ...]
    headings: bool = True
    quoted: bool = False

    def schema(self) -> pa.Schema:
        return schema(*self.columns)


def delimited(
    table: str,
    files: str | Sequence[str],
    spec: str,
    *,
    headings: bool = True,
    quoted: bool = False,
) -> Delimited:
    names = (files,) if isinstance(files, str) else tuple(files)
    return Delimited(table, names, parse_columns(spec, headings=headings), headings, quoted)


def schemas_of(specs: Sequence[Delimited]) -> dict[str, pa.Schema]:
    found: dict[str, pa.Schema] = {}
    for spec in specs:
        if spec.table in found:
            raise StagingError(f"table {spec.table} is declared twice")
        found[spec.table] = spec.schema()
    return found


# -- reading files ---------------------------------------------------------------------------------


def read_text(tree: Path, artifact: str) -> str:
    """The UTF-8 text of a payload file (a `.gz` file decompressed, a `.zip` archive's one
    member extracted)."""
    try:
        raw = (tree / artifact).read_bytes()
        if artifact.endswith(".gz"):
            raw = gzip.decompress(raw)
        elif artifact.endswith(".zip"):
            with zipfile.ZipFile(io.BytesIO(raw)) as archive:
                members = archive.namelist()
                if len(members) != 1:
                    raise StagingError(
                        f"{artifact}: expected one archive member, found {len(members)}"
                    )
                raw = archive.read(members[0])
        return raw.decode("utf-8")
    except (OSError, EOFError, gzip.BadGzipFile, zipfile.BadZipFile, UnicodeDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as UTF-8 text: {error}") from error


def split_lines(content: str) -> list[str]:
    """The lines of a text, split on `\\n` only; a final newline ends the last line."""
    lines = content.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    return lines


def where(artifact: str, line: int) -> str:
    return f"{artifact}#L{line}"


def cells_of(
    artifact: str, lines: Sequence[str], *, quoted: bool = False
) -> Iterator[tuple[int, list[str]]]:
    """`(line number, cells)` for every line; a blank line is an error."""
    if quoted:
        reader = csv.reader(lines, delimiter="\t", quotechar='"', strict=True)
        try:
            for cells in reader:
                if not cells:
                    raise StagingError(f"{where(artifact, reader.line_num)}: blank line")
                yield reader.line_num, cells
        except csv.Error as error:
            raise StagingError(f"{artifact}: malformed quoting: {error}") from error
        return
    for number, line in enumerate(lines, start=1):
        if line == "":
            raise StagingError(f"{where(artifact, number)}: blank line")
        yield number, line.split("\t")


def value_of(kind: str, cell: str, place: str, name: str) -> object:
    """The typed value of a cell; a blank is `None`."""
    if kind == "s":
        return None if cell == "" else cell
    stripped = cell.strip()
    if stripped == "":
        return None
    if kind == "f":
        if _NUMBER.match(cell):
            return float(cell)
        if cell == "nan":
            return math.nan
        raise StagingError(f"{place}, column {name}: {cell!r} is not a decimal number or nan")
    if kind in ("i", "c"):
        if _INTEGER.match(cell):
            number = int(cell)
            if not -(2**63) <= number < 2**63:
                raise StagingError(f"{place}, column {name}: {cell!r} does not fit 64 bits")
            return number
        raise StagingError(f"{place}, column {name}: {cell!r} is not an integer")
    if kind == "b":
        if cell in ("True", "False"):
            return cell == "True"
        raise StagingError(f"{place}, column {name}: {cell!r} is not True or False")
    raise StagingError(f"{place}, column {name}: kind {kind!r} cannot be read from a cell")


def check_headings(artifact: str, spec: Delimited, cells: Sequence[str]) -> None:
    expected = [c.heading or "" for c in spec.columns]
    if list(cells) != expected:
        raise StagingError(
            f"{where(artifact, 1)}: the headings {list(cells)!r} differ from the declared "
            f"headings {expected!r}; extend the reader and bump its READER_VERSION"
        )


def table_rows(artifact: str, spec: Delimited, content: str) -> Iterator[dict[str, object]]:
    """The rows of one file of `spec`."""
    lines = split_lines(content)
    first = True
    width = len(spec.columns)
    for number, cells in cells_of(artifact, lines, quoted=spec.quoted):
        if first and spec.headings:
            first = False
            check_headings(artifact, spec, cells)
            continue
        first = False
        place = where(artifact, number)
        if len(cells) != width:
            raise StagingError(f"{place}: {len(cells)} cells, the table declares {width}")
        row: dict[str, object] = {"_artifact": artifact, "_locator": place}
        for spec_column, cell in zip(spec.columns, cells, strict=True):
            row[spec_column.name] = value_of(spec_column.kind, cell, place, spec_column.name)
        yield row


def read_delimited(tree: Path, data_root: str, spec: Delimited, rel: str, writer: Writer) -> int:
    """Stage one file `data_root/rel` of `spec`; returns the rows written."""
    artifact = f"{data_root}/{rel}"
    content = read_text(tree, artifact)
    count = writer.rows(spec.table, table_rows(artifact, spec, content))
    writer.opened(artifact)
    return count


# -- JSON values -----------------------------------------------------------------------------------


def load_json(tree: Path, artifact: str) -> object:
    try:
        return json.loads((tree / artifact).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise StagingError(f"{artifact}: cannot be read as JSON: {error}") from error


def as_object(value: object, place: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise StagingError(f"{place}: expected a JSON object, found {type(value).__name__}")
    return value


def as_list(value: object, place: str) -> list[object]:
    if not isinstance(value, list):
        raise StagingError(f"{place}: expected a JSON array, found {type(value).__name__}")
    return value


def only_keys(obj: Mapping[str, object], allowed: Sequence[str], place: str) -> None:
    """Refuse a key no column declares, so a new field is a visible failure."""
    extra = sorted(set(obj) - set(allowed))
    if extra:
        raise StagingError(
            f"{place}: keys no column declares: {', '.join(extra)}; extend the reader and bump "
            "its READER_VERSION"
        )


def json_float(value: object, place: str) -> float | None:
    """A JSON number as float64; `null` is `None`."""
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, int | float):
        raise StagingError(f"{place}: expected a number, found {value!r}")
    return float(value)


def json_int(value: object, place: str) -> int | None:
    if value is None:
        return None
    if isinstance(value, bool) or not isinstance(value, int):
        raise StagingError(f"{place}: expected an integer, found {value!r}")
    return value


def json_text(value: object, place: str) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        raise StagingError(f"{place}: expected text, found {value!r}")
    return value


def json_flag(value: object, place: str) -> bool | None:
    if value is None:
        return None
    if not isinstance(value, bool):
        raise StagingError(f"{place}: expected true or false, found {value!r}")
    return value


def json_floats(value: object, place: str) -> list[float | None]:
    return [json_float(item, f"{place}[{i}]") for i, item in enumerate(as_list(value, place))]


def pointer(*tokens: object) -> str:
    """A JSON pointer from its tokens."""
    return "".join(f"/{str(token).replace('~', '~0').replace('/', '~1')}" for token in tokens)


def locator(artifact: str, position: str) -> str:
    return f"{artifact}#{position}"


Handler = Callable[[Path, str, Writer], None]
"""Stages one payload file: `handler(tree, artifact, writer)`."""

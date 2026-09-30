# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reading thermo.inp (NASA TP-2002-211556, Appendix A).

The file is a comment block (lines starting with `!`), the word `thermo`, a grid line
(`4f10.3, a10`), the product records up to a line `END PRODUCTS`, the reactant-only records up
to a line `END REACTANTS`. A record is

- line 1: `a15` name, `a65` notes;
- line 2: `i2` interval count, blank, `a6` reference-date code, blank, five `(a2, f6.2)`
  element-count pairs, `i2` phase flag, `f13.5` molecular weight, `f15.3` heat of formation;
- per interval, three lines: `2f11.3, i1, 8f5.1, 2x, f15.3` (bounds, exponent count, eight
  exponent slots, H(298.15)-H(0)), then nine coefficients in `5d16.8 / 2d16.8, 16x, 2d16.8`;
- a record whose interval count is 0 has one interval-format line and no coefficient lines.
"""

from __future__ import annotations

from collections.abc import Iterator

from thermo_knowledge.readers.nasa_cea.fortran import FieldError, Line
from thermo_knowledge.readers.nasa_cea.rows import Rows
from thermo_knowledge.staging.errors import StagingError

MAX_WIDTH = 80
PAIRS = 5
SLOTS = 8


def locator(line: Line) -> str:
    return f"{line.artifact}#L{line.number}"


class _Cursor:
    def __init__(self, lines: list[Line]) -> None:
        self._lines = lines
        self.index = 0

    def peek(self) -> Line | None:
        return self._lines[self.index] if self.index < len(self._lines) else None

    def take(self, what: str) -> Line:
        line = self.peek()
        if line is None:
            last = self._lines[-1]
            raise StagingError(
                f"{last.artifact}: the file ends after line {last.number}, inside {what}"
            )
        self.index += 1
        return line

    def rest(self) -> Iterator[Line]:
        while self.index < len(self._lines):
            yield self._lines[self.index]
            self.index += 1


def read_thermo(lines: list[Line], rows: Rows) -> None:
    """Emit the rows of `thermo.inp`; every line becomes part of exactly one row."""
    cursor = _Cursor(lines)
    artifact = lines[0].artifact if lines else "thermo.inp"
    while (line := cursor.peek()) is not None:
        cursor.index += 1
        if line.text.startswith("!"):
            rows.frame(line, "comment")
        elif line.text.strip().lower().startswith("thermo"):
            rows.frame(line, "keyword")
            break
        else:
            raise StagingError(
                f"{artifact}, line {line.number}: expected a comment line or the word thermo, "
                f"found {line.text[:40]!r}"
            )
    else:
        raise StagingError(f"{artifact}: no line starting with the word thermo")
    grid = cursor.take("the temperature grid line")
    temperatures = [grid.real(1 + 10 * i, 10 * (i + 1), "f10.3", 3) for i in range(4)]
    rows.frame(grid, "grid", temperatures=temperatures, date=grid.text_field(41, 50))
    section = "products"
    record_index = 0
    while (line := cursor.peek()) is not None:
        if line.text.startswith("END"):
            cursor.index += 1
            rows.frame(line, "sentinel")
            if "ROD" in line.text:
                section = "reactants"
                continue
            break
        cursor.index += 1
        read_record(cursor, rows, line, section, record_index)
        record_index += 1
    else:
        raise StagingError(f"{artifact}: the file ends without an END REACTANTS line")
    for line in cursor.rest():
        rows.frame(line, "trailing")


def check_width(line: Line) -> None:
    if len(line.text) > MAX_WIDTH:
        raise FieldError(
            f"{line.artifact}, line {line.number}: {len(line.text)} characters; "
            f"a record line has at most {MAX_WIDTH}"
        )


def read_record(
    cursor: _Cursor, rows: Rows, first: Line, section: str, record_index: int
) -> None:
    second = cursor.take(f"the record of {first.text[:15].strip()!r} (line {first.number})")
    check_width(first)
    check_width(second)
    interval_count = second.integer(1, 2, "i2")
    if interval_count is None or interval_count < 0:
        raise FieldError(
            f"{second.where(1, 2)}: the interval count must be a non-negative integer, "
            f"found {second.field(1, 2).strip()!r}"
        )
    where = locator(first)
    rows.add(
        "species_records",
        first,
        section=section,
        record_index=record_index,
        name=first.text_field(1, 15),
        notes=first.text_field(16, 80),
        interval_count=interval_count,
        reference_date_code=second.text_field(4, 9),
        phase_flag=second.integer(51, 52, "i2"),
        molecular_weight=second.real(53, 65, "f13.5", 5),
        heat_of_formation=second.real(66, 80, "f15.3", 3),
        line=first.number,
        raw_line_1=first.text,
        raw_line_2=second.text,
    )
    for slot in range(PAIRS):
        start = 11 + 8 * slot
        symbol = second.text_field(start, start + 1)
        count = second.real(start + 2, start + 7, "f6.2", 2)
        if symbol is None and not count:
            continue
        rows.add(
            "species_formula_pairs",
            second,
            suffix=f"/{slot + 1}",
            species_locator=where,
            slot=slot + 1,
            symbol=symbol,
            count=count,
        )
    if interval_count == 0:
        header = cursor.take(f"the reference line of {first.text[:15].strip()!r}")
        check_width(header)
        read_interval(rows, where, 0, header, None, None)
        return
    for interval_index in range(interval_count):
        header = cursor.take(f"interval {interval_index + 1} of {first.text[:15].strip()!r}")
        one = cursor.take(f"interval {interval_index + 1} of {first.text[:15].strip()!r}")
        two = cursor.take(f"interval {interval_index + 1} of {first.text[:15].strip()!r}")
        for line in (header, one, two):
            check_width(line)
        read_interval(rows, where, interval_index, header, one, two)


def read_interval(
    rows: Rows,
    species_locator: str,
    interval_index: int,
    header: Line,
    one: Line | None,
    two: Line | None,
) -> None:
    exponents = [header.real(24 + 5 * i, 28 + 5 * i, "f5.1", 1) for i in range(SLOTS)]
    coefficients: list[float | None] | None = None
    if one is not None and two is not None:
        coefficients = [one.real(1 + 16 * i, 16 * (i + 1), "d16.8", 8) for i in range(5)]
        coefficients += [two.real(1 + 16 * i, 16 * (i + 1), "d16.8", 8) for i in range(2)]
        coefficients += [two.real(49 + 16 * i, 48 + 16 * (i + 1), "d16.8", 8) for i in range(2)]
    rows.add(
        "thermo_intervals",
        header,
        species_locator=species_locator,
        interval_index=interval_index,
        t_low=header.real(1, 11, "f11.3", 3),
        t_high=header.real(12, 22, "f11.3", 3),
        exponent_count=header.integer(23, 23, "i1"),
        exponents=exponents,
        enthalpy_increment=header.real(66, 80, "f15.3", 3),
        coefficients=coefficients,
        raw_line_interval=header.text,
        raw_line_coefficients_1=one.text if one is not None else None,
        raw_line_coefficients_2=two.text if two is not None else None,
    )

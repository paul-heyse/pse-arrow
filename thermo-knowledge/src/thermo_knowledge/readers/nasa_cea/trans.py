# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reading trans.inp.

The file is a title line, entries, and a closing `end` line. An entry is

- a header line: `2a16` names (the second blank for a single species), 2 blanks, `a1` letter V,
  `i1` number of viscosity intervals, `a1` letter C, `i1` number of conductivity intervals, then
  reference text;
- one line per interval, viscosity lines first, then conductivity lines: blank, `a1` property
  letter, `2f9.2` bounds, `4e15.8` fit coefficients.

The exponent of a coefficient may be written with a space instead of a sign (`0.61205763E 00`),
which Fortran reads as a plus.
"""

from __future__ import annotations

from thermo_knowledge.readers.nasa_cea.fortran import FieldError, Line
from thermo_knowledge.readers.nasa_cea.rows import Rows
from thermo_knowledge.readers.nasa_cea.thermo import check_width
from thermo_knowledge.staging.errors import StagingError


def read_trans(lines: list[Line], rows: Rows) -> None:
    artifact = lines[0].artifact if lines else "trans.inp"
    if not lines:
        raise StagingError(f"{artifact}: the file is empty")
    rows.frame(lines[0], "title")
    position = 1
    entry_index = 0
    while position < len(lines):
        header = lines[position]
        position += 1
        if header.text.strip().lower() == "end":
            rows.frame(header, "end")
            for line in lines[position:]:
                rows.frame(line, "trailing")
            return
        check_width(header)
        viscosity_letter = header.field(35, 35)
        conductivity_letter = header.field(37, 37)
        if viscosity_letter != "V" or conductivity_letter != "C":
            raise FieldError(
                f"{header.where(35, 37)}: an entry header has the letters V and C in columns "
                f"35 and 37, found {viscosity_letter!r} and {conductivity_letter!r}"
            )
        viscosity = header.integer(36, 36, "i1")
        conductivity = header.integer(38, 38, "i1")
        if viscosity is None or conductivity is None:
            raise FieldError(f"{header.where(36, 38)}: the interval counts must be given")
        where = f"{artifact}#L{header.number}"
        rows.add(
            "trans_entries",
            header,
            entry_index=entry_index,
            name_1=header.text_field(1, 16),
            name_2=header.text_field(17, 32),
            viscosity_interval_count=viscosity,
            conductivity_interval_count=conductivity,
            reference=header.text_field(39, 80),
            line=header.number,
            raw_line=header.text,
        )
        for letter, count in (("V", viscosity), ("C", conductivity)):
            for interval_index in range(count):
                if position >= len(lines):
                    raise StagingError(
                        f"{artifact}: the file ends after line {lines[-1].number}, inside the "
                        f"entry at line {header.number}"
                    )
                line = lines[position]
                position += 1
                check_width(line)
                stated = line.field(2, 2)
                if stated != letter:
                    raise FieldError(
                        f"{line.where(2, 2)}: expected the property letter {letter} "
                        f"(interval {interval_index + 1} of {count}) for the entry at line "
                        f"{header.number}, found {stated!r}"
                    )
                rows.add(
                    "trans_intervals",
                    line,
                    entry_locator=where,
                    property=letter,
                    interval_index=interval_index,
                    t_low=line.real(3, 11, "f9.2", 2),
                    t_high=line.real(12, 20, "f9.2", 2),
                    coefficients=[
                        line.real(21 + 15 * i, 35 + 15 * i, "e15.8", 8) for i in range(4)
                    ],
                    raw_line=line.text,
                )
        entry_index += 1
    raise StagingError(f"{artifact}: the file ends without the closing end line")

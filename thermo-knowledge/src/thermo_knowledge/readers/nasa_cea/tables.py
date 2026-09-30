# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The declared tables of the NASA CEA reader. Columns are named for the fields of the record
layouts (NASA TP-2002-211556, Appendix A: thermo.inp; the transport layout of trans.inp). The
files state no units, so every quantity column says `not stated`."""

from __future__ import annotations

import pyarrow as pa

from thermo_knowledge.staging.schema import (
    FLOAT64,
    INT64,
    NOT_STATED,
    STRING,
    column,
    table_schema,
)

NOT_APPLICABLE = "not applicable"


def _text(name: str, source_name: str | None = None, note: str | None = None) -> pa.Field:
    return column(name, STRING, source_name=source_name, unit=NOT_APPLICABLE, note=note)


def _real(name: str, source_name: str, unit: str = NOT_STATED, note: str | None = None) -> pa.Field:
    return column(name, FLOAT64, source_name=source_name, unit=unit, note=note)


def _int(name: str, source_name: str, note: str | None = None) -> pa.Field:
    return column(name, INT64, source_name=source_name, unit=NOT_APPLICABLE, note=note)


def _reals(name: str, source_name: str, note: str | None = None) -> pa.Field:
    return column(name, pa.list_(FLOAT64), source_name=source_name, unit=NOT_STATED, note=note)


LINE = _int("line", "line number of the record's first line")
RAW = "the line exactly as written, without the line ending (the files use CRLF)"
SPECIES_LOCATOR = _text("species_locator", "_locator of the species_records row")
ENTRY_LOCATOR = _text("entry_locator", "_locator of the trans_entries row")

TABLES: dict[str, pa.Schema] = {
    "frame_lines": table_schema(
        _text(
            "kind",
            "role of the line",
            "comment (a line starting with !), keyword (the word thermo), grid (the temperature "
            "grid line after it), sentinel (an END line), title (first line of trans.inp), end "
            "(the closing line of trans.inp), trailing (anything after the last sentinel)",
        ),
        _text("text", "line content", RAW),
        _reals(
            "temperatures",
            "4f10.3",
            "the four temperatures of the grid line; set only for kind grid",
        ),
        _text("date", "a10", "the date of the grid line; set only for kind grid"),
    ),
    "species_records": table_schema(
        _text(
            "section",
            "position in the file",
            "products (before END PRODUCTS) or reactants (before END REACTANTS)",
        ),
        _int("record_index", "order of the record in the file (0-based)"),
        _text("name", "a15, line 1 columns 1-15", "surrounding blanks removed"),
        _text("notes", "a65, line 1 columns 16-80", "surrounding blanks removed; null if blank"),
        _int("interval_count", "i2, line 2 columns 1-2", "0 marks a record without coefficients"),
        _text(
            "reference_date_code",
            "a6, line 2 columns 4-9",
            "surrounding blanks removed; null if blank",
        ),
        _int("phase_flag", "i2, line 2 columns 51-52"),
        _real("molecular_weight", "f13.5, line 2 columns 53-65"),
        _real("heat_of_formation", "f15.3, line 2 columns 66-80"),
        LINE,
        _text("raw_line_1", "species line 1", RAW),
        _text("raw_line_2", "species line 2", RAW),
    ),
    "species_formula_pairs": table_schema(
        SPECIES_LOCATOR,
        _int("slot", "position of the (element, count) pair in the formula, 1 to 5"),
        _text(
            "symbol",
            "a2",
            "element symbol, surrounding blanks removed; null when the symbol field is blank",
        ),
        _real(
            "count",
            "f6.2",
            note="null when the count field is blank; a written 0.00 is 0.0. A slot whose symbol "
            "is blank and whose count is blank or zero is padding and is not emitted",
        ),
    ),
    "thermo_intervals": table_schema(
        SPECIES_LOCATOR,
        _int("interval_index", "order of the interval within its record (0-based)"),
        _real("t_low", "f11.3, interval line columns 1-11"),
        _real("t_high", "f11.3, interval line columns 12-22"),
        _int(
            "exponent_count",
            "i1, interval line column 23",
            "number of exponents given (the source states 7)",
        ),
        _reals(
            "exponents",
            "8f5.1, interval line columns 24-63",
            "the eight exponent slots as written; null where a slot is blank",
        ),
        _real(
            "enthalpy_increment",
            "f15.3, interval line columns 66-80",
            note="the H(298.15)-H(0) column of the interval line",
        ),
        _reals(
            "coefficients",
            "5d16.8/2d16.8,16x,2d16.8",
            "nine values in file order: the seven polynomial coefficients, then the two "
            "integration constants; null for a record with interval_count 0, whose single "
            "interval-format line states a reference temperature in t_low and has no "
            "coefficient lines",
        ),
        _text("raw_line_interval", "interval line", RAW),
        _text("raw_line_coefficients_1", "first coefficient line", RAW),
        _text("raw_line_coefficients_2", "second coefficient line", RAW),
    ),
    "trans_entries": table_schema(
        _int("entry_index", "order of the entry in the file (0-based)"),
        _text("name_1", "a16, columns 1-16", "surrounding blanks removed"),
        _text(
            "name_2",
            "a16, columns 17-32",
            "surrounding blanks removed; null when blank (a single-species entry)",
        ),
        _int("viscosity_interval_count", "i1, column 36, after the letter V in column 35"),
        _int("conductivity_interval_count", "i1, column 38, after the letter C in column 37"),
        _text("reference", "columns 39-80", "surrounding blanks removed; null if blank"),
        LINE,
        _text("raw_line", "entry header line", RAW),
    ),
    "trans_intervals": table_schema(
        ENTRY_LOCATOR,
        _text("property", "a1, column 2", "V (viscosity) or C (conductivity), as written"),
        _int("interval_index", "order of the interval among those of its property (0-based)"),
        _real("t_low", "f9.2, columns 3-11"),
        _real("t_high", "f9.2, columns 12-20"),
        _reals(
            "coefficients",
            "4e15.8, columns 21-80",
            "the four fit coefficients in file order (A, B, C, D); null where a field is blank",
        ),
        _text("raw_line", "interval line", RAW),
    ),
}

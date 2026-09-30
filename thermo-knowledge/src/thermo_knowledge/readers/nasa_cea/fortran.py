# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Fixed-column fields of NASA's Fortran text formats.

A field is a slice of a line at fixed columns (1-based, inclusive). The numeric fields follow the
Fortran formatted-read rules that the formats rely on:

- blanks inside a numeric field are ignored (`0.61205763E 00` reads as `0.61205763E00`), so the
  space that stands for a plus sign in an exponent is exact;
- the exponent letter may be `E` or `D` (either case); an exponent may also be written as a bare
  signed number after the mantissa (`1.5-03`);
- a field that is entirely blank reads as no value (`None`), not as zero, so a missing value
  stays distinct from a written `0.0`;
- a field without a decimal point carries an implied one `d` digits from the right of the
  mantissa, where `d` is the format's decimal count.

An unreadable field raises `FieldError` naming the line, the columns and the format.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from thermo_knowledge.staging.errors import StagingError

_MANTISSA = re.compile(r"^([+-]?)(\d*)(\.?)(\d*)$")
_EXPONENT_LETTER = re.compile(r"^(.*?)[EeDd]([+-]?\d+)$")
_EXPONENT_SIGN = re.compile(r"^([+-]?[\d.]+)([+-]\d+)$")
_INTEGER = re.compile(r"^[+-]?\d+$")


class FieldError(StagingError):
    """A fixed-column field that cannot be read as its format says."""


@dataclass(frozen=True)
class Line:
    """One line of a payload file: its text (without the line ending) and 1-based number."""

    artifact: str
    number: int
    text: str

    def where(self, first: int, last: int) -> str:
        return f"{self.artifact}, line {self.number}, columns {first}-{last}"

    def field(self, first: int, last: int) -> str:
        """Columns `first` to `last` (1-based, inclusive); a short line is padded with blanks."""
        return self.text[first - 1 : last].ljust(last - first + 1)

    def text_field(self, first: int, last: int) -> str | None:
        """A character field with surrounding blanks removed; `None` when it is all blank."""
        value = self.field(first, last).strip()
        return value or None

    def integer(self, first: int, last: int, width: str) -> int | None:
        raw = self.field(first, last).replace(" ", "")
        if not raw:
            return None
        if not _INTEGER.match(raw):
            raise FieldError(f"{self.where(first, last)}: cannot read {raw!r} as {width}")
        return int(raw)

    def real(self, first: int, last: int, width: str, decimals: int) -> float | None:
        raw = self.field(first, last)
        try:
            return read_real(raw, decimals)
        except ValueError:
            raise FieldError(
                f"{self.where(first, last)}: cannot read {raw.strip()!r} as {width}"
            ) from None


def read_real(field: str, decimals: int) -> float | None:
    """The value of a Fortran real field, or `None` for a blank one; `ValueError` otherwise."""
    text = "".join(field.split())
    if not text:
        return None
    exponent = 0
    match = _EXPONENT_LETTER.match(text)
    if match:
        text, exponent = match.group(1), int(match.group(2))
    else:
        signed = _EXPONENT_SIGN.match(text)
        if signed and re.search(r"\d", signed.group(1)):
            text, exponent = signed.group(1), int(signed.group(2))
    parts = _MANTISSA.match(text)
    if parts is None or not (parts.group(2) or parts.group(4)):
        raise ValueError(field)
    sign, whole, point, fraction = parts.groups()
    if point:
        mantissa = f"{sign}{whole or '0'}.{fraction or '0'}"
    else:
        digits = whole.rjust(decimals + 1, "0")
        mantissa = f"{sign}{digits[:-decimals] if decimals else digits}" + (
            f".{digits[-decimals:]}" if decimals else ""
        )
    return float(f"{mantissa}e{exponent}")

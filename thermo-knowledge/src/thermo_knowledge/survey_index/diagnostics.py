# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""One deviation of a survey record or a disposition from the specification."""

from __future__ import annotations

from enum import StrEnum

import msgspec


class Code(StrEnum):
    """Stable kinds of deviation; the tooling counts and filters by them."""

    TOML_SYNTAX = "toml-syntax"
    UNKNOWN_KEY = "unknown-key"
    MISSING_KEY = "missing-key"
    WRONG_TYPE = "wrong-type"
    BAD_VALUE = "bad-value"
    NOT_A_TABLE = "not-a-table"
    SOURCE_MISMATCH = "source-mismatch"
    DUPLICATE_NAME = "duplicate-name"
    # Dispositions.
    UNKNOWN_CONSTRUCT = "unknown-construct"
    DUPLICATE_DISPOSITION = "duplicate-disposition"
    NOT_NEEDED = "disposition-not-needed"
    MISSING_REF = "missing-ref"
    BAD_REF = "bad-ref"
    UNKNOWN_REF_TARGET = "unknown-ref-target"
    UNKNOWN_SOURCE = "unknown-source"
    # Inputs.
    DECLARATION_UNAVAILABLE = "declaration-unavailable"
    INPUT_UNAVAILABLE = "input-unavailable"


class SurveyDiagnostic(msgspec.Struct, frozen=True, kw_only=True):
    """Where a deviation is (file, table, record, key), its kind and what is wrong.

    `file` is relative to the tree; `table` is the record table (`construct`, `disposition`, ...)
    or empty for the file itself; `record` is the record's `name` (or its position when it has
    none); `key` is the key concerned, with nested positions (`fields[2].role`).
    """

    file: str
    table: str = ""
    record: str = ""
    key: str = ""
    code: Code
    message: str

    def __str__(self) -> str:
        where = [self.file]
        if self.table:
            where.append(f"[[{self.table}]]")
        if self.record:
            where.append(repr(self.record))
        if self.key:
            where.append(self.key)
        return f"{' '.join(where)}: [{self.code}] {self.message}"

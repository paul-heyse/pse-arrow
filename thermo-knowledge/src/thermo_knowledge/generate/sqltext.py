# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The one place the generated DDL quotes identifiers and literals.

Quoting is delegated to `psycopg.sql`, which needs no connection: an identifier is always
double-quoted with embedded quotes doubled, a string literal is single-quoted with embedded
quotes doubled (and written `E'...'` when it contains a backslash). The generated file is
applied with `standard_conforming_strings` on, PostgreSQL's default.
"""

from __future__ import annotations

from datetime import date, datetime

from psycopg import sql

MAX_IDENTIFIER_BYTES = 63
"""PostgreSQL silently truncates longer identifiers; the generator refuses them instead."""


def ident(name: str) -> str:
    """A double-quoted identifier."""
    return sql.Identifier(name).as_string()


def qname(schema: str, name: str) -> str:
    """A schema-qualified, quoted name."""
    return f"{ident(schema)}.{ident(name)}"


def literal(value: str | int | float | bool | date | datetime | bytes) -> str:
    """A SQL literal for a Python value.

    Strings and dates are returned as written by psycopg; the leading space psycopg puts
    before an `E'...'` string is removed.
    """
    return sql.Literal(value).as_string().strip()


def columns(names: list[str] | tuple[str, ...]) -> str:
    """A comma-separated, quoted column list."""
    return ", ".join(ident(name) for name in names)

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Parameter declarations of the older property packages: `self.<name> = Param(...)` and
`self.<name> = Var(...)` statements of the data-definition modules.

Each declaration becomes a row with the expressions of its `units`, `initialize`, `default`,
`value`, `bounds`, `within` and `mutable` keywords (rendered from the syntax tree) and its
positional arguments (the index sets). The `initialize` keyword usually names a local dictionary
of literal values (for example through a helper call); `initialize_name` is that dictionary's
name, which is the `target` of the matching rows of `literal_entries` in the same scope. Text
keywords such as `doc` are not read.
"""

from __future__ import annotations

import ast

import pyarrow as pa

from thermo_knowledge.readers.idaes.literals import (
    NOT_APPLICABLE,
    NOT_STATED,
    fold,
)
from thermo_knowledge.staging.schema import FLOAT64, INT64, STRING, column, table_schema

CONSTRUCTORS = ("Param", "Var")
KEYWORDS = ("units", "initialize", "default", "value", "bounds", "within", "mutable")


def _text(
    name: str, source_name: str, *, nullable: bool = True, note: str | None = None
) -> pa.Field:
    return column(
        name, STRING, source_name=source_name, unit=NOT_APPLICABLE, note=note, nullable=nullable
    )


PARAMETER_DECLARATIONS = table_schema(
    column("line", INT64, source_name="line of the statement", unit=NOT_APPLICABLE, nullable=False),
    column(
        "column",
        INT64,
        source_name="column of the statement (0-based)",
        unit=NOT_APPLICABLE,
        nullable=False,
    ),
    _text("scope_kind", "module, class or function", nullable=False),
    _text("scope", "dotted names of the enclosing classes and functions", nullable=False),
    _text("target", "the assignment target", nullable=False),
    _text("name", "the attribute name the declaration is assigned to", nullable=False),
    _text("constructor", "Param or Var", nullable=False),
    column(
        "index_sets",
        pa.list_(STRING),
        source_name="positional arguments of the call",
        unit=NOT_APPLICABLE,
        note="rendered from the syntax tree",
    ),
    _text("units_text", "units keyword", note="the unit expression as the source writes it"),
    _text("initialize_text", "initialize keyword"),
    _text(
        "initialize_name",
        "the name inside the initialize keyword",
        note="the local dictionary the initial values come from, when the keyword names one",
    ),
    _text("default_text", "default keyword"),
    column(
        "default_number",
        FLOAT64,
        source_name="default keyword as a number",
        unit=NOT_STATED,
        note="the unit is in units_text",
    ),
    _text("value_text", "value keyword"),
    _text("bounds_text", "bounds keyword"),
    _text("within_text", "within keyword"),
    _text("mutable_text", "mutable keyword"),
)

SCHEMAS = {"parameter_declarations": PARAMETER_DECLARATIONS}


def _first_name(node: ast.expr) -> str | None:
    """The identifier a value expression stands on: itself, or the first name argument of a
    call."""
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Call):
        for argument in node.args:
            if isinstance(argument, ast.Name):
                return argument.id
    return None


def declaration_row(
    artifact: str, statement: ast.stmt, scope_kind: str, scope: str
) -> dict[str, object] | None:
    if not isinstance(statement, ast.Assign) or len(statement.targets) != 1:
        return None
    call = statement.value
    if not (
        isinstance(call, ast.Call)
        and isinstance(call.func, ast.Name)
        and call.func.id in CONSTRUCTORS
    ):
        return None
    target = statement.targets[0]
    name = target.attr if isinstance(target, ast.Attribute) else ast.unparse(target)
    keywords = {kw.arg: kw.value for kw in call.keywords if kw.arg in KEYWORDS}
    default = keywords.get("default")
    initialize = keywords.get("initialize")
    row: dict[str, object] = {
        "_artifact": artifact,
        "_locator": f"{artifact}#L{statement.lineno}:{statement.col_offset}",
        "line": statement.lineno,
        "column": statement.col_offset,
        "scope_kind": scope_kind,
        "scope": scope,
        "target": ast.unparse(target),
        "name": name,
        "constructor": call.func.id,
        "index_sets": [ast.unparse(argument) for argument in call.args],
        "initialize_name": None if initialize is None else _first_name(initialize),
        "default_number": None if default is None else fold(default),
    }
    for keyword in KEYWORDS:
        value = keywords.get(keyword)
        row[f"{keyword}_text"] = None if value is None else ast.unparse(value)
    return row

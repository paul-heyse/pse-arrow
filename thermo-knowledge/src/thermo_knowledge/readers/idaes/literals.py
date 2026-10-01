# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Literal parameter data in Python assignments: one row per leaf value.

Only the values are read. A leaf's text is the value as the source spells a plain number, the
string itself for a string and the syntax tree's rendering (`ast.unparse`) of any other
expression; nothing else of the modules is kept, so no code, docstring or comment
of the source appears in a row.

An assignment is taken when its right-hand side is a literal structure: a dict, list or tuple
(nested to any depth) with at least one numeric leaf; its other leaves are strings, names,
attribute chains such as unit expressions, or any other expression, kept as text. A bare number on the right-hand side is taken only in the data-definition
modules (`DATA_MODULES`), where every numeric assignment in any scope is a parameter value; in
every other module only literal containers are taken, at module, class and function level, so
the small integer codes of enumerations and solver constants are not read.

The tree of a container is flattened into one row per leaf: `path` is the list of keys and
positions from the assigned target to the leaf (a string key is the string itself, any other
key its rendering, a position `[i]`), `kind` says what the leaf is, and `number` holds its value
when it is a number or arithmetic on numbers only (folded exactly as Python would).

For configuration dictionaries, whose `parameter_data` entries carry the component and reaction
parameters, a leaf below a `parameter_data` key also has `parameter_scope` (the path before that
key), `parameter` (the key after it) and `parameter_index` (the rest of the path: a coefficient
letter, a tuple of species, or `[0]`/`[1]` for the value and the unit of a `(value, unit)` pair).
"""

from __future__ import annotations

import ast
import operator
from collections.abc import Callable, Iterator

import pyarrow as pa

from thermo_knowledge.staging.schema import FLOAT64, INT64, STRING, column, table_schema

NOT_APPLICABLE = "not applicable"
NOT_STATED = "not stated"

DATA_MODULES = (
    "idaes/models/properties/activity_coeff_models/",
    "idaes/models/properties/examples/",
    "idaes/models/properties/general_helmholtz/components/parameters/",
    "idaes/models/properties/modular_properties/examples/",
)
"""Modules that define parameter data rather than compute with it."""

KINDS = ("number", "string", "boolean", "none", "name", "expression", "empty")


def _text(
    name: str, source_name: str, *, nullable: bool = True, note: str | None = None
) -> pa.Field:
    return column(
        name, STRING, source_name=source_name, unit=NOT_APPLICABLE, note=note, nullable=nullable
    )


def _int(name: str, source_name: str, *, nullable: bool = False) -> pa.Field:
    return column(name, INT64, source_name=source_name, unit=NOT_APPLICABLE, nullable=nullable)


LITERAL_ENTRIES = table_schema(
    _int("line", "line of the leaf value"),
    _int("column", "column of the leaf value (0-based)"),
    _int("assignment_line", "line of the assignment statement"),
    _int("assignment_column", "column of the assignment statement (0-based)"),
    _text("scope_kind", "module, class or function", nullable=False),
    _text("scope", "dotted names of the enclosing classes and functions (empty at module level)", nullable=False),
    _text("target", "the assignment target", nullable=False, note="rendered from the syntax tree"),
    column(
        "path",
        pa.list_(STRING),
        source_name="keys and positions from the target to the leaf",
        unit=NOT_APPLICABLE,
        note="a string key is the string itself; another key is its rendering; a position is [i]",
    ),
    _text("path_text", "path joined with /", nullable=False),
    _text("kind", "number, string, boolean, none, name, expression or empty", nullable=False),
    column(
        "number",
        FLOAT64,
        source_name="the leaf as a number",
        unit=NOT_STATED,
        note=(
            "set for kind number: a numeric literal or arithmetic on numeric literals only; "
            "the unit, where the source gives one, is a separate leaf of a (value, unit) pair"
        ),
    ),
    _text(
        "text",
        "the leaf expression",
        nullable=False,
        note=(
            "a string leaf is the string itself; a plain number is as the source spells it; "
            "any other leaf is rendered from the syntax tree"
        ),
    ),
    _text("parameter_scope", "path before a parameter_data key", note="only below parameter_data"),
    _text("parameter", "the key right after parameter_data", note="only below parameter_data"),
    _text("parameter_index", "the path after the parameter key", note="only below parameter_data"),
)

SCHEMAS = {"literal_entries": LITERAL_ENTRIES}

_BINARY: dict[type[ast.operator], Callable[[float, float], float | complex]] = {
    ast.Add: operator.add,
    ast.Sub: operator.sub,
    ast.Mult: operator.mul,
    ast.Div: operator.truediv,
    ast.FloorDiv: operator.floordiv,
    ast.Mod: operator.mod,
    ast.Pow: operator.pow,
}


def fold(node: ast.expr) -> float | None:
    """The value of a numeric literal or of arithmetic on numeric literals, else None."""
    if isinstance(node, ast.Constant):
        value = node.value
        if isinstance(value, bool) or not isinstance(value, int | float):
            return None
        return float(value)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub | ast.UAdd):
        inner = fold(node.operand)
        if inner is None:
            return None
        return -inner if isinstance(node.op, ast.USub) else inner
    if isinstance(node, ast.BinOp) and type(node.op) in _BINARY:
        left, right = fold(node.left), fold(node.right)
        if left is None or right is None:
            return None
        try:
            result = _BINARY[type(node.op)](left, right)
        except (ArithmeticError, ValueError):
            return None
        return float(result) if isinstance(result, int | float) else None
    return None


def is_literal_container(node: ast.expr) -> bool:
    """A dict, list or tuple with at least one numeric leaf (a number or arithmetic on numbers)."""
    if not isinstance(node, ast.Dict | ast.List | ast.Tuple):
        return False
    return any(fold(leaf) is not None for _, leaf in leaves(node, ()))


def key_text(key: ast.expr | None) -> str:
    if key is None:
        return "**"
    if isinstance(key, ast.Constant) and isinstance(key.value, str):
        return key.value
    return ast.unparse(key)


def leaves(node: ast.expr, path: tuple[str, ...]) -> Iterator[tuple[tuple[str, ...], ast.expr]]:
    """The leaves of a literal container in source order with their paths; an empty container
    is a leaf of its own."""
    if isinstance(node, ast.Dict):
        if not node.keys:
            yield path, node
        for key, value in zip(node.keys, node.values, strict=True):
            yield from leaves(value, (*path, key_text(key)))
    elif isinstance(node, ast.List | ast.Tuple):
        if not node.elts:
            yield path, node
        for index, item in enumerate(node.elts):
            yield from leaves(item, (*path, f"[{index}]"))
    else:
        yield path, node


def _plain_number(node: ast.expr) -> bool:
    """A numeric literal, optionally signed: its source text holds no comment."""
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub | ast.UAdd):
        return _plain_number(node.operand)
    return (
        isinstance(node, ast.Constant)
        and isinstance(node.value, int | float)
        and not isinstance(node.value, bool)
    )


def number_text(node: ast.expr, source: str) -> str:
    """A number as the source spells it (`48.9e5`); arithmetic, which may span lines, is
    rendered from the syntax tree."""
    if _plain_number(node):
        segment = ast.get_source_segment(source, node)
        if segment is not None and "\n" not in segment and "#" not in segment:
            return segment
    return ast.unparse(node)


def leaf_fields(node: ast.expr, source: str) -> tuple[str, float | None, str]:
    """`(kind, number, text)` of a leaf."""
    if isinstance(node, ast.Dict | ast.List | ast.Tuple):
        return "empty", None, ast.unparse(node)
    number = fold(node)
    if number is not None:
        return "number", number, number_text(node, source)
    if isinstance(node, ast.Constant):
        value = node.value
        if isinstance(value, bool):
            return "boolean", None, str(value)
        if value is None:
            return "none", None, "None"
        if isinstance(value, str):
            return "string", None, value
    if isinstance(node, ast.Name | ast.Attribute):
        return "name", None, ast.unparse(node)
    return "expression", None, ast.unparse(node)


def parameter_fields(path: tuple[str, ...]) -> tuple[str | None, str | None, str | None]:
    """The `parameter_data` decomposition of a path."""
    positions = [index for index, step in enumerate(path) if step == "parameter_data"]
    if not positions:
        return None, None, None
    at = positions[-1]
    scope = "/".join(path[:at])
    parameter = path[at + 1] if at + 1 < len(path) else None
    rest = "/".join(path[at + 2 :]) or None
    return scope, parameter, rest


def literal_rows(
    artifact: str, assignment: ast.stmt, scope_kind: str, scope: str, data: bool, source: str
) -> Iterator[dict[str, object]]:
    """The rows of one assignment statement (none when it is not literal parameter data)."""
    if isinstance(assignment, ast.Assign):
        value, targets = assignment.value, assignment.targets
    elif isinstance(assignment, ast.AnnAssign) and assignment.value is not None:
        value, targets = assignment.value, [assignment.target]
    else:
        return
    container = isinstance(value, ast.Dict | ast.List | ast.Tuple)
    if container:
        if not is_literal_container(value):
            return
    elif not (data and fold(value) is not None):
        return
    target = " = ".join(ast.unparse(item) for item in targets)
    for path, leaf in leaves(value, ()):
        kind, number, text = leaf_fields(leaf, source)
        parameter_scope, parameter, parameter_index = parameter_fields(path)
        yield {
            "_artifact": artifact,
            "_locator": f"{artifact}#L{leaf.lineno}:{leaf.col_offset}",
            "line": leaf.lineno,
            "column": leaf.col_offset,
            "assignment_line": assignment.lineno,
            "assignment_column": assignment.col_offset,
            "scope_kind": scope_kind,
            "scope": scope,
            "target": target,
            "path": list(path),
            "path_text": "/".join(path),
            "kind": kind,
            "number": number,
            "text": text,
            "parameter_scope": parameter_scope,
            "parameter": parameter,
            "parameter_index": parameter_index,
        }

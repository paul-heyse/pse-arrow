# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The canonical serialisation of an expression tree and its content hash.

The serialisation is expression text in the grammar itself, with one spelling for each tree:
single spaces around binary operators, `, ` between arguments, parentheses only where the tree
needs them and floats as `repr`. Parsing it gives back an equal tree, and two texts that differ
only in whitespace, comments or redundant parentheses serialise identically. The content hash is
the SHA-256 of that text.
"""

from __future__ import annotations

import hashlib
import json

from thermo_knowledge.declaration import model as m
from thermo_knowledge.expression import tree as t
from thermo_knowledge.expression.parser import parse, parse_residual

# Binding strength, weakest first: conditional, or, and, not, comparison, sum, product,
# unary sign, power, and the atoms (names, calls, numbers).
_TERNARY, _OR, _AND, _NOT, _COMPARE, _SUM, _PRODUCT, _UNARY, _POWER, _ATOM = range(1, 11)


def _strength(node: t.Expr) -> int:
    match node:
        case t.IfExp():
            return _TERNARY
        case t.BoolOp():
            return _OR if node.op == "or" else _AND
        case t.UnaryOp():
            return _NOT if node.op == "not" else _UNARY
        case t.Compare():
            return _COMPARE
        case t.BinOp():
            return {"+": _SUM, "-": _SUM, "*": _PRODUCT, "/": _PRODUCT, "**": _POWER}[node.op]
        case t.Num():
            return _UNARY if node.value < 0 else _ATOM
        case _:
            return _ATOM


def _at(node: t.Expr, least: int) -> str:
    text = _emit(node)
    return f"({text})" if _strength(node) < least else text


def _emit(node: t.Expr) -> str:
    match node:
        case t.Num():
            return str(node.value) if isinstance(node.value, int) else repr(node.value)
        case t.UnitLiteral():
            return f"unit({node.text!r})"
        case t.Name():
            return node.id
        case t.Attribute():
            return f"{_at(node.value, _ATOM)}.{node.attr}"
        case t.Subscript():
            return f"{_at(node.base, _ATOM)}[{', '.join(_emit(i) for i in node.indices)}]"
        case t.Call():
            arguments = [_emit(a) for a in node.args]
            arguments += [f"{k.name}={_emit(k.value)}" for k in node.keywords]
            return f"{_at(node.func, _ATOM)}({', '.join(arguments)})"
        case t.Func():
            return f"{node.name}({', '.join(_emit(a) for a in node.args)})"
        case t.BinOp():
            strength = _strength(node)
            if node.op == "**":
                return f"{_at(node.left, _ATOM)} ** {_at(node.right, _UNARY)}"
            # Left-associative: an equal-strength right operand needs parentheses.
            return f"{_at(node.left, strength)} {node.op} {_at(node.right, strength + 1)}"
        case t.UnaryOp():
            if node.op == "not":
                return f"not {_at(node.operand, _NOT)}"
            return f"{node.op}{_at(node.operand, _UNARY)}"
        case t.Compare():
            return f"{_at(node.left, _SUM)} {node.op} {_at(node.right, _SUM)}"
        case t.BoolOp():
            strength = _strength(node)
            return f" {node.op} ".join(_at(v, strength + 1) for v in node.values)
        case t.IfExp():
            return (
                f"{_at(node.body, _OR)} if {_at(node.test, _OR)} else {_at(node.orelse, _TERNARY)}"
            )
        case t.Range():
            return f"range({_emit(node.start)}, {_emit(node.stop)})"
        case t.Reduce():
            clauses = " ".join(f"for {c.target} in {_at(c.iterable, _OR)}" for c in node.clauses)
            return f"{node.kind}({_emit(node.body)} {clauses})"
        case t.Comprehension():
            if node.basis is not None:
                return f"basis({node.basis!r}, [{_body(node)}])"
            return f"[{_body(node)}]"
        case t.Derivative():
            return f"d({_emit(node.expr)}, {_emit(node.wrt)})"
        case t.At():
            return f"at({_emit(node.family)}, {_emit(node.value)})"
        case t.Position():
            return f"position({_emit(node.array)}, {_emit(node.member)})"
        case t.Integral():
            return f"integral({_emit(node.body)}, {node.var}, {_emit(node.lower)}, {_emit(node.upper)})"
    raise TypeError(f"not an expression node: {node!r}")


def _body(node: t.Comprehension) -> str:
    clauses = " ".join(f"for {c.target} in {_at(c.iterable, _OR)}" for c in node.clauses)
    return f"{_emit(node.body)} {clauses}"


def serialise(node: t.Expr) -> str:
    """The canonical text of `node`."""
    return _emit(node)


def serialise_residual(node: t.Expr) -> str:
    """The canonical text of a residual: a comprehension is written without its brackets."""
    return _body(node) if isinstance(node, t.Comprehension) else _emit(node)


def content_hash(text: str) -> str:
    """The hex SHA-256 of the canonical serialisation of the tree of `text`; raises
    `ExpressionError` when `text` is not in the grammar."""
    return hashlib.sha256(serialise(parse(text)).encode("utf-8")).hexdigest()


def residual_hash(text: str) -> str:
    """The hex SHA-256 of the canonical serialisation of the residual `text`; raises
    `ExpressionError` when it is not in the grammar."""
    return hashlib.sha256(serialise_residual(parse_residual(text)).encode("utf-8")).hexdigest()


def evaluation_hash(form: m.Form, output: str) -> str:
    """The hex SHA-256 of what evaluating `output` of `form` reads from the form's text: its
    locals, the expression of `output` and its implicit blocks (residuals, selection rule and
    unknowns with their bounds), each expression in canonical serialisation, and the slot group
    that names the convention set of each component for a fact read per component. Editing any of them
    changes it; the forms chosen for sub-form slots are other forms, with hashes of their own."""
    parts: list[object] = [["local", e.name, serialise(parse(e.text))] for e in form.locals]
    parts += [
        ["output", e.name, serialise(parse(e.text))] for e in form.outputs if e.name == output
    ]
    parts += [
        ["component_convention", c.name, c.group, c.over]
        for c in form.conventions
        if c.group is not None
    ]
    for block in form.implicit:
        parts.append(
            [
                "implicit",
                block.name,
                [serialise_residual(parse_residual(r.text)) for r in block.residuals],
                block.select,
                block.select_by,
                [
                    [u.name, u.over, _bound(u.lower), _bound(u.upper), _bound(u.start)]
                    for u in block.unknowns
                ],
            ]
        )
    text = json.dumps(parts, sort_keys=True, separators=(",", ":"), default=str)
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _bound(bound: m.Bound | None) -> object:
    return None if bound is None else [bound.number, bound.text]

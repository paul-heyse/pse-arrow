# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Conditions stay boolean structure (expressions.md section 5).

A conditional expression is a `Piecewise`. A comparison of one with something else, the test
`r == 5` of a local `r` that holds a region, is not a condition SymPy can carry: a `Piecewise`
inside the condition of another is rewritten into `ITE` terms when the outer one is built, and
the NumPy printer then runs `simplify_logic` on every condition that contains one, which cancels
and expands each relation in it, the saturation equation of a regional formulation among them.
That simplification is algebra on the formula, it does not finish on an expression of that size,
and the printed condition is not one the evaluator wrote.

Here the comparison is stated as the predicate that selects a branch: `Piecewise` branch `k`
holds where its own condition holds and every earlier one fails, so `r == 5` is the disjunction,
over the branches, of that selection and the comparison of the branch's value with 5. Only the
conditions are restructured. No arithmetic is touched: each branch value is compared as it
stands, and the relation is evaluated by NumPy on the same operands as before.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from typing import cast

import sympy
from sympy.core.relational import Relational
from sympy.functions.elementary.piecewise import ExprCondPair


def lower_relation(
    relation: type[Relational], left: sympy.Basic, right: sympy.Basic
) -> sympy.Basic:
    """The relation `left <op> right` as a condition over relations of non-conditional
    operands, for operands that are conditional expressions or hold them."""
    if isinstance(left, sympy.Piecewise):
        return _select(left, lambda value: lower_relation(relation, value, right))
    if isinstance(right, sympy.Piecewise):
        return _select(right, lambda value: lower_relation(relation, left, value))
    if left is sympy.nan or right is sympy.nan:
        # NumPy: every comparison with a NaN is false, except "differs"
        return sympy.true if relation is sympy.Ne else sympy.false
    return relation(left, right)


def _select(pieces: sympy.Piecewise, test: Callable[[sympy.Basic], sympy.Basic]) -> sympy.Basic:
    """`test` of the value of `pieces`, as the disjunction over its branches of: the branch is
    the one selected, and `test` holds of its value."""
    terms: list[sympy.Basic] = []
    failed: list[sympy.Basic] = []
    pairs = [pair for pair in pieces.args if isinstance(pair, ExprCondPair)]
    for value, condition in pairs:
        terms.append(sympy.And(*failed, condition, test(value)))
        failed.append(sympy.Not(condition))
    if pairs[-1].cond is not sympy.true:
        terms.append(sympy.And(*failed, test(sympy.nan)))  # no branch holds: NumPy gives NaN
    return sympy.Or(*terms)


def substitute[E: sympy.Basic](expr: E, mapping: Mapping[sympy.Basic, sympy.Basic]) -> E:
    """`expr.xreplace(mapping)`, except that a relation one of whose operands becomes a
    conditional expression is lowered (`lower_relation`) in place of being rebuilt around it."""
    if not mapping:
        return expr
    done: dict[sympy.Basic, sympy.Basic] = {}

    def walk(node: sympy.Basic) -> sympy.Basic:
        if node in mapping:
            return mapping[node]
        if not node.args:
            return node
        found = done.get(node)
        if found is not None:
            return found
        args = tuple(walk(a) for a in node.args)
        if all(new is old for new, old in zip(args, node.args, strict=True)):
            result = node
        elif isinstance(node, Relational) and any(isinstance(a, sympy.Piecewise) for a in args):
            result = lower_relation(type(node), *args)
        else:
            result = node.func(*args)
        done[node] = result
        return result

    return cast(E, walk(expr))

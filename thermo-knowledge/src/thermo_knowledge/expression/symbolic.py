# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The SymPy operations the expression machinery uses, with the types its stubs withhold.

SymPy's operators on `Expr` (`+`, `-`, `*`, `/`) are declared through a decorator the type
checker cannot call, its function classes are declared to build an `UndefinedFunction`, and
`free_symbols` is declared as a set of `Basic`. Each helper here does what the operator or
call does (the operators are `Add`, `Mul` and `Pow` of their operands) and states the type of
the result.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import cast

import sympy


def free_symbols(expr: sympy.Basic) -> set[sympy.Symbol]:
    """The symbols `expr` contains."""
    return {s for s in expr.free_symbols if isinstance(s, sympy.Symbol)}


def add(left: sympy.Expr, right: sympy.Expr) -> sympy.Expr:
    """`left + right`."""
    return sympy.Add(left, right)


def sub(left: sympy.Expr, right: sympy.Expr) -> sympy.Expr:
    """`left - right`."""
    return sympy.Add(left, -right)


def mul(left: sympy.Expr, right: sympy.Expr) -> sympy.Expr:
    """`left * right`."""
    return sympy.Mul(left, right)


def div(left: sympy.Expr, right: sympy.Expr) -> sympy.Expr:
    """`left / right`."""
    return sympy.Mul(left, sympy.Pow(right, sympy.S.NegativeOne))


def apply(function: Callable[..., object], *args: sympy.Basic | int) -> sympy.Expr:
    """`function(*args)` for a SymPy function class (`exp`, `log`, `factorial`, an undefined
    function ...), whose application is an expression."""
    # the stubs declare the application of a function class as the class itself
    return cast(sympy.Expr, function(*args))

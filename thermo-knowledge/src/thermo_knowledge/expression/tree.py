# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The typed expression tree: one immutable node type for each construct of the grammar
(expressions.md section 3).

The tree is what the parser produces and what every backend reads. A node records where it was
written (`pos`, line and column from 1) but the position is left out of equality, so two
spellings of the same expression are equal trees. Nodes say what was written; what a name
refers to is decided against a contract and a form (`scope.py`), never stored in the tree.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass, field, fields

type Pos = tuple[int, int]
"""A position in expression text: line and column, both from 1."""

NO_POS: Pos = (0, 0)

ARITHMETIC = ("+", "-", "*", "/", "**")
COMPARISONS = ("<", "<=", ">", ">=", "==", "!=")
ELEMENTARY = (
    "exp",
    "log",
    "log10",
    "sqrt",
    "abs",
    "sinh",
    "cosh",
    "tanh",
    "sin",
    "cos",
    "atan",
    "min",
    "max",
)
SPECIAL = ("erf", "chebyshev_t", "debye")
FUNCTIONS = (*ELEMENTARY, *SPECIAL)
"""Functions called as `f(x, ...)`; `Func` nodes carry exactly these names."""
HEADS = ("sum", "prod", "d", "at", "integral", "unit", "range", "basis")
"""The other names the grammar reserves for its own constructs."""


@dataclass(frozen=True, slots=True, kw_only=True)
class Node:
    pos: Pos = field(default=NO_POS, compare=False, repr=False)


@dataclass(frozen=True, slots=True, kw_only=True)
class Num(Node):
    """A number. `1` is an integer, `1.0` and `1e-3` are floats."""

    value: int | float


@dataclass(frozen=True, slots=True, kw_only=True)
class UnitLiteral(Node):
    """`unit('Pa')`: the unit as a dimensioned factor, a `pint` unit text."""

    text: str


@dataclass(frozen=True, slots=True, kw_only=True)
class Name(Node):
    id: str


@dataclass(frozen=True, slots=True, kw_only=True)
class Attribute(Node):
    """`value.attr`: a slot of a slot group, a family of a group, an output of a sub-form."""

    value: Expr
    attr: str


@dataclass(frozen=True, slots=True, kw_only=True)
class Subscript(Node):
    """`base[i, j]`: the subjects and indices of an argument, a slot or a family slot."""

    base: Expr
    indices: tuple[Expr, ...]


@dataclass(frozen=True, slots=True, kw_only=True)
class Keyword(Node):
    name: str
    value: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Call(Node):
    """A call that is not a function of the grammar: a sub-form or nested-set call
    (`alpha.value(i=i, T=T)`), a family's index set (`pure.piece(i)`) or a sub-form iterated
    per subject (`terms(i=i)`)."""

    func: Expr
    args: tuple[Expr, ...] = ()
    keywords: tuple[Keyword, ...] = ()


@dataclass(frozen=True, slots=True, kw_only=True)
class Func(Node):
    """An elementary or special function; `name` is one of `FUNCTIONS`."""

    name: str
    args: tuple[Expr, ...]


@dataclass(frozen=True, slots=True, kw_only=True)
class BinOp(Node):
    op: str  # one of ARITHMETIC
    left: Expr
    right: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class UnaryOp(Node):
    op: str  # `-`, `+` or `not`
    operand: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Compare(Node):
    op: str  # one of COMPARISONS
    left: Expr
    right: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class BoolOp(Node):
    """`a and b and c`: nested operations of one operator are one flat node."""

    op: str  # `and` or `or`
    values: tuple[Expr, ...]


@dataclass(frozen=True, slots=True, kw_only=True)
class IfExp(Node):
    test: Expr
    body: Expr
    orelse: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Range(Node):
    """`range(a, b)`: the integers from `a` up to but not including `b` (an index set)."""

    start: Expr
    stop: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Clause(Node):
    """`for target in iterable`."""

    target: str
    iterable: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Reduce(Node):
    """`sum(body for ...)` or `prod(body for ...)`."""

    kind: str  # `sum` or `prod`
    body: Expr
    clauses: tuple[Clause, ...]


@dataclass(frozen=True, slots=True, kw_only=True)
class Comprehension(Node):
    """`[body for i in S for j in T]`: a value indexed over contract sets, one element for each
    combination. As a residual of an implicit block it is written without brackets, `body for i
    in S`, and states one residual for each element. `basis('name', [...])` asserts, in `basis`,
    the composition basis of the vector the comprehension builds."""

    body: Expr
    clauses: tuple[Clause, ...]
    basis: str | None = None


@dataclass(frozen=True, slots=True, kw_only=True)
class Derivative(Node):
    """`d(expr, name)`: the partial derivative with respect to an argument (subscripted when it
    is indexed) or a local."""

    expr: Expr
    wrt: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class At(Node):
    """`at(pure.piece(i), T)`: the index of the piece whose interval contains the value."""

    family: Expr
    value: Expr


@dataclass(frozen=True, slots=True, kw_only=True)
class Integral(Node):
    """`integral(expr, z, a, b)`: a definite integral in the scalar variable `z`."""

    body: Expr
    var: str
    lower: Expr
    upper: Expr


type Expr = (
    Num
    | UnitLiteral
    | Name
    | Attribute
    | Subscript
    | Call
    | Func
    | BinOp
    | UnaryOp
    | Compare
    | BoolOp
    | IfExp
    | Range
    | Reduce
    | Comprehension
    | Derivative
    | At
    | Integral
)


def children(node: Node) -> Iterator[Node]:
    """The nodes directly below `node`, in the order they are written."""
    for item in fields(node):
        value = getattr(node, item.name)
        if isinstance(value, Node):
            yield value
        elif isinstance(value, tuple):
            for element in value:
                if isinstance(element, Node):
                    yield element


def walk(node: Node) -> Iterator[Node]:
    """`node` and every node below it."""
    yield node
    for child in children(node):
        yield from walk(child)

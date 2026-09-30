# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Type expressions (section 2), units and quantity expressions (section 3.3).

Units are parsed and combined by `pint`'s default registry, the unit authority. A quantity
expression is parsed with the standard library's `ast` (the operators are Python's own:
`*`, `/` and `**`, with `^` accepted for the power) and evaluated over `pint` units.
"""

from __future__ import annotations

import ast
import math
import re
from collections.abc import Callable
from dataclasses import dataclass
from functools import cache
from typing import Literal

import pint

PRIMITIVES = ("Boolean", "Integer", "Text", "Date", "Timestamp", "Hash")
META_CONSTRUCTS = (
    "quantity_type",
    "kind",
    "contract",
    "form",
    "slot_group",
    "slot",
    "family",
    "subform_slot",
)
GENERICS = ("Id", "Range", "Array", "Set", "Meta")

ElementKind = Literal[
    "primitive",
    "identifier",
    "quantity",
    "expression",
    "enum",
    "kind",
    "record",
    "meta",
    "real",
    "source_text",
]
Container = Literal["scalar", "range", "array", "set"]


@dataclass(frozen=True, kw_only=True)
class TypeRef:
    """A resolved type expression.

    `element` is the primitive's name, the scheme, quantity type, enum or kind name, the
    normalised quantity expression, the `Meta` construct, or `Record`, `Real`, `SourceText`.
    """

    container: Container
    element_kind: ElementKind
    element: str
    text: str

    @property
    def is_floating(self) -> bool:
        """Whether values are floating point: quantities, expressions, `Real` and ranges."""
        return self.container == "range" or self.element_kind in ("quantity", "expression", "real")

    @property
    def identity_eligible(self) -> bool:
        """Whether the type may appear in an identity or a relation key."""
        return self.container == "scalar" and self.element_kind in (
            "primitive",
            "identifier",
            "enum",
            "kind",
            "record",
            "meta",
        )


_GENERIC = re.compile(r"^(?P<head>[A-Za-z]+)<(?P<inner>.*)>$", re.DOTALL)
_NAME = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")


@dataclass(frozen=True)
class Shape:
    """A type expression split into its outer form, before names are resolved."""

    head: str | None  # one of GENERICS, or None for a bare expression
    inner: str


def split_type(text: str) -> Shape:
    """Split `Set<K>` style generics from a bare name or quantity expression."""
    stripped = text.strip()
    match = _GENERIC.match(stripped)
    if match and match["head"] in GENERICS:
        return Shape(head=match["head"], inner=match["inner"].strip())
    return Shape(head=None, inner=stripped)


def is_bare_name(text: str) -> bool:
    """Whether `text` is one name rather than a quantity expression."""
    return _NAME.match(text) is not None


# -- units -------------------------------------------------------------------------------


class UnitError(ValueError):
    """A unit string or quantity expression is refused; `code` is the diagnostic code."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class UnitInfo:
    """A canonical unit and its dimensionality as base-dimension exponents."""

    unit: str
    dimensions: tuple[tuple[str, int], ...]


@cache
def registry() -> pint.UnitRegistry:
    """The one `pint` registry of the process (the default registry, no custom units)."""
    return pint.UnitRegistry()


def _info(unit: pint.Unit) -> UnitInfo:
    dimensions: list[tuple[str, int]] = []
    for dimension, exponent in unit.dimensionality.items():
        if float(exponent) != int(exponent):
            raise UnitError("bad-unit", f"unit `{unit}` has a fractional dimension exponent")
        dimensions.append((dimension.strip("[]"), int(exponent)))
    return UnitInfo(unit=format(unit, "~") or "dimensionless", dimensions=tuple(sorted(dimensions)))


def parse_storage_unit(text: str) -> tuple[pint.Unit, UnitInfo]:
    """Parse a storage unit: a coherent SI unit product whose factor to base units is 1."""
    ureg = registry()
    try:
        unit = ureg.parse_units(text)
        magnitude = ureg.Quantity(1.0, unit).to_root_units().magnitude
    except Exception as error:  # pint raises many error types for bad text
        raise UnitError("bad-unit", f"`{text}` is not a unit pint can parse: {error}") from error
    mass = unit.dimensionality.get("[mass]", 0)
    # pint's root mass unit is the gram; the coherent SI base unit is the kilogram.
    factor = magnitude * 1e-3**mass
    if not math.isclose(factor, 1.0, rel_tol=1e-12):
        raise UnitError(
            "unit-not-coherent",
            f"`{text}` is not a coherent SI unit product (its factor to base units is {factor:g})",
        )
    return unit, _info(unit)


def evaluate_expression(
    text: str, unit_of: Callable[[str], pint.Unit | None]
) -> tuple[str, pint.Unit]:
    """Evaluate a quantity expression over the storage units of declared quantity types.

    `unit_of` returns a type's unit, or `None` for an undeclared name (refused with code
    `unknown-name`). Returns the normalised expression and its unit.
    """
    try:
        tree = ast.parse(text.replace("^", "**"), mode="eval")
    except SyntaxError as error:
        raise UnitError("bad-quantity-expression", f"`{text}` is not an expression: {error.msg}")
    names: list[str] = []

    def integer(node: ast.expr) -> int:
        if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)):
            value = integer(node.operand)
            return -value if isinstance(node.op, ast.USub) else value
        if isinstance(node, ast.Constant) and type(node.value) is int:
            return node.value
        raise UnitError("bad-quantity-expression", f"`{text}`: a power must be an integer")

    def walk(node: ast.expr) -> pint.Unit:
        if isinstance(node, ast.Name):
            unit = unit_of(node.id)
            if unit is None:
                raise UnitError("unknown-name", node.id)
            names.append(node.id)
            return unit
        if isinstance(node, ast.Constant) and node.value == 1 and type(node.value) is int:
            return registry().Unit("dimensionless")
        if isinstance(node, ast.BinOp):
            if isinstance(node.op, ast.Mult):
                return walk(node.left) * walk(node.right)
            if isinstance(node.op, ast.Div):
                return walk(node.left) / walk(node.right)
            if isinstance(node.op, ast.Pow):
                return walk(node.left) ** integer(node.right)
        raise UnitError(
            "bad-quantity-expression",
            f"`{text}` may only multiply, divide and raise declared quantity types to integer "
            "powers",
        )

    unit = walk(tree.body)
    normalised = ast.unparse(tree).replace(" ** ", "^")
    return normalised, unit


def expression_info(unit: pint.Unit) -> UnitInfo:
    """The canonical unit and dimensionality of an evaluated expression."""
    return _info(unit)

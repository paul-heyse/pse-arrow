# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Dimensions of expression values, as `pint` dimensionalities.

A dimension is a `pint.util.UnitsContainer` of base dimensions (`[length]`, `[mass]`, ...);
the empty container is dimensionless. Multiplication, division and integer powers are the
container's own. The unit authority is the one `pint` registry of the tree
(`declaration.types.registry`).
"""

from __future__ import annotations

import math
from functools import cache

from pint.util import UnitsContainer

from thermo_knowledge.declaration.model import Declaration
from thermo_knowledge.declaration.types import TypeRef, UnitInfo, real_exponent, registry

type Dim = UnitsContainer

DIMENSIONLESS: Dim = UnitsContainer({})


def from_info(info: UnitInfo) -> Dim:
    """The dimension of a recorded unit."""
    return UnitsContainer({f"[{name}]": exponent for name, exponent in info.dimensions})


def dependent_dimension(decl: Declaration, type_name: str, extra_order: int = 0) -> Dim:
    """The dimension a declaration states for a dependent quantity type: one opaque symbol
    `[<TypeName>]`, stated once per type, times the dimension of a bulk concentration to the power
    minus the extra order. The concrete dimension follows the reaction and is checked per set
    when it is written; an expression sees the symbol, which closes under multiplication with the
    other quantities of the expression and only with a value of the same type and extra order."""
    quantity = decl.quantity_types[type_name]
    assert quantity.dependent is not None
    symbol = UnitsContainer({f"[{type_name}]": 1})
    concentration = from_info(decl.units[quantity.dependent.concentration])
    return symbol * concentration ** (-extra_order)


def type_dimension(decl: Declaration, type_: TypeRef, extra_order: int = 0) -> Dim | None:
    """The dimension of values of a quantity type or quantity expression; `None` for any other
    type (an enum, a reference, a primitive), which has none. `extra_order` is the extra
    concentration power of a field of a dependent quantity type."""
    if type_.container != "scalar":
        return None
    if type_.element_kind == "quantity":
        if decl.quantity_types[type_.element].dependent is not None:
            return dependent_dimension(decl, type_.element, extra_order)
        return from_info(decl.units[decl.quantity_types[type_.element].unit])
    if type_.element_kind == "expression":
        return from_info(decl.units[decl.expressions[type_.element]])
    return None


def describe(dim: Dim) -> str:
    """A dimension as text for a diagnostic: `dimensionless` or `[mass] / [length] / [time] ** 2`."""
    return str(dim)


def is_even(dim: Dim) -> bool:
    """Whether every exponent is an even integer, so a square root is a dimension."""
    return all(
        real_exponent(exponent).is_integer() and int(real_exponent(exponent)) % 2 == 0
        for _, exponent in dim.items()
    )


def sqrt_dim(dim: Dim) -> Dim:
    """The square root of a dimension whose exponents are all even."""
    return UnitsContainer(
        {name: int(real_exponent(exponent)) // 2 for name, exponent in dim.items()}
    )


class UnitLiteralError(ValueError):
    """The text of a `unit(...)` literal is not a unit the grammar accepts."""


@cache
def unit_literal(text: str) -> tuple[float, Dim]:
    """The factor from the unit `text` to the coherent SI unit of its dimension, and that
    dimension. `unit('kPa')` is 1000 of the Pa a stored pressure is in."""
    ureg = registry()
    try:
        unit = ureg.parse_units(text)
        one = ureg.Quantity(1.0, unit)
        two = ureg.Quantity(2.0, unit)
        factor = float(one.to_base_units().magnitude)
        doubled = float(two.to_base_units().magnitude)
    except Exception as error:  # pint raises many error types for bad text
        raise UnitLiteralError(f"`{text}` is not a unit pint can parse: {error}") from error
    if not math.isclose(doubled, 2 * factor, rel_tol=1e-12):
        raise UnitLiteralError(f"`{text}` is an offset unit; only multiplicative units are allowed")
    return factor, UnitsContainer(dict(unit.dimensionality.items()))

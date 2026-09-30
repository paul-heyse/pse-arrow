# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The concrete unit of a dependent quantity type for one reaction (meta-model section 3.3).

A rate constant's dimension follows its reaction: the rate per volume (per area when any
participant is on a surface) divided by the concentration of each species to its forward order,
and by one more bulk concentration for each extra order the slot declares. This module holds the
unit algebra only; what a reaction's participants, orders and phases are is the writer's to read.
The stored number is in the coherent SI unit of the result, so a value stated in another unit is
converted to it and a value of another dimension is refused.
"""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from fractions import Fraction

from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.types import registry

ORDER_PRECISION = 10**9
"""Orders are exact rationals with at most this denominator: 1.5 and 0.3 are 3/2 and 3/10."""


@dataclass(frozen=True)
class Species:
    """One species form whose concentration enters the unit: its order in the forward rate and
    whether it is counted per area (on a surface) rather than per volume."""

    order: float
    per_area: bool


def exponent(order: float) -> Fraction:
    """The exact rational the order denotes."""
    return Fraction(order).limit_denominator(ORDER_PRECISION)


def concrete_unit(
    decl: m.Declaration,
    type_name: str,
    extra_order: int,
    *,
    surface: bool,
    species: Iterable[Species],
) -> str:
    """The storage unit, as text `pint` parses, of a value of dependent quantity type `type_name`
    with `extra_order` extra concentration powers, for a reaction that has a surface participant
    when `surface` is true and whose forward rate depends on `species`."""
    quantity = decl.quantity_types[type_name]
    dependent = quantity.dependent
    assert dependent is not None, f"{type_name} is not a dependent quantity type"
    ureg = registry()
    bulk = ureg.parse_units(dependent.concentration)
    area = ureg.parse_units(dependent.surface_concentration)
    unit = ureg.parse_units(dependent.surface_unit if surface else quantity.unit)
    for each in species:
        unit = unit / (area if each.per_area else bulk) ** exponent(each.order)
    if extra_order:
        unit = unit / bulk**extra_order
    return str(unit)

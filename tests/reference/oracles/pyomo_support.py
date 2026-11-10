# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Precise boundaries for Pyomo's deferred units and numeric evaluation.

Pyomo 6.10.1 exposes units through a deferred singleton whose inferred attribute
return is the singleton class, although resolved attributes are NumericValue
instances. Numeric evaluation with exception=True either returns a number or
raises; it never supplies an absent numerical observation to these oracles.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, cast

from pyomo.environ import units as _units
from pyomo.environ import value as _value

if TYPE_CHECKING:
    from pyomo.core.expr.numeric_expr import NumericValue


class UnitContainer(Protocol):
    """The public unit lookup and conversion surface consumed by the references."""

    def __getattr__(self, name: str) -> NumericValue:
        """Resolve a unit name through Pyomo's deferred unit registry."""
        ...

    def convert(self, expression: object, *, to_units: object) -> NumericValue:
        """Convert a Pyomo expression into the selected representation unit."""
        ...

    def get_units(self, expression: object) -> NumericValue:
        """Read the physical units of one Pyomo expression."""
        ...

    def convert_value(
        self, value: float, *, from_units: object, to_units: object
    ) -> float:
        """Convert an unmodified scalar between declared unit expressions."""
        ...


units = cast("UnitContainer", _units)


def value(expression: object) -> float:
    """Evaluate one numeric expression, refusing absent or nonnumeric evidence."""
    result: object = _value(expression, exception=True)
    if isinstance(result, bool) or not isinstance(result, (int, float)):
        raise TypeError("Pyomo reference observation is not a real number")
    return float(result)

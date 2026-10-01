# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Values a mapping hands the canonical writer, and their validation against a declared type.

A dimensioned value is given with the unit the source states (or the mapping assumes); the
writer converts it to the storage unit with `pint` and never guesses a unit. A stateful slot
takes one of the state markers instead of a number.
"""

from __future__ import annotations

import math
import re
import unicodedata
import uuid
from collections.abc import Sequence
from dataclasses import dataclass
from datetime import UTC, date, datetime

import pint

from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.types import TypeRef, registry

type Converted = (
    str
    | int
    | float
    | bool
    | date
    | datetime
    | bytes
    | uuid.UUID
    | list[str]
    | list[int]
    | list[float]
    | list[bool]
    | list[uuid.UUID]
    | None
)
"""A value after validation and conversion: what a canonical column holds."""


@dataclass(frozen=True)
class Quantity:
    """A number with the unit its source states."""

    value: float
    unit: str


@dataclass(frozen=True)
class QuantityArray:
    """Numbers sharing one source unit."""

    values: Sequence[float]
    unit: str


@dataclass(frozen=True)
class NotApplicable:
    """A stateful slot with no meaning for its subject."""


@dataclass(frozen=True)
class Withheld:
    """A stateful slot whose value exists and may not be stored."""


@dataclass(frozen=True)
class Redirect:
    """A stateful slot whose value is that of another parameter set of the same slot group."""

    parameter_set: uuid.UUID


@dataclass(frozen=True)
class StatedDefault:
    """A stateful slot the source leaves at its stated default and gives no value for: the value is
    the default the selection policy in force states for the slot (`policy_default`)."""


type SlotState = NotApplicable | Withheld | Redirect | StatedDefault

HEX_HASH = re.compile(r"^[0-9a-fA-F]{64}$")


class Problems:
    """The problems found while validating one record, each naming the value it is about."""

    def __init__(self) -> None:
        self.items: list[str] = []

    def add(self, where: str, message: str) -> None:
        self.items.append(f"{where}: {message}")

    def __bool__(self) -> bool:
        return bool(self.items)


class ValueRefused(ValueError):
    """One value does not fit its type; the message says why."""


def storage_unit(decl: m.Declaration, type_: TypeRef) -> str | None:
    """The storage unit of a quantity or quantity-expression type, else `None`. A dependent
    quantity type has none: its unit follows the reaction of the set and the writer supplies it as
    the `unit_hint`."""
    if type_.element_kind == "quantity":
        quantity = decl.quantity_types[type_.element]
        return None if quantity.dependent is not None else quantity.unit
    if type_.element_kind == "expression":
        return decl.expressions[type_.element]
    return None


def is_dimensionless(unit: str) -> bool:
    ureg = registry()
    return ureg.parse_units(unit).dimensionless


def convert(value: float, source: str, target: str) -> float:
    """`value` in the unit `source` expressed in `target`, through `pint`."""
    ureg = registry()
    try:
        quantity = ureg.Quantity(float(value), ureg.parse_units(source))
        return float(quantity.to(ureg.parse_units(target)).magnitude)
    except pint.DimensionalityError as error:
        raise ValueRefused(
            f"a value in `{source}` cannot be converted to `{target}`: {error}"
        ) from error
    except Exception as error:  # pint raises many error types for bad unit text
        raise ValueRefused(f"`{source}` is not a unit pint can parse: {error}") from error


def conversion_factor(source: str, target: str) -> float | None:
    """The factor from `source` to `target`, or `None` for a unit with an offset (a
    temperature in degrees Celsius has no factor)."""
    one = convert(1.0, source, target)
    zero = convert(0.0, source, target)
    return one if math.isclose(zero, 0.0, abs_tol=1e-300) else None


def _finite(number: float) -> float:
    if not math.isfinite(number):
        raise ValueRefused(f"{number!r} is not finite")
    return number


def _number(decl: m.Declaration, type_: TypeRef, raw: object, unit_hint: str | None) -> float:
    """A dimensioned value converted to the storage unit of `type_` (or `unit_hint`)."""
    target = storage_unit(decl, type_) or unit_hint
    if target is None:
        raise ValueRefused(f"no storage unit is known for a value of type {type_.text}")
    if isinstance(raw, Quantity):
        number = _finite(convert(_finite(float(raw.value)), raw.unit, target))
    elif isinstance(raw, bool) or not isinstance(raw, (int, float)):
        raise ValueRefused(f"{raw!r} is not a number with a unit")
    elif is_dimensionless(target):
        number = _finite(float(raw))
    else:
        raise ValueRefused(
            f"{raw!r} has no unit, and the storage unit of {type_.text} is `{target}`; "
            "give the value with its source unit"
        )
    if (
        type_.element_kind == "quantity"
        and decl.quantity_types[type_.element].scale == "absolute"
        and number < 0
    ):
        raise ValueRefused(f"{number!r} is negative, as an absolute quantity must not be")
    return number


def scalar(
    decl: m.Declaration,
    type_: TypeRef,
    raw: object,
    *,
    unit_hint: str | None = None,
    meta_ids: dict[str, dict[str, uuid.UUID]] | None = None,
) -> Converted:
    """Validate one scalar `raw` against `type_` (container `scalar`) and convert it."""
    kind = type_.element_kind
    if kind == "primitive":
        name = type_.element
        if name == "Boolean" and isinstance(raw, bool):
            return raw
        if name == "Integer" and isinstance(raw, int) and not isinstance(raw, bool):
            return raw
        if name == "Text" and isinstance(raw, str):
            return raw
        if name == "Date" and isinstance(raw, date) and not isinstance(raw, datetime):
            return raw
        if name == "Timestamp" and isinstance(raw, datetime):
            if raw.tzinfo is None:
                raise ValueRefused(f"{raw!r} is a timestamp without a time zone")
            return raw.astimezone(UTC)
        if name == "Hash":
            if isinstance(raw, bytes) and len(raw) == 32:
                return raw
            if isinstance(raw, str) and HEX_HASH.match(raw):
                return bytes.fromhex(raw)
        raise ValueRefused(f"{raw!r} is not a {name}")
    if kind == "identifier":
        if isinstance(raw, str) and raw != "":
            return raw
        raise ValueRefused(f"{raw!r} is not a non-empty identifier")
    if kind in ("quantity", "expression", "real"):
        return _number(decl, type_, raw, unit_hint)
    if kind == "enum":
        members = {member.name for member in decl.enums[type_.element].members}
        if isinstance(raw, str) and raw in members:
            return raw
        raise ValueRefused(
            f"{raw!r} is not a member of enum `{type_.element}` "
            f"(one of {', '.join(sorted(members))})"
        )
    if kind in ("kind", "record"):
        if isinstance(raw, uuid.UUID):
            return raw
        raise ValueRefused(f"{raw!r} is not an identifier (a UUID) of a {type_.element}")
    if kind == "meta":
        if isinstance(raw, uuid.UUID):
            return raw
        if isinstance(raw, str) and meta_ids is not None:
            found = meta_ids.get(type_.element, {}).get(raw)
            if found is not None:
                return found
            raise ValueRefused(f"`{raw}` is not a declared {type_.element}")
        raise ValueRefused(f"{raw!r} is not a {type_.text}")
    if kind == "source_text" and isinstance(raw, str):
        return raw
    raise ValueRefused(f"a value of type {type_.text} cannot be written here")


def array(
    decl: m.Declaration,
    type_: TypeRef,
    raw: object,
    *,
    unit_hint: str | None = None,
) -> Converted:
    """Validate an `Array<T>` value: a sequence of primitives or enum members, or a
    `QuantityArray` for dimensioned elements."""
    element = TypeRef(
        container="scalar",
        element_kind=type_.element_kind,
        element=type_.element,
        text=type_.element,
    )
    if type_.element_kind in ("quantity", "expression", "real"):
        if not isinstance(raw, QuantityArray):
            raise ValueRefused(f"{type_.text} takes a QuantityArray (numbers with one unit)")
        return [_number(decl, element, Quantity(v, raw.unit), unit_hint) for v in raw.values]
    if not isinstance(raw, (list, tuple)):
        raise ValueRefused(f"{type_.text} takes a list")
    return [scalar(decl, element, item) for item in raw]  # type: ignore[misc]


def nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text)

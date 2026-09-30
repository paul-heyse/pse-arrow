# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Deterministic identifiers (meta-model section 5).

An instance's identifier is a version-5 UUID. Its namespace is itself a version-5 UUID
derived from the one root namespace below and a name: the root kind's name for a kind
instance, the relation's name for a relation row, `meta:<construct>` for a reified
declaration row. The identifier's name is the canonical encoding of the identity values:

* a compact JSON array (no spaces, non-ASCII characters kept) in declaration order;
* text, identifier-scheme values and enum members as JSON strings, NFC-normalised;
* references (kind instances, records, reified rows) as lowercase hyphenated UUID strings;
* integers as JSON integers in decimal, booleans as `true` and `false`;
* a `Hash` as lowercase hexadecimal, a `Date` as ISO 8601 and a `Timestamp` as a UTC ISO 8601
  instant with microseconds and a `Z` suffix.

Floating-point values never take part in an identifier and are refused.
"""

from __future__ import annotations

import json
import unicodedata
import uuid
from collections.abc import Sequence
from datetime import UTC, date, datetime

ROOT_NAMESPACE = uuid.UUID("5ad3ac16-681d-4c3c-878f-2408aa1bef6f")
"""The fixed root of every identifier namespace. Declared here and nowhere else."""

type IdentityValue = str | int | bool | uuid.UUID | date | datetime | bytes
"""A value an identifier may be computed from."""


class IdentityError(ValueError):
    """A value cannot take part in an identifier."""


def namespace(name: str) -> uuid.UUID:
    """The namespace of a kind's root name, a relation's name or `meta:<construct>`."""
    return uuid.uuid5(ROOT_NAMESPACE, unicodedata.normalize("NFC", name))


def _encode_value(value: IdentityValue) -> str | int | bool:
    # bool before int (a bool is an int); datetime before date (a datetime is a date).
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        return unicodedata.normalize("NFC", value)
    if isinstance(value, uuid.UUID):
        return str(value)
    if isinstance(value, datetime):
        if value.tzinfo is None:
            raise IdentityError("a timestamp in an identifier must carry a time zone")
        instant = value.astimezone(UTC)
        return instant.isoformat(timespec="microseconds").removesuffix("+00:00") + "Z"
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, bytes):
        return value.hex()
    if isinstance(value, float):
        raise IdentityError("a floating-point value may not take part in an identifier")
    raise IdentityError(f"{type(value).__name__} cannot take part in an identifier")


def canonical_encoding(values: Sequence[IdentityValue]) -> str:
    """The canonical JSON text of identity `values`, in the order given."""
    return json.dumps(
        [_encode_value(value) for value in values], ensure_ascii=False, separators=(",", ":")
    )


def identifier(namespace_name: str, values: Sequence[IdentityValue]) -> uuid.UUID:
    """The identifier of the instance named by `values` in `namespace_name`'s namespace."""
    return uuid.uuid5(namespace(namespace_name), canonical_encoding(values))


def meta_identifier(construct: str, qualified_name: str) -> uuid.UUID:
    """The identifier of a reified declaration row (`Meta<construct>`).

    `qualified_name` is the construct's name: `form.group` for a slot group,
    `form.group.slot` for a slot, `form.group.family` for a family and `form.subform` for a
    sub-form slot.
    """
    return identifier(f"meta:{construct}", [qualified_name])

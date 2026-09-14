# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Strict structuring for the Python boundary (blueprint §21.5).

Two serialisation libraries with one division of labour: attrs classes describe
the relation-row contracts and are structured by cattrs; msgspec describes the
manifests and other documents that are read from and written to disk as JSON.

Both are strict. cattrs' default converter *ignores* unknown keys, which turns a
renamed column into a silently missing value; :func:`converter` therefore
installs a structure-hook factory that rebuilds every attrs hook with
``_cattrs_forbid_extra_keys=True``. msgspec structs declare
``forbid_unknown_fields=True`` for the same reason. A payload that does not
match the contract is an error at the boundary, never a default further in.
"""

from collections.abc import Callable, Iterable, Mapping
from datetime import UTC, datetime
from typing import TypeVar

import attrs
import cattrs
import msgspec
import msgspec.json
from cattrs.gen import make_dict_structure_fn, make_dict_unstructure_fn, override

from pse.contracts.manifest import Manifest
from pse.contracts.values import FIELD_NAME_METADATA, ContentHash, SemanticId

__all__ = [
    "Manifest",
    "converter",
    "decode_json",
    "encode_json",
    "render_errors",
    "structure_rows",
]

T = TypeVar("T")
S = TypeVar("S", bound=msgspec.Struct)


def converter() -> cattrs.Converter:
    """Build a cattrs converter that refuses unknown keys.

    A fresh converter per call: converters carry mutable hook registries, and a
    module-level singleton would let one caller's registration change another
    caller's parsing.

    Returns:
        A converter whose attrs structure hooks are generated with
        ``_cattrs_forbid_extra_keys=True`` (blueprint §21.5).
    """
    conv = cattrs.Converter()
    conv.register_structure_hook_factory(
        attrs.has,
        lambda cls: make_dict_structure_fn(
            cls,
            conv,
            _cattrs_forbid_extra_keys=True,
            _cattrs_use_linecache=True,
            _cattrs_prefer_attrib_converters="from_converter",
            _cattrs_detailed_validation="from_converter",
            _cattrs_use_alias="from_converter",
            _cattrs_include_init_false=False,
            **{
                attribute: override(rename=wire)
                for attribute, wire in _wire_names(cls).items()
            },
        ),
    )
    conv.register_unstructure_hook_factory(
        attrs.has,
        lambda cls: make_dict_unstructure_fn(
            cls,
            conv,
            _cattrs_omit_if_default=False,
            _cattrs_use_linecache=True,
            _cattrs_use_alias="from_converter",
            _cattrs_include_init_false=False,
            **{
                attribute: override(rename=wire)
                for attribute, wire in _wire_names(cls).items()
            },
        ),
    )
    conv.register_structure_hook(SemanticId, _structure_id)
    conv.register_structure_hook(ContentHash, _structure_hash)
    conv.register_structure_hook(int, _structure_int)
    conv.register_structure_hook(float, _structure_float)
    conv.register_structure_hook(str, _structure_text)
    conv.register_structure_hook(bool, _structure_bool)
    conv.register_structure_hook(datetime, _structure_datetime)
    conv.register_unstructure_hook(SemanticId, bytes)
    conv.register_unstructure_hook(ContentHash, bytes)
    conv.register_unstructure_hook(datetime, datetime.isoformat)
    return conv


def _wire_names(cls: type) -> dict[str, str]:
    """Read the exact generated field mapping, refusing ambiguous metadata."""
    result: dict[str, str] = {}
    used: set[str] = set()
    for field in attrs.fields(cls):
        wire = field.metadata.get(FIELD_NAME_METADATA, field.name)
        if not isinstance(wire, str) or wire in used:
            message = "generated field names must be unique declared text names"
            raise ValueError(message)
        result[field.name] = wire
        used.add(wire)
    return result


def structure_rows(rows: Iterable[Mapping[str, object]], cls: type[T]) -> list[T]:
    """Structure an iterable of row mappings into contract instances.

    Args:
        rows: The mappings to structure, typically ``RecordBatch.to_pylist()``.
        cls: The attrs contract class each row must satisfy.

    Returns:
        One instance of ``cls`` per row, in input order.

    Raises:
        cattrs.BaseValidationError: If any row carries an unknown key, is
            missing a field, or holds a value of the wrong type. Render it with
            :func:`render_errors`.
    """
    conv = converter()
    return [conv.structure(row, cls) for row in rows]


def render_errors(exc: Exception) -> list[str]:
    """Flatten a cattrs validation error into one finding per offending field.

    Args:
        exc: The exception raised by :func:`structure_rows`, normally a
            ``cattrs.BaseValidationError``.

    Returns:
        One human-readable finding per leaf error, each naming the path to the
        field it is about. Never a single flattened "structuring failed".
    """
    return list(cattrs.transform_error(exc))


def decode_json(data: bytes | str, type: type[S]) -> S:
    """Decode JSON into a msgspec struct, refusing unknown fields.

    Args:
        data: The JSON document.
        type: The msgspec struct the document must match.

    Returns:
        The decoded struct.

    Raises:
        msgspec.ValidationError: If the document does not match the struct.
        msgspec.DecodeError: If the document is not well-formed JSON.
    """
    return msgspec.json.decode(data, type=type)


def encode_json(obj: msgspec.Struct) -> bytes:
    """Encode a msgspec struct as JSON bytes.

    Args:
        obj: The struct to encode.

    Returns:
        The encoded document, as bytes.
    """
    return msgspec.json.encode(obj)


def _structure_id(value: object, _cls: type[SemanticId]) -> SemanticId:
    if isinstance(value, bytes):
        return SemanticId(value)
    if isinstance(value, str):
        return SemanticId.from_hex(value)
    message = "identity must be sixteen bytes or explicit hexadecimal text"
    raise ValueError(message)


def _structure_hash(value: object, _cls: type[ContentHash]) -> ContentHash:
    if isinstance(value, bytes):
        return ContentHash(value)
    if isinstance(value, str):
        return ContentHash.from_prefixed(value)
    message = "digest must be thirty-two bytes or blake3-prefixed text"
    raise ValueError(message)


def _structure_int(value: object, _cls: type[int]) -> int:
    if type(value) is int:
        return value
    message = "integer fields do not coerce text, floats or booleans"
    raise ValueError(message)


def _structure_float(value: object, _cls: type[float]) -> float:
    if type(value) is float:
        return value
    if type(value) is int:
        result = float(value)
        if int(result) == value:
            return result
    message = "Float64 fields require an exactly representable numeric value"
    raise ValueError(message)


def _structure_text(value: object, construct: Callable[[str], str]) -> str:
    if isinstance(value, str):
        return construct(value)
    message = "text fields do not coerce other value kinds"
    raise ValueError(message)


def _structure_bool(value: object, _cls: type[bool]) -> bool:
    if type(value) is bool:
        return value
    message = "boolean fields do not coerce numbers or text"
    raise ValueError(message)


def _structure_datetime(value: object, _cls: type[datetime]) -> datetime:
    if isinstance(value, str):
        value = datetime.fromisoformat(value)
    if (
        isinstance(value, datetime)
        and value.tzinfo is not None
        and value.utcoffset() == UTC.utcoffset(None)
    ):
        return value
    message = "timestamps require an aware UTC datetime or RFC3339 value"
    raise ValueError(message)

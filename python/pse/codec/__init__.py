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
import msgspec.inspect
import msgspec.json
from cattrs.gen import make_dict_structure_fn, make_dict_unstructure_fn, override

from pse.contracts.values import FIELD_NAME_METADATA, ContentHash, SemanticId
from pse.governance import check_class

__all__ = [
    "converter",
    "decode_json",
    "decode_rows_json",
    "document_rows",
    "encode_json",
    "render_errors",
    "structure_rows",
]

T = TypeVar("T")


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
    check_class(cls)
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


def _version_children(info: msgspec.inspect.Type) -> tuple[msgspec.inspect.Type, ...]:
    if isinstance(
        info,
        (
            msgspec.inspect.StructType,
            msgspec.inspect.TypedDictType,
            msgspec.inspect.DataclassType,
            msgspec.inspect.NamedTupleType,
        ),
    ):
        return tuple(field.type for field in info.fields)
    if isinstance(info, msgspec.inspect.UnionType):
        return info.types
    if isinstance(info, msgspec.inspect.Metadata):
        return (info.type,)
    if isinstance(info, msgspec.inspect.DictType):
        return (info.value_type,)
    if isinstance(info, msgspec.inspect.CollectionType):
        return (info.item_type,)
    if isinstance(info, msgspec.inspect.TupleType):
        return info.item_types
    return ()


def _has_required_version(info: msgspec.inspect.Type, seen: set[int]) -> bool:
    if isinstance(info, msgspec.inspect.StructType) and isinstance(
        getattr(info.cls, "_pse_required_version", None), int
    ):
        return True
    identity = id(info)
    if identity in seen:
        return False
    seen.add(identity)
    return any(_has_required_version(child, seen) for child in _version_children(info))


def _check_version_headers(
    value: object, info: msgspec.inspect.Type, path: str
) -> None:
    """Check supplied schema-required headers before constructor defaults apply."""
    if isinstance(info, msgspec.inspect.Metadata):
        _check_version_headers(value, info.type, path)
    elif isinstance(info, msgspec.inspect.UnionType):
        for alternative in info.types:
            _check_version_headers(value, alternative, path)
    elif isinstance(info, msgspec.inspect.StructType) and isinstance(value, dict):
        if info.tag_field is not None and value.get(info.tag_field) != info.tag:
            return
        expected = getattr(info.cls, "_pse_required_version", None)
        if expected is not None:
            if "version" not in value:
                message = f"Missing required document version - at `{path}.version`"
                raise msgspec.ValidationError(message)
            version = value["version"]
            if (
                not isinstance(version, int)
                or isinstance(version, bool)
                or version != expected
            ):
                message = (
                    f"Unsupported document version (current: {expected}) "
                    f"- at `{path}.version`"
                )
                raise msgspec.ValidationError(message)
        for field in info.fields:
            if field.encode_name in value:
                _check_version_headers(
                    value[field.encode_name], field.type, f"{path}.{field.encode_name}"
                )
    elif isinstance(
        info, (msgspec.inspect.TypedDictType, msgspec.inspect.DataclassType)
    ) and isinstance(value, dict):
        for field in info.fields:
            if field.encode_name in value:
                _check_version_headers(
                    value[field.encode_name], field.type, f"{path}.{field.encode_name}"
                )
    elif isinstance(info, msgspec.inspect.NamedTupleType) and isinstance(value, list):
        for index, (item, field) in enumerate(zip(value, info.fields, strict=False)):
            _check_version_headers(item, field.type, f"{path}[{index}]")
    elif isinstance(info, msgspec.inspect.DictType) and isinstance(value, dict):
        for key, item in value.items():
            _check_version_headers(item, info.value_type, f"{path}[{key!r}]")
    elif isinstance(info, msgspec.inspect.CollectionType) and isinstance(value, list):
        for index, item in enumerate(value):
            _check_version_headers(item, info.item_type, f"{path}[{index}]")
    elif isinstance(info, msgspec.inspect.TupleType) and isinstance(value, list):
        for index, (item, item_type) in enumerate(
            zip(value, info.item_types, strict=False)
        ):
            _check_version_headers(item, item_type, f"{path}[{index}]")


def decode_json(data: bytes | str, type: type[T]) -> T:
    """Decode JSON into the selected generated document, refusing unknown fields.

    Args:
        data: The JSON document.
        type: The generated document or tagged union the payload must match.

    Returns:
        The decoded struct.

    Raises:
        msgspec.ValidationError: If the document does not match the struct.
        msgspec.DecodeError: If the document is not well-formed JSON.
    """
    info = msgspec.inspect.type_info(type)
    if _has_required_version(info, set()):
        raw: object = msgspec.json.decode(data, type=object)
        _check_version_headers(raw, info, "$")
    return msgspec.json.decode(data, type=type)


def decode_rows_json(data: bytes | str, row_type: type[T]) -> tuple[T, ...]:
    """Decode a native inventory whose row shape is generated by the registry."""
    return tuple(
        converter().structure(msgspec.json.decode(data, type=object), list[row_type])
    )


def document_rows(values: Iterable[object], row_type: type[T]) -> tuple[T, ...]:
    """Project document rows to the registry's generated relation-row type."""
    conv = converter()
    return tuple(
        conv.structure(msgspec.to_builtins(value), row_type) for value in values
    )


def encode_json(obj: object) -> bytes:
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

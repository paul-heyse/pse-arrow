# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Observations of an explicit consumer schema; no implicit registration claims."""

from typing import Literal

import attrs
import pyarrow as pa

_NAME = b"ARROW:extension:name"
_METADATA = b"ARROW:extension:metadata"


@attrs.frozen
class FieldTransfer:
    """One nested field's observed transfer state.

    Attributes:
        path: Field names from root to nested child; empty for a schema-level
            order or metadata mismatch.
        state: Retained, storage-only, missing metadata or incompatible shape.
        extension_name: Declared extension name, or None for ordinary storage.
    """

    path: tuple[str, ...]
    state: Literal["retained", "storage_only", "metadata_lost", "mismatch"]
    extension_name: str | None


def _storage(ty: pa.DataType) -> pa.DataType:
    return ty.storage_type if isinstance(ty, pa.BaseExtensionType) else ty


def _children(ty: pa.DataType) -> tuple[pa.Field, ...]:
    ty = _storage(ty)
    if pa.types.is_struct(ty):
        return tuple(ty)
    if (
        pa.types.is_list(ty)
        or pa.types.is_large_list(ty)
        or pa.types.is_fixed_size_list(ty)
    ):
        return (ty.value_field,)
    return ()


def _flatten(schema: pa.Schema) -> dict[tuple[str, ...], pa.Field]:
    result: dict[tuple[str, ...], pa.Field] = {}

    def visit(field: pa.Field, parent: tuple[str, ...]) -> None:
        path = (*parent, field.name)
        if path in result:
            message = "consumer schema contains a duplicate nested field path"
            raise ValueError(message)
        result[path] = field
        for child in _children(field.type):
            visit(child, path)

    for field in schema:
        visit(field, ())
    return result


def _extension(field: pa.Field) -> tuple[bytes | None, bytes | None]:
    if isinstance(field.type, pa.ExtensionType):
        return field.type.extension_name.encode(), field.type.__arrow_ext_serialize__()
    metadata = field.metadata or {}
    return metadata.get(_NAME), metadata.get(_METADATA)


def _same_shape(expected: pa.Field, actual: pa.Field) -> bool:
    if expected.nullable != actual.nullable:
        return False
    left, right = _storage(expected.type), _storage(actual.type)
    if left.id != right.id:
        return False
    children = _children(left)
    if children:
        if tuple(child.name for child in children) != tuple(
            child.name for child in _children(right)
        ):
            return False
        if pa.types.is_fixed_size_list(left):
            return left.list_size == right.list_size
        return True
    return left == right


def _extra_metadata(field: pa.Field) -> dict[bytes, bytes]:
    return {
        key: value
        for key, value in (field.metadata or {}).items()
        if key not in (_NAME, _METADATA)
    }


def compare_schemas(
    source: pa.Schema, consumer: pa.Schema
) -> tuple[FieldTransfer, ...]:
    """Compare actual schemas without inferring a consumer's extension registry.

    Args:
        source: Exact admitted source schema, interpreted by the registered adapter.
        consumer: Schema supplied by the actual consumer after transfer.

    Returns:
        Complete per-field observations; incompatible fields are never admitted.
    """
    expected, observed = _flatten(source), _flatten(consumer)
    result: list[FieldTransfer] = []
    if source.names != consumer.names or source.metadata != consumer.metadata:
        result.append(FieldTransfer((), "mismatch", None))
    for path in dict.fromkeys((*expected, *observed)):
        field = expected.get(path)
        actual = observed.get(path)
        name, metadata = (None, None) if field is None else _extension(field)
        extension_name = (
            None if name is None else name.decode("utf-8", errors="replace")
        )
        state: Literal["retained", "storage_only", "metadata_lost", "mismatch"]
        if field is None or actual is None or not _same_shape(field, actual):
            state = "mismatch"
        else:
            actual_name, actual_metadata = _extension(actual)
            if _extra_metadata(field) != _extra_metadata(actual):
                state = "mismatch"
            elif (name, metadata) != (actual_name, actual_metadata):
                state = (
                    "metadata_lost"
                    if name is not None and actual_name is None
                    else "mismatch"
                )
            elif name is not None and not isinstance(actual.type, pa.BaseExtensionType):
                state = "storage_only"
            else:
                state = "retained"
        result.append(FieldTransfer(path, state, extension_name))
    return tuple(result)

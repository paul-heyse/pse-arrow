# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Observations of actual consumer schemas, including nested transfer degradation."""

from typing import Literal

import attrs
import pyarrow as pa

_NAME = b"ARROW:extension:name"
_METADATA = b"ARROW:extension:metadata"
_DICTIONARY_VALUES = "<dictionary-values>"
TransferState = Literal["retained", "storage_only", "metadata_lost", "mismatch"]
_SEVERITY: dict[TransferState, int] = {
    "retained": 0,
    "storage_only": 1,
    "metadata_lost": 2,
    "mismatch": 3,
}


@attrs.frozen
class FieldTransfer:
    """One field's observed transfer state, including its descendants.

    Attributes:
        path: Native field names from root to child. A dictionary has a virtual
            <dictionary-values> step before its value type; this is not an Arrow
            field or a value-nullability declaration. Empty denotes a schema-level
            order or metadata mismatch.
        state: Retained, storage-only, missing metadata or incompatible meaning.
            Parents carry the strongest descendant degradation as well.
        extension_name: This node's extension name, or None for ordinary storage.
    """

    path: tuple[str, ...]
    state: TransferState
    extension_name: str | None


def _storage(ty: pa.DataType) -> pa.DataType:
    return ty.storage_type if isinstance(ty, pa.BaseExtensionType) else ty


def _children(ty: pa.DataType) -> tuple[pa.Field, ...]:
    ty = _storage(ty)
    if isinstance(ty, pa.DictionaryType):
        return (pa.field(_DICTIONARY_VALUES, ty.value_type),)
    return tuple(ty.field(index) for index in range(ty.num_fields))


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


def _extension(field: pa.Field) -> tuple[bytes | None, bytes | None, bool]:
    if isinstance(field.type, pa.ExtensionType):
        return (
            field.type.extension_name.encode(),
            field.type.__arrow_ext_serialize__(),
            True,
        )
    if isinstance(field.type, pa.BaseExtensionType):
        # Native canonical extensions need not expose Python serialization. Native
        # equality can prove retained meaning; opaque payloads cannot prove a raw
        # storage-only descriptor equivalent.
        return field.type.extension_name.encode(), None, False
    metadata = field.metadata or {}
    return metadata.get(_NAME), metadata.get(_METADATA), True


def _same_shape(expected: pa.Field, actual: pa.Field) -> bool:
    if expected.name != actual.name or expected.nullable != actual.nullable:
        return False
    left, right = _storage(expected.type), _storage(actual.type)
    if left.equals(right, check_metadata=True):
        return True
    if left.id != right.id:
        return False
    if tuple(f.name for f in _children(left)) != tuple(
        f.name for f in _children(right)
    ):
        return False
    # Compare container parameters separately only when descendant extension loss
    # prevents complete native equality. Child fields are compared independently.
    if isinstance(left, pa.DictionaryType) and isinstance(right, pa.DictionaryType):
        return (
            left.index_type.equals(right.index_type) and left.ordered == right.ordered
        )
    if isinstance(left, pa.UnionType) and isinstance(right, pa.UnionType):
        return left.mode == right.mode and left.type_codes == right.type_codes
    if isinstance(left, pa.MapType) and isinstance(right, pa.MapType):
        return left.keys_sorted == right.keys_sorted
    if isinstance(left, pa.FixedSizeListType) and isinstance(
        right, pa.FixedSizeListType
    ):
        return left.list_size == right.list_size
    return isinstance(
        left,
        (
            pa.StructType,
            pa.ListType,
            pa.LargeListType,
            pa.ListViewType,
            pa.LargeListViewType,
            pa.RunEndEncodedType,
        ),
    )


def _extra_metadata(field: pa.Field) -> dict[bytes, bytes]:
    return {
        key: value
        for key, value in (field.metadata or {}).items()
        if key not in (_NAME, _METADATA)
    }


def _state(expected: pa.Field, actual: pa.Field) -> TransferState:
    if expected.equals(actual, check_metadata=True):
        return (
            "storage_only"
            if _extension(actual)[0] is not None
            and not isinstance(actual.type, pa.BaseExtensionType)
            else "retained"
        )
    # Python exposes metadata as a dict, which cannot preserve duplicate keys.
    # Only decompose a native inequality when that view is lossless; otherwise
    # an apparently matching extension descriptor could hide changed meaning.
    if any(
        not field.equals(field.with_metadata(field.metadata), check_metadata=True)
        for field in (expected, actual)
    ):
        return "mismatch"
    if not _same_shape(expected, actual) or _extra_metadata(
        expected
    ) != _extra_metadata(actual):
        return "mismatch"
    name, payload, known = _extension(expected)
    observed_name, observed_payload, observed_known = _extension(actual)
    if name != observed_name:
        return (
            "metadata_lost"
            if name is not None and observed_name is None
            else "mismatch"
        )
    if isinstance(expected.type, pa.BaseExtensionType) and isinstance(
        actual.type, pa.BaseExtensionType
    ):
        return (
            "retained" if expected.equals(actual, check_metadata=True) else "mismatch"
        )
    if not known or not observed_known or payload != observed_payload:
        return "mismatch"
    if name is not None and not isinstance(actual.type, pa.BaseExtensionType):
        return "storage_only"
    return "retained"


def _schema_metadata(schema: pa.Schema) -> pa.Schema:
    # Native projection preserves ordered duplicate KeyValueMetadata entries.
    return pa.Table.from_batches([], schema=schema).select([]).schema


def compare_schemas(
    source: pa.Schema, consumer: pa.Schema
) -> tuple[FieldTransfer, ...]:
    """Observe exact field semantics without inferring consumer registration.

    Full physical metadata is compared even when execution admission classifies
    some metadata as documentation. No observation grants value validity.
    """
    equal = source.equals(consumer, check_metadata=True)
    expected = _flatten(source)
    # Native dictionary equality omits nested value-field metadata at this pin.
    # Extension equality may also stop at its descriptor instead of storage fields.
    # Keep the full field walk for either container, including nested occurrences.
    equal = equal and not any(
        isinstance(_storage(field.type), pa.DictionaryType)
        or (isinstance(field.type, pa.BaseExtensionType) and _children(field.type))
        for field in expected.values()
    )
    observed = expected if equal else _flatten(consumer)
    result: dict[tuple[str, ...], FieldTransfer] = {}
    if not equal and (
        source.names != consumer.names
        or not _schema_metadata(source).equals(
            _schema_metadata(consumer), check_metadata=True
        )
    ):
        result[()] = FieldTransfer((), "mismatch", None)
    for path in dict.fromkeys((*expected, *observed)):
        field, actual = expected.get(path), observed.get(path)
        name = None if field is None else _extension(field)[0]
        state: TransferState
        if equal and actual is not None:
            state = (
                "storage_only"
                if name is not None
                and not isinstance(actual.type, pa.BaseExtensionType)
                else "retained"
            )
        else:
            state = (
                "mismatch" if field is None or actual is None else _state(field, actual)
            )
        result[path] = FieldTransfer(
            path,
            state,
            None if name is None else name.decode("utf-8", errors="replace"),
        )
    # Depth order also handles observed-only children appended after expected paths.
    for path in sorted(result, key=len, reverse=True):
        parent = result.get(path[:-1]) if path else None
        if (
            parent is not None
            and _SEVERITY[result[path].state] > _SEVERITY[parent.state]
        ):
            result[path[:-1]] = attrs.evolve(parent, state=result[path].state)
    return tuple(result.values())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreign reference wire support from the original generated descriptor world.

This module contains no scientific answers or candidate transformations. Reference
oracles independently construct their numerical outputs and use the generated
message class for binary interchange. Native scientific admission remains separate.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, TypeVar, cast

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.descriptor import FieldDescriptor
from google.protobuf.message import Message

if TYPE_CHECKING:
    from collections.abc import Iterable, Iterator

    from google.protobuf.descriptor import Descriptor

T = TypeVar("T")

MAXIMUM_BYTES = 16 << 20


def reference_type(descriptors: bytes) -> type[Message]:
    """Load one private descriptor world, including its complete declared imports."""
    if not descriptors or len(descriptors) > MAXIMUM_BYTES:
        raise ValueError("reference descriptors exceed the selected byte bound")
    files = descriptor_pb2.FileDescriptorSet.FromString(descriptors).file
    if not files or len(files) > 512:
        raise ValueError("reference descriptor file population is invalid")
    pending = {file.name: file for file in files}
    if len(pending) != len(files):
        raise ValueError("reference descriptor file names are not unique")
    pool = descriptor_pool.DescriptorPool()
    loaded: set[str] = set()
    while pending:
        ready = [
            name
            for name, file in pending.items()
            if all(dependency in loaded for dependency in file.dependency)
        ]
        if not ready:
            raise ValueError("reference descriptors have unresolved or cyclic imports")
        for name in ready:
            pool.AddSerializedFile(pending.pop(name).SerializeToString())
            loaded.add(name)
    return message_factory.GetMessageClass(
        pool.FindMessageTypeByName("pse.reference.v1.ReferenceEnvelope")
    )


def decode_reference(descriptors: bytes, payload: bytes) -> Message:
    """Bounded binary decode; unknown fields remain in the original message."""
    if len(payload) > MAXIMUM_BYTES:
        raise ValueError("reference envelope exceeds the selected byte bound")
    message = reference_type(descriptors)()
    message.ParseFromString(payload)
    return message


def encode_reference(message: Message) -> bytes:
    """Serialize the original binary interpretation without JSON normalization."""
    if message.DESCRIPTOR.full_name != "pse.reference.v1.ReferenceEnvelope":
        raise ValueError("a different message cannot supply a reference envelope")
    if message.ByteSize() > MAXIMUM_BYTES:
        raise ValueError("reference envelope exceeds the selected byte bound")
    return message.SerializeToString(deterministic=True)


def message_class(descriptor: object) -> type[Message]:
    """Use the protobuf factory for Python or accelerated native descriptors.

    The stubs name only the Python Descriptor, while the selected upb runtime
    exposes an equivalent descriptor through the same public factory contract.
    """
    return message_factory.GetMessageClass(cast("Descriptor", descriptor))


def message_field(parent: Message, name: str, *path: str) -> Message:
    """Access original singular message fields without imposing presence."""
    for part in (name, *path):
        field = parent.DESCRIPTOR.fields_by_name[part]
        if field.is_repeated or field.message_type is None:
            raise TypeError("selected field is not a singular protobuf message")
        parent = cast("Message", getattr(parent, part))
    return parent


def set_scalar(parent: Message, name: str, value: object) -> None:
    """Set a scalar through the original protobuf descriptor's validation."""
    field = parent.DESCRIPTOR.fields_by_name[name]
    if field.is_repeated or field.message_type is not None:
        raise TypeError("selected field is not a singular protobuf scalar")
    setattr(parent, name, value)


def scalar_field(parent: Message, name: str, kind: type[T]) -> T:
    """Read an original scalar, checking its actual runtime representation."""
    field = parent.DESCRIPTOR.fields_by_name[name]
    if field.is_repeated or field.message_type is not None:
        raise TypeError("selected field is not a singular protobuf scalar")
    result: object = getattr(parent, name)
    if not isinstance(result, kind) or (kind is int and isinstance(result, bool)):
        raise TypeError("protobuf scalar has a different runtime representation")
    return result


class RepeatedField(Protocol[T]):
    """The consumed public interface of original repeated protobuf containers."""

    def __iter__(self) -> Iterator[T]:
        """Iterate the original ordered field population."""
        ...

    def __len__(self) -> int:
        """Count the original field population."""
        ...

    def __getitem__(self, index: int) -> T:
        """Select one original field element."""
        ...

    def __delitem__(self, index: int | slice) -> None:
        """Delete selected original field elements in mutation controls."""
        ...

    def add(self, **fields: object) -> T:
        """Construct a new message in this container's descriptor world."""
        ...

    def extend(self, values: Iterable[T]) -> None:
        """Append values through the original protobuf container."""
        ...

    def reverse(self) -> None:
        """Reverse the original population in a correspondence mutation control."""
        ...

    def append(self, value: T) -> None:
        """Append one value through the original protobuf container."""
        ...


def repeated_field(parent: Message, name: str, kind: type[T]) -> RepeatedField[T]:
    """Read a repeated field, checking its actual element representation."""
    field = parent.DESCRIPTOR.fields_by_name[name]
    if not field.is_repeated:
        raise TypeError("selected field is not a repeated protobuf field")
    # The descriptor establishes element kind even for an empty population.
    if field.message_type is not None:
        valid_kind = issubclass(kind, Message)
    else:
        scalar_kind = {
            FieldDescriptor.TYPE_BOOL: bool,
            FieldDescriptor.TYPE_STRING: str,
            FieldDescriptor.TYPE_BYTES: bytes,
            FieldDescriptor.TYPE_FLOAT: float,
            FieldDescriptor.TYPE_DOUBLE: float,
        }.get(field.type, int)
        valid_kind = kind is scalar_kind
    if not valid_kind:
        raise TypeError("protobuf repeated field has a different declared element kind")
    result = cast("RepeatedField[object]", getattr(parent, name))
    if not all(
        isinstance(item, kind) and not (kind is int and isinstance(item, bool))
        for item in result
    ):
        raise TypeError(
            "protobuf repeated field has a different element representation"
        )
    return cast("RepeatedField[T]", result)

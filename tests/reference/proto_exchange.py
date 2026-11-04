# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreign reference wire support from the original generated descriptor world.

This module contains no scientific answers or candidate transformations. Reference
oracles independently construct their numerical outputs and use the generated
message class for binary interchange. Native scientific admission remains separate.
"""

from __future__ import annotations

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import Message

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

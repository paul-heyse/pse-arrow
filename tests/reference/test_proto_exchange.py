# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent foreign-runtime controls over the generated reference contract."""

from pathlib import Path
import unittest

from google.protobuf import descriptor_pb2
from google.protobuf.message import DecodeError

from proto_exchange import decode_reference, encode_reference, reference_type

DESCRIPTORS = (
    Path(__file__).resolve().parents[2]
    / "crates/pse-runtime/src/generated/reference-exchange/descriptor.bin"
)


def fixed64_bits_presence_and_future_binary_fields_survive_foreign_runtime(
    test: unittest.TestCase,
) -> None:
    descriptors = DESCRIPTORS.read_bytes()
    # Independently specified wire: output field6 fixed64 is the IEEE negative
    # zero bit pattern; optional basis field4 is explicitly present with nil ID.
    observable = b"\x32"  # Wrong wire type intentionally refuses below.
    with test.assertRaises(DecodeError):
        decode_reference(descriptors, observable)
    output = b"\x22\x10" + bytes(16) + b"\x31" + (1 << 63).to_bytes(8, "little")
    completed = b"\x0a" + bytes([len(output)]) + output
    outcome = b"\x0a" + bytes([len(completed)]) + completed
    unknown = b"\xc0\x3e\x07"  # field1000 varint7
    wire = b"\x08\x01\x22" + bytes([len(outcome)]) + outcome + unknown
    message = decode_reference(descriptors, wire)
    fields = dict(message.ListFields())
    outcome_message = fields[message.DESCRIPTOR.fields_by_name["outcome"]]
    completed_message = dict(outcome_message.ListFields())[
        outcome_message.DESCRIPTOR.fields_by_name["completed"]
    ]
    observations = dict(completed_message.ListFields())[
        completed_message.DESCRIPTOR.fields_by_name["outputs"]
    ]
    observation = observations[0]
    assert observation.HasField("basis")
    assert not observation.HasField("datum")
    values = dict(observation.ListFields())
    assert values[observation.DESCRIPTOR.fields_by_name["value"]] == 1 << 63
    assert unknown in encode_reference(message)
    second = decode_reference(descriptors, encode_reference(message))
    assert encode_reference(second) == encode_reference(message)
    assert type(message).FromString(encode_reference(message)) == message


def private_descriptor_worlds_and_incomplete_imports_remain_distinct(
    test: unittest.TestCase,
) -> None:
    descriptors = DESCRIPTORS.read_bytes()
    assert reference_type(descriptors) is not reference_type(descriptors)
    missing = descriptor_pb2.FileDescriptorSet.FromString(descriptors)
    selected = [file for file in missing.file if file.name == "reference.proto"]
    assert len(selected) == 1
    incomplete = descriptor_pb2.FileDescriptorSet()
    incomplete.file.add().CopyFrom(selected[0])
    with test.assertRaisesRegex(ValueError, "unresolved"):
        reference_type(incomplete.SerializeToString())


class ForeignProtoExchangeTests(unittest.TestCase):
    """The isolated pinned interpreter supplies the foreign implementation."""

    def test_bits_presence_and_unknown_fields(self) -> None:
        fixed64_bits_presence_and_future_binary_fields_survive_foreign_runtime(self)

    def test_descriptor_worlds_and_imports(self) -> None:
        private_descriptor_worlds_and_incomplete_imports_remain_distinct(self)

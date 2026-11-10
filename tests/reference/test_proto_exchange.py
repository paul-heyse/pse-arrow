# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent foreign-runtime controls over the generated reference contract."""

import unittest
from pathlib import Path

from google.protobuf import descriptor_pb2
from google.protobuf.message import DecodeError, Message

from proto_exchange import (
    decode_reference,
    encode_reference,
    message_class,
    message_field,
    reference_type,
    repeated_field,
    scalar_field,
    set_scalar,
)

DESCRIPTORS = (
    Path(__file__).resolve().parents[2]
    / "crates/pse-runtime/src/generated/reference-exchange/descriptor.bin"
)


def fixed64_bits_presence_and_future_binary_fields_survive_foreign_runtime(
    test: unittest.TestCase,
) -> None:
    """Fixed64 bits presence and future binary fields survive foreign runtime."""
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
    outcome_message = message_field(message, "outcome")
    completed_message = message_field(outcome_message, "completed")
    observation = repeated_field(completed_message, "outputs", Message)[0]
    assert observation.HasField("basis")
    assert not observation.HasField("datum")
    assert scalar_field(observation, "value", int) == 1 << 63
    assert unknown in encode_reference(message)
    second = decode_reference(descriptors, encode_reference(message))
    assert encode_reference(second) == encode_reference(message)
    assert type(message).FromString(encode_reference(message)) == message


def private_descriptor_worlds_and_incomplete_imports_remain_distinct(
    test: unittest.TestCase,
) -> None:
    """Private descriptor worlds and incomplete imports remain distinct."""
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
        """Bits presence and unknown fields."""
        fixed64_bits_presence_and_future_binary_fields_survive_foreign_runtime(self)

    def test_descriptor_worlds_and_imports(self) -> None:
        """Descriptor worlds and imports."""
        private_descriptor_worlds_and_incomplete_imports_remain_distinct(self)

    def test_degenerate_certificate_retains_unreported_and_explicit_false(self) -> None:
        """Decode independent support alternatives and exact multiplier bits."""
        envelope = reference_type(DESCRIPTORS.read_bytes())
        pool = envelope.DESCRIPTOR.file.pool
        certificate = message_class(
            pool.FindMessageTypeByName(
                "pse.reference.v1.ReferenceDegenerateCertificate"
            )
        )
        unreported = certificate.FromString(b"\x12\x00")
        assert unreported.WhichOneof("kind") == "unreported"
        assert certificate().WhichOneof("kind") is None

        # The repeated fixed64 field is packed. A signed zero remains a distinct
        # bit pattern and false is explicitly present in the Reported alternative.
        bits = [1 << 63, 0x3FF0000000000000]
        packed = b"".join(value.to_bytes(8, "little") for value in bits)
        reported = certificate.FromString(b"\x0a\x14\x0a\x10" + packed + b"\x10\x00")
        assert reported.WhichOneof("kind") == "reported"
        assert (
            list(
                repeated_field(message_field(reported, "reported"), "multipliers", int)
            )
            == bits
        )
        assert message_field(reported, "reported").HasField("irreducible_at_tolerance")
        assert not scalar_field(
            message_field(reported, "reported"), "irreducible_at_tolerance", bool
        )

    def test_reflection_checks_field_shape_and_preserves_absence(self) -> None:
        """Keep reflection typed without materializing optional message presence."""
        envelope = reference_type(DESCRIPTORS.read_bytes())(version=1)
        self.assertEqual(scalar_field(envelope, "version", int), 1)
        self.assertFalse(envelope.HasField("outcome"))
        outcome = message_field(envelope, "outcome")
        self.assertFalse(envelope.HasField("outcome"))
        self.assertIsNone(outcome.WhichOneof("kind"))
        with self.assertRaises(TypeError):
            message_field(envelope, "version")
        with self.assertRaises(TypeError):
            scalar_field(envelope, "version", str)
        with self.assertRaises(TypeError):
            scalar_field(envelope, "outcome", int)
        with self.assertRaises(TypeError):
            repeated_field(envelope, "outcome", Message)
        with self.assertRaises(TypeError):
            set_scalar(envelope, "outcome", b"")
        required = repeated_field(envelope, "required", bytes)
        with self.assertRaises(TypeError):
            repeated_field(envelope, "required", Message)
        with self.assertRaises(TypeError):
            repeated_field(envelope, "required", int)
        quantity_kind = message_class(
            envelope.DESCRIPTOR.file.pool.FindMessageTypeByName(
                "pse.reference.v1.PhysicalQuantityKind"
            )
        )(extensive=True)
        self.assertTrue(scalar_field(quantity_kind, "extensive", bool))
        with self.assertRaises(TypeError):
            scalar_field(quantity_kind, "extensive", int)
        required.extend((bytes(16), b"\x80" * 16))
        self.assertEqual(list(required), [bytes(16), b"\x80" * 16])
        with self.assertRaises(TypeError):
            repeated_field(envelope, "required", Message)
        encoded = encode_reference(envelope)
        self.assertFalse(envelope.HasField("outcome"))
        self.assertEqual(
            encode_reference(decode_reference(DESCRIPTORS.read_bytes(), encoded)),
            encoded,
        )

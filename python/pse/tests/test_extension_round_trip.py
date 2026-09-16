# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Extension types survive IPC, and degrade visibly when unregistered.

The Rust -> Python -> Rust round trip of blueprint §24.1 needs the native
boundary, which lands in phase 1. What phase 0 can prove is the asymmetry §4.4
names: the ``ARROW:extension:name`` key survives an IPC round trip either way,
but the *type* only reconstructs when the name is registered. That asymmetry is
why the contract layer asserts per column rather than trusting the type.
"""

import pyarrow as pa
import pyarrow.ipc
import pytest

from pse.contracts.extension_types import (
    EXTENSION_NAMES,
    PseBound,
    PseEnum,
    PseSemanticId,
    PseSourceSpan,
)

#: A name deliberately absent from the registry, to observe the degradation.
UNREGISTERED_NAME = "pse.not_a_real_extension"

EXTENSION_NAME_KEY = b"ARROW:extension:name"


def _round_trip(schema: pa.Schema) -> pa.Schema:
    sink = pa.BufferOutputStream()
    with pa.ipc.new_stream(sink, schema):
        pass
    with pa.ipc.open_stream(sink.getvalue()) as reader:
        return reader.schema


@pytest.mark.unit
def test_every_declared_name_is_registered() -> None:
    assert len(set(EXTENSION_NAMES)) == len(EXTENSION_NAMES)
    assert all(name.startswith("pse.") for name in EXTENSION_NAMES)


@pytest.mark.unit
def test_registered_type_reconstructs() -> None:
    schema = pa.schema([pa.field("sid", PseSemanticId())])
    restored = _round_trip(schema)
    field = restored.field("sid")
    assert isinstance(field.type, PseSemanticId)
    assert field.type.extension_name == "pse.semantic_id"
    assert field.type.storage_type == pa.binary(16)


@pytest.mark.unit
def test_bound_enum_child_reconstructs_with_its_declared_domain() -> None:
    declared = PseBound()
    restored = _round_trip(pa.schema([pa.field("bound", declared)])).field("bound")
    assert isinstance(restored.type, PseBound)
    kind = restored.type.storage_type.field("kind")
    assert isinstance(kind.type, PseEnum)
    assert kind.type.storage_type == pa.utf8()
    expected = declared.storage_type.field("kind")
    assert kind.equals(expected, check_metadata=True)
    assert kind.metadata is not None
    assert kind.metadata[b"pse.semantic.enum"].decode() == kind.type.binding_id

    forged = pa.struct([kind.with_metadata({}), declared.storage_type.field("value")])
    with pytest.raises(ValueError, match="storage"):
        PseBound.__arrow_ext_deserialize__(forged, b'{"v":1}')

    other_domain = kind.with_type(PseEnum("01" * 16))
    forged = pa.struct([other_domain, declared.storage_type.field("value")])
    with pytest.raises(ValueError, match="storage"):
        PseBound.__arrow_ext_deserialize__(forged, b'{"v":1}')


@pytest.mark.unit
def test_enum_extension_identity_includes_its_parameter() -> None:
    one = PseEnum("01" * 16)
    same = PseEnum("01" * 16)
    other = PseEnum("02" * 16)
    assert one == same
    assert one != other
    assert len({one, same, other}) == 2
    assert not pa.field("enum", one).equals(pa.field("enum", other))


@pytest.mark.unit
def test_source_span_signed_storage_preserves_its_declared_bounds() -> None:
    declared = PseSourceSpan()
    restored = _round_trip(pa.schema([pa.field("span", declared)])).field("span")
    assert isinstance(restored.type, PseSourceSpan)
    for name in ("start", "end"):
        field = restored.type.storage_type.field(name)
        assert field.type == pa.int64()
        assert field.metadata is not None
        assert field.metadata[b"pse.semantic.integer_range"] == b"[0,4294967295]"
    fields = list(declared.storage_type)
    fields[1] = fields[1].with_metadata({})
    with pytest.raises(ValueError, match="storage"):
        PseSourceSpan.__arrow_ext_deserialize__(pa.struct(fields), b'{"v":1}')


@pytest.mark.unit
def test_unregistered_name_degrades_to_storage_but_keeps_metadata() -> None:
    storage = pa.binary(16)
    schema = pa.schema(
        [
            pa.field(
                "sid",
                storage,
                metadata={EXTENSION_NAME_KEY: UNREGISTERED_NAME.encode()},
            )
        ]
    )
    restored = _round_trip(schema)
    field = restored.field("sid")
    assert field.type == storage, "an unregistered pse.* type must not reconstruct"
    assert field.metadata is not None
    assert field.metadata[EXTENSION_NAME_KEY] == UNREGISTERED_NAME.encode()


@pytest.mark.unit
def test_metadata_generation_is_checked() -> None:
    with pytest.raises(ValueError, match="generation"):
        PseSemanticId.__arrow_ext_deserialize__(pa.binary(16), b'{"v":99}')


@pytest.mark.unit
def test_storage_type_mismatch_is_refused() -> None:
    with pytest.raises(ValueError, match=r"pse\.semantic_id"):
        PseSemanticId.__arrow_ext_deserialize__(pa.binary(32), b'{"v":1}')

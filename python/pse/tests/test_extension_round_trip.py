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

from pse.contracts.extension_types import EXTENSION_NAMES, PseSemanticId

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

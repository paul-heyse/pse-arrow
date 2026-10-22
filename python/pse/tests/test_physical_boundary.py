# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public physical transport imports actual registry schemas and owns input custody."""

from typing import cast

import pyarrow as pa
import pytest

import pse
from pse.contracts.enums import PhysicalInputKind
from pse.contracts.values import SemanticId
from pse.tests.physical_input_fixture import coordinate_batch


@pytest.mark.unit
def test_physical_input_reflection_retains_complete_schema_metadata(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    identities: set[bytes] = set()
    for kind in PhysicalInputKind:
        schema = pa.schema(runtime.physical_input_schema(kind))
        assert schema.metadata is not None
        assert schema.metadata[b"pse.contract.fingerprint"]
        identities.add(schema.metadata[b"pse.contract.id"])
        roundtrip = pa.ipc.read_schema(pa.BufferReader(schema.serialize()))
        assert roundtrip.equals(schema, check_metadata=True)
    assert len(identities) == len(PhysicalInputKind)
    with pytest.raises(TypeError):
        runtime.physical_input_schema(cast(PhysicalInputKind, "state_tolerances"))


@pytest.mark.unit
def test_simulation_physical_boundary_refuses_metadata_loss_and_duplicate_keys(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    empty = coordinate_batch(runtime)
    def settings(atol: pa.RecordBatch) -> pse.SimulationSettings:
        return pse.SimulationSettings(runtime=runtime, start=-0.125, end=0.125,
                                      samples=[-0.125, 0.125], parameter_scales=empty, atol=atol)
    with pytest.raises(pse.InspectionError):
        settings(empty.replace_schema_metadata(None))
    coordinate = SemanticId.from_hex("01010101010101010101010101010101")
    duplicate = coordinate_batch(runtime, [(coordinate, 1.0, "Scalar"), (coordinate, 2.0, "Scalar")])
    with pytest.raises(pse.InspectionError, match="duplicate"):
        settings(duplicate)
    admitted = settings(empty)
    assert admitted.document.version == 1
    assert admitted.document.controls.start == -0.125
    assert admitted.document.controls.end == 0.125

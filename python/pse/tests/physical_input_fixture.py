# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Physical test inputs use registered schemas and the authored primitives fixture."""

from collections.abc import Sequence
from pathlib import Path

import msgspec
import pyarrow as pa

import pse
from pse import codec
from pse.contracts.enums import NativePrimalCoordinateRole, PhysicalInputKind
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

Coordinate = tuple[SemanticId, float, str]


def primitive_quantity(name: str) -> tuple[SemanticId, SemanticId]:
    """Read quantity and canonical-unit identities from the actual source fixture."""
    root = Path(__file__).resolve().parents[3] / "tests/fixtures/packages/physical-primitives/materials"
    found: list[tuple[SemanticId, SemanticId]] = []
    for path in root.glob("*.yaml"):
        source = "\n".join(line for line in path.read_text().splitlines() if not line.lstrip().startswith("#"))
        document = codec.decode_json(source, dict[str, msgspec.Raw])
        for row in codec.decode_json(bytes(document["quantity_types"]), list[dict[str, msgspec.Raw]]):
            if codec.decode_json(bytes(row["name"]), str) == name:
                found.append((
                    SemanticId.from_hex(codec.decode_json(bytes(row["quantity_type_id"]), str)),
                    SemanticId.from_hex(codec.decode_json(bytes(row["canonical_unit_id"]), str)),
                ))
    (meaning,) = found
    return meaning


def member(package: pse.ModelingPackage, case: DeclarationId, name: str) -> SemanticId:
    """Select one explicitly named source member, refusing ambiguous fixture paths."""
    (selected,) = (
        value.id for value in package.inspect(case, pse.SolveSettings()).members
        if value.lineage.path.rsplit(".", 1)[-1].split("[", 1)[0] == name
    )
    return selected


def _coordinate(id_: SemanticId, value: float, quantity: str) -> dict[str, object]:
    quantity_id, unit_id = primitive_quantity(quantity)
    return {"coordinate_id": bytes(id_), "value": {
        "value": value, "quantity_type_id": bytes(quantity_id), "unit_id": bytes(unit_id),
    }}


def coordinate_batch(
    runtime: pse.Runtime,
    coordinates: Sequence[Coordinate] = (),
    *,
    kind: PhysicalInputKind = PhysicalInputKind.STATE_TOLERANCES,
) -> pa.RecordBatch:
    """Construct physical coordinate rows using the complete native registry schema."""
    schema = pa.schema(runtime.physical_input_schema(kind))
    rows = [_coordinate(*coordinate) for coordinate in coordinates]
    if kind is PhysicalInputKind.PRIMAL_VALUES:
        for row in rows:
            row["coordinate_role"] = NativePrimalCoordinateRole.ORIGINAL.value
    return pa.RecordBatch.from_pylist(rows, schema=schema)


def stage_batch(runtime: pse.Runtime, stages: Sequence[Sequence[Coordinate]]) -> pa.RecordBatch:
    """Retain ordered sparse stages, including explicitly empty stages."""
    schema = pa.schema(runtime.physical_input_schema(PhysicalInputKind.INITIALIZATION_STAGES))
    return pa.RecordBatch.from_pylist([
        {"stage": stage, "values": [_coordinate(*coordinate) for coordinate in values]}
        for stage, values in enumerate(stages)
    ], schema=schema)


def sample_batch(
    runtime: pse.Runtime, samples: Sequence[tuple[SemanticId, Sequence[Coordinate]]],
) -> pa.RecordBatch:
    """Retain explicit sample identity and sparse physical source coordinates."""
    schema = pa.schema(runtime.physical_input_schema(PhysicalInputKind.DIAGNOSTIC_SAMPLES))
    return pa.RecordBatch.from_pylist([
        {"sample_id": bytes(sample), "values": [_coordinate(*coordinate) for coordinate in values]}
        for sample, values in samples
    ], schema=schema)

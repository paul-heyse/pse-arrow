# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Physical test inputs use registered schemas and the authored primitives fixture."""

from collections.abc import Sequence
from pathlib import Path

import pyarrow as pa

import pse
from pse import codec
from pse.contracts import documents
from pse.contracts.enums import NativePrimalCoordinateRole, PhysicalInputKind
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

Coordinate = tuple[SemanticId, float, str]


def inspection_request(
    case: DeclarationId, settings: pse.SolveSettings
) -> documents.ModelingInspectionRequest:
    """Explicit finite policy for the small authored fixtures."""
    return documents.ModelingInspectionRequest(
        case=case,
        preparation=documents.PreparationSettingsBody(
            compiler=documents.StudyCompilerProfile(
                class_proof_work=1_000_000,
                assembly=documents.AssemblyLimits(
                    contributions=1_000_000,
                    native_index=2_147_483_647,
                    worker_bytes=1 << 30,
                ),
                optimization=documents.Optimization(
                    cores=1,
                    horner_iterations=2,
                    cpe_iterations=1,
                    evaluator=documents.EvaluatorInterpreted(),
                ),
                evaluation=documents.EvaluationLimits(
                    derivative_components=4096,
                    operations=1_000_000,
                    scratch_bytes=8 << 20,
                    provider_calls=4096,
                ),
            ),
            limits=documents.Limits(depth=64, items=100_000, members=100_000),
        ),
        settings=settings,
        offset=0,
        limit=128,
    )


def _primitive_context(runtime: pse.Runtime) -> pse.PhysicalContext:
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    return runtime.physical_from_documents(
        {
            str(path.relative_to(root)): path.read_bytes()
            for path in root.rglob("*")
            if path.is_file()
        }
    )


def primitive_quantity(
    runtime: pse.Runtime, name: str
) -> tuple[SemanticId, SemanticId]:
    """Resolve fixture meaning through the admitted physical API."""
    return _quantity(_primitive_context(runtime), name)


def _quantity(
    physical: pse.PhysicalContext, name: str
) -> tuple[SemanticId, SemanticId]:
    page = physical.describe(
        codec.convert_document(
            {
                "document_version": 1,
                "query": {"kind": "quantity_named", "name": name},
                "offset": 0,
                "limit": 1,
            },
            documents.PhysicalInspectionRequest,
        )
    )
    if not isinstance(page, documents.PhysicalInspectionDocumentPage):
        message = "physical quantity description unavailable"
        raise TypeError(message)
    (meaning,) = page.quantities
    return meaning.quantity.id, meaning.canonical_unit.id


def member(package: pse.ModelingPackage, case: DeclarationId, name: str) -> SemanticId:
    """Select one explicitly named source member, refusing ambiguous fixture paths."""
    (selected,) = (
        value.id
        for value in package.inspect(
            inspection_request(case, pse.SolveSettings())
        ).members
        if value.lineage.path.rsplit(".", 1)[-1].split("[", 1)[0] == name
    )
    return selected


def _coordinate(
    meaning: tuple[SemanticId, SemanticId], id_: SemanticId, value: float
) -> dict[str, object]:
    quantity_id, unit_id = meaning
    return {
        "coordinate_id": bytes(id_),
        "value": {
            "value": value,
            "quantity_type_id": bytes(quantity_id),
            "unit_id": bytes(unit_id),
        },
    }


def coordinate_batch(
    runtime: pse.Runtime,
    coordinates: Sequence[Coordinate] = (),
    *,
    kind: PhysicalInputKind = PhysicalInputKind.STATE_TOLERANCES,
) -> pa.RecordBatch:
    """Construct physical coordinate rows using the complete native registry schema."""
    schema = pa.schema(runtime.physical_input_schema(kind))
    physical = _primitive_context(runtime) if coordinates else None
    meanings = (
        {
            name: _quantity(physical, name)
            for name in {coordinate[2] for coordinate in coordinates}
        }
        if physical is not None
        else {}
    )
    rows = [_coordinate(meanings[name], id_, value) for id_, value, name in coordinates]
    if kind is PhysicalInputKind.PRIMAL_VALUES:
        for row in rows:
            row["coordinate_role"] = NativePrimalCoordinateRole.ORIGINAL.value
    return pa.RecordBatch.from_pylist(rows, schema=schema)


def _coordinate_lists(
    runtime: pse.Runtime, groups: Sequence[Sequence[Coordinate]], type_: pa.DataType
) -> pa.ListArray:
    """Build each original extension child before nesting, retaining registry fields."""
    if not isinstance(type_, pa.ListType) or not isinstance(
        type_.value_type, pa.StructType
    ):
        message = "physical coordinate groups require the declared list of structs"
        raise TypeError(message)
    names = {coordinate[2] for group in groups for coordinate in group}
    physical = _primitive_context(runtime) if names else None
    meanings = (
        {name: _quantity(physical, name) for name in names}
        if physical is not None
        else {}
    )
    rows = [
        _coordinate(meanings[name], id_, value)
        for group in groups
        for id_, value, name in group
    ]
    struct = type_.value_type
    values = pa.StructArray.from_arrays(
        [
            pa.array([row[field.name] for row in rows], type=field.type)
            for field in struct
        ],
        type=struct,
    )
    offsets = [0]
    for group in groups:
        offsets.append(offsets[-1] + len(group))
    return pa.ListArray.from_arrays(
        pa.array(offsets, type=pa.int32()), values, type=type_
    )


def stage_batch(
    runtime: pse.Runtime, stages: Sequence[Sequence[Coordinate]]
) -> pa.RecordBatch:
    """Retain ordered sparse stages, including explicitly empty stages."""
    schema = pa.schema(
        runtime.physical_input_schema(PhysicalInputKind.INITIALIZATION_STAGES)
    )
    return pa.RecordBatch.from_arrays(
        [
            pa.array(range(len(stages)), type=schema.field("stage").type),
            _coordinate_lists(runtime, stages, schema.field("values").type),
        ],
        schema=schema,
    )


def sample_batch(
    runtime: pse.Runtime,
    samples: Sequence[tuple[SemanticId, Sequence[Coordinate]]],
) -> pa.RecordBatch:
    """Retain explicit sample identity and sparse physical source coordinates."""
    schema = pa.schema(
        runtime.physical_input_schema(PhysicalInputKind.DIAGNOSTIC_SAMPLES)
    )
    return pa.RecordBatch.from_arrays(
        [
            pa.array(
                [bytes(sample) for sample, _values in samples],
                type=schema.field("sample_id").type,
            ),
            _coordinate_lists(
                runtime,
                [values for _sample, values in samples],
                schema.field("values").type,
            ),
        ],
        schema=schema,
    )

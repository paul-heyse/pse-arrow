# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Generated physical points are ordinary local study authoring inputs."""

import math

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts import documents
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.study_fixtures import point
from pse.tests.study_fixtures import request as study_request
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents
from pse.tests.test_studies import _assert_scalar_root, _package


@pytest.mark.unit
@pytest.mark.parametrize("kind", ["grid", "classic"])
def test_design_generation_is_local_and_exports_realized_points(
    inspection_settings: pse.EngineSettings, kind: str
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        physical = runtime.physical_from_documents(physical_documents())
        package = runtime.modeling_from_documents(
            [
                package_documents("""package design { def Root {
                var x:Scalar; annotation bounds x(0,1); eq root:x==0.5;
            } }""")
            ],
            physical,
        )
        root = next(
            row.declaration_id for row in package.declarations() if row.name == "Root"
        )
        request = codec.decode_json(
            msgspec.json.encode(
                {
                    "root": root.to_hex(),
                    "coordinates": [
                        {
                            "target": {"kind": "path", "value": "x"},
                            "quantity": physical.resolve_quantity("Scalar").to_hex(),
                            "unit": physical.resolve_unit("dimensionless").to_hex(),
                            "range": [0.0, 1.0],
                        }
                    ],
                    "kind": kind,
                    "count": 3,
                    "seed": 91,
                    "grid_levels": [3] if kind == "grid" else [],
                    "maximum_work": 1_000_000,
                }
            ),
            documents.DesignRequest,
        )
        design = package.generate_design(request)
        assert len(design) == 3
        exported = design.export()
        repeated = package.generate_design(request).export()
        assert exported == repeated
        assert len(exported.points) == 3
        values = [point.assignments[0].value.magnitude for point in exported.points]
        if kind == "grid":
            assert values == [0.0, 0.5, 1.0]
        else:
            assert sorted(min(int(value * 3), 2) for value in values) == [0, 1, 2]
        assert all(0 <= value <= 1 for value in values)
        assert exported.attribution.seed == 91
        assert package.canonical_revision is None
        assert not runtime.durable
        # The independent authoring document survives its native handle.
        del design
        assert [
            point.assignments[0].value.magnitude for point in exported.points
        ] == values
    finally:
        runtime.close()


@pytest.mark.integration
@pytest.mark.managed_primary
def test_generated_design_records_realized_points_for_a_new_runtime_replay(
    managed_observer_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    physical = package.physical_context
    request = codec.decode_json(
        msgspec.json.encode(
            {
                "root": case.to_hex(),
                "coordinates": [
                    {
                        "target": {"kind": "path", "value": "a"},
                        "quantity": physical.resolve_quantity("Scalar").to_hex(),
                        "unit": physical.resolve_unit("dimensionless").to_hex(),
                        "range": [4.0, 9.0],
                    }
                ],
                "kind": "grid",
                "count": 3,
                "seed": 91,
                "grid_levels": [3],
                "maximum_work": 1_000_000,
            }
        ),
        documents.DesignRequest,
    )
    design = package.generate_design(request)
    exported = design.export()
    expected = (4.0, 6.5, 9.0)
    assert (
        tuple(item.assignments[0].value.magnitude for item in exported.points)
        == expected
    )
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    points = tuple(
        msgspec.structs.replace(
            point(case, settings, 13 + index),
            overlay=codec.decode_json(
                codec.encode_json(overlay), documents.PointOverlay
            ),
        )
        for index, overlay in enumerate(exported.points)
    )
    local = package.admit_study(study_request(*points))
    recorded = runtime.record_study(local)
    hashes = tuple(item.binding_hash for item in recorded.points)
    assert len(set(hashes)) == 3
    assert (
        tuple(
            next(iter(item.binding.entries.values())).canonical
            for item in recorded.points
        )
        == expected
    )
    payload = codec.encode_json(recorded)
    # Replay uses the recorded realized values after the generating native owners close.
    del local, design, exported, package, physical
    runtime.clear_program_cache()
    runtime.close()
    replayed = codec.decode_json(payload, documents.StudyDefinition)
    assert tuple(item.binding_hash for item in replayed.points) == hashes
    receiver = canonical_substrate.runtime(managed_observer_settings)
    report = receiver.execute_study(replayed)
    assert report.count == 3
    assert report.unattempted == 0
    assert report.conclusion.availability == "complete"
    for index, value in enumerate(expected):
        assert report.outcome(index).key == 13 + index
        retained = report.result(index)
        assert isinstance(retained, pse.StoredResult)
        assert retained.usable
        table = pa.table(retained.table("runtime.solve_variables"))
        _assert_scalar_root(table, math.sqrt(value))
        assert [row["value"] for row in table.to_pylist() if row["parameter"]] == [
            value
        ]

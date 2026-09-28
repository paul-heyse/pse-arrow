# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Durable studies from Python (Plan 22 O7): stored, run by a worker, published once."""

import uuid
from datetime import timedelta
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.contracts.documents import PointOverlay
from pse.contracts.enums import (
    AttemptState,
    JobState,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
    StudyPointState,
    StudyState,
)
from pse.contracts.values import SemanticId

SOURCE = """package algebraic { def Root {
    param a:Scalar = 4;
    var x:Scalar;
    eq square:x*x==a;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
} }"""


def _package(runtime: pse.Runtime) -> tuple[pse.ModelingPackage, SemanticId]:
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents(
        {
            str(p.relative_to(primitives)): p.read_text()
            for p in primitives.rglob("*")
            if p.is_file()
        }
    )
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest += f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{SemanticId(bytes([31]) * 16).to_hex()}"\n'
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": SOURCE}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package, case


@pytest.mark.integration
def test_durable_study_publishes_once(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
    tmp_path: Path,
) -> None:
    runtime = pse.Runtime(inspection_settings, store=operational_store)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    workspace = runtime.register_workspace(f"study-{uuid.uuid4().hex}", tmp_path)
    handle = package.study(
        (case, case, case),
        settings,
        predecessors=(None, 0, None),
        overlays=(
            PointOverlay(),
            PointOverlay(values={"a": 9.0}),
            PointOverlay(values={"a": -1.0}),
        ),
        runtime=runtime,
        workspace=workspace,
    )
    status = handle.status()
    assert status.state == StudyState.OPEN
    assert [point.job_state for point in status.points] == [
        JobState.QUEUED,
        JobState.WAITING,
        JobState.QUEUED,
    ]
    assert handle.result() is None
    assert any(
        row.study_id == handle.study_id
        for row in runtime.studies(states=(StudyState.OPEN,))
    )

    # This process serves the queue: the points, then the study's finalization (and any
    # other job the shared store holds).
    assert runtime.work() >= 4
    published = handle.wait(timeout=timedelta(seconds=60))
    status = runtime.study(handle.study_id).status()
    assert status.state == StudyState.PUBLISHED
    assert status.attempt_state == AttemptState.PARTIAL
    assert [point.state for point in status.points] == [
        StudyPointState.COMPLETED,
        StudyPointState.COMPLETED,
        StudyPointState.FAILED,
    ]
    assert published.attempt_id == status.attempt_id
    assert runtime.head(workspace.id) == published.id

    publication = runtime.open(published.id)
    outcomes = pa.table(
        publication.table("study", "runtime", "study_outcomes")
    ).to_pylist()
    assert [
        row["member_catalog"]
        for row in sorted(outcomes, key=lambda row: row["point_index"])
    ] == [
        "point_0",
        "point_1",
        None,
    ]
    catalogs = {name.catalog for name in publication.tables()}
    assert catalogs == {"study", "point_0", "point_1"}


@pytest.mark.integration
def test_durable_study_cancel_and_its_refusals(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
    tmp_path: Path,
) -> None:
    runtime = pse.Runtime(inspection_settings, store=operational_store)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
    )
    workspace = runtime.register_workspace(f"study-cancel-{uuid.uuid4().hex}", tmp_path)
    handle = package.study(
        (case, case),
        settings,
        predecessors=(None, 0),
        overlays=(PointOverlay(), PointOverlay(values={"a": 9.0})),
        runtime=runtime,
        workspace=workspace,
    )
    cancelled = handle.cancel()
    assert cancelled.cancelled == (0, 1)
    assert cancelled.concluded
    status = handle.status()
    assert status.state == StudyState.CONCLUDED
    assert status.attempt_state == AttemptState.CANCELLED
    assert all(point.state == StudyPointState.CANCELLED for point in status.points)

    # Overlays select a durable study; a package changed in memory has no authored sources.
    with pytest.raises(ValueError, match="durable study"):
        package.study((case,), settings, overlays=(PointOverlay(),))
    with pytest.raises(pse.InspectionError, match="changed in memory"):
        package.with_limits(pse.ModelingLimits()).study(
            (case,), settings, runtime=runtime, workspace=workspace
        )

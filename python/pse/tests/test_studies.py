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
    NativeTermination,
    PresolvePolicyKind,
    StudyPointState,
    StudyState,
)
from pse.contracts.identities import DeclarationId
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


def _package(runtime: pse.Runtime) -> tuple[pse.ModelingPackage, DeclarationId]:
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
    manifest += (
        '\n[[quantity_aliases]]\nname = "Scalar"\n'
        f'quantity_type_id = "{SemanticId(bytes([31]) * 16).to_hex()}"\n'
    )
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

    # A workspace selects a durable study; a package changed in memory has no authored
    # sources.
    with pytest.raises(ValueError, match="durable study"):
        # pyrefly: ignore[no-matching-overload] -- the overloads reject this call; its
        # runtime refusal is what is under test
        package.study((case,), settings, workspace=workspace)
    with pytest.raises(pse.InspectionError, match="changed in memory"):
        package.with_limits(pse.ModelingLimits()).study(
            (case,), settings, runtime=runtime, workspace=workspace
        )


FLASH_BT_IDEAL = DeclarationId(SemanticId.from_hex("3ca89c2ae2704e21a11e3512bf3cdb7e"))


@pytest.mark.integration
def test_flash_sweep_prepares_structure_once(
    inspection_settings: pse.EngineSettings,
) -> None:
    """A feed-temperature sweep of the BT ideal flash changes values only (CT-S08)."""
    runtime = pse.Runtime(inspection_settings)
    reference = Path(__file__).resolve().parents[3] / "packages/reference"

    def documents(path: Path) -> dict[str, str]:
        return {
            p.relative_to(path).as_posix(): p.read_text()
            for p in path.rglob("*")
            if p.is_file() and p.suffix in {".toml", ".yaml", ".yml", ".pse"}
        }

    physical = runtime.physical_from_documents(documents(reference / "physical"))
    package = runtime.modeling_from_documents(
        [
            documents(reference / name)
            for name in ("seed-data", "process", "thermodynamics", "methods", "physical")
        ],
        physical,
    )
    temperatures = (366.0, 367.0, 368.0, 369.0)
    study = package.study(
        tuple(FLASH_BT_IDEAL for _ in temperatures),
        pse.SolveSettings(
            backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
        ),
        overlays=tuple(PointOverlay(values={"root.inlet.T": t}) for t in temperatures),
    )
    assert study.count == len(temperatures)
    values = []
    for index, temperature in enumerate(temperatures):
        result = study.result(index)
        assert result is not None, study.failure(index)
        # Every point converges; the fixture's IDAES expectations hold at 368 K only.
        attempt = result.attempt()
        assert attempt is not None
        assert attempt.termination == NativeTermination.SUCCESS, attempt.termination
        assert result.accepted == (temperature == 368.0), result.failure()
        values.append(attempt.primal())
    # Each point solved its own feed temperature ...
    assert values[0] != values[-1]
    # ... on one prepared structure: every later point only rebound its values.
    preparations = study.preparations
    assert preparations.views == 1, preparations
    assert preparations.rebuilt + preparations.shared == len(temperatures) - 1, (
        preparations
    )

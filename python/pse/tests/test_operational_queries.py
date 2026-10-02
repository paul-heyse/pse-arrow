# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The query surface over the operational store (Plan 22 O9).

Progress streams, job listings and SQL over ``pse_ops`` joined with results.
"""

import uuid
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.contracts.enums import (
    AttemptState,
    JobState,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.contracts.identities import DeclarationId
from pse.tests.study_fixtures import assignment, physical_ids, point, request

#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)

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
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": SOURCE}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package, case


def _settings() -> pse.SolveSettings:
    return pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )


@pytest.mark.integration
def test_progress_stream_python(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
) -> None:
    runtime = pse.Runtime(inspection_settings, store=operational_store)
    package, case = _package(runtime)
    handle = package.prepare_solve(case, _settings()).start()
    attempt = handle.attempt_id
    assert attempt is not None
    # Follow the live attempt: the stream ends once the attempt has ended and every
    # stored event was read.
    with runtime.progress(attempt, follow=True, page=4) as stream:
        assert stream.attempt_id == attempt
        followed = list(stream)
    result = handle.wait()
    assert result.usable
    (listed,) = runtime.runs(run_id=result.run_id)
    assert listed.state == AttemptState.COMPLETED
    assert followed, "a durable Ipopt solve streams its iterations"
    # Every stored event carries its position and its instant.
    positions = [event.sequence for event in followed if event.incumbent is None]
    sequences = [sequence for sequence in positions if sequence is not None]
    assert sequences == positions
    assert sequences == sorted(sequences)
    assert len(set(sequences)) == len(sequences)
    instants = [event.at for event in followed if event.at is not None]
    assert len(instants) == len(followed)
    assert instants == sorted(instants)
    for event in followed:
        assert event.step == 0
        assert event.phase
        assert event.elapsed_seconds >= 0
        assert isinstance(event.values(), dict)
        # An interior-point solve has no incumbents.
        assert event.incumbent is None
    # Without following, the stream reads what is stored and ends.
    stored = list(runtime.progress(attempt, follow=False))
    assert [(e.sequence, e.phase) for e in stored] == [
        (e.sequence, e.phase) for e in followed
    ]
    # The in-memory events of the handle carry no stored position.
    events, _ = handle.progress()
    assert all(event.step is None and event.sequence is None for event in events)
    # A closed stream reads nothing more.
    closed = runtime.progress(attempt, follow=False, page=1)
    closed.close()
    assert list(closed) == []


@pytest.mark.integration
def test_jobs_and_studies_listed_and_queried(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
    tmp_path: Path,
) -> None:
    runtime = pse.Runtime(inspection_settings, store=operational_store)
    package, case = _package(runtime)
    workspace = runtime.register_workspace(f"queries-{uuid.uuid4().hex}", tmp_path)
    scalar, one = physical_ids(
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives/materials/physical.yaml",
        "Scalar",
        "dimensionless",
    )
    definition = package.admit_study(
        request(
            point(case, _settings(), 0),
            point(
                case, _settings(), 1, assignments=(assignment("a", 9.0, scalar, one),)
            ),
        )
    )
    handle = package.study(definition, runtime=runtime, workspace=workspace)
    status = handle.status()
    point_attempts = {point.attempt_id for point in status.points}

    # The study's point jobs and its waiting finalization are listed, newest first.
    queued = runtime.jobs(states=(JobState.QUEUED,), limit=1000)
    assert point_attempts <= {job.attempt_id.to_hex() for job in queued}
    assert all(job.state == JobState.QUEUED for job in queued)
    newest = runtime.jobs(limit=1000)
    enqueued = [job.enqueued_at for job in newest]
    assert enqueued == sorted(enqueued, reverse=True)
    assert len(runtime.jobs(limit=1)) == 1
    assert any(study.study_id == handle.study_id for study in runtime.studies())

    # SQL over pse_ops: the study's points joined with their jobs and attempts.
    study_hex = handle.study_id.to_hex()
    points = pa.table(
        runtime.query(
            "SELECT p.point_index, j.state AS job_state, a.state AS attempt_state "  # noqa: S608 - literals of store-minted identities; query() binds no parameters
            "FROM pse_ops.study_points AS p "
            "JOIN pse_ops.jobs AS j ON j.job_id = p.job_id "
            "JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id "
            f"WHERE p.study_id = X'{study_hex}' ORDER BY p.point_index"
        )
    ).to_pylist()
    assert [row["point_index"] for row in points] == [0, 1]
    assert {row["job_state"] for row in points} == {"queued"}

    # Run the study here; its publication and every point's attempt are then
    # queryable.
    assert runtime.work() >= 3
    published = handle.wait()
    literals = ", ".join("X'" + attempt + "'" for attempt in sorted(point_attempts))
    done = pa.table(
        runtime.query(
            "SELECT count(*) AS completed FROM pse_ops.attempts "  # noqa: S608 - literals of store-minted identities; query() binds no parameters
            f"WHERE state = 'completed' AND attempt_id IN ({literals})"
        )
    ).to_pylist()
    assert done == [{"completed": 2}]
    (publication,) = pa.table(
        runtime.query(
            "SELECT p.publication_id, count(m.table_name) AS members "  # noqa: S608 - literals of store-minted identities; query() binds no parameters
            "FROM pse_ops.publications AS p "
            "JOIN pse_ops.publication_members AS m "
            "ON m.publication_id = p.publication_id "
            f"WHERE p.publication_id = X'{published.publication_id}' "
            "GROUP BY p.publication_id"
        )
    ).to_pylist()
    assert publication["members"] > 0

    # A run's retained results join its stored attempt.
    result = package.prepare_solve(case, _settings()).start().wait()
    assert result.attempt_id is not None
    joined = pa.table(
        runtime.query(
            "SELECT a.state, s.step FROM pse_ops.attempts AS a "  # noqa: S608 - literals of store-minted identities; query() binds no parameters
            "JOIN workspace.runtime.solve_runs AS s ON a.run_id = s.run_id "
            f"WHERE a.attempt_id = X'{result.attempt_id.to_hex()}'",
            result=result,
        )
    ).to_pylist()
    assert joined == [{"state": "completed", "step": 0}]


@pytest.mark.integration
def test_ephemeral_runtime_has_no_operational_tables(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    with pytest.raises(pse.InspectionError):
        pa.table(runtime.query("SELECT * FROM pse_ops.attempts"))
    with pytest.raises(pse.InspectionError):
        runtime.jobs()

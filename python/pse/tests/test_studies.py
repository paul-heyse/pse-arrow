# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable occurrences, scientific permissions and durable study retention."""

import math
import os
import sys
import time
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest
from scripts import thermodynamic_campaign

import pse
from pse import codec
from pse._scientific_inputs import family_packages, package_documents
from pse.contracts import documents
from pse.contracts.documents import (
    BoundaryDiagnosticOutput,
    CaptureBudget,
    ClosedDuration,
    ControllerOperation,
    DetailRequest,
    EvaluatorCompiledScalar,
    EvaluatorInterpreted,
    ObservationContracts,
    SelectedStudyVariant,
    SourceChoice,
    StudyDetailDemand,
    StudyInputsExplicit,
    StudyInputsLazy,
    StudyMemberDemandVariable,
    StudyOutputDemandCompact,
    StudyOutputDemandFull,
    StudyRequest,
    StudyVariants,
    WorkLimits,
)
from pse.contracts.enums import (
    DiagnosticCode,
    FeralOrdering,
    NativeBackend,
    NativeSolveIntent,
    NativeTermination,
    NumericalStartOrigin,
    PresolvePolicyKind,
    StudyPointState,
    StudyState,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import ContentHash, SemanticId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.managed_study_fixture import (
    CallerStarted,
    ManagedStudyFixture,
    NativeEntries,
    _read_object,
    _read_private,
    _write_private,
)
from pse.tests.study_fixtures import assignment, physical_ids, point
from pse.tests.study_fixtures import request as study_request
from pse.tests.test_native_workflow import declaration

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


@pytest.mark.unit
def test_campaign_constructs_current_explicit_study_input_variant() -> None:
    request = thermodynamic_campaign.explicit_study_request(())
    assert isinstance(request.inputs, StudyInputsExplicit)
    assert request.inputs.value == ()
    assert isinstance(request.output, StudyOutputDemandFull)


@pytest.mark.unit
@pytest.mark.parametrize("compiled", [False, True])
def test_current_study_request_roundtrip_preserves_explicit_evaluator(
    compiled: bool,
) -> None:
    selected = point(declaration(101), pse.SolveSettings(), 0)
    optimization = selected.preparation.compiler.optimization
    assert optimization is not None
    evaluator = (
        EvaluatorCompiledScalar(module_bytes=64 << 20)
        if compiled
        else EvaluatorInterpreted()
    )
    optimization = msgspec.structs.replace(optimization, evaluator=evaluator)
    compiler = msgspec.structs.replace(
        selected.preparation.compiler, optimization=optimization
    )
    preparation = msgspec.structs.replace(selected.preparation, compiler=compiler)
    selected = msgspec.structs.replace(selected, preparation=preparation)
    request = study_request(selected)
    assert request.version == 8
    assert codec.decode_json(codec.encode_json(request), StudyRequest) == request


def _assert_scalar_root(table: pa.Table, expected: float) -> None:
    """Assess the original root using its published production allowance."""
    variables = [row for row in table.to_pylist() if not row["parameter"]]
    assert len(variables) == 1
    value = variables[0]["value"]
    allowance = variables[0]["tolerance"]
    assert isinstance(value, float)
    assert math.isfinite(value)
    assert isinstance(allowance, float)
    assert math.isfinite(allowance)
    assert allowance > 0
    assert abs(value - expected) <= allowance


def _package(
    runtime: pse.Runtime, *, source: str = SOURCE
) -> tuple[pse.ModelingPackage, DeclarationId]:
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
        [{"package.toml": manifest, "models/root.pse": source}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package, case


def _assert_sixteen_held_native_owners(observation: NativeEntries) -> None:
    assert observation.active == observation.maximum == 16
    assert len(observation.entries) == 16
    assert len({entry.thread for entry in observation.entries}) == 16
    assert all(not entry.released for entry in observation.entries)
    assert all(not entry.stop_observed for entry in observation.entries)


def _run_managed_public_study(
    settings: pse.EngineSettings, state: CanonicalFixture, *, cancel: bool
) -> None:
    assert settings.memory_limit_bytes <= 4 << 30
    with ManagedStudyFixture(
        state.state,
        database=state.database,
        resource=state.resource,
        test=os.environ.get("PYTEST_CURRENT_TEST", "").removesuffix(" (call)"),
    ) as fixture:
        # Transfer the fixture's selected settings, never a second allocation.
        _write_private(
            fixture.directory / "caller-settings.json",
            {
                "state": str(fixture.state),
                "nonce": fixture.nonce,
                "database": fixture.database,
                "worker_pid": fixture.worker_pid,
                "observer_group": fixture.observer_group,
                "spill_dir": settings.spill_dir,
                "engine": {
                    key: getattr(settings, key)
                    for key in (
                        "memory_limit_bytes",
                        "threads",
                        "target_partitions",
                        "max_spill_bytes",
                        "batch_size",
                        "math_workspace_bytes",
                        "math_worker_bytes",
                        "math_artifact_bytes",
                    )
                },
                "cache": {
                    key: getattr(settings.cache, key)
                    for key in (
                        "working_bytes",
                        "concurrent_loads",
                        "inflight_bytes",
                        "inspection_bytes",
                    )
                },
            },
        )
        fixture.call(
            [
                sys.executable,
                "-c",
                (
                    "import sys; from pse.tests.test_studies "
                    "import _managed_public_study_caller; "
                    "_managed_public_study_caller(sys.argv[1], sys.argv[2])"
                ),
                str(fixture.directory),
                "cancel" if cancel else "complete",
            ],
            _assert_sixteen_held_native_owners,
            cancel=cancel,
        )
        drained = fixture.observation(lambda value: value.active == 0, timeout=0)
        assert drained.maximum == 16
        assert len(drained.entries) == 16
        assert all(
            entry.released and entry.stop_observed == cancel
            for entry in drained.entries
        )
        assert (fixture.directory / "release.json").exists() != cancel


def _managed_public_study_caller(directory: str, mode: str) -> None:
    """All public result assertions run in the supervised synchronous caller."""
    document = _read_object(Path(directory) / "caller-settings.json")
    engine = msgspec.convert(document["engine"], type=dict[str, int])
    cache = msgspec.convert(document["cache"], type=dict[str, int])
    settings = pse.EngineSettings(
        memory_limit_bytes=engine["memory_limit_bytes"],
        threads=engine["threads"],
        target_partitions=engine["target_partitions"],
        spill_dir=msgspec.convert(document["spill_dir"], type=str),
        max_spill_bytes=engine["max_spill_bytes"],
        batch_size=engine["batch_size"],
        math_workspace_bytes=engine["math_workspace_bytes"],
        math_worker_bytes=engine["math_worker_bytes"],
        math_artifact_bytes=engine["math_artifact_bytes"],
        cache=pse.CacheSettings(
            working_bytes=cache["working_bytes"],
            concurrent_loads=cache["concurrent_loads"],
            inflight_bytes=cache["inflight_bytes"],
            inspection_bytes=cache["inspection_bytes"],
        ),
    )
    assert settings.memory_limit_bytes <= 4 << 30
    fixture = ManagedStudyFixture(msgspec.convert(document["state"], type=str))
    fixture.directory = Path(directory)
    fixture.nonce = msgspec.convert(document["nonce"], type=str)
    fixture.database = msgspec.convert(document["database"], type=str)
    fixture.worker_pid = msgspec.convert(document["worker_pid"], type=int)
    group = next(
        line.removeprefix("0::")
        for line in Path("/proc/self/cgroup").read_text().splitlines()
        if line.startswith("0::")
    )
    assert group == document["observer_group"], (
        "public caller must inherit the aggregate observer group"
    )
    runtime = pse.Runtime(
        settings, substrate=str(fixture.state), database=fixture.database
    )
    package, case = _package(runtime)
    solve = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    assert mode in {"complete", "cancel"}
    keys = (
        tuple(7 + 4 * index for index in range(16))
        if mode == "complete"
        else tuple(3 + 4 * index for index in range(20))
    )
    definition = package.admit_study(
        study_request(*(point(case, solve, key) for key in keys))
    )
    assert len({item.binding_hash for item in definition.points}) == 1
    # The parent uses this original clock, including every result assertion.
    _write_private(
        fixture.directory / "calling.json",
        CallerStarted(fixture.nonce, time.monotonic()),
    )
    if mode == "complete":
        study = runtime.execute_study(runtime.record_study(definition))
        _assert_completed_managed_study(fixture, study, keys)
        return
    handle = runtime.start_study(runtime.record_study(definition))
    assert isinstance(handle, pse.StudyHandle)
    deadline = time.monotonic() + 90
    while not (fixture.directory / "cancel.json").exists():
        assert time.monotonic() < deadline, (
            "public cancellation authorization timed out"
        )
        time.sleep(0.02)
    assert (
        msgspec.json.decode(_read_private(fixture.directory / "cancel.json"), type=str)
        == fixture.nonce
    )
    _assert_cancelled_managed_study(fixture, runtime, handle, keys)


def _assert_completed_managed_study(
    fixture: ManagedStudyFixture, study: pse.StudyReport, keys: tuple[int, ...]
) -> None:
    assert isinstance(study, pse.StudyReport)
    assert study.count == 16
    assert study.unattempted == 0
    assert study.conclusion.availability == "complete"
    assert study.conclusion.lifecycle == "terminal"
    assert tuple(study.outcome(index).key for index in range(16)) == keys
    runs = set()
    attempts = set()
    for index in range(16):
        outcome = study.outcome(index)
        assert outcome.lifecycle == StudyPointState.COMPLETED
        assert outcome.scientific.usable
        assert len(outcome.attempts) == 1
        assert outcome.attempts[0].attempt_id is not None
        assert outcome.diagnostic is None
        assert study.failure(index) is None
        result = study.result(index)
        assert isinstance(result, pse.StoredResult)
        assert result.usable
        runs.add(result.run_id)
        attempts.add(result.attempt_key)
        (attempt,) = pa.table(result.attempt_record()).to_pylist()
        assert attempt["key"] == result.attempt_key
        assert attempt["run"] == result.run_key
        values = pa.table(result.table("runtime.solve_variables"))
        _assert_scalar_root(values, 2.0)
        assert [row["value"] for row in values.to_pylist() if row["parameter"]] == [4.0]
    assert len(runs) == len(attempts) == 16
    assert (
        tuple(row["point_index"] for row in pa.table(study.table()).to_pylist()) == keys
    )
    drained = fixture.observation(lambda value: value.active == 0)
    assert drained.maximum == 16
    assert len(drained.entries) == 16
    assert all(entry.released and not entry.stop_observed for entry in drained.entries)
    # The caller's preparation counters exclude the external primary's work.
    preparations = study.preparations
    assert (
        preparations.views,
        preparations.rebuilt,
        preparations.shared,
        preparations.observations,
    ) == (0, 0, 0, 0)


def _assert_cancelled_managed_study(
    fixture: ManagedStudyFixture,
    runtime: pse.Runtime,
    handle: pse.StudyHandle,
    keys: tuple[int, ...],
) -> None:
    sys.stderr.write("managed cancellation: reading admitted status\n")
    sys.stderr.flush()
    issued = handle.status()
    assert tuple(item.point_index for item in issued.points) == keys
    assert all(item.attempt is not None for item in issued.points[:16])
    assert all(item.attempt is None for item in issued.points[16:])
    sys.stderr.write("managed cancellation: requesting cancellation\n")
    sys.stderr.flush()
    cancelled = handle.cancel()
    assert not cancelled.already_concluded
    assert cancelled.study_id == handle.study_id
    # Do not release the barrier: the actual native stop flags must wake all
    # sixteen admitted owners, while the effect-free tail never enters it.
    sys.stderr.write("managed cancellation: waiting for native drain\n")
    sys.stderr.flush()
    drained = fixture.observation(lambda value: value.active == 0)
    assert drained.maximum == 16
    assert len(drained.entries) == 16
    assert all(entry.released and entry.stop_observed for entry in drained.entries)
    assert not (fixture.directory / "release.json").exists()
    sys.stderr.write("managed cancellation: waiting for terminal study\n")
    sys.stderr.flush()
    retained = handle.wait(controls=pse.StudyWaitControls(timeout_seconds=90))
    status = runtime.study(handle.study_id).status()
    assert status.state == StudyState.CONCLUDED
    assert status.cancelled
    assert tuple(item.point_index for item in status.points) == keys
    assert len({item.run for item in status.points}) == 20
    assert retained.run == status.run
    assert retained.attempt == status.result_attempt
    assert retained.points == tuple(
        (item.point_index, item.run, item.attempt) for item in status.points
    )
    for item in status.points:
        assert item.settled
        assert item.state == StudyPointState.CANCELLED
        assert item.outcome is not None
        assert item.outcome.key == item.point_index
        assert item.outcome.lifecycle == StudyPointState.CANCELLED
        assert not item.outcome.scientific.usable
    assert all(item.attempt is not None for item in status.points[:16])
    for item in status.points[16:]:
        assert item.attempt is None
        assert item.outcome is not None
        assert item.outcome.attempts == ()


@pytest.mark.integration
@pytest.mark.managed_primary
def test_public_study_sixteen_native_owners_retain_equal_occurrences_and_original_roots(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    """One ordinary Python call drives sixteen fresh non-batching Ipopt owners."""
    _run_managed_public_study(
        managed_observer_settings, canonical_substrate, cancel=False
    )


@pytest.mark.integration
@pytest.mark.managed_primary
def test_public_study_cancel_drains_sixteen_native_owners_and_never_starts_tail(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    """Cancel through the public durable handle after sixteen of twenty enter."""
    _run_managed_public_study(
        managed_observer_settings, canonical_substrate, cancel=True
    )


@pytest.mark.integration
@pytest.mark.managed_primary
def test_study_repeated_bindings_retain_distinct_occurrences_and_owned_results(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    definition = package.admit_study(
        study_request(point(case, settings, 7), point(case, settings, 11))
    )
    assert definition.points[0].binding_hash == definition.points[1].binding_hash
    assert [item.policy.key for item in definition.points] == [7, 11]
    with pytest.raises(TypeError):
        codec.encode_json(definition)
    recorded = runtime.record_study(definition)
    assert codec.decode_json(codec.encode_json(recorded), type(recorded)) == recorded
    study = runtime.execute_study(recorded)
    assert isinstance(study, pse.StudyReport)
    assert study.count == 2
    assert study.unattempted == 0
    assert study.conclusion.availability == "complete"
    assert study.conclusion.lifecycle == "terminal"
    first, second = (study.result(index) for index in range(2))
    assert isinstance(first, pse.StoredResult)
    assert isinstance(second, pse.StoredResult)
    assert first.usable
    assert second.usable
    assert first.run_id != second.run_id
    for index, key in enumerate((7, 11)):
        outcome = study.outcome(index)
        assert outcome.key == key
        assert outcome.lifecycle == StudyPointState.COMPLETED
        assert outcome.scientific.usable
        assert len(outcome.attempts) == 1
        assert outcome.attempts[0].attempt_id is not None
        retained = study.result(index)
        assert isinstance(retained, pse.StoredResult)
        (attempt,) = pa.table(retained.attempt_record()).to_pylist()
        assert attempt["key"] == retained.attempt_key
        assert attempt["run"] == retained.run_key
        assert outcome.diagnostic is None
        assert study.failure(index) is None
    history = study.table()
    values = first.table("runtime.solve_variables")
    del first, second, study, package, runtime
    rows = pa.table(history).to_pylist()
    assert [row["point_index"] for row in rows] == [7, 11]
    assert all(row["usable"] and len(row["attempts"]) == 1 for row in rows)
    retained_values = pa.table(values)
    variables = retained_values.to_pylist()
    _assert_scalar_root(retained_values, 2.0)
    assert [row["value"] for row in variables if row["parameter"]] == [4.0]


@pytest.mark.integration
@pytest.mark.managed_primary
@pytest.mark.parametrize("select_member", [True, False])
def test_compact_study_retains_selected_original_member_or_declared_empty_relation(
    managed_observer_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    select_member: bool,
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    original = runtime.execute_study(
        runtime.record_study(
            package.admit_study(study_request(point(case, settings, 7)))
        )
    ).result(0)
    assert isinstance(original, pse.StoredResult)
    original_table = pa.table(original.table("runtime.solve_variables"))
    variables = [row for row in original_table.to_pylist() if not row["parameter"]]
    last = max(variables, key=lambda row: row["step"])
    member = StudyMemberDemandVariable(
        step=last["step"] if select_member else last["step"] + 1,
        symbol_id=last["symbol_id"].hex(),
    )
    demand = StudyDetailDemand(
        maximum_bytes=1 << 20,
        maximum_rows=1,
        members=(member,),
        relations=(),
    )
    no_detail = StudyDetailDemand(
        maximum_bytes=0, maximum_rows=0, members=(), relations=()
    )
    request = msgspec.structs.replace(
        study_request(point(case, settings, 11)),
        output=StudyOutputDemandCompact(detail=demand, failed_detail=no_detail),
    )
    definition = package.admit_study(request)
    assert definition.points[0].operation.output == request.output
    study = runtime.execute_study(runtime.record_study(definition))
    assert study.count == 1
    assert study.unattempted == 0
    assert study.conclusion.availability == "complete"
    assert study.outcome(0).scientific.usable
    result = study.result(0)
    assert isinstance(result, pse.StoredResult)
    assert result.usable
    assert result.completion.solves
    assert result.completion.assessments
    stream = result.table("runtime.solve_variables")
    del study, package, runtime
    selected = pa.table(stream)
    assert selected.schema == original_table.schema
    rows = selected.to_pylist()
    if select_member:
        (row,) = rows
        assert row["step"] == last["step"]
        assert row["symbol_id"] == last["symbol_id"]
        assert not row["parameter"]
        assert row["value"].hex() == last["value"].hex()
        assert row["tolerance"] == last["tolerance"]
        assert abs(row["value"] - 2.0) <= row["tolerance"]
    else:
        assert rows == []


@pytest.mark.integration
@pytest.mark.managed_primary
def test_study_unusable_predecessor_retains_refusal_without_dispatch(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    scalar, one = physical_ids(package.physical_context, "Scalar", "dimensionless")
    definition = package.admit_study(
        study_request(
            point(case, settings, 3, assignments=(assignment("a", -1.0, scalar, one),)),
            point(case, settings, 9, predecessor=3),
            point(case, settings, 17),
        )
    )
    study = runtime.execute_study(runtime.record_study(definition))
    assert study.unattempted == 1
    assert not study.outcome(0).scientific.usable
    failed = study.result(0)
    assert isinstance(failed, pse.StoredResult)
    assert not failed.usable
    refused = study.outcome(1)
    assert refused.key == 9
    assert refused.lifecycle == StudyPointState.FAILED
    assert not refused.scientific.usable
    assert refused.attempts == ()
    assert refused.start is None
    assert refused.diagnostic is not None
    assert refused.diagnostic.rule == "study.dependency.unusable"
    assert study.result(1) is None
    failure = study.failure(1)
    assert failure is not None
    assert failure.rule == refused.diagnostic.rule
    assert failure.envelope is not None
    assert failure.envelope == refused.diagnostic
    independent = study.result(2)
    assert isinstance(independent, pse.StoredResult)
    assert independent.usable
    assert study.conclusion.availability == "partial"
    assert study.conclusion.lifecycle == "terminal"
    rows = pa.table(study.table()).to_pylist()
    assert rows[1]["result_id"] is None
    assert rows[1]["diagnostic"]["rule"] == "study.dependency.unusable"
    assert rows[1]["attempts"] == []


@pytest.mark.component
def test_study_binding_identity_uses_admitted_member_and_physical_value(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    scalar, one = physical_ids(package.physical_context, "Scalar", "dimensionless")
    by_path = package.admit_study(
        study_request(
            point(case, settings, 2, assignments=(assignment("a", 9.0, scalar, one),))
        )
    )
    entries = by_path.points[0].binding.entries
    (member,) = entries
    by_member = package.admit_study(
        study_request(
            point(
                case,
                settings,
                13,
                assignments=(assignment(member, 9.0, scalar, one, member=True),),
            )
        )
    )
    assert by_path.points[0].binding_hash == by_member.points[0].binding_hash
    assert by_path.points[0].binding.entries[member].canonical == 9.0
    assert by_member.points[0].binding.entries[member].canonical == 9.0
    # Source spelling remains attributable without becoming a second content identity.
    assert (
        by_path.points[0].binding.entries[member].supplied
        != by_member.points[0].binding.entries[member].supplied
    )


@pytest.mark.component
def test_study_physical_mismatch_preserves_full_boundary_envelope(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    length, metre = physical_ids(package.physical_context, "Length", "m")
    with pytest.raises(pse.InspectionError) as raised:
        package.admit_study(
            study_request(
                point(
                    case,
                    settings,
                    2,
                    assignments=(assignment("a", 1.0, length, metre),),
                )
            )
        )
    report = raised.value.report
    assert report.rule == "study.binding.physical"
    assert report.stage == "study.binding"
    diagnostic = report.envelope
    assert isinstance(diagnostic, BoundaryDiagnosticOutput)
    assert diagnostic.rule == report.rule
    assert diagnostic.stage == report.stage
    assert diagnostic.code == report.code
    assert diagnostic.class_ == report.boundary_class
    assert diagnostic.locations == tuple(
        msgspec.convert(
            location, type=documents.SourceLocationBodyOutput, from_attributes=True
        )
        for location in report.source_locations
    )
    assert tuple(source.to_hex() for source in diagnostic.sources) == report.source_ids
    assert diagnostic.observations == report.observations
    assert diagnostic.locations[0].path == "a"
    assert diagnostic.locations[0].source in diagnostic.sources
    revision = diagnostic.locations[0].revision
    assert isinstance(revision, ContentHash)
    assert report.source_locations[0].path == "a"
    assert revision.to_prefixed().startswith("blake3:")
    assert report.source_locations[0].revision == revision
    operands = diagnostic.observations["operands"]
    assert isinstance(operands, ObservationContracts)
    assert len(operands.value) == 2
    assert operands.value[0].quantity != operands.value[1].quantity
    scalar, _ = physical_ids(package.physical_context, "Scalar", "dimensionless")
    assert bytes(operands.value[0].quantity).hex() == scalar
    assert bytes(operands.value[1].quantity).hex() == length
    for contract in operands.value:
        assert len(contract.quantity) == 16
        assert all(0 <= byte <= 255 for byte in contract.quantity)
        assert contract.indices == ()
    encoded = codec.encode_json(diagnostic)
    assert codec.decode_json(encoded, BoundaryDiagnosticOutput) == diagnostic
    cause = msgspec.convert(
        diagnostic, type=documents.BoundaryDiagnosticBodyOutput, from_attributes=True
    )
    nested = msgspec.structs.replace(diagnostic, causes=(cause,))
    restored = codec.decode_json(codec.encode_json(nested), BoundaryDiagnosticOutput)
    assert restored == nested
    assert restored.causes[0] == cause
    unknown = msgspec.json.decode(encoded, type=dict[str, object])
    unknown["unregistered"] = True
    with pytest.raises(msgspec.ValidationError, match="unregistered"):
        codec.decode_json(msgspec.json.encode(unknown), BoundaryDiagnosticOutput)
    indexed_operand = msgspec.structs.replace(
        operands.value[0],
        indices=((tuple(range(16)), tuple(range(16, 32)), tuple(range(32, 48))),),
    )
    indexed = msgspec.structs.replace(
        diagnostic,
        observations={
            **diagnostic.observations,
            "operands": ObservationContracts(
                value=(indexed_operand, operands.value[1])
            ),
        },
    )
    indexed_encoded = codec.encode_json(indexed)
    assert codec.decode_json(indexed_encoded, BoundaryDiagnosticOutput) == indexed
    for invalid_indices in (
        [[list(range(16))] * 2],
        [[list(range(16))] * 4],
        [[[0] * 15, list(range(16)), list(range(16))]],
        [[[0] * 17, list(range(16)), list(range(16))]],
    ):
        malformed = msgspec.json.decode(indexed_encoded, type=dict[str, object])
        malformed_observations = msgspec.convert(
            malformed["observations"], type=dict[str, object]
        )
        malformed_operands = msgspec.convert(
            malformed_observations["operands"], type=dict[str, object]
        )
        malformed_contracts = msgspec.convert(
            malformed_operands["value"], type=list[dict[str, object]]
        )
        malformed_contracts[0]["indices"] = invalid_indices
        malformed_operands["value"] = malformed_contracts
        malformed_observations["operands"] = malformed_operands
        malformed["observations"] = malformed_observations
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(msgspec.json.encode(malformed), BoundaryDiagnosticOutput)
    for length in (15, 17):
        malformed = msgspec.json.decode(encoded, type=dict[str, object])
        malformed_observations = msgspec.convert(
            malformed["observations"], type=dict[str, object]
        )
        malformed_operands = msgspec.convert(
            malformed_observations["operands"], type=dict[str, object]
        )
        malformed_contracts = msgspec.convert(
            malformed_operands["value"], type=list[dict[str, object]]
        )
        malformed_contracts[0]["quantity"] = [0] * length
        malformed_operands["value"] = malformed_contracts
        malformed_observations["operands"] = malformed_operands
        malformed["observations"] = malformed_observations
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(msgspec.json.encode(malformed), BoundaryDiagnosticOutput)


@pytest.mark.unit
def test_study_controller_moves_preserve_exact_typed_tuple_positions() -> None:
    document: dict[str, object] = {
        "case": {
            "case": "01" * 16,
            "route": "steady",
            "settings": msgspec.to_builtins(pse.SolveSettings()),
        },
        "bindings": [],
        "moves": [["control", 0]],
        "predictions": None,
    }
    decoded = codec.decode_json(msgspec.json.encode(document), ControllerOperation)
    assert decoded.moves == (("control", 0),)
    for invalid in (
        [["control"]],
        [["control", 0, 1]],
        [[0, "control"]],
        [["control", -1]],
    ):
        document["moves"] = invalid
        with pytest.raises(msgspec.ValidationError):
            codec.decode_json(msgspec.json.encode(document), ControllerOperation)


@pytest.mark.component
@pytest.mark.parametrize(
    ("operation", "version"),
    [("admit_study", 2), ("start_study", 4)],
)
def test_native_study_version_precedes_nested_current_decode(
    inspection_settings: pse.EngineSettings,
    operation: str,
    version: int,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, _case = _package(runtime)
    # Malformed current fields precede the header at the actual native boundary.
    historical = msgspec.json.encode({"points": "retired shape", "version": version})
    handle = package._handle
    native_call = (
        handle.admit_study
        if operation == "admit_study"
        else runtime._handle.start_study
    )
    with pytest.raises(pse.InspectionError) as caught:
        native_call(historical)
    location = caught.value.report.decode_location
    assert location is not None
    assert location.stage == "header"
    assert location.category == "data"
    assert "retired shape" not in caught.value.report.message


@pytest.mark.component
def test_study_request_excludes_owner_seed_capability(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    _package_owner, case = _package(runtime)
    document = codec.encode_json(study_request(point(case, pse.SolveSettings(), 0)))
    wire = msgspec.json.decode(document, type=dict[str, object])
    inputs = msgspec.convert(wire["inputs"], type=dict[str, object])
    points = msgspec.convert(inputs["value"], type=list[dict[str, object]])
    policy = msgspec.convert(points[0]["policy"], type=dict[str, object])
    assert "seed_need" not in policy
    policy["seed_need"] = "not_needed"
    points[0]["policy"] = policy
    inputs["value"] = points
    wire["inputs"] = inputs
    with pytest.raises(msgspec.ValidationError, match="seed_need"):
        codec.decode_json(msgspec.json.encode(wire), StudyRequest)


@pytest.mark.integration
def test_durable_study_retains_exact_results(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    scalar, one = physical_ids(package.physical_context, "Scalar", "dimensionless")
    definition = package.admit_study(
        study_request(
            point(case, settings, 0),
            point(
                case,
                settings,
                1,
                predecessor=0,
                continuation=True,
                assignments=(assignment("a", 9.0, scalar, one),),
            ),
            point(case, settings, 2, assignments=(assignment("a", -1.0, scalar, one),)),
            point(case, settings, 3, predecessor=2),
        )
    )
    handle = runtime.start_study(runtime.record_study(definition))
    status = handle.status()
    assert status.state == StudyState.OPEN
    assert all(point.attempt is None and not point.settled for point in status.points)
    assert handle.result() is None
    assert runtime.work(maximum_actions=0) == 0
    assert all(
        point.attempt is None and not point.settled for point in handle.status().points
    )
    assert runtime.study(handle.study_id).status().run == status.run

    # This process serves the queue: the points, then the study's finalization (and any
    # other job the shared store holds).
    assert runtime.work() >= 4
    retained = handle.wait(controls=pse.StudyWaitControls(timeout_seconds=60))
    status = runtime.study(handle.study_id).status()
    assert status.state == StudyState.CONCLUDED
    assert status.result_attempt == retained.attempt
    (attempt,) = pa.table(runtime.attempt_record(retained.attempt)).to_pylist()
    assert attempt["outcome"] == "partial"
    assert [point.state for point in status.points] == [
        StudyPointState.COMPLETED,
        StudyPointState.COMPLETED,
        StudyPointState.FAILED,
        StudyPointState.FAILED,
    ]
    refused = status.points[3].outcome
    assert refused is not None
    assert refused.attempts == ()
    assert not refused.scientific.usable
    assert refused.diagnostic is not None
    assert refused.diagnostic.rule == "study.dependency.unusable"
    assert retained.run == status.run
    outcomes = pa.table(
        runtime.results(retained.run, retained.attempt, "runtime.study_outcomes")
    ).to_pylist()
    assert [
        row["point_index"]
        for row in sorted(outcomes, key=lambda row: row["point_index"])
    ] == [0, 1, 2, 3]
    failed_member = next(row for row in outcomes if row["point_index"] == 2)
    assert failed_member["state"] == StudyPointState.FAILED
    assert failed_member["usable"] is False
    assert retained.points == tuple(
        (point.point_index, point.run, point.attempt) for point in status.points
    )
    for point_status in status.points[:2]:
        assert point_status.attempt is not None
        assert (
            pa.table(
                runtime.results(
                    point_status.run, point_status.attempt, "runtime.solve_variables"
                )
            ).num_rows
            > 0
        )


@pytest.mark.integration
def test_durable_study_cancel_and_its_refusals(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
    )
    scalar, one = physical_ids(package.physical_context, "Scalar", "dimensionless")
    definition = package.admit_study(
        study_request(
            point(case, settings, 0),
            point(
                case,
                settings,
                1,
                predecessor=0,
                continuation=True,
                assignments=(assignment("a", 9.0, scalar, one),),
            ),
        )
    )
    handle = runtime.start_study(runtime.record_study(definition))
    cancelled = handle.cancel()
    assert not cancelled.already_concluded
    assert cancelled.study_id == handle.study_id
    runtime.work()
    status = handle.status()
    assert status.study_id == cancelled.study_id
    assert status.state == StudyState.CONCLUDED
    assert status.result_attempt is not None
    (attempt,) = pa.table(runtime.attempt_record(status.result_attempt)).to_pylist()
    assert attempt["outcome"] == "cancelled"
    assert all(
        occurrence.state == StudyPointState.CANCELLED for occurrence in status.points
    )
    for occurrence in status.points:
        assert occurrence.outcome is not None
        assert occurrence.outcome.lifecycle == StudyPointState.CANCELLED
        assert occurrence.outcome.attempts == ()
        assert not occurrence.outcome.scientific.usable


@pytest.mark.integration
@pytest.mark.parametrize(
    "ephemeral",
    [
        pytest.param(True, id="ephemeral-preparation"),
        pytest.param(
            False, marks=pytest.mark.managed_primary, id="managed-durable-science"
        ),
    ],
)
def test_flash_sweep_prepares_structure_once_per_lane(
    request: pytest.FixtureRequest,
    canonical_substrate: CanonicalFixture,
    ephemeral: bool,
) -> None:
    """A feed-temperature sweep of the BT ideal flash changes values only (CT-S08)."""
    fixture = "inspection_settings" if ephemeral else "managed_observer_settings"
    engine_settings = request.getfixturevalue(fixture)
    assert isinstance(engine_settings, pse.EngineSettings)
    runtime = canonical_substrate.runtime(engine_settings, ephemeral=ephemeral)
    reference = Path(__file__).resolve().parents[3] / "packages/reference"

    physical_documents = package_documents(reference / "physical")
    assert physical_documents is not None
    physical = runtime.physical_from_documents(physical_documents)
    package = runtime.modeling_from_documents(
        family_packages("campaign-transport"),
        physical,
    )
    case = next(
        row.declaration_id
        for row in package.declarations()
        if row.name == "measurement_value_sweep"
    )
    temperatures = tuple(360.0 + 10.0 * index / 999 for index in range(1000))
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
    )
    temperature, kelvin = physical_ids(
        physical, "Temperature", "K"
    )
    definition = package.admit_study(
        study_request(
            *(
                point(
                    case,
                    settings,
                    index,
                    assignments=(assignment("root.inlet.T", t, temperature, kelvin),),
                )
                for index, t in enumerate(temperatures)
            )
        )
    )
    controls = pse.StudyRunControls(maximum_points=len(temperatures))
    study = (
        package.study(definition, controls=controls)
        if ephemeral
        else runtime.execute_study(runtime.record_study(definition), controls=controls)
    )
    assert study.count == len(temperatures)
    values = []
    for index in range(len(temperatures)):
        result = study.result(index)
        assert isinstance(result, (pse.RunResult, pse.StoredResult)), study.failure(
            index
        )
        if not result.usable:
            checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
            failed_checks = [row for row in checks if not row["satisfied"]]
            pytest.fail(
                f"point {index}, inlet temperature {temperatures[index]} K: "
                f"{tuple(diagnostic.message for diagnostic in result.diagnostics())}; "
                f"failed physical checks {failed_checks}"
            )
        # Independent points retain physical checks; the original oracle fixture remains
        # qualified separately at its single 368 K feed.
        (attempt,) = pa.table(result.table("runtime.solve_runs")).to_pylist()
        if attempt["state"] == "constant_evaluation":
            # Complete block assembly is independently assessed in original space;
            # it has no redundant outer native solve or native termination code.
            assert attempt["candidate_kind"] == "constant_evaluation"
            assert attempt["qualification"] == "feasible"
            assert attempt["feasible"] is True
            assert attempt["error"] is None
            assert attempt["validation_error"] is None
            assert attempt["backend"] is None
            assert attempt["termination"] is None
            assert attempt["native_code"] is None
            assert attempt["native_status"] is None
            events = pa.table(result.table("runtime.solve_strategy_events")).to_pylist()
            assert any(
                row["mechanism"] == "block"
                and row["kind"] == "finished"
                and row["phase"] == "assessment"
                and row["original_conclusion"] == "satisfied"
                and row["permission"] == "usable"
                for row in events
            )
        else:
            assert attempt["termination"] == NativeTermination.SUCCESS
        values.append(
            pa.table(result.table("runtime.solve_variables"))
            .column("value")
            .to_pylist()
        )
    # Each point solved its own feed temperature ...
    assert values[0] != values[-1]
    preparations = study.preparations
    if ephemeral:
        # Each concurrently executing lane owns its prepared view. A value change
        # rebinds that owner's numerical products rather than rebuilding structure.
        # Count actual work in this process across the complete scientific sweep.
        lane_width = min(engine_settings.threads, engine_settings.math_jobs)
        assert 1 <= preparations.views <= lane_width, preparations
        assert preparations.views + preparations.rebuilt + preparations.shared == len(
            temperatures
        ), preparations
    else:
        # These counters observe the caller process. The separate ephemeral mode
        # proves reuse; zero here does not measure preparation in the primary worker.
        assert (
            preparations.views,
            preparations.rebuilt,
            preparations.shared,
            preparations.observations,
        ) == (0, 0, 0, 0), preparations


@pytest.mark.integration
@pytest.mark.managed_primary
def test_fresh_capped_study_preserves_individual_automatic_execution(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    native = pse.PounceSettings()
    settings = pse.SolveSettings(
        backend=NativeBackend.POUNCE,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
        settings=msgspec.structs.replace(
            native,
            linear=msgspec.structs.replace(native.linear, ordering=FeralOrdering.AMD),
        ),
    )
    settings = msgspec.structs.replace(
        settings,
        composition=msgspec.structs.replace(
            settings.composition,
            limits=WorkLimits(attempts=4, evaluations=500),
        ),
    )
    alone = runtime.execute_study(
        runtime.record_study(
            package.admit_study(study_request(point(case, settings, 3)))
        )
    )
    paired = runtime.execute_study(
        runtime.record_study(
            package.admit_study(
                study_request(point(case, settings, 7), point(case, settings, 11))
            )
        )
    )
    assert alone.outcome(0).scientific.usable, codec.encode_json(
        alone.outcome(0)
    ).decode()
    for index in range(2):
        assert paired.outcome(index).scientific.usable
        result = paired.result(index)
        assert result is not None
        _assert_scalar_root(pa.table(result.table("runtime.solve_variables")), 2.0)
        events = pa.table(result.table("runtime.solve_strategy_events")).to_pylist()
        assert any(
            row["kind"] == "started" and row["decision_identity"] for row in events
        )
        assert all(row["observation"] != "contract_failure" for row in events)

    # This adapter admits callback evaluations; opaque native counters still refuse.
    unsupported = msgspec.structs.replace(
        settings,
        composition=msgspec.structs.replace(
            settings.composition,
            limits=WorkLimits(
                attempts=4, evaluations=500, iterations=500, factorizations=500
            ),
        ),
    )
    refused = runtime.execute_study(
        runtime.record_study(
            package.admit_study(study_request(point(case, unsupported, 13)))
        )
    )
    outcome = refused.outcome(0)
    assert not outcome.scientific.usable
    assert not outcome.scientific.seed_permission
    assert outcome.diagnostic is not None
    assert outcome.diagnostic.code == DiagnosticCode.NATIVE_UNSUPPORTED


@pytest.mark.integration
@pytest.mark.managed_primary
def test_capped_related_root_study_charges_prediction_and_screening(
    managed_observer_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(
        runtime, source=SOURCE.replace("    annotation bounds x(0,10);\n", "")
    )
    settings = pse.SolveSettings(
        backend=NativeBackend.KINSOL,
        intent=NativeSolveIntent.ROOT,
        presolve=PresolvePolicyKind.OFF,
    )
    settings = msgspec.structs.replace(
        settings,
        composition=msgspec.structs.replace(
            settings.composition,
            limits=WorkLimits(attempts=4, evaluations=500),
            recovery=(NumericalStartOrigin.PREDICTED,),
        ),
    )
    scalar, one = physical_ids(package.physical_context, "Scalar", "dimensionless")
    study = runtime.execute_study(
        runtime.record_study(
            package.admit_study(
                study_request(
                    point(case, settings, 0),
                    point(
                        case,
                        settings,
                        1,
                        predecessor=0,
                        continuation=True,
                        assignments=(assignment("a", 9.0, scalar, one),),
                    ),
                )
            )
        )
    )
    assert all(study.outcome(index).scientific.usable for index in range(2)), tuple(
        codec.encode_json(study.outcome(index)).decode() for index in range(2)
    )
    result = study.result(1)
    assert result is not None
    events = pa.table(result.table("runtime.solve_strategy_events")).to_pylist()
    assert any(
        row["start_origin"] == "predicted" and row["kind"] == "started"
        for row in events
    )
    charged = [
        row["evaluations"] for row in events if row["charging_owner"] is not None
    ]
    assert charged
    assert all(value is not None for value in charged)
    # The target original-screening callbacks share its native task's finite counter.
    assert 2 <= sum(value for value in charged if value is not None) <= 500


@pytest.mark.component
def test_lazy_study_public_descriptor_retains_exact_source_without_admitting_science(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    variants = StudyVariants(
        source=msgspec.convert(
            package.operation_source,
            type=documents.OperationSourceBody,
            from_attributes=True,
        ),
        choices=(
            SourceChoice(
                alternatives=(
                    documents.SourceDeltaBody(
                        parent=package.revision_id,
                        removals=(DeclarationId(SemanticId.from_hex("0" * 32)),),
                    ),
                )
            ),
        ),
        selected=(
            SelectedStudyVariant(ordinal=0, point=point(case, pse.SolveSettings(), 17)),
        ),
    )
    definition = package.admit_study(
        StudyRequest(
            output=StudyOutputDemandFull(),
            inputs=StudyInputsLazy(value=variants),
        )
    )
    assert definition.points == ()
    assert definition.variants is not None
    retained = definition.variants.variants
    assert retained.source == msgspec.convert(
        package.operation_source,
        type=documents.OperationSourceBodyOutput,
        from_attributes=True,
    )
    assert retained.selected[0].point.policy.key == 17
    local_parent = definition.modeling_revision
    assert local_parent == (package.canonical_revision or package.revision_id)
    assert retained.choices[0].alternatives[0].parent == local_parent
    with pytest.raises(TypeError):
        codec.encode_json(definition)
    recorded = runtime.record_study(definition)
    assert codec.decode_json(codec.encode_json(recorded), type(recorded)) == recorded
    assert recorded.variants is not None
    assert (
        recorded.variants.variants.choices[0].alternatives[0].parent
        == recorded.modeling_revision
    )
    assert retained.choices[0].alternatives[0].parent == local_parent


@pytest.mark.component
def test_local_lazy_study_executes_selected_occurrence_without_store(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.ROOT,
        presolve=PresolvePolicyKind.OFF,
    )
    variants = StudyVariants(
        source=msgspec.convert(
            package.operation_source,
            type=documents.OperationSourceBody,
            from_attributes=True,
        ),
        choices=(
            SourceChoice(
                alternatives=(
                    documents.SourceDeltaBody(parent=package.revision_id),
                    documents.SourceDeltaBody(
                        parent=package.revision_id,
                        removals=(DeclarationId(SemanticId.from_hex("0" * 32)),),
                    ),
                )
            ),
        ),
        selected=(SelectedStudyVariant(ordinal=0, point=point(case, settings, 19)),),
    )
    assert variants.version == 2
    local = package.admit_study(
        StudyRequest(
            output=StudyOutputDemandFull(), inputs=StudyInputsLazy(value=variants)
        )
    )
    assert local.points == ()
    del package
    report = local.execute()
    del local
    assert report.count == 1
    assert report.outcome(0).key == 19
    assert report.outcome(0).scientific.usable
    result = report.result(0)
    assert isinstance(result, pse.RunResult)
    _assert_scalar_root(pa.table(result.table("runtime.solve_variables")), 2.0)


@pytest.mark.component
def test_local_study_runs_without_store_and_retains_source_after_package_release(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    package, case = _package(runtime)
    assert package.canonical_revision is None
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.ROOT,
        presolve=PresolvePolicyKind.OFF,
    )
    local = package.admit_study(
        study_request(point(case, settings, 7), point(case, settings, 11))
    )
    assert isinstance(local, pse.LocalStudy)
    assert local.points[0].binding_hash == local.points[1].binding_hash
    with pytest.raises(TypeError):
        codec.encode_json(local)
    with pytest.raises(pse.InspectionError):
        runtime.record_study(local)
    del package
    report = local.execute()
    del local
    assert report.count == 2
    assert report.diagnostic_selection() is None
    assert tuple(report.outcome(index).key for index in range(2)) == (7, 11)
    for index in range(2):
        assert report.outcome(index).scientific.usable
        result = report.result(index)
        assert isinstance(result, pse.RunResult)
        _assert_scalar_root(pa.table(result.table("runtime.solve_variables")), 2.0)


@pytest.mark.integration
@pytest.mark.managed_primary
def test_managed_selected_detail_retains_actual_worker_owners_and_complete_prefixes(
    managed_observer_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(managed_observer_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    local = package.admit_study(
        study_request(point(case, settings, 7), point(case, settings, 11))
    )
    slots = sum(item.policy.attempt_limit for item in local.points)
    request = DetailRequest(
        families=("native_owner_return.v1",),
        requirement="complete",
        sample_every=1,
        duration=ClosedDuration(secs=120, nanos=0),
        output=str(tmp_path / "managed-detail"),
        budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=16 << 20),
        managed_budget=CaptureBudget(
            memory_bytes=slots * (4 << 20), output_bytes=slots * (8 << 20)
        ),
    )
    with runtime.observations().collect(request) as capture:
        recorded = runtime.record_study(local)
        report = runtime.execute_study(recorded)
        assert report.conclusion.availability == "complete"
        assert report.count == 2
        for index in range(report.count):
            assert report.outcome(index).scientific.usable
        document = report.diagnostic_selection()
        assert document is not None
        assert document.selection is not None
        assert document.selection.refusal is None
        assert document.omitted_points == 0
        assert len(document.references) == 2
        for reference in document.references:
            assert reference.state == "captured"
            assert reference.owner is not None
            assert reference.owner.pid != os.getpid()
            assert reference.owner.run == reference.run
            assert reference.owner.attempt == reference.attempt
            assert reference.detail is not None
            assert reference.detail.outcome == "complete"
            assert not reference.detail.source_partial
            assert (
                reference.detail.families[0].observed
                == reference.detail.families[0].acknowledged
                > 0
            )
    assert capture.snapshot().outcome == "incomplete", (
        "local absent family cannot establish managed graph completeness"
    )

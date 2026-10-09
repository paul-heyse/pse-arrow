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

import pse
from pse import codec
from pse.contracts.documents import (
    BoundaryDiagnostic,
    ControllerOperation,
    ObservationContracts,
    StudyRequest,
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


def _physical_ids(quantity: str, unit: str) -> tuple[str, str]:
    return physical_ids(
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives/materials/physical.yaml",
        quantity,
        unit,
    )


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
                        "metadata_bytes",
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
            metadata_bytes=cache["metadata_bytes"],
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
        study = package.study(definition)
        _assert_completed_managed_study(fixture, study, keys)
        return
    handle = package.study(definition, runtime=runtime)
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
    issued = handle.status()
    assert tuple(item.point_index for item in issued.points) == keys
    assert all(item.attempt is not None for item in issued.points[:16])
    assert all(item.attempt is None for item in issued.points[16:])
    cancelled = handle.cancel()
    assert not cancelled.already_concluded
    assert cancelled.study_id == handle.study_id.to_hex()
    # Do not release the barrier: the actual native stop flags must wake all
    # sixteen admitted owners, while the effect-free tail never enters it.
    drained = fixture.observation(lambda value: value.active == 0)
    assert drained.maximum == 16
    assert len(drained.entries) == 16
    assert all(entry.released and entry.stop_observed for entry in drained.entries)
    assert not (fixture.directory / "release.json").exists()
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
    assert (
        codec.decode_json(codec.encode_json(definition), type(definition)) == definition
    )
    study = package.study(definition)
    assert isinstance(study, pse.StudyReport)
    assert study.count == 2
    assert study.unattempted == 0
    assert study.conclusion.availability == "complete"
    assert study.conclusion.lifecycle == "terminal"
    results = [study.result(index) for index in range(2)]
    assert all(result is not None and result.usable for result in results)
    first, second = results
    assert first is not None
    assert second is not None
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
    del first, second, results, study, package, runtime
    rows = pa.table(history).to_pylist()
    assert [row["point_index"] for row in rows] == [7, 11]
    assert all(row["usable"] and len(row["attempts"]) == 1 for row in rows)
    retained_values = pa.table(values)
    variables = retained_values.to_pylist()
    _assert_scalar_root(retained_values, 2.0)
    assert [row["value"] for row in variables if row["parameter"]] == [4.0]


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
    scalar, one = _physical_ids("Scalar", "dimensionless")
    definition = package.admit_study(
        study_request(
            point(case, settings, 3, assignments=(assignment("a", -1.0, scalar, one),)),
            point(case, settings, 9, predecessor=3),
            point(case, settings, 17),
        )
    )
    study = package.study(definition)
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
    assert independent is not None
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
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
    length, metre = _physical_ids("Length", "m")
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
    assert isinstance(diagnostic, BoundaryDiagnostic)
    assert diagnostic.rule == report.rule
    assert diagnostic.stage == report.stage
    assert diagnostic.code == report.code
    assert diagnostic.class_ == report.boundary_class
    assert diagnostic.locations == report.source_locations
    assert diagnostic.sources == report.source_ids
    assert diagnostic.observations == report.observations
    assert diagnostic.locations[0].path == "a"
    assert diagnostic.locations[0].source in diagnostic.sources
    revision = diagnostic.locations[0].revision
    assert isinstance(revision, str)
    assert report.source_locations[0].path == "a"
    assert revision.startswith("blake3:")
    assert report.source_locations[0].revision == revision
    operands = diagnostic.observations["operands"]
    assert isinstance(operands, ObservationContracts)
    assert len(operands.value) == 2
    assert operands.value[0].quantity != operands.value[1].quantity
    scalar, _ = _physical_ids("Scalar", "dimensionless")
    assert bytes(operands.value[0].quantity).hex() == scalar
    assert bytes(operands.value[1].quantity).hex() == length
    for contract in operands.value:
        assert len(contract.quantity) == 16
        assert all(0 <= byte <= 255 for byte in contract.quantity)
        assert contract.indices == ()
    encoded = codec.encode_json(diagnostic)
    assert codec.decode_json(encoded, BoundaryDiagnostic) == diagnostic
    nested = msgspec.structs.replace(diagnostic, causes=(diagnostic,))
    restored = codec.decode_json(codec.encode_json(nested), BoundaryDiagnostic)
    assert restored == nested
    assert restored.causes[0] == diagnostic
    unknown = msgspec.json.decode(encoded, type=dict[str, object])
    unknown["unregistered"] = True
    with pytest.raises(msgspec.ValidationError, match="unregistered"):
        codec.decode_json(msgspec.json.encode(unknown), BoundaryDiagnostic)
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
    assert codec.decode_json(indexed_encoded, BoundaryDiagnostic) == indexed
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
            codec.decode_json(msgspec.json.encode(malformed), BoundaryDiagnostic)
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
            codec.decode_json(msgspec.json.encode(malformed), BoundaryDiagnostic)


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
    [("admit_study", 2), ("study", 4), ("start_study", 4)],
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
    pattern = "explicit readmission is required"
    if operation == "admit_study":
        with pytest.raises(pse.InspectionError, match=pattern):
            handle.admit_study(historical)
    elif operation == "study":
        with pytest.raises(pse.InspectionError, match=pattern):
            handle.study(historical)
    else:
        with pytest.raises(pse.InspectionError, match=pattern):
            handle.start_study(runtime._handle, historical)


@pytest.mark.component
def test_study_request_excludes_owner_seed_capability(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    _package_owner, case = _package(runtime)
    document = codec.encode_json(study_request(point(case, pse.SolveSettings(), 0)))
    wire = msgspec.json.decode(document, type=dict[str, object])
    points = msgspec.convert(wire["points"], type=list[dict[str, object]])
    policy = msgspec.convert(points[0]["policy"], type=dict[str, object])
    assert "seed_need" not in policy
    policy["seed_need"] = "not_needed"
    points[0]["policy"] = policy
    wire["points"] = points
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
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
    handle = package.study(definition, runtime=runtime)
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
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
    handle = package.study(definition, runtime=runtime)
    cancelled = handle.cancel()
    assert not cancelled.already_concluded
    assert cancelled.study_id == handle.study_id.to_hex()
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
def test_flash_sweep_prepares_structure_once(
    request: pytest.FixtureRequest,
    canonical_substrate: CanonicalFixture,
    ephemeral: bool,
) -> None:
    """A feed-temperature sweep of the BT ideal flash changes values only (CT-S08)."""
    fixture = "inspection_settings" if ephemeral else "managed_observer_settings"
    settings = request.getfixturevalue(fixture)
    assert isinstance(settings, pse.EngineSettings)
    runtime = canonical_substrate.runtime(settings, ephemeral=ephemeral)
    reference = Path(__file__).resolve().parents[3] / "packages/reference"

    def documents(path: Path) -> dict[str, str | bytes]:
        return {
            p.relative_to(path).as_posix(): p.read_bytes()
            for p in path.rglob("*")
            if p.is_file()
            and p.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}
        }

    physical = runtime.physical_from_documents(documents(reference / "physical"))
    package = runtime.modeling_from_documents(
        [
            documents(reference / name)
            for name in (
                "data/oracles/teqp-0.23.1",
                "data/oracles/feos-0.10.1",
                "data/gross-sadowski-2001",
                "data/references",
                "data/nist",
                "data/perry7",
                "data/poling2000",
                "data/oracles/idaes-2.13",
                "data/species",
                "data/ciaaw",
                "seed-data",
                "campaign",
                "process",
                "thermodynamics",
                "methods",
                "domain",
                "physical",
            )
        ],
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
        reference / "physical/materials/physical.yaml", "Temperature", "K"
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
    study = package.study(
        definition, controls=pse.StudyRunControls(maximum_points=len(temperatures))
    )
    assert study.count == len(temperatures)
    values = []
    for index in range(len(temperatures)):
        result = study.result(index)
        assert result is not None, study.failure(index)
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
        # Structural admission prepares no solver view. Ready execution freezes at
        # most one shared view and admits every point's actual numerical values.
        # This same scientific sweep independently measures preparation in its owner.
        assert preparations.views <= 1, preparations
        assert preparations.views + preparations.rebuilt + preparations.shared >= len(
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
    alone = package.study(package.admit_study(study_request(point(case, settings, 3))))
    paired = package.study(
        package.admit_study(
            study_request(point(case, settings, 7), point(case, settings, 11))
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
    refused = package.study(
        package.admit_study(study_request(point(case, unsupported, 13)))
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
    study = package.study(
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

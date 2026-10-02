# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable occurrences, scientific permissions and durable study publication."""

import uuid
from datetime import timedelta
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts.documents import (
    BoundaryDiagnostic,
    ControllerOperation,
    StudyRequest,
)
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


def _physical_ids(quantity: str, unit: str) -> tuple[str, str]:
    return physical_ids(
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives/materials/physical.yaml",
        quantity,
        unit,
    )


@pytest.mark.unit
def test_study_repeated_bindings_retain_distinct_occurrences_and_owned_results(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    definition = package.admit_study(
        request(point(case, settings, 7), point(case, settings, 11))
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
    assert first is not None and second is not None
    assert first.run_id != second.run_id
    for index, key in enumerate((7, 11)):
        outcome = study.outcome(index)
        assert outcome.key == key
        assert outcome.lifecycle == StudyPointState.COMPLETED
        assert outcome.scientific.usable
        assert len(outcome.attempts) == 1
        assert outcome.attempts[0].attempt_id is None
        assert outcome.diagnostic is None
        assert study.failure(index) is None
    history = study.table()
    values = first.table("runtime.solve_variables")
    del first, second, results, study, package, runtime
    rows = pa.table(history).to_pylist()
    assert [row["point_index"] for row in rows] == [7, 11]
    assert all(row["usable"] and len(row["attempts"]) == 1 for row in rows)
    variables = pa.table(values).to_pylist()
    assert [row["value"] for row in variables if not row["parameter"]] == pytest.approx(
        [2.0]
    )
    assert [row["value"] for row in variables if row["parameter"]] == [4.0]


@pytest.mark.unit
def test_study_unusable_predecessor_retains_refusal_without_dispatch(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    scalar, one = _physical_ids("Scalar", "dimensionless")
    definition = package.admit_study(
        request(
            point(case, settings, 3, assignments=(assignment("a", -1.0, scalar, one),)),
            point(case, settings, 9, predecessor=3),
            point(case, settings, 17),
        )
    )
    study = package.study(definition)
    assert study.unattempted == 1
    assert not study.outcome(0).scientific.usable
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
    assert (
        codec.decode_json(failure.envelope, type(refused.diagnostic))
        == refused.diagnostic
    )
    independent = study.result(2)
    assert independent is not None and independent.usable
    assert study.conclusion.availability == "partial"
    assert study.conclusion.lifecycle == "terminal"
    rows = pa.table(study.table()).to_pylist()
    assert rows[1]["result_id"] is None
    assert rows[1]["diagnostic"]["rule"] == "study.dependency.unusable"
    assert rows[1]["attempts"] == []


@pytest.mark.unit
def test_study_binding_identity_uses_admitted_member_and_physical_value(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    scalar, one = _physical_ids("Scalar", "dimensionless")
    by_path = package.admit_study(
        request(
            point(case, settings, 2, assignments=(assignment("a", 9.0, scalar, one),))
        )
    )
    entries = by_path.points[0].binding.entries
    (member,) = entries
    by_member = package.admit_study(
        request(
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


@pytest.mark.unit
def test_study_physical_mismatch_preserves_full_boundary_envelope(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package, case = _package(runtime)
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    length, metre = _physical_ids("Length", "m")
    with pytest.raises(pse.InspectionError) as raised:
        package.admit_study(
            request(
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
    assert report.envelope is not None
    envelope = msgspec.json.decode(report.envelope, type=dict[str, object])
    assert envelope["rule"] == report.rule
    assert envelope["stage"] == report.stage
    locations = msgspec.convert(envelope["locations"], type=list[dict[str, object]])
    assert locations[0]["path"] == "a"
    revision = locations[0]["revision"]
    assert isinstance(revision, str)
    assert report.source_locations[0].path == "a"
    assert report.source_locations[0].revision == revision.removeprefix("blake3:")
    operands = next(value for value in report.observations if value.name == "operands")
    assert operands.kind == "contracts"
    assert len(operands.contracts) == 2
    assert operands.contracts[0][0] != operands.contracts[1][0]
    observations = msgspec.convert(envelope["observations"], type=dict[str, object])
    observation = msgspec.convert(observations["operands"], type=dict[str, object])
    assert observation["kind"] == "contracts"
    contracts = msgspec.convert(observation["value"], type=list[dict[str, object]])
    assert len(contracts) == 2
    for contract, (quantity, indices) in zip(
        contracts, operands.contracts, strict=True
    ):
        encoded_quantity = msgspec.convert(contract["quantity"], type=list[int])
        assert bytes(encoded_quantity).hex() == quantity
        assert contract["indices"] == list(indices) == []
    diagnostic = codec.decode_json(report.envelope, BoundaryDiagnostic)
    assert diagnostic.rule == report.rule
    nested = msgspec.structs.replace(diagnostic, causes=(diagnostic,))
    assert codec.decode_json(codec.encode_json(nested), BoundaryDiagnostic) == nested
    unknown = msgspec.json.decode(report.envelope, type=dict[str, object])
    unknown["unregistered"] = True
    with pytest.raises(msgspec.ValidationError, match="unregistered"):
        codec.decode_json(msgspec.json.encode(unknown), BoundaryDiagnostic)
    for length in (15, 17):
        malformed = msgspec.json.decode(report.envelope, type=dict[str, object])
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


@pytest.mark.unit
def test_study_request_excludes_owner_seed_capability(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package, case = _package(runtime)
    document = codec.encode_json(request(point(case, pse.SolveSettings(), 0)))
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
    definition = package.admit_study(
        request(
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
    handle = package.study(definition, runtime=runtime, workspace=workspace)
    status = handle.status()
    assert status.state == StudyState.OPEN
    assert [point.job_state for point in status.points] == [
        JobState.QUEUED,
        JobState.WAITING,
        JobState.QUEUED,
        JobState.WAITING,
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
        StudyPointState.FAILED,
    ]
    refused = status.points[3].outcome
    assert refused is not None
    assert refused.attempts == ()
    assert not refused.scientific.usable
    assert refused.diagnostic is not None
    assert refused.diagnostic.rule == "study.dependency.unusable"
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
    scalar, one = _physical_ids("Scalar", "dimensionless")
    definition = package.admit_study(
        request(
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
    handle = package.study(definition, runtime=runtime, workspace=workspace)
    cancelled = handle.cancel()
    assert cancelled.cancelled == (0, 1)
    assert cancelled.concluded
    status = handle.status()
    assert status.state == StudyState.CONCLUDED
    assert status.attempt_state == AttemptState.CANCELLED
    assert all(
        occurrence.state == StudyPointState.CANCELLED for occurrence in status.points
    )
    for occurrence in status.points:
        assert occurrence.outcome is not None
        assert occurrence.outcome.lifecycle == StudyPointState.CANCELLED
        assert occurrence.outcome.attempts == ()
        assert not occurrence.outcome.scientific.usable

    # A workspace selects a durable study; a package changed in memory has no authored
    # sources.
    with pytest.raises(ValueError, match="durable study"):
        # pyrefly: ignore[unexpected-keyword] -- the overloads reject this call; its
        # runtime refusal is what is under test
        package.study(definition, workspace=workspace)
    with pytest.raises(pse.InspectionError, match="immutable source documents"):
        package.with_limits(pse.ModelingLimits()).study(
            definition, runtime=runtime, workspace=workspace
        )


@pytest.mark.integration
def test_flash_sweep_prepares_structure_once(
    inspection_settings: pse.EngineSettings,
) -> None:
    """A feed-temperature sweep of the BT ideal flash changes values only (CT-S08)."""
    runtime = pse.Runtime(inspection_settings)
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
        request(
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
    study = package.study(definition, maximum_points=len(temperatures))
    assert study.count == len(temperatures)
    values = []
    for index in range(len(temperatures)):
        result = study.result(index)
        assert result is not None, study.failure(index)
        # Independent points retain physical checks; the original oracle fixture remains
        # qualified separately at its single 368 K feed.
        (attempt,) = pa.table(result.table("runtime.solve_runs")).to_pylist()
        assert attempt["termination"] == NativeTermination.SUCCESS
        assert result.usable, result.diagnostics()
        values.append(
            pa.table(result.table("runtime.solve_variables"))
            .column("value")
            .to_pylist()
        )
    # Each point solved its own feed temperature ...
    assert values[0] != values[-1]
    # ... on one prepared structure: every later point only rebound its values.
    preparations = study.preparations
    assert preparations.views == 1, preparations
    assert preparations.rebuilt + preparations.shared == len(temperatures) - 1, (
        preparations
    )

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public declaration, lifecycle and owned-result contract units."""

import asyncio
import contextlib
import gc
import json
from collections.abc import Callable
from pathlib import Path
from typing import cast

import attrs
import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse import modeling as w
from pse.contracts import authored as a
from pse.contracts import documents
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import (
    AttemptKind,
    AttemptState,
    ModelingAnalysisRoute,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.contracts.identities import (
    DeclarationId,
    FitId,
    InstanceId,
    PublicationId,
    WorkspaceId,
)
from pse.contracts.values import SemanticId

#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


def declaration(n: int) -> DeclarationId:
    """The identity an authored `@id` gives a case or fixture declaration."""
    return DeclarationId(identity(n))


@pytest.mark.unit
def test_explicit_cone_strategy_preserves_native_qualification(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    def port(n: int) -> dict[str, str]:
        return {
            "symbol_id": identity(n).to_hex(),
            "quantity_id": identity(31).to_hex(),
            "unit_id": identity(10).to_hex(),
        }

    request: dict[str, object] = {
        "variables": [port(201)],
        "rows": [port(202)],
        "objective_port": port(0),
        "quadratic": {
            "rows": 1,
            "columns": 1,
            "column_starts": [0, 0],
            "row_indices": [],
            "values": [],
        },
        "objective": [1.0],
        "constraints": {
            "rows": 1,
            "columns": 1,
            "column_starts": [0, 1],
            "row_indices": [0],
            "values": [-1.0],
        },
        "rhs": [-2.0],
        "cones": [{"kind": "nonnegative", "dimension": 1}],
        "objective_constant": 3.0,
    }
    prepared = runtime.prepare_conic(
        codec.decode_json(codec.encode_json(request), pse.ConicRequest),
        physical,
        pse.SolveSettings(backend=NativeBackend.CLARABEL),
    )
    assert [route.backend for route in prepared.routes] == ["clarabel"]
    result = prepared.run()
    assert not result.failures()
    (row,) = result.attempts()
    attempt = row.report
    assert attempt is not None
    assert attempt.qualification == "optimal_within_tolerance"
    assert attempt.objective == pytest.approx(5.0, abs=1e-6)
    assert dict(attempt.primal())[identity(201).to_hex()] == pytest.approx(
        2.0, abs=1e-6
    )
    assert attempt.normalized_violation is not None
    assert attempt.normalized_violation <= 1.0


@pytest.mark.unit
def test_explicit_primal_seed_and_transactional_initialization(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/roots.pse": (
                    "package roots {def Root {"
                    "var x:Scalar; eq square:x*x==4; annotation start x(-1);"
                    "}}"
                ),
            }
        ],
        physical,
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    settings = pse.SolveSettings(
        intent=NativeSolveIntent.ROOT,
        backend=NativeBackend.KINSOL,
        presolve=PresolvePolicyKind.OFF,
    )
    member = package.inspect(case, settings).members[0]
    x = SemanticId.from_hex(member.id)
    prepared = package.prepare_solve(case, settings)
    assert prepared.eligibility
    explicit = prepared.with_primal_start({x: 1.0}).start().wait()
    seed = explicit.available_start()
    assert seed is not None
    snapshot = codec.decode_json(seed.snapshot(), pse.WarmStartSnapshot)
    assert isinstance(snapshot.payload, documents.WarmPayloadSnapshotRoot)
    assert isinstance(snapshot.payload.primal[0], documents.ObservationReal)
    assert snapshot.payload.primal[0].value == pytest.approx(2.0, abs=1e-6)
    assert snapshot.origin is not None
    assert snapshot.origin.run == explicit.run_id.to_hex()
    result = package.prepare_block_initialization(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.INITIALIZE, backend=NativeBackend.KINSOL
        ),
        [{}],
    ).run()
    stage = result.initialization()
    assert stage is not None
    assert stage.completed_stages == 1
    assert stage.original[x.to_hex()] == -1.0
    assert stage.solved_unknowns[x.to_hex()] == pytest.approx(-2.0, abs=1e-6)


@pytest.fixture(scope="module")
def runtime(inspection_settings: pse.EngineSettings) -> pse.Runtime:
    return pse.Runtime(inspection_settings)


@pytest.fixture(scope="module")
def durable_runtime(
    inspection_settings: pse.EngineSettings, operational_store: pse.OperationalStore
) -> pse.Runtime:
    return pse.Runtime(inspection_settings, store=operational_store)


@pytest.fixture(scope="module")
def physical(runtime: pse.Runtime) -> pse.PhysicalContext:
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    documents = {
        str(path.relative_to(root)): path.read_text()
        for path in root.rglob("*")
        if path.is_file()
    }
    return runtime.physical_from_documents(documents)


def package_documents(source: str) -> dict[str, str]:
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    return {"package.toml": manifest, "models/fixed.pse": source}


def revision(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> pse.ModelingPackage:
    source = (
        f'package atomic {{ @id("{identity(101).to_hex()}") test Fixed '
        "fixture {dof 0; fix x=2{m};} "
        "{var x:Length; annotation bounds x(0{m},10{m});} }"
    )
    return runtime.modeling_from_documents([package_documents(source)], physical)


@pytest.mark.unit
def test_empty_library_is_admitted_but_missing_case_is_refused(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    manifest = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text()
    package = runtime.modeling_from_documents([{"package.toml": manifest}], physical)
    assert not package.declarations()
    with pytest.raises(pse.InspectionError):
        package.prepare_solve(
            declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )


@pytest.mark.unit
def test_native_capability_discovery_and_hard_cut(runtime: pse.Runtime) -> None:
    capabilities = runtime.capabilities()
    assert capabilities
    assert "clarabel" in {c.backend for c in capabilities}
    assert all(c.classes and c.reuse and c.cancellation for c in capabilities)
    for capability in capabilities:
        pse.SolveSettings(backend=NativeBackend(capability.backend))
    assert not hasattr(pse, "probe_host")
    assert not hasattr(pse, "HostCapabilities")


@pytest.mark.unit
def test_revision_edit_is_atomic_and_has_no_python_math(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    model = revision(runtime, physical)
    original = model.declarations()
    with pytest.raises(pse.InspectionError):
        model.with_declarations((*original, *original))
    changed = tuple(
        attrs.evolve(row, name="Renamed")
        if row.declaration_id == identity(101)
        else row
        for row in original
    )
    revised = model.with_declarations(changed)
    assert model.declarations() == original
    assert (
        next(
            row.name
            for row in revised.declarations()
            if row.declaration_id == identity(101)
        )
        == "Renamed"
    )
    result = (
        model.prepare_solve(
            declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
        .start()
        .wait()
    )
    assert pa.table(result.table("runtime.solve_variables")).column(
        "value"
    ).to_pylist() == [2.0]


@pytest.mark.unit
def test_blocking_async_share_terminal_report_and_last_array_owner(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    model = revision(runtime, physical)
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    prepared = model.prepare_solve(declaration(101), settings)
    assert prepared.route.constant
    assert prepared.route.backend is None
    handle = prepared.start()
    result = handle.wait()

    async def twice(job: pse.RunHandle, expected: SemanticId) -> None:
        a, b = await asyncio.gather(job.wait_async(), job.wait_async())
        assert a.run_id == b.run_id == expected

    asyncio.run(twice(handle, result.run_id))
    runs = (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_runs"))
        .read_all()
        .to_pylist()
    )
    assert runs[0]["state"] == "constant_evaluation"
    assert runs[0]["termination"] is None
    assert runs[0]["candidate_kind"] == "constant_evaluation"
    assert runs[0]["backend"] is None
    stream = result.table("runtime.solve_variables")
    table = pa.RecordBatchReader.from_stream(stream).read_all()
    values = table.column("value").chunk(0)
    del model, prepared, result, handle, table
    stream.close()
    gc.collect()
    assert values.to_pylist() == [2.0]


@pytest.mark.unit
@pytest.mark.parametrize(
    "construct",
    [
        lambda: pse.SolveSettings(presolve=cast("PresolvePolicyKind", "magic")),
        lambda: pse.SolveSettings(controls=pse.SolveControls(time_limit=-1.0)),
        lambda: pse.SolveSettings(numerics=documents.NumericalPolicy(integrality=-1.0)),
        lambda: pse.SolveSettings(presolve_options={"max_passes": 3}),
        lambda: pse.SolveSettings(convexity_absolute=1e-9),
        lambda: pse.SolveSettings(
            controls=pse.SolveControls(options={"invalid": cast("str", object())})
        ),
    ],
)
def test_solver_profile_refuses_unsupported_or_partial_controls(
    runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    construct: Callable[[], pse.SolveSettings],
) -> None:
    # A document is refused where it enters native code: at decoding for a value outside
    # its type, and by admission for the rules that relate its fields. A native option
    # outside the typed union cannot even be encoded.
    with pytest.raises((pse.InspectionError, TypeError)):
        revision(runtime, physical).prepare_solve(declaration(101), construct())


@pytest.mark.unit
def test_cancelled_async_waiter_does_not_consume_terminal_result(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    prepared = revision(runtime, physical).prepare_solve(
        declaration(101),
        pse.SolveSettings(intent=NativeSolveIntent.ROOT),
    )
    handle = runtime.start([prepared] * 20)

    async def cancel_and_rejoin() -> pse.RunResult:
        waiter = asyncio.create_task(handle.wait_async())
        await asyncio.sleep(0)
        waiter.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await waiter
        return await handle.wait_async()

    result = asyncio.run(cancel_and_rejoin())
    terminal = handle.result()
    assert terminal is not None
    assert terminal.run_id == result.run_id
    runs = pa.RecordBatchReader.from_stream(
        result.table("runtime.solve_runs")
    ).read_all()
    assert runs.num_rows == 20


@pytest.mark.unit
def test_simulation_controls_round_trip_exact_native_options(
    runtime: pse.Runtime,
) -> None:

    settings = pse.SimulationSettings(
        start=0.0, end=1.0, samples=[0.0, 1.0], atol=[1e-8], parameter_scales=[1.0]
    )
    wire = json.loads(settings.to_json())
    wire["native"]["pi_control_proportional"] = 0.4
    restored = pse.SimulationSettings.from_json(json.dumps(wire))
    assert json.loads(restored.to_json())["native"]["pi_control_proportional"] == 0.4
    assert "diffsol" in {c.backend for c in runtime.capabilities()}
    wire["native"]["misspelled_option"] = 1
    with pytest.raises(pse.InspectionError):
        pse.SimulationSettings.from_json(json.dumps(wire))


@pytest.mark.unit
def test_fixed_fitting_sources_round_trip_and_use_shared_result_lifecycle(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:

    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/fit.pse": (
                    "package fitting { def Measurement { param length:Length=2{m}; "
                    "annotation check length(length>0{m}); } "
                    "entity kind origin provenance {attribute title:Text;} "
                    'entity origin experiment {title="known length"} '
                    "enum role {measured facets(measured)} "
                    "entity kind reading {attribute value:Length?;"
                    "attribute sigma:Length?;} "
                    '@id("9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d") '
                    "entity reading measured provenance(experiment,role.measured) "
                    "{value=2{m},sigma=0.1{m}} }"
                ),
            }
        ],
        physical,
    )
    case = next(
        row.declaration_id
        for row in package.declarations()
        if row.name == "Measurement"
    )
    fit = w.FitDeclaration(
        fit_id=FitId(identity(158)),
        parameters=(
            a.AuthoredFitCasesFieldParametersItem(
                symbol_id=identity(153),
                fixed=True,
                value=2.0,
                lower=0.0,
                upper=10.0,
                scale=1.0,
            ),
        ),
        experiments=(
            a.AuthoredFitCasesFieldExperimentsItem(
                experiment_id=InstanceId(identity(159)),
                case_id=case,
                route=ModelingAnalysisRoute.STEADY,
                bindings=(
                    a.AuthoredFitCasesFieldExperimentsItemBindingsItem(
                        parameter_id=identity(153), path="length"
                    ),
                ),
            ),
        ),
        observations=(
            a.AuthoredFitCasesFieldObservationsItem(
                observation_id=DeclarationId(identity(157)),
                value_attribute="value",
                standard_deviation_attribute="sigma",
                experiment_id=InstanceId(identity(159)),
                output_path="length",
                time=None,
                time_basis=None,
                time_unit_id=None,
                included=True,
                importance=1.0,
            ),
        ),
    )
    package = package.with_fit_declarations((fit,))
    job = package.prepare_fit(
        FitId(identity(158)),
        pse.FitPreparationDocument(
            solver=pse.SolveSettings(intent=NativeSolveIntent.OPTIMIZE)
        ),
    ).start()
    result = job.wait()
    rows = (
        pa.RecordBatchReader.from_stream(result.table("runtime.fit_observations"))
        .read_all()
        .to_pylist()
    )
    assert rows[0]["prediction"] == 2.0
    assert rows[0]["objective_contribution"] == 0.0
    assert job.wait().run_id == result.run_id
    assert not result.diagnostics()
    assert result.usable
    assert "authored.modeling_declarations" in result.tables()
    assert "authored.computation_models" not in result.tables()
    checks = (
        pa.RecordBatchReader.from_stream(result.table("runtime.modeling_checks"))
        .read_all()
        .to_pylist()
    )
    assert checks
    assert all(row["satisfied"] for row in checks)


@pytest.mark.unit
def test_completion_projection_and_pre_effect_publication_ticket(
    runtime: pse.Runtime,
    durable_runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    tmp_path: Path,
) -> None:
    prepared = revision(runtime, physical).prepare_solve(
        declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    )
    # An ephemeral run records nothing and cannot publish (ADR-0112 Outcome 16).
    assert not runtime.durable
    ephemeral = prepared.start().wait()
    assert ephemeral.attempt_id is None
    unregistered = pse.Workspace(
        workspace_id=identity(240).to_hex(),
        name="ephemeral",
        root_uri=tmp_path.as_uri() + "/",
    )
    with pytest.raises(pse.InspectionError, match="ephemeral"):
        ephemeral.prepare_publication(unregistered)
    with pytest.raises(pse.InspectionError, match="durable runtime"):
        runtime.runs()
    assert durable_runtime.durable
    handle = (
        revision(durable_runtime, physical)
        .prepare_solve(
            declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
        .start()
    )
    result = handle.wait()
    assert result.attempt_id is not None
    assert handle.attempt_id == result.attempt_id
    # The durable attempt is listed from the store, as the registry relation.
    (listed,) = durable_runtime.runs(run_id=result.run_id)
    assert isinstance(listed, pse.OperationalAttempt)
    assert listed.attempt_id == result.attempt_id
    assert listed.run_id == result.run_id
    assert listed.kind == AttemptKind.MODELING
    assert listed.state == AttemptState.COMPLETED
    assert listed.finished_at is not None
    failed = durable_runtime.runs(run_id=result.run_id, states=[AttemptState.FAILED])
    assert failed == ()
    completion = result.completion
    assert completion == result.completion
    converter = codec.converter()
    solves = (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_runs"))
        .read_all()
        .to_pylist()
    )
    assert completion.solves == tuple(
        converter.structure(row, result_contracts.RuntimeSolveRunsRow) for row in solves
    )
    lineage = (
        pa.RecordBatchReader.from_stream(result.table("runtime.run_lineage"))
        .read_all()
        .to_pylist()
    )
    assert completion.lineage == tuple(
        converter.structure(row, result_contracts.RuntimeRunLineageRow)
        for row in lineage
    )
    assert completion.computation is None
    assert completion.solves[0].candidate_kind == "constant_evaluation"
    assert completion.lineage[0].model_id == identity(101).to_hex()
    assert not result.diagnostics()
    durable_runtime.clear_program_cache()
    assert result.completion == completion
    workspace = durable_runtime.register_workspace(
        f"ticket-{result.run_id.to_hex()}", tmp_path
    )
    assert durable_runtime.workspace(workspace.name) == workspace
    assert (
        durable_runtime.head(WorkspaceId(SemanticId.from_hex(workspace.workspace_id)))
        is None
    )
    # The publication attempt is the durable attempt; the ticket exists before any
    # effect.
    attempt = result.prepare_publication(
        workspace, publication_id=PublicationId(identity(241))
    )
    ticket = attempt.ticket
    assert attempt.publication_id == identity(241)
    assert attempt.attempt_id == result.attempt_id
    wire = msgspec.json.decode(ticket.json, type=dict[str, object])
    candidate = cast("dict[str, object]", wire["candidate"])
    assert candidate["attempt_id"] == result.attempt_id.to_hex()
    assert candidate["publication_id"] == identity(241).to_hex()
    assert candidate["workspace_id"] == workspace.workspace_id
    assert not tuple(tmp_path.iterdir())
    # Nothing was registered: the catalog proves nothing was committed.
    settled = durable_runtime.settle_publication(ticket)
    assert isinstance(settled, pse.PublicationSettlementProvedNoncommit)
    assert settled == durable_runtime.settle_publication(ticket)
    assert not tuple(tmp_path.iterdir())


@pytest.mark.unit
@pytest.mark.parametrize(
    "source",
    [
        "package broken {def Root {var x:Length; eq invalid:x+ ==0{m};}}",
        "package broken {def Root {var :Length;}}",
    ],
)
def test_compiler_failure_retains_typed_authored_source_span(
    runtime: pse.Runtime, physical: pse.PhysicalContext, source: str
) -> None:
    with pytest.raises(pse.InspectionError) as failure:
        runtime.modeling_from_documents([package_documents(source)], physical)
    report = failure.value.report
    assert report.boundary_class == "invalid_model"
    assert report.source_locations, (report.message, report.rule, report.source_ids)
    location = report.source_locations[0]
    assert isinstance(location, pse.SourceLocation)
    assert location.start is not None
    assert location.end is not None
    assert 0 <= location.start <= location.end <= len(source)

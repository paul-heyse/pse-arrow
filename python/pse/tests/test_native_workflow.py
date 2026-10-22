# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public declaration, lifecycle and owned-result contract units."""

import asyncio
import contextlib
import dis
import gc
import sys
import threading
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
    AccuracyGoalStatus,
    CandidateQualifier,
    CandidateRefusal,
    CandidateUse,
    IncumbentPolicy,
    ModelingAnalysisRoute,
    Namespace,
    NativeAssurance,
    NativeBackend,
    NativeBoundaryClass,
    NativeCandidateKind,
    NativeQualification,
    NativeSolveIntent,
    NativeTermination,
    NumericalAccuracyClass,
    NumericalCompositionPolicy,
    NumericalEventKind,
    NumericalPosition,
    NumericalTransition,
    PhysicalInputKind,
    PresolvePolicyKind,
)
from pse.contracts.identities import (
    DeclarationId,
    FitId,
    InstanceId,
)
from pse.contracts.reference import ReferenceSchemaRelationsRow
from pse.contracts.values import SemanticId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.invalid_call import invalid_call
from pse.tests.physical_input_fixture import (
    coordinate_batch,
    primitive_quantity,
    stage_batch,
)

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


@pytest.mark.component
def test_runtime_startup_allows_python_thread_progress_inside_native_constructor(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    assert sys.implementation.name == "cpython"
    assert sys.version_info[:3] == (
        3,
        14,
        8,
    ), "the constructor frame witness requires the pinned CPython 3.14.8 interpreter"
    # Accept progress only while the public wrapper is executing its actual
    # native constructor call. An event set before calling Runtime, or thread
    # activity observed only after return, would not establish this boundary.
    # A warmed CALL_KW_NON_PY can hand off the GIL after Vectorcall returns while
    # f_lasti still names that call. A fresh code object retains the same globals,
    # defaults and behavior but resets specialization. In the pinned interpreter,
    # generic CALL_KW has no post-return periodic check; require it to stay generic.
    constructor = pse.Runtime.__init__.__code__.replace()
    monkeypatch.setattr(pse.Runtime.__init__, "__code__", constructor)
    instructions = iter(dis.get_instructions(constructor, adaptive=True))
    native_load = next(
        instruction
        for instruction in instructions
        if instruction.opname.startswith("LOAD_GLOBAL")
        and instruction.argval == "_NativeRuntime"
    )
    native_call = next(
        instruction
        for instruction in instructions
        if instruction.opname.startswith("CALL")
    )
    assert native_call.offset > native_load.offset
    assert native_call.opname == "CALL_KW"
    caller = threading.get_ident()
    ready = threading.Event()
    stop = threading.Event()
    progress: list[tuple[int, int]] = []

    def observe_constructor() -> None:
        ready.set()
        for _ in range(6000):
            if stop.wait(0.001):
                return
            frame = sys._current_frames().get(caller)
            while frame is not None:
                if frame.f_code is constructor and frame.f_lasti == native_call.offset:
                    # This work and its witness execute in Python while the caller
                    # remains inside the native call, rather than in a C-only peer.
                    progress.append(
                        (threading.get_ident(), sum(value for value in range(1024)))
                    )
                    return
                frame = frame.f_back

    observer = threading.Thread(target=observe_constructor)
    observer.start()
    try:
        assert ready.wait(5), "the observer must be ready before Runtime startup"
        runtime = canonical_substrate.runtime(inspection_settings)
        assert runtime.durable
    finally:
        stop.set()
        observer.join(5)
    assert not observer.is_alive()
    completed_call = next(
        instruction
        for instruction in dis.get_instructions(constructor, adaptive=True)
        if instruction.offset == native_call.offset
    )
    assert completed_call.opname == "CALL_KW", (
        "a specialized native call cannot establish the entered-constructor witness"
    )
    assert progress == [(observer.ident, 523776)]
    assert observer.ident != caller
    # The constructor also detaches for deployment acquisition and connection.
    # This control proves concurrency at that real boundary, not the particular
    # hashing phase. Source places startup observation in blocking(); its native
    # cancellation/charge controls independently establish settlement ownership.


@pytest.mark.integration
@pytest.mark.parametrize("context_selected", [False, True])
def test_explicit_cone_strategy_preserves_native_qualification(
    runtime: pse.Runtime, physical: pse.PhysicalContext, context_selected: bool
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
    settings = pse.SolveSettings(backend=NativeBackend.CLARABEL)
    prepared = runtime.prepare_conic(
        codec.decode_json(
            codec.encode_json(codec.convert_document(request, pse.ConicRequest)),
            pse.ConicRequest,
        ),
        physical,
        settings,
    )
    assert [route.backend for route in prepared.routes] == ["clarabel"]
    result = prepared.run(
        review_context=documents.ReviewContextRequest(
            steps=(),
            include_native_records=True,
            include_sources=True,
            maximum_bytes=16 * 1024 * 1024,
        )
        if context_selected
        else None
    )
    assert not result.failures()
    if context_selected:
        with pytest.raises(pse.InspectionError, match="context unavailable"):
            result.review_context()
    events = codec.structure_rows(
        pa.table(result.strategy_events()).to_pylist(),
        result_contracts.RuntimeSolveStrategyEventsRow,
    )
    assert events
    assert result.run_id is not None
    assert {event.run_id for event in events} == {result.run_id}
    assert (
        codec.structure_rows(
            pa.table(result.strategy_events()).to_pylist(),
            result_contracts.RuntimeSolveStrategyEventsRow,
        )
        == events
    )
    (row,) = result.attempts()
    attempt = row.report
    assert attempt is not None
    assert attempt.qualification == "optimal_within_tolerance"
    # This neutral-scalar cone has unit objective coordinates. Compare its
    # empirical oracle at the admitted gap resolution, retaining qualification.
    assert attempt.objective == pytest.approx(
        5.0, rel=settings.numerics.gap_relative, abs=settings.numerics.gap_absolute
    )
    primal_batch = attempt.primal()
    primal = codec.structure_rows(
        pa.table(primal_batch).to_pylist(), result_contracts.RuntimePrimalValuesRow
    )
    (coordinate,) = primal
    assert coordinate.coordinate_id == identity(201)
    assert (
        coordinate.value.quantity_type_id,
        coordinate.value.unit_id,
    ) == primitive_quantity("Scalar")
    assert coordinate.coordinate_role.value == "original"
    assert coordinate.value.value == pytest.approx(
        2.0, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    assert attempt.normalized_violation is not None
    assert attempt.normalized_violation <= 1.0


@pytest.mark.integration
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
    x = member.id
    prepared = package.prepare_solve(case, settings)
    assert prepared.eligibility
    start_input = coordinate_batch(
        runtime, [(x, 1.0, "Scalar")], kind=PhysicalInputKind.PRIMAL_VALUES
    )
    selected = prepared.with_primal_start(start_input)
    # Mutate the exported source's actual numeric buffer after admission. The
    # selected preparation must retain its owned immutable +1 snapshot.
    source_values = start_input.column("value").storage.field("value")
    source_buffer = source_values.buffers()[1]
    assert source_buffer.is_mutable
    memoryview(source_buffer).cast("d")[0] = -1.0
    del source_values, source_buffer, start_input
    gc.collect()
    explicit = selected.start().wait()
    resolved = [
        row
        for row in pa.table(explicit.table("runtime.resolved_numerics")).to_pylist()
        if row["target_kind"] in {"variable", "row"}
    ]
    assert len(resolved) == 2
    assert {row["target_kind"] for row in resolved} == {"variable", "row"}
    assert not settings.numerics.engineering_rules
    assert all(
        row["engineering"] is not None
        and row["engineering"]["relative_fraction"]
        == settings.numerics.engineering_relative_fraction
        for row in resolved
    )
    seed = explicit.available_start()
    assert seed is not None
    snapshot = seed.snapshot()
    assert isinstance(snapshot.payload, documents.WarmPayloadSnapshotRoot)
    assert isinstance(snapshot.payload.primal[0], documents.ObservationReal)
    assert snapshot.payload.primal[0].value == pytest.approx(
        2.0, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    assert snapshot.origin is not None
    assert snapshot.origin.run == explicit.run_id.to_hex()
    result = package.prepare_block_initialization(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.INITIALIZE,
            backend=NativeBackend.KINSOL,
            numerics=settings.numerics,
        ),
        stage_batch(runtime, [()]),
    ).run()
    stage = result.initialization()
    assert stage is not None
    assert stage.completed_stages == 1
    assert stage.original[x.to_hex()] == -1.0
    assert stage.solved_unknowns[x.to_hex()] == pytest.approx(
        -2.0, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    events = codec.structure_rows(
        pa.table(result.strategy_events()).to_pylist(),
        result_contracts.RuntimeSolveStrategyEventsRow,
    )
    assert events
    assert result.run_id is not None
    assert {event.run_id for event in events} == {result.run_id}
    assert (
        codec.structure_rows(
            pa.table(result.strategy_events()).to_pylist(),
            result_contracts.RuntimeSolveStrategyEventsRow,
        )
        == events
    )


@pytest.fixture
def runtime(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> pse.Runtime:
    return canonical_substrate.runtime(inspection_settings, ephemeral=True)


@pytest.fixture
def durable_runtime(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> pse.Runtime:
    return canonical_substrate.runtime(inspection_settings)


def physical_context(runtime: pse.Runtime) -> pse.PhysicalContext:
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


@pytest.fixture
def physical(runtime: pse.Runtime) -> pse.PhysicalContext:
    return physical_context(runtime)


def package_documents(source: str) -> dict[str, str]:
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    return {"package.toml": manifest, "models/fixed.pse": source}


@pytest.mark.unit
def test_contextual_accuracy_native_solve_admission_refuses_obsolete_request_version(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        old = msgspec.structs.replace(pse.SolveSettings(), version=3)
        with pytest.raises(pse.InspectionError, match="version"):
            revision(runtime, physical_context(runtime)).prepare_solve(
                declaration(101), old
            )
    finally:
        runtime.close()


@pytest.mark.integration
def test_contextual_accuracy_retained_rust_python_arrow_outcomes_match_without_new_work(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    package = runtime.modeling_from_documents(
        [
            package_documents(
                "package accuracy { def Root { var x:Scalar; eq balance:x==3; "
                "annotation start x(0); "
                "annotation accuracy_goal x(selected_output, steady, "
                "resolution=0.1); } }"
            )
        ],
        physical,
    )
    root = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    prepared = package.prepare_solve(
        root,
        pse.SolveSettings(
            intent=NativeSolveIntent.ROOT,
            backend=NativeBackend.KINSOL,
            presolve=PresolvePolicyKind.OFF,
        ),
    )
    result = runtime.start([prepared]).wait()
    assert result.usable
    completion_before = codec.encode_json(result.completion)
    work_before = (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_strategy_events"))
        .read_all()
        .to_pylist()
    )
    retained = result.accuracy_goals
    assert len(retained) == 1
    assert retained[0].status is AccuracyGoalStatus.SATISFIED
    assert retained[0].accuracy_class is NumericalAccuracyClass.ESTIMATED
    assert retained[0].error is not None
    assert 0 < retained[0].error <= 0.1
    rows = codec.structure_rows(
        pa.RecordBatchReader.from_stream(
            result.table("runtime.accuracy_goal_assessments")
        )
        .read_all()
        .to_pylist(),
        result_contracts.RuntimeAccuracyGoalAssessmentsRow,
    )
    assert tuple(rows) == codec.document_rows(
        retained, result_contracts.RuntimeAccuracyGoalAssessmentsRow
    )
    assert retained == result.completion.accuracy_goals
    assert result.accuracy_goals == retained
    assert codec.encode_json(result.completion) == completion_before
    assert (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_strategy_events"))
        .read_all()
        .to_pylist()
        == work_before
    )


@pytest.mark.integration
def test_declared_numerical_strategy_uses_original_permission_and_stops_unused_rung(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    """The public document controls actual execution without inventing Python policy."""
    package = runtime.modeling_from_documents(
        [
            package_documents(
                "package strategy {def Root {var x:Scalar; eq row:x==3; "
                "annotation start x(0);}}"
            )
        ],
        physical,
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    prepared = package.prepare_solve(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.ROOT,
            backend=NativeBackend.KINSOL,
            presolve=PresolvePolicyKind.OFF,
        ),
    )
    direct = prepared.numerical_strategy
    assert direct.version == 4
    assert prepared.composition_request.policy == NumericalCompositionPolicy.AUTO
    assert not prepared.composition_request.recovery
    limits = msgspec.structs.replace(direct.strategy.limits, attempts=2)
    first = msgspec.structs.replace(
        direct.strategy.mechanisms[0],
        profile=documents.ProfileRef(
            method=documents.NumericalMethodBackend(backend=NativeBackend.KINSOL),
            key=prepared.strategy_profile,
        ),
        transitions=(
            NumericalTransition.FINISH,
            NumericalTransition.RECOVER,
            NumericalTransition.STOP,
        ),
    )
    second = msgspec.structs.replace(first, position=NumericalPosition.RECOVERY)
    declaration = msgspec.structs.replace(
        direct,
        strategy=msgspec.structs.replace(
            direct.strategy, limits=limits, mechanisms=(first, second)
        ),
    )
    composed = prepared.with_numerical_strategy(declaration, (prepared, prepared))
    assert composed.numerical_strategy == declaration
    result = composed.start().wait()
    assert result.usable
    assert "runtime.solve_strategy_events" in result.tables()
    rows = pa.table(result.table("runtime.solve_strategy_events")).to_pylist()
    assert sum(row["kind"] == NumericalEventKind.STARTED for row in rows) == 1
    finished = [
        row
        for row in rows
        if row["kind"] == NumericalEventKind.FINISHED and row["transition"] is not None
    ]
    assert len(finished) == 1
    assert finished[0]["transition"] == NumericalTransition.FINISH
    assert finished[0]["original_conclusion"] == "satisfied"
    charged = [
        row["charging_owner"] for row in rows if row["charging_owner"] is not None
    ]
    assert len(charged) == len(set(charged))


def revision(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> pse.ModelingPackage:
    source = (
        f'package atomic {{ @id("{identity(101).to_hex()}") test Fixed '
        "fixture {dof 0; fix x=2{m};} "
        "{var x:Length; annotation bounds x(0{m},10{m});} }"
    )
    return runtime.modeling_from_documents([package_documents(source)], physical)


@pytest.mark.component
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


@pytest.mark.component
def test_native_capability_discovery_and_hard_cut(runtime: pse.Runtime) -> None:
    capabilities = runtime.capabilities()
    assert capabilities
    assert "clarabel" in {c.backend for c in capabilities}
    assert all(c.classes and c.reuse and c.cancellation for c in capabilities)
    for capability in capabilities:
        pse.SolveSettings(backend=NativeBackend(capability.backend))
    assert not hasattr(pse, "probe_host")
    assert not hasattr(pse, "HostCapabilities")


@pytest.mark.integration
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


@pytest.mark.integration
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


@pytest.mark.component
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


@pytest.mark.integration
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


@pytest.mark.component
def test_simulation_controls_round_trip_exact_native_options(
    runtime: pse.Runtime,
) -> None:

    settings = pse.SimulationSettings(
        runtime=runtime,
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=coordinate_batch(runtime),
        parameter_scales=coordinate_batch(runtime),
    )
    configured = msgspec.structs.replace(
        settings.document.controls.idas, native_initial_step="configured"
    )
    document = msgspec.structs.replace(
        settings.document,
        controls=msgspec.structs.replace(settings.document.controls, idas=configured),
    )
    restored = pse.SimulationSettings.from_document(runtime, document)
    assert restored.document == document
    assert restored.idas.native_initial_step == "configured"
    assert "diffsol" in {c.backend for c in runtime.capabilities()}
    with pytest.raises(TypeError, match="PhysicalSimulationRequest"):
        invalid_call(
            pse.SimulationSettings.from_document, runtime, codec.encode_json(document)
        )


def _measurement_fit_package(
    runtime: pse.Runtime, physical: pse.PhysicalContext, *, fixed: bool
) -> pse.ModelingPackage:
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
                fixed=fixed,
                value=2.0 if fixed else 1.0,
                lower=0.0 if fixed else None,
                upper=10.0 if fixed else None,
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
    return package.with_fit_declarations((fit,))


@pytest.mark.integration
def test_residual_lm_public_result_exposes_actual_stopping_and_original_qualification(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    package = _measurement_fit_package(runtime, physical, fixed=False)
    prepared = package.prepare_fit(
        FitId(identity(158)),
        pse.PhysicalFitRequest(
            settings=documents.FitPreparationDocumentForPhysicalSimulationRequest(
                optimizer="residual_lm",
                solver=pse.SolveSettings(intent=NativeSolveIntent.OPTIMIZE),
            ),
        ),
    )
    result = prepared.start().wait()
    execution = result.fit_execution
    assert isinstance(execution, documents.FitExecutionDocumentResidualLm)
    report = execution.report
    assert report.failure is None
    assert report.qualification is NativeQualification.STATIONARY
    assert report.stationarity is not None
    assert result.usable
    stopping = report.stopping
    assert stopping is not None
    assert stopping.criteria.version == 1
    for threshold in (
        stopping.criteria.relative_cost_change,
        stopping.criteria.relative_step,
        stopping.criteria.column_residual_cosine,
    ):
        assert threshold == 30.0 * sys.float_info.epsilon
    assert 0 < report.residual_evaluations <= stopping.residual_evaluation_limit
    assert report.parameter_updates <= stopping.parameter_update_limit
    assert codec.decode_json(codec.encode_json(execution), type(execution)) == execution
    rows = (
        pa.RecordBatchReader.from_stream(result.table("runtime.fit_observations"))
        .read_all()
        .to_pylist()
    )
    assert rows[0]["prediction"] == pytest.approx(2.0)
    assert rows[0]["objective_contribution"] == pytest.approx(0.0, abs=1e-20)


@pytest.mark.integration
def test_fixed_fitting_sources_round_trip_and_use_shared_result_lifecycle(
    runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    inspection_settings: pse.EngineSettings,
) -> None:
    package = _measurement_fit_package(runtime, physical, fixed=True)
    prepared = package.prepare_fit(
        FitId(identity(158)),
        codec.decode_json(
            codec.encode_json(
                {
                    "version": 1,
                    "settings": {
                        "solver": pse.SolveSettings(intent=NativeSolveIntent.OPTIMIZE),
                    },
                }
            ),
            pse.PhysicalFitRequest,
        ),
    )
    direct = prepared.numerical_strategy
    assert direct.version == 4
    assert prepared.composition_request.policy == NumericalCompositionPolicy.AUTO
    assert codec.decode_json(codec.encode_json(direct), type(direct)) == direct
    assert len(direct.strategy.mechanisms) == 1
    assert direct.strategy.mechanisms[0].support
    assert prepared.strategy_profile.to_prefixed()
    with pytest.raises(
        pse.InspectionError, match="numerical composition requires an algebraic solve"
    ):
        prepared.with_numerical_strategy(direct, (prepared,))
    job = prepared.start()
    result = job.wait()
    rows = (
        pa.RecordBatchReader.from_stream(result.table("runtime.fit_observations"))
        .read_all()
        .to_pylist()
    )
    assert rows[0]["prediction"] == 2.0
    assert rows[0]["objective_contribution"] == 0.0
    assert isinstance(result.fit_execution, documents.FitExecutionDocumentEvaluation)
    assert job.wait().run_id == result.run_id
    assert not result.diagnostics()
    assert result.usable
    with (
        pse.registry_table(
            "reference.schema_relations", settings=inspection_settings
        ) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        relations = codec.structure_rows(
            reader.read_all().to_pylist(), ReferenceSchemaRelationsRow
        )
    fit_relation = next(
        relation.relation_id
        for relation in relations
        if relation.namespace is Namespace.AUTHORED and relation.name == "fit_cases"
    )
    sources = package.source_tables()
    fits = codec.structure_rows(
        pa.table(sources[fit_relation]).to_pylist(), a.AuthoredFitCasesRow
    )
    assert len(fits) == 1
    assert fits[0].fit_id == FitId(identity(158))
    assert len(fits[0].parameters) == 1
    assert fits[0].parameters[0].fixed
    assert fits[0].parameters[0].value == 2.0
    for stream in sources.values():
        stream.close()
    assert "runtime.fit_observations" in result.tables()
    checks = (
        pa.RecordBatchReader.from_stream(result.table("runtime.modeling_checks"))
        .read_all()
        .to_pylist()
    )
    assert checks
    assert all(row["satisfied"] for row in checks)

    events = (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_strategy_events"))
        .read_all()
        .to_pylist()
    )
    assert all(event["decision_identity"] is not None for event in events)
    assert events[-1]["transition"] == NumericalTransition.FINISH.value
    assert events[-1]["evaluations"] is None


@pytest.mark.integration
def test_completion_projection_and_canonical_admission(
    runtime: pse.Runtime,
    durable_runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    tmp_path: Path,
) -> None:
    prepared = revision(runtime, physical).prepare_solve(
        declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    )
    # Ephemeral numerical execution is explicit; ordinary deployment persists.
    assert not runtime.durable
    ephemeral = prepared.start().wait()
    assert ephemeral.canonical_run_key is None
    assert ephemeral.canonical_attempt_key is None
    assert durable_runtime.durable
    handle = (
        revision(durable_runtime, physical)
        .prepare_solve(
            declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
        .start()
    )
    result = handle.wait()
    assert result.canonical_run_key is not None
    assert result.canonical_attempt_key is not None
    assert handle.canonical_run_key == result.canonical_run_key
    assert handle.canonical_attempt_key == result.canonical_attempt_key
    (listed,) = pa.table(
        durable_runtime.attempt_record(result.canonical_attempt_key)
    ).to_pylist()
    assert listed["key"] == result.canonical_attempt_key
    assert listed["run"] == result.canonical_run_key
    assert listed["terminal"]
    assert listed["outcome"] == "succeeded"
    completion = result.completion
    assert completion == result.completion
    converter = codec.converter()
    solves = (
        pa.RecordBatchReader.from_stream(result.table("runtime.solve_runs"))
        .read_all()
        .to_pylist()
    )
    assert codec.document_rows(
        completion.solves, result_contracts.RuntimeSolveRunsRow
    ) == tuple(
        converter.structure(row, result_contracts.RuntimeSolveRunsRow) for row in solves
    )
    lineage = (
        pa.RecordBatchReader.from_stream(result.table("runtime.run_lineage"))
        .read_all()
        .to_pylist()
    )
    assert codec.document_rows(
        completion.lineage, result_contracts.RuntimeRunLineageRow
    ) == tuple(
        converter.structure(row, result_contracts.RuntimeRunLineageRow)
        for row in lineage
    )
    assert completion.computation is None
    assert completion.solves[0].candidate_kind == "constant_evaluation"
    assert completion.lineage[0].model_id == declaration(101)
    assert not result.diagnostics()
    durable_runtime.clear_program_cache()
    assert result.completion == completion
    stored = pa.table(
        durable_runtime.results(
            result.canonical_run_key, result.canonical_attempt_key, "runtime.solve_runs"
        )
    ).to_pylist()
    assert stored == solves
    destination = tmp_path / "solve.arrow"
    durable_runtime.export_results(
        result.canonical_run_key,
        result.canonical_attempt_key,
        "runtime.solve_runs",
        destination,
    )
    with pa.ipc.open_stream(destination) as reader:
        assert reader.read_all().to_pylist() == solves


def price_taker_package(
    runtime: pse.Runtime,
) -> tuple[w.ModelingPackage, DeclarationId]:
    root = Path(__file__).resolve().parents[3]
    physical_root = root / "packages/reference/physical"
    physical = runtime.physical_from_documents(
        {
            str(path.relative_to(physical_root)): path.read_text()
            for path in physical_root.rglob("*")
            if path.is_file() and path.suffix in {".toml", ".yaml", ".yml", ".pse"}
        }
    )
    physical_manifest = msgspec.toml.decode(
        (physical_root / "package.toml").read_bytes(),
        type=dict[str, dict[str, object]],
    )["package"]
    physical_id = cast("str", physical_manifest["package_id"])
    major, minor, patch = map(int, cast("str", physical_manifest["version"]).split("."))
    dependency = (
        f'dependencies = [{{ package_id = "{physical_id}", '
        f'version_req = {{ operator = "exact", major = {major}, '
        f"minor = {minor}, patch = {patch} }} }}]"
    )
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", dependency)
    )
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/price-taker.pse": (
                    root / "tests/fixtures/models/native-price-taker.pse"
                ).read_text(),
            }
        ],
        physical,
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package, case


@pytest.mark.integration
@pytest.mark.parametrize(
    "incumbent",
    [IncumbentPolicy.ACCEPT_FEASIBLE, IncumbentPolicy.REFUSE],
)
def test_durable_limited_native_incumbent_is_feasible_and_nonoptimal(
    durable_runtime: pse.Runtime,
    incumbent: IncumbentPolicy,
) -> None:
    package, case = price_taker_package(durable_runtime)
    settings = pse.SolveSettings(
        intent=NativeSolveIntent.OPTIMIZE,
        backend=NativeBackend.SCIP,
        controls=pse.SolveControls(
            foreign_bytes=16 << 20,
            options={"limits/solutions": 1},
        ),
        numerics=documents.NumericalPolicy(incumbent=incumbent),
    )
    handle = package.prepare_solve(case, settings).start()
    result = handle.wait()
    assert result.canonical_run_key is not None, tuple(
        diagnostic.message for diagnostic in result.diagnostics()
    )
    assert result.canonical_attempt_key is not None
    assert handle.canonical_attempt_key == result.canonical_attempt_key
    (stored,) = pa.table(
        durable_runtime.attempt_record(result.canonical_attempt_key)
    ).to_pylist()
    assert stored["key"] == result.canonical_attempt_key
    assert stored["terminal"]
    (solve,) = result.completion.solves
    assert solve.backend == NativeBackend.SCIP
    assert solve.termination == NativeTermination.SOLUTION_LIMIT
    # The limited SCIP search retains the actual fixed-assignment Ipopt iterate;
    # original feasibility and incumbent permission are assessed separately below.
    assert solve.candidate_kind == NativeCandidateKind.FINAL_ITERATE
    assert solve.commitment is not None
    assert solve.commitment
    assert solve.feasible is True
    assert solve.qualification == NativeQualification.FEASIBLE
    assert solve.assurance != NativeAssurance.GLOBAL_BOUND
    assert solve.validation_error is None
    # The shared authored MIQP's independent enumeration gives an optimum of 22 W;
    # the first feasible native incumbent is explicitly below that optimum.
    assert solve.objective is not None
    metrics = {
        row["name"]: row["real"]
        for row in pa.table(result.table("runtime.solve_metrics")).to_pylist()
        if row["namespace"] == "metric"
    }
    gap_absolute = metrics["global.gap_absolute"]
    gap_relative = metrics["global.gap_relative"]
    assert gap_absolute is not None
    assert gap_absolute > 0.0
    assert gap_relative is not None
    assert gap_relative > 0.0
    allowance = max(gap_absolute, gap_relative * min(abs(solve.objective), 22.0))
    assert solve.objective < 22.0 - allowance
    (assessment,) = result.completion.assessments
    assert assessment.native_termination == NativeTermination.SOLUTION_LIMIT
    assert assessment.numerically_feasible is True
    assert assessment.validated is True
    assert assessment.qualification == NativeQualification.FEASIBLE
    assert assessment.incumbent_policy == incumbent
    if incumbent == IncumbentPolicy.ACCEPT_FEASIBLE:
        assert result.usable
        assert assessment.usability == CandidateUse.USABLE
        assert assessment.permits_result
        assert CandidateQualifier.ACCEPTED_INCUMBENT_FEASIBLE in assessment.qualifiers
        assert stored["outcome"] == "succeeded"
    else:
        assert not result.usable
        assert assessment.usability == CandidateUse.SEED_ONLY
        assert not assessment.permits_result
        assert CandidateRefusal.INCUMBENT_REFUSED in assessment.refusals
        assert stored["outcome"] == "failed"
    assert assessment.permits_seed
    variables = pa.table(result.table("runtime.solve_variables")).to_pylist()
    candidates = [row for row in variables if not row["parameter"]]
    assert len(candidates) == 9
    assert all(row["value"] is not None for row in candidates)
    constraints = pa.table(result.table("runtime.solve_constraints")).to_pylist()
    assert len(constraints) == 10
    for row in constraints:
        assert row["quantity_id"] is not None
        assert row["unit_id"] is not None
        assert row["value"] is not None
        assert row["tolerance"] is not None
        assert row["lower_violation"] <= row["tolerance"]
        assert row["upper_violation"] <= row["tolerance"]
        if row["equality_residual"] is not None:
            assert abs(row["equality_residual"]) <= row["tolerance"]


@pytest.mark.integration
def test_durable_native_memory_stop_keeps_unavailable_feasibility(
    durable_runtime: pse.Runtime,
) -> None:
    package, case = price_taker_package(durable_runtime)
    prepared = package.prepare_solve(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.OPTIMIZE,
            backend=NativeBackend.SCIP,
            controls=pse.SolveControls(foreign_bytes=1 << 20),
        ),
    )
    handle = prepared.start()
    result = handle.wait()
    assert result.canonical_run_key is not None, tuple(
        diagnostic.message for diagnostic in result.diagnostics()
    )
    assert result.canonical_attempt_key is not None
    assert handle.canonical_attempt_key == result.canonical_attempt_key
    (stored,) = pa.table(
        durable_runtime.attempt_record(result.canonical_attempt_key)
    ).to_pylist()
    assert stored["key"] == result.canonical_attempt_key
    assert stored["terminal"]
    assert stored["outcome"] == "failed"
    (persisted,) = pa.table(
        durable_runtime.results(
            result.canonical_run_key, result.canonical_attempt_key, "runtime.solve_runs"
        )
    ).to_pylist()
    assert persisted["termination"] == NativeTermination.RESOURCE_EXHAUSTED.value
    (solve,) = result.completion.solves
    assert solve.backend == NativeBackend.SCIP
    assert solve.native_status == "SCIP_STATUS_MEMLIMIT"
    assert solve.termination == NativeTermination.RESOURCE_EXHAUSTED
    assert solve.candidate_kind is None
    assert solve.feasible is None
    assert solve.objective is None
    assert solve.qualification == NativeQualification.UNQUALIFIED
    assert solve.assurance == NativeAssurance.NONE
    assert not result.usable
    (assessment,) = result.completion.assessments
    assert assessment.numerically_feasible is None
    assert assessment.validated is False
    assert assessment.qualification == NativeQualification.UNQUALIFIED
    assert CandidateRefusal.NO_CANDIDATE in assessment.refusals
    assert CandidateRefusal.FEASIBILITY_UNAVAILABLE in assessment.refusals
    assert CandidateRefusal.INFEASIBLE not in assessment.refusals
    assert not assessment.permits_result
    assert not assessment.permits_seed
    diagnostics = result.completion.diagnostics
    assert any(d.class_ == NativeBoundaryClass.RESOURCE_LIMIT for d in diagnostics)
    retained_assessments = pa.table(
        durable_runtime.results(
            result.canonical_run_key,
            result.canonical_attempt_key,
            "runtime.candidate_assessments",
        )
    )
    assert tuple(
        codec.structure_rows(
            retained_assessments.to_pylist(),
            result_contracts.RuntimeCandidateAssessmentsRow,
        )
    ) == codec.document_rows(
        result.completion.assessments, result_contracts.RuntimeCandidateAssessmentsRow
    )
    retained_findings = pa.table(
        durable_runtime.results(
            result.canonical_run_key,
            result.canonical_attempt_key,
            "runtime.modeling_findings",
        )
    )
    assert retained_findings.equals(pa.table(result.table("runtime.modeling_findings")))
    assert any(
        row.class_ == NativeBoundaryClass.RESOURCE_LIMIT
        for row in codec.structure_rows(
            retained_findings.to_pylist(), result_contracts.RuntimeModelingFindingsRow
        )
    )
    variables = pa.table(result.table("runtime.solve_variables")).to_pylist()
    candidates = [row for row in variables if not row["parameter"]]
    assert len(candidates) == 9
    assert all(row["value"] is None for row in candidates)


@pytest.mark.component
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
    assert isinstance(location, documents.SourceLocationOutput)
    assert location.start is not None
    assert location.end is not None
    assert 0 <= location.start <= location.end <= len(source)


@pytest.mark.component
def test_parsed_reload_noop_accepts_text_and_bytes_on_exact_package_handle(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    sources = package_documents("package reload {param a:Scalar=1;}")
    package = runtime.modeling_from_documents([sources], physical)
    original = package.declarations()
    same_text = package.reload_from_parent([sources])
    same_bytes = package.reload_from_parent(
        [{path: text.encode("utf-8") for path, text in sources.items()}]
    )
    assert same_text.revision_id == package.revision_id
    assert same_bytes.revision_id == package.revision_id
    assert same_bytes.canonical_problem == package.canonical_problem
    assert same_bytes.declarations() == original


@pytest.mark.component
def test_parsed_reload_complete_order_and_removal_match_independent_fresh_ingress(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    def inventory(
        current: pse.ModelingPackage,
    ) -> dict[DeclarationId, a.AuthoredModelingDeclarationsRow]:
        rows = current.declarations()
        keyed = {row.declaration_id: row for row in rows}
        assert len(keyed) == len(rows), "declaration identities must be unique"
        return keyed

    sources = package_documents("package reload {param a:Scalar=1;}")
    package = runtime.modeling_from_documents([sources], physical)
    original = package.declarations()
    original_source = package.knowledge().source_revision
    expanded = dict(sources)
    expanded["models/a.pse"] = "package earlier {param b:Scalar=2;}"
    changed = package.reload_from_parent([expanded])
    fresh = runtime.modeling_from_documents([expanded], physical)
    assert changed.canonical_problem == package.canonical_problem
    assert changed.revision_id != package.revision_id
    # Public inventories follow membership keys, not parsed document order.
    # Compare every authored field by identity; V4 separately witnesses the exact
    # ordered source frame, document content and admitted physical context.
    assert inventory(changed) == inventory(fresh)
    changed_source = changed.knowledge().source_revision
    assert changed_source == fresh.knowledge().source_revision
    assert changed_source != original_source
    assert any(
        row.name == "earlier" and row.parent_id is None
        for row in changed.declarations()
    )
    assert package.declarations() == original
    assert package.knowledge().source_revision == original_source
    restored = changed.reload_from_parent([sources])
    assert restored.canonical_problem == package.canonical_problem
    assert inventory(restored) == inventory(package)
    assert restored.knowledge().source_revision == original_source
    assert all(row.name != "earlier" for row in restored.declarations())
    assert inventory(changed) == inventory(fresh), "expanded history remains immutable"
    assert changed.knowledge().source_revision == changed_source


@pytest.mark.component
def test_parsed_reload_failure_and_wrong_parent_delta_preserve_original_revision(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    sources = package_documents("package reload {param a:Scalar=1;}")
    package = runtime.modeling_from_documents([sources], physical)
    revision_key = package.revision_id
    original = package.declarations()
    malformed = dict(sources)
    malformed["models/fixed.pse"] = "package reload {def Broken {eq e:1 +;}}"
    with pytest.raises(pse.InspectionError):
        package.reload_from_parent([malformed])
    with pytest.raises(pse.InspectionError):
        package.with_delta(documents.SourceDelta(parent="different-canonical-parent"))
    assert package.revision_id == revision_key
    assert package.declarations() == original
    assert package.reload_from_parent([sources]).revision_id == revision_key

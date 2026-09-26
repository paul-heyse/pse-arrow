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
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import NativeVariableDomain
from pse.contracts.values import ContentHash, SemanticId, SourceSpan


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


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
        "quadratic": {"m": 1, "n": 1, "colptr": [0, 0], "rowval": [], "nzval": []},
        "objective": [1.0],
        "constraints": {
            "m": 1,
            "n": 1,
            "colptr": [0, 1],
            "rowval": [0],
            "nzval": [-1.0],
        },
        "rhs": [-2.0],
        "cones": [{"NonnegativeConeT": 1}],
        "objective_constant": 3.0,
        "gram_factors": [],
        "gram_weights": [],
    }
    prepared = runtime.prepare_conic(
        request, physical, pse.SolveSettings(backend="clarabel")
    )
    assert prepared.routes == ("Native(Clarabel)",)
    result = prepared.run()
    assert not result.failures()
    (attempt,) = result.attempts()
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
    model = runtime.model(identity(210), "two roots", physical)
    model.definition(
        w.Definition(
            definition_id=identity(211),
            sources=("x*x",),
            formals=(w.Formal(path="x", quantity_id=identity(31)),),
            domains=(),
            groups=(),
            providers=(),
            units=(),
            literals=(),
        )
    )
    case = pse.CaseBuilder.create(identity(212), "roots")
    case.variable(
        w.Variable(
            port=w.Port(
                symbol_id=identity(213), quantity_id=identity(31), unit_id=identity(10)
            ),
            fixed=False,
            domain=NativeVariableDomain.CONTINUOUS,
            lower=None,
            upper=None,
        ),
        -1.0,
    )
    case.row(
        w.Row(row_id=identity(214), quantity_id=identity(31), lower=4.0, upper=4.0)
    )
    case.instance(
        w.Instance(
            instance_id=identity(215),
            definition_id=identity(211),
            slots=(
                w.Slot(
                    source_id=identity(213),
                    formal_quantity_id=identity(31),
                    formal_unit_id=identity(10),
                ),
            ),
            contributions=(w.Contribution(output=0, row_id=identity(214), scale=1.0),),
        )
    )
    revision = model.case(case).freeze()
    settings = pse.SolveSettings(intent="root", backend="kinsol", presolve="off")
    prepared = revision.prepare(identity(212), settings)
    assert prepared.eligibility
    explicit = prepared.with_primal_start({identity(213): 1.0}).start().wait()
    seed = explicit.available_start()
    assert seed is not None
    snapshot = json.loads(seed.snapshot_json())
    assert snapshot["payload"]["primal"][0] == pytest.approx(2.0, abs=1e-6)
    assert snapshot["origin"]["run"] == explicit.run_id.to_hex()
    result = revision.prepare_initialization(
        identity(212), pse.SolveSettings(intent="initialize", backend="kinsol"), [{}]
    ).run()
    stage = result.initialization()
    assert stage is not None
    assert stage["completed_stages"] == 1
    assert cast("dict[str, float]", stage["original"])[identity(213).to_hex()] == -1.0
    assert cast("dict[str, float]", stage["solved_unknowns"])[
        identity(213).to_hex()
    ] == pytest.approx(-2.0, abs=1e-6)


@pytest.fixture(scope="module")
def runtime(inspection_settings: pse.EngineSettings) -> pse.Runtime:
    return pse.Runtime(inspection_settings)


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


def revision(runtime: pse.Runtime, physical: pse.PhysicalContext) -> pse.ModelRevision:
    # An all-fixed length uses the public lifecycle without a native solver.
    case = pse.CaseBuilder.create(identity(101), "fixed length")
    case.variable(
        w.Variable(
            port=w.Port(
                symbol_id=identity(102), quantity_id=identity(30), unit_id=identity(1)
            ),
            fixed=True,
            domain=NativeVariableDomain.CONTINUOUS,
            lower=0.0,
            upper=10.0,
        ),
        2.0,
    )
    return (
        runtime.model(identity(100), "declaration unit", physical).case(case).freeze()
    )


@pytest.mark.unit
def test_document_frontdoor_refuses_packages_without_native_declarations(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    manifest = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text()
    with pytest.raises(pse.InspectionError):
        runtime.models_from_documents({"package.toml": manifest}, physical)


@pytest.mark.unit
def test_native_capability_discovery_and_hard_cut(runtime: pse.Runtime) -> None:
    capabilities = runtime.capabilities()
    assert capabilities
    assert "clarabel" in {c.backend for c in capabilities}
    assert all(c.classes and c.reuse and c.cancellation for c in capabilities)
    for capability in capabilities:
        pse.SolveSettings(backend=capability.backend)
    assert not hasattr(pse, "probe_host")
    assert not hasattr(pse, "HostCapabilities")


@pytest.mark.unit
def test_selected_admission_preserves_structured_source_diagnostics(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    manifest = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/minimal_explicit/package.toml"
    ).read_text()
    model_id = "01010101010101010101010101010101"
    root_id = "03030303030303030303030303030303"
    document = json.dumps(
        {
            "computation_models": [
                {
                    "model_id": model_id,
                    "name": "missing root",
                    "definitions": [],
                    "domains": [],
                    "groups": [],
                    "cases": [
                        {
                            "case_id": "02020202020202020202020202020202",
                            "name": "selected",
                            "variables": [],
                            "parameters": [],
                            "instances": [],
                            "rows": [],
                            "objective": None,
                            "values": [],
                        }
                    ],
                }
            ],
            "model_compositions": [{"model_id": model_id, "root_instance_id": root_id}],
        }
    )
    with pytest.raises(pse.InspectionError) as error:
        runtime.models_from_documents(
            {
                "package.toml": manifest,
                "computation_models/model.yaml": document,
            },
            physical,
        )
    report = error.value.report
    assert report.boundary_class == "invalid_model"
    assert report.stage == "selected_admission"
    assert report.rule is not None
    assert root_id in report.source_ids
    assert report.observations == ()


@pytest.mark.unit
def test_revision_edit_is_atomic_and_has_no_python_math(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    model = revision(runtime, physical)
    draft = model.edit()
    draft.declaration = attrs.evolve(
        draft.declaration, cases=(*draft.declaration.cases, *draft.declaration.cases)
    )
    with pytest.raises(pse.InspectionError):
        draft.freeze()
    assert model.edit().freeze().identity == model.identity
    assert model.declaration.cases[0].values[0].value == 2.0


@pytest.mark.unit
def test_blocking_async_share_terminal_report_and_last_array_owner(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    model = revision(runtime, physical)
    settings = pse.SolveSettings(intent="root")
    prepared = model.prepare(identity(101), settings)
    assert prepared.route == "Constant"
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
        lambda: pse.SolveSettings(presolve="magic"),
        lambda: pse.SolveSettings(time_limit=-1.0),
        lambda: pse.SolveSettings(numerics={"integrality": -1.0}),
        lambda: pse.SolveSettings(
            options={"invalid": cast("str", object())},
        ),
    ],
)
def test_solver_profile_refuses_unsupported_or_partial_controls(
    construct: Callable[[], pse.SolveSettings],
) -> None:
    with pytest.raises((pse.InspectionError, TypeError, ValueError)):
        construct()


@pytest.mark.unit
def test_cancelled_async_waiter_does_not_consume_terminal_result(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    prepared = revision(runtime, physical).prepare(
        identity(101),
        pse.SolveSettings(intent="root"),
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

    model = runtime.model(identity(150), "fit length", physical)
    model.definition(
        w.Definition(
            definition_id=identity(151),
            sources=("length",),
            formals=(w.Formal(path="length", quantity_id=identity(30)),),
            domains=(),
            groups=(),
            providers=(),
            units=(),
            literals=(),
        )
    )
    case = pse.CaseBuilder.create(identity(152), "measurement")
    case.parameter(
        w.Parameter(
            symbol_id=identity(153), quantity_id=identity(30), unit_id=identity(1)
        ),
        2.0,
    )
    case.row(
        w.Row(row_id=identity(154), quantity_id=identity(30), lower=None, upper=None)
    )
    case.instance(
        w.Instance(
            instance_id=identity(155),
            definition_id=identity(151),
            slots=(
                w.Slot(
                    source_id=identity(153),
                    formal_quantity_id=identity(30),
                    formal_unit_id=identity(1),
                ),
            ),
            contributions=(w.Contribution(output=0, row_id=identity(154), scale=1.0),),
        )
    )
    model.case(case)
    model.dataset(
        a.AuthoredDatasetsRow(
            dataset_id=identity(156),
            name="known",
            source="unit",
            content_hash=ContentHash(bytes([2]) * 32),
        )
    )
    model.observation(
        a.AuthoredObservationsRow(
            observation_id=identity(157),
            dataset_id=identity(156),
            target="length",
            value=2.0,
            unit_id=identity(1),
            std_dev=0.1,
            timestamp=None,
            tag=None,
            source_span=SourceSpan(document_id=identity(156), start=0, end=0),
        )
    )
    model.fit(
        w.FitDeclaration(
            fit_id=identity(158),
            model_id=identity(150),
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
                    experiment_id=identity(159), case_id=identity(152), dynamic_id=None
                ),
            ),
            observations=(
                a.AuthoredFitCasesFieldObservationsItem(
                    observation_id=identity(157),
                    experiment_id=identity(159),
                    output_id=identity(154),
                    time=None,
                    time_basis=None,
                    time_unit_id=None,
                    included=True,
                    importance=1.0,
                ),
            ),
        )
    )
    revision = model.freeze()
    assert revision.edit().freeze().identity == revision.identity
    job = revision.prepare_fit(
        identity(158),
        pse.SolveSettings(intent="optimize"),
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


@pytest.mark.unit
def test_completion_projection_and_pre_effect_publication_ticket(
    runtime: pse.Runtime, physical: pse.PhysicalContext, tmp_path: Path
) -> None:
    result = (
        revision(runtime, physical)
        .prepare(identity(101), pse.SolveSettings(intent="root"))
        .start()
        .wait()
    )
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
    assert completion.lineage[0].model_id == identity(100)
    assert not result.diagnostics()
    runtime.clear_program_cache()
    assert result.completion == completion
    request = pse.PublicationRequest(
        tmp_path.as_uri() + "/", identity(240), identity(241), identity(242)
    )
    attempt = result.prepare_publication_request(request)
    ticket = attempt.ticket
    assert attempt.publication_id == request.publication_id
    assert attempt.attempt_id == request.attempt_id
    wire = msgspec.json.decode(ticket.json, type=dict[str, object])
    candidate = cast("dict[str, object]", wire["candidate"])
    assert candidate["attempt_id"] == request.attempt_id.to_hex()
    assert candidate["publication_id"] == request.publication_id.to_hex()
    assert not tuple(tmp_path.iterdir())
    settled = runtime.settle_publication(ticket)
    assert isinstance(settled, pse.PublicationUnresolved)
    assert settled == runtime.settle_publication(ticket)
    assert not tuple(tmp_path.iterdir())


@pytest.mark.unit
def test_compiler_failure_retains_typed_authored_source_span(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> None:
    model = revision(runtime, physical).edit()
    model.definition(
        w.Definition(
            definition_id=identity(243),
            sources=("x +",),
            formals=(w.Formal(path="x", quantity_id=identity(30)),),
            domains=(),
            groups=(),
            providers=(),
            units=(),
            literals=(),
        )
    )
    case = model.declaration.cases[0]
    case = attrs.evolve(
        case,
        instances=(
            w.Instance(
                instance_id=identity(244),
                definition_id=identity(243),
                slots=(
                    w.Slot(
                        source_id=identity(102),
                        formal_quantity_id=identity(30),
                        formal_unit_id=identity(1),
                    ),
                ),
                contributions=(
                    w.Contribution(output=0, row_id=identity(245), scale=1.0),
                ),
            ),
        ),
        rows=(
            w.Row(
                row_id=identity(245), quantity_id=identity(30), lower=0.0, upper=10.0
            ),
        ),
    )
    model.declaration = attrs.evolve(model.declaration, cases=(case,))
    with pytest.raises(pse.InspectionError) as failure:
        model.freeze().prepare(identity(101), pse.SolveSettings(intent="root"))
    report = failure.value.report
    assert report.boundary_class == "invalid_model"
    assert report.source_locations
    location = report.source_locations[0]
    assert isinstance(location, pse.DiagnosticSourceLocation)
    assert location.source == identity(243).to_hex()
    assert location.start is not None
    assert location.end is not None
    assert 0 <= location.start <= location.end <= len("x +")

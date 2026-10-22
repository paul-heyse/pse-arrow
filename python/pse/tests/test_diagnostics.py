# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Diagnostic boundaries use copied facts and never require a scientific store."""

import asyncio
import gc
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec, diagnostics
from pse.contracts import documents
from pse.contracts.values import SemanticId
from pse.contracts.documents import (
    CaptureBudget,
    CaptureDocumentOutput,
    CaptureRequest,
    CheckpointIdentity,
    ClosedDuration,
    ComparisonPolicy,
    DetailRequest,
    DiagnosticListRequest,
    DiagnosticProcessPage,
    DiagnosticProcessPageOutput,
    InspectionDocument,
    InspectionDocumentOutput,
    InspectionRequest,
    InvocationComparisonDocumentOutput,
    InvocationEvidenceDocumentOutput,
    LaunchCaptureRequest,
    MetricSnapshotOutput,
    NativeCountersOutputKinsol,
    ProcessCheckpoint,
    ProcessLocator,
    ProcessResources,
    SolveControls,
    SummaryCoverage,
    ValueBoolean,
    ValueCount,
    ValueInteger,
    ValueNumber,
    ValueText,
    ValueUnavailable,
    WorkObservationOutput,
)
from pse.contracts.enums import (
    CaptureKind,
    CaptureStatus,
    NativeBackend,
    NativeSolveIntent,
    OperationOutcome,
    PresolvePolicyKind,
    StatementKind,
)
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import declaration, package_documents, revision


@pytest.mark.unit
def test_rust_owned_inspection_fixture_preserves_exact_observations_and_outcome() -> (
    None
):
    fixture = (
        Path(__file__).parent / "fixtures/generated-native-boundaries/products.json"
    )
    products = codec.decode_json(fixture.read_bytes(), dict[str, msgspec.Raw])
    document = codec.decode_json(
        bytes(products["diagnostic_inspection"]), InspectionDocument
    )
    assert (
        codec.decode_json(codec.encode_json(document), InspectionDocument) == document
    )
    snapshot = document.operation
    assert snapshot is not None
    assert snapshot.id == document.request.operation
    assert (
        diagnostics.operation_text(
            codec.convert_document(
                codec.document_builtins(snapshot.id), documents.OperationId
            )
        )
        == "00000000000000010000000000000002:3"
    )
    assert snapshot.outcome == OperationOutcome.CANCELLED
    assert snapshot.finished_at is not None
    metrics = {metric.name: metric for metric in snapshot.metrics}
    calls = metrics["callback.calls"]
    exact_count = {"kind": "Count", "value": (1 << 53) + 1}
    assert (
        codec.decode_json(codec.encode_json(calls.latest), dict[str, int | str])
        == exact_count
    )
    assert (
        codec.decode_json(codec.encode_json(calls.cumulative), dict[str, int | str])
        == exact_count
    )
    assert codec.decode_json(
        codec.encode_json(metrics["ipopt.native_residual"].latest), dict[str, str]
    ) == {"kind": "NonFinite", "value": "PositiveInfinity"}
    assert codec.decode_json(
        codec.encode_json(metrics["sensitivity.withheld_cause"].latest), dict[str, str]
    ) == {"kind": "Unavailable", "value": "original assessment unavailable"}
    outcome = next(
        statement
        for statement in document.statements
        if statement.code == "operation.outcome"
    )
    assert outcome.kind == StatementKind.ESTABLISHED
    assert outcome.evidence[0].operation == snapshot.id
    assert "Scientific permission" in outcome.message
    assert not hasattr(document, "usable")
    assert not hasattr(snapshot, "scientific")


def observed_runtime(
    settings: pse.EngineSettings,
) -> tuple[pse.Runtime, pse.Diagnostics, pse.OperationObservations]:
    runtime = pse.Runtime(settings, ephemeral=True)
    observations = runtime.observations()
    runtime.physical_from_documents(physical_documents())
    page = observations.list(limit=32)
    assert page.operations, "physical preparation publishes its actual operation"
    reader = observations.reader(
        codec.convert_document(
            codec.document_builtins(page.operations[-1].id),
            documents.OperationId,
        )
    )
    assert reader is not None
    return runtime, observations, reader


def diagnostic_native_solve(
    runtime: pse.Runtime, *, history: int = 256
) -> pse.RunHandle:
    physical = runtime.physical_from_documents(physical_documents())
    package = runtime.modeling_from_documents(
        [
            package_documents(
                "package diagnostic { def Root { var x:Scalar; eq balance:x==3; "
                "annotation start x(0); } }"
            )
        ],
        physical,
    )
    selected = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package.prepare_solve(
        selected,
        pse.SolveSettings(
            intent=NativeSolveIntent.ROOT,
            backend=NativeBackend.KINSOL,
            presolve=PresolvePolicyKind.OFF,
            controls=SolveControls(history=history),
        ),
    ).start()


def diagnostic_scientific_probe(
    settings: pse.EngineSettings,
) -> tuple[
    bool, float, WorkObservationOutput, NativeCountersOutputKinsol, bool, bool, int
]:
    """Fresh-process probe compares actual native work independently of the hub."""
    runtime = pse.Runtime(settings, ephemeral=True)
    try:
        handle = diagnostic_native_solve(runtime)
        result = handle.wait()
        summary = result.attempt_summary
        step = summary.steps[0]
        assert step.work is not None
        assert step.work.evaluations is not None
        assert step.work.evaluations > 0
        assert step.native_observations is not None
        counters = step.native_observations.counters
        assert isinstance(counters, NativeCountersOutputKinsol)
        assert counters.function_evaluations is not None
        assert counters.function_evaluations > 0
        observations = runtime.observations()
        page = observations.list(limit=32)
        before: list[
            tuple[
                pse.OperationObservations,
                tuple[MetricSnapshotOutput, ...],
                ClosedDuration | None,
            ]
        ] = []
        for operation in page.operations:
            reader = observations.reader(
                codec.convert_document(
                    codec.document_builtins(operation.id),
                    documents.OperationId,
                )
            )
            assert reader is not None
            snapshot = reader.snapshot()
            before.append((reader, snapshot.metrics, snapshot.observed_at))
        for _ in range(8):
            for reader, metrics, observed_at in before:
                copied = reader.inspect().operation
                assert copied is not None
                assert copied.metrics == metrics
                assert copied.observed_at == observed_at
        assert result.attempt_summary == summary
        values = (
            pa.table(result.table("runtime.solve_variables"))
            .column("value")
            .to_pylist()
        )
        assert values == pytest.approx([3.0])
        if not page.coverage.enabled:
            absent = handle.observations()
            if absent is not None:
                snapshot = absent.snapshot()
                assert not snapshot.coverage.enabled
                assert not snapshot.coverage.available
                assert not snapshot.metrics
        return (
            result.usable,
            values[0],
            step.work,
            counters,
            page.coverage.enabled,
            page.coverage.available,
            len(page.operations),
        )
    finally:
        runtime.close()


@pytest.mark.unit
def test_diagnostics_opt_out_preserves_native_work_in_fresh_processes(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    program = """
import sys
import pse
from pse import codec
from pse.tests.test_diagnostics import diagnostic_scientific_probe
settings = pse.EngineSettings(
    memory_limit_bytes=int(sys.argv[1]), threads=2, target_partitions=1,
    spill_dir=sys.argv[2], max_spill_bytes=1 << 30, batch_size=7,
    cache=pse.CacheSettings(
        working_bytes=16 << 30, concurrent_loads=2,
        inflight_bytes=64 << 20, inspection_bytes=4 << 20,
    ),
)
print(codec.encode_json(diagnostic_scientific_probe(settings)).decode())
"""
    probes = []
    for mode in ("on", "off"):
        spill = tmp_path / f"{mode}-spill"
        spill.mkdir()
        environment = dict(
            os.environ, PSE_DIAGNOSTICS=mode, PSE_DIAGNOSTICS_DIR=str(tmp_path / mode)
        )
        process = subprocess.run(
            [
                sys.executable,
                "-c",
                program,
                str(inspection_settings.memory_limit_bytes),
                str(spill),
            ],
            env=environment,
            capture_output=True,
            text=True,
            check=True,
            timeout=120,
        )
        probes.append(
            codec.decode_json(
                process.stdout,
                tuple[
                    bool,
                    float,
                    WorkObservationOutput,
                    NativeCountersOutputKinsol,
                    bool,
                    bool,
                    int,
                ],
            )
        )
    enabled, disabled = probes
    assert enabled[:4] == disabled[:4], (
        "observation opt-out cannot add scientific requests or change the root"
    )
    assert enabled[0]
    assert enabled[4:6] == (True, True)
    assert enabled[6] > 0
    assert disabled[4:] == (False, False, 0), (
        "disabled observations are absent, not zero scientific work"
    )


@pytest.mark.unit
def test_nonempty_native_checkpoint_related_python_and_cli_reads_are_passive(
    inspection_settings: pse.EngineSettings,
    capsys: pytest.CaptureFixture[str],
    tmp_path: Path,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        handle = diagnostic_native_solve(runtime)
        result = handle.wait()
        assert result.usable
        summary = result.attempt_summary
        observations = runtime.observations()
        reader = handle.observations()
        assert reader is not None
        original = reader.snapshot()
        assert original.children, (
            "actual native dispatch must publish its related owner"
        )
        request = InspectionRequest(
            version=2,
            operation=msgspec.convert(
                reader.id, type=documents.OperationIdBody, from_attributes=True
            ),
            window=ClosedDuration(secs=60, nanos=0),
            include_related=True,
            related_limit=32,
        )
        deadline = time.monotonic() + 5
        while True:
            try:
                document = diagnostics.inspect_checkpoint(
                    observations.checkpoint, request
                )
                if (
                    document.operation is not None
                    and document.related
                    and document.operation.outcome == original.outcome
                ):
                    break
            except pse.InspectionError:
                pass
            assert time.monotonic() < deadline, (
                "publisher must expose nonempty actual related work"
            )
            time.sleep(0.02)
        expected_related = {child.id for child in original.children}
        assert {child.id for child in document.related} == expected_related
        assert document.operation is not None
        assert document.operation.metrics
        retained = {
            operation.id: operation.metrics
            for operation in (document.operation, *document.related)
        }
        for _ in range(8):
            assert (
                diagnostics.main(
                    [
                        "inspect",
                        str(observations.checkpoint),
                        diagnostics.operation_text(reader.id),
                        "--json",
                        "--related-limit",
                        "32",
                    ]
                )
                == 0
            )
            copied = codec.decode_json(
                capsys.readouterr().out, InspectionDocumentOutput
            )
            assert copied.operation is not None
            assert diagnostics.operation_text(
                copied.operation.id
            ) == diagnostics.operation_text(reader.id)
            assert copied.operation.outcome == original.outcome
            assert {child.id for child in copied.related} == expected_related
            for operation in (copied.operation, *copied.related):
                assert operation.metrics == retained[operation.id]
            assert reader.snapshot().metrics == original.metrics
            assert result.attempt_summary == summary
        # Discover an owned copy of the actual nonempty publication, independent of
        # unrelated process files in the shared publisher directory.
        checkpoint_copy = tmp_path / observations.checkpoint.name
        checkpoint_copy.write_bytes(observations.checkpoint.read_bytes())
        assert (
            diagnostics.main(
                ["list", "--directory", str(tmp_path), "--json", "--limit", "32"]
            )
            == 0
        )
        discovered = codec.decode_json(
            capsys.readouterr().out, DiagnosticProcessPageOutput
        )
        selected = next(
            process
            for process in discovered.processes
            if process.checkpoint == str(checkpoint_copy)
        )
        assert any(
            diagnostics.operation_text(operation.id)
            == diagnostics.operation_text(reader.id)
            for operation in selected.operations
        )
    finally:
        runtime.close()


@pytest.mark.unit
def test_diagnostic_operation_locator_roundtrip_and_exact_admission() -> None:
    text = "0123456789abcdeffedcba9876543210:18446744073709551615"
    operation = diagnostics.operation_id(text)
    assert diagnostics.operation_text(operation) == text
    for invalid in (
        "123:1",
        "0" * 32 + ":-1",
        "g" * 32 + ":1",
        "0" * 32 + ":18446744073709551616",
    ):
        with pytest.raises(pse.InspectionError):
            diagnostics.operation_id(invalid)


@pytest.mark.unit
def test_exact_prefix_read_and_inspection_use_same_image_after_process_exit(
    tmp_path: Path,
) -> None:
    products = codec.decode_json(
        (
            Path(__file__).parent / "fixtures/generated-native-boundaries/products.json"
        ).read_bytes(),
        dict[str, msgspec.Raw],
    )
    document = codec.decode_json(
        bytes(products["diagnostic_inspection"]), InspectionDocument
    )
    assert document.operation is not None
    locator = ProcessLocator(pid=(1 << 32) - 1, start_ticks=77, executable=None)
    locator_body = msgspec.convert(
        locator, type=documents.ProcessLocatorBody, from_attributes=True
    )
    image = ProcessCheckpoint(
        version=4,
        process="0000000000000001-0000000000000002",
        sequence=1,
        published_unix_micros=1,
        operations=(document.operation,),
        omitted=0,
        summaries=SummaryCoverage(refused=0, expired=0, pressure_lost=0),
        registry_coverage=document.operation.coverage,
        resources=ProcessResources(
            resident_bytes=None,
            lifetime_peak_resident_bytes=None,
            user_cpu_ticks=None,
            system_cpu_ticks=None,
        ),
        parent=None,
        locator=locator_body,
        clock=None,
        threads=(),
        threads_partial=False,
    )
    checkpoint = tmp_path / "retained.json"
    checkpoint.write_bytes(codec.encode_json(image))
    expected = CheckpointIdentity(process=image.process, locator=locator_body)
    retained = diagnostics.read_checkpoint(checkpoint, expected=expected)
    assert codec.document_builtins(retained) == codec.document_builtins(image)
    checkpoint.write_bytes(b"replacement/truncated path")
    inspected = diagnostics.inspect_image(
        retained, document.request, checkpoint=checkpoint
    )
    assert inspected.operation is not None
    assert inspected.operation.id == document.operation.id
    assert inspected.operation.outcome == document.operation.outcome
    assert (
        inspected.transport.partial
    )  # Last published prefix has no live clock mapping.
    checkpoint.write_bytes(codec.encode_json(image))
    reused = CheckpointIdentity(
        process=image.process,
        locator=documents.ProcessLocatorBody(
            pid=locator.pid, start_ticks=78, executable=None
        ),
    )
    with pytest.raises(pse.InspectionError, match="identity mismatch"):
        diagnostics.read_checkpoint(checkpoint, expected=reused)
    with pytest.raises(pse.InspectionError, match="identity mismatch"):
        diagnostics.read_checkpoint(
            checkpoint,
            expected=CheckpointIdentity(
                process="wrong",
                locator=locator_body,
            ),
        )


@pytest.mark.unit
def test_standalone_diagnostic_discovery_preserves_unavailable_prefixes(
    tmp_path: Path,
) -> None:
    missing = tmp_path / "unpublished"
    assert not diagnostics.list_processes(directory=missing).processes
    assert not missing.exists()
    for name in ("a", "b", "c"):
        (tmp_path / f"{name}.json").write_bytes(b"{truncated")
    first = diagnostics.list_processes(
        DiagnosticListRequest(after=None, limit=2), directory=tmp_path
    )
    assert len(first.processes) == 2
    assert all(process.failure and process.partial for process in first.processes)
    assert first.next is not None
    second = diagnostics.list_processes(
        DiagnosticListRequest(after=first.next, limit=2), directory=tmp_path
    )
    assert len(second.processes) == 1
    assert {process.checkpoint for process in first.processes}.isdisjoint(
        process.checkpoint for process in second.processes
    )
    assert (
        codec.decode_json(codec.encode_json(first), DiagnosticProcessPageOutput)
        == first
    )


@pytest.mark.unit
def test_operation_reader_survives_runtime_release_without_scientific_permission(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime, observations, reader = observed_runtime(inspection_settings)
    before = reader.inspect()
    assert before.operation is not None
    assert diagnostics.operation_text(
        before.operation.id
    ) == diagnostics.operation_text(reader.id)
    assert before.request.window == ClosedDuration(secs=60, nanos=0)
    assert (
        codec.decode_json(codec.encode_json(before), InspectionDocumentOutput) == before
    )
    runtime.close()
    del runtime
    gc.collect()
    after = reader.inspect(window=0.01)
    assert after.operation is not None
    assert after.operation.id == before.operation.id
    assert after.operation.outcome == before.operation.outcome
    assert observations.reader(reader.id) is not None
    assert diagnostics.operation_text(
        reader.snapshot().id
    ) == diagnostics.operation_text(reader.id)
    assert reader.inspect(window=3601.0).request.window == ClosedDuration(
        secs=3601, nanos=0
    )
    for bad_window in (float("nan"), float("inf"), -1.0):
        with pytest.raises(ValueError, match="window"):
            reader.inspect(window=bad_window)
    with pytest.raises(ValueError):
        reader.inspect(window=float.fromhex("0x1.fffffffffffffp+1023"))


@pytest.mark.unit
def test_run_observation_reader_outlives_joined_handle(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        physical = runtime.physical_from_documents(physical_documents())
        prepared = revision(runtime, physical).prepare_solve(
            declaration(101), pse.SolveSettings(intent=NativeSolveIntent.ROOT)
        )
        handle = prepared.start()
        reader = handle.observations()
        assert reader is not None
        assert diagnostics.operation_text(
            reader.snapshot().id
        ) == diagnostics.operation_text(reader.id)
        result = handle.wait()
        assert result.usable
        origin = result.diagnostic_origin()
        assert origin.run_id == result.run_id
        assert origin.operation == result.invocation_evidence.operation
        assert origin.failure is None
        with pytest.raises(pse.InspectionError):
            result.diagnostic_origin(9999)
        joined = reader.snapshot()
        del handle, prepared, result
        gc.collect()
        runtime.close()
        assert reader.snapshot().outcome == joined.outcome
        assert reader.inspect().operation is not None
    finally:
        runtime.close()


@pytest.mark.unit
def test_checkpoint_inspection_and_capture_wait_remain_independent(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    runtime, observations, reader = observed_runtime(inspection_settings)
    try:
        request = InspectionRequest(
            version=2,
            operation=msgspec.convert(
                reader.id, type=documents.OperationIdBody, from_attributes=True
            ),
            window=ClosedDuration(secs=60, nanos=0),
            include_related=True,
            related_limit=16,
        )
        deadline = time.monotonic() + 5
        while True:
            try:
                inspected = diagnostics.inspect_checkpoint(
                    observations.checkpoint, request
                )
                if inspected.operation is not None:
                    break
            except pse.InspectionError:
                pass
            assert time.monotonic() < deadline, (
                "publisher must make the selected operation visible"
            )
            time.sleep(0.02)
        before = reader.snapshot()
        session = diagnostics.capture(
            CaptureRequest(
                version=1,
                checkpoint=str(observations.checkpoint),
                operation=msgspec.convert(
                    reader.id, type=documents.OperationIdBody, from_attributes=True
                ),
                kind=CaptureKind.TIMELINE,
                output=str(tmp_path / "capture"),
                duration=ClosedDuration(secs=10, nanos=0),
                budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=256 << 20),
            )
        )
        waiting = threading.Event()
        completed: list[CaptureDocumentOutput] = []
        failures: list[BaseException] = []

        def wait() -> None:
            waiting.set()
            try:
                completed.append(session.wait())
            except BaseException as error:
                failures.append(error)

        thread = threading.Thread(target=wait)
        thread.start()
        try:
            assert waiting.wait(2)
            selected_operation = session.snapshot().operation
            assert selected_operation is not None
            assert diagnostics.operation_text(
                selected_operation
            ) == diagnostics.operation_text(reader.id)
            session.request_stop()
        finally:
            stopped = session.stop()
            thread.join(timeout=5)
        assert not thread.is_alive(), (
            "waiting releases Python execution and joins only capture"
        )
        assert not failures
        assert completed
        assert completed[0] == stopped == session.wait()
        assert not stopped.errors
        assert not stopped.loss.unknown_clock
        assert not stopped.loss.output_exhausted
        assert stopped.loss.refused_items == stopped.loss.queued_packets_dropped == 0
        expected_status = (
            CaptureStatus.PARTIAL
            if stopped.loss.source_partial
            else CaptureStatus.STOPPED
        )
        assert stopped.status == expected_status, (
            "retained source coalescing/coverage survives capture stop"
        )
        assert (
            codec.decode_json(codec.encode_json(stopped), CaptureDocumentOutput)
            == stopped
        )
        capture_view = diagnostics.capture_arrow(stopped)
        assert capture_view.num_rows == 1
        assert capture_view.column("operation")[
            0
        ].as_py() == diagnostics.operation_text(reader.id)
        assert (
            codec.decode_json(
                capture_view.column("document_json")[0].as_py(), CaptureDocumentOutput
            )
            == stopped
        )
        # The view retains the raw capture outcome/coverage, not scientific permission.
        assert capture_view.column("status")[0].as_py() == stopped.status.value
        assert reader.snapshot().outcome == before.outcome
        runtime.physical_from_documents(physical_documents())
    finally:
        runtime.close()


@pytest.mark.unit
def test_diagnostic_cli_projects_typed_documents_and_unavailable_errors(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    assert diagnostics.main(["list", "--directory", str(tmp_path), "--json"]) == 0
    encoded = capsys.readouterr().out
    assert not codec.decode_json(encoded, DiagnosticProcessPage).processes
    assert diagnostics.main(["inspect", str(tmp_path / "missing.json"), "invalid"]) == 1
    output = capsys.readouterr()
    assert not output.out
    assert "diagnostics unavailable" in output.err
    assert not (tmp_path / "missing.json").exists()
    assert (
        diagnostics.main(
            [
                "inspect",
                str(tmp_path / "missing.json"),
                "0" * 32 + ":1",
                "--window",
                "1e308",
            ]
        )
        == 1
    )
    assert "diagnostics unavailable" in capsys.readouterr().err


@pytest.mark.unit
@pytest.mark.parametrize(
    "arguments",
    [
        ["list", "--limit", "33"],
        ["inspect", "missing.json", "0" * 32 + ":1", "--window", "nan"],
        [
            "capture",
            "missing.json",
            "0" * 32 + ":1",
            "--output",
            "unused",
            "--duration",
            "inf",
        ],
    ],
)
def test_diagnostic_cli_refuses_unbounded_requests(arguments: list[str]) -> None:
    with pytest.raises(SystemExit) as error:
        diagnostics.main(arguments)
    assert error.value.code == 2


@pytest.mark.unit
@pytest.mark.parametrize("output_bytes", [16 << 20, 1])
def test_selected_native_detail_current_history_zero_and_invocation_endpoints(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    output_bytes: int,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        request = DetailRequest(
            families=("root_evaluation.v1", "native_owner_return.v1"),
            requirement="complete",
            sample_every=1,
            boundary_thread_cpu=True,
            duration=ClosedDuration(secs=30, nanos=0),
            output=str(tmp_path / "selected"),
            budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=output_bytes),
        )
        with runtime.observations().collect(request) as selected:
            handle = diagnostic_native_solve(runtime, history=0)
            selected.leave()

            async def join() -> pse.RunResult:
                return await handle.wait_async()

            result = asyncio.run(join())
            assert result.usable
            assert not handle.progress()[0]
            checkpoint = handle.current_progress()
            assert checkpoint.current is not None
            assert checkpoint.current_is_latest
            assert checkpoint.current.source is not None
            assert (
                checkpoint.current.source.sequence == checkpoint.highest_issued_sequence
            )
            assert checkpoint.current.sequence is None
            invocation = result.invocation_evidence
            assert invocation.version == 2
            assert "before dispatch" in invocation.clock_scope
            invocation_before = codec.encode_json(invocation)
            summary_before = codec.encode_json(result.attempt_summary)
            explanation = result.explain_invocation()
            comparison = result.compare_invocation(result)
            assert explanation.baseline.run_id == invocation.run_id
            assert explanation.baseline.operation == invocation.operation
            assert explanation.candidate is None
            assert comparison.candidate == comparison.baseline
            assert comparison.baseline.run_id == invocation.run_id
            assert (
                codec.decode_json(
                    codec.encode_json(comparison), InvocationComparisonDocumentOutput
                )
                == comparison
            )
            assert all(
                evidence.invocation.run_id == invocation.run_id
                and evidence.invocation.operation == invocation.operation
                and evidence.field.startswith("/")
                and evidence.clock_scope
                for statement in explanation.statements + comparison.statements
                for evidence in statement.evidence
            )
            unavailable = next(
                statement
                for statement in comparison.statements
                if statement.code == "comparison.unit_cost_unavailable"
            )
            assert unavailable.kind == StatementKind.LIMITATION
            assert unavailable.next
            assert all(
                isinstance(evidence.value, ValueUnavailable)
                for evidence in unavailable.evidence
            ), "KINSOL counters are not an Ipopt native wall/iteration population"
            assert any(
                statement.code == "comparison.scientific_input_context_unknown"
                and statement.kind == StatementKind.LIMITATION
                for statement in comparison.statements
            )
            bounded = result.compare_invocation(
                result, policy=ComparisonPolicy(statements=1, source_items=0)
            )
            assert len(bounded.statements) <= 1
            assert bounded.partial
            assert bounded.omitted_source_items > 0
            invocation_view = result.invocation_arrow()
            assert invocation_view.num_rows == 1
            assert invocation_view.column("attempt")[0].as_py() is None
            assert (
                invocation_view.column("clock_scope")[0].as_py()
                == invocation.clock_scope
            )
            assert (
                codec.decode_json(
                    invocation_view.column("document_json")[0].as_py(),
                    InvocationEvidenceDocumentOutput,
                )
                == invocation
            )
            for field in (
                "qualification_seconds",
                "terminal_seconds",
                "response_ready_seconds",
            ):
                assert invocation_view.column(field)[0].as_py() == getattr(
                    invocation.endpoints, field
                )
            escaped_run_column = invocation_view.column("run")
            del invocation_view
            gc.collect()
            assert invocation.run_id is not None
            assert escaped_run_column[0].as_py() == invocation.run_id.to_hex()
            assert codec.encode_json(result.invocation_evidence) == invocation_before
            assert codec.encode_json(result.attempt_summary) == summary_before
            assert invocation.timing_unavailable is None
            endpoints = invocation.endpoints
            assert endpoints.qualification_seconds is not None
            assert endpoints.terminal_seconds is not None
            assert endpoints.response_ready_seconds is not None
            assert (
                endpoints.qualification_seconds
                <= endpoints.terminal_seconds
                <= endpoints.response_ready_seconds
            )
        document = selected.snapshot()
        if output_bytes == 1:
            assert document.outcome == "incomplete"
            assert document.error is not None
            return
        assert document.outcome == "complete"
        assert not document.source_partial
        assert all(
            family.acknowledged == family.observed > 0 for family in document.families
        )
        assert (
            diagnostics.read_detail(document.records).readable_prefix_bytes
            == document.readable_prefix_bytes
        )
        page = diagnostics.detail_records(document.records)
        assert {record.family for record in page.records} == set(request.families)
        assert all(record.family_sequence is not None for record in page.records)
        assert all(record.observed_at is not None for record in page.records)
        selected_page = diagnostics.detail_records(document.records, limit=4)
        page_before = codec.encode_json(selected_page)
        view = diagnostics.detail_arrow(Path(document.records), limit=4)
        assert view.num_rows == sum(
            max(1, len(record.values)) for record in selected_page.records
        )
        assert view.schema.field("event_sequence").type == pa.uint64()
        assert view.schema.field("observed_nanoseconds").type == pa.uint32()
        row = 0
        for record in selected_page.records:
            for ordinal, field in enumerate(record.values, 1):
                assert view.column("operation")[
                    row
                ].as_py() == diagnostics.operation_text(
                    codec.convert_document(
                        codec.document_builtins(record.producer.operation),
                        documents.OperationId,
                    )
                )
                assert view.column("producer")[row].as_py() == record.producer.sequence
                assert view.column("event_sequence")[row].as_py() == record.sequence
                assert (
                    view.column("family_sequence")[row].as_py()
                    == record.family_sequence
                )
                assert view.column("generation")[row].as_py() == record.generation
                assert view.column("family")[row].as_py() == record.family
                assert (
                    view.column("observed_seconds")[row].as_py()
                    == record.observed_at.secs
                )
                assert (
                    view.column("observed_nanoseconds")[row].as_py()
                    == record.observed_at.nanos
                )
                assert view.column("field_ordinal")[row].as_py() == ordinal
                assert view.column("field_name")[row].as_py() == field.name
                assert view.column("unit")[row].as_py() == field.unit
                populated = None
                expected_value: bool | int | float | str | None = None
                if isinstance(field.value, ValueCount):
                    populated = "count_value"
                    expected_value = field.value.value
                elif isinstance(field.value, ValueInteger):
                    populated = "integer_value"
                    expected_value = field.value.value
                elif isinstance(field.value, ValueNumber):
                    populated = "number_value"
                    expected_value = field.value.value
                elif isinstance(field.value, ValueBoolean):
                    populated = "boolean_value"
                    expected_value = field.value.value
                elif isinstance(field.value, (ValueText, ValueUnavailable)):
                    populated = "text_value"
                    expected_value = field.value.value
                if populated is not None:
                    assert view.column(populated)[row].as_py() == expected_value
                    assert all(
                        view.column(other)[row].as_py() is None
                        for other in (
                            "count_value",
                            "integer_value",
                            "number_value",
                            "boolean_value",
                            "text_value",
                        )
                        if other != populated
                    ), "unavailable and typed values cannot become numeric zero"
                row += 1
            if not record.values:
                assert view.column("field_ordinal")[row].as_py() == 0
                assert view.column("field_name")[row].as_py() is None
                row += 1
        assert row == view.num_rows
        with pytest.raises(pse.InspectionError):
            diagnostics.detail_arrow(Path(document.records), limit=4, memory_bytes=1)
        assert (
            codec.encode_json(diagnostics.detail_records(document.records, limit=4))
            == page_before
        ), "projection/refusal does not rewrite retained producer evidence"
        assert codec.encode_json(result.invocation_evidence) == invocation_before
        assert codec.encode_json(result.attempt_summary) == summary_before
    finally:
        runtime.close()


@pytest.mark.unit
def test_launch_capture_public_command_survives_sampler_stop(tmp_path: Path) -> None:
    """Actual public owners remain independent even when perf is unavailable."""
    request = LaunchCaptureRequest(
        version=1,
        argv=("/bin/sh", "-c", "sleep 60 & wait"),
        cwd=None,
        output=str(tmp_path / "capture"),
        duration=ClosedDuration(secs=1, nanos=0),
        startup_timeout=ClosedDuration(secs=1, nanos=0),
        budget=CaptureBudget(memory_bytes=4 << 20, output_bytes=1 << 20),
    )
    command, session = diagnostics.launch_capture(request)
    try:
        assert command.locator.pid > 0
        document = session.stop()
        assert command.try_wait() is None
        assert document.operation is None
        assert document.launch_process is not None
        assert document.launch_process.pid == command.locator.pid
        view = diagnostics.capture_arrow(document)
        assert view.column("operation")[0].as_py() is None
        assert view.column("launch_process_json")[0].as_py() is not None
    finally:
        session.stop()
        command.close()
    assert command.try_wait() == 137


@pytest.mark.unit
def test_launch_capture_cli_retains_command_and_returns_scientific_exit(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A sampler failure cannot hide or replace a nonzero scientific outcome."""
    assert (
        diagnostics.main(
            [
                "capture-launch",
                "--output",
                str(tmp_path / "capture"),
                "--duration",
                "0.05",
                "--startup-timeout",
                "1",
                "--",
                "/bin/sh",
                "-c",
                'printf finished > "$1"; exit 7',
                "science",
                str(tmp_path / "science-finished"),
            ]
        )
        == 7
    )
    assert (tmp_path / "science-finished").read_text() == "finished"
    document = codec.decode_json(capsys.readouterr().out, CaptureDocumentOutput)
    assert document.operation is None
    assert document.launch_process is not None


@pytest.mark.unit
def test_launch_capture_public_literal_arguments_and_working_directory(
    tmp_path: Path,
) -> None:

    command, session = diagnostics.launch_capture(
        LaunchCaptureRequest(
            version=1,
            argv=(
                "/bin/sh",
                "-c",
                "printf '%s' \"$1\" > literal",
                "science",
                "$(touch WRONG); a b",
            ),
            cwd=str(tmp_path),
            output=str(tmp_path / "capture"),
            duration=ClosedDuration(secs=0, nanos=50_000_000),
            startup_timeout=ClosedDuration(secs=1, nanos=0),
            budget=CaptureBudget(memory_bytes=4 << 20, output_bytes=1 << 20),
        )
    )
    try:
        assert command.wait() == 0
        session.stop()
    finally:
        session.stop()
        command.close()
    assert (tmp_path / "literal").read_text() == "$(touch WRONG); a b"
    assert not (tmp_path / "WRONG").exists()


@pytest.mark.unit
def test_launch_capture_cli_requires_explicit_executable(tmp_path: Path) -> None:
    with pytest.raises(SystemExit) as error:
        diagnostics.main(["capture-launch", "--output", str(tmp_path / "unused")])
    assert error.value.code == 2
    assert not (tmp_path / "unused").exists()


def _reproducer_text_request(settings: pse.EngineSettings) -> documents.ReproducerExtractionRequest:
    policy = codec.decode_json(
        (Path(__file__).resolve().parents[3] / "packages/reference/diagnostics/idaes-2.13.json").read_bytes(),
        documents.ModelingDiagnosticPolicy,
    )
    operation = documents.DiagnosticExecutionRequest(
        physical_documents={"package.toml": "[package]\nnot a valid manifest"},
        model_documents=({"models/original.pse": "package original {def Root {var x:Scalar;eq e:x==1;}}"},),
        case_id=SemanticId.from_hex("ab" * 16),
        settings=pse.SolveSettings(),
        diagnostics=msgspec.convert(policy, type=documents.ModelingDiagnosticPolicyBody, from_attributes=True),
        origin=None,
        allowance=ClosedDuration(secs=10, nanos=0),
        maximum_source_bytes=4 << 20,
    )
    return codec.decode_json(
        codec.encode_json({
            "version": 1,
            "runtime": {
                "memory_limit_bytes": settings.memory_limit_bytes,
                "threads": settings.threads,
                "spill_dir": settings.spill_dir,
                "max_spill_bytes": settings.max_spill_bytes,
                "batch_size": settings.batch_size,
            },
            "input": {"family": "original_text", "request": operation},
            "original_diagnostic": None,
            "predicate": None,
        }),
        documents.ReproducerExtractionRequest,
    )


@pytest.mark.unit
def test_reproducer_cli_extraction_is_offline_and_keeps_actual_source_text(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    request = _reproducer_text_request(inspection_settings)
    source = tmp_path / "recorded.json"
    destination = tmp_path / "reproducer.json"
    source.write_bytes(codec.encode_json(request))

    def no_runtime(_settings: documents.DiagnosticRuntimeSettings) -> pse.Runtime:
        pytest.fail("offline extraction must not construct or execute a runtime")

    monkeypatch.setattr(diagnostics, "_diagnostic_runtime", no_runtime)
    assert diagnostics.main(["extract-reproducer", "--request", str(source), "--output", str(destination)]) == 0
    extracted = codec.decode_json(destination.read_bytes(), documents.ReproducerCommandRequest)
    assert isinstance(extracted.reproducer.input, documents.ReproducerInputBodyOriginalText)
    assert extracted.reproducer.input.request.physical_documents["package.toml"] == "[package]\nnot a valid manifest"
    assert extracted.reproducer.predicate is None
    assert "Explicit new run:" in capsys.readouterr().out


@pytest.mark.unit
def test_reproducer_explicit_runner_retains_fresh_operation_and_strict_original_comparison(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
) -> None:
    request = _reproducer_text_request(inspection_settings)
    extracted = diagnostics.extract_reproducer(request)
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        first = runtime.run_reproducer(extracted.reproducer)
        assert first.failure is not None
        assert first.run_id is None
        assert first.effective_allowance.secs <= 10
        assert first.status == "reproduction_unavailable"
        assert first.original.original_failure_text is None
        # Only the selected typed actual failure supplies the strict original predicate.
        typed = msgspec.structs.replace(request, original_diagnostic=codec.decode_json(codec.encode_json(first.failure), documents.BoundaryDiagnosticBody), original_operation=first.operation)
        selected = diagnostics.extract_reproducer(typed)
        second = runtime.run_reproducer(selected.reproducer)
        assert second.operation != first.operation
        assert second.status == "original_failure_reproduced"
        assert second.run_id is None
        assert second.original.original_operation == first.operation
        driver = tmp_path / "explicit.json"
        driver.write_bytes(codec.encode_json(selected))
    finally:
        runtime.close()
    # The driver is an explicit new process; its deployment defaults must not be
    # mixed with this fixture's already configured process-wide resource owner.
    completed = subprocess.run(
        [sys.executable, "-m", "pse.diagnostics", "reproduce", "--request", str(driver), "--json"],
        check=False, capture_output=True, timeout=30,
    )
    assert completed.returncode == 0, completed.stderr.decode()
    result = codec.decode_json(completed.stdout, documents.ReproductionDocumentOutput)
    assert result.status == "original_failure_reproduced"

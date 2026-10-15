# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Diagnostic boundaries use copied facts and never require a scientific store."""

import gc
import threading
import time
from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec, diagnostics
from pse.contracts.documents import (
    CaptureBudget,
    CaptureDocument,
    CaptureRequest,
    ClosedDuration,
    DiagnosticListRequest,
    DiagnosticProcessPage,
    InspectionDocument,
    InspectionRequest,
)
from pse.contracts.enums import (
    CaptureKind,
    CaptureStatus,
    NativeSolveIntent,
    OperationOutcome,
    StatementKind,
)
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import declaration, revision


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
        diagnostics.operation_text(snapshot.id) == "00000000000000010000000000000002:3"
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
    reader = observations.reader(page.operations[-1].id)
    assert reader is not None
    return runtime, observations, reader


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
    assert codec.decode_json(codec.encode_json(first), DiagnosticProcessPage) == first


@pytest.mark.unit
def test_operation_reader_survives_runtime_release_without_scientific_permission(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime, observations, reader = observed_runtime(inspection_settings)
    before = reader.inspect()
    assert before.operation is not None
    assert before.operation.id == reader.id
    assert before.request.window == ClosedDuration(secs=60, nanos=0)
    assert codec.decode_json(codec.encode_json(before), InspectionDocument) == before
    runtime.close()
    del runtime
    gc.collect()
    after = reader.inspect(window=0.01)
    assert after.operation is not None
    assert after.operation.id == before.operation.id
    assert after.operation.outcome == before.operation.outcome
    assert observations.reader(reader.id) is not None
    assert reader.snapshot().id == reader.id
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
        assert reader.snapshot().id == reader.id
        result = handle.wait()
        assert result.usable
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
            version=1,
            operation=reader.id,
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
                operation=reader.id,
                kind=CaptureKind.TIMELINE,
                output=str(tmp_path / "capture"),
                duration=ClosedDuration(secs=10, nanos=0),
                budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=256 << 20),
            )
        )
        waiting = threading.Event()
        completed: list[CaptureDocument] = []
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
            assert session.snapshot().operation == reader.id
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
        assert codec.decode_json(codec.encode_json(stopped), CaptureDocument) == stopped
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

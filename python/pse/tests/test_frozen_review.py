# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Recorded review uses copied facts after original runtimes and paths disappear."""

import shutil
from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec, diagnostics
from pse.contracts import documents
from pse.contracts.enums import TaskMetricsScope
from pse.tests.test_diagnostics import diagnostic_native_solve


@pytest.mark.unit
def test_frozen_review_public_ingress_is_recorded_and_relocatable(
    tmp_path: Path,
) -> None:
    products = codec.decode_json(
        (
            Path(__file__).parent / "fixtures/generated-native-boundaries/products.json"
        ).read_bytes(),
        dict[str, msgspec.Raw],
    )
    original = codec.decode_json(
        bytes(products["diagnostic_inspection"]), documents.InspectionDocument
    )
    assert original.operation is not None
    image = documents.ProcessCheckpoint(
        version=5,
        process="0000000000000001-0000000000000002",
        sequence=1,
        published_unix_micros=1,
        operations=(original.operation,),
        omitted=0,
        summaries=documents.SummaryCoverage(refused=0, expired=0, pressure_lost=0),
        registry_coverage=original.operation.coverage,
        resources=documents.ProcessResources(
            resident_bytes=None,
            lifetime_peak_resident_bytes=None,
            user_cpu_ticks=None,
            system_cpu_ticks=None,
        ),
        parent=None,
        locator=documents.ProcessLocatorBody(
            pid=(1 << 32) - 1, start_ticks=None, executable=None
        ),
        clock=None,
        threads=(),
        threads_partial=False,
        runtime_metrics=documents.RuntimeMetricsSnapshot(
            locator=documents.ProcessLocatorBody(
                pid=(1 << 32) - 1, start_ticks=None, executable=None
            ),
            sampled_unix_micros=1,
            coverage=documents.RuntimeMetricsCoverage(
                executor_registry_available=True,
                refused_executor_registrations=0,
                task_scope=TaskMetricsScope.SELECTED_PROCESS_FUTURES,
            ),
            executors=(),
            task_classes=(),
        ),
    )
    supplied = documents.FrozenEvidenceSet(version=3, checkpoints=(image,))
    request = codec.decode_json(
        b'{"version":1,"inspection":'
        + codec.encode_json(original.request)
        + b',"window":{"kind":"recorded"}}',
        documents.FrozenInspectionRequest,
    )
    expected = diagnostics.inspect_frozen(supplied, request)
    path = tmp_path / "recorded.json"
    raw = codec.encode_json(supplied)
    path.write_bytes(raw)
    (tmp_path / "conflicting-neighbour.json").write_bytes(b"not a checkpoint")
    restored = diagnostics.read_evidence(path)
    relocated = tmp_path / "relocated.json"
    path.rename(relocated)
    assert diagnostics.inspect_frozen(restored, request) == expected
    assert (
        diagnostics.inspect_frozen(diagnostics.read_evidence(relocated), request)
        == expected
    )
    assert expected.inspection.operation is not None
    assert expected.inspection.operation.read_at == original.operation.read_at
    assert relocated.read_bytes() == raw
    inventory_request = documents.ReviewInventoryRequest(
        attempts="current", maximum_members=32
    )
    inventory = diagnostics.inventory_from_evidence(restored, inventory_request)
    assert any(
        member.operation == original.operation.id for member in inventory.members
    )
    assert diagnostics.inventory_page(inventory).members == inventory.members
    assert (
        diagnostics.inventory_from_evidence(
            diagnostics.read_evidence(relocated), inventory_request
        )
        == inventory
    )
    path.write_bytes(b'{"version":99,"checkpoints":"unsupported body"}')
    with pytest.raises(pse.InspectionError, match="unsupported"):
        diagnostics.read_evidence(path)
    conflicting = msgspec.structs.replace(supplied, checkpoints=(image, image))
    with pytest.raises(pse.InspectionError, match="duplicate"):
        diagnostics.inspect_frozen(conflicting, request)


@pytest.mark.unit
def test_saved_invocation_and_context_need_no_live_result(
    inspection_settings: pse.EngineSettings, tmp_path: Path
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        handle = diagnostic_native_solve(runtime)
        result = handle.wait()
        assert result.usable
        source = result.invocation_evidence
        context = result.review_context()
        expected = result.explain_invocation()
        invocation_path = tmp_path / "invocation.json"
        invocation_path.write_bytes(codec.encode_json(source))
        context_path = tmp_path / "context.json"
        context_path.write_bytes(codec.encode_json(context))
    finally:
        runtime.close()
    del result, handle, runtime
    saved = diagnostics.read_invocation(invocation_path)
    assert saved == source
    assert diagnostics.explain_invocation(saved) == expected
    assert diagnostics.compare_invocations(saved, saved).candidate == expected.baseline
    assert diagnostics.read_context(context_path) == context
    assert context.run_id == source.run_id
    assert context.sources
    assert context.facts
    assert saved.attempts.native_attempts
    duplicated = msgspec.structs.replace(
        saved,
        attempts=msgspec.structs.replace(
            saved.attempts,
            native_attempts=(
                *saved.attempts.native_attempts,
                saved.attempts.native_attempts[0],
            ),
        ),
    )
    invocation_path.write_bytes(codec.encode_json(duplicated))
    with pytest.raises(pse.InspectionError, match="duplicate"):
        diagnostics.read_invocation(invocation_path)
    with pytest.raises(pse.InspectionError, match="duplicate"):
        diagnostics.explain_invocation(duplicated)


def source_recording_request(
    directory: Path, *, review: bool
) -> tuple[Path, documents.SourceRecordingRequest]:
    """Select source recording and independent optional descriptive output."""
    request = codec.decode_json(
        codec.encode_json(
            {
                "version": 2,
                "detail": {
                    "version": 1,
                    "families": ["root_evaluation.v1", "native_owner_return.v1"],
                    "requirement": "complete",
                    "sample_every": 1,
                    "boundary_thread_cpu": False,
                    "managed_budget": None,
                    "expiry_monotonic_ns": None,
                    "duration": {"secs": 30, "nanos": 0},
                    "output": str(directory / "recorded-source"),
                    "budget": {
                        "memory_bytes": 16 << 20,
                        "output_bytes": 8 << 20,
                    },
                },
                "acknowledgement": str(directory / "recording-ack.json"),
                "review": (
                    {
                        "context": {
                            "steps": [],
                            "include_sources": True,
                            "include_native_records": True,
                            "maximum_bytes": 16 << 20,
                        },
                        "maximum_bytes": 32 << 20,
                        "maximum_members": 4,
                    }
                    if review
                    else None
                ),
            }
        ),
        documents.SourceRecordingRequest,
    )
    path = directory / "recording-request.json"
    path.write_bytes(codec.encode_json(request))
    return path, request


@pytest.mark.unit
@pytest.mark.parametrize("review_selected", [False, True])
def test_optional_source_recording_retain_result_and_context_readback_relocates(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    review_selected: bool,
) -> None:

    request_path, request = source_recording_request(tmp_path, review=review_selected)
    assert request.version == 2
    assert request.detail.version == 1
    monkeypatch.setenv("PSE_DIAGNOSTIC_RECORD_REQUEST", str(request_path))
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    receipt_path: Path | None = None
    try:
        selected = runtime.observations().record_from_environment(
            ("root_evaluation.v1", "native_owner_return.v1")
        )
        assert selected is not None
        acknowledgement = selected.acknowledgement
        assert acknowledgement is not None
        assert acknowledgement.accepted
        assert acknowledgement.version == 2
        with selected:
            handle = diagnostic_native_solve(runtime)
            selected.leave()
            result = handle.wait()
            assert result.usable
            invocation_before = result.invocation_evidence
            selected.__enter__()
            joined = selected.retain_result(result)
            with pytest.raises(ValueError, match="already entered"):
                selected.__enter__()
            selected.leave()
            repeated = selected.retain_result(result)
            context = documents.ReviewContextDocument(
                version=2,
                run_id=None,
                attempt_id=None,
                operation=acknowledgement.operation,
                selection=documents.ReviewContextRequestBody(
                    steps=(),
                    include_sources=True,
                    include_native_records=True,
                    maximum_bytes=4 << 20,
                ),
                sources=(),
                source_positions=(),
                solver_settings=(),
                facts=(
                    documents.ReviewContextFact(
                        path="/submitted/request",
                        codec="submitted-request.v1",
                        payload=tuple(codec.encode_json("original unparsed request")),
                    ),
                ),
                coverage=None,
                failure="caller-retained failure before run establishment",
                unavailable=(),
            )
            described = selected.retain_context(context)
            assert result.usable
            assert result.invocation_evidence == invocation_before
            if review_selected:
                assert joined is not None
                assert repeated == joined
                assert len(joined.members) == 1
                assert described is not None
                assert len(described.members) == 2
                assert acknowledgement.review is not None
                assert acknowledgement.review.accepted
                receipt_path = Path(acknowledgement.review.receipt)
            else:
                assert joined is None
                assert repeated is None
                assert described is None
                assert acknowledgement.review is None
    finally:
        runtime.close()
    del result, handle, selected, runtime
    if not review_selected:
        assert not (Path(request.detail.output) / "review").exists()
        return
    assert receipt_path is not None
    receipt = codec.decode_json(
        receipt_path.read_bytes(), documents.ReviewHandoffReceipt
    )
    assert str(receipt.state) == "complete"
    assert not receipt.errors
    restored = diagnostics.read_handoff(receipt_path)
    assert len(restored.invocations) == 1
    assert len(restored.contexts) == 2
    assert restored.invocations[0] == invocation_before
    assert restored.contexts[0].run_id == invocation_before.run_id
    assert codec.document_builtins(restored.contexts[1]) == codec.document_builtins(
        context
    )
    assert restored.invocations[0].attempts.native_attempts
    output = Path(request.detail.output)
    relocated = tmp_path / "relocated-recording"
    shutil.move(output, relocated)
    # Original absolute paths and neighboring files cannot substitute membership.
    (output / "review").mkdir(parents=True)
    (output / "review" / "index.json").write_text("deceptive original receipt")
    monkeypatch.setattr(
        diagnostics, "read_checkpoint", lambda *_a, **_k: pytest.fail("live lookup")
    )
    moved = diagnostics.read_handoff(relocated / "review" / "index.json")
    assert codec.encode_json(moved) == codec.encode_json(restored)
    assert bytes(moved.contexts[1].facts[0].payload) == codec.encode_json(
        "original unparsed request"
    )

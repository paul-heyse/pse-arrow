# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Small original science anchors qualify composed recorded review propagation."""

import shutil
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec, diagnostics
from pse.contracts import documents as d
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind
from pse.tests.test_frozen_review import source_recording_request
from pse.tests.physical_input_fixture import stage_batch
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents
from pse.tests.test_native_workflow import (
    test_explicit_cone_strategy_preserves_native_qualification as assert_cone,
)


@pytest.mark.unit
@pytest.mark.parametrize("kind", ["root", "nlp"])
def test_original_analytic_run_survives_recording_analysis_and_relocated_pack(
    kind: str,
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    request_path, _ = source_recording_request(tmp_path, review=True)
    monkeypatch.setenv("PSE_DIAGNOSTIC_RECORD_REQUEST", str(request_path))
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        physical = runtime.physical_from_documents(physical_documents())
        body = (
            "var x:Scalar; eq balance:x==3; annotation start x(0);"
            if kind == "root"
            else "var x:Scalar; let cost:Scalar=(x-3)*(x-3); "
            "annotation start x(0); annotation objective cost(minimize);"
        )
        package = runtime.modeling_from_documents(
            [package_documents("package review { def Root {" + body + "} }")],
            physical,
        )
        case = next(
            row.declaration_id for row in package.declarations() if row.name == "Root"
        )
        settings = pse.SolveSettings(
            intent=NativeSolveIntent.ROOT
            if kind == "root"
            else NativeSolveIntent.OPTIMIZE,
            backend=NativeBackend.KINSOL if kind == "root" else NativeBackend.POUNCE,
            presolve=PresolvePolicyKind.OFF,
        )
        assert settings.numerics.engineering_relative_fraction == 0.001
        source_map = package.diagnostic_source_map(case, settings)
        recording = runtime.observations().record_from_environment(
            ("root_evaluation.v1", "native_owner_return.v1")
        )
        assert recording is not None
        with recording:
            handle = package.prepare_solve(case, settings).start()
            recording.leave()
            result = handle.wait()
            assert result.usable
            original_rows = pa.table(result.table("runtime.solve_variables"))
            assert original_rows.column("value").to_pylist() == pytest.approx(
                [3.0], rel=0.001, abs=0.0
            )
            original_ids = original_rows.column("symbol_id").to_pylist()
            assert len(original_ids) == 1
            assert source_map.columns
            assert original_ids[0].hex() in source_map.columns
            original_invocation = result.invocation_evidence
            original_summary = result.attempt_summary
            assert original_summary.steps[0].work is not None
            assert original_summary.steps[0].work.evaluations is not None
            assert original_summary.steps[0].work.evaluations > 0
            context = result.review_context()
            assert context.run_id == result.run_id.to_hex()
            assert context.operation == original_invocation.operation
            assert context.source_positions == (0,)
            native = [
                fact
                for fact in context.facts
                if fact.codec == "native-attempt-record.v1"
            ]
            assert native
            supplied_invocation = next(
                fact for fact in context.facts if fact.codec == "invocation-evidence.v2"
            )
            assert (
                codec.decode_json(
                    bytes(supplied_invocation.payload), d.InvocationEvidenceDocument
                )
                == original_invocation
            )
            receipt = recording.retain_result(result)
            assert receipt is not None
            acknowledgement = recording.acknowledgement
            assert acknowledgement is not None
            assert acknowledgement.review is not None
            handoff = Path(acknowledgement.review.receipt)
        evidence = diagnostics.read_handoff(handoff)
        assert evidence.invocations == (original_invocation,)
        assert evidence.contexts == (context,)
        # A static source map is explicitly supplied, never silently prepared by review.
        attached = msgspec.structs.replace(
            context,
            facts=(
                *context.facts,
                d.ReviewContextFact(
                    path="/steps/0/source_map",
                    codec="modeling-source-map.v1",
                    payload=tuple(codec.encode_json(source_map)),
                ),
            ),
        )
        evidence = msgspec.structs.replace(evidence, contexts=(attached,))
        operation = original_invocation.operation
        assert operation is not None
        request = codec.decode_json(
            codec.encode_json({"version": 2, "operation": operation}), d.AnalysisRequest
        )
        analysis = diagnostics.analyze_evidence(evidence, request)
        output = tmp_path / "pack"
        index = diagnostics.export_review_pack(
            output,
            evidence,
            analysis,
            d.PackExportRequest(
                selection=d.PackSelection(
                    members=(),
                    maximum_output_bytes=32 << 20,
                    maximum_document_bytes=16 << 20,
                )
            ),
        )
        assert index.publication == "sealed"
        assert result.attempt_summary == original_summary
        assert result.invocation_evidence == original_invocation
        assert pa.table(result.table("runtime.solve_variables")).equals(original_rows)
    finally:
        runtime.close()
    del result, handle, package, physical, recording, runtime
    relocated = tmp_path / "relocated"
    shutil.move(output, relocated)
    handoff.rename(tmp_path / "inaccessible-original-handoff.json")
    (relocated / "context.json").write_bytes(b"conflicting live sibling")
    monkeypatch.delenv("PSE_DIAGNOSTIC_RECORD_REQUEST")

    def forbidden_runtime(*_args: object, **_kwargs: object) -> None:
        message = "offline review attempted scientific runtime access"
        raise AssertionError(message)

    monkeypatch.setattr(pse, "Runtime", forbidden_runtime)
    assert diagnostics.review_pack(relocated) == index
    request_file = tmp_path / "analysis-request.json"
    request_file.write_bytes(codec.encode_json(request))
    assert (
        diagnostics.main(
            ["analyze", str(relocated), "--request", str(request_file), "--json"]
        )
        == 0
    )
    restored_analysis = codec.decode_json(
        capsys.readouterr().out.encode(), d.AnalysisDocument
    )
    assert restored_analysis == analysis
    frozen_member = next(
        member for member in index.members if member.selection.id == "frozen"
    )
    frozen = diagnostics.read_evidence(relocated / frozen_member.relative_path)
    assert frozen.invocations == (original_invocation,)
    assert frozen.contexts == (attached,)
    retained_map = next(
        fact
        for fact in frozen.contexts[0].facts
        if fact.codec == "modeling-source-map.v1"
    )
    assert bytes(retained_map.payload) == codec.encode_json(source_map)
    assert frozen.invocations[0].attempts == original_summary


@pytest.mark.unit
@pytest.mark.parametrize("context_bytes", [None, 16 << 20, 1])
def test_original_initialization_selects_context_without_changing_scientific_result(
    context_bytes: int | None,
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    request_path, _ = source_recording_request(tmp_path, review=True)
    monkeypatch.setenv("PSE_DIAGNOSTIC_RECORD_REQUEST", str(request_path))
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        physical = runtime.physical_from_documents(physical_documents())
        package = runtime.modeling_from_documents(
            [
                package_documents(
                    "package initialization { def Root {var x:Scalar; "
                    "eq balance:x==3; annotation start x(0);}}"
                )
            ],
            physical,
        )
        case = next(
            row.declaration_id for row in package.declarations() if row.name == "Root"
        )
        settings = pse.SolveSettings(
            intent=NativeSolveIntent.INITIALIZE, backend=NativeBackend.KINSOL
        )
        assert settings.numerics.engineering_relative_fraction == 0.001
        prepared = package.prepare_block_initialization(case, settings, stage_batch(runtime, [()]))
        selection = (
            None
            if context_bytes is None
            else d.ReviewContextRequest(
                steps=(),
                include_sources=True,
                include_native_records=True,
                maximum_bytes=context_bytes,
            )
        )
        recording = runtime.observations().record_from_environment(
            ("root_evaluation.v1", "native_owner_return.v1")
        )
        assert recording is not None
        with recording:
            result = prepared.run(review_context=selection)
            recording.leave()
            original = result.initialization()
            assert original is not None
            assert original.completed_stages == 1
            assert not original.cancelled
            assert tuple(original.solved_unknowns.values()) == pytest.approx(
                [3.0], rel=0.001, abs=0.0
            )
            assert not result.failures()
            if context_bytes is None:
                with pytest.raises(pse.InspectionError, match="not selected"):
                    result.review_context()
            elif context_bytes == 1:
                with pytest.raises(pse.InspectionError, match="unavailable"):
                    result.review_context()
                assert result.initialization() == original
            else:
                context = result.review_context()
                run_id = result.run_id
                assert run_id is not None
                assert context.run_id == run_id.to_hex()
                assert context.operation is not None
                assert context.sources
                assert not any(
                    fact.codec == "invocation-evidence.v2" for fact in context.facts
                )
                outcome = next(
                    fact
                    for fact in context.facts
                    if fact.codec == "initialization-outcome.v1"
                )
                fields = codec.decode_json(
                    bytes(outcome.payload), dict[str, msgspec.Raw]
                )
                assert (
                    codec.decode_json(
                        bytes(fields["initialization"]), d.InitializationDocument
                    )
                    == original
                )
                receipt = recording.retain_context(context)
                assert receipt is not None
                acknowledgement = recording.acknowledgement
                assert acknowledgement is not None
                review = acknowledgement.review
                assert review is not None
                frozen = diagnostics.read_handoff(Path(review.receipt))
                assert frozen.contexts == (context,)
                request = codec.decode_json(
                    codec.encode_json({"version": 2, "operation": context.operation}),
                    d.AnalysisRequest,
                )
                analysis = diagnostics.analyze_evidence(frozen, request)
                output = tmp_path / "initialization-pack"
                diagnostics.export_review_pack(
                    output,
                    frozen,
                    analysis,
                    d.PackExportRequest(
                        selection=d.PackSelection(
                            members=(),
                            maximum_output_bytes=32 << 20,
                            maximum_document_bytes=16 << 20,
                        )
                    ),
                )
                relocated = tmp_path / "relocated-initialization-pack"
                shutil.move(output, relocated)
                assert diagnostics.review_pack(relocated).publication == "sealed"
                assert result.initialization() == original
    finally:
        runtime.close()


@pytest.mark.unit
def test_unsupported_cone_review_context_preserves_original_qualification(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        physical = runtime.physical_from_documents(physical_documents())
        assert_cone(runtime, physical, context_selected=True)
    finally:
        runtime.close()

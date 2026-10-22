# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive CLI-to-relocated-pack and explicit bounded tool/command controls."""

import shutil
import sqlite3
import subprocess
import sys
import threading
import time
from pathlib import Path
from typing import TYPE_CHECKING

import msgspec
import pyarrow as pa
import pytest
from msgspec.structs import replace
from scripts import diagnostic_trace

import pse
from pse import codec, diagnostics
from pse._build import _diagnostic_query_specification
from pse._diagnostic_query import run_query
from pse.contracts import documents as d
from pse.contracts.enums import NativeSolveIntent
from pse.contracts.identities import RunId
from pse.contracts.values import SemanticId
from pse.tests.test_diagnostics import diagnostic_native_solve
from pse.tests.test_frozen_review import source_recording_request
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents

if TYPE_CHECKING:
    from pse.tests.canonical_fixture import CanonicalFixture


def saved() -> tuple[d.FrozenEvidenceSet, d.AnalysisRequest, d.FrozenInspectionRequest]:
    products = codec.decode_json(
        (
            Path(__file__).parent / "fixtures/generated-native-boundaries/products.json"
        ).read_bytes(),
        dict[str, msgspec.Raw],
    )
    source = codec.decode_json(
        bytes(products["diagnostic_inspection"]), d.InspectionDocument
    )
    assert source.operation is not None
    evidence = d.FrozenEvidenceSet(
        version=2, checkpoints=(), operations=(source.operation,)
    )
    request = codec.decode_json(
        b'{"version":2,"operation":'
        + codec.encode_json(source.operation.id)
        + b',"baseline":null,"numerical_history_policy":null}',
        d.AnalysisRequest,
    )
    inspecting = d.FrozenInspectionRequest(version=1, inspection=source.request)
    return evidence, request, inspecting


def export_request() -> d.PackExportRequest:
    return d.PackExportRequest(
        version=2,
        selection=d.PackSelection(
            version=2,
            members=(),
            maximum_output_bytes=8 << 20,
            maximum_document_bytes=2 << 20,
        ),
        files=(),
    )


@pytest.mark.unit
def test_cli_passive_journey_uses_same_owner_after_relocation(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    evidence, request, inspection = saved()
    source = tmp_path / "original.json"
    source.write_bytes(codec.encode_json(evidence))
    analysis_request = tmp_path / "analysis.json"
    analysis_request.write_bytes(codec.encode_json(request))
    inspect_request = tmp_path / "inspect.json"
    inspect_request.write_bytes(codec.encode_json(inspection))
    export = tmp_path / "export.json"
    export.write_bytes(codec.encode_json(export_request()))
    assert (
        diagnostics.main(
            [
                "inspect-recorded",
                str(source),
                "--request",
                str(inspect_request),
                "--json",
            ]
        )
        == 0
    )
    recorded = codec.decode_json(
        capsys.readouterr().out.encode(), d.FrozenInspectionDocument
    )
    assert recorded.inspection.operation is not None
    assert recorded.inspection.operation.read_at == evidence.operations[0].read_at
    assert (
        diagnostics.main(
            ["analyze", str(source), "--request", str(analysis_request), "--json"]
        )
        == 0
    )
    analysis = codec.decode_json(capsys.readouterr().out.encode(), d.AnalysisDocument)
    assert len(analysis.questions) == 10
    pack = tmp_path / "pack"
    assert (
        diagnostics.main(
            [
                "export",
                str(source),
                "--analysis",
                str(analysis_request),
                "--request",
                str(export),
                "--output",
                str(pack),
                "--json",
            ]
        )
        == 0
    )
    index = codec.decode_json(capsys.readouterr().out.encode(), d.PackIndex)
    assert str(index.publication) == "sealed"
    moved = tmp_path / "moved"
    shutil.move(pack, moved)
    source.unlink()
    (tmp_path / "original.json").write_text("deceptive live neighbour")
    # Offline reads must not enter live checkpoint lookup or scientific execution.
    monkeypatch.setattr(
        diagnostics, "inspect_checkpoint", lambda *_a, **_k: pytest.fail("live lookup")
    )
    assert diagnostics.main(["review", str(moved), "--json"]) == 0
    restored = codec.decode_json(capsys.readouterr().out.encode(), d.PackIndex)
    assert restored == index
    assert (
        diagnostics.main(
            ["analyze", str(moved), "--request", str(analysis_request), "--json"]
        )
        == 0
    )
    assert (
        codec.decode_json(capsys.readouterr().out.encode(), d.AnalysisDocument)
        == analysis
    )
    report = moved / next(
        m.relative_path for m in restored.members if m.selection.id == "report_html"
    )
    html = report.read_text()
    assert "https://" not in html
    assert "<script" not in html


def query_request(question: str = "timeline") -> d.ReviewQueryRequest:
    return codec.decode_json(
        codec.encode_json(
            {
                "version": 1,
                "input_member": "capture-A",
                "question": question,
                "clock_scope": "capture-A:monotonic",
                "interval": {"start": 4, "end": 8},
                "timeout_seconds": 1.0,
                "output_bytes": 8192,
            }
        ),
        d.ReviewQueryRequest,
    )


@pytest.mark.unit
def test_declared_interval_query_uses_running_max_and_preserves_populations() -> None:

    spec = _diagnostic_query_specification(query_request())
    database = sqlite3.connect(":memory:")
    database.executescript(
        "CREATE TABLE track(id INTEGER,name TEXT); CREATE TABLE "
        "slice(id INTEGER,track_id INTEGER,ts INTEGER,dur "
        "INTEGER,name TEXT); INSERT INTO track VALUES(1,'owner'); "
        "INSERT INTO slice "
        "VALUES(1,1,0,10,'parent'),(2,1,2,3,'child'),(3,1,7,1,'last"
        "'),(4,1,8,1,'outside'),(5,1,4,0,'instant'),(6,1,5,-1,'open"
        "');"
    )
    try:
        rows = database.execute(spec.sql).fetchall()
    finally:
        database.close()
    # [4,8): parent4 + child1 + last1=6 inclusive, union4. Original10+3+1=14.
    assert rows == [(1, "owner", 3, 14, 6, 4, 0, 10, 1, 1)]
    # Same numeric track id in another capture is never implicitly joined.
    assert query_request().input_member == "capture-A"


def tool(tmp_path: Path, body: str) -> Path:
    binary = tmp_path / "owned query tool"
    binary.write_text(
        f"#!{sys.executable}\n"
        "import sys\n"
        "if '--version' in sys.argv:\n"
        " print('fixture-tool 1.0'); sys.exit(0)\n"
        f"{body}\n"
    )
    binary.chmod(0o755)
    return binary


@pytest.mark.unit
def test_query_context_distinguishes_empty_missing_failed_malformed_and_timeout(
    tmp_path: Path,
) -> None:

    raw = tmp_path / "trace"
    raw.write_bytes(b"original raw")
    request = query_request("coverage")
    empty = tool(tmp_path, "print('name,severity,source,value')")
    result = diagnostics.query_trace(request, raw, tmp_path / "empty", empty)
    assert str(result.status) == "completed"
    assert result.rows == 0
    assert result.tool_version == "fixture-tool 1.0"
    assert result.sql
    assert result.query_version == 2
    missing = diagnostics.query_trace(
        request, raw, tmp_path / "missing", tmp_path / "absent"
    )
    assert str(missing.status) == "unavailable"
    assert missing.rows is None
    empty.write_text(
        f"#!{sys.executable}\n"
        "import sys\n"
        "if '--version' in sys.argv: print('fixture-tool 1.0');sys.exit(0)\n"
        "print('name,severity,source,value');sys.exit(3)\n"
    )
    failed = diagnostics.query_trace(request, raw, tmp_path / "failed", empty)
    assert str(failed.status) == "failed"
    assert failed.output_prefix_bytes > 0
    assert failed.rows is None
    empty.write_text(
        f"#!{sys.executable}\n"
        "import sys\n"
        "if '--version' in sys.argv: print('fixture-tool 1.0');sys.exit(0)\n"
        "print('wrong,schema')\n"
    )
    malformed = diagnostics.query_trace(request, raw, tmp_path / "malformed", empty)
    assert str(malformed.status) == "malformed"
    empty.write_text(
        f"#!{sys.executable}\n"
        "import sys,time\n"
        "if '--version' in sys.argv: print('fixture-tool 1.0');sys.exit(0)\n"
        "print('name,severity,source,value',flush=True);time.sleep(10)\n"
    )
    timed = diagnostics.query_trace(
        replace(request, timeout_seconds=0.1), raw, tmp_path / "timed", empty
    )
    assert str(timed.status) == "timed_out"
    assert timed.output_prefix_bytes > 0
    assert raw.read_bytes() == b"original raw"


@pytest.mark.unit
def test_noncooperating_command_success_is_not_source_recording(tmp_path: Path) -> None:
    request_file = tmp_path / "request.json"
    request_file.write_bytes(
        codec.encode_json(
            {
                "version": 2,
                "detail": {
                    "version": 1,
                    "families": ["native_iteration.v1"],
                    "requirement": "complete",
                    "sample_every": 1,
                    "boundary_thread_cpu": False,
                    "managed_budget": None,
                    "expiry_monotonic_ns": None,
                    "duration": {"secs": 1, "nanos": 0},
                    "output": str(tmp_path / "source"),
                    "budget": {"memory_bytes": 4 << 20, "output_bytes": 1 << 20},
                },
                "acknowledgement": str(tmp_path / "ack.json"),
            }
        )
    )
    request = codec.decode_json(
        codec.encode_json(
            {
                "version": 1,
                "request_file": str(request_file),
                "argv": [sys.executable, "-c", "print('success')"],
                "cwd": None,
                "acknowledgement_timeout": {"secs": 1, "nanos": 0},
                "command_timeout": {"secs": 2, "nanos": 0},
                "output": str(tmp_path / "launch"),
                "output_bytes": 1024,
            }
        ),
        d.SourceRecordingLaunchRequest,
    )
    result = diagnostics.record_launch(request)
    assert result.command_status == 0
    assert result.acknowledgement is None
    assert result.stdout_prefix_bytes == 8
    assert not result.logs_truncated
    assert any("unavailable" in limit for limit in result.limitations)


@pytest.mark.unit
def test_malformed_source_ack_does_not_stop_science_and_claim_is_exclusive(
    tmp_path: Path,
) -> None:

    request_file = tmp_path / "request.json"
    source = {
        "version": 2,
        "detail": {
            "version": 1,
            "families": ["root_evaluation.v1"],
            "requirement": "complete",
            "sample_every": 1,
            "boundary_thread_cpu": False,
            "managed_budget": None,
            "expiry_monotonic_ns": None,
            "duration": {"secs": 2, "nanos": 0},
            "output": str(tmp_path / "source"),
            "budget": {"memory_bytes": 4 << 20, "output_bytes": 1 << 20},
        },
        "acknowledgement": str(tmp_path / "ack.json"),
    }
    request_file.write_bytes(codec.encode_json(source))
    marker = tmp_path / "scientific-completion"
    command = (
        "from pathlib import Path;import time;"
        f"Path({str(tmp_path / 'ack.json')!r}).write_text('malformed ACK');"
        "time.sleep(.3);"
        f"Path({str(marker)!r}).write_text('finished')"
    )
    request = codec.decode_json(
        codec.encode_json(
            {
                "version": 1,
                "request_file": str(request_file),
                "argv": [sys.executable, "-c", command],
                "cwd": None,
                "acknowledgement_timeout": {"secs": 1, "nanos": 0},
                "command_timeout": {"secs": 2, "nanos": 0},
                "output": str(tmp_path / "launch"),
                "output_bytes": 1024,
            }
        ),
        d.SourceRecordingLaunchRequest,
    )
    results: list[d.SourceRecordingLaunchDocumentOutput] = []
    errors: list[BaseException] = []

    def run() -> None:
        try:
            results.append(diagnostics.record_launch(request))
        except BaseException as error:
            errors.append(error)

    first = threading.Thread(target=run)
    first.start()
    claim = tmp_path / "ack.json.launch"
    until = time.monotonic() + 2
    while not claim.exists() and time.monotonic() < until:
        time.sleep(0.005)
    with pytest.raises(FileExistsError):
        diagnostics.record_launch(replace(request, output=str(tmp_path / "second")))
    first.join(timeout=5)
    assert not first.is_alive()
    assert not errors
    assert marker.read_text() == "finished"
    assert results[0].command_status == 0
    assert results[0].acknowledgement is None
    assert not results[0].cancelled
    assert results[0].drained
    assert any("acknowledgement unavailable" in item for item in results[0].limitations)
    assert not claim.exists()


@pytest.mark.unit
def test_query_stops_normal_exit_descendants_before_finalization(
    tmp_path: Path,
) -> None:
    raw = tmp_path / "trace"
    raw.write_bytes(b"raw")
    later = tmp_path / "should-not-appear"
    body = (
        "import os,time,signal\n"
        "if os.fork()==0:\n"
        " signal.signal(signal.SIGTERM,signal.SIG_IGN)\n"
        " os.close(1);os.close(2)\n"
        " time.sleep(.8)\n"
        f" open({str(later)!r},'w').write('late')\n"
        " os._exit(0)\n"
        "print('name,severity,source,value')"
    )
    binary = tool(tmp_path, body)
    result = diagnostics.query_trace(
        query_request("coverage"), raw, tmp_path / "query", binary
    )
    assert str(result.status) == "failed"
    assert result.rows is None
    time.sleep(0.9)
    assert not later.exists()


@pytest.mark.unit
def test_explicit_diagnostic_work_keeps_origin_and_prehandle_failure(
    inspection_settings: pse.EngineSettings,
) -> None:

    root = Path(__file__).resolve().parents[3]
    policy = codec.decode_json(
        (root / "packages/reference/diagnostics/idaes-2.13.json").read_bytes(),
        d.ModelingDiagnosticPolicy,
    )
    case = "abababababababababababababababab"
    request = d.DiagnosticExecutionRequest(
        physical_documents=physical_documents(),
        model_documents=(
            package_documents(
                "package explicit { def D {var x:Scalar; eq e:x==3; "
                "annotation start x(0);} "
                f'@id("{case}") case run {{child root:D=D();}} }}'
            ),
        ),
        case_id=SemanticId.from_hex(case),
        settings=pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        diagnostics=codec.convert_document(
            codec.document_builtins(policy),
            d.ModelingDiagnosticPolicyBody,
        ),
        origin=d.DiagnosticOriginBody(run_id=RunId(SemanticId.from_hex("cd" * 16))),
        allowance=d.ClosedDuration(secs=10, nanos=0),
        maximum_source_bytes=4 << 20,
    )
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        result = runtime.diagnose_documents(request)
        assert result.origin == request.origin
        assert result.failure is None
        assert result.run_id is not None
        assert result.complete
        assert result.row_nominals
        assert result.variable_nominals
        failed = runtime.diagnose_documents(replace(request, maximum_source_bytes=1))
        assert failed.operation != result.operation
        assert failed.origin == request.origin
        assert failed.failure is not None
        assert failed.run_id is None
        assert not failed.complete
        expired = runtime.diagnose_documents(
            replace(request, allowance=d.ClosedDuration(secs=0, nanos=1))
        )
        assert expired.operation not in {result.operation, failed.operation}
        assert expired.failure is not None
        assert expired.run_id is None
        assert not expired.complete
        assert request.settings.numerics.engineering_relative_fraction == 0.001
    finally:
        runtime.close()


@pytest.mark.unit
def test_query_cancellation_and_output_cap_keep_failure_prefixes(
    tmp_path: Path,
) -> None:

    raw = tmp_path / "raw"
    raw.write_bytes(b"raw")
    binary = tool(tmp_path, "import os\nos.write(1,b'x'*100000)")
    limited = run_query(
        replace(query_request("coverage"), output_bytes=4096),
        raw,
        tmp_path / "limited",
        binary,
    )
    assert limited.status == "output_limit"
    assert limited.rows is None
    assert 0 < limited.output_prefix_bytes <= 2048
    stopped = run_query(
        query_request("coverage"),
        raw,
        tmp_path / "stopped",
        binary,
        cancelled=lambda: True,
    )
    assert stopped.status == "cancelled"
    assert stopped.rows is None
    assert raw.read_bytes() == b"raw"


@pytest.mark.unit
def test_trace_cli_adapter_uses_literal_paths_and_contextual_result(
    tmp_path: Path,
) -> None:

    raw = tmp_path / "raw $(not shell).pftrace"
    raw.write_bytes(b"retained raw")
    binary = tool(
        tmp_path,
        "print('name,severity,source,value')\nprint('declared,info,fixture,1')\nopen(sys.argv[-1]+'.argv','w').write(repr(sys.argv))",
    )
    assert (
        diagnostic_trace.query(raw, "coverage", tmp_path / "query", 2, 8192, binary)
        == 0
    )
    assert str(raw) in Path(str(raw) + ".argv").read_text()
    assert raw.read_bytes() == b"retained raw"
    result = codec.decode_json(
        (tmp_path / "query/query-result.json").read_bytes(), d.ReviewQueryDocument
    )
    assert result.status == "completed"
    assert result.rows == 1


@pytest.mark.unit
def test_query_refuses_invalid_allowance_or_existing_destination(
    tmp_path: Path,
) -> None:

    raw = tmp_path / "raw"
    raw.write_bytes(b"raw")
    binary = tool(tmp_path, "raise AssertionError('must not run')")
    request = query_request("coverage")
    for invalid in (
        replace(request, timeout_seconds=0),
        replace(request, timeout_seconds=float("nan")),
        replace(request, timeout_seconds=float("inf")),
        replace(request, output_bytes=1),
    ):
        with pytest.raises((ValueError, pse.InspectionError)):
            diagnostics.query_trace(invalid, raw, tmp_path / "refused", binary)
        assert not (tmp_path / "refused").exists()
    existing = tmp_path / "existing"
    existing.mkdir()
    (existing / "sentinel").write_text("preserve")
    with pytest.raises(FileExistsError):
        diagnostics.query_trace(request, raw, existing, binary)
    assert (existing / "sentinel").read_text() == "preserve"


@pytest.mark.unit
def test_query_table_cli_writes_registry_typed_ipc_stream(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:

    request = query_request("coverage")
    specification = _diagnostic_query_specification(request)
    query = d.ReviewQueryDocument(
        version=1,
        request=request,
        query_version=specification.query_version,
        sql=specification.sql,
        tool_version="fixture 1.0",
        status="completed",
        exit_code=0,
        columns=tuple(codec.convert_document(
            codec.document_builtins(column),
            d.ReviewQueryColumn,
        ) for column in specification.columns),
        covered_interval=None,
        output="rows.csv",
        output_prefix_bytes=(1 << 53) + 1,
        error_output="stderr.txt",
        error_prefix_bytes=0,
        rows=(1 << 53) + 1,
        limitations=specification.limitations,
    )
    evidence = replace(saved()[0], query_results=(query,))
    source = tmp_path / "frozen.json"
    source.write_bytes(codec.encode_json(evidence))
    table_request = d.ReviewTableRequest(kind="query", index=0, memory_bytes=4 << 20)
    selection = tmp_path / "table.json"
    selection.write_bytes(codec.encode_json(table_request))
    output = tmp_path / "table"
    assert (
        diagnostics.main(
            [
                "query",
                str(source),
                "--table",
                "--request",
                str(selection),
                "--output",
                str(output),
                "--json",
            ]
        )
        == 0
    )
    result = codec.decode_json(capsys.readouterr().out.encode(), d.ReviewTableDocument)
    assert result.rows == 1
    assert result.request == table_request
    assert result.schema_metadata
    with pa.ipc.open_stream(output / "rows.arrow") as reader:
        table = reader.read_all()
    assert table.column("rows").type == pa.uint64()
    assert table.column("rows")[0].as_py() == (1 << 53) + 1
    # The public supplied-only publisher also declares and exports this typed
    # diagnostic member using its registry/source contract, without science lineage.
    analysis = diagnostics.analyze_evidence(evidence, saved()[1])
    selected = replace(
        export_request(),
        tables=(d.PackTableInput(member="query-table", request=table_request),),
    )
    pack = tmp_path / "typed-pack"
    index = diagnostics.export_review_pack(pack, evidence, analysis, selected)
    assert index.publication == "sealed"
    assert index.membership_complete
    member = next(
        member for member in index.members if member.selection.id == "query-table"
    )
    assert member.selection.result is None
    assert member.selection.diagnostic_table is not None
    with pa.ipc.open_stream(pack / member.relative_path) as reader:
        assert reader.read_all().column("rows")[0].as_py() == (1 << 53) + 1
    assert diagnostics.review_pack(pack) == index


@pytest.mark.unit
def test_cli_explicit_live_source_joins_selected_handoff_without_replacing_snapshot(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    request_path, _ = source_recording_request(tmp_path, review=True)
    monkeypatch.setenv("PSE_DIAGNOSTIC_RECORD_REQUEST", str(request_path))
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        observations = runtime.observations()
        selected = observations.record_from_environment(
            ("root_evaluation.v1", "native_owner_return.v1")
        )
        assert selected is not None
        with selected:
            handle = diagnostic_native_solve(runtime)
            selected.leave()
            result = handle.wait()
            assert result.usable
            reader = handle.observations()
            assert reader is not None
            receipt = selected.retain_result(result)
            assert receipt is not None
            assert len(receipt.members) == 1
            acknowledgement = selected.acknowledgement
            assert acknowledgement is not None
            assert acknowledgement.review is not None
            handoff = Path(acknowledgement.review.receipt)
        # Wait for the actual one-second publisher, retaining the exact operation
        # reader so its source membership cannot expire while selecting it.
        until = time.monotonic() + 8
        while True:
            if observations.checkpoint.is_file():
                image = diagnostics.read_checkpoint(observations.checkpoint)
                if any(operation.id == reader.id for operation in image.operations):
                    break
            assert time.monotonic() < until, "joined operation was not published"
            time.sleep(0.02)
        original = next(
            operation for operation in image.operations if operation.id == reader.id
        )
        source = tmp_path / "selected-live-checkpoint.json"
        source.write_bytes(codec.encode_json(image))
        operation = reader.id
    finally:
        runtime.close()
    del result, handle, reader, selected, runtime
    retained = diagnostics.read_handoff(handoff)
    assert retained.contexts
    assert retained.invocations
    assert retained.invocations[0].operation == operation
    inspecting = d.FrozenInspectionRequest(
        version=1, inspection=d.InspectionRequest(operation=codec.convert_document(
            codec.document_builtins(operation),
            d.OperationIdBody,
        ))
    )
    inspect_path = tmp_path / "inspect-live.json"
    inspect_path.write_bytes(codec.encode_json(inspecting))
    assert (
        diagnostics.main(
            [
                "inspect-recorded",
                str(source),
                "--source",
                "live",
                "--handoff",
                str(handoff),
                "--request",
                str(inspect_path),
                "--json",
            ]
        )
        == 0
    )
    inspected = codec.decode_json(
        capsys.readouterr().out.encode(), d.FrozenInspectionDocument
    )
    assert inspected.inspection.operation is not None
    assert inspected.inspection.operation.id == operation
    assert inspected.inspection.operation.read_at == original.read_at
    request = codec.decode_json(
        codec.encode_json(
            {
                "version": 2,
                "operation": operation,
                "baseline": None,
                "question": "physical_attribution",
                "numerical_history_policy": None,
            }
        ),
        d.AnalysisRequest,
    )
    analysis_path = tmp_path / "analyze-live.json"
    analysis_path.write_bytes(codec.encode_json(request))
    assert (
        diagnostics.main(
            [
                "analyze",
                str(source),
                "--source",
                "live",
                "--handoff",
                str(handoff),
                "--request",
                str(analysis_path),
                "--json",
            ]
        )
        == 0
    )
    joined = codec.decode_json(capsys.readouterr().out.encode(), d.AnalysisDocument)
    assert any(
        reference.member == "frozen" and reference.pointer.startswith("/contexts/")
        for question in joined.questions
        for statement in question.statements
        for reference in statement.evidence
    )
    chosen = {operation.id for operation in image.operations}
    expected = replace(
        retained,
        checkpoints=(image,),
        operations=tuple(
            operation for operation in retained.operations if operation.id not in chosen
        ),
    )
    assert joined == diagnostics.analyze_evidence(expected, request)
    preparation = diagnostics.analyze_evidence(
        expected, replace(request, question="preparation")
    )
    assert any(
        statement.code == "engine.actual_session_policy"
        and any(
            reference.pointer.startswith("/contexts/")
            for reference in statement.evidence
        )
        for question in preparation.questions
        for statement in question.statements
    )
    assert (
        diagnostics.main(
            [
                "analyze",
                str(handoff),
                "--source",
                "handoff",
                "--request",
                str(analysis_path),
                "--json",
            ]
        )
        == 0
    )
    offline = codec.decode_json(capsys.readouterr().out.encode(), d.AnalysisDocument)
    assert offline == diagnostics.analyze_evidence(retained, request)


@pytest.mark.component
def test_cli_handoff_exports_multiple_protected_scientific_selections(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: "CanonicalFixture",
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    request_path, _ = source_recording_request(tmp_path, review=True)
    monkeypatch.setenv("PSE_DIAGNOSTIC_RECORD_REQUEST", str(request_path))
    runtime = canonical_substrate.runtime(inspection_settings)
    selected = runtime.observations().record_from_environment(
        ("root_evaluation.v1", "native_owner_return.v1")
    )
    assert selected is not None
    with selected:
        handle = diagnostic_native_solve(runtime)
        selected.leave()
        result = handle.wait()
        assert result.usable
        invocation = result.invocation_evidence
        run, attempt = result.canonical_run_key, result.canonical_attempt_key
        assert run is not None
        assert attempt is not None
        assert invocation.operation is not None
        retained = selected.retain_result(result)
        assert retained is not None
        ack = selected.acknowledgement
        assert ack is not None
        assert ack.review is not None
        handoff = Path(ack.review.receipt)
    evidence = diagnostics.read_handoff(handoff)
    assert evidence.contexts
    assert evidence.invocations[0].canonical_attempt == attempt
    with runtime.results(run, attempt, "runtime.solve_variables") as rows:
        original = pa.table(rows)
    assert original.num_rows > 0
    symbol = original.column("symbol_id")[0].as_py()
    assert isinstance(symbol, bytes)
    request = d.PackExportRequest(
        selection=d.PackSelection(
            members=(), maximum_output_bytes=32 << 20, maximum_document_bytes=16 << 20
        ),
        scientific_results=(
            d.ReviewScientificSelection(
                member="original",
                run=run,
                attempt=attempt,
                relation="runtime.solve_variables",
                row_start=0,
                row_end=1,
            ),
            d.ReviewScientificSelection(
                member="selected",
                run=run,
                attempt=attempt,
                relation="runtime.solve_variables",
                row_start=0,
                row_end=(1 << 64) - 1,
                output=d.ReviewOutputSelection(
                    owner=SemanticId(symbol),
                    field="value",
                    partition=f"step:{original.column('step')[0].as_py()}",
                ),
            ),
        ),
    )
    analysis = codec.decode_json(
        codec.encode_json(
            {
                "version": 2,
                "operation": invocation.operation,
                "question": "physical_attribution",
            }
        ),
        d.AnalysisRequest,
    )
    deployment = d.ReviewExportDeployment(
        runtime=d.DiagnosticRuntimeSettings(
            memory_limit_bytes=inspection_settings.memory_limit_bytes,
            threads=inspection_settings.threads,
            spill_dir=str(tmp_path / "export-spill"),
            max_spill_bytes=1 << 30,
            batch_size=7,
        ),
        substrate=canonical_substrate.state,
        database=canonical_substrate.database,
        producer=None,
    )
    (tmp_path / "export-spill").mkdir()
    paths = []
    for name, value in [
        ("analysis", analysis),
        ("export", request),
        ("deployment", deployment),
    ]:
        path = tmp_path / (name + ".json")
        path.write_bytes(codec.encode_json(value))
        paths.append(path)
    output = tmp_path / "scientific-pack"
    # A standalone CLI process owns its explicit deployment budget. The parent
    # keeps its existing runtime budget; neither process rewrites the other's.
    command = subprocess.run(
        [
            sys.executable,
            "-m",
            "pse.diagnostics",
            "export",
            str(handoff),
            "--source",
            "handoff",
            "--analysis",
            str(paths[0]),
            "--request",
            str(paths[1]),
            "--deployment",
            str(paths[2]),
            "--output",
            str(output),
            "--json",
        ],
        capture_output=True,
        timeout=60,
        check=False,
    )
    assert command.returncode == 0, command.stderr.decode()
    index = codec.decode_json(command.stdout, d.PackIndex)
    assert index.publication == "sealed"
    assert index.membership_complete
    assert index.self_contained
    assert not index.missing_context
    for name in ("original", "selected"):
        member = next(member for member in index.members if member.selection.id == name)
        assert member.selection.result is not None
        assert member.selection.result.run == run
        assert member.selection.result.attempt == attempt
        assert member.selection.result.usability == "unknown"
        with pa.ipc.open_stream(output / member.relative_path) as stream:
            assert stream.schema.metadata[b"pse.canonical.attempt"].decode() == attempt
            assert stream.read_all().num_rows == 1
    # The original runtime remains usable; CLI readers were drained before sealing.
    with runtime.results(
        run, attempt, "runtime.solve_variables", start=0, end=1
    ) as rows:
        assert pa.table(rows).num_rows == 1
    moved = tmp_path / "relocated-scientific-pack"
    shutil.move(output, moved)
    assert diagnostics.review_pack(moved) == index

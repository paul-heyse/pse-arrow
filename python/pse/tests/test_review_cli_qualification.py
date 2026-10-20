# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""An actual cooperating solve reaches relocated offline review through the CLI."""

import shutil
import sys
from pathlib import Path

import pytest

import pse
from pse import codec, diagnostics
from pse.contracts import documents as d
from pse.tests.test_frozen_review import source_recording_request

_COOPERATOR = """
import sys
import pyarrow as pa
import pse
from pse.tests.test_diagnostics import diagnostic_native_solve
settings = pse.EngineSettings(
    memory_limit_bytes=int(sys.argv[1]), threads=2, target_partitions=1,
    spill_dir=sys.argv[2], max_spill_bytes=1 << 30, batch_size=7,
    cache=pse.CacheSettings(working_bytes=16 << 30, metadata_bytes=8 << 20,
        concurrent_loads=2, inflight_bytes=64 << 20, inspection_bytes=4 << 20),
)
runtime = pse.Runtime(settings, ephemeral=True)
try:
    recording = runtime.observations().record_from_environment(
        ('root_evaluation.v1', 'native_owner_return.v1'))
    assert recording is not None
    with recording:
        handle = diagnostic_native_solve(runtime)
        recording.leave()
        result = handle.wait()
        assert result.usable
        values = (
            pa.table(result.table('runtime.solve_variables'))
            .column('value').to_pylist()
        )
        assert len(values) == 1
        assert abs(values[0] - 3.0) <= 0.003
        assert result.attempt_summary.steps[0].work.evaluations > 0
        assert recording.retain_result(result) is not None
finally:
    runtime.close()
"""


@pytest.mark.component
def test_cli_cooperating_native_solve_to_relocated_offline_review(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source, _ = source_recording_request(tmp_path, review=True)
    spill = tmp_path / "spill"
    spill.mkdir()
    launch = d.SourceRecordingLaunchRequest(
        request_file=str(source),
        argv=(
            sys.executable,
            "-c",
            _COOPERATOR,
            str(inspection_settings.memory_limit_bytes),
            str(spill),
        ),
        acknowledgement_timeout=d.ClosedDuration(secs=30, nanos=0),
        command_timeout=d.ClosedDuration(secs=60, nanos=0),
        output=str(tmp_path / "launch"),
        output_bytes=64 << 10,
    )
    launch_path = tmp_path / "launch-request.json"
    launch_path.write_bytes(codec.encode_json(launch))
    assert (
        diagnostics.main(["record-launch", "--request", str(launch_path), "--json"])
        == 0
    )
    outcome = codec.decode_json(
        capsys.readouterr().out, d.SourceRecordingLaunchDocument
    )
    assert outcome.command_status == 0, outcome
    assert outcome.drained
    assert not outcome.cancelled
    assert outcome.acknowledgement is not None
    assert outcome.acknowledgement.review is not None
    handoff = Path(outcome.acknowledgement.review.receipt)
    evidence = diagnostics.read_handoff(handoff)
    assert len(evidence.invocations) == len(evidence.contexts) == 1
    invocation = evidence.invocations[0]
    assert invocation.operation is not None
    assert invocation.attempts.usable
    context = evidence.contexts[0]
    assert context.run_id == invocation.run_id
    assert any(fact.codec == "native-attempt-record.v1" for fact in context.facts)
    analysis_request = d.AnalysisRequest(operation=invocation.operation)
    request_path = tmp_path / "analysis.json"
    request_path.write_bytes(codec.encode_json(analysis_request))
    export_path = tmp_path / "export.json"
    export_path.write_bytes(
        codec.encode_json(
            d.PackExportRequest(
                selection=d.PackSelection(
                    members=(),
                    maximum_output_bytes=32 << 20,
                    maximum_document_bytes=16 << 20,
                )
            )
        )
    )
    pack = tmp_path / "pack"
    assert (
        diagnostics.main(
            [
                "export",
                str(handoff),
                "--source",
                "handoff",
                "--analysis",
                str(request_path),
                "--request",
                str(export_path),
                "--output",
                str(pack),
                "--json",
            ]
        )
        == 0
    )
    index = codec.decode_json(capsys.readouterr().out, d.PackIndex)
    assert index.publication == "sealed"
    moved = tmp_path / "external-review"
    shutil.move(pack, moved)
    handoff.rename(tmp_path / "unavailable-original-handoff.json")
    (moved / "context.json").write_bytes(b"conflicting live sibling")

    def forbidden_runtime(*_args: object, **_kwargs: object) -> None:
        message = "offline review entered the scientific runtime"
        raise AssertionError(message)

    monkeypatch.setattr(pse, "Runtime", forbidden_runtime)
    assert diagnostics.main(["review", str(moved), "--json"]) == 0
    assert codec.decode_json(capsys.readouterr().out, d.PackIndex) == index
    assert (
        diagnostics.main(
            ["analyze", str(moved), "--request", str(request_path), "--json"]
        )
        == 0
    )
    analysis = codec.decode_json(capsys.readouterr().out, d.AnalysisDocument)
    assert analysis.request.operation == invocation.operation

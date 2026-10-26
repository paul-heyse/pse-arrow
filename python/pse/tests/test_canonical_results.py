# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact canonical identities, bounded result streams and local IPC export."""

import ctypes
import hashlib
import json
import os
import shlex
import signal
import socket
import subprocess
import sys
import threading
import time
from collections.abc import Callable
from pathlib import Path
from types import FrameType
from typing import cast

import msgspec
import pyarrow as pa
import pytest
from scripts import surreal_server as server

import pse
from pse import codec
from pse._build import compiled_features
from pse.contracts.documents import (
    AnalysisDirection,
    ArtifactIntent,
    HistoryObligation,
    HistoryOwner,
    ObservationText,
    SensitivityRequest,
)
from pse.contracts.enums import (
    ArtifactOwnerKind,
    CanonicalAttemptPhase,
    CanonicalTerminalOutcome,
    DiagnosticCode,
    FailureClass,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
    ScientificArtifactKind,
)
from pse.contracts.identities import DeclarationId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.physical_input_fixture import inspection_request

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


def _physical(runtime: pse.Runtime) -> pse.PhysicalContext:
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    return runtime.physical_from_documents(
        {
            str(p.relative_to(primitives)): p.read_text()
            for p in primitives.rglob("*")
            if p.is_file()
        }
    )


def _package(
    runtime: pse.Runtime, source: str = SOURCE
) -> tuple[pse.ModelingPackage, DeclarationId]:
    root = Path(__file__).resolve().parents[3]
    physical = _physical(runtime)
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": source}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    return package, case


def _settings() -> pse.SolveSettings:
    return pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )


def _physical_cache_usage(runtime: pse.Runtime) -> tuple[int, int]:
    # Resource usage covers the shared process; each workflow runtime registers
    # its own component under this family name.
    caches = [
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.physical_admission"
    ]
    assert caches
    return (
        sum(cache.entries for cache in caches),
        sum(cache.retained_bytes for cache in caches),
    )


@pytest.mark.component
def test_canonical_constructor_preserves_interpretation_and_transport_timeout(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    admitted_study_module: str,
    no_canonical_allocation: None,
) -> None:
    state = tmp_path / "private-storage"
    state.mkdir(mode=0o700)
    credentials = state / "credentials.json"
    credentials.write_text(
        json.dumps(
            {
                "username": "fixture",
                "password": "fixture",
                "selection_username": "fixture_viewer",
                "selection_password": "fixture",
            }
        )
    )
    credentials.chmod(0o600)
    # Borrow the immutable selected artifact receipt. The private endpoint below
    # is synthetic, but its local module admission must use real pinned evidence.
    config = {
        "profile_version": 3,
        "study_module": json.loads(admitted_study_module),
        "endpoint": "grpc://127.0.0.1:1",
        "namespace": "fixture",
        "database": "fixture",
        "schema_interpretation": "unsupported-fixture-interpretation",
        "max_message_bytes": 4 * 1024 * 1024,
        "credentials_file": str(credentials),
        "accepting_writes": False,
        "resources": "deliberately unused native policy",
    }
    path = state / "config.json"

    def publish() -> None:
        path.write_text(json.dumps(config))
        path.chmod(0o600)

    publish()
    # Construction shares the process resource budget with other runtime handles;
    # changing that budget would refuse before either canonical boundary is reached.
    with pytest.raises(pse.InspectionError) as mismatch:
        pse.Runtime(inspection_settings, substrate=str(state))
    assert mismatch.value.report.code == DiagnosticCode.RUNTIME_INFRASTRUCTURE, (
        mismatch.value.report.message
    )
    assert mismatch.value.report.failure_class == FailureClass.RUNTIME_INFRASTRUCTURE
    interpretation_detail = mismatch.value.report.observations["detail"]
    assert isinstance(interpretation_detail, ObservationText)
    assert "unsupported-fixture-interpretation" in interpretation_detail.value
    stopped = threading.Event()
    accepted = threading.Event()
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        config["endpoint"] = f"grpc://127.0.0.1:{listener.getsockname()[1]}"
        config["schema_interpretation"] = server.SUBSTRATE_INTERPRETATION
        publish()

        def withhold_handshake() -> None:
            with listener.accept()[0]:
                accepted.set()
                stopped.wait()

        peer = threading.Thread(target=withhold_handshake)
        peer.start()
        try:
            # Consume the production constructor's original clock, without a test-only
            # timeout override or an invented shorter healthy-command deadline.
            with pytest.raises(pse.InspectionError) as timeout:
                pse.Runtime(inspection_settings, substrate=str(state))
            assert accepted.is_set()
            assert timeout.value.report.code == DiagnosticCode.RUNTIME_INFRASTRUCTURE, (
                timeout.value.report.message
            )
            assert (
                timeout.value.report.failure_class
                == FailureClass.RUNTIME_INFRASTRUCTURE
            )
            # An expired gRPC capability handshake keeps the public typed
            # infrastructure boundary and its owned diagnostic detail.
            detail = timeout.value.report.observations["detail"]
            assert isinstance(detail, ObservationText)
            assert "deadline expired" in detail.value
        finally:
            stopped.set()
            # If construction refused before connecting, wake the fixture's accept.
            if not accepted.is_set():
                with socket.create_connection(listener.getsockname()):
                    pass
            peer.join()


@pytest.mark.integration
def test_canonical_open_failure_and_invalid_user_range_keep_distinct_diagnostics(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    with pytest.raises(pse.InspectionError) as unavailable:
        pse.Runtime(
            inspection_settings,
            substrate=canonical_substrate.state,
            database=canonical_substrate.database + "_uninitialized",
        )
    assert unavailable.value.report.code == DiagnosticCode.RUNTIME_INFRASTRUCTURE
    assert unavailable.value.report.failure_class == FailureClass.RUNTIME_INFRASTRUCTURE
    runtime = canonical_substrate.runtime(inspection_settings)
    with pytest.raises(pse.InspectionError, match="range is reversed") as invalid:
        runtime.results(
            "unselected", "unselected", "runtime.solve_variables", start=2, end=1
        )
    assert invalid.value.report.code == DiagnosticCode.WORKFLOW_INPUT
    assert invalid.value.report.failure_class == FailureClass.COMPILE_MATH


@pytest.mark.integration
def test_canonical_close_and_idempotent_cache_clear_with_retained_payload(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    assert _physical_cache_usage(runtime) == (0, 0)
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    handle = prepared.start()
    result = handle.wait()
    assert result.usable
    assert result.canonical_run_key is not None
    attempt = result.canonical_attempt_key
    assert attempt is not None
    variables = pa.table(result.table("runtime.solve_variables"))
    (variable,) = [row for row in variables.to_pylist() if not row["parameter"]]
    assert variable["tolerance"] > 0.0
    assert abs(variable["value"] - 2.0) <= variable["tolerance"]
    (record,) = pa.table(runtime.attempt_record(attempt)).to_pylist()
    assert record["phase"] == CanonicalAttemptPhase.TERMINAL
    assert record["outcome"] == "succeeded"
    # The immutable receipt payload fits retention; active caller protection
    # is a separate owner and is never kept alive by this optional entry.
    retained = _physical_cache_usage(runtime)
    assert retained[0] == 1
    assert retained[1] > 0
    del variables, result, handle, prepared, package
    assert _physical_cache_usage(runtime) == retained
    # Caller close and explicit optional-cache clearing are separate operations.
    runtime.close()
    runtime.close()
    assert _physical_cache_usage(runtime) == retained
    with pytest.raises(pse.InspectionError, match="runtime caller is closed"):
        runtime.attempt_record(attempt)
    runtime.clear_program_cache()
    runtime.clear_program_cache()
    assert _physical_cache_usage(runtime) == (0, 0)


@pytest.mark.integration
def test_canonical_close_preserves_live_result_reader_and_independent_caller(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    assert _physical_cache_usage(runtime) == (0, 0)
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    handle = prepared.start()
    result = handle.wait()
    assert result.usable
    run = result.canonical_run_key
    attempt = result.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    expected = pa.table(result.table("runtime.solve_variables")).to_pylist()
    stream = runtime.results(run, attempt, "runtime.solve_variables")
    reader = pa.RecordBatchReader.from_stream(stream)
    del result, handle, prepared, package
    independent = canonical_substrate.runtime(inspection_settings)
    try:
        # Retained payloads own no active protection. Evict the optional entry;
        # the admitted reader must keep its independent permission for the fetch.
        retained = _physical_cache_usage(runtime)
        assert retained[0] == 1
        assert retained[1] > 0
        runtime.clear_program_cache()
        runtime.close()
        runtime.close()
        assert _physical_cache_usage(runtime) == (0, 0)
        with pytest.raises(pse.InspectionError, match="runtime caller is closed"):
            runtime.results(run, attempt, "runtime.solve_variables")
        # The admitted reader keeps its own activity and fetches the actual
        # canonical block after its original logical caller has closed.
        batch = reader.read_next_batch()
        assert batch.to_pylist() == expected
        assert (
            pa.table(
                independent.results(run, attempt, "runtime.solve_variables")
            ).to_pylist()
            == expected
        )
    finally:
        reader.close()
        stream.close()
    del batch, reader, stream
    runtime.close()


@pytest.mark.integration
def test_canonical_close_preserves_admitted_solve_but_refuses_new_start(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    handle = prepared.start()
    runtime.close()
    with pytest.raises(pse.InspectionError, match="runtime caller is closed"):
        prepared.start()
    result = handle.wait()
    assert result.usable
    attempt = result.canonical_attempt_key
    assert attempt is not None
    independent = canonical_substrate.runtime(inspection_settings)
    (record,) = pa.table(independent.attempt_record(attempt)).to_pylist()
    assert record["phase"] == CanonicalAttemptPhase.TERMINAL
    assert record["outcome"] == "succeeded"


@pytest.mark.integration
def test_canonical_result_selection_and_progress_reopen(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    assert runtime.durable
    package, case = _package(
        runtime,
        SOURCE.replace(
            "annotation check", "annotation objective x(minimize); annotation check"
        ),
    )
    parameter = next(
        member.id
        for member in package.inspect(inspection_request(case, _settings())).members
        if member.lineage.path == "a" or member.lineage.path.endswith(".a")
    )
    settings = msgspec.structs.replace(
        _settings(),
        intent=NativeSolveIntent.OPTIMIZE,
        sensitivity=SensitivityRequest(parameters=(parameter,)),
    )
    prepared = package.prepare_solve(case, settings)
    controls = pse.AnalysisControls(roots=(), direction=AnalysisDirection.DOWNSTREAM)
    dependencies = prepared.dependency_analysis(controls)
    dependency_rows = pa.table(dependencies.edges()).to_pylist()
    assert any(row["kind"] == "numerical_incidence" for row in dependency_rows)
    assert any(row["kind"] == "execution_dependency" for row in dependency_rows)
    assert all(row["evidence_artifact"] is None for row in dependency_rows)
    dependency_header = pa.table(dependencies.header())
    (dependency_occurrence,) = dependency_header.to_pylist()
    content_fields = [
        "revision",
        "method",
        "configuration",
        "input_digest",
        "primary_problem",
        "primary_authority",
        "interpretation",
        "node_count",
        "edge_count",
    ]

    def assert_same_dependency_content(other: pse.Analysis) -> None:
        other_header = pa.table(other.header())
        (occurrence,) = other_header.to_pylist()
        assert other.key != dependencies.key
        assert occurrence["key"] == other.key
        assert occurrence["creation_nonce"] != dependency_occurrence["creation_nonce"]
        assert occurrence["active"]
        assert not occurrence["retiring"]
        assert other_header.select(content_fields).equals(
            dependency_header.select(content_fields)
        )

    repeated_dependencies = prepared.dependency_analysis(controls)
    assert_same_dependency_content(repeated_dependencies)
    # Selection fails before creating an occurrence; no recovery key is invented.
    with pytest.raises(
        pse.InspectionError, match="analysis root is not a selected scientific object"
    ) as invalid_analysis:
        prepared.dependency_analysis(
            pse.AnalysisControls(
                roots=("missing-scientific-object",),
                direction=AnalysisDirection.DOWNSTREAM,
            )
        )
    assert isinstance(invalid_analysis.value.report, pse.DiagnosticReport)
    assert invalid_analysis.value.report.analysis_key is None
    handle = prepared.start()
    result = handle.wait()
    assert result.usable
    run = result.canonical_run_key
    attempt = result.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    assert handle.canonical_run_key == run
    assert handle.canonical_attempt_key == attempt

    # Another workflow handle reopens exact immutable members, without a retained
    # RunResult or a SQL/secondary publication authority.
    reopened = canonical_substrate.runtime(inspection_settings)
    (header,) = pa.table(reopened.run_record(run)).to_pylist()
    (record,) = pa.table(reopened.attempt_record(attempt)).to_pylist()
    (manifest,) = pa.table(reopened.result_manifest(attempt)).to_pylist()
    assert header["key"] == run
    assert record["key"] == attempt
    assert record["run"] == run
    assert record["phase"] == CanonicalAttemptPhase.TERMINAL
    assert record["outcome"] == "succeeded"
    assert manifest["attempt"] == attempt
    history_owner = codec.decode_json(
        json.dumps({"document_version": 1, "kind": "run", "key": run}).encode(),
        HistoryOwner,
    )
    audit = codec.decode_json(
        json.dumps(
            {
                "document_version": 1,
                "key": "python-history-audit",
                "owner": {"kind": "run", "key": run},
                "purpose": "audit",
                "attribution": "installed Python boundary",
            }
        ).encode(),
        HistoryObligation,
    )
    reopened.retain_history_obligation(audit)
    with pytest.raises(pse.InspectionError):
        reopened.release_history(history_owner)
    reopened.release_history_obligation(audit)
    with pytest.raises(pse.InspectionError):
        reopened.retain_history_obligation(audit)
    # Releasing an audit does not implicitly retire scientific payloads.
    with pytest.raises(pse.InspectionError):
        reopened.release_history(history_owner)
    with pytest.raises(pse.InspectionError):
        reopened.reclaim_history_page(history_owner)
    assert pa.table(reopened.run_record(run)).num_rows == 1
    reopened_dependencies = reopened.analysis(dependencies.key)
    assert pa.table(reopened_dependencies.edges()).to_pylist() == dependency_rows
    provenance = reopened.result_analysis(run, attempt, controls)
    (analysis,) = pa.table(provenance.header()).to_pylist()
    assert analysis["revision"] == header["revision"]
    assert analysis["method"] == "result-sensitivity-provenance-reachability:v2"
    assert any(
        row["kind"] == "input_provenance"
        for row in pa.table(provenance.edges()).to_pylist()
    )
    response_edges = pa.table(provenance.edges()).to_pylist()
    optimal = next(
        row
        for row in response_edges
        if row["kind"] == "retained_optimal_parametric_primal"
    )
    assert optimal["evidence_field"] == "primal"
    evidence_artifact = reopened.scientific_artifact(optimal["evidence_artifact"])
    assert evidence_artifact.kind == "local-analysis"
    quantitative = pa.table(
        evidence_artifact.block(optimal["evidence_block"])
    ).to_pylist()[optimal["evidence_row"]]
    assert quantitative["primal"] is not None
    assert quantitative["step"] == 0
    validity_edge = next(
        row for row in response_edges if row["kind"] == "response_validity"
    )
    original_validity = pa.table(
        evidence_artifact.block(validity_edge["evidence_block"])
    ).to_pylist()[validity_edge["evidence_row"]]["validity"]
    assert original_validity["certified"]
    assert original_validity["licq"]
    assert original_validity["second_order"]
    assert original_validity["root_rank"] is None
    # A selection refusal after copied scientific admission retains its exact
    # outstanding staging owner/read identity, without inventing an analysis.
    with pytest.raises(pse.InspectionError) as graph_transfer:
        reopened.result_analysis(
            run,
            attempt,
            pse.AnalysisControls(
                roots=("missing-scientific-object",),
                direction=AnalysisDirection.DOWNSTREAM,
            ),
        )
    transfer_report = graph_transfer.value.report
    assert transfer_report.analysis_key is None
    assert transfer_report.artifact_key is not None
    assert isinstance(transfer_report.artifact_intent, ArtifactIntent)
    assert transfer_report.artifact_owner_kind == "audit"
    assert transfer_report.artifact_owner is not None
    assert transfer_report.artifact_read_key is not None
    transfer_settlement = reopened.recover_artifact_intent(
        transfer_report.artifact_intent
    )
    assert transfer_settlement.key == transfer_report.artifact_key
    transfer_settlement.release(ArtifactOwnerKind.AUDIT, transfer_report.artifact_owner)
    assert not transfer_settlement.collect()

    # Invalid initial owner activation happens after actual staging. Its original
    # strict recovery document settles the inactive occurrence without a reader.
    with pytest.raises(pse.InspectionError) as inactive_transfer:
        reopened.promote_scientific_artifact(
            run,
            attempt,
            ScientificArtifactKind.FINAL_STATE,
            owner_kind=ArtifactOwnerKind.USER,
            owner="",
        )
    inactive_report = inactive_transfer.value.report
    assert inactive_report.artifact_key is not None
    assert isinstance(inactive_report.artifact_intent, ArtifactIntent)
    inactive_settlement = reopened.recover_artifact_intent(
        inactive_report.artifact_intent
    )
    assert inactive_settlement.key == inactive_report.artifact_key
    with pytest.raises(pse.InspectionError):
        reopened.scientific_artifact(inactive_settlement.key)
    assert not inactive_settlement.collect(), (
        "original finite creation window stays pending"
    )

    artifact = reopened.promote_scientific_artifact(
        run,
        attempt,
        ScientificArtifactKind.FINAL_STATE,
        owner_kind=ArtifactOwnerKind.USER,
        owner="python-final-state",
    )
    assert isinstance(artifact.original_intent, ArtifactIntent)
    settlement = reopened.recover_artifact_intent(artifact.original_intent)
    assert settlement.key == artifact.key
    assert not hasattr(settlement, "block")
    altered_intent = msgspec.structs.replace(
        artifact.original_intent, creation_request_digest="f" * 64
    )
    altered_settlement = reopened.recover_artifact_intent(altered_intent)
    with pytest.raises(pse.InspectionError):
        altered_settlement.release(ArtifactOwnerKind.USER, "python-final-state")
    with pytest.raises(TypeError):
        reopened.recover_artifact_intent(cast("ArtifactIntent", b"{}"))
    with pytest.raises(pse.InspectionError):
        reopened.recover_artifact_intent(
            msgspec.structs.replace(
                artifact.original_intent, provenance=b" " * (1024 * 1024 + 1)
            )
        )
    artifact.retain(ArtifactOwnerKind.AUDIT, "python-independent-audit")
    (artifact_header,) = pa.table(artifact.metadata()).to_pylist()
    assert artifact_header["kind"] == "final-state"
    copied = [
        pa.table(artifact.block(i)) for i in range(artifact_header["block_count"])
    ]
    assert any(table.num_rows and "value" in table.column_names for table in copied)
    artifact.release(ArtifactOwnerKind.USER, "python-final-state")
    with pytest.raises(pse.InspectionError):
        artifact.retain(ArtifactOwnerKind.USER, "python-final-state")
    assert pa.table(reopened.scientific_artifact(artifact.key).metadata()).num_rows == 1
    relation = "runtime.solve_runs"
    exact = pa.table(reopened.results(run, attempt, relation))
    latest = pa.table(
        reopened.latest_results(
            header["problem"], relation, classes=(CanonicalTerminalOutcome.SUCCEEDED,)
        )
    )
    assert exact.equals(latest)
    assert exact.num_rows == 1
    assert (
        pa.table(reopened.results(run, attempt, relation, start=1, end=1)).num_rows == 0
    )

    with reopened.progress(run, attempt) as stream:
        assert stream.run_key == run
        assert stream.attempt_key == attempt
        coverage = msgspec.to_builtins(stream.coverage)
        events = list(stream)
    assert coverage["kind"] == "sampled"
    assert coverage["observed"] == coverage["queued"] + coverage["dropped_before_queue"]
    assert coverage["acknowledged"] == len(events)
    assert coverage["unacknowledged"] == 0
    assert coverage["writer"] == {"kind": "complete"}
    assert events, "the canonical Ipopt attempt retains iteration observations"
    positions = [event.sequence for event in events]
    sequences = [sequence for sequence in positions if sequence is not None]
    assert sequences == positions
    assert sequences == sorted(sequences)
    assert len(set(sequences)) == len(sequences)
    assert all(event.step == 0 and event.phase for event in events)
    assert [
        (event.sequence, event.phase) for event in reopened.progress(run, attempt)
    ] == [(event.sequence, event.phase) for event in events]
    closed = reopened.progress(run, attempt)
    closed.close()
    assert list(closed) == []
    assert msgspec.to_builtins(closed.coverage) == coverage

    destination = tmp_path / "results.arrow"
    reopened.export_results(run, attempt, relation, destination)
    with pa.ipc.open_stream(destination) as reader:
        assert reader.schema.metadata[b"pse.canonical.attempt"].decode() == attempt
        assert reader.read_all().num_rows == exact.num_rows
    original = destination.read_bytes()
    with pytest.raises(pse.InspectionError):
        reopened.export_results(run, attempt, relation, destination)
    assert destination.read_bytes() == original

    # Retirement stays pending while the immutable creation grant is live.
    # The fixture's server shares this host's Unix clock; the header records micros.
    creation_expires_at = dependency_occurrence["creation_expires_at"]
    assert isinstance(creation_expires_at, int)
    retired = reopened.forget_analysis_results(dependencies.key)
    assert isinstance(retired, bool)
    if time.time_ns() // 1_000 < creation_expires_at:
        assert retired is False
    while (remaining := creation_expires_at - time.time_ns() // 1_000) > 0:
        time.sleep(min(0.25, remaining / 1_000_000))
    # Expiry only permits settlement; actual bounded cleanup must return True.
    for _ in range(32):
        if retired:
            break
        retired = reopened.forget_analysis_results(dependencies.key)
        assert isinstance(retired, bool)
        if retired:
            break
    assert retired, (
        "analysis retirement did not finish within the bounded page allowance"
    )
    with pytest.raises(pse.InspectionError):
        reopened.analysis(dependencies.key)
    with pytest.raises(pse.InspectionError):
        pa.table(reopened_dependencies.edges())
    with pytest.raises(pse.InspectionError):
        pa.table(dependencies.header())
    recreated_dependencies = prepared.dependency_analysis(controls)
    assert_same_dependency_content(recreated_dependencies)
    assert recreated_dependencies.key != repeated_dependencies.key
    assert pa.table(reopened.analysis(recreated_dependencies.key).header()).equals(
        pa.table(recreated_dependencies.header())
    )
    # Recreating identical content never revives a retired occurrence's reader.
    with pytest.raises(pse.InspectionError):
        pa.table(reopened_dependencies.edges())


@pytest.mark.integration
@pytest.mark.producer_deployment
def test_canonical_eligible_deployment_receipt_reopens_original_scalar(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    receipt_path = os.environ.get("PSE_PRODUCER_RECEIPT")
    if receipt_path is None:
        pytest.fail(
            "This control requires the reviewed current pse-py deployment receipt."
        )
    receipt = msgspec.json.decode(
        Path(receipt_path).read_bytes(), type=dict[str, object]
    )
    assert receipt["frame"] == "pse.producer.v1"
    assert receipt["package"] == "pse-py"
    assert receipt["persistent_reuse_eligible"] is True
    assert receipt["reasons"] == []
    assert receipt["units"]
    assert receipt["receipt_version"] == 2
    association = receipt["deployment"]
    assert isinstance(association, dict)
    artifact = association["artifact"]
    assert isinstance(artifact, dict)
    module_file = sys.modules["pse._native"].__file__
    assert isinstance(module_file, str)
    loaded = Path(module_file).resolve()
    assert loaded == Path(str(artifact["canonical"]))
    assert hashlib.sha256(loaded.read_bytes()).hexdigest() == artifact["sha256"]

    # Refuse the actual other role and contradictory stale byte claims at the
    # loaded native boundary, independently of scientific receipt identity.
    worker = os.environ.get("PSE_WORKER_PRODUCER_RECEIPT")
    if worker is None:
        pytest.fail("This control requires the actual worker deployment receipt.")
    with pytest.raises(pse.InspectionError, match="actual deployed artifact"):
        canonical_substrate.runtime(inspection_settings, producer=worker)
    stale = msgspec.json.decode(Path(receipt_path).read_bytes(), type=dict[str, object])
    stale_association = stale["deployment"]
    assert isinstance(stale_association, dict)
    stale_artifact = stale_association["artifact"]
    assert isinstance(stale_artifact, dict)
    stale_artifact["sha256"] = "0" * 64
    stale_path = tmp_path / "stale-artifact.json"
    stale_path.write_bytes(msgspec.json.encode(stale))
    with pytest.raises(pse.InspectionError, match="consumed file changed"):
        canonical_substrate.runtime(inspection_settings, producer=str(stale_path))

    runtime = canonical_substrate.runtime(inspection_settings, producer=receipt_path)
    package, case = _package(runtime)
    revision = package.canonical_revision
    assert revision is not None
    settings = _settings()
    prepared = package.prepare_solve(case, settings)
    first = prepared.start().wait()
    assert first.usable
    run, attempt = first.canonical_run_key, first.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    original = pa.table(first.table("runtime.solve_variables"))
    variables = original.to_pylist()
    (variable,) = [row for row in variables if not row["parameter"]]
    (binding,) = [row for row in variables if row["parameter"]]
    assert binding["fixed"] is True
    assert binding["value"] == 4.0
    # Use the actual production-resolved physical allowance for this variable.
    # Exact equality below checks retention of original result bytes.
    allowance = variable["tolerance"]
    assert allowance > 0.0
    assert abs(variable["value"] - 2.0) <= allowance
    checks = pa.table(first.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(row["satisfied"] for row in checks)
    (header,) = pa.table(runtime.run_record(run)).to_pylist()
    # These identities come from the actual loaded extension's deployment, not
    # from feeding a receipt's own identities back into its qualification mint.
    expected_attestation = msgspec.json.decode(
        header["attestation"], type=tuple[str, str]
    )
    assert len(expected_attestation) == 2
    assert all(value.startswith("blake3:") for value in expected_attestation)
    if observed_path := os.environ.get("PSE_PYTHON_DEPLOYMENT_ATTESTATION"):
        # The native cross-role control consumes this independently observed
        # loaded-extension header, rather than a receipt's self-reported pair.
        observed = Path(observed_path)
        if observed.exists():
            assert observed.read_bytes() == header["attestation"]
        else:
            observed.write_bytes(header["attestation"])
    assert header["revision"] == revision
    del first, prepared, package
    runtime.clear_program_cache()
    del runtime

    reopened = canonical_substrate.runtime(inspection_settings, producer=receipt_path)
    recreated = reopened.modeling_revision(revision, _physical(reopened))
    assert recreated.canonical_revision == revision
    assert pa.table(reopened.results(run, attempt, "runtime.solve_variables")).equals(
        original
    )
    following = recreated.prepare_solve(case, _settings()).start().wait()
    assert following.usable
    repeated = pa.table(following.table("runtime.solve_variables"))
    assert repeated.drop(["run_id"]).equals(original.drop(["run_id"]))
    repeated_checks = pa.table(following.table("runtime.modeling_checks")).to_pylist()
    assert repeated_checks
    assert all(row["satisfied"] for row in repeated_checks)
    assert following.canonical_run_key is not None
    (following_header,) = pa.table(
        reopened.run_record(following.canonical_run_key)
    ).to_pylist()
    assert following_header["revision"] == revision
    assert (
        msgspec.json.decode(following_header["attestation"], type=tuple[str, str])
        == expected_attestation
    )
    # Public Python observes actual deployment admission, immutable reopening and
    # unchanged outputs. Strict native recipe controls separately distinguish
    # persisted reconstruction from fresh semantic admission; no timing claim.


@pytest.mark.integration
def test_canonical_default_no_receipt_reopens_scalar_with_sibling_loader(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
    request: pytest.FixtureRequest,
) -> None:
    # Bound the complete native preparation, not just Thread.join: a TLS/loader
    # lock inversion on the receiving thread must not hang the enclosing suite.
    child_marker = "PSE_TEST_DEFAULT_REPLAY_LOADER_CHILD"
    if os.environ.get(child_marker) != "1":
        # The child is this one control, not the enclosing campaign collector.
        # Preserve artifact checks without overwriting the parent's selection.
        child_environment = {
            key: value
            for key, value in os.environ.items()
            if key != "PSE_TEST_ENUMERATION"
        }
        child_environment[child_marker] = "1"
        # xdist loadgroup makes the group suffix part of the frozen identity.
        # Reproduce this actual parent's identity for the same one-test subset.
        collection = (
            ["-n", "1", "--dist", "loadgroup"]
            if request.node.nodeid.endswith("@scientific-owner")
            else ["-n", "0"]
        )
        command = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                f"{Path(__file__).resolve()}::{test_canonical_default_no_receipt_reopens_scalar_with_sibling_loader.__name__}",
                *collection,
                "--basetemp",
                str(tmp_path / "loader-child"),
                "-q",
                "-s",
            ],
            env=child_environment,
            check=False,
            capture_output=True,
            text=True,
            timeout=240,
        )
        assert command.returncode == 0, command.stdout + command.stderr
        return

    # Python can already retain system DSOs such as libutil. A private C fixture
    # with TLS guarantees genuine namespace/TLS publication and balanced removal.
    source = tmp_path / "native-loader-control.c"
    library = tmp_path / "native-loader-control.so"
    source.write_text(
        "_Thread_local int pse_loader_control_tls;\n"
        "int pse_loader_control(void) { return ++pse_loader_control_tls; }\n"
    )
    compiler = shlex.split(os.environ.get("CC", "cc"))
    assert compiler, "the native environment must supply a C compiler"
    compiled = subprocess.run(
        [*compiler, "-std=c11", "-shared", "-fPIC", str(source), "-o", str(library)],
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert compiled.returncode == 0, compiled.stdout + compiled.stderr
    library_path = str(library.resolve())
    library_bytes = os.fsencode(library_path)

    # ctypes/threading are imported before either receiving Runtime observes its
    # real imported-extension/native context. CDLL releases the GIL during calls.
    loader = ctypes.CDLL(None)
    dlopen = loader.dlopen
    dlopen.argtypes = [ctypes.c_char_p, ctypes.c_int]
    dlopen.restype = ctypes.c_void_p
    dlclose = loader.dlclose
    dlclose.argtypes = [ctypes.c_void_p]
    dlclose.restype = ctypes.c_int

    def loader_cycle() -> None:
        mappings = Path("/proc/self/maps")
        assert library_path not in mappings.read_text(), (
            "control requires a new namespace publication, not a retained handle"
        )
        handle = dlopen(library_bytes, os.RTLD_NOW | os.RTLD_LOCAL)
        assert handle, "the private native loader control must load"
        try:
            assert library_path in mappings.read_text()
        finally:
            assert dlclose(handle) == 0
        assert library_path not in mappings.read_text(), (
            "balanced dlclose must remove the control's actual mapped artifact"
        )

    loader_cycle()
    # No producer argument: ordinary imported Python chooses its supported local
    # admission independently, even if an optional strict campaign is configured.
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case = _package(runtime)
    revision = package.canonical_revision
    assert revision is not None
    prepared = package.prepare_solve(case, _settings())
    first = prepared.start().wait()
    assert first.usable
    run, attempt = first.canonical_run_key, first.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    original = pa.table(first.table("runtime.solve_variables"))
    (variable,) = [row for row in original.to_pylist() if not row["parameter"]]
    (binding,) = [row for row in original.to_pylist() if row["parameter"]]
    assert binding["fixed"] is True
    assert binding["value"] == 4.0
    allowance = variable["tolerance"]
    assert allowance > 0.0
    assert abs(variable["value"] - 2.0) <= allowance
    original_checks = pa.table(first.table("runtime.modeling_checks"))
    assert original_checks.num_rows > 0
    assert all(row["satisfied"] for row in original_checks.to_pylist())
    del first, prepared, package
    runtime.clear_program_cache()
    del runtime

    receiving = canonical_substrate.runtime(inspection_settings)
    recreated = receiving.modeling_revision(revision, _physical(receiving))
    assert recreated.canonical_revision == revision
    started = threading.Event()
    stop = threading.Event()
    failures: list[Exception] = []
    completed_cycles = 0

    def sibling_loader() -> None:
        nonlocal completed_cycles
        try:
            while not stop.is_set():
                loader_cycle()
                completed_cycles += 1
                started.set()
                stop.wait(0.001)
        except (AssertionError, OSError) as error:
            failures.append(error)
            started.set()

    sibling = threading.Thread(target=sibling_loader, daemon=True)
    sibling.start()
    try:
        assert started.wait(5), "sibling loader did not begin its actual glibc work"
        assert not failures, failures
        # A changing loader context may conservatively choose fresh preparation.
        # Either path must construct owned mathematical values without deadlock.
        concurrent = recreated.prepare_solve(case, _settings())
    finally:
        stop.set()
        sibling.join(timeout=10)
    assert not sibling.is_alive(), "sibling loader did not drain after preparation"
    assert not failures, failures
    assert completed_cycles > 0
    loader_cycle()  # Loader remains usable after bounded native preparation.
    following = concurrent.start().wait()
    assert following.usable
    (following_variable,) = [
        row
        for row in pa.table(following.table("runtime.solve_variables")).to_pylist()
        if not row["parameter"]
    ]
    assert abs(following_variable["value"] - 2.0) <= allowance
    del following, concurrent, recreated
    receiving.clear_program_cache()
    del receiving

    # Quiet reopening after loader activity verifies the ordinary composed path
    # and exact retained historical members; it makes no timing/replay-only claim.
    reopened = canonical_substrate.runtime(inspection_settings)
    assert pa.table(reopened.results(run, attempt, "runtime.solve_variables")).equals(
        original
    )
    assert pa.table(reopened.results(run, attempt, "runtime.modeling_checks")).equals(
        original_checks
    )
    quiet = reopened.modeling_revision(revision, _physical(reopened))
    assert quiet.canonical_revision == revision
    result = quiet.prepare_solve(case, _settings()).start().wait()
    assert result.usable
    (quiet_variable,) = [
        row
        for row in pa.table(result.table("runtime.solve_variables")).to_pylist()
        if not row["parameter"]
    ]
    assert abs(quiet_variable["value"] - 2.0) <= allowance
    quiet_checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert quiet_checks
    assert all(row["satisfied"] for row in quiet_checks)
    # RefuseFreshAdmission in the native persisted replay control separately proves
    # reconstruction versus fresh admission. This is imported-Python composition,
    # dynamic TLS/loader concurrency, scientific output and historical retention.


def _queued_server_bytes(pid: int) -> int:
    # The caller holds the selected process by pidfd for a disposable fixture's
    # sole server. Observe receipt on its actual established sockets, without
    # reading or consuming another process's network data.
    inodes: set[str] = set()
    for descriptor in Path(f"/proc/{pid}/fd").iterdir():
        try:
            target = str(descriptor.readlink())
        except FileNotFoundError:
            continue
        if target.startswith("socket:[") and target.endswith("]"):
            inodes.add(target[8:-1])
    queued = 0
    for family in ("tcp", "tcp6"):
        for line in Path(f"/proc/{pid}/net/{family}").read_text().splitlines()[1:]:
            fields = line.split()
            if fields[3] == "01" and fields[9] in inodes:
                queued += int(fields[4].split(":")[1], 16)
    return queued


def _assert_owned_delayed_canonical_sigint(
    settings: pse.EngineSettings,
    *,
    state: Path,
    database: str,
    run: str,
    attempt: str,
    owned_instance: str,
    relation: str,
    file_instances: tuple[str, str],
) -> dict[str, object]:
    """Explicit private-server control; never selected by the normal test suite.

    The caller owns this exact disposable supervisor instance exclusively. The
    public pool's named reservation proves entry into the canonical pull; receipt
    on the stopped server's actual socket proves dispatch before cancellation.
    """
    features = compiled_features()
    assert "force-validate" in features, "install the actual correctness extension"
    state = state.resolve(strict=True)
    config = server.config_for(state)
    assert config["instance_id"] == owned_instance
    assert server.service_directory(state) == state, "select the actual service owner"
    assert server._owns_listener(state, config), (
        "exact physical listener owner required"
    )
    process = server.read_json(state / "server-process.json")
    assert process["instance_id"] == owned_instance
    pid = server.integer(process["pid"])
    pidfd = os.pidfd_open(pid)
    assert len(file_instances) == 2
    assert file_instances[0] != file_instances[1]
    runtime: pse.Runtime | None = None
    cleanup: pse.Runtime | None = None
    stream: pse.TableStream | None = None
    reservation_baselines: dict[str, int] = {}

    def reserved(name: str) -> int:
        assert runtime is not None
        return sum(
            consumer.reserved_bytes
            for consumer in runtime.resource_usage().top_consumers
            if consumer.name == name
        )

    def delayed_call(
        work: Callable[[], object],
        consumer: str,
        *,
        prompt_join: bool,
        after_interrupt: Callable[[], None] | None = None,
    ) -> float:
        before = reserved(consumer)
        reservation_baselines[consumer] = before
        ready = threading.Event()
        observed_signal = threading.Event()
        returned = threading.Event()
        discarded = threading.Event()
        failures: list[BaseException] = []
        times: dict[str, float] = {}

        original_cause = RuntimeError("original signal handler cause")
        original_error = KeyboardInterrupt("original entered signal")
        original_error.__cause__ = original_cause

        def interrupted(_number: int, _frame: FrameType | None) -> None:
            observed_signal.set()
            raise original_error

        original_handler = signal.signal(signal.SIGINT, interrupted)

        def controller() -> None:
            stopped = False
            try:
                current = server.config_for(state)
                assert current["instance_id"] == owned_instance
                assert server.read_json(state / "server-process.json") == process
                assert server._owns_listener(state, current)
                signal.pidfd_send_signal(pidfd, signal.SIGSTOP)
                stopped = True
                stop_deadline = time.monotonic() + 5
                while True:
                    status = (
                        Path(f"/proc/{pid}/stat")
                        .read_text()
                        .rsplit(")", 1)[1]
                        .split()[0]
                    )
                    if status in {"T", "t"}:
                        break
                    assert time.monotonic() < stop_deadline, "owned server did not stop"
                    time.sleep(0.005)
                queued_before = _queued_server_bytes(pid)
                ready.set()
                entry_deadline = time.monotonic() + 5
                # The endpoint owns its finite response extent. Observe that actual
                # grant with dispatch receipt instead of copying a historical budget.
                while True:
                    entered_reserved = reserved(consumer)
                    if (
                        entered_reserved > before
                        and _queued_server_bytes(pid) > queued_before
                    ):
                        break
                    assert not returned.is_set(), (
                        "call returned before a real canonical RPC entry"
                    )
                    assert time.monotonic() < entry_deadline, (
                        "no actual allocation and stopped-server socket receipt"
                    )
                    time.sleep(0.005)
                times["entered"] = time.monotonic()
                os.kill(os.getpid(), signal.SIGINT)
                assert observed_signal.wait(2), (
                    "Python did not observe SIGINT during the entered call"
                )
                if prompt_join:
                    assert returned.wait(2), (
                        "entered canonical pull did not cancel and join while delayed"
                    )
                    assert discarded.is_set(), (
                        "caller-owned reader was not discarded before resume"
                    )
                    assert reserved(consumer) >= entered_reserved, (
                        "late dispatched response lost its actual grant "
                        "before the server resumed"
                    )
                else:
                    # The ordinary header bridge has no cancellation callback;
                    # signal observation is prompt, while settlement still joins
                    # its original request after this exact listener resumes.
                    assert not returned.wait(0.2), (
                        "ordinary delayed header unexpectedly settled"
                    )
            except BaseException as error:
                failures.append(error)
            finally:
                ready.set()
                if stopped:
                    signal.pidfd_send_signal(pidfd, signal.SIGCONT)

        peer = threading.Thread(target=controller, name="canonical-signal-owner")
        peer.start()
        interruption: KeyboardInterrupt | None = None
        try:
            assert ready.wait(6), "owned delay controller did not establish its state"
            if failures:
                raise failures[0]
            try:
                work()
            except KeyboardInterrupt as error:
                interruption = error
            if interruption is not None:
                assert interruption is original_error
                assert interruption.__cause__ is original_cause
                # Traceback frames can themselves retain the Python stream wrapper.
                # Dispose those observer aliases before asserting caller release.
                original_error.__traceback__ = None
                if after_interrupt is not None:
                    after_interrupt()
                    discarded.set()
        finally:
            times["returned"] = time.monotonic()
            returned.set()
            peer.join(timeout=10)
            signal.signal(signal.SIGINT, original_handler)
        assert not peer.is_alive(), "owned delay controller did not drain"
        if failures:
            raise failures[0]
        assert interruption is not None, "entered canonical call ignored SIGINT"
        assert observed_signal.is_set()
        return times["returned"] - times["entered"]

    try:
        runtime = pse.Runtime(settings, substrate=str(state), database=database)
        cleanup = pse.Runtime(settings, substrate=str(state), database=database)
        stream = runtime.results(run, attempt, relation)
        schema = pa.schema(stream)
        # A signal before consumption is a separate control, never evidence that
        # the native producer entered or cancelled an in-flight operation.
        with pytest.raises(KeyboardInterrupt):
            signal.raise_signal(signal.SIGINT)
        first = stream.next_batch()
        assert first is not None
        assert first.num_rows > 0
        assert first.schema == schema
        expected = first.to_pylist()
        with pytest.raises(pse.InspectionError, match="already consumed"):
            stream.__arrow_c_stream__()
        ordinary_seconds = delayed_call(
            lambda: runtime.attempt_record(attempt),
            "canonical:result-header",
            prompt_join=False,
        )

        def discard_interrupted_reader() -> None:
            nonlocal stream
            assert stream is not None
            assert not any(
                task.joinpath("comm").read_text().strip() == "pse-python-pull"
                for task in Path("/proc/self/task").iterdir()
                if task.joinpath("comm").exists()
            ), "the original Python-owned native pull must already be joined"
            failure = stream.failure
            assert failure is not None
            assert failure.code == DiagnosticCode.RUNTIME_CANCELLED
            with pytest.raises(pse.InspectionError) as repeated:
                stream.next_batch()
            assert repeated.value.report.code == failure.code
            stream.close()
            stream = None  # Release every caller-owned reader while server is stopped.
            assert first.to_pylist() == expected
            assert runtime is not None
            runtime.clear_program_cache()
            runtime.close()

        def next_file() -> object:
            assert stream is not None
            return stream.next_batch()

        pull_seconds = delayed_call(
            next_file,
            "canonical:result-file-open-response",
            prompt_join=True,
            after_interrupt=discard_interrupted_reader,
        )
        drain_deadline = time.monotonic() + 5
        while any(
            reserved(consumer) > before
            for consumer, before in reservation_baselines.items()
        ):
            assert time.monotonic() < drain_deadline, (
                "original metadata response did not drain"
            )
            time.sleep(0.005)
        # Reclamation supplies an actual canonical release/collection control
        # after the original late frame, rather than treating worker join as drain.
        collection_deadline = time.monotonic() + 10
        while True:
            try:
                page = cleanup.reclaim_run_results(run)
            except pse.InspectionError as error:
                if "run results have protected readers" not in str(error):
                    raise
                assert time.monotonic() < collection_deadline, (
                    "original ticket did not release"
                )
                time.sleep(0.01)
                continue
            if page.complete:
                break
            assert time.monotonic() < collection_deadline, (
                "result collection did not complete"
            )
        assert first.schema == schema
        assert first.to_pylist() == expected
        return {
            "event": "complete",
            "entry": (
                "public canonical:result-file-open-response reservation "
                "and stopped-server socket receipt"
            ),
            "controlled_pull_joined_while_delayed": True,
            "response_grant_retained_until_server_resume": True,
            "response_grant_released_after_joined_pull_and_reader_release": True,
            "reader_discarded_while_server_stopped": True,
            "original_exception_and_cause_preserved": True,
            "exact_ticket_released_and_results_collected": True,
            "first_page_file_instances": file_instances,
            "ordinary_signal_observed_while_delayed": True,
            "ordinary_join_seconds": ordinary_seconds,
            "controlled_join_seconds": pull_seconds,
            "retained_rows_after_close": first.num_rows,
            "features": features,
            "owned_instance": owned_instance,
        }
    finally:
        # The pidfd denotes the original process even if its numeric PID were
        # replaced. Always resume it before any protection/transport drain.
        try:
            signal.pidfd_send_signal(pidfd, signal.SIGCONT)
        finally:
            os.close(pidfd)
        if stream is not None:
            stream.close()
        if runtime is not None:
            runtime.clear_program_cache()
            runtime.close()
        if cleanup is not None:
            cleanup.clear_program_cache()
            cleanup.close()


@pytest.mark.integration
@pytest.mark.parametrize("active_bound", [False, True])
def test_result_analysis_root_scope_and_withheld_neighborhood(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    active_bound: bool,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    source = (
        SOURCE.replace(
            "annotation bounds x(0,10)", "annotation bounds x(2,10)"
        ).replace("annotation start x(1)", "annotation start x(2)")
        if active_bound
        else SOURCE
    )
    package, case = _package(runtime, source)
    settings = msgspec.structs.replace(
        _settings(), backend=NativeBackend.IPOPT, intent=NativeSolveIntent.ROOT
    )
    parameter = next(
        member.id
        for member in package.inspect(inspection_request(case, settings)).members
        if member.lineage.path == "a" or member.lineage.path.endswith(".a")
    )
    settings = msgspec.structs.replace(
        settings, sensitivity=SensitivityRequest(parameters=(parameter,))
    )
    result = package.prepare_solve(case, settings).start().wait()
    assert result.usable
    assert result.canonical_run_key is not None
    assert result.canonical_attempt_key is not None
    graph = runtime.result_analysis(
        result.canonical_run_key,
        result.canonical_attempt_key,
        pse.AnalysisControls(roots=(), direction=AnalysisDirection.DOWNSTREAM),
    )
    edges = pa.table(graph.edges()).to_pylist()
    kinds = {row["kind"] for row in edges}
    assert not any("optimal_parametric" in kind for kind in kinds)
    artifact_key = next(
        row["evidence_artifact"]
        for row in edges
        if row["evidence_artifact"] is not None
    )
    artifact = runtime.scientific_artifact(artifact_key)
    assert artifact.kind == "local-analysis"
    blocks = pa.table(artifact.metadata()).to_pylist()[0]["block_count"]
    validity = []
    routes = []
    for ordinal in range(blocks):
        table = pa.table(artifact.block(ordinal))
        if "validity" in table.column_names:
            validity.extend(table.to_pylist())
        if "request_identity" in table.column_names and "intent" in table.column_names:
            routes.extend(table.to_pylist())
    assert len(validity) == 1
    assert routes
    assert all(row["intent"] == "root" for row in routes)
    original = validity[0]["validity"]
    assert original["licq"] is None
    assert original["second_order"] is None
    if active_bound:
        assert not original["certified"]
        assert original["reason"] == "neighborhood_unavailable"
        assert "retained_root_parametric_primal" not in kinds
    else:
        assert original["certified"]
        assert original["root_rank"] == 1
        assert original["root_neighborhood"]
        assert "retained_root_parametric_primal" in kinds

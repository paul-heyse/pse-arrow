# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact canonical identities, bounded result streams and local IPC export."""

import hashlib
import os
import sys
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse.contracts.enums import (
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.contracts.identities import DeclarationId

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


def _package(runtime: pse.Runtime) -> tuple[pse.ModelingPackage, DeclarationId]:
    root = Path(__file__).resolve().parents[3]
    physical = _physical(runtime)
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": SOURCE}], physical
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


@pytest.mark.integration
def test_canonical_result_selection_and_progress_reopen(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: str,
    tmp_path: Path,
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    assert runtime.durable
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    controls = pse.AnalysisControls(roots=(), direction="downstream")
    dependencies = prepared.dependency_analysis(controls)
    dependency_rows = pa.table(dependencies.edges()).to_pylist()
    assert any(row["kind"] == "numerical_incidence" for row in dependency_rows)
    assert any(row["kind"] == "execution_dependency" for row in dependency_rows)
    assert all(row["evidence"] is None for row in dependency_rows)
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
    reopened = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    (header,) = pa.table(reopened.run_record(run)).to_pylist()
    (record,) = pa.table(reopened.attempt_record(attempt)).to_pylist()
    (manifest,) = pa.table(reopened.result_manifest(attempt)).to_pylist()
    assert header["key"] == run
    assert record["key"] == attempt
    assert record["run"] == run
    assert record["terminal"]
    assert record["outcome"] == "succeeded"
    assert manifest["attempt"] == attempt
    reopened_dependencies = reopened.analysis(dependencies.key)
    assert pa.table(reopened_dependencies.edges()).to_pylist() == dependency_rows
    provenance = reopened.result_analysis(run, attempt, controls)
    (analysis,) = pa.table(provenance.header()).to_pylist()
    assert analysis["revision"] == header["revision"]
    assert analysis["method"] == "result-sensitivity-provenance-reachability:v1"
    assert any(
        row["kind"] == "input_provenance"
        for row in pa.table(provenance.edges()).to_pylist()
    )
    relation = "runtime.solve_runs"
    exact = pa.table(reopened.results(run, attempt, relation))
    latest = pa.table(
        reopened.latest_results(header["problem"], relation, classes=("succeeded",))
    )
    assert exact.equals(latest)
    assert exact.num_rows == 1
    assert (
        pa.table(reopened.results(run, attempt, relation, start=1, end=1)).num_rows == 0
    )

    with reopened.progress(run, attempt) as stream:
        assert stream.run_key == run
        assert stream.attempt_key == attempt
        events = list(stream)
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

    destination = tmp_path / "results.arrow"
    reopened.export_results(run, attempt, relation, destination)
    with pa.ipc.open_stream(destination) as reader:
        assert reader.schema.metadata[b"pse.canonical.attempt"].decode() == attempt
        assert reader.read_all().num_rows == exact.num_rows
    original = destination.read_bytes()
    with pytest.raises(pse.InspectionError):
        reopened.export_results(run, attempt, relation, destination)
    assert destination.read_bytes() == original


@pytest.mark.integration
def test_canonical_eligible_deployment_receipt_reopens_original_scalar(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: str,
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
        pse.Runtime(inspection_settings, substrate=canonical_substrate, producer=worker)
    stale = msgspec.json.decode(Path(receipt_path).read_bytes(), type=dict[str, object])
    stale_association = stale["deployment"]
    assert isinstance(stale_association, dict)
    stale_artifact = stale_association["artifact"]
    assert isinstance(stale_artifact, dict)
    stale_artifact["sha256"] = "0" * 64
    stale_path = tmp_path / "stale-artifact.json"
    stale_path.write_bytes(msgspec.json.encode(stale))
    with pytest.raises(pse.InspectionError, match="consumed file changed"):
        pse.Runtime(
            inspection_settings, substrate=canonical_substrate, producer=str(stale_path)
        )

    runtime = pse.Runtime(
        inspection_settings, substrate=canonical_substrate, producer=receipt_path
    )
    package, case = _package(runtime)
    revision = package.canonical_revision
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

    reopened = pse.Runtime(
        inspection_settings, substrate=canonical_substrate, producer=receipt_path
    )
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

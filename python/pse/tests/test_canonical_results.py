# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact canonical identities, bounded result streams and local IPC export."""

import ctypes
import hashlib
import os
import shlex
import subprocess
import sys
import threading
import time
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
from pse.tests.canonical_fixture import CanonicalFixture

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
def test_canonical_close_releases_optional_cache_after_completed_run(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
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
    assert record["terminal"]
    assert record["outcome"] == "succeeded"
    del variables, result, handle, prepared, package
    (physical_cache,) = [
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.physical_admission"
    ]
    assert physical_cache.entries > 0
    # The public drain releases its own optional retention without a manual clear.
    runtime.close()


@pytest.mark.integration
def test_canonical_close_refuses_live_result_reader_then_allows_departure(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
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
    (physical_cache,) = [
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.physical_admission"
    ]
    assert physical_cache.entries > 0
    with pytest.raises(pse.InspectionError, match="live protected readers"):
        runtime.close()
    (physical_cache,) = [
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.physical_admission"
    ]
    assert physical_cache.entries == 0
    # The unread selection still owns its protection and fetches an actual
    # canonical block after refusal, before requesting EOF can release it.
    batch = reader.read_next_batch()
    assert batch.to_pylist() == expected
    with pytest.raises(pse.InspectionError, match="live protected readers"):
        runtime.close()
    del batch, reader, stream
    runtime.close()


@pytest.mark.integration
def test_canonical_result_selection_and_progress_reopen(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
    tmp_path: Path,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    assert runtime.durable
    package, case = _package(runtime)
    prepared = package.prepare_solve(case, _settings())
    controls = pse.AnalysisControls(roots=(), direction="downstream")
    dependencies = prepared.dependency_analysis(controls)
    dependency_rows = pa.table(dependencies.edges()).to_pylist()
    assert any(row["kind"] == "numerical_incidence" for row in dependency_rows)
    assert any(row["kind"] == "execution_dependency" for row in dependency_rows)
    assert all(row["evidence"] is None for row in dependency_rows)
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
                roots=("missing-scientific-object",), direction="downstream"
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
            if request.node.nodeid.endswith("@canonical-owner")
            else ["-n", "0"]
        )
        command = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                f"{Path(__file__).resolve()}::{test_canonical_default_no_receipt_reopens_scalar_with_sibling_loader.__name__}",
                *collection,
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

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authored algebraic jobs retain native ownership through checks and retention."""

import asyncio
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.contracts.documents import (
    BindingAssignment,
    BindingQuantity,
    BindingTargetMember,
    BindingTargetPath,
    PointOverlay,
)
from pse.contracts.enums import (
    NativeBackend,
    NativeSolveIntent,
    NativeStartPolicy,
    PresolvePolicyKind,
    ReusePolicy,
)
from pse.contracts.values import SemanticId
from pse.tests.canonical_fixture import CanonicalFixture

#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)


@pytest.mark.integration
def test_authored_solve_join_warm_start_checks_and_retention(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: CanonicalFixture,
) -> None:
    # Ordinary runs retain exact canonical attempt selections.
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents(
        {
            str(p.relative_to(primitives)): p.read_text()
            for p in primitives.rglob("*")
            if p.is_file()
        }
    )
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
    source = """package algebraic { def Root {
        var x:Scalar;
        eq square:x*x==4;
        annotation start x(1);
        annotation bounds x(0,10);
        annotation report x("root");
        annotation check x(x>1);
    } }"""
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": source}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.FEASIBLE_POINT,
        presolve=PresolvePolicyKind.OFF,
    )
    prepared = package.prepare_solve(case, settings)
    handle = prepared.start()

    async def wait_twice() -> pse.RunResult:
        a, b = await asyncio.gather(handle.wait_async(), handle.wait_async())
        assert a.run_id == b.run_id
        return a

    result = asyncio.run(wait_twice())
    assert result.run_id == handle.wait().run_id
    assert result.usable
    assert result.completion.solves[0].backend == "ipopt"
    rows = pa.table(result.table("runtime.solve_variables")).to_pylist()
    assert len(rows) == 1
    # Empirical scalar reference agreement at the shared production resolution.
    assert rows[0]["value"] == pytest.approx(
        2, rel=settings.numerics.engineering_relative_fraction, abs=0.0
    )
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(row["satisfied"] for row in checks)
    seed = result.available_start()
    assert seed is not None
    warmed = prepared.with_start(seed).start().wait()
    assert warmed.usable
    assert warmed.run_id != result.run_id
    primal = {SemanticId(rows[0]["symbol_id"]): 1.5}
    assert prepared.with_primal_start(primal).start().wait().usable
    # Source inventory is explicitly read from the immutable package revision.
    sources = package.source_tables()
    packages = pa.table(
        sources[SemanticId.from_hex("0836e02bdfe70fb065cdea4dc123f230")]
    ).to_pylist()
    assert any(
        dependency["version_req"]["operator"] == "exact"
        for row in packages
        for dependency in row["dependencies"]
    ), packages
    documents = pa.table(
        sources[SemanticId.from_hex("a98911fe15eafd4f6520725e5b50f7d1")]
    ).to_pylist()
    assert any(row["source_text"] == source for row in documents), documents
    for stream in sources.values():
        stream.close()
    run = result.canonical_run_key
    attempt = result.canonical_attempt_key
    assert run is not None
    assert attempt is not None
    assert attempt == handle.canonical_attempt_key
    (stored,) = pa.table(runtime.attempt_record(attempt)).to_pylist()
    assert stored["run"] == run
    assert stored["outcome"] == "succeeded"
    assert (
        pa.table(runtime.results(run, attempt, "runtime.modeling_checks")).to_pylist()
        == checks
    )
    destination = tmp_path / "checks.arrow"
    runtime.export_results(run, attempt, "runtime.modeling_checks", destination)
    assert destination.is_file()
    following = package.prepare_solve(
        case,
        pse.SolveSettings(
            backend=NativeBackend.IPOPT,
            intent=NativeSolveIntent.FEASIBLE_POINT,
            presolve=PresolvePolicyKind.OFF,
            controls=pse.SolveControls(
                reuse=ReusePolicy.REQUIRE_REUSE,
                start=NativeStartPolicy.PREVIOUS_ACCEPTED,
            ),
        ),
    )
    sequence = runtime.start([prepared, following]).wait()
    assert sequence.usable
    assert len(sequence.completion.solves) == 2
    sequence_checks = pa.table(sequence.table("runtime.modeling_checks")).to_pylist()
    assert {row["step"] for row in sequence_checks} == {0, 1}
    assert {row["sample_index"] for row in sequence_checks} == {0}
    assert sequence.available_start(1) is not None
    assert sequence.canonical_run_key is not None
    assert sequence.canonical_attempt_key is not None
    assert (
        pa.table(
            runtime.results(
                sequence.canonical_run_key,
                sequence.canonical_attempt_key,
                "runtime.modeling_checks",
            )
        ).to_pylist()
        == sequence_checks
    )
    sequence_cancelled = runtime.start([prepared, following])
    sequence_cancelled.cancel()
    sequence_partial = sequence_cancelled.wait()
    assert sequence_partial.run_id == sequence_cancelled.wait().run_id
    assert pa.table(sequence_partial.table("runtime.solve_runs")).num_rows == 2
    assert (
        sequence_partial.canonical_attempt_key
        == sequence_cancelled.canonical_attempt_key
    )
    assert sequence_partial.canonical_attempt_key is not None
    (stopped,) = pa.table(
        runtime.attempt_record(sequence_partial.canonical_attempt_key)
    ).to_pylist()
    assert stopped["outcome"] in {"cancelled", "partial", "succeeded"}
    rejected = (
        runtime.modeling_from_documents(
            [
                {
                    "package.toml": manifest,
                    "models/root.pse": source.replace("check x(x>1)", "check x(x<1)"),
                }
            ],
            physical,
        )
        .prepare_solve(case, settings)
        .start()
        .wait()
    )
    assert not rejected.usable
    assert rejected.completion.solves[0].feasible
    assert any(
        not row["satisfied"]
        for row in pa.table(rejected.table("runtime.modeling_checks")).to_pylist()
    )
    # Immediate cancellation must still produce a joined, repeatedly readable attempt.
    cancelled = prepared.start()
    cancelled.cancel()
    partial = cancelled.wait()
    assert partial.run_id == cancelled.wait().run_id
    assert not partial.usable
    assert pa.table(partial.table("runtime.solve_runs")).num_rows == 1


@pytest.mark.integration
def test_typed_bound_solve_preserves_parent_and_returns_exact_admission(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents(
        {
            str(path.relative_to(primitives)): path.read_text()
            for path in primitives.rglob("*")
            if path.is_file()
        }
    )
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", PRIMITIVES)
    )
    source = """package binding {
        test Root fixture {dof 0; fix f=2;} {
            param p:Scalar=3; var f:Scalar; var x:Scalar;
            eq e:x==p*f; annotation start x(1);
            annotation report x("solution"); annotation check x(x==p*f);
        }
    }"""
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/root.pse": source}], physical
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    original_revision = package.canonical_revision
    settings = pse.SolveSettings(
        backend=NativeBackend.KINSOL,
        intent=NativeSolveIntent.ROOT,
        presolve=PresolvePolicyKind.OFF,
    )

    def assignment(path: str, value: float) -> BindingAssignment:
        return BindingAssignment(
            target=BindingTargetPath(value=path),
            value=BindingQuantity(
                magnitude=value,
                quantity="1f" * 16,
                unit="0a" * 16,
            ),
        )

    bound = package.solve_bound(
        case,
        PointOverlay(
            assignments=(
                assignment("p", 4.0),
                assignment("f", 5.0),
                assignment("x", 7.0),
            )
        ),
        settings,
    )
    assert bound.accepted
    receipt = bound.bound_source
    assert receipt is not None
    assert bound.binding_identity is not None
    assert len(receipt.entries) == 3
    admitted = {entry.supplied.value: entry for entry in receipt.entries.values()}
    assert admitted["p"].parameter
    assert admitted["p"].canonical == 4.0
    assert admitted["f"].canonical == 5.0
    assert admitted["x"].canonical == 7.0
    assert not admitted["f"].parameter
    assert not admitted["x"].parameter
    reports = pa.table(bound.table("runtime.modeling_reports")).to_pylist()
    assert next(
        row["value"] for row in reports if row["label"] == "solution"
    ) == pytest.approx(20.0)
    assert package.canonical_revision == original_revision
    reopened = package.solve_case(case, settings)
    reports = pa.table(reopened.table("runtime.modeling_reports")).to_pylist()
    assert next(
        row["value"] for row in reports if row["label"] == "solution"
    ) == pytest.approx(6.0)
    assert reopened.bound_source is None
    assert reopened.binding_identity is None
    same_member = BindingAssignment(
        target=BindingTargetMember(value=admitted["p"].member),
        value=assignment("p", 4.0).value,
    )
    with pytest.raises(pse.InspectionError):
        package.solve_bound(
            case,
            PointOverlay(assignments=(assignment("p", 4.0), same_member)),
            settings,
        )
    with pytest.raises(pse.InspectionError):
        package.solve_bound(
            case, PointOverlay(assignments=(assignment("missing", 1.0),)), settings
        )

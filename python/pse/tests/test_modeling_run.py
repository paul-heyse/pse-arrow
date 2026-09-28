# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authored algebraic jobs retain native ownership through checks and publication."""

import asyncio
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.contracts.enums import (
    NativeBackend,
    NativeSolveIntent,
    NativeStartPolicy,
    PresolvePolicyKind,
    ReusePolicy,
)
from pse.contracts.enums import AttemptState
from pse.contracts.values import SemanticId


@pytest.mark.integration
def test_authored_solve_join_warm_start_checks_and_publication(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
    tmp_path: Path,
) -> None:
    # Only durable runs publish (ADR-0112 Outcome 16); every run is a stored attempt.
    runtime = pse.Runtime(inspection_settings, store=operational_store)
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
    manifest += f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{SemanticId(bytes([31]) * 16).to_hex()}"\n'
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
    assert len(rows) == 1 and rows[0]["value"] == pytest.approx(2, abs=1e-7)
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks and all(row["satisfied"] for row in checks)
    seed = result.available_start()
    assert seed is not None
    warmed = prepared.with_start(seed).start().wait()
    assert warmed.usable and warmed.run_id != result.run_id
    primal = {SemanticId(rows[0]["symbol_id"]): 1.5}
    assert prepared.with_primal_start(primal).start().wait().usable
    aliases = pa.table(result.table("authored.package_quantity_aliases")).to_pylist()
    assert any(row["name"] == "Scalar" for row in aliases)
    documents = pa.table(result.table("authored.documents")).to_pylist()
    assert any(row["source_text"] == source for row in documents), documents
    assert result.attempt_id is not None
    assert result.attempt_id == handle.attempt_id
    (listed,) = runtime.runs(run_id=result.run_id)
    assert listed.attempt_id == result.attempt_id
    assert listed.state == AttemptState.COMPLETED
    workspace = runtime.register_workspace(
        f"modeling-{result.run_id.to_hex()}", tmp_path
    )
    command = result.prepare_publication(workspace)
    ticket = command.ticket
    published = command.commit()
    settled = runtime.settle_publication(ticket)
    assert isinstance(settled, pse.PublicationCommitted)
    assert settled.publication_id == published.publication_id
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
    sequence_command = sequence.prepare_publication(workspace, parent=published.id)
    sequence_published = sequence_command.commit()
    sequence_settled = runtime.settle_publication(sequence_command.ticket)
    assert isinstance(sequence_settled, pse.PublicationCommitted)
    assert sequence_settled.publication_id == sequence_published.publication_id
    assert sequence_published.parent == published.publication_id
    assert runtime.head(workspace.id) == sequence_published.id
    sequence_cancelled = runtime.start([prepared, following])
    sequence_cancelled.cancel()
    sequence_partial = sequence_cancelled.wait()
    assert sequence_partial.run_id == sequence_cancelled.wait().run_id
    assert pa.table(sequence_partial.table("runtime.solve_runs")).num_rows == 2
    (stopped,) = runtime.runs(run_id=sequence_partial.run_id)
    assert stopped.attempt_id == sequence_cancelled.attempt_id
    assert stopped.state in {
        AttemptState.CANCELLED,
        AttemptState.PARTIAL,
        AttemptState.COMPLETED,
    }
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

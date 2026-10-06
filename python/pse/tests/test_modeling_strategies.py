# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public declared topology, causal functions and owned native strategy reports."""

from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts import runtime as result_contracts
from pse.contracts.enums import (
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
    TearMethod,
)

#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)


@pytest.mark.integration
def test_authored_recycle_uses_declared_ports_and_owned_results(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
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
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/recycle.pse": (
                    "package recycle {def Root {"
                    "param a:Scalar=2; var x:Scalar; let output:Scalar=x/2+a; "
                    "port inlet:Scalar=x; annotation connectivity inlet(1,0); "
                    "port outlet:Scalar=output; annotation connectivity outlet(0,1); "
                    "connect outlet -> inlet; annotation start x(1);"
                    "}}"
                ),
            }
        ],
        physical,
    )
    case = next(
        row.declaration_id for row in package.declarations() if row.name == "Root"
    )
    settings = pse.SolveSettings(
        intent=NativeSolveIntent.ROOT,
        backend=NativeBackend.KINSOL,
        presolve=PresolvePolicyKind.OFF,
    )
    inspected = package.inspect(case, settings)
    connections = inspected.connections
    inlet = next(p for p in inspected.ports if p.lineage.path.endswith(".inlet"))
    outlet = next(p for p in inspected.ports if p.lineage.path.endswith(".outlet"))
    selection = codec.decode_json(
        codec.encode_json(
            {
                "nodes": [case.to_hex()],
                "connections": [
                    {
                        "connection": c.id,
                        "group": c.id,
                        "cost": 2.0,
                        "policy": "mandatory",
                    }
                    for c in connections
                ],
            }
        ),
        pse.FlowSelectionDocument,
    )
    flow = package.prepare_flow(case, selection, settings)
    with pytest.raises(pse.InspectionError, match="duplicate selected flow node"):
        package.prepare_flow(
            case,
            pse.FlowSelectionDocument(
                nodes=(case.to_hex(), case.to_hex()), connections=selection.connections
            ),
            settings,
        )
    selected = flow.select_tears(TearMethod.UNWEIGHTED_HEURISTIC, settings).tears()
    assert selected is not None
    assert selected.cost == 2.0
    request = codec.decode_json(
        codec.encode_json(
            {
                "tears": selected.decisions,
                "units": [
                    {
                        "node": case.to_hex(),
                        "realization": {"kind": "explicit_map"},
                        "inputs": [inlet.id],
                        "outputs": [outlet.id],
                    }
                ],
                "anderson": 1,
                "damping": 1.0,
            }
        ),
        pse.RecycleRequest,
    )
    prepared = package.prepare_recycle(case, selection, request, settings)
    assert [route.backend for route in prepared.routes] == ["kinsol"]
    result = prepared.run()
    del package, runtime, flow, prepared
    assert not result.failures()
    events = codec.structure_rows(
        pa.table(result.strategy_events()).to_pylist(),
        result_contracts.RuntimeSolveStrategyEventsRow,
    )
    assert events
    assert result.run_id is not None
    assert {event.run_id for event in events} == {result.run_id}
    assert (
        codec.structure_rows(
            pa.table(result.strategy_events()).to_pylist(),
            result_contracts.RuntimeSolveStrategyEventsRow,
        )
        == events
    )
    (row,) = result.attempts()
    attempt = row.report
    assert attempt is not None
    assert row.failure is None
    assert attempt.qualification == "feasible"
    assert dict(attempt.primal())[inlet.id] == pytest.approx(4.0, abs=1e-6)

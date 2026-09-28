# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public declared topology, causal functions and owned native strategy reports."""

from pathlib import Path
from typing import cast

import pytest

import pse
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind, TearMethod
from pse.contracts.values import SemanticId


@pytest.mark.integration
def test_authored_recycle_uses_declared_ports_and_owned_results(inspection_settings: pse.EngineSettings) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents({str(p.relative_to(primitives)): p.read_text() for p in primitives.rglob("*") if p.is_file()})
    manifest = (root / "tests/fixtures/packages/minimal_explicit/package.toml").read_text().replace('id_policy = "explicit"', 'id_policy = "named"')
    manifest += f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{SemanticId(bytes([31]) * 16).to_hex()}"\n'
    package = runtime.modeling_from_documents([{"package.toml": manifest, "models/recycle.pse": "package recycle {def Root {param a:Scalar=2; var x:Scalar; let output:Scalar=x/2+a; port inlet:Scalar=x; annotation connectivity inlet(1,0); port outlet:Scalar=output; annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(1);}}"}], physical)
    case = next(row.declaration_id for row in package.declarations() if row.name == "Root")
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT, backend=NativeBackend.KINSOL, presolve=PresolvePolicyKind.OFF)
    inspected = package.inspect(case, settings)
    connections = cast("list[dict[str, object]]", inspected["connections"])
    ports = cast("list[dict[str, object]]", inspected["ports"])
    inlet = next(p for p in ports if cast("str", cast("dict[str, object]", p["lineage"])["path"]).endswith(".inlet"))
    outlet = next(p for p in ports if cast("str", cast("dict[str, object]", p["lineage"])["path"]).endswith(".outlet"))
    selection: dict[str, object] = {"nodes": [case.to_hex()], "connections": [{"connection": c["id"], "group": c["id"], "cost": 2.0, "policy": "mandatory"} for c in connections]}
    flow = package.prepare_flow(case, selection, settings)
    with pytest.raises(pse.InspectionError, match="duplicate selected flow node"):
        package.prepare_flow(case, {**selection, "nodes": [case.to_hex(), case.to_hex()]}, settings)
    selected = flow.select_tears(TearMethod.UNWEIGHTED_HEURISTIC, settings).tears()
    assert selected is not None
    assert selected["cost"] == 2.0
    request: dict[str, object] = {"tears": selected["decisions"], "units": [{"node": case.to_hex(), "inputs": [inlet["id"]], "outputs": [outlet["id"]]}], "anderson": 1, "damping": 1.0}
    prepared = package.prepare_recycle(case, selection, request, settings)
    assert [route.backend for route in prepared.routes] == ["kinsol"]
    result = prepared.run()
    del package, runtime, flow, prepared
    assert not result.failures()
    (row,) = result.attempts()
    attempt = row.report
    assert attempt is not None
    assert row.failure is None
    assert attempt.qualification == "feasible"
    assert dict(attempt.primal())[cast("str", inlet["id"])] == pytest.approx(4.0, abs=1e-6)

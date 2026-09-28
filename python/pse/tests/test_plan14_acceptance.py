# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""M22 public native workflow; shared authored fixtures and independent references."""

import asyncio
import gc
import subprocess
import sys
from pathlib import Path
from typing import cast

import pyarrow as pa
import pytest

import pse
from pse.contracts.enums import HessianMode, NativeBackend, NativeSolveIntent, PresolvePolicyKind
from pse import codec
from pse.contracts import authored
from pse.contracts import runtime as runtime_contracts
from pse.contracts.enums import AttemptKind, AttemptState
from pse.contracts.values import SemanticId

@pytest.mark.integration
def test_public_native_process_and_exact_results(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3] / "packages/reference"
    def documents(path: Path) -> dict[str, str]:
        return {p.relative_to(path).as_posix(): p.read_text() for p in path.rglob("*") if p.is_file() and p.suffix in {".toml", ".yaml", ".yml", ".pse"}}
    physical = runtime.physical_from_documents(documents(root / "physical"))
    package = runtime.modeling_from_documents([documents(root / name) for name in ("seed-data", "process", "thermodynamics", "methods", "physical")], physical)
    case = SemanticId.from_hex("68ba8dc2d6b05d9a9fe1b1a3625d8015")
    settings = pse.SolveSettings(backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT)
    members = cast("list[dict[str, object]]", package.inspect(case, settings)["members"])
    coordinates = {cast("str", cast("dict[str, object]", m["lineage"])["path"]): SemanticId.from_hex(cast("str", m["id"])) for m in members}
    prepared = package.prepare_solve(case, settings)
    handle = prepared.start()

    async def wait_twice() -> pse.RunResult:
        first, second = await asyncio.gather(handle.wait_async(), handle.wait_async())
        assert first.run_id == second.run_id
        return first

    result = asyncio.run(wait_twice())
    assert handle.wait().run_id == result.run_id
    table = pa.RecordBatchReader.from_stream(
        result.table("runtime.solve_variables")
    ).read_all()
    rows = table.to_pylist()
    for path, expected in [("root.phase.T", 350.0), ("root.phase.rho", 34.565566349336066), ("root.recycle", 5.0)]:
        actual = next(row["value"] for row in rows if SemanticId(row["symbol_id"]) == coordinates[f"heater_recycle.{path}"])
        assert actual == pytest.approx(expected, abs=1e-5)
    assert result.usable
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks and all(row["satisfied"] for row in checks)
    source = pa.table(result.table("authored.modeling_declarations"))
    assert any(SemanticId(row["declaration_id"]) == case for row in source.to_pylist())
    arrays = table.column("value").chunks
    del table, result, prepared, package, runtime
    gc.collect()
    assert any(a.null_count < len(a) for a in arrays)


@pytest.mark.integration
def test_public_dynamic_and_transient_fit(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
    tmp_path: Path,
) -> None:
    # Only durable runs publish (ADR-0112 Outcome 16); every run is a stored attempt.
    runtime = pse.Runtime(inspection_settings, store=operational_store)
    root = Path(__file__).resolve().parents[3]
    primitives = root / "tests/fixtures/packages/physical-primitives"
    physical = runtime.physical_from_documents({str(p.relative_to(primitives)):p.read_text() for p in primitives.rglob("*") if p.is_file()})
    identity = lambda n: SemanticId(bytes([n])*16)
    manifest = (root / "tests/fixtures/packages/minimal_explicit/package.toml").read_text().replace('id_policy = "explicit"','id_policy = "named"')
    manifest += f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{identity(31).to_hex()}"\n'
    manifest += f'\n[[quantity_aliases]]\nname = "Time"\nquantity_type_id = "{identity(222).to_hex()}"\n'
    package = runtime.modeling_from_documents([{
        "package.toml":manifest,
        "models/accumulation.pse":"""package accumulation { def Experiment {
          domain t:Time from 0{s} to 1{s};
          discretize grid on t using integrated(elements=1,order=1);
          param rate:Scalar=3;
          var total[i in t]:Time;
          eq balance[i in t]:d(total[i])/di==rate;
          eq initial:total[0{s}]==2{s};
          let observed[i in t]:Time=total[i];
          annotation report observed("measurement");
          annotation check total(total[i]>=2{s});
        } }""",
    }],physical)
    case = next(d.declaration_id for d in package.declarations() if d.name=="Experiment")
    settings = pse.SimulationSettings(start=0.,end=1.,samples=[0.,.5,1.],atol=[1e-10],rtol=1e-9,parameter_scales=[1.],sensitivities=True)
    simulation = package.simulate(case,settings)
    assert simulation.accepted and simulation.termination=="completed"
    samples = pa.table(simulation.table()).to_pylist()
    assert len(samples)==6
    assert {sample["time"] for sample in samples}=={0.,.5,1.}
    for sample in samples:
        assert sample["value"]==pytest.approx(2.+3.*sample["time"],abs=1e-6)
    prepared = package.prepare_simulation(case, settings)
    handle = prepared.start()
    joined = handle.wait()
    assert handle.wait().run_id == joined.run_id
    assert joined.usable
    assert joined.completion.computation is not None
    assert joined.completion.computation.backend == "diffsol"
    assert pa.table(joined.table("runtime.simulation_samples")).to_pylist() == [
        {**row, "run_id": bytes.fromhex(joined.run_id.to_hex())} for row in samples
    ]
    assert pa.table(joined.table("authored.modeling_declarations")).num_rows > 0
    (listed,) = runtime.runs(run_id=joined.run_id)
    assert joined.attempt_id is not None
    assert listed.attempt_id == joined.attempt_id
    assert listed.kind == AttemptKind.SIMULATION
    assert listed.state == AttemptState.COMPLETED
    command = joined.prepare_publication(tmp_path.as_uri() + "/", identity(110))
    ticket = command.ticket
    location, version = command.commit()
    settled = runtime.settle_publication(ticket)
    assert isinstance(settled, pse.PublicationCommitted)
    assert settled.root.location == location and settled.root.version == version
    converter = codec.converter()
    fit = converter.structure({
        "fit_id":identity(101),
        "parameters":[{"symbol_id":identity(102),"fixed":False,"value":1.,"lower":0.,"upper":10.,"scale":1.}],
        "experiments":[{"experiment_id":identity(103),"case_id":case,"route":"integrated","bindings":[{"parameter_id":identity(102),"path":"rate"}]}],
        "observations":[{"observation_id":identity(104),"experiment_id":identity(103),"output_path":"observed[0{s}]","time":1.,"time_basis":None,"time_unit_id":None,"included":True,"importance":1.}],
    },authored.AuthoredFitCasesRow)
    observation = converter.structure({"observation_id":identity(104),"dataset_id":identity(105),"target":"analytic total at 1 s","value":5.,"unit_id":identity(3),"std_dev":1.,"timestamp":None,"tag":None,"source_span":{"document_id":identity(105),"start":0,"end":0}},authored.AuthoredObservationsRow)
    dataset = converter.structure({"dataset_id":identity(105),"name":"analytic accumulation","source":"total(t)=2 s+3*t","content_hash":"blake3:"+"03"*32},authored.AuthoredDatasetsRow)
    package = package.with_fit_data((fit,),(observation,),(dataset,))
    result = package.prepare_fit(fit.fit_id,pse.SolveSettings(backend=NativeBackend.IPOPT,intent=NativeSolveIntent.OPTIMIZE,presolve=PresolvePolicyKind.OFF, controls=pse.SolveControls(hessian=HessianMode.LIMITED_MEMORY)),{fit.experiments[0].experiment_id:settings}).start().wait()
    parameters = pa.table(result.table("runtime.fit_parameters")).to_pylist()
    assert len(parameters)==1
    assert parameters[0]["value"]==pytest.approx(3.,abs=1e-5)
    run = pa.table(result.table("runtime.computation_runs")).to_pylist()
    assert len(run)==1 and run[0]["termination"] in {"success","acceptable"}
    assert run[0]["error"] is None and run[0]["estimate_qualified"]
    assert result.completion.computation==converter.structure(run[0],runtime_contracts.RuntimeComputationRunsRow)
    assert result.usable
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks and all(check["satisfied"] for check in checks)


@pytest.mark.integration
def test_import_without_pyomo() -> None:
    program = """
import importlib.abc, sys
class NoPyomo(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == 'pyomo' or fullname.startswith('pyomo.'):
            raise AssertionError('production attempted to import Pyomo')
sys.meta_path.insert(0, NoPyomo())
import pse
assert pse.Runtime and pse.ModelingPackage and pse.SolveSettings
assert not any(n == 'pyomo' or n.startswith('pyomo.') for n in sys.modules)
"""
    subprocess.run(
        [sys.executable, "-c", program], check=True, capture_output=True, text=True
    )


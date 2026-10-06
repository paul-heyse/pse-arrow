# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""M22 public native workflow; shared authored fixtures and independent references."""

import asyncio
import gc
import subprocess
import sys
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse import codec
from pse.contracts import authored
from pse.contracts import runtime as runtime_contracts
from pse.contracts.documents import (
    AuthoredNumericalRequirementsRow,
    KktTolerances,
    NumericalPolicy,
    Profile,
)
from pse.contracts.enums import (
    DynamicsMethod,
    HessianMode,
    NativeBackend,
    NativeSolveIntent,
    NumericalCoordinates,
    NumericalTarget,
    PresolvePolicyKind,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId


@pytest.mark.integration
def test_public_native_process_and_exact_results(
    inspection_settings: pse.EngineSettings, canonical_substrate: str
) -> None:
    runtime = pse.Runtime(inspection_settings, substrate=canonical_substrate)
    root = Path(__file__).resolve().parents[3] / "packages/reference"

    def documents(path: Path) -> dict[str, str | bytes]:
        return {
            p.relative_to(path).as_posix(): p.read_bytes()
            for p in path.rglob("*")
            if p.is_file()
            and p.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}
        }

    physical = runtime.physical_from_documents(documents(root / "physical"))
    package = runtime.modeling_from_documents(
        [
            documents(root / name)
            for name in (
                "data/oracles/teqp-0.23.1",
                "data/oracles/feos-0.10.1",
                "data/gross-sadowski-2001",
                "data/references",
                "data/nist",
                "data/perry7",
                "data/poling2000",
                "data/oracles/idaes-2.13",
                "data/species",
                "data/ciaaw",
                "seed-data",
                "process",
                "thermodynamics",
                "methods",
                "domain",
                "physical",
            )
        ],
        physical,
    )
    package = package.with_limits(
        pse.ModelingLimits(body_occurrences=262144, body_slots=65536)
    )
    case = DeclarationId(SemanticId.from_hex("68ba8dc2d6b05d9a9fe1b1a3625d8015"))
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
    )
    members = package.inspect(case, settings).members
    coordinates = {m.lineage.path: SemanticId.from_hex(m.id) for m in members}
    prepared = package.prepare_solve(case, settings)
    handle = prepared.start()

    async def wait_twice() -> pse.RunResult:
        first, second = await asyncio.gather(handle.wait_async(), handle.wait_async())
        assert first.run_id == second.run_id
        return first

    result = asyncio.run(wait_twice())
    assert handle.wait().run_id == result.run_id
    assert result.usable, tuple(
        diagnostic.message for diagnostic in result.diagnostics()
    )
    table = pa.RecordBatchReader.from_stream(
        result.table("runtime.solve_variables")
    ).read_all()
    rows = table.to_pylist()
    for path, expected in [
        ("root.phase.T", 350.0),
        ("root.phase.rho", 34.565566349336066),
        ("root.recycle", 5.0),
    ]:
        actual = next(
            row["value"]
            for row in rows
            if SemanticId(row["symbol_id"]) == coordinates[f"heater_recycle.{path}"]
        )
        assert actual == pytest.approx(expected, abs=1e-5)
    assert result.usable
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(row["satisfied"] for row in checks)
    sources = package.source_tables()
    source = pa.table(sources[SemanticId.from_hex("8023798ccdf06c39d0badbf7250f144b")])
    assert any(SemanticId(row["declaration_id"]) == case for row in source.to_pylist())
    for stream in sources.values():
        stream.close()
    arrays = table.column("value").chunks
    del table, result, prepared, package, runtime
    gc.collect()
    assert any(a.null_count < len(a) for a in arrays)


@pytest.mark.integration
def test_public_dynamic_and_transient_fit(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: str,
) -> None:
    # Every ordinary run retains its exact canonical scientific attempt.
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

    def identity(n: int) -> SemanticId:
        return SemanticId(bytes([n]) * 16)

    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace(
        "dependencies = []",
        'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
        'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]',
    )
    package = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/accumulation.pse": """package accumulation { def Experiment {
          domain t:Time from 0{s} to 1{s};
          discretize grid on t using integrated(elements=1,order=1);
          param rate:Scalar=3;
          var total[i in t]:Time;
          eq balance[i in t]:d(total[i])/di==rate;
          eq initial:total[0{s}]==2{s};
          let observed[i in t]:Time=total[i];
          annotation report observed("measurement");
          annotation check total(total[i]>=2{s});
        }
        test experiment_integrated fixture {
          dof 0; route integrated; procedure integrate;
          integrate samples(0{s},0.5{s},1{s}) relative(1e-9)
            normalized_absolute(1e-10) step(1e-4{s});
        } { child root:Experiment=Experiment(); }
        entity kind origin provenance {attribute title:Text;}
        entity origin experiment {title="analytic total(t)=2 s+3*t"}
        enum role {measured facets(measured)}
        entity kind reading {attribute value:Time?;attribute sigma:Time?;}
        @id("68686868686868686868686868686868")
        entity reading measured provenance(experiment,role.measured) {
          value=5{s},sigma=1{s}
        } }""",
            }
        ],
        physical,
    )
    case = next(
        d.declaration_id
        for d in package.declarations()
        if d.name == "experiment_integrated"
    )
    settings = pse.SimulationSettings(
        method=DynamicsMethod.DIFFSOL,
        start=0.0,
        end=1.0,
        samples=[0.0, 0.5, 1.0],
        atol=[1e-10],
        rtol=1e-9,
        parameter_scales=[1.0],
        sensitivity="forward",
    )
    simulation = package.simulate(case, settings)
    assert simulation.accepted
    assert simulation.termination == "completed"
    samples = pa.table(simulation.table()).to_pylist()
    assert len(samples) == 6
    assert {sample["time"] for sample in samples} == {0.0, 0.5, 1.0}
    for sample in samples:
        assert sample["value"] == pytest.approx(2.0 + 3.0 * sample["time"], abs=1e-6)
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
    sources = package.source_tables()
    assert (
        pa.table(
            sources[SemanticId.from_hex("8023798ccdf06c39d0badbf7250f144b")]
        ).num_rows
        > 0
    )
    for stream in sources.values():
        stream.close()
    assert joined.canonical_run_key is not None
    assert joined.canonical_attempt_key is not None
    (stored,) = pa.table(
        runtime.attempt_record(joined.canonical_attempt_key)
    ).to_pylist()
    assert stored["run"] == joined.canonical_run_key
    assert stored["outcome"] == "succeeded"
    retained_samples = pa.table(
        runtime.results(
            joined.canonical_run_key,
            joined.canonical_attempt_key,
            "runtime.simulation_samples",
        )
    )
    # Dense storage groups the declared output and sample keys so a selected
    # output reads only its own blocks. Every original scientific column is
    # preserved; relation row ranges address that canonical stored coverage.
    expected_samples = sorted(
        pa.table(joined.table("runtime.simulation_samples")).to_pylist(),
        key=lambda row: (row["symbol_id"], row["sample"]),
    )
    assert retained_samples.to_pylist() == expected_samples
    destination = tmp_path / "simulation.arrow"
    runtime.export_results(
        joined.canonical_run_key,
        joined.canonical_attempt_key,
        "runtime.simulation_samples",
        destination,
    )
    assert destination.is_file()
    with pa.ipc.open_stream(destination) as reader:
        assert reader.read_all().equals(retained_samples)
    converter = codec.converter()
    fit = converter.structure(
        {
            "fit_id": identity(101),
            "parameters": [
                {
                    "symbol_id": identity(102),
                    "fixed": False,
                    "value": 1.0,
                    "lower": 0.0,
                    "upper": 10.0,
                    "scale": 1.0,
                }
            ],
            "experiments": [
                {
                    "experiment_id": identity(103),
                    "case_id": case,
                    "route": "integrated",
                    "bindings": [{"parameter_id": identity(102), "path": "root.rate"}],
                }
            ],
            "observations": [
                {
                    "observation_id": identity(104),
                    "value_attribute": "value",
                    "standard_deviation_attribute": "sigma",
                    "experiment_id": identity(103),
                    "output_path": "root.observed[0{s}]",
                    "time": 1.0,
                    "time_basis": None,
                    "time_unit_id": None,
                    "included": True,
                    "importance": 1.0,
                }
            ],
        },
        authored.AuthoredFitCasesRow,
    )
    package = package.with_fit_declarations((fit,))
    simulation_profile = codec.decode_json(settings.to_json(), Profile)
    result = (
        package.prepare_fit(
            fit.fit_id,
            pse.FitPreparationDocument(
                solver=pse.SolveSettings(
                    backend=NativeBackend.IPOPT,
                    intent=NativeSolveIntent.OPTIMIZE,
                    presolve=PresolvePolicyKind.OFF,
                    controls=pse.SolveControls(hessian=HessianMode.LIMITED_MEMORY),
                    # This analytic fit promises a 1e-5 rate, with a standardized
                    # scalar least-squares objective. Bind its actual coordinates
                    # and retain independent first-order optimality requirements.
                    numerics=NumericalPolicy(
                        kkt=KktTolerances(stationarity=1e-8, complementarity=1e-8),
                        requirements=tuple(
                            AuthoredNumericalRequirementsRow(
                                requirement_id=identity(index).to_hex(),
                                fit_id=fit.fit_id.to_hex(),
                                target_id=target,
                                target_kind=kind,
                                unit_id=identity(10).to_hex(),
                                coordinates=NumericalCoordinates.PHYSICAL,
                                absolute_tolerance=absolute,
                                relative_tolerance=0.0,
                                priority=1,
                                required=True,
                                provenance="analytic transient fit output accuracy",
                            )
                            for index, target, kind, absolute in (
                                (
                                    107,
                                    fit.parameters[0].symbol_id.to_hex(),
                                    NumericalTarget.VARIABLE,
                                    1e-7,
                                ),
                                (
                                    108,
                                    identity(0).to_hex(),
                                    NumericalTarget.OBJECTIVE,
                                    1e-10,
                                ),
                            )
                        ),
                    ),
                ),
                simulations={
                    fit.experiments[0].experiment_id.to_hex(): simulation_profile
                },
            ),
        )
        .start()
        .wait()
    )
    parameters = pa.table(result.table("runtime.fit_parameters")).to_pylist()
    assert len(parameters) == 1
    assert parameters[0]["value"] == pytest.approx(3.0, abs=1e-5)
    run = pa.table(result.table("runtime.computation_runs")).to_pylist()
    assert len(run) == 1
    assert run[0]["termination"] in {"success", "acceptable"}
    assert run[0]["error"] is None
    assert run[0]["estimate_qualified"]
    exported = pa.table(result.export_fit_parameters()).to_pylist()
    assert len(exported) == 1
    assert exported[0]["value"] == pytest.approx(3.0, abs=1e-5)
    assert exported[0]["run_id"] == bytes(result.run_id)
    assert result.completion.computation is not None
    assert codec.document_rows(
        (result.completion.computation,), runtime_contracts.RuntimeComputationRunsRow
    ) == (converter.structure(run[0], runtime_contracts.RuntimeComputationRunsRow),)
    assert result.usable
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(check["satisfied"] for check in checks)


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


#: A manifest dependency on the physical primitives fixture. Its document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)
assert pse.Runtime and pse.ModelingPackage and pse.SolveSettings
assert not any(n == 'pyomo' or n.startswith('pyomo.') for n in sys.modules)
"""
    subprocess.run(
        [sys.executable, "-c", program], check=True, capture_output=True, text=True
    )

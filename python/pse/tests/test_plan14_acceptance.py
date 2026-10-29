# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""M22 public native workflow; shared authored fixtures and independent references."""

import gc
import math
import subprocess
import sys
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse import codec
from pse._scientific_inputs import family_packages, package_documents
from pse.contracts import authored
from pse.contracts.documents import NumericalPolicy
from pse.contracts.enums import (
    DynamicSensitivity,
    DynamicsMethod,
    ModelingDeclarationKind,
    Namespace,
    NativeBackend,
    NativeSolveIntent,
    NumericalTarget,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.reference import ReferenceSchemaRelationsRow
from pse.contracts.values import SemanticId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.physical_input_fixture import (
    coordinate_batch,
    inspection_request,
    member,
)


def _source_declarations(
    package: pse.ModelingPackage, settings: pse.EngineSettings
) -> list[authored.AuthoredModelingDeclarationsRow]:
    with (
        pse.registry_table("reference.schema_relations", settings=settings) as stream,
        pa.RecordBatchReader.from_stream(stream) as reader,
    ):
        relations = codec.structure_rows(
            reader.read_all().to_pylist(), ReferenceSchemaRelationsRow
        )
    declaration_relation = next(
        relation.relation_id
        for relation in relations
        if relation.namespace is Namespace.AUTHORED
        and relation.name == "modeling_declarations"
    )
    sources = package.source_tables()
    try:
        return codec.structure_rows(
            pa.table(sources[declaration_relation]).to_pylist(),
            authored.AuthoredModelingDeclarationsRow,
        )
    finally:
        for stream in sources.values():
            stream.close()


@pytest.mark.integration
def test_public_native_process_and_exact_results(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3] / "packages/reference"

    physical_documents = package_documents(root / "physical")
    assert physical_documents is not None
    physical = runtime.physical_from_documents(physical_documents)
    package = runtime.modeling_from_documents(
        family_packages("seed-transport"),
        physical,
    )
    package = package.with_limits(
        pse.ModelingLimits(body_occurrences=262144, body_slots=65536)
    )
    case = DeclarationId(SemanticId.from_hex("68ba8dc2d6b05d9a9fe1b1a3625d8015"))
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.FEASIBLE_POINT
    )
    members = package.inspect(inspection_request(case, settings)).members
    coordinates = {m.lineage.path: m.id for m in members}
    prepared = package.prepare_solve(case, settings)
    handle = prepared.start()

    result = handle.wait()
    assert handle.wait().run_id == result.run_id
    assert result.usable, tuple(
        diagnostic.message for diagnostic in result.diagnostics()
    )
    table = pa.RecordBatchReader.from_stream(
        result.table("runtime.solve_variables")
    ).read_all()
    rows = table.to_pylist()
    for path in ("root.phase.T", "root.phase.rho", "root.recycle"):
        (coordinate,) = (
            row
            for row in rows
            if SemanticId(row["symbol_id"]) == coordinates[f"heater_recycle.{path}"]
        )
        assert coordinate["tolerance"] > 0.0
        assert math.isfinite(coordinate["value"])
    assert result.usable
    checks = pa.table(result.table("runtime.modeling_checks")).to_pylist()
    assert checks
    assert all(row["satisfied"] for row in checks)
    # The authored expectations own the historical values and shared physical
    # resolutions. Variable-bound tolerances do not bound forward output error.
    expectations = {
        SemanticId.from_hex(identity)
        for identity in (
            "964b1c0ea202559c8488c703872008f2",
            "73792f4040a15d0caf4a3bab8f0e6541",
            "23f779ec23815d84b4611b52f9b3d5f0",
        )
    }
    assert expectations <= {SemanticId(row["source_id"]) for row in checks}
    declarations = _source_declarations(package, inspection_settings)
    assert any(row.declaration_id == case for row in declarations)
    arrays = table.column("value").chunks
    del table, result, prepared, package, runtime
    gc.collect()
    assert any(a.null_count < len(a) for a in arrays)


@pytest.mark.integration
def test_public_dynamic_stored_results_and_export(
    inspection_settings: pse.EngineSettings,
    tmp_path: Path,
    canonical_substrate: CanonicalFixture,
) -> None:
    # Every ordinary run retains its exact canonical scientific attempt.
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
          integrate samples(0{s},0.5{s},1{s}) relative(global)
            normalized_absolute(global) step(1e-4{s});
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
    numerics = NumericalPolicy()
    settings = pse.SimulationSettings(
        runtime=runtime,
        method=DynamicsMethod.DIFFSOL,
        start=0.0,
        end=1.0,
        samples=[0.0, 0.5, 1.0],
        atol=coordinate_batch(
            runtime,
            [
                (
                    member(package, case, "total"),
                    numerics.engineering_relative_fraction,
                    "Time",
                )
            ],
        ),
        rtol=numerics.engineering_relative_fraction,
        parameter_scales=coordinate_batch(
            runtime, [(member(package, case, "rate"), 1.0, "Scalar")]
        ),
        sensitivity=DynamicSensitivity.FORWARD,
    )
    simulation = package.simulate(case, settings)
    assert simulation.accepted
    assert simulation.termination == "completed"
    samples = pa.table(simulation.table()).to_pylist()
    assert len(samples) == 6
    assert {sample["time"] for sample in samples} == {0.0, 0.5, 1.0}
    prepared = package.prepare_simulation(case, settings)
    handle = prepared.start()
    joined = handle.wait()
    assert handle.wait().run_id == joined.run_id
    assert joined.usable
    assert joined.completion.computation is not None
    assert joined.completion.computation.backend == "diffsol"
    (state_policy,) = (
        row
        for row in pa.table(joined.table("runtime.resolved_numerics")).to_pylist()
        if row["target_kind"] == NumericalTarget.VARIABLE
    )
    assert state_policy["budget"] > 0.0
    # The authored observation is the total state itself in the same Time unit.
    # Both channels therefore use that state's actual frozen physical allowance.
    for sample in samples:
        assert sample["value"] == pytest.approx(
            2.0 + 3.0 * sample["time"], rel=0.0, abs=state_policy["budget"]
        )
    assert pa.table(joined.table("runtime.simulation_samples")).to_pylist() == [
        {**row, "run_id": bytes.fromhex(joined.run_id.to_hex())} for row in samples
    ]
    declarations = _source_declarations(package, inspection_settings)
    experiment = next(
        row
        for row in declarations
        if row.name == "Experiment"
        and row.value.kind is ModelingDeclarationKind.DEFINITION
    )
    children = {
        row.name: row
        for row in declarations
        if row.parent_id == experiment.declaration_id
    }
    rate = children["rate"]
    assert rate.value.kind is ModelingDeclarationKind.PARAMETER
    assert rate.value.binding is not None
    assert rate.value.binding.expression == "3"
    total = children["total"]
    assert total.value.kind is ModelingDeclarationKind.VARIABLE
    assert total.value.binding is not None
    assert len(total.value.binding.indices) == 1
    assert total.value.binding.indices[0].domain == "t"
    assert any(
        row.declaration_id == case and row.name == "experiment_integrated"
        for row in declarations
    )
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

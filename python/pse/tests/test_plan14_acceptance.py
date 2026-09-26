# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""M22 public native workflow; shared authored fixtures and independent references."""

import asyncio
import gc
import subprocess
import sys
import uuid
from pathlib import Path

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec
from pse import modeling as w
from pse.contracts import authored
from pse.contracts import runtime as runtime_contracts
from pse.contracts.values import SemanticId

FIXTURE = Path(__file__).resolve().parents[3] / "tests/fixtures/plan14"


def numerical_requirement(
    model: SemanticId,
    case: SemanticId | None,
    target: SemanticId,
    kind: str,
    tolerance: float,
) -> dict[str, object]:
    return {
        "requirement_id": uuid.uuid5(
            uuid.NAMESPACE_URL, f"plan14:{model}:{case}:{target}:{kind}"
        ).hex,
        "model_id": model.to_hex(),
        "case_id": None if case is None else case.to_hex(),
        "target_id": target.to_hex(),
        "target_kind": kind,
        "nominal": None,
        "scaling_factor": None,
        "absolute_tolerance": tolerance,
        "relative_tolerance": None,
        "unit_id": None,
        "coordinates": "physical",
        "priority": 0,
        "required": True,
        "provenance": "Plan 14 acceptance budget migrated by semantic identity",
    }


@pytest.mark.integration
def test_public_native_process_and_exact_results(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package = FIXTURE / "package"
    docs = {
        str(p.relative_to(package)): p.read_text()
        for p in package.rglob("*")
        if p.is_file()
    }
    physical = runtime.physical_from_documents(docs)
    declaration = codec.converter().structure(
        msgspec.json.decode(
            (FIXTURE / "model.json").read_bytes(), type=dict[str, object]
        ),
        w.ModelDeclaration,
    )
    draft = runtime.from_declaration(declaration, physical)
    for row in msgspec.json.decode(
        (FIXTURE / "providers.json").read_bytes(), type=list[dict[str, object]]
    ):
        draft.native_provider(
            codec.converter().structure(row, authored.AuthoredNativeProvidersRow)
        )
    balances: list[authored.AuthoredPhysicalBalancesRow] = []
    for row in msgspec.json.decode(
        (FIXTURE / "balances.json").read_bytes(), type=list[dict[str, object]]
    ):
        balance = codec.converter().structure(row, authored.AuthoredPhysicalBalancesRow)
        balances.append(balance)
        draft.balance(balance)
    selected = next(c for c in declaration.cases if c.name == "heater-recycle")
    targets = [(v.port.symbol_id, "variable", 1e-6) for v in selected.variables]
    targets.extend((r.row_id, "row", 1e-4) for r in selected.rows)
    targets.extend(
        (b.balance_id, "row", 1e-4) for b in balances if b.case_id == selected.case_id
    )
    for target, kind, tolerance in targets:
        draft.numerical_requirement(
            codec.converter().structure(
                numerical_requirement(
                    declaration.model_id, selected.case_id, target, kind, tolerance
                ),
                authored.AuthoredNumericalRequirementsRow,
            )
        )
    revision = draft.freeze()
    prepared = revision.prepare(
        selected.case_id,
        pse.SolveSettings(
            backend="ipopt",
            intent="feasible_point",
        ),
    )
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
    for source, expected in zip(
        selected.variables, [76.85, 34.565566349336066, 5.0], strict=True
    ):
        actual = next(
            row["value"]
            for row in rows
            if SemanticId(row["symbol_id"]) == source.port.symbol_id
        )
        assert actual == pytest.approx(expected, abs=1e-5)
    source = pa.RecordBatchReader.from_stream(
        result.table("authored.computation_models")
    ).read_all()
    assert source.num_rows == 1
    arrays = table.column("value").chunks
    del table, result, prepared, revision, draft, runtime
    gc.collect()
    assert any(a.null_count < len(a) for a in arrays)


@pytest.mark.integration
def test_public_dynamic_and_transient_fit(
    inspection_settings: pse.EngineSettings,
) -> None:
    runtime = pse.Runtime(inspection_settings)
    package = FIXTURE / "package"
    physical = runtime.physical_from_documents(
        {
            str(p.relative_to(package)): p.read_text()
            for p in package.rglob("*")
            if p.is_file()
        }
    )
    converter = codec.converter()
    model = converter.structure(
        msgspec.json.decode(
            (FIXTURE / "dynamic-model.json").read_bytes(), type=dict[str, object]
        ),
        w.ModelDeclaration,
    )
    dynamic = converter.structure(
        msgspec.json.decode(
            (FIXTURE / "dynamic-source.json").read_bytes(), type=dict[str, object]
        ),
        authored.AuthoredDynamicCasesRow,
    )
    fit = converter.structure(
        msgspec.json.decode(
            (FIXTURE / "fit-source.json").read_bytes(), type=dict[str, object]
        ),
        authored.AuthoredFitCasesRow,
    )
    draft = runtime.from_declaration(model, physical)
    draft.dynamics(dynamic)
    draft.fit(fit)
    draft.observation(
        converter.structure(
            msgspec.json.decode(
                (FIXTURE / "fit-observation.json").read_bytes(), type=dict[str, object]
            ),
            authored.AuthoredObservationsRow,
        )
    )
    draft.dataset(
        converter.structure(
            msgspec.json.decode(
                (FIXTURE / "fit-dataset.json").read_bytes(), type=dict[str, object]
            ),
            authored.AuthoredDatasetsRow,
        )
    )
    revision = draft.freeze()
    settings = pse.SimulationSettings(
        start=0.0,
        end=1.0,
        samples=[0.0, 0.5, 1.0],
        atol=[1e-8],
        parameter_scales=[1.0],
        sensitivities=True,
    )
    simulation = (
        revision.prepare_simulation(dynamic.dynamic_id, settings).start().wait()
    )
    run = (
        pa.RecordBatchReader.from_stream(simulation.table("runtime.computation_runs"))
        .read_all()
        .to_pylist()
    )
    assert len(run) == 1
    assert run[0]["trajectory_termination"] == "completed"
    assert run[0]["error"] is None
    samples = (
        pa.RecordBatchReader.from_stream(simulation.table("runtime.simulation_samples"))
        .read_all()
        .to_pylist()
    )
    assert len(samples) == 6  # State and declared output at each requested time.
    assert {SemanticId(sample["symbol_id"]) for sample in samples} == {
        dynamic.states[0].symbol_id,
        dynamic.outputs[0],
    }
    assert {sample["time"] for sample in samples} == {0.0, 0.5, 1.0}
    for sample in samples:
        assert sample["value"] == pytest.approx(2.0 + 3.0 * sample["time"], abs=1e-6)
    result = (
        revision.prepare_fit(
            fit.fit_id,
            pse.SolveSettings(
                backend="ipopt",
                intent="optimize",
                hessian="limited_memory",
                numerics={
                    "requirements": [
                        numerical_requirement(
                            fit.model_id,
                            None,
                            fit.parameters[0].symbol_id,
                            "variable",
                            1e-6,
                        )
                    ]
                },
            ),
            {fit.experiments[0].experiment_id: settings},
        )
        .start()
        .wait()
    )
    parameters = (
        pa.RecordBatchReader.from_stream(result.table("runtime.fit_parameters"))
        .read_all()
        .to_pylist()
    )
    assert len(parameters) == 1
    assert parameters[0]["value"] == pytest.approx(3.0, abs=1e-5)
    run = (
        pa.RecordBatchReader.from_stream(result.table("runtime.computation_runs"))
        .read_all()
        .to_pylist()
    )
    assert len(run) == 1
    assert run[0]["termination"] in {"success", "acceptable"}
    assert result.completion.computation == converter.structure(
        run[0], runtime_contracts.RuntimeComputationRunsRow
    )
    assert run[0]["error"] is None


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
assert pse.Runtime and pse.ModelBuilder and pse.SolveSettings
assert not any(n == 'pyomo' or n.startswith('pyomo.') for n in sys.modules)
"""
    subprocess.run(
        [sys.executable, "-c", program], check=True, capture_output=True, text=True
    )


@pytest.mark.unit
def test_shared_source_contracts() -> None:
    """Decode generated declarations without constructing a runtime."""
    declaration = codec.converter().structure(
        msgspec.json.decode(
            (FIXTURE / "model.json").read_bytes(), type=dict[str, object]
        ),
        w.ModelDeclaration,
    )
    assert {c.name for c in declaration.cases} == {
        "heater-recycle",
        "heater-optimization",
        "flash",
        "separator-0",
        "separator-1",
        "separator-2",
    }
    assert declaration.domains
    assert declaration.groups
    assert any(c.objective is not None for c in declaration.cases)
    for row in msgspec.json.decode(
        (FIXTURE / "providers.json").read_bytes(), type=list[dict[str, object]]
    ):
        codec.converter().structure(row, authored.AuthoredNativeProvidersRow)
    for row in msgspec.json.decode(
        (FIXTURE / "balances.json").read_bytes(), type=list[dict[str, object]]
    ):
        codec.converter().structure(row, authored.AuthoredPhysicalBalancesRow)
    malformed = msgspec.json.decode(
        (FIXTURE / "model.json").read_bytes(), type=dict[str, object]
    )
    malformed["obsolete_math_ir"] = []
    with pytest.raises(ExceptionGroup):
        codec.converter().structure(malformed, w.ModelDeclaration)

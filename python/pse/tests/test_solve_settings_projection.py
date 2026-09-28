# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed settings projection, registry names and typed eligibility (ADR-0113)."""

import json
from pathlib import Path

import pytest

import pse
from pse.contracts.enums import (
    NativeBackend,
    NativeIneligibility,
    NativeProblemClass,
    NativeRunState,
)
from pse.contracts.values import SemanticId

MODELS = """package settings {
 def Lp {var x:Scalar; var y:Scalar; eq total:x+y>=1; eq cap:x<=0.5;
  annotation bounds x(0, 10); annotation bounds y(0, 10);
  annotation start x(0); annotation start y(0); annotation objective y(minimize);}
 def Nlp {var x:Scalar; var y:Scalar; var f:Scalar;
  eq distance:f==(x-1)*(x-1)+(y-2)*(y-2); eq budget:x+y<=2;
  annotation bounds x(0, 2); annotation bounds y(0, 2); annotation bounds f(0, 10);
  annotation start x(0.5); annotation start y(0.5); annotation start f(1);
  annotation objective f(minimize);}
 def Root {var x:Scalar; eq square:x*x==4; annotation start x(1);}
}"""


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


@pytest.fixture(scope="module")
def runtime(inspection_settings: pse.EngineSettings) -> pse.Runtime:
    return pse.Runtime(inspection_settings)


@pytest.fixture(scope="module")
def physical(runtime: pse.Runtime) -> pse.PhysicalContext:
    root = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/packages/physical-primitives"
    )
    return runtime.physical_from_documents(
        {
            str(path.relative_to(root)): path.read_text()
            for path in root.rglob("*")
            if path.is_file()
        }
    )


@pytest.fixture(scope="module")
def cases(
    runtime: pse.Runtime, physical: pse.PhysicalContext
) -> tuple[pse.ModelingPackage, dict[str, SemanticId]]:
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    scalar = identity(31).to_hex()
    manifest += (
        f'\n[[quantity_aliases]]\nname = "Scalar"\nquantity_type_id = "{scalar}"\n'
    )
    package = runtime.modeling_from_documents(
        [{"package.toml": manifest, "models/settings.pse": MODELS}], physical
    )
    return package, {
        row.name: row.declaration_id
        for row in package.declarations()
        if row.name in {"Lp", "Nlp", "Root"}
    }


def attempt_of(result: pse.ModelingResult) -> pse.NativeAttempt:
    attempt = result.attempt()
    assert attempt is not None, result.failure()
    return attempt


@pytest.mark.unit
def test_solve_settings_backend_projection(
    runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    cases: tuple[pse.ModelingPackage, dict[str, SemanticId]],
) -> None:
    variants: dict[str, dict[str, object]] = {
        "highs": {"method": "simplex"},
        "clarabel": {"mode": "reusable_data", "max_step_fraction": 0.9},
        "pounce": {"method": "active_set_sqp"},
        "kinsol": {"strategy": "newton", "linear": {"dense": {"limit": 16}}},
        "scip": {"seed": 7, "nodes": 1000},
        "ipopt": {"linear": {"mumps": {"ordering": "amd"}}, "mu_strategy": "adaptive"},
    }
    settings: dict[str, pse.BackendSettings] = {}
    for backend, fields in variants.items():
        typed = pse.BackendSettings(backend, **fields)
        settings[backend] = typed
        assert typed.backend == backend
        # Omitted fields take the Rust defaults; nothing is restated in Python.
        effective = typed.fields()
        assert set(effective) == set(pse.BackendSettings(backend).fields())
        assert {key: effective[key] for key in fields} == fields
        # The versioned document and the solve settings round-trip every variant.
        assert pse.BackendSettings.from_json(typed.to_json()) == typed
        assert pse.BackendSettings.from_json(typed.to_json()).identity == typed.identity
        solve = pse.SolveSettings(backend=backend, settings=typed)
        assert solve.backend == backend
        assert solve.settings == typed
    # Unknown fields, versions and backends are refused natively.
    with pytest.raises(pse.InspectionError, match="unknown field"):
        pse.BackendSettings("highs", methd="simplex")
    document = json.loads(settings["highs"].to_json())
    document["version"] = document["version"] + 1
    with pytest.raises(pse.InspectionError, match="version"):
        pse.BackendSettings.from_json(json.dumps(document))
    with pytest.raises(pse.InspectionError, match="simulation profile"):
        pse.BackendSettings("idas")
    with pytest.raises(pse.InspectionError, match="expected one of"):
        pse.BackendSettings("gurobi")
    # Solve defaults are read from Rust; Python passes no default of its own.
    default = pse.SolveSettings()
    assert (default.intent, default.backend, default.settings) == (
        "optimize",
        None,
        None,
    )
    assert (default.presolve, default.hessian, default.reuse, default.start) == (
        "auto",
        "exact",
        "fresh",
        "no_prior_start",
    )
    assert default.iterations > 0
    assert default.time_limit > 0
    assert default.threads == 1
    # Dynamics profile settings project the same way.
    idas = pse.IdasSettings(linear={"spgmr": {"dimension": 8}}, sensitivity="staggered")
    assert pse.IdasSettings.from_json(idas.to_json()) == idas
    assert (
        idas.fields()["initialization"] == pse.IdasSettings().fields()["initialization"]
    )
    diffsol = pse.DiffsolSettings(method="tr_bdf2")
    assert diffsol.fields()["linear"] == pse.DiffsolSettings().fields()["linear"]
    simulation = pse.SimulationSettings(
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=[1e-8],
        parameter_scales=[],
        method="idas",
        idas=idas,
        diffsol=diffsol,
    )
    assert simulation.idas == idas
    assert simulation.diffsol == diffsol
    assert json.loads(simulation.to_json())["idas"]["sensitivity"] == "staggered"
    with pytest.raises(pse.InspectionError, match="unknown field"):
        pse.DiffsolSettings(scheme="bdf")

    # Each variant reaches its backend: the report records the effective settings.
    package, ids = cases
    highs = attempt_of(
        package.solve_case(
            ids["Lp"], pse.SolveSettings(backend="highs", settings=settings["highs"])
        )
    )
    assert highs.backend == "highs"
    assert highs.options()["solver"] == "simplex"
    pounce = attempt_of(
        package.solve_case(
            ids["Nlp"], pse.SolveSettings(backend="pounce", settings=settings["pounce"])
        )
    )
    assert pounce.backend == "pounce"
    assert pounce.options()["algorithm"] == "active-set-sqp"
    kinsol = attempt_of(
        package.solve_case(
            ids["Root"],
            pse.SolveSettings(
                intent="root",
                backend="kinsol",
                presolve="off",
                settings=settings["kinsol"],
            ),
        )
    )
    assert kinsol.backend == "kinsol"
    method = json.loads(kinsol.provenance()["settings"])
    assert method["strategy"] == "newton"
    assert method["linear"] == {"dense": {"limit": 16}}
    scip = attempt_of(
        package.solve_case(
            ids["Nlp"], pse.SolveSettings(backend="scip", settings=settings["scip"])
        )
    )
    assert scip.backend == "scip"
    assert scip.options()["randomization/randomseedshift"] == 7
    ipopt = attempt_of(
        package.solve_case(
            ids["Nlp"], pse.SolveSettings(backend="ipopt", settings=settings["ipopt"])
        )
    )
    assert ipopt.backend == "ipopt"
    options = ipopt.options()
    assert options["linear_solver"] == "mumps"
    assert options["mumps_pivot_order"] == 0
    assert options["mu_strategy"] == "adaptive"
    assert json.loads(ipopt.provenance()["linear"]) == {"mumps": {"ordering": "amd"}}
    port = {
        "quantity_id": identity(31).to_hex(),
        "unit_id": identity(10).to_hex(),
    }
    request: dict[str, object] = {
        "variables": [{"symbol_id": identity(201).to_hex(), **port}],
        "rows": [{"symbol_id": identity(202).to_hex(), **port}],
        "objective_port": {"symbol_id": identity(0).to_hex(), **port},
        "quadratic": {
            "rows": 1,
            "columns": 1,
            "column_starts": [0, 0],
            "row_indices": [],
            "values": [],
        },
        "objective": [1.0],
        "constraints": {
            "rows": 1,
            "columns": 1,
            "column_starts": [0, 1],
            "row_indices": [0],
            "values": [-1.0],
        },
        "rhs": [-2.0],
        "cones": [{"kind": "nonnegative", "dimension": 1}],
        "objective_constant": 3.0,
        "gram_factors": [],
        "gram_weights": [],
    }
    (row,) = (
        runtime.prepare_conic(
            request,
            physical,
            pse.SolveSettings(backend="clarabel", settings=settings["clarabel"]),
        )
        .run()
        .attempts()
    )
    assert row.report is not None
    native = json.loads(row.report.provenance()["settings.effective"])
    assert native["max_step_fraction"] == 0.9
    assert native["presolve_enable"] is False


@pytest.mark.unit
def test_route_and_eligibility_are_typed(
    cases: tuple[pse.ModelingPackage, dict[str, SemanticId]],
) -> None:
    package, ids = cases
    backends = {member.value for member in NativeBackend}
    reasons = {member.value for member in NativeIneligibility}
    prepared = package.prepare_solve(ids["Lp"], pse.SolveSettings())
    assert prepared.route.backend == "highs"
    assert not prepared.route.constant
    rows = {row.backend: row for row in prepared.eligibility}
    assert set(rows) <= backends
    assert rows["highs"].eligible
    assert not rows["highs"].reasons
    for row in rows.values():
        assert row.eligible == (not row.reasons)
        assert {reason.code for reason in row.reasons} <= reasons
    # Typed detail values: the problem classes the facts establish, by registry name.
    (kinsol,) = [r for r in rows["kinsol"].reasons if r.code == "class"]
    assert "linear" in kinsol.problem_classes
    assert set(kinsol.problem_classes) <= {
        member.value for member in NativeProblemClass
    }
    assert kinsol.derivative_order is None
    assert kinsol.sign_bounds is None
    (bounds,) = [r for r in rows["kinsol"].reasons if r.code == "bounds"]
    assert bounds.sign_bounds is True
    assert bounds.problem_classes == []

    # Strategy routes are typed rows, one per initialization block, and attempts and
    # failures share one index space.
    initialization = package.prepare_block_initialization(
        ids["Root"], pse.SolveSettings(intent="initialize"), [{}, {}]
    )
    assert [route.backend for route in initialization.routes] == ["kinsol"]
    result = initialization.run()
    attempts = result.attempts()
    assert [a.stage for a in attempts] == [0, 1]
    for attempt in attempts:
        assert (attempt.report is None) != (attempt.failure is None)
        assert attempt.route == initialization.routes[0]
        assert attempt.committed
    assert result.failures() == ()

    # Initialization admission is native: no Python check precedes it.
    with pytest.raises(pse.InspectionError, match="root or initialize intent"):
        package.prepare_block_initialization(ids["Root"], pse.SolveSettings(), [{}])
    with pytest.raises(pse.InspectionError, match="settings do not match"):
        package.prepare_block_initialization(
            ids["Root"],
            pse.SolveSettings(
                intent="initialize", settings=pse.BackendSettings("highs")
            ),
            [{}],
        )

    # Result observations carry registry names, never Rust Debug text.
    limited = package.solve_case(
        ids["Root"],
        pse.SolveSettings(
            intent="root", backend="kinsol", presolve="off", iterations=1
        ),
    )
    assert not limited.accepted
    assert limited.outcome_kind == NativeRunState.NATIVE
    failure = limited.failure()
    assert failure is not None
    texts = {o.name: o.text for o in failure.observations}
    assert texts["termination"] == "iteration_limit"
    assert texts["qualification"] == "unqualified"

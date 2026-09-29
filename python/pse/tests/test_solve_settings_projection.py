# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed settings documents, registry names, typed eligibility (ADR-0113, ADR-0116)."""

import json
from pathlib import Path

import msgspec
import pytest

import pse
from pse import codec
from pse.contracts import documents
from pse.contracts.enums import (
    ClarabelMode,
    DiffsolMethod,
    DynamicSensitivity,
    HessianMode,
    HighsMethod,
    KinsolStrategy,
    MumpsOrdering,
    MuStrategy,
    NativeBackend,
    NativeIneligibility,
    NativeProblemClass,
    NativeRunState,
    NativeSolveIntent,
    NativeStartPolicy,
    PounceMethod,
    PresolvePolicyKind,
    ReusePolicy,
    SensitivityCorrector,
)
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId


#: A manifest dependency on the physical primitives fixture, whose physical document names
#: `Scalar`, `Length` and `Time` (ADR-0123 Outcome 6).
PRIMITIVES = (
    'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", '
    'version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]'
)

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
) -> tuple[pse.ModelingPackage, dict[str, DeclarationId]]:
    root = Path(__file__).resolve().parents[3]
    manifest = (
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace("dependencies = []", PRIMITIVES)
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
def test_backend_settings_typed() -> None:
    """Each backend's settings are a generated document type (Plan 22 X13)."""
    typed: dict[NativeBackend, pse.BackendSettings] = {
        NativeBackend.HIGHS: pse.HighsSettings(method=HighsMethod.SIMPLEX),
        NativeBackend.CLARABEL: pse.ClarabelSettings(
            mode=ClarabelMode.REUSABLE_DATA, max_step_fraction=0.9
        ),
        NativeBackend.POUNCE: pse.PounceSettings(method=PounceMethod.ACTIVE_SET_SQP),
        NativeBackend.KINSOL: pse.KinsolSettings(
            strategy=KinsolStrategy.NEWTON, linear=documents.KinsolLinearDense(limit=16)
        ),
        NativeBackend.SCIP: pse.ScipSettings(seed=7, nodes=1000),
        NativeBackend.IPOPT: pse.IpoptSettings(
            linear=documents.IpoptLinearMumps(ordering=MumpsOrdering.AMD),
            mu_strategy=MuStrategy.ADAPTIVE,
        ),
    }
    for backend, settings in typed.items():
        # The document is tagged by the registry backend spelling, and every omitted
        # field is present with the default the Rust type states.
        encoded = msgspec.to_builtins(settings)
        assert encoded["backend"] == backend.value
        decoded = msgspec.json.decode(
            msgspec.json.encode(settings), type=pse.BackendSettings
        )
        assert decoded == settings
        # Each field of each document type is typed: no field is `Any` or untyped.
        for field in msgspec.structs.fields(type(settings)):
            assert field.type is not object, (backend, field.name)
    # The generated types refuse what their schema refuses, before anything is native.
    with pytest.raises(msgspec.ValidationError, match="unknown field"):
        msgspec.json.decode(
            b'{"backend": "highs", "methd": "simplex"}', type=pse.BackendSettings
        )
    with pytest.raises(msgspec.ValidationError):
        msgspec.json.decode(b'{"backend": "gurobi"}', type=pse.BackendSettings)
    with pytest.raises(
        msgspec.ValidationError, match=r"Expected `float` >= 0\.0|> 0\.0"
    ):
        msgspec.json.decode(
            b'{"backend": "ipopt", "bound_push": -1.0}', type=pse.BackendSettings
        )


@pytest.mark.unit
def test_solve_settings_enum_types() -> None:
    """Every enumeration of the settings documents is the registry's generated type."""
    hints = {
        field.name: field.type for field in msgspec.structs.fields(pse.SolveSettings)
    }
    assert hints["intent"] is NativeSolveIntent
    assert hints["presolve"] is PresolvePolicyKind
    controls = {
        field.name: field.type for field in msgspec.structs.fields(pse.SolveControls)
    }
    assert controls["hessian"] is HessianMode
    assert controls["reuse"] is ReusePolicy
    assert controls["start"] is NativeStartPolicy
    ipopt = {
        field.name: field.type for field in msgspec.structs.fields(pse.IpoptSettings)
    }
    assert ipopt["mu_strategy"] is MuStrategy
    # Defaults are the Rust document's, stated once by the generator.
    default = pse.SolveSettings()
    assert (default.intent, default.backend, default.settings) == (
        NativeSolveIntent.OPTIMIZE,
        None,
        None,
    )
    assert default.presolve is PresolvePolicyKind.AUTO
    assert (
        default.controls.hessian,
        default.controls.reuse,
        default.controls.start,
    ) == (
        HessianMode.EXACT,
        ReusePolicy.FRESH,
        NativeStartPolicy.NO_PRIOR_START,
    )
    assert default.controls.iterations > 0
    assert default.controls.time_limit > 0
    assert default.controls.threads == 1
    # A misspelled member is refused where the document is decoded.
    with pytest.raises(msgspec.ValidationError):
        msgspec.json.decode(
            b'{"version": 1, "intent": "rooot"}', type=pse.SolveSettings
        )
    with pytest.raises(msgspec.ValidationError):
        msgspec.json.decode(b'{"version": 2}', type=pse.SolveSettings)


@pytest.mark.unit
def test_solve_settings_backend_projection(
    runtime: pse.Runtime,
    physical: pse.PhysicalContext,
    cases: tuple[pse.ModelingPackage, dict[str, DeclarationId]],
) -> None:
    # Dynamics profile settings are versioned documents too.
    idas = pse.IdasSettings(
        linear=documents.IdasLinearSpgmr(dimension=8),
        sensitivity=SensitivityCorrector.STAGGERED,
    )
    assert msgspec.json.decode(msgspec.json.encode(idas), type=pse.IdasSettings) == idas
    assert idas.initialization == pse.IdasSettings().initialization
    diffsol = pse.DiffsolSettings(method=DiffsolMethod.TR_BDF2)
    assert diffsol.linear == pse.DiffsolSettings().linear
    simulation = pse.SimulationSettings(
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=[1e-8],
        parameter_scales=[],
        method="idas",
        idas=codec.encode_json(idas),
        diffsol=codec.encode_json(diffsol),
    )
    assert msgspec.json.decode(simulation.idas, type=pse.IdasSettings) == idas
    assert msgspec.json.decode(simulation.diffsol, type=pse.DiffsolSettings) == diffsol
    assert json.loads(simulation.to_json())["idas"]["sensitivity"] == "staggered"
    # The adjoint checkpoints are a document; the sensitivity is a registry name.
    adjoint = pse.AdjointSettings(steps_between_checkpoints=50)
    assert adjoint.max_checkpoints == pse.AdjointSettings().max_checkpoints
    gradient = pse.SimulationSettings(
        start=0.0,
        end=1.0,
        samples=[0.0, 1.0],
        atol=[1e-8],
        parameter_scales=[1.0],
        sensitivity=DynamicSensitivity.ADJOINT.value,
        adjoint=codec.encode_json(adjoint),
    )
    assert msgspec.json.decode(gradient.adjoint, type=pse.AdjointSettings) == adjoint
    assert json.loads(gradient.to_json())["sensitivity"] == "adjoint"
    # An unknown field or version is refused natively as well.
    with pytest.raises(pse.InspectionError, match="unknown field"):
        pse.SimulationSettings(
            start=0.0,
            end=1.0,
            samples=[0.0],
            atol=[1e-8],
            parameter_scales=[],
            diffsol=b'{"version": 1, "scheme": "bdf"}',
        )
    with pytest.raises(pse.InspectionError, match="unknown document version"):
        pse.SimulationSettings(
            start=0.0,
            end=1.0,
            samples=[0.0],
            atol=[1e-8],
            parameter_scales=[],
            idas=b'{"version": 3}',
        )

    # Each variant reaches its backend: the report records the effective settings.
    package, ids = cases
    highs = attempt_of(
        package.solve_case(
            ids["Lp"],
            pse.SolveSettings(
                backend=NativeBackend.HIGHS,
                settings=pse.HighsSettings(method=HighsMethod.SIMPLEX),
            ),
        )
    )
    assert highs.backend == "highs"
    assert highs.options()["solver"] == "simplex"
    pounce = attempt_of(
        package.solve_case(
            ids["Nlp"],
            pse.SolveSettings(
                backend=NativeBackend.POUNCE,
                settings=pse.PounceSettings(method=PounceMethod.ACTIVE_SET_SQP),
            ),
        )
    )
    assert pounce.backend == "pounce"
    assert pounce.options()["algorithm"] == "active-set-sqp"
    kinsol = attempt_of(
        package.solve_case(
            ids["Root"],
            pse.SolveSettings(
                intent=NativeSolveIntent.ROOT,
                backend=NativeBackend.KINSOL,
                presolve=PresolvePolicyKind.OFF,
                settings=pse.KinsolSettings(
                    strategy=KinsolStrategy.NEWTON,
                    linear=documents.KinsolLinearDense(limit=16),
                ),
            ),
        )
    )
    assert kinsol.backend == "kinsol"
    method = json.loads(kinsol.provenance()["settings"])
    assert method["strategy"] == "newton"
    assert method["linear"] == {"kind": "dense", "limit": 16}
    scip = attempt_of(
        package.solve_case(
            ids["Nlp"],
            pse.SolveSettings(
                backend=NativeBackend.SCIP,
                settings=pse.ScipSettings(seed=7, nodes=1000),
            ),
        )
    )
    assert scip.backend == "scip"
    assert scip.options()["randomization/randomseedshift"] == 7
    ipopt = attempt_of(
        package.solve_case(
            ids["Nlp"],
            pse.SolveSettings(
                backend=NativeBackend.IPOPT,
                settings=pse.IpoptSettings(
                    linear=documents.IpoptLinearMumps(ordering=MumpsOrdering.AMD),
                    mu_strategy=MuStrategy.ADAPTIVE,
                ),
            ),
        )
    )
    assert ipopt.backend == "ipopt"
    options = ipopt.options()
    assert options["linear_solver"] == "mumps"
    assert options["mumps_pivot_order"] == 0
    assert options["mu_strategy"] == "adaptive"
    assert json.loads(ipopt.provenance()["linear"]) == {
        "kind": "mumps",
        "ordering": "amd",
    }
    # A document the native decoder refuses is refused at the entry point.
    with pytest.raises(pse.InspectionError, match=r"bound_push|Tolerance"):
        package.solve_case(
            ids["Nlp"],
            pse.SolveSettings(
                backend=NativeBackend.IPOPT, settings=pse.IpoptSettings(bound_push=-1.0)
            ),
        )
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
    }
    (row,) = (
        runtime.prepare_conic(
            request,
            physical,
            pse.SolveSettings(
                backend=NativeBackend.CLARABEL,
                settings=pse.ClarabelSettings(
                    mode=ClarabelMode.REUSABLE_DATA, max_step_fraction=0.9
                ),
            ),
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
    cases: tuple[pse.ModelingPackage, dict[str, DeclarationId]],
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
        ids["Root"], pse.SolveSettings(intent=NativeSolveIntent.INITIALIZE), [{}, {}]
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
                intent=NativeSolveIntent.INITIALIZE, settings=pse.HighsSettings()
            ),
            [{}],
        )

    # Result observations carry registry names, never Rust Debug text.
    limited = package.solve_case(
        ids["Root"],
        pse.SolveSettings(
            intent=NativeSolveIntent.ROOT,
            backend=NativeBackend.KINSOL,
            presolve=PresolvePolicyKind.OFF,
            controls=pse.SolveControls(iterations=1),
        ),
    )
    assert not limited.accepted
    assert limited.outcome_kind == NativeRunState.NATIVE
    failure = limited.failure()
    assert failure is not None
    texts = {o.name: o.text for o in failure.observations}
    assert texts["termination"] == "iteration_limit"
    assert texts["qualification"] == "unqualified"

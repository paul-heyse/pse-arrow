# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Parametric sensitivity against Pyomo's sIPOPT interface (``sens.py``; ADR-0118).

Both sides solve one quadratic program with an active bound,
``min x²/2 + y² - b x + z²/2 + a z`` subject to ``x + y = a`` and ``z ∈ [0, 10]`` at
``(a, b) = (1, 2)``. sIPOPT returns the first-order perturbed solution at
``(a, b) = (1.1, 2.1)``; pse's sensitivities, read from its one KKT-point analysis,
predict the same point from the solution. The program's KKT conditions are linear in
the parameters. Production derivatives are evaluated at the actual qualified returned
candidate; its base-point engineering error is separate from the derivative increment.
The exact perturbed solution is
``x = (b + 2a)/3``, ``y = (a - b)/3``, ``z = 0``.
"""

import pyomo.environ as pyo
import pytest
from pyomo.contrib.sensitivity_toolbox.sens import sensitivity_calculation

import pse
from pse.contracts import documents
from pse.contracts.enums import (
    DerivedQuantity,
    DualQualification,
    NativeBackend,
    NativeSolveIntent,
    NumericalSource,
    NumericalTarget,
)

from . import support

QUADRATIC = """package parity {
 def Root { param a: Scalar = 1; param b: Scalar = 2;
  var x: Scalar; var y: Scalar; var z: Scalar;
  eq coupling: x + y == a;
  let cost: Scalar = 0.5*x*x + y*y - b*x + 0.5*z*z + a*z;
  annotation objective cost(minimize);
  annotation bounds z(0, 10);
  annotation start x(0.5); annotation start y(0.5); annotation start z(0.5); } }"""
NOMINAL = (1.0, 2.0)
PERTURBED = (1.1, 2.1)


def sipopt() -> dict[str, float]:
    """The first-order perturbed solution sIPOPT returns."""
    m = pyo.ConcreteModel()
    m.a = pyo.Param(initialize=NOMINAL[0], mutable=True)
    m.b = pyo.Param(initialize=NOMINAL[1], mutable=True)
    m.x = pyo.Var(initialize=0.5)
    m.y = pyo.Var(initialize=0.5)
    m.z = pyo.Var(bounds=(0, 10), initialize=0.5)
    m.coupling = pyo.Constraint(expr=m.x + m.y == m.a)
    m.cost = pyo.Objective(
        expr=0.5 * m.x**2 + m.y**2 - m.b * m.x + 0.5 * m.z**2 + m.a * m.z
    )
    solved = sensitivity_calculation("sipopt", m, [m.a, m.b], list(PERTURBED))
    return {
        name: pyo.value(solved.sens_sol_state_1[getattr(solved, name)])
        for name in ("x", "y", "z")
    }


def pse_prediction(
    runtime: pse.Runtime,
) -> tuple[dict[str, float], dict[str, float], dict[str, float], dict[str, float]]:
    """Actual production candidate, physical increment, prediction, and budgets."""
    authored, declarations = support.package(runtime, QUADRATIC)
    root = declarations["Root"]
    plain = pse.SolveSettings(
        backend=NativeBackend.IPOPT, intent=NativeSolveIntent.OPTIMIZE
    )
    ids = support.members(authored, root, plain)
    a, b = support.by_suffix(ids, "a"), support.by_suffix(ids, "b")
    settings = pse.SolveSettings(
        backend=NativeBackend.IPOPT,
        intent=NativeSolveIntent.OPTIMIZE,
        sensitivity=documents.SensitivityRequest(parameters=(a.to_hex(), b.to_hex())),
    )
    result = authored.prepare_solve(root, settings).start().wait()
    assert result.usable, result.diagnostics()
    variable_rows = support.rows(result.table("runtime.solve_variables"))
    variables = {support.identity(row["symbol_id"]): row for row in variable_rows}
    assert len(variables) == len(variable_rows)
    solved = {
        identity: support.real(row["value"]) for identity, row in variables.items()
    }
    # Authored parameter bindings remain exact. SensitivityRequest supplies no
    # private scale override; returned dx/dp and the parameter step use the
    # original Scalar units, regardless of the runner's normalized KKT scales.
    assert solved[a] == NOMINAL[0]
    assert solved[b] == NOMINAL[1]
    assert variables[a]["parameter"] is True
    assert variables[b]["parameter"] is True
    assert variables[a]["fixed"] is True
    assert variables[b]["fixed"] is True
    assert variables[a]["unit_id"] is not None
    assert variables[a]["quantity_id"] is not None
    assert settings.numerics == documents.NumericalPolicy()
    assert settings.sensitivity is not None
    assert settings.sensitivity.parameters == (a.to_hex(), b.to_hex())
    (assessment,) = support.rows(result.table("runtime.candidate_assessments"))
    assert assessment["validated"] is True
    assert assessment["permits_result"] is True
    assert assessment["refusals"] == []
    (local,) = support.rows(result.table("runtime.local_validity"))
    assert local["quantity"] == DerivedQuantity.PARAMETRIC_SENSITIVITY
    validity = support.record(local["validity"])
    assert validity["certified"] is True
    assert validity["reason"] is None
    assert validity["conditional"] is False
    assert validity["licq"] is True
    assert validity["strict_complementarity"] is True
    assert validity["second_order"] is True
    assert validity["weakly_active"] == 0
    basis = (local["run_id"], local["step"])
    assert (assessment["run_id"], assessment["step"]) == basis
    assert all((row["run_id"], row["step"]) == basis for row in variable_rows)
    z = support.by_suffix(ids, "z")
    bound = variables[z]
    assert bound["lower"] == 0.0
    assert bound["upper"] == 10.0
    assert bound["dual_qualification"] == DualQualification.SENSITIVITY_CERTIFIED
    assert abs(solved[z] - support.real(bound["lower"])) <= support.real(
        bound["tolerance"]
    )
    assert support.real(bound["lower_dual"]) > 0.0
    assert support.real(bound["upper_dual"]) < support.real(bound["lower_dual"])
    # The active lower-bound branch is regular at both parameter points: a>0
    # implies the unconstrained z=-a remains outside the admitted z>=0 domain.
    assert NOMINAL[0] > 0.0
    assert PERTURBED[0] > 0.0
    resolved = {}
    for row in support.rows(result.table("runtime.resolved_numerics")):
        assert (row["run_id"], row["step"]) == basis
        assert support.real(row["nominal"]) > 0.0
        assert support.real(row["coordinate_scale"]) > 0.0
        engineering = support.record(row["engineering"])
        assert (
            engineering["relative_fraction"]
            == settings.numerics.engineering_relative_fraction
        )
        assert all(
            not item["selected"] or item["source"] != NumericalSource.ANALYSIS
            for item in support.records(row["provenance"])
        )
        if row["target_kind"] == NumericalTarget.VARIABLE:
            target = support.identity(row["target_id"])
            assert target not in resolved
            assert row["unit_id"] == variables[target]["unit_id"]
            assert row["quantity_id"] == variables[target]["quantity_id"]
            resolved[target] = row
            assert support.real(engineering["budget"]) > 0.0
            # No explicit numerical requirement changes the default physical
            # allowance in this fixture. Keep that premise visible.
            assert engineering["budget"] == row["budget"]
    required = {a, b} | {support.by_suffix(ids, name) for name in ("x", "y", "z")}
    assert required <= set(resolved)
    sensitivities = support.rows(result.table("runtime.parametric_sensitivities"))
    assert all((row["run_id"], row["step"]) == basis for row in sensitivities)
    step = {a: PERTURBED[0] - NOMINAL[0], b: PERTURBED[1] - NOMINAL[1]}
    jacobian = {
        "x": {a: 2.0 / 3.0, b: 1.0 / 3.0},
        "y": {a: 1.0 / 3.0, b: -1.0 / 3.0},
        "z": {a: 0.0, b: 0.0},
    }
    base, increment, predicted, allowances = {}, {}, {}, {}
    for name in ("x", "y", "z"):
        target = support.by_suffix(ids, name)
        assert (
            variables[target]["dual_qualification"]
            == DualQualification.SENSITIVITY_CERTIFIED
        )
        allowances[name] = support.real(
            support.record(resolved[target]["engineering"])["budget"]
        )
        selected = [
            row
            for row in sensitivities
            if support.identity(row["target_id"]) == target
            and row["target_kind"] == NumericalTarget.VARIABLE
        ]
        derivative = {}
        for row in selected:
            parameter = support.identity(row["parameter_id"])
            assert parameter not in derivative
            assert parameter in {a, b}
            assert variables[target]["fixed"] is False
            assert variables[target]["parameter"] is False
            assert variables[parameter]["quantity_id"] == variables[a]["quantity_id"]
            assert variables[target]["quantity_id"] == variables[a]["quantity_id"]
            assert row["parameter_unit_id"] == variables[parameter]["unit_id"]
            assert row["target_unit_id"] == variables[target]["unit_id"]
            assert row["parameter_unit_id"] == row["target_unit_id"]
            derivative[parameter] = support.real(row["primal"])
            parameter_scale = support.real(resolved[parameter]["coordinate_scale"])
            output_scale = support.real(resolved[target]["coordinate_scale"])
            # Compare the known response to one normalized parameter step in
            # the production normalization, J_hat = J*S_p/S_x. Agreement with
            # the analytic response is empirical evidence at the frozen output
            # engineering allowance; it establishes no backward-error bound.
            normalized = derivative[parameter] * parameter_scale / output_scale
            want = jacobian[name][parameter] * parameter_scale / output_scale
            normalized_allowance = allowances[name] / output_scale
            assert normalized == pytest.approx(
                want, rel=0.0, abs=normalized_allowance
            ), (name, parameter)
        assert set(derivative) == {a, b}, sensitivities
        base[name] = solved[target]
        increment[name] = sum(derivative[p] * step[p] for p in (a, b))
        predicted[name] = base[name] + increment[name]
    return base, increment, predicted, allowances


@pytest.mark.integration
@pytest.mark.parity
def test_sensitivity_agrees_with_ipopt_sens(runtime: pse.Runtime) -> None:
    """ADR-0118's named comparison on a nondegenerate program.

    Original-coordinate increments are checked independently of the actual base
    candidate's engineering error. End predictions use the same global Scalar
    accuracy as production; sIPOPT remains an independent comparison.
    """
    reference = sipopt()
    base, increment, predicted, allowances = pse_prediction(runtime)
    a, b = PERTURBED
    exact = {"x": (b + 2 * a) / 3, "y": (a - b) / 3, "z": 0.0}
    a0, b0 = NOMINAL
    nominal = {"x": (b0 + 2 * a0) / 3, "y": (a0 - b0) / 3, "z": 0.0}
    for name in ("x", "y", "z"):
        # This empirical comparison uses the production target's frozen
        # engineering budget. A model-coordinate allowance itself does not
        # establish an output-error guarantee; the analytic oracle checks it.
        allowance = allowances[name]
        assert base[name] == pytest.approx(nominal[name], rel=0.0, abs=allowance), name
        exact_increment = exact[name] - nominal[name]
        assert increment[name] == pytest.approx(
            exact_increment, rel=0.0, abs=allowance
        ), name
        assert predicted[name] == pytest.approx(exact[name], rel=0.0, abs=allowance), (
            name
        )
        assert reference[name] == pytest.approx(exact[name], rel=0.0, abs=allowance), (
            name
        )
        # Both independent predictions were admitted against the same analytic
        # point above. Their pairwise difference carries both actual allowances.
        assert predicted[name] == pytest.approx(
            reference[name], rel=0.0, abs=2.0 * allowance
        ), name

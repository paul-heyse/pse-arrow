# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Parametric sensitivity against Pyomo's sIPOPT interface (``sens.py``; ADR-0118).

Both sides solve one quadratic program with an active bound,
``min x²/2 + y² - b x + z²/2 + a z`` subject to ``x + y = a`` and ``z ∈ [0, 10]`` at
``(a, b) = (1, 2)``. sIPOPT returns the first-order perturbed solution at
``(a, b) = (1.1, 2.1)``; pse's sensitivities, read from its one KKT-point analysis,
predict the same point from the solution. The program's KKT conditions are linear in
the parameters, so both predictions are the exact perturbed solution
``x = (b + 2a)/3``, ``y = (a - b)/3``, ``z = 0``.
"""

import pyomo.environ as pyo
import pytest
from pyomo.contrib.sensitivity_toolbox.sens import sensitivity_calculation

import pse
from pse.contracts import documents
from pse.contracts.enums import NativeBackend, NativeSolveIntent

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


def pse_prediction(runtime: pse.Runtime) -> dict[str, float]:
    """The pse solution plus its sensitivities times the parameter step."""
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
    solved = {
        support.identity(row["symbol_id"]): support.real(row["value"])
        for row in support.rows(result.table("runtime.solve_variables"))
    }
    sensitivities = support.rows(result.table("runtime.parametric_sensitivities"))
    step = {a: PERTURBED[0] - NOMINAL[0], b: PERTURBED[1] - NOMINAL[1]}
    predicted = {}
    for name in ("x", "y", "z"):
        target = support.by_suffix(ids, name)
        derivative = {
            support.identity(row["parameter_id"]): support.real(row["primal"])
            for row in sensitivities
            if support.identity(row["target_id"]) == target
            and row["primal"] is not None
        }
        assert set(derivative) == {a, b}, sensitivities
        predicted[name] = solved[target] + sum(derivative[p] * step[p] for p in (a, b))
    return predicted


@pytest.mark.integration
@pytest.mark.parity
def test_sensitivity_agrees_with_ipopt_sens(runtime: pse.Runtime) -> None:
    """ADR-0118's named comparison on a nondegenerate program.

    The pse first-order prediction equals sIPOPT's within 1e-6, and both equal the exact
    perturbed solution.
    """
    reference = sipopt()
    predicted = pse_prediction(runtime)
    a, b = PERTURBED
    exact = {"x": (b + 2 * a) / 3, "y": (a - b) / 3, "z": 0.0}
    for name in ("x", "y", "z"):
        assert predicted[name] == pytest.approx(reference[name], abs=1e-6), name
        assert reference[name] == pytest.approx(exact[name], abs=1e-6), name

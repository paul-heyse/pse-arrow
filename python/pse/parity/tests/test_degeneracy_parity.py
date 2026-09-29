# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Irreducible degenerate sets against IDAES's DegeneracyHunter (blueprint §15.4).

A square system of five linear equations has two known irreducible degenerate sets: a row
and its double, ``{first, second}`` with multipliers proportional to ``(2, −1)``, and three
rows whose sum closes, ``{third, fourth, fifth}`` with multipliers proportional to
``(1, 1, −1)``. DegeneracyHunter solves its candidate and irreducible-set MILPs with SCIP on
the unscaled Jacobian; pse's Jacobian diagnostic solves its minimum-support MILPs on rows
scaled by their recorded nominals.

The two differ in scope, not in answer. DegeneracyHunter's candidate MILP finds one
minimum-L1 null combination and searches irreducible sets only through its rows; both sets
here tie at that norm, so it reports one of them. pse pivots on every row and reports both.
Every set DegeneracyHunter reports is one of pse's, with pse's weights divided by its row
nominals equal to IDAES's multipliers up to one scale per set.
"""

import pyomo.environ as pyo
import pytest
from idaes.core.util.diagnostics_tools.degeneracy_hunter import DegeneracyHunter

import pse
from pse.contracts.enums import NativeSolveIntent
from pse.contracts.identities import DeclarationId
from pse.contracts.values import SemanticId

from . import support

ROWS = ("first", "second", "third", "fourth", "fifth")
DEPENDENT = """package parity {
 def Dependent { var a: Scalar; var b: Scalar; var c: Scalar; var d: Scalar; var e: Scalar;
  eq first: a + b == 1;
  eq second: 2*a + 2*b == 2;
  eq third: c + d == 1;
  eq fourth: d + e == 1;
  eq fifth: c + 2*d + e == 2;
  annotation start a(0.5); annotation start b(0.5); annotation start c(0.5);
  annotation start d(0.5); annotation start e(0.5); } }"""

Sets = dict[frozenset[str], dict[str, float]]


def idaes_sets() -> Sets:
    """DegeneracyHunter's irreducible degenerate sets with their multipliers."""
    m = pyo.ConcreteModel()
    for name in "abcde":
        setattr(m, name, pyo.Var(initialize=0.5))
    m.first = pyo.Constraint(expr=m.a + m.b == 1)
    m.second = pyo.Constraint(expr=2 * m.a + 2 * m.b == 2)
    m.third = pyo.Constraint(expr=m.c + m.d == 1)
    m.fourth = pyo.Constraint(expr=m.d + m.e == 1)
    m.fifth = pyo.Constraint(expr=m.c + 2 * m.d + m.e == 2)
    hunter = DegeneracyHunter(m, solver="scip")
    hunter.find_irreducible_degenerate_sets()
    return {
        frozenset(c.local_name for c in found): {
            c.local_name: float(nu) for c, nu in found.items()
        }
        for found in hunter.irreducible_degenerate_sets
    }


def row_names(
    authored: pse.ModelingPackage, case: DeclarationId, settings: pse.SolveSettings
) -> dict[SemanticId, str]:
    """Each row's name, read from the source locations of the IDAES-profile findings."""
    profile = pse.ModelingDiagnosticSettings.from_json(
        (support.ROOT / "packages/reference/diagnostics/idaes-2.13.json").read_text()
    )
    findings = authored.diagnose(case, settings, profile).table(
        "runtime.modeling_findings"
    )
    names: dict[SemanticId, str] = {}
    for finding in support.rows(findings):
        for location in finding["locations"]:  # type: ignore[union-attr]
            name = location["path"].rsplit(".", 1)[-1]
            if name in ROWS:
                names[SemanticId(location["source_id"])] = name
    assert sorted(names.values()) == sorted(ROWS), names
    return names


def pse_sets(runtime: pse.Runtime) -> Sets:
    """pse's degenerate sets with their weights on unscaled rows."""
    authored, declarations = support.package(runtime, DEPENDENT)
    case = declarations["Dependent"]
    settings = pse.SolveSettings(intent=NativeSolveIntent.ROOT)
    names = row_names(authored, case, settings)
    (report,) = support.rows(
        authored.diagnose_jacobian(case, settings, maximum_attempts=16).table()
    )
    assert report["complete"], report
    nominals = {
        SemanticId(v["source_id"]): v["value"]
        for v in report["row_nominals"]  # type: ignore[union-attr]
    }
    found: Sets = {}
    for degenerate in report["degenerate"]:  # type: ignore[union-attr]
        assert degenerate["irreducible_at_tolerance"], degenerate
        rows = frozenset(names[SemanticId(r)] for r in degenerate["rows"])
        found[rows] = {
            names[SemanticId(w["source_id"])]: w["value"]
            / nominals[SemanticId(w["source_id"])]
            for w in degenerate["certificate"]["weights"]
            if w["value"] != 0.0
        }
    return found


EXPECTED = {
    frozenset(ROWS[:2]): {"first": 2.0, "second": -1.0},
    frozenset(ROWS[2:]): {"third": 1.0, "fourth": 1.0, "fifth": -1.0},
}


def proportional(left: dict[str, float], right: dict[str, float]) -> None:
    """`left` and `right` name the same rows with multipliers equal up to one scale."""
    assert set(left) == set(right)
    pivot = min(left)
    for row in left:
        assert left[row] / left[pivot] == pytest.approx(
            right[row] / right[pivot], rel=1e-6
        ), (sorted(left), row)


@pytest.mark.integration
@pytest.mark.parity
def test_degenerate_sets_agree_with_degeneracy_hunter(runtime: pse.Runtime) -> None:
    """Every DegeneracyHunter set is one of pse's with the same multipliers, and pse
    reports both constructed sets."""
    reference = idaes_sets()
    found = pse_sets(runtime)
    assert set(found) == set(EXPECTED)
    assert reference
    assert set(reference) <= set(found)
    for rows, multipliers in reference.items():
        proportional(found[rows], multipliers)
    for rows, multipliers in EXPECTED.items():
        proportional(found[rows], multipliers)

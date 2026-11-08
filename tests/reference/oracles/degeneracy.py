# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES oracle for original degenerate-Jacobian row supports."""

from __future__ import annotations

import contextlib
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, cast

from google.protobuf import message_factory

from .ordinary_exchange import _message, float_bits
from .reference_environment import ReferenceEnvironment
from .scalar_case import ScalarCase, require

if TYPE_CHECKING:
    from pyomo.core.base.var import ScalarVar


@dataclass(frozen=True)
class LinearRow:
    """One original unscaled linear equation."""

    name: str
    coefficients: list[float]
    rhs: float


@dataclass(frozen=True)
class LinearCase:
    """Explicit scientific inputs shared with the native test."""

    id: str
    variables: list[str]
    point: list[float]
    rows: list[LinearRow]


@dataclass(frozen=True)
class RowSet:
    """Row support and multipliers in original equation coordinates."""

    rows: list[str]
    multipliers: dict[str, float]
    irreducible_at_tolerance: bool


# Independent algebraic answers retained from test_degeneracy_parity.py:
# first is half of second; third + fourth equals fifth. IDAES may find either
# minimum-L1 support, while the native all-anchor analysis must find both.
EXPECTED = {
    frozenset(("first", "second")): {"first": 2.0, "second": -1.0},
    frozenset(("third", "fourth", "fifth")): {
        "third": 1.0,
        "fourth": 1.0,
        "fifth": -1.0,
    },
}
Sets = dict[frozenset[str], dict[str, float]]


def validate_case(case: LinearCase) -> None:
    """Bind the algebraic expectations to exactly the original parity case."""
    require(case.id == "degenerate-jacobian-five-rows-v1", "unknown reference case")
    require(case.variables == list("abcde"), "unexpected original variables")
    require(case.point == [0.5] * 5, "unexpected original evaluation point")
    original = [
        ("first", [1.0, 1.0, 0.0, 0.0, 0.0], 1.0),
        ("second", [2.0, 2.0, 0.0, 0.0, 0.0], 2.0),
        ("third", [0.0, 0.0, 1.0, 1.0, 0.0], 1.0),
        ("fourth", [0.0, 0.0, 0.0, 1.0, 1.0], 1.0),
        ("fifth", [0.0, 0.0, 1.0, 2.0, 1.0], 2.0),
    ]
    require(
        [(row.name, row.coefficients, row.rhs) for row in case.rows] == original,
        "explicit equations differ from the independent algebraic reference",
    )


def reference_sets(case: LinearCase, environment: ReferenceEnvironment) -> list[RowSet]:
    """Execute IDAES on explicit equations without consulting candidate values."""
    import pyomo.environ as pyo
    from idaes.core.util.diagnostics_tools.degeneracy_hunter import DegeneracyHunter

    model = pyo.ConcreteModel()
    variables = {}
    for name, value in zip(case.variables, case.point, strict=True):
        variable = cast("ScalarVar", pyo.Var(initialize=value))
        model.add_component(name, variable)
        variables[name] = variable
    for row in case.rows:
        expression = sum(
            coefficient * variables[variable]
            for coefficient, variable in zip(
                row.coefficients, case.variables, strict=True
            )
            if coefficient != 0.0
        )
        model.add_component(row.name, pyo.Constraint(expr=expression == row.rhs))
    alias = "pse_reference_degeneracy"
    with environment.registration("scip", "degeneracy.search", alias=alias):
        hunter = DegeneracyHunter(
            model, solver=alias, solver_options={"limits/time": 60}
        )
        hunter.find_irreducible_degenerate_sets()
    # One candidate MILP and one irreducible-set MILP per candidate equation.
    environment.require_solves("degeneracy.search", 1 + len(hunter.degenerate_set))
    found: dict[frozenset[str], RowSet] = {}
    for members in hunter.irreducible_degenerate_sets:
        item = RowSet(
            rows=sorted(constraint.local_name for constraint in members),
            multipliers={
                constraint.local_name: float(weight)
                for constraint, weight in members.items()
            },
            irreducible_at_tolerance=True,
        )
        support = frozenset(item.rows)
        if support in found:
            # IDAES searches through multiple anchors of the same support. The
            # old parity test collapsed these; retain the scale consistency check.
            proportional(item.multipliers, found[support].multipliers)
        else:
            found[support] = item
    require(bool(found), "IDAES reported no irreducible set")
    return list(found.values())


def proportional(left: dict[str, float], right: dict[str, float]) -> None:
    """Preserve the original per-set scaling and pytest.approx relative tolerance."""
    require(set(left) == set(right) and bool(left), "multiplier support mismatch")
    require(
        all(
            math.isfinite(value) and value != 0.0
            for value in [*left.values(), *right.values()]
        ),
        "multipliers must be finite and nonzero on their support",
    )
    pivot = min(left)
    for row, value in left.items():
        observed = value / left[pivot]
        expected = right[row] / right[pivot]
        require(
            math.isfinite(observed)
            and abs(observed - expected) <= max(1e-12, 1e-6 * abs(expected)),
            f"multiplier ratio differs for {sorted(left)} at {row}",
        )


def indexed(sets: list[RowSet]) -> Sets:
    """Refuse duplicate supports instead of silently overwriting evidence."""
    found: Sets = {}
    for item in sets:
        support = frozenset(item.rows)
        require(
            len(support) == len(item.rows) and bool(support), "duplicate or empty rows"
        )
        require(support not in found, "duplicate degenerate set")
        require(set(item.multipliers) == support, "certificate and row support differ")
        require(item.irreducible_at_tolerance, "set is not irreducible at tolerance")
        found[support] = item.multipliers
    return found


def _case(original: ScalarCase) -> LinearCase:
    variables = list("abcde")
    rows = ("first", "second", "third", "fourth", "fifth")
    point = [f"point:{name}" for name in variables]
    inputs = list(point)
    for row in rows:
        inputs.extend(f"row:{row}:coefficient:{name}" for name in variables)
        inputs.append(f"row:{row}:rhs")
    values = original.inputs(inputs)
    original.outputs([f"row:{row}" for row in rows])
    case = LinearCase(
        id=original.contract.case_name,
        variables=variables,
        point=[values[key] for key in point],
        rows=[
            LinearRow(
                row,
                [values[f"row:{row}:coefficient:{name}"] for name in variables],
                values[f"row:{row}:rhs"],
            )
            for row in rows
        ],
    )
    validate_case(case)
    return case


def main() -> None:
    """Publish the original sparse certificates with explicit unreported supports."""
    require(len(sys.argv) == 4, "expected descriptor, contract and response paths")
    require(sys.flags.isolated == 1, "oracle requires the interpreter's -I option")
    original = ScalarCase.read(Path(sys.argv[1]), Path(sys.argv[2]))
    case = _case(original)
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            ("idaes-pse", "pyomo", "protobuf", "numpy", "pint", "scipy"),
            role="PSE_DEGENERACY_REFERENCE_PYTHON",
        )
        raw_reference = reference_sets(case, environment)
    reference = indexed(raw_reference)
    require(set(reference) <= set(EXPECTED), "IDAES found an unexpected row support")
    for rows, multipliers in reference.items():
        proportional(multipliers, EXPECTED[rows])
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    pool = original.envelope.DESCRIPTOR.file.pool
    result_type = message_factory.GetMessageClass(
        pool.FindMessageTypeByName("pse.reference.v1.ReferenceDegeneracyResult")
    )
    result = result_type(version=1)
    result.meaning.CopyFrom(_message(original.contract, "meaning"))
    row_ids = {item.key: item.output for item in original.contract.observables}
    for support, expected in EXPECTED.items():
        # Dict insertion order is the independently declared original row order.
        members = result.supports.add()
        members.rows.extend(row_ids[f"row:{row}"] for row in expected)
        if support in reference:
            reported = members.certificate.reported
            reported.multipliers.extend(float_bits(reference[support][row]) for row in expected)
            reported.irreducible_at_tolerance = True
        else:
            members.certificate.unreported.SetInParent()
    original.publish_result(Path(sys.argv[3]), result, 64 << 10)
    sys.stderr.write(
        "degeneracy_reference_environment "
        + json.dumps(environment.evidence, sort_keys=True)
        + "\n"
    )

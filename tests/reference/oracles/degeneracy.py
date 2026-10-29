# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Test-only IDAES oracle for explicit native degenerate-Jacobian case/results.

Run directly with the selected reference interpreter's -I option. This file does
not import a product package, generated contracts, or fixture adapters. Scientific
comparison failures are individual results; unavailable reference execution is a
process failure, never a skipped or passing comparison.
"""

from __future__ import annotations

import contextlib
import importlib.metadata
import math
import re
import shutil
import subprocess
import sys
from pathlib import Path

import msgspec


class LinearRow(msgspec.Struct, forbid_unknown_fields=True):
    """One original unscaled linear equation."""

    name: str
    coefficients: list[float]
    rhs: float


class LinearCase(msgspec.Struct, forbid_unknown_fields=True):
    """Explicit scientific inputs shared with the native test."""

    id: str
    variables: list[str]
    point: list[float]
    rows: list[LinearRow]


class RowSet(msgspec.Struct, forbid_unknown_fields=True):
    """Row support and multipliers in original equation coordinates."""

    rows: list[str]
    multipliers: dict[str, float]
    irreducible_at_tolerance: bool


class CandidateReport(msgspec.Struct, forbid_unknown_fields=True):
    """Native observations, without an assumed passing verdict."""

    complete: bool
    unavailable: list[str]
    sets: list[RowSet]


class Candidate(msgspec.Struct, forbid_unknown_fields=True):
    """An identified result or an explicitly missing result."""

    id: str
    report: CandidateReport | None


class Request(msgspec.Struct, forbid_unknown_fields=True):
    """The entire bounded exchange, independent of Python product state."""

    version: int
    case: LinearCase
    pins: dict[str, str]
    candidates: list[Candidate]


class Comparison(msgspec.Struct, forbid_unknown_fields=True):
    """One outcome, retained even when adjacent candidates pass or fail."""

    id: str
    status: str
    reasons: list[str]


class Response(msgspec.Struct, forbid_unknown_fields=True):
    """Reference observations and every candidate's comparison outcome."""

    version: int
    case_id: str
    environment: dict[str, str]
    reference: list[RowSet]
    comparisons: list[Comparison]


# Independent algebraic answers retained from test_degeneracy_parity.py:
# first is half of second; third + fourth equals fifth. IDAES may find either
# minimum-L1 support, while the native all-anchor analysis must find both.
EXPECTED = {
    frozenset(("first", "second")): {"first": 2.0, "second": -1.0},
    frozenset(("third", "fourth", "fifth")): {
        "third": 1.0, "fourth": 1.0, "fifth": -1.0
    },
}
Sets = dict[frozenset[str], dict[str, float]]


def require(condition: bool, reason: str) -> None:
    """Fail independently of Python's assertion optimization setting."""
    if not condition:
        raise ValueError(reason)


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


def environment(pins: dict[str, str]) -> dict[str, str]:
    """Preserve parity pin/interpreter/Ipopt checks and require SCIP for this oracle."""
    require((3, 11) <= sys.version_info[:2] < (3, 15), "reference requires Python 3.11-3.14")
    require(set(pins) == {"idaes-pse", "pyomo"}, "missing or unknown reference pins")
    observed = {name: importlib.metadata.version(name) for name in pins}
    require(observed == pins, f"reference pin mismatch: expected {pins}, observed {observed}")
    executable = shutil.which("ipopt")
    require(executable is not None, "ipopt is absent; reference execution never skips")
    if executable is None:
        raise ValueError("unreachable missing ipopt")
    probe = subprocess.run(  # noqa: S603 - resolved reference solver, fixed version probe
        [executable, "-v"], capture_output=True, text=True, timeout=30, check=False
    )
    match = re.search(r"\bIpopt\s+(\d+(?:\.\d+)*)", probe.stdout + probe.stderr)
    require(probe.returncode == 0 and match is not None, "ipopt version probe failed")
    if match is None:
        raise ValueError("unreachable missing ipopt version")
    require(match[1].startswith("3.14."), f"unexpected Ipopt series {match[1]}")
    observed["ipopt"] = match[1]
    observed["python"] = ".".join(str(part) for part in sys.version_info[:3])
    return observed


def reference_sets(case: LinearCase) -> list[RowSet]:
    """Execute IDAES on explicit equations without consulting candidate values."""
    import idaes
    import pyomo.environ as pyo
    from idaes import config
    from idaes.core.util.diagnostics_tools.degeneracy_hunter import DegeneracyHunter

    # Keep the explicitly prepared solver prefix ahead of IDAES's bundled fallback.
    idaes.cfg.use_idaes_solvers = False
    config.reconfig(idaes.cfg)
    require(pyo.SolverFactory("ipopt").available(False), "Pyomo cannot see ipopt")
    require(pyo.SolverFactory("scip").available(False), "Pyomo cannot see SCIP")
    model = pyo.ConcreteModel()
    for name, value in zip(case.variables, case.point, strict=True):
        model.add_component(name, pyo.Var(initialize=value))
    for row in case.rows:
        expression = sum(
            coefficient * model.component(variable)
            for coefficient, variable in zip(row.coefficients, case.variables, strict=True)
            if coefficient != 0.0
        )
        model.add_component(row.name, pyo.Constraint(expr=expression == row.rhs))
    hunter = DegeneracyHunter(
        model, solver="scip", solver_options={"limits/time": 60}
    )
    hunter.find_irreducible_degenerate_sets()
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
        all(math.isfinite(value) and value != 0.0 for value in [*left.values(), *right.values()]),
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
        require(len(support) == len(item.rows) and bool(support), "duplicate or empty rows")
        require(support not in found, "duplicate degenerate set")
        require(set(item.multipliers) == support, "certificate and row support differ")
        require(item.irreducible_at_tolerance, "set is not irreducible at tolerance")
        found[support] = item.multipliers
    return found


def compare(candidate: Candidate, reference: Sets) -> Comparison:
    """Retain pass/fail/missing separately across the entire supplied population."""
    report = candidate.report
    if report is None:
        return Comparison(candidate.id, "missing", ["native result missing"])
    try:
        require(report.complete, "native all-anchor analysis incomplete")
        require(not report.unavailable, f"native analysis unavailable: {report.unavailable}")
        found = indexed(report.sets)
        require(set(found) == set(EXPECTED), "native result must contain both constructed sets")
        require(set(reference) <= set(found), "IDAES support missing from native result")
        for rows, multipliers in reference.items():
            proportional(found[rows], multipliers)
        for rows, multipliers in EXPECTED.items():
            proportional(found[rows], multipliers)
    except ValueError as error:
        return Comparison(candidate.id, "failed", [str(error)])
    return Comparison(candidate.id, "passed", [])


def main() -> None:
    """Read one exchange and publish one response only after reference execution."""
    require(len(sys.argv) == 3, "expected request and response paths")
    require(sys.flags.isolated == 1, "oracle must run with the interpreter's -I option")
    request = msgspec.json.decode(Path(sys.argv[1]).read_bytes(), type=Request)
    require(request.version == 1, "unsupported exchange version")
    validate_case(request.case)
    require(bool(request.candidates), "candidate population is empty")
    ids = [candidate.id for candidate in request.candidates]
    require(all(ids) and len(set(ids)) == len(ids), "candidate identities must be unique and nonempty")
    # Library chatter is diagnostic output, never part of the result document.
    with contextlib.redirect_stdout(sys.stderr):
        observed = environment(request.pins)
        raw_reference = reference_sets(request.case)
    reference = indexed(raw_reference)
    require(set(reference) <= set(EXPECTED), "IDAES found an unexpected row support")
    for rows, multipliers in reference.items():
        proportional(multipliers, EXPECTED[rows])
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    response = Response(
        version=1,
        case_id=request.case.id,
        environment=observed,
        reference=raw_reference,
        comparisons=[compare(candidate, reference) for candidate in request.candidates],
    )
    Path(sys.argv[2]).write_bytes(msgspec.json.encode(response))


if __name__ == "__main__":
    main()

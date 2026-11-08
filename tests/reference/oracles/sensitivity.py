# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated sIPOPT oracle for explicit native original-unit sensitivity results.

No candidate fixture, product Python, or generated contract is imported. sIPOPT
predicts the perturbed state independently; the analytic KKT branch distinguishes
the derivative increment from the actual nominal candidate's engineering error.
"""

from __future__ import annotations

import contextlib
import math
import sys
from pathlib import Path
from typing import TYPE_CHECKING, cast

import msgspec

from .legacy_exchange import admit_candidates, read_request, write_response
from .reference_environment import LEGACY_PACKAGES, ReferenceEnvironment

if TYPE_CHECKING:
    from pyomo.core.base.param import ScalarParam
    from pyomo.core.base.var import ScalarVar


class SensitivityCase(msgspec.Struct, forbid_unknown_fields=True):
    """One dimensionless quadratic optimization and its parameter perturbation."""

    id: str
    nominal: dict[str, float]
    perturbed: dict[str, float]
    variables: list[str]
    unit: str
    initial: float
    z_bounds: tuple[float, float]
    coupling: str
    objective: str


class QuantityReport(msgspec.Struct, forbid_unknown_fields=True):
    """Physical values and dx/dp; coordinate scales are normalization metadata."""

    base: float
    increment: float
    predicted: float
    allowance: float
    coordinate_scale: float
    derivative: dict[str, float]


class CandidateReport(msgspec.Struct, forbid_unknown_fields=True):
    """Native observations with explicit coverage."""

    complete: bool
    unit: str
    coordinates: str
    parameter_scales: dict[str, float]
    quantities: dict[str, QuantityReport]


class Candidate(msgspec.Struct, forbid_unknown_fields=True):
    """Identified observation or an explicit missing native result."""

    id: str
    report: CandidateReport | None


class Request(msgspec.Struct, forbid_unknown_fields=True):
    """Scientific inputs, existing declared pins, and a mixed result population."""

    version: int
    case: SensitivityCase
    pins: dict[str, str]
    candidates: list[Candidate]


class Comparison(msgspec.Struct, forbid_unknown_fields=True):
    """A comparison retains its own pass/fail/missing result."""

    id: str
    status: str
    reasons: list[str]


class Response(msgspec.Struct, forbid_unknown_fields=True):
    """Independent sIPOPT prediction and all comparison outcomes."""

    version: int
    case_id: str
    environment: dict[str, str]
    reference: dict[str, float]
    comparisons: list[Comparison]


# The exact branch is affine, so the physical increment has no Taylor truncation
# error. Permit roundoff and numerical linear-system error separately from the
# two solver point errors, which include the active lower-bound relaxation.
DERIVATIVE_ABSOLUTE_ERROR = 1e-7
INCREMENT_ABSOLUTE_ERROR = 1e-8
NATIVE_VALUE_ABSOLUTE_ERROR = 5e-7
SIPOPT_VALUE_ABSOLUTE_ERROR = 5e-7


def require(condition: bool, message: str) -> None:
    """An absent environment or invalid scientific premise fails, never skips."""
    if not condition:
        raise ValueError(message)


def validate_case(case: SensitivityCase) -> None:
    """Keep the original independent quadratic and its active-bound branch."""
    require(case.id == "active-bound-quadratic-response", "unknown sensitivity case")
    require(case.nominal == {"a": 1.0, "b": 2.0}, "unexpected nominal parameters")
    require(case.perturbed == {"a": 1.1, "b": 2.1}, "unexpected perturbed parameters")
    require(case.variables == ["x", "y", "z"], "unexpected variables")
    require(case.unit == "dimensionless Scalar", "original Scalar units required")
    require(
        case.initial == 0.5 and case.z_bounds == (0.0, 10.0), "changed start/bounds"
    )
    require(case.coupling == "x + y == a", "changed equality")
    require(
        case.objective == "0.5*x*x + y*y - b*x + 0.5*z*z + a*z",
        "changed quadratic objective",
    )
    require(case.nominal["a"] > 0 and case.perturbed["a"] > 0, "active branch changed")


def sipopt_prediction(
    case: SensitivityCase,
    environment: ReferenceEnvironment,
) -> tuple[dict[str, float], dict[str, float]]:
    """Solve the original independent Pyomo model through its sIPOPT interface."""
    import pyomo.environ as pyo
    from pyomo.contrib.sensitivity_toolbox.sens import sensitivity_calculation

    model = pyo.ConcreteModel()
    a = cast("ScalarParam", pyo.Param(initialize=case.nominal["a"], mutable=True))
    b = cast("ScalarParam", pyo.Param(initialize=case.nominal["b"], mutable=True))
    x = cast("ScalarVar", pyo.Var(initialize=case.initial))
    y = cast("ScalarVar", pyo.Var(initialize=case.initial))
    z = cast("ScalarVar", pyo.Var(bounds=case.z_bounds, initialize=case.initial))
    model.a, model.b = a, b
    model.x, model.y, model.z = x, y, z
    model.coupling = pyo.Constraint(expr=x + y == a)
    model.cost = pyo.Objective(expr=0.5 * x**2 + y**2 - b * x + 0.5 * z**2 + a * z)
    # Pyomo's sIPOPT path constructs its own AMPL shell plugin and performs
    # nominal and perturbed calculation in that one invocation.
    with environment.registration(
        "ipopt_sens", "sensitivity.sipopt", alias="ipopt_sens"
    ):
        solved = sensitivity_calculation(
            "sipopt", model, [a, b], [case.perturbed["a"], case.perturbed["b"]]
        )
    environment.require_solves("sensitivity.sipopt", 1)
    prediction = {
        name: float(
            cast("float", pyo.value(solved.sens_sol_state_1[getattr(solved, name)]))
        )
        for name in case.variables
    }
    require(
        all(math.isfinite(value) for value in prediction.values()),
        "nonfinite sIPOPT response",
    )
    nominal = {
        name: float(cast("float", pyo.value(getattr(solved, name))))
        for name in case.variables
    }
    return nominal, prediction


def exact(parameters: dict[str, float]) -> dict[str, float]:
    """The regular active-bound branch has linear KKT conditions in a and b."""
    a, b = parameters["a"], parameters["b"]
    return {"x": (b + 2 * a) / 3, "y": (a - b) / 3, "z": 0.0}


def close(actual: float, expected: float, allowance: float, message: str) -> None:
    """Check a case-owned absolute error in the original physical coordinate."""
    require(
        math.isfinite(actual)
        and math.isfinite(expected)
        and math.isfinite(allowance)
        and allowance >= 0
        and abs(actual - expected) <= allowance,
        message,
    )


def check_sipopt(
    case: SensitivityCase, nominal: dict[str, float], prediction: dict[str, float]
) -> None:
    """Check the external solver before consulting the native candidate population."""
    require(
        set(nominal) == set(prediction) == set(case.variables),
        "sIPOPT response population differs",
    )
    for stage, observed, parameters in (
        ("nominal", nominal, case.nominal),
        ("prediction", prediction, case.perturbed),
    ):
        expected = exact(parameters)
        for name in case.variables:
            close(
                observed[name],
                expected[name],
                SIPOPT_VALUE_ABSOLUTE_ERROR,
                f"sIPOPT {stage} differs for {name}",
            )


def compare(
    candidate: Candidate, case: SensitivityCase, reference: dict[str, float]
) -> Comparison:
    """Check physical increments apart from nominal solve error for every result."""
    report = candidate.report
    if report is None:
        return Comparison(candidate.id, "missing", ["native result missing"])
    nominal, perturbed = exact(case.nominal), exact(case.perturbed)
    jacobian = {
        "x": {"a": 2.0 / 3.0, "b": 1.0 / 3.0},
        "y": {"a": 1.0 / 3.0, "b": -1.0 / 3.0},
        "z": {"a": 0.0, "b": 0.0},
    }
    try:
        require(report.complete, "native response incomplete")
        require(report.unit == case.unit, "native sensitivity unit differs")
        require(
            report.coordinates == "physical", "native sensitivity coordinates differ"
        )
        require(
            set(report.quantities) == set(case.variables),
            "native target population incomplete",
        )
        require(
            set(report.parameter_scales) == {"a", "b"},
            "native parameter scales incomplete",
        )
        require(
            all(
                math.isfinite(value) and value > 0
                for value in report.parameter_scales.values()
            ),
            "invalid native parameter scale",
        )
        for name in case.variables:
            quantity = report.quantities[name]
            require(
                set(quantity.derivative) == {"a", "b"},
                "native derivative population incomplete",
            )
            for parameter in ("a", "b"):
                close(
                    quantity.derivative[parameter],
                    jacobian[name][parameter],
                    DERIVATIVE_ABSOLUTE_ERROR,
                    f"physical derivative differs for {name}/{parameter}",
                )
            close(
                quantity.base,
                nominal[name],
                NATIVE_VALUE_ABSOLUTE_ERROR,
                f"nominal solution differs for {name} ({quantity.base!r} versus {nominal[name]!r}, "
                f"absolute allowance {NATIVE_VALUE_ABSOLUTE_ERROR:g})",
            )
            close(
                quantity.increment,
                perturbed[name] - nominal[name],
                INCREMENT_ABSOLUTE_ERROR,
                f"physical increment differs for {name}",
            )
            close(
                quantity.predicted,
                perturbed[name],
                NATIVE_VALUE_ABSOLUTE_ERROR,
                f"predicted solution differs for {name}",
            )
            # Each prediction has independently met its analytic physical allowance.
            close(
                quantity.predicted,
                reference[name],
                NATIVE_VALUE_ABSOLUTE_ERROR + SIPOPT_VALUE_ABSOLUTE_ERROR,
                f"native/sIPOPT difference for {name}",
            )
            require(
                quantity.predicted == quantity.base + quantity.increment,
                f"incoherent prediction for {name}",
            )
            physical_increment = sum(
                quantity.derivative[p] * (case.perturbed[p] - case.nominal[p])
                for p in ("a", "b")
            )
            close(
                quantity.increment,
                physical_increment,
                INCREMENT_ABSOLUTE_ERROR,
                f"incoherent increment for {name}",
            )
            require(
                math.isfinite(quantity.allowance) and quantity.allowance > 0,
                "invalid native engineering budget",
            )
            require(
                math.isfinite(quantity.coordinate_scale)
                and quantity.coordinate_scale > 0,
                "invalid native output scale",
            )
    except ValueError as error:
        return Comparison(candidate.id, "failed", [str(error)])
    return Comparison(candidate.id, "passed", [])


def main() -> None:
    """Execute independent science before consulting any candidate results."""
    require(len(sys.argv) == 3, "expected request and response paths")
    require(sys.flags.isolated == 1, "oracle must run with the interpreter's -I option")
    request = read_request(Path(sys.argv[1]), Request)
    admit_candidates(
        (candidate.id for candidate in request.candidates), len(request.candidates)
    )
    require(request.version == 1, "unsupported exchange version")
    validate_case(request.case)
    require(bool(request.candidates), "empty candidate population")
    ids = [candidate.id for candidate in request.candidates]
    require(
        all(ids) and len(set(ids)) == len(ids),
        "candidate identities must be nonempty and unique",
    )
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            LEGACY_PACKAGES,
            role="PSE_SENSITIVITY_REFERENCE_PYTHON",
            declared=request.pins,
        )
        nominal, reference = sipopt_prediction(request.case, environment)
        check_sipopt(request.case, nominal, reference)
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    response = Response(
        version=1,
        case_id=request.case.id,
        environment=environment.evidence,
        reference=reference,
        comparisons=[
            compare(candidate, request.case, reference)
            for candidate in request.candidates
        ],
    )
    write_response(
        Path(sys.argv[2]),
        response,
        len(request.candidates),
        response.environment,
        response.comparisons,
    )

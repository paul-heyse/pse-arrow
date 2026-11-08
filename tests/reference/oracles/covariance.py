# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated parmest oracle for explicit native weighted-fit case/results.

Preserves the original covariance parity experiment: known observation standard
deviations, SSE_weighted, parmest estimates and finite-difference inverse Fisher
information. No product package, generated contract or candidate fixture imports.
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
    from pyomo.core.base.var import ScalarVar


class LineCase(msgspec.Struct, forbid_unknown_fields=True):
    """The explicit original physical experiment, independent of candidate values."""

    id: str
    x: list[float]
    y: list[float]
    sigma: float
    parameters: list[str]
    response_unit: str
    parameter_units: list[str]
    covariance_convention: str


class Covariance(msgspec.Struct, forbid_unknown_fields=True):
    """Named physical row/column coordinates; values have parameter-unit products."""

    parameters: list[str]
    parameter_units: list[str]
    parameter_scales: list[float]
    coordinates: str
    layout: str
    convention: str
    values: list[float]


class CandidateReport(msgspec.Struct, forbid_unknown_fields=True):
    """Explicit native observations, without an assumed scientific verdict."""

    complete: bool
    estimates: dict[str, float]
    estimate_allowances: dict[str, float]
    response_unit: str
    covariance: Covariance


class Candidate(msgspec.Struct, forbid_unknown_fields=True):
    """An identified result or an explicitly missing result."""

    id: str
    report: CandidateReport | None


class Request(msgspec.Struct, forbid_unknown_fields=True):
    """One bounded exchange over the independently specified experiment."""

    version: int
    case: LineCase
    pins: dict[str, str]
    candidates: list[Candidate]


class Comparison(msgspec.Struct, forbid_unknown_fields=True):
    """One retained outcome in a possibly mixed population."""

    id: str
    status: str
    reasons: list[str]


class Reference(msgspec.Struct, forbid_unknown_fields=True):
    """Actual parmest results and the independent analytic information inverse."""

    estimates: dict[str, float]
    covariance: Covariance
    inverse_information: list[float]


# This dimensionless linear fit has a well-conditioned two-column information
# matrix. Allow one part in a million of a unit for the numerical fit, independent
# of the candidate's engineering budget and the larger measurement error (0.5).
ESTIMATE_ABSOLUTE_ERROR = 1e-6


class Response(msgspec.Struct, forbid_unknown_fields=True):
    """The independent reference and every input candidate's comparison outcome."""

    version: int
    case_id: str
    environment: dict[str, str]
    reference: Reference
    comparisons: list[Comparison]


def require(condition: bool, reason: str) -> None:
    """Checks remain active with Python optimization enabled."""
    if not condition:
        raise ValueError(reason)


def covariance_close(observed: float, expected: float) -> bool:
    """Original pytest.approx(rel=1e-6), including its default 1e-12 floor."""
    return (
        math.isfinite(observed)
        and math.isfinite(expected)
        and abs(observed - expected) <= max(1e-12, 1e-6 * abs(expected))
    )


def validate_case(case: LineCase) -> None:
    """Bind the comparison to the independently specified original experiment."""
    require(case.id == "weighted-line-known-sigma-v1", "unknown covariance case")
    require(case.x == [1.0, 2.0, 3.0, 4.0, 5.0], "original abscissae differ")
    require(case.y == [2.1, 3.9, 6.2, 7.8, 10.1], "original observations differ")
    require(case.sigma == 0.5, "original known standard deviation differs")
    require(case.parameters == ["a", "b"], "original parameter names/order differ")
    require(case.response_unit == "1", "original response is dimensionless Scalar")
    require(case.parameter_units == ["1", "1"], "original parameters are Scalar")
    require(
        case.covariance_convention == "inverse_information_known_sigma",
        "original covariance uses known sigma without residual-variance scaling",
    )


def parmest_reference(case: LineCase, environment: ReferenceEnvironment) -> Reference:
    """Execute the original independent parmest experiment without candidate inputs."""
    import pyomo.contrib.parmest.parmest as parmest
    import pyomo.environ as pyo
    from pyomo.contrib.parmest.experiment import Experiment

    class LineExperiment(Experiment):
        """One independently constructed, annotated measurement."""

        def __init__(self, x: float, y: float) -> None:
            self.x, self.y = x, y

        def get_labeled_model(self) -> pyo.ConcreteModel:
            model = pyo.ConcreteModel()
            a = cast("ScalarVar", pyo.Var(initialize=1.0))
            b = cast("ScalarVar", pyo.Var(initialize=1.0))
            model.a, model.b = a, b
            a.fix()
            b.fix()
            y = cast("ScalarVar", pyo.Var(initialize=self.y))
            model.y = y
            model.response = pyo.Constraint(expr=y == a + b * self.x)
            outputs = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.experiment_outputs = outputs
            outputs.update([(y, self.y)])
            parameters = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.unknown_parameters = parameters
            parameters.update((parameter, pyo.value(parameter)) for parameter in (a, b))
            error = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.measurement_error = error
            error.update([(y, case.sigma)])
            return model

    estimator = parmest.Estimator(
        [LineExperiment(x, y) for x, y in zip(case.x, case.y, strict=True)],
        obj_function="SSE_weighted",
    )
    # theta_est's extensive-form path selects the literal "ipopt" in
    # Pyomo 6.10.1. cov_est accepts an independent named solver for its re-solves.
    with environment.registration("ipopt", "parmest.fit", alias="ipopt"):
        _, theta = estimator.theta_est()
    environment.require_solves("parmest.fit", 1)
    alias = "pse_reference_parmest_covariance"
    with environment.registration("ipopt", "parmest.covariance", alias=alias):
        covariance = estimator.cov_est(
            method="finite_difference", solver=alias, step=1e-3
        )
    # One SSE solve and one central-difference baseline per experiment, then
    # one solve for each direction of each parameter perturbation.
    environment.require_solves(
        "parmest.covariance", len(case.x) * (2 + 2 * len(case.parameters))
    )
    values = [
        float(covariance.loc[p, q]) for p in case.parameters for q in case.parameters
    ]
    estimates = {name: float(theta[name]) for name in case.parameters}
    require(
        all(math.isfinite(value) for value in estimates.values()),
        "parmest estimates are nonfinite",
    )

    # The original model is linear: this information operator depends ONLY on
    # the experiment, never on fitted estimates or candidate allowances. Known
    # sigma means no sample residual variance or degrees-of-freedom multiplier.
    weight = 1.0 / case.sigma**2
    s00 = weight * len(case.x)
    s01 = weight * sum(case.x)
    s11 = weight * sum(x * x for x in case.x)
    determinant = s00 * s11 - s01 * s01
    require(determinant > 0.0, "reference information is singular")
    inverse = [
        s11 / determinant,
        -s01 / determinant,
        -s01 / determinant,
        s00 / determinant,
    ]
    rhs = [
        weight * sum(case.y),
        weight * sum(x * y for x, y in zip(case.x, case.y, strict=True)),
    ]
    for index, name in enumerate(case.parameters):
        exact = sum(inverse[2 * index + column] * rhs[column] for column in range(2))
        require(
            abs(estimates[name] - exact) <= ESTIMATE_ABSOLUTE_ERROR,
            f"parmest estimate differs from independent normal equations at {name}",
        )
    require(
        all(
            covariance_close(actual, expected)
            for actual, expected in zip(values, inverse, strict=True)
        ),
        "parmest covariance differs from independent inverse information",
    )
    return Reference(
        estimates=estimates,
        covariance=Covariance(
            parameters=case.parameters,
            parameter_units=case.parameter_units,
            parameter_scales=[1.0, 1.0],
            coordinates="physical",
            layout="row_major",
            convention=case.covariance_convention,
            values=values,
        ),
        inverse_information=inverse,
    )


def compare(candidate: Candidate, request: Request, reference: Reference) -> Comparison:
    """Compare every result separately; keep missing, failed and passed outcomes."""
    report = candidate.report
    if report is None:
        return Comparison(candidate.id, "missing", ["native result missing"])
    try:
        require(report.complete, "native fit incomplete")
        names = request.case.parameters
        require(set(report.estimates) == set(names), "native estimates are incomplete")
        require(
            report.response_unit == request.case.response_unit, "response unit differs"
        )
        for name in names:
            actual = report.estimates[name]
            require(
                math.isfinite(actual)
                and abs(actual - reference.estimates[name]) <= ESTIMATE_ABSOLUTE_ERROR,
                f"estimate differs at {name}",
            )
        require(
            set(report.estimate_allowances) == set(names)
            and all(
                math.isfinite(budget) and budget > 0.0
                for budget in report.estimate_allowances.values()
            ),
            "native engineering estimate budgets are incomplete or invalid",
        )
        covariance = report.covariance
        require(
            len(covariance.parameters) == len(names)
            and set(covariance.parameters) == set(names),
            "covariance parameters are incomplete or duplicated",
        )
        require(len(covariance.values) == len(names) ** 2, "covariance is incomplete")
        require(covariance.coordinates == "physical", "covariance coordinates differ")
        require(covariance.layout == "row_major", "covariance layout differs")
        require(
            covariance.convention == request.case.covariance_convention,
            "covariance convention differs",
        )
        require(
            covariance.parameter_units == ["1", "1"],
            "covariance parameter units differ",
        )
        require(
            covariance.parameter_scales == [1.0, 1.0],
            "covariance parameter scales differ",
        )
        order = {name: i for i, name in enumerate(covariance.parameters)}
        for i, p in enumerate(names):
            for j, q in enumerate(names):
                actual = covariance.values[order[p] * len(names) + order[q]]
                require(
                    covariance_close(
                        actual, reference.covariance.values[i * len(names) + j]
                    ),
                    f"covariance differs at ({p}, {q})",
                )
                require(
                    covariance_close(
                        actual, reference.inverse_information[i * len(names) + j]
                    ),
                    f"covariance differs from inverse information at ({p}, {q})",
                )
    except ValueError as error:
        return Comparison(candidate.id, "failed", [str(error)])
    return Comparison(candidate.id, "passed", [])


def main() -> None:
    """Publish one explicit response after real reference execution."""
    require(len(sys.argv) == 3, "expected request and response paths")
    require(sys.flags.isolated == 1, "oracle must run with the interpreter's -I option")
    request = read_request(Path(sys.argv[1]), Request)
    admit_candidates(
        (candidate.id for candidate in request.candidates), len(request.candidates)
    )
    require(request.version == 1, "unsupported exchange version")
    validate_case(request.case)
    require(bool(request.candidates), "candidate population is empty")
    ids = [candidate.id for candidate in request.candidates]
    require(
        all(ids) and len(set(ids)) == len(ids),
        "candidate identities must be unique and nonempty",
    )
    # Library chatter belongs in diagnostics, never the exchange document.
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            (*LEGACY_PACKAGES, "numpy", "pandas", "scipy"),
            role="PSE_COVARIANCE_REFERENCE_PYTHON",
            declared=request.pins,
        )
        reference = parmest_reference(request.case, environment)
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
            compare(candidate, request, reference) for candidate in request.candidates
        ],
    )
    write_response(
        Path(sys.argv[2]),
        response,
        len(request.candidates),
        response.environment,
        response.comparisons,
    )

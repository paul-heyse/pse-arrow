# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated parmest oracle for explicit native weighted-fit case/results.

Preserves the original covariance parity experiment: known observation standard
deviations, SSE_weighted, parmest estimates and finite-difference inverse Fisher
information. The oracle consumes passive source-owned case facts and emits only its own results.
"""

from __future__ import annotations

import contextlib
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, cast

from .reference_environment import ReferenceEnvironment
from .scalar_case import ScalarCase, require

if TYPE_CHECKING:
    from pyomo.core.base.var import ScalarVar


@dataclass(frozen=True)
class LineCase:
    """The explicit original physical experiment, independent of candidate values."""

    id: str
    x: list[float]
    y: list[float]
    sigma: float
    parameters: list[str]
    response_unit: str
    parameter_units: list[str]
    covariance_convention: str


@dataclass(frozen=True)
class Reference:
    """Actual parmest results and the independent analytic information inverse."""

    estimates: dict[str, float]
    covariance: list[float]
    inverse_information: list[float]


# Independent absolute allowance for agreement with the analytic result.
ESTIMATE_ABSOLUTE_ERROR = 1e-6

_PARAMETERS = ("a", "b")


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
        covariance=values,
        inverse_information=inverse,
    )


def _case(original: ScalarCase) -> LineCase:
    x_keys = [f"x:{index}" for index in range(5)]
    y_keys = [f"y:{index}" for index in range(5)]
    values = original.inputs([*x_keys, *y_keys, "sigma"])
    case = LineCase(
        id=original.contract.case_name,
        x=[values[key] for key in x_keys],
        y=[values[key] for key in y_keys],
        sigma=values["sigma"],
        parameters=list(_PARAMETERS),
        response_unit="1",
        parameter_units=["1", "1"],
        covariance_convention="inverse_information_known_sigma",
    )
    validate_case(case)
    return case


def main() -> None:
    """Validate the exact received case before performing independent solver work."""
    require(len(sys.argv) == 4, "expected descriptor, contract and response paths")
    require(sys.flags.isolated == 1, "oracle requires the interpreter's -I option")
    original = ScalarCase.read(Path(sys.argv[1]), Path(sys.argv[2]))
    case = _case(original)
    expected_keys = (
        [f"estimate:{name}" for name in _PARAMETERS]
        + [
            f"{matrix}:{row}:{column}"
            for matrix in ("covariance", "inverse_information")
            for row in _PARAMETERS
            for column in _PARAMETERS
        ]
    )
    original.outputs(expected_keys)
    with contextlib.redirect_stdout(sys.stderr):
        environment = ReferenceEnvironment.from_process(
            ("idaes-pse", "pyomo", "protobuf", "numpy", "pandas", "pint", "scipy"),
            role="PSE_COVARIANCE_REFERENCE_PYTHON",
        )
        reference = parmest_reference(case, environment)
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    original.publish(
        Path(sys.argv[3]),
        [reference.estimates[name] for name in _PARAMETERS]
        + reference.covariance
        + reference.inverse_information,
    )
    sys.stderr.write(
        "covariance_reference_environment "
        + json.dumps(environment.evidence, sort_keys=True)
        + "\n"
    )

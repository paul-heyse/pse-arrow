# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated parmest oracle for explicit native weighted-fit case/results.

Preserves the original covariance parity experiment: known observation standard
deviations, SSE_weighted, parmest estimates and finite-difference inverse Fisher
information. No product package, generated contract or candidate fixture imports.
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
    """One bounded exchange with its original frozen estimate error budgets."""

    version: int
    case: LineCase
    pins: dict[str, str]
    frozen_estimate_allowances: dict[str, float]
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
    return math.isfinite(observed) and math.isfinite(expected) and abs(
        observed - expected
    ) <= max(1e-12, 1e-6 * abs(expected))


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


def environment(pins: dict[str, str]) -> dict[str, str]:
    """Reference absence or wrong declared pins fails the whole exchange."""
    require((3, 11) <= sys.version_info[:2] < (3, 15), "reference requires Python 3.11-3.14")
    require(set(pins) == {"idaes-pse", "pyomo"}, "missing or unknown reference pins")
    observed = {name: importlib.metadata.version(name) for name in pins}
    require(observed == pins, f"reference pin mismatch: expected {pins}, observed {observed}")
    executable = shutil.which("ipopt")
    require(executable is not None, "ipopt is absent; reference execution never skips")
    if executable is None:
        raise ValueError("unreachable missing ipopt")
    probe = subprocess.run(  # noqa: S603 - selected reference solver, fixed version probe
        [executable, "-v"], capture_output=True, text=True, timeout=30, check=False
    )
    match = re.search(r"\bIpopt\s+(\d+(?:\.\d+)*)", probe.stdout + probe.stderr)
    require(probe.returncode == 0 and match is not None, "ipopt version probe failed")
    if match is None:
        raise ValueError("unreachable missing ipopt version")
    observed["ipopt"] = match[1]
    observed["python"] = ".".join(str(part) for part in sys.version_info[:3])
    return observed


def parmest_reference(case: LineCase) -> Reference:
    """Execute the original independent parmest experiment without candidate inputs."""
    import idaes
    import pyomo.contrib.parmest.parmest as parmest
    import pyomo.environ as pyo
    from idaes import config
    from pyomo.contrib.parmest.experiment import Experiment

    # Keep the prepared external solver prefix selected by the caller.
    idaes.cfg.use_idaes_solvers = False
    config.reconfig(idaes.cfg)
    require(pyo.SolverFactory("ipopt").available(False), "Pyomo cannot see ipopt")

    class LineExperiment(Experiment):  # type: ignore[misc]
        """One independently constructed, annotated measurement."""

        def __init__(self, x: float, y: float) -> None:
            self.x, self.y = x, y

        def get_labeled_model(self) -> pyo.ConcreteModel:
            model = pyo.ConcreteModel()
            model.a = pyo.Var(initialize=1.0)
            model.b = pyo.Var(initialize=1.0)
            model.a.fix()
            model.b.fix()
            model.y = pyo.Var(initialize=self.y)
            model.response = pyo.Constraint(expr=model.y == model.a + model.b * self.x)
            model.experiment_outputs = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.experiment_outputs.update([(model.y, self.y)])
            model.unknown_parameters = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.unknown_parameters.update((parameter, pyo.value(parameter)) for parameter in (model.a, model.b))
            model.measurement_error = pyo.Suffix(direction=pyo.Suffix.LOCAL)
            model.measurement_error.update([(model.y, case.sigma)])
            return model

    estimator = parmest.Estimator(
        [LineExperiment(x, y) for x, y in zip(case.x, case.y, strict=True)],
        obj_function="SSE_weighted",
    )
    _, theta = estimator.theta_est()
    covariance = estimator.cov_est(method="finite_difference")
    values = [float(covariance.loc[p, q]) for p in case.parameters for q in case.parameters]
    estimates = {name: float(theta[name]) for name in case.parameters}
    require(all(math.isfinite(value) for value in estimates.values()), "parmest estimates are nonfinite")

    # The original model is linear: this information operator depends ONLY on
    # the experiment, never on fitted estimates or candidate allowances. Known
    # sigma means no sample residual variance or degrees-of-freedom multiplier.
    weight = 1.0 / case.sigma**2
    s00 = weight * len(case.x)
    s01 = weight * sum(case.x)
    s11 = weight * sum(x * x for x in case.x)
    determinant = s00 * s11 - s01 * s01
    require(determinant > 0.0, "reference information is singular")
    inverse = [s11 / determinant, -s01 / determinant, -s01 / determinant, s00 / determinant]
    require(
        all(covariance_close(actual, expected) for actual, expected in zip(values, inverse, strict=True)),
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
            report.estimate_allowances == request.frozen_estimate_allowances,
            "estimate allowances differ from frozen native budgets",
        )
        require(report.response_unit == request.case.response_unit, "response unit differs")
        for name in names:
            actual = report.estimates[name]
            require(
                math.isfinite(actual)
                and abs(actual - reference.estimates[name]) <= request.frozen_estimate_allowances[name],
                f"estimate differs at {name}",
            )
        covariance = report.covariance
        require(
            len(covariance.parameters) == len(names) and set(covariance.parameters) == set(names),
            "covariance parameters are incomplete or duplicated",
        )
        require(len(covariance.values) == len(names)**2, "covariance is incomplete")
        require(covariance.coordinates == "physical", "covariance coordinates differ")
        require(covariance.layout == "row_major", "covariance layout differs")
        require(
            covariance.convention == request.case.covariance_convention,
            "covariance convention differs",
        )
        require(covariance.parameter_units == ["1", "1"], "covariance parameter units differ")
        require(covariance.parameter_scales == [1.0, 1.0], "covariance parameter scales differ")
        order = {name: i for i, name in enumerate(covariance.parameters)}
        for i, p in enumerate(names):
            for j, q in enumerate(names):
                actual = covariance.values[order[p] * len(names) + order[q]]
                require(
                    covariance_close(actual, reference.covariance.values[i * len(names) + j]),
                    f"covariance differs at ({p}, {q})",
                )
                require(
                    covariance_close(actual, reference.inverse_information[i * len(names) + j]),
                    f"covariance differs from inverse information at ({p}, {q})",
                )
    except ValueError as error:
        return Comparison(candidate.id, "failed", [str(error)])
    return Comparison(candidate.id, "passed", [])


def main() -> None:
    """Publish one explicit response after real reference execution."""
    require(len(sys.argv) == 3, "expected request and response paths")
    require(sys.flags.isolated == 1, "oracle must run with the interpreter's -I option")
    request = msgspec.json.decode(Path(sys.argv[1]).read_bytes(), type=Request)
    require(request.version == 1, "unsupported exchange version")
    validate_case(request.case)
    require(bool(request.candidates), "candidate population is empty")
    ids = [candidate.id for candidate in request.candidates]
    require(all(ids) and len(set(ids)) == len(ids), "candidate identities must be unique and nonempty")
    require(
        set(request.frozen_estimate_allowances) == set(request.case.parameters)
        and all(math.isfinite(value) and value > 0.0 for value in request.frozen_estimate_allowances.values()),
        "missing or invalid frozen native estimate budgets",
    )
    # Library chatter belongs in diagnostics, never the exchange document.
    with contextlib.redirect_stdout(sys.stderr):
        observed = environment(request.pins)
        reference = parmest_reference(request.case)
    require(
        not any(name == "pse" or name.startswith("pse.") for name in sys.modules),
        "reference execution imported the candidate product",
    )
    response = Response(
        version=1,
        case_id=request.case.id,
        environment=observed,
        reference=reference,
        comparisons=[compare(candidate, request, reference) for candidate in request.candidates],
    )
    Path(sys.argv[2]).write_bytes(msgspec.json.encode(response))


if __name__ == "__main__":
    main()

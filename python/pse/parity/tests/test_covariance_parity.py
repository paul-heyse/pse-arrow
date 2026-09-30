# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fit estimate and covariance against Pyomo's parmest (ADR-0118, blueprint §19.4).

Both sides fit a weighted linear regression ``y = a + b x`` at five points with a
declared standard deviation of 0.5 on every observation. parmest estimates with the
``SSE_weighted`` objective and takes the covariance from its finite-difference Fisher
information; pse fits the same observations and publishes the covariance of its
estimate. For a model linear in its parameters both equal the weighted least-squares
covariance ``(JᵀWJ)⁻¹``, so the estimates and every covariance entry agree to solver
accuracy.
"""

import pyomo.contrib.parmest.parmest as parmest
import pyomo.environ as pyo
import pytest
from pyomo.contrib.parmest.experiment import Experiment

import pse
from pse import modeling as w
from pse.contracts import authored as a
from pse.contracts.enums import ModelingAnalysisRoute, NativeSolveIntent
from pse.contracts.identities import DeclarationId, FitId, InstanceId
from pse.contracts.values import SemanticId

from . import support

X = (1.0, 2.0, 3.0, 4.0, 5.0)
Y = (2.1, 3.9, 6.2, 7.8, 10.1)
SIGMA = 0.5
LINE = (
    "package parity { def Line { param a: Scalar = 1; param b: Scalar = 1; "
    + " ".join(f"let y{k}: Scalar = a + b*{x};" for k, x in enumerate(X))
    + " } entity kind origin provenance {attribute title:Text;} "
    'entity origin experiment {title="weighted analytic line"} '
    "enum role {measured facets(measured)} "
    "entity kind reading {attribute value:Scalar?;attribute sigma:Scalar?;} "
    + " ".join(
        f'@id("{bytes([160 + k]).hex() * 16}") entity reading measured{k} '
        f"provenance(experiment,role.measured) {{value={y},sigma={SIGMA}}}"
        for k, y in enumerate(Y)
    )
    + " }"
)


class LineExperiment(Experiment):  # type: ignore[misc]
    """One observation of the line, labelled for parmest."""

    def __init__(self, x: float, y: float) -> None:
        self.x, self.y = x, y

    def get_labeled_model(self) -> pyo.ConcreteModel:
        m = pyo.ConcreteModel()
        m.a = pyo.Var(initialize=1.0)
        m.b = pyo.Var(initialize=1.0)
        m.a.fix()
        m.b.fix()
        m.y = pyo.Var(initialize=self.y)
        m.response = pyo.Constraint(expr=m.y == m.a + m.b * self.x)
        m.experiment_outputs = pyo.Suffix(direction=pyo.Suffix.LOCAL)
        m.experiment_outputs.update([(m.y, self.y)])
        m.unknown_parameters = pyo.Suffix(direction=pyo.Suffix.LOCAL)
        m.unknown_parameters.update((k, pyo.value(k)) for k in (m.a, m.b))
        m.measurement_error = pyo.Suffix(direction=pyo.Suffix.LOCAL)
        m.measurement_error.update([(m.y, SIGMA)])
        return m


def parmest_estimate() -> tuple[dict[str, float], dict[tuple[str, str], float]]:
    """The estimate and covariance parmest reports."""
    estimator = parmest.Estimator(
        [LineExperiment(x, y) for x, y in zip(X, Y, strict=True)],
        obj_function="SSE_weighted",
    )
    _, theta = estimator.theta_est()
    covariance = estimator.cov_est()
    names = ("a", "b")
    return (
        {name: float(theta[name]) for name in names},
        {(p, q): float(covariance.loc[p, q]) for p in names for q in names},
    )


def identity(n: int) -> SemanticId:
    return SemanticId(bytes([n]) * 16)


def pse_estimate(
    runtime: pse.Runtime,
) -> tuple[dict[str, float], dict[tuple[str, str], float]]:
    """The pse fitted parameters and published covariance."""
    authored, declarations = support.package(runtime, LINE)
    case = declarations["Line"]
    parameters = {"a": identity(151), "b": identity(152)}
    experiment = InstanceId(identity(159))
    fit = w.FitDeclaration(
        fit_id=FitId(identity(158)),
        parameters=tuple(
            a.AuthoredFitCasesFieldParametersItem(
                symbol_id=identity_,
                fixed=False,
                value=1.0,
                lower=-100.0,
                upper=100.0,
                scale=1.0,
            )
            for identity_ in parameters.values()
        ),
        experiments=(
            a.AuthoredFitCasesFieldExperimentsItem(
                experiment_id=experiment,
                case_id=case,
                route=ModelingAnalysisRoute.STEADY,
                bindings=tuple(
                    a.AuthoredFitCasesFieldExperimentsItemBindingsItem(
                        parameter_id=identity_, path=name
                    )
                    for name, identity_ in parameters.items()
                ),
            ),
        ),
        observations=tuple(
            a.AuthoredFitCasesFieldObservationsItem(
                observation_id=DeclarationId(identity(160 + k)),
                value_attribute="value",
                standard_deviation_attribute="sigma",
                experiment_id=experiment,
                output_path=f"y{k}",
                time=None,
                time_basis=None,
                time_unit_id=None,
                included=True,
                importance=1.0,
            )
            for k in range(len(Y))
        ),
    )
    authored = authored.with_fit_declarations((fit,))
    result = (
        authored.prepare_fit(
            FitId(identity(158)), pse.SolveSettings(intent=NativeSolveIntent.OPTIMIZE)
        )
        .start()
        .wait()
    )
    assert result.usable, result.diagnostics()
    estimates = {
        support.identity(row["parameter_id"]): support.real(row["value"])
        for row in support.rows(result.table("runtime.fit_parameters"))
    }
    (published,) = support.rows(result.table("runtime.parameter_covariances"))
    parameter_ids, values = published["parameters"], published["values"]
    assert isinstance(parameter_ids, list)
    assert isinstance(values, list)
    order = [support.identity(p) for p in parameter_ids]
    n = len(order)
    names = {identity_: name for name, identity_ in parameters.items()}
    return (
        {names[p]: estimates[p] for p in order},
        {
            (names[order[i]], names[order[j]]): support.real(values[i * n + j])
            for i in range(n)
            for j in range(n)
        },
    )


@pytest.mark.integration
@pytest.mark.parity
def test_fit_covariance_agrees_with_parmest(runtime: pse.Runtime) -> None:
    """The estimate within 1e-6 and every covariance entry within 1e-6 relative."""
    theta, covariance = parmest_estimate()
    estimate, published = pse_estimate(runtime)
    for name in ("a", "b"):
        assert estimate[name] == pytest.approx(theta[name], abs=1e-6), name
    for key, value in covariance.items():
        assert published[key] == pytest.approx(value, rel=1e-6), key

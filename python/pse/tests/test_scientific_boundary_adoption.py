# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public scientific consumers retain physical custody and actual fit authority."""

import gc
from collections.abc import Iterator

import msgspec
import pyarrow as pa
import pytest

import pse
from pse import codec, modeling
from pse.contracts import authored as a
from pse.contracts import documents as d
from pse.contracts.enums import (
    ModelingAnalysisRoute,
    NativeBackend,
    NativeSolveIntent,
    PresolvePolicyKind,
)
from pse.contracts.identities import DeclarationId, FitId, InstanceId
from pse.contracts.values import SemanticId
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.physical_input_fixture import member
from pse.tests.test_native_workflow import identity, package_documents, physical_context


@pytest.fixture
def boundary_runtime(inspection_settings: pse.EngineSettings) -> Iterator[pse.Runtime]:
    runtime = pse.Runtime(inspection_settings, ephemeral=True)
    try:
        yield runtime
    finally:
        runtime.clear_program_cache()
        runtime.close()


def regression(
    runtime: pse.Runtime, *, importance: float = 1.0
) -> tuple[pse.ModelingPackage, DeclarationId, FitId, tuple[SemanticId, ...]]:
    """Reuse the Rust regression oracle's X/Y/sigma and declared parameter scales."""
    observed = (3.1, 4.9, 7.2, 8.8)
    sigma = (0.5, 1.0, 0.5, 2.0)
    source = """package regression {
 def Root {
  param a:Scalar=1; param b:Scalar=1;
  var z:Scalar; eq contrast:z==a-b; annotation start z(0);
  let y1:Scalar=a+b; let y2:Scalar=a+2*b;
  let y3:Scalar=a+3*b; let y4:Scalar=a+4*b;
 }
 entity kind origin provenance {attribute title:Text;}
 entity origin experiment {title="weighted regression"}
 enum role {measured facets(measured)}
 entity kind reading {attribute value:Scalar?;attribute sigma:Scalar?;}
"""
    for index, (value, deviation) in enumerate(zip(observed, sigma, strict=True)):
        source += (
            f'@id("{identity(170 + index).to_hex()}") '
            f"entity reading point{index} provenance(experiment,role.measured) "
            f"{{value={value},sigma={deviation}}}\n"
        )
    source += "}"
    package = runtime.modeling_from_documents(
        [package_documents(source)], physical_context(runtime)
    )
    case = next(row.declaration_id for row in package.declarations() if row.name == "Root")
    ids = tuple(member(package, case, name) for name in ("a", "b", "z"))
    fit_id = FitId(identity(180))
    fit = modeling.FitDeclaration(
        fit_id=fit_id,
        parameters=tuple(
            a.AuthoredFitCasesFieldParametersItem(
                symbol_id=id_, fixed=False, value=1.0, lower=-100.0,
                upper=100.0, scale=scale,
            )
            for id_, scale in zip(ids[:2], (1.0, 2.0), strict=True)
        ),
        experiments=(a.AuthoredFitCasesFieldExperimentsItem(
            experiment_id=InstanceId(identity(181)), case_id=case,
            route=ModelingAnalysisRoute.STEADY,
            bindings=tuple(
                a.AuthoredFitCasesFieldExperimentsItemBindingsItem(parameter_id=id_, path=name)
                for id_, name in zip(ids[:2], ("a", "b"), strict=True)
            ),
        ),),
        observations=tuple(
            a.AuthoredFitCasesFieldObservationsItem(
                observation_id=DeclarationId(identity(170 + index)),
                value_attribute="value", standard_deviation_attribute="sigma",
                experiment_id=InstanceId(identity(181)), output_path=f"y{index + 1}",
                time=None, time_basis=None, time_unit_id=None, included=True,
                importance=importance if index == 2 else 1.0,
            )
            for index in range(4)
        ),
    )
    return package.with_fit_declarations((fit,)), case, fit_id, ids


def fit(package: pse.ModelingPackage, fit_id: FitId) -> pse.RunResult:
    return package.prepare_fit(
        fit_id,
        pse.PhysicalFitRequest(settings=d.FitPreparationDocumentForPhysicalSimulationRequest(
            optimizer="native_nlp",
            solver=pse.SolveSettings(intent=NativeSolveIntent.OPTIMIZE, backend=NativeBackend.IPOPT),
        )),
    ).start().wait()


def covariance_input(value: d.ParameterCovarianceOutput) -> d.ParameterCovariance:
    return codec.decode_json(codec.encode_json(value), d.ParameterCovariance)


def downstream(
    package: pse.ModelingPackage, case: DeclarationId,
    ids: tuple[SemanticId, ...], covariance: d.ParameterCovariance,
    *, source: pse.RunResult | None = None,
) -> pse.PreparedOperation:
    return package.prepare_solve(
        case,
        pse.SolveSettings(
            intent=NativeSolveIntent.ROOT, backend=NativeBackend.KINSOL,
            presolve=PresolvePolicyKind.OFF,
            sensitivity=d.SensitivityRequest(
                parameters=ids[:2],
                propagation=d.Propagation(covariance=covariance, outputs=(ids[2],)),
            ),
        ),
        covariance_source=source,
    )


def analytic_covariance() -> tuple[float, ...]:
    # Independent normal-equation inverse for X=(1,2,3,4), sigma=(.5,1,.5,2).
    weights = (4.0, 1.0, 4.0, 0.25)
    s00 = sum(weights)
    s01 = sum(w * x for w, x in zip(weights, (1, 2, 3, 4), strict=True))
    s11 = sum(w * x * x for w, x in zip(weights, (1, 2, 3, 4), strict=True))
    determinant = s00 * s11 - s01 * s01
    return (s11 / determinant, -s01 / determinant, -s01 / determinant, s00 / determinant)


def assert_contrast(result: pse.RunResult, covariance: tuple[float, ...]) -> dict[str, object]:
    assert result.usable
    (row,) = pa.table(result.table("runtime.propagated_covariances")).to_pylist()
    # J=(1,-1) gives Σ00-Σ01-Σ10+Σ11 without a fitted-value oracle.
    assert row["values"] == pytest.approx(
        [covariance[0] - covariance[1] - covariance[2] + covariance[3]], abs=1e-7
    )
    validity = pa.table(result.table("runtime.local_validity")).to_pylist()
    assert any(v["quantity"] == "propagated_covariance" and v["validity"]["certified"] for v in validity)
    return row


@pytest.mark.integration
def test_fit_covariance_public_binding_preserves_content_and_refuses_tampering(
    boundary_runtime: pse.Runtime,
) -> None:
    package, case, fit_id, ids = regression(boundary_runtime)
    fitted = fit(package, fit_id)
    assert fitted.usable
    covariance = fitted.parameter_covariance
    assert covariance is not None
    assert covariance.values == pytest.approx(analytic_covariance(), abs=1e-7)
    assert covariance.scales == (1.0, 2.0)
    assert isinstance(covariance.origin, d.CovarianceOriginOutputQualifiedFit)
    assert covariance.origin.run_id == fitted.run_id
    assert covariance.origin.record is None
    decoded = covariance_input(covariance)
    assert isinstance(decoded.origin, d.CovarianceOriginQualifiedFit)
    with pytest.raises(pse.InspectionError, match="decoded in-process covariance"):
        downstream(package, case, ids, decoded)
    prepared = downstream(package, case, ids, decoded, source=fitted)
    # The accepted preparation owns the recovered evidence after the producer dies.
    del fitted, covariance
    gc.collect()
    result = prepared.start().wait()
    row = assert_contrast(result, analytic_covariance())
    assert row["covariance_run_id"] == decoded.origin.run_id
    assert result.parameter_covariance is None
    fitted = fit(package, fit_id)
    changes = (
        {"values": (decoded.values[0] * 2, *decoded.values[1:])},
        {"parameters": tuple(reversed(decoded.parameters))},
        {"parameter_quantities": (identity(250), decoded.parameter_quantities[1])},
        {"parameter_units": (identity(250), decoded.parameter_units[1])},
        {"scales": (2.0, 2.0)},
    )
    # Use the same genuine run's payload for each content mutation.
    actual = fitted.parameter_covariance
    assert actual is not None
    actual_input = covariance_input(actual)
    for change in changes:
        with pytest.raises(pse.InspectionError, match="differs from the actual retained qualified fit"):
            downstream(package, case, ids, msgspec.structs.replace(actual_input, **change), source=fitted)


@pytest.mark.integration
def test_external_covariance_singular_psd_and_indefinite_public_assessments(
    boundary_runtime: pse.Runtime,
) -> None:
    package, case, fit_id, ids = regression(boundary_runtime)
    fitted = fit(package, fit_id)
    covariance = fitted.parameter_covariance
    assert covariance is not None
    external = msgspec.structs.replace(
        covariance_input(covariance),
        origin=d.CovarianceOriginExternal(label="declared singular", assumptions=("common error",)),
        values=(1.0, 1.0, 1.0, 1.0),
    )
    row = assert_contrast(downstream(package, case, ids, external).start().wait(), external.values)
    assert row["covariance_run_id"] is None
    assert row["covariance_origin"] == "external"
    assert row["covariance_label"] == "declared singular"
    assert row["covariance_assumptions"] == ["common error"]
    # Symmetric, finite and positive on the diagonal is insufficient: JΣJᵀ=-2.
    indefinite = downstream(
        package, case, ids,
        msgspec.structs.replace(external, values=(1.0, 2.0, 2.0, 1.0)),
    ).start().wait()
    # Propagation is an explicitly requested lazy projection of the joined solve.
    # Its mathematical refusal must be observed at the actual product boundary.
    with pytest.raises(pse.InspectionError, match="negative eigenvalue"):
        indefinite.table("runtime.propagated_covariances")


@pytest.mark.integration
def test_withheld_fit_covariance_cannot_fabricate_an_available_product(
    boundary_runtime: pse.Runtime,
) -> None:
    package, _case, fit_id, _ids = regression(boundary_runtime, importance=2.0)
    fitted = fit(package, fit_id)
    assert pa.table(fitted.table("runtime.parameter_covariances")).num_rows == 0
    with pytest.raises(pse.InspectionError, match="nonunit_importance"):
        _ = fitted.parameter_covariance


@pytest.mark.component
def test_canonical_fit_covariance_reopens_actual_content_and_refuses_missing_reference(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    package, case, fit_id, ids = regression(runtime)
    fitted = fit(package, fit_id)
    covariance = fitted.parameter_covariance
    assert covariance is not None
    run, attempt = fitted.canonical_run_key, fitted.canonical_attempt_key
    assert run is not None and attempt is not None
    del fitted, package
    runtime.clear_program_cache()
    runtime.close()
    reopened = canonical_substrate.runtime(inspection_settings)
    actual = reopened.fit_covariance(run, attempt)
    assert isinstance(actual.origin, d.CovarianceOriginOutputQualifiedFit)
    assert actual == covariance
    assert actual.values == pytest.approx(analytic_covariance(), abs=1e-7)
    package, case, _fit_id, ids = regression(reopened)
    decoded = covariance_input(actual)
    # Canonical references restore authority without an in-process fit handle.
    row = assert_contrast(downstream(package, case, ids, decoded).start().wait(), actual.values)
    assert row["covariance_run_id"] == actual.origin.run_id
    with pytest.raises(pse.InspectionError):
        reopened.fit_covariance(run, "missing-fit-attempt")
    for change in (
        {"values": (actual.values[0] * 2, *actual.values[1:])},
        {"parameters": tuple(reversed(actual.parameters))},
        {"parameter_units": (identity(250), actual.parameter_units[1])},
        {"scales": (2.0, 2.0)},
    ):
        with pytest.raises(pse.InspectionError, match="differs from the actual qualified fit"):
            downstream(package, case, ids, msgspec.structs.replace(decoded, **change))

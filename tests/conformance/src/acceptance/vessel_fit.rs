// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Seed data owns the vessel equations, observations and original physical checks.
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Termination};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::NumericalTarget,
        runtime::{fit_observations, fit_parameters, resolved_numerics, response_sensitivities},
    },
};
use pse_runtime::{CancelSource, workflow::RunReport};

#[tokio::test]
async fn authored_vessel_fitting_preserves_shared_parameters_checks_and_identifiability() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    // Read the actual admitted Power accounting requirement. Its authored
    // tolerance consumes numerical_policy.power; that constant is not marked as
    // an engineering-default rule. The fitted parameter is EnergyTransferRate;
    // a matching unit alone would not make its allowance authoritative here.
    let meter = seed_prepare(
        &package,
        SemanticId::parse_hex("f1b94720fef75b3bab2e29090ef0c315").unwrap(),
        profile(Backend::Ipopt, false),
        &CancelSource::new(),
    )
    .await
    .unwrap();
    let power = meter
        .model
        .model
        .compiled()
        .model
        .closures
        .values()
        .find(|closure| {
            closure.lineage.declaration.as_id()
                == SemanticId::parse_hex("8a830bf9c42043debdf9e7d34d429973").unwrap()
        })
        .expect("original meter's heat_rate Power accounting closure");
    assert_eq!(
        power.mode,
        pse_model::generated::enums::ModelingAccumulatorMode::Accounting
    );
    assert!(!power.observation_only);
    let pse_modeling::specialize::Value::Number { bits, quantity } = &power.tolerance else {
        panic!("typed engineering Power allowance")
    };
    let registry = &meter.model.case.compiled().quantities;
    let power_difference =
        pse_quantity::scheme::Scheme::Delta(Box::new(power.ty.quantity_scheme().unwrap().clone()))
            .resolve(registry, &Default::default())
            .unwrap();
    pse_quantity::admission::require_same_contract(power_difference, *quantity, registry).unwrap();
    let power_allowance = f64::from_bits(*bits);
    assert!(power_allowance.is_finite() && power_allowance > 0.);
    drop(meter);
    for kind in ["steady", "transient", "mixed", "unidentifiable"] {
        let (id, profile) = heat_fit(&package, kind).await;
        let prepared = package
            .prepare_fit(id, profile, compiler(), seed_limits(), &CancelSource::new())
            .await
            .unwrap();
        let numerics = prepared.numerics().clone();
        let declaration = prepared.declaration().clone();
        let heat = declaration.parameters[0].symbol_id;
        assert_eq!(heat.to_string(), "6188c9d1084e5883afdee80c9fee9145");
        let heat_policy = numerics
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Variable && target.id == heat)
            .unwrap();
        assert!(heat_policy.engineering.is_some());
        let objective_policy = numerics
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Objective)
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            matches!(
                report.solve.as_ref().unwrap().termination.category,
                Termination::Success | Termination::Acceptable
            ),
            "{kind}: {report:?}"
        );
        assert!(
            report.quality.as_ref().unwrap().feasible(),
            "{kind}: {report:?}"
        );
        let candidate = report.candidate.as_ref().unwrap()[0];
        near(candidate, 10., heat_policy.budget);
        assert!(
            report.objective.unwrap() <= objective_policy.budget,
            "{kind}: {report:?}"
        );
        let published_numerics =
            resolved_numerics::Row::rows(&result.table("runtime.resolved_numerics").unwrap())
                .unwrap();
        let published_heat = published_numerics
            .iter()
            .find(|target| {
                target.target_kind == NumericalTarget::Variable && target.target_id == heat
            })
            .unwrap();
        assert_eq!(published_heat.budget, heat_policy.budget);
        assert_eq!(
            published_heat.coordinate_scale,
            heat_policy.coordinate_scale
        );
        let observations =
            fit_observations::Row::rows(&result.table("runtime.fit_observations").unwrap())
                .unwrap();
        assert_eq!(observations.len(), declaration.observations.len());
        for (row, (published, authored)) in observations
            .iter()
            .zip(&declaration.observations)
            .enumerate()
        {
            assert_eq!(published.run_id, result.run_id);
            assert_eq!(published.observation_id, authored.observation_id.into());
            assert_eq!(published.experiment_id, authored.experiment_id);
            assert_eq!(published.included, authored.included);
            assert_eq!(published.prediction, report.predictions[row]);
        }
        let parameters =
            fit_parameters::Row::rows(&result.table("runtime.fit_parameters").unwrap()).unwrap();
        assert_eq!(parameters.len(), declaration.parameters.len());
        let published_heat_parameter = parameters
            .iter()
            .find(|parameter| parameter.parameter_id == heat)
            .unwrap();
        assert_eq!(published_heat_parameter.value, Some(candidate));
        assert_eq!(published_heat_parameter.unit_id, heat_policy.unit);
        let independent_objective = observations
            .iter()
            .filter(|row| row.included)
            .map(|row| row.objective_contribution.unwrap())
            .sum::<f64>();
        near(
            report.objective.unwrap(),
            independent_objective,
            f64::EPSILON * (1. + independent_objective.abs()),
        );
        assert_eq!(
            report.rank,
            Some(1),
            "zero inlet flow leaves inlet enthalpy unidentifiable: {report:?}"
        );
        assert_eq!(
            report.candidate.as_ref().unwrap().len(),
            if kind == "unidentifiable" { 2 } else { 1 }
        );
        let responses = response_sensitivities::Row::rows(
            &result.table("runtime.response_sensitivities").unwrap(),
        )
        .unwrap();
        let free = declaration
            .parameters
            .iter()
            .filter(|parameter| !parameter.fixed)
            .collect::<Vec<_>>();
        let included = declaration
            .observations
            .iter()
            .filter(|observation| observation.included)
            .count();
        assert_eq!(responses.len(), free.len() * included);
        let identities = responses
            .iter()
            .map(|response| (response.sample, response.parameter_id))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            identities.len(),
            responses.len(),
            "exact unique response cells"
        );
        for published in &responses {
            let sample = usize::try_from(published.sample).unwrap();
            let authored = &declaration.observations[sample];
            let observation = &observations[sample];
            let column = free
                .iter()
                .position(|parameter| parameter.symbol_id == published.parameter_id)
                .unwrap();
            let parameter = parameters
                .iter()
                .find(|parameter| parameter.parameter_id == published.parameter_id)
                .unwrap();
            assert!(authored.included);
            assert_eq!(published.run_id, result.run_id);
            assert_eq!(published.experiment_id, authored.experiment_id);
            assert_eq!(published.time, authored.time);
            assert_eq!(published.output_unit_id, observation.unit_id);
            assert_eq!(published.parameter_unit_id, parameter.unit_id);
            assert_eq!(parameter.run_id, result.run_id);
            assert_eq!(parameter.fixed, free[column].fixed);
            assert_eq!(
                published.value,
                report.responses.as_ref().unwrap()[(sample, column)]
            );
        }
        assert_eq!(
            heat_policy.coordinate_scale,
            declaration.parameters[0].scale
        );
        for (row, sensitivity) in report.responses.as_ref().unwrap().col(0).iter().enumerate() {
            let observation = &declaration.observations[row];
            let published = responses
                .iter()
                .find(|response| response.parameter_id == heat && response.sample == row as i64)
                .unwrap();
            assert_eq!(published.value, *sensitivity);
            // Current fixture premise: vessel-measurements.pse's five observation
            // IDs carry +/-standard(1) in W or J, selected by the YAML declarations
            // with standard_deviation_attribute=null. Sigma=1 is that authored
            // measurement interpretation, not a new numerical accuracy policy.
            // These are the same weighted, parameter-scaled coordinates production
            // uses for response rank. The 1 s closed balance and direct meter are
            // affine in heat at the reported candidate. The direct meter's
            // material Power response is resolved by the admitted heat allowance.
            // A 5 W action over this 1 s experiment changes Energy by only 5 J,
            // below the shared 100 J design resolution. Its retained response
            // cells establish identity/transport here; independent integrated
            // derivative agreement is exercised by the material p09 controls.
            let sigma = 1.;
            let scale = heat_policy.coordinate_scale * observation.importance.sqrt() / sigma;
            if let Some(time) = observation.time {
                assert_eq!(time, 1.);
            }
            let room = declaration.parameters[0].upper.unwrap() - candidate;
            let delta = (0.5 * heat_policy.coordinate_scale).min(room);
            assert!(
                delta > heat_policy.budget,
                "material response direction beyond physical resolution"
            );
            let direction = delta / heat_policy.coordinate_scale;
            let actual = *sensitivity * scale * direction;
            let prediction = |heat: f64| {
                observation
                    .time
                    .map_or(heat, |time| 1202.3168203660216 + heat * time)
            };
            let expected = (prediction(candidate + delta) - prediction(candidate))
                * observation.importance.sqrt()
                / sigma;
            let action_error = (actual - expected).abs();
            assert!(action_error.is_finite());
            if observation.time.is_none() {
                let allowance = power_allowance * observation.importance.sqrt() / sigma;
                assert!(expected.abs() > allowance);
                assert!(
                    action_error <= allowance,
                    "{kind}: empirical Power response at accepted heat={candidate}, sample={row}, action error={action_error}, allowance={allowance}"
                );
            }
        }
        assert!(
            report.checks_complete && report.checks.iter().all(|c| c.satisfied),
            "{kind}: {report:?}"
        );
        assert!(!report.checks.is_empty());
        assert_eq!(report.estimate_qualified(), kind != "unidentifiable");
        if report.estimate_qualified() {
            use pse_relations::{
                columnar::RelationRow, generated::runtime::fitted_parameter_cells,
            };
            let exported =
                fitted_parameter_cells::Row::rows(&result.export_fit_parameters().unwrap())
                    .unwrap();
            assert_eq!(exported.len(), 1);
            let cell = &exported[0];
            assert_eq!(cell.run_id, result.run_id);
            assert_eq!(cell.fit_id, id);
            assert_eq!(cell.parameter_id, heat);
            assert_eq!(cell.value, candidate);
            if kind == "steady" {
                // Publication is an explicit new admission with its fit/run/source receipt;
                // the fitted bank never mutates the measurements or the solved revision.
                let bank = format!(
                    "package fitted_bank {{identifier scheme run;identifier scheme fit;identifier scheme revision;entity kind receipt provenance {{attribute run_id:Id<run>;attribute fit_id:Id<fit>;attribute source_revision:Id<revision>;attribute fit_source:Id<revision>;}} enum role {{fitted facets(requires_fit)}} entity receipt qualified {{run_id=Id<run>(\"{}\"),fit_id=Id<fit>(\"{}\"),source_revision=Id<revision>(\"{}\"),fit_source=Id<revision>(\"{}\")}} entity kind heat_parameter {{attribute value:EnergyTransferRate;}} entity heat_parameter estimate provenance(qualified,role.fitted,lineage(fit qualified)) {{value={}{{W}}}} }}",
                    cell.run_id, cell.fit_id, cell.source_revision, cell.fit_source, cell.value
                );
                let declarations = |text: &str| {
                    pse_authoring::language::parse(
                        text,
                        SemanticId::NIL,
                        pse_authoring::language::IdentityPolicy::Named,
                        Default::default(),
                    )
                    .unwrap()
                };
                let physical = physical(&owner).await;
                let admitted = runtime(&owner)
                    .modeling_package(declarations(&bank), physical.clone())
                    .await
                    .unwrap();
                let knowledge = admitted
                    .knowledge(None, 128, 1 << 20, &owner.cancel)
                    .await
                    .unwrap();
                let cells = pse_relations::generated::runtime::modeling_knowledge::Row::rows(
                    knowledge.table(),
                )
                .unwrap();
                let fitted = cells.iter().find(|row| row.slot == "value").unwrap();
                assert_eq!(
                    fitted.value.last().unwrap().quantity_type_id,
                    Some(cell.quantity_type_id)
                );
                assert_eq!(fitted.value.last().unwrap().magnitude, Some(cell.value));
                assert_eq!(fitted.lineage.len(), 1);
                assert_eq!(
                    fitted.lineage[0].kind,
                    pse_model::generated::enums::ModelingLineageKind::Fit
                );
                let refused = bank.replace(",lineage(fit qualified)", "");
                assert!(
                    runtime(&owner)
                        .modeling_package(declarations(&refused), physical)
                        .await
                        .is_err()
                );
            }
        } else {
            assert!(result.export_fit_parameters().is_err());
        }
        assert!(
            result.usable(),
            "a checked prediction remains usable when a parameter is unidentifiable"
        );
        assert!(!package.declarations().await.unwrap().is_empty());
        assert!(result.table("authored.modeling_declarations").is_err());
        assert_eq!(package.fit_declarations().await.unwrap().fits.len(), 4);
        assert!(result.table("authored.fit_cases").is_err());
        assert!(result.table("authored.computation_models").is_err());
    }
}

/// ADR-0118 item 8: zero inlet flow leaves the inlet enthalpy unidentifiable, so the fit's
/// covariance is withheld as rank deficient, and the response directions report the
/// identifiable subspace: the heat alone.
#[tokio::test]
async fn unidentifiable_fit_withholds_covariance() {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{CovarianceApproximation, DerivedQuantity, WithheldReason},
            runtime::{local_validity, response_directions},
        },
    };
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let (id, profile) = heat_fit(&package, "unidentifiable").await;
    // Identifiability is a numerical-rank condition in declared-scale coordinates,
    // separate from physical engineering resolution or integration convergence.
    let direction_accuracy = profile.rank_tolerance;
    let prepared = package
        .prepare_fit(id, profile, compiler(), seed_limits(), &CancelSource::new())
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    let covariance = report.covariance.as_ref().unwrap();
    // A quasi-Newton fit's covariance is Gauss–Newton.
    assert_eq!(
        covariance.approximation,
        CovarianceApproximation::GaussNewton
    );
    assert!(
        matches!(
            covariance.values,
            Err(pse_runtime::workflow::FitWithheld::RankDeficient {
                rank: 1,
                parameters: 2
            })
        ),
        "{covariance:?}"
    );
    // The identifiable direction is the heat, in declared-scale coordinates.
    let directions = report.directions.as_ref().unwrap();
    near(directions[(0, 0)].abs(), 1., direction_accuracy);
    near(directions[(1, 0)], 0., direction_accuracy);
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert_eq!(validity[0].quantity, DerivedQuantity::ParameterCovariance);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(WithheldReason::RankDeficient)
    );
    assert_eq!(
        result
            .table("runtime.parameter_covariances")
            .unwrap()
            .batch()
            .num_rows(),
        0
    );
    let published =
        response_directions::Row::rows(&result.table("runtime.response_directions").unwrap())
            .unwrap();
    assert_eq!(published.len(), 4);
    assert_eq!(published.iter().filter(|r| r.identifiable).count(), 2);
    assert!(
        published
            .iter()
            .filter(|r| !r.identifiable)
            .all(|r| r.direction == 1)
    );
}

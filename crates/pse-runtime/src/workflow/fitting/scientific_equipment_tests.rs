// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Joint inference over the actual authored co-current exchanger and original balances.
use super::*;
use crate::workflow::{
    PreparedOperation, PreparedStudyOperation, RunHandle, RunReport, RunResult, Runtime,
    tests as fixture,
};
use pse_model::generated::enums::{DerivedQuantity, ModelingCheckKind, WithheldReason};
use pse_model::scalars::PositiveCount;
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{fit_observations, local_validity},
};

#[path = "../../../../../tests/fixtures/reference-source-fixture.rs"]
mod source_fixture;

struct Executed {
    runtime: Runtime,
    problem: Arc<FitProblem>,
    result: Arc<RunResult>,
}

async fn fit_exchanger(confounded: bool) -> Executed {
    let rows = source_fixture::rows(&source_fixture::reference_family_sources(
        "exchanger-inference",
    ));
    let namespace = rows
        .iter()
        .find(|row| row.parent_id.is_none() && row.name == "exchanger_inference")
        .unwrap()
        .declaration_id;
    let source = |name: &str| {
        rows.iter()
            .find(|row| row.parent_id == Some(namespace) && row.name == name)
            .unwrap()
            .declaration_id
    };
    let apparatus = source("Apparatus");
    let parameter = |name: &str| {
        rows.iter()
            .find(|row| row.parent_id == Some(apparatus) && row.name == name)
            .unwrap()
            .declaration_id
            .as_id()
    };
    let coefficient = parameter("coefficient");
    let area = parameter("surface_area");
    let fit_id = pse_ids::named_id(
        namespace.as_id(),
        if confounded {
            "fit.joint-coefficient-area"
        } else {
            "fit.known-area"
        },
    );
    let cases = [source("experiment_one"), source("experiment_two")];
    let experiment_ids =
        cases.map(|case| pse_ids::named_id(case.as_id(), "calibration-experiment"));
    let experiments = cases.into_iter().zip(experiment_ids).map(|(case, experiment)| serde_json::json!({
        "experiment_id":experiment, "case_id":case, "route":"steady",
        "bindings":[{"parameter_id":coefficient,"path":"root.coefficient"},{"parameter_id":area,"path":"root.surface_area"}]
    })).collect::<Vec<_>>();
    let observations = [
        ("hot_one", 0, "hot"), ("cold_one", 0, "cold"),
        ("hot_two", 1, "hot"), ("cold_two", 1, "cold"),
    ].into_iter().map(|(record, experiment, side)| serde_json::json!({
        "observation_id":source(record), "experiment_id":experiment_ids[experiment],
        "output_path":format!("root.exchanger.{side}.outlet.T"),
        "time":null,"time_basis":null,"time_unit_id":null,
        "included":true,"importance":1.0,"value_attribute":"value","standard_deviation_attribute":"sigma"
    })).collect::<Vec<_>>();
    let mut declarations = FitDeclarations::default();
    declarations.fits.push(serde_json::from_value(serde_json::json!({
        "fit_id":fit_id,
        "parameters":[
            {"symbol_id":coefficient,"fixed":false,"value":7.0,"lower":1.0,"upper":30.0,"scale":10.0},
            {"symbol_id":area,"fixed":!confounded,"value":2.0,"lower":0.5,"upper":4.0,"scale":2.0}
        ],
        "experiments":experiments,"observations":observations
    })).unwrap());
    let runtime = fixture::runtime_with_workspace(128 << 20);
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap()
        .with_fit_declarations(&runtime.source_services(), declarations)
        .await
        .unwrap();
    let services = runtime.execution_services();
    let mut profile = regression::profile(HessianMode::GaussNewton, None);
    // Both independent experiments have their own admitted native evaluator.
    profile.experiment_workers = pse_model::scalar!(PositiveCount(2));
    let cancel = crate::CancelSource::new();
    let prepared = package
        .prepare_fit(
            services.preparation(),
            fit_id.into(),
            profile,
            pse_compiler::workspace::Profile::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(prepared.declaration().experiments.len(), 2);
    let problem = prepared.problem.clone();
    let rules = &problem.numerics.policy.engineering_rules;
    let heat_rule = rules
        .iter()
        .find(|rule| rule.rule_id.to_string() == "4b8c1108211445f180f66c602f10b982")
        .expect("original authored energy-transfer engineering rule survives joint preparation");
    assert_eq!(heat_rule.physical_allowance, Some(1.0));
    assert!(
        heat_rule
            .provenance
            .contains("shared engineering rule constant")
    );
    assert!(
        problem
            .numerics
            .targets
            .iter()
            .filter_map(|target| target.engineering.as_ref()?.rule_id)
            .all(|id| rules.iter().any(|rule| rule.rule_id == id))
    );
    assert_eq!(problem.measurements.len(), 4);
    let temperature = problem
        .quantities
        .quantity_types()
        .find(|quantity| quantity.name.as_deref() == Some("Temperature"))
        .unwrap()
        .id;
    for measurement in &problem.measurements {
        assert_eq!(measurement.port.quantity, temperature);
        assert_eq!(measurement.sigma, Some(0.1));
        assert_eq!(measurement.importance, 1.0);
        assert!(measurement.time.is_none() && measurement.sample_index.is_none());
        assert!(measurement.included && measurement.value.is_some());
    }
    let operation = PreparedOperation::Study(PreparedStudyOperation::Fit(Box::new(prepared)));
    let handle: RunHandle = operation.start(&services, &cancel).await.unwrap();
    let result = handle.wait().await.unwrap();
    Executed {
        runtime,
        problem,
        result,
    }
}

fn report(result: &RunResult) -> &FitReport {
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("actual fit report")
    };
    report
}

// Independent solution of the co-current distributed energy ODE. This helper
// does not call the production LMTD, exchanger evaluator or fitting responses.
fn independent_point(
    hot_capacity: f64,
    cold_capacity: f64,
    hot_inlet: f64,
    cold_inlet: f64,
    coefficient: f64,
    area: f64,
) -> (f64, [f64; 2], [f64; 2]) {
    let inverse_capacity = 1.0 / hot_capacity + 1.0 / cold_capacity;
    let asymptotic_heat = (hot_inlet - cold_inlet) / inverse_capacity;
    let decay = (-coefficient * area * inverse_capacity).exp();
    let heat = asymptotic_heat * (1.0 - decay);
    let coefficient_response = asymptotic_heat * area * inverse_capacity * decay;
    (
        heat,
        [
            hot_inlet - heat / hot_capacity,
            cold_inlet + heat / cold_capacity,
        ],
        [
            -coefficient_response / hot_capacity,
            coefficient_response / cold_capacity,
        ],
    )
}

fn assert_original_physics(executed: &Executed) {
    let fitted = report(&executed.result);
    assert!(
        fitted.checks_complete && fitted.validation_error.is_none(),
        "{fitted:?}"
    );
    assert!(
        !fitted.checks.is_empty() && fitted.checks.iter().all(|check| check.satisfied),
        "{:?}",
        fitted.checks
    );
    assert!(
        fitted.quality.as_ref().unwrap().feasible(),
        "{:?}",
        fitted.quality
    );
    let closure_count = |source: &str| {
        fitted
            .checks
            .iter()
            .filter(|check| {
                check.kind == ModelingCheckKind::Closure && check.source_id.to_string() == source
            })
            .count()
    };
    assert_eq!(
        closure_count("24a82c6a754355f6b519c96985fc6329"),
        8,
        "two components on both sides of both experiments"
    );
    assert_eq!(
        closure_count("1a8a8d57c60a54f48e30750a70aadb8f"),
        4,
        "original energy closure on both sides of both experiments"
    );
    let controls = [(60.0, 30.0, 400.0, 300.0), (90.0, 45.0, 370.0, 310.0)];
    for (index, (hot_capacity, cold_capacity, hot_inlet, cold_inlet)) in
        controls.into_iter().enumerate()
    {
        let (heat, outlets, _) = independent_point(
            hot_capacity,
            cold_capacity,
            hot_inlet,
            cold_inlet,
            10.0,
            2.0,
        );
        for (side, expected) in outlets.into_iter().enumerate() {
            let actual = fitted.predictions[index * 2 + side].unwrap();
            assert!(
                (actual - expected).abs() <= 0.1,
                "experiment {index} side {side}: {actual} versus ODE mean {expected}"
            );
        }
        let case = ["experiment_one.", "experiment_two."][index];
        let reported = fitted
            .reports
            .iter()
            .find(|value| value.label == "transferred heat" && value.path.starts_with(case))
            .unwrap();
        assert!(
            (reported.value - heat).abs() <= 1.0,
            "{}: {} versus independent heat {heat}",
            reported.path,
            reported.value
        );
    }
    let observations =
        fit_observations::Row::rows(&executed.result.table("runtime.fit_observations").unwrap())
            .unwrap();
    assert_eq!(observations.len(), 4);
    for (reading, binding) in observations
        .iter()
        .zip(&executed.problem.declaration.observations)
    {
        assert_eq!(reading.observation_id, binding.observation_id.as_id());
        assert_eq!(reading.experiment_id, binding.experiment_id);
        assert_eq!(reading.run_id, executed.result.run_id);
        assert!(
            reading.included
                && reading.prediction.is_some()
                && reading.standardized_residual.is_some()
        );
    }
}

async fn close(executed: Executed) {
    executed.runtime.close();
    executed.runtime.clear_program_cache();
    executed
        .runtime
        .shared
        .math()
        .shutdown_native_lanes()
        .await
        .unwrap();
}

#[tokio::test]
async fn scientific_equipment_joint_exchanger_fit_recovers_shared_coefficient_from_two_experiments()
{
    let executed = fit_exchanger(false).await;
    // Repeated source ownership is idempotent, while the same rule identity with
    // changed scientific/engineering meaning must refuse rather than overwrite.
    let source_policy = &executed.problem.numerics.policy;
    let experiment = executed.problem.declaration.experiments[0].experiment_id;
    let mut retained = source_policy.clone();
    modeling::retain_experiment_numerics(&mut retained, source_policy, experiment, &[])
        .unwrap();
    assert_eq!(retained.engineering_rules, source_policy.engineering_rules);
    let mut conflicting = source_policy.clone();
    conflicting.engineering_rules[0].physical_allowance = Some(123.0);
    assert!(
        modeling::retain_experiment_numerics(&mut retained, &conflicting, experiment, &[])
            .is_err()
    );
    assert_original_physics(&executed);
    let fitted = report(&executed.result);
    assert_eq!(fitted.rank, Some(1));
    assert!(fitted.estimate_qualified(), "{fitted:?}");
    let coefficient = fitted.candidate.as_ref().unwrap()[0];
    let parameter = executed.problem.declaration.parameters[0].symbol_id;
    let allowance = fixture::engineering_target(
        &executed.problem.numerics,
        NumericalTarget::Variable,
        parameter,
    )
    .budget;
    assert!(
        (coefficient - 10.0).abs() <= allowance,
        "{coefficient} versus generating U=10, physical allowance {allowance}"
    );
    // Absolute known sigma gives local inverse information, without estimating
    // residual variance or asserting empirical confidence/interval coverage.
    let derivatives = [
        independent_point(60.0, 30.0, 400.0, 300.0, coefficient, 2.0).2,
        independent_point(90.0, 45.0, 370.0, 310.0, coefficient, 2.0).2,
    ];
    let information: f64 = derivatives
        .into_iter()
        .flatten()
        .map(|derivative| (derivative / 0.1).powi(2))
        .sum();
    let variance = fitted.covariance.as_ref().unwrap().values.as_ref().unwrap()[0];
    assert!(
        (variance * information - 1.0).abs() <= 1e-5,
        "local covariance {variance}, independent information {information}"
    );
    let published = executed.result.export_fit_parameters().unwrap();
    let cells =
        pse_relations::generated::runtime::fitted_parameter_cells::Row::rows(&published).unwrap();
    assert_eq!(
        cells.len(),
        2,
        "shared U and explicitly fixed area retain their declared identities"
    );
    close(executed).await;
}

#[tokio::test]
async fn scientific_equipment_joint_exchanger_fit_with_free_area_retains_only_ua_identifiability() {
    let executed = fit_exchanger(true).await;
    assert_original_physics(&executed);
    let fitted = report(&executed.result);
    assert_eq!(fitted.rank, Some(1), "{fitted:?}");
    let candidate = fitted.candidate.as_ref().unwrap();
    assert!(
        (candidate[0] * candidate[1] - 20.0).abs() <= 0.1,
        "identifiable UA: U={} A={}",
        candidate[0],
        candidate[1]
    );
    assert!(
        !fitted.estimate_qualified(),
        "a valid exchanger prediction does not identify U and area separately"
    );
    assert!(
        matches!(
            fitted.covariance.as_ref().unwrap().values,
            Err(FitWithheld::RankDeficient {
                rank: 1,
                parameters: 2
            })
        ),
        "{fitted:?}"
    );
    let validity =
        local_validity::Row::rows(&executed.result.table("runtime.local_validity").unwrap())
            .unwrap();
    let covariance = validity
        .iter()
        .find(|row| row.quantity == DerivedQuantity::ParameterCovariance)
        .unwrap();
    assert!(!covariance.validity.certified);
    assert_eq!(
        covariance.validity.reason,
        Some(WithheldReason::RankDeficient)
    );
    let completion = executed.result.completion.as_ref().unwrap();
    let permission = completion
        .products
        .iter()
        .find(|product| {
            product.scope == crate::workflow::completion::ProductScope::Task
                && product.quantity == DerivedQuantity::ParameterCovariance
        })
        .unwrap();
    assert!(
        !permission.permits_result(),
        "producer-owned covariance permission must preserve the rank refusal"
    );
    assert!(
        executed.result.export_fit_parameters().is_err(),
        "qualified parameter publication must refuse the confounded estimate"
    );
    close(executed).await;
}

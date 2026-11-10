// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered Ipopt fitting populations through the actual Study owner.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "authored Study controls require valid setup and exact result variants"
)]
use super::super::{FitPreparationDocument, PointOverlay, StudyPoint, StudyPointPolicy};
use super::*;
use pse_model::diagnostic::BoundaryDiagnostic;
use pse_model::study::{OccurrenceKey, StartPolicy};
fn point(operation: OperationRequest) -> StudyPoint {
    StudyPoint {
        operation,
        preparation: PreparationSettings {
            compiler: super::super::tests::compiler_profile(),
            limits: Limits::default(),
        },
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(7),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    }
}
fn original_domain(diagnostic: &BoundaryDiagnostic, source: SemanticId) -> bool {
    (diagnostic.rule == DiagnosticRule::MathDomain && diagnostic.sources.contains(&source))
        || diagnostic
            .causes
            .iter()
            .any(|cause| original_domain(cause, source))
}
fn original_local_result(report: &super::super::StudyReport, index: usize) -> &RunResult {
    let outcome = &report.outcomes[index];
    assert!(outcome.scientific.usable, "{outcome:?}");
    assert!(outcome.diagnostic.is_none(), "{outcome:?}");
    let occurrence = report.results[index].as_ref().unwrap();
    assert!(occurrence.stored_keys().is_none());
    let super::super::StudyOccurrenceResult::Ephemeral(result) = occurrence else {
        panic!("full local study must retain its original ephemeral result: {occurrence:?}");
    };
    assert!(matches!(
        result.durability(),
        super::super::RunDurability::Ephemeral
    ));
    assert!(result.canonical_run_key().is_none());
    assert!(result.canonical_attempt_key().is_none());
    assert!(result.usable(), "{:?}", result.assessments());
    result
}

#[tokio::test]
async fn ordered_study_fitting_population_keeps_original_points_failures_and_cancellation() {
    use super::super::{RunReport, StudyInputs, StudyOutputDemand, fitting::regression};
    use pse_backend_native::solve::{Backend, HessianMode, SolveIntent};
    use pse_model::generated::enums::StudyPointState;
    use pse_model::study::Dependency;
    let runtime = super::super::tests::runtime();
    let services = runtime.execution_services();
    let package = regression::population_package(&runtime).await;
    let declarations = package
        .declarations(&runtime.preparation_services())
        .await
        .unwrap();
    let failed_definition = declarations
        .iter()
        .find(|declaration| declaration.name == "Failed")
        .unwrap()
        .declaration_id;
    let failed_source = declarations
        .iter()
        .find(|declaration| {
            declaration.parent_id == Some(failed_definition) && declaration.name == "y"
        })
        .unwrap()
        .declaration_id
        .as_id();
    let request = |fit| {
        OperationRequest::Fit(FitOperation {
            fit: super::super::tests::id(fit).into(),
            settings: super::super::PhysicalFitRequest::new(FitPreparationDocument {
                optimizer: super::super::FitOptimizer::NativeNlp,
                solver: SolveSettings {
                    intent: SolveIntent::Optimize,
                    backend: Some(Backend::Ipopt),
                    // The analytic physical prediction check below requires
                    // tighter stopping than the default engineering KKT budget.
                    numerics: pse_model::numerics::NumericalPolicy {
                        kkt: pse_model::numerics::KktTolerances {
                            stationarity: 1e-10,
                            complementarity: 1e-10,
                        },
                        ..Default::default()
                    },
                    controls: pse_backend_native::solve::Controls {
                        hessian: HessianMode::GaussNewton,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                simulations: BTreeMap::new(),
                rank_tolerance: 1e-8,
                max_cells: 100_000,
                derivatives: FitDerivatives::Responses,
                experiment_workers: pse_model::scalar!(PositiveCount(1)),
                uncertainty: None,
            }),
        })
    };
    let mut first = point(request(32));
    first.policy.key = OccurrenceKey(90);
    let mut failed = point(request(35));
    failed.policy.key = OccurrenceKey(10);
    let mut last = point(request(32));
    last.policy.key = OccurrenceKey(70);
    last.policy.dependencies = vec![Dependency::Ordering(OccurrenceKey(10))];
    let definition = package
        .admit_study(
            &services,
            &StudyInputs::Explicit(vec![first, failed, last]),
            &StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let report = definition
        .execute(&services, 3, &CancelSource::new())
        .await
        .unwrap();
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        [OccurrenceKey(90), OccurrenceKey(10), OccurrenceKey(70)]
    );
    assert!(!report.outcomes[1].scientific.usable);
    let failed = report.outcomes[1].diagnostic.as_ref().unwrap();
    assert!(original_domain(failed, failed_source), "{failed:#?}");
    assert_eq!(report.outcomes[1].attempts.len(), 1);
    let failed_attempt = report.outcomes[1].attempts[0].diagnostic.as_ref().unwrap();
    assert!(
        original_domain(failed_attempt, failed_source),
        "{failed_attempt:#?}"
    );
    for index in [0, 2] {
        let result = original_local_result(&report, index);
        let RunReport::Fit(fit) = result.report().unwrap() else {
            panic!("original nonlinear fitting result");
        };
        let parameter = fit.candidate.as_ref().unwrap()[0];
        assert!(
            (parameter - 4.5_f64.sqrt()).abs() < 1e-6,
            "candidate={parameter}"
        );
        assert_eq!(fit.predictions.len(), 4);
        for prediction in &fit.predictions {
            let prediction = prediction.unwrap();
            assert!(
                (prediction - parameter * parameter).abs() < 1e-12,
                "candidate={parameter}, prediction={prediction}"
            );
            assert!(
                (prediction - 4.5).abs() < 1e-6,
                "candidate={parameter}, prediction={prediction}"
            );
        }
    }
    assert_ne!(
        original_local_result(&report, 0).run_id,
        original_local_result(&report, 2).run_id
    );
    let cancel = CancelSource::new();
    cancel.cancel();
    let canceled = definition.execute(&services, 3, &cancel).await.unwrap();
    assert_eq!(
        canceled
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        [OccurrenceKey(90), OccurrenceKey(10), OccurrenceKey(70)]
    );
    assert!(
        canceled
            .outcomes
            .iter()
            .all(|outcome| outcome.lifecycle == StudyPointState::Cancelled
                && outcome.attempts.is_empty()
                && !outcome.scientific.usable)
    );
    assert!(canceled.results.iter().all(Option::is_none));
}
